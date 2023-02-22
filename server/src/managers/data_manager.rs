use std::sync::Arc;
use std::sync::mpsc::Receiver;
use std::thread;
use std::time::Duration;
use std::str::FromStr;
use anyhow::Result;
use parking_lot::Mutex;
use uuid::Uuid;

use crate::messages_pool::{PoolMessage, MessagesPool};
use crate::state::UserData;

use super::manager::Manager;
use super::stream_manager::StreamManager;
use super::types::{
  AuthStatus, 
  SygnalData, 
  SygnalHeader, 
  AuthConnectionError,
  IncomingMessageError,
  SygnalType
};

pub fn process_incoming_message(messages_pool: Arc<Mutex<MessagesPool>>, sygnal: String) -> Result<()> {
  let data = SygnalData::from_str(&sygnal)?;

  if !data.with_message || data.username.is_none() {
    return Err(IncomingMessageError.into())
  }

  messages_pool.lock().push(PoolMessage {
    id: Uuid::new_v4().to_string(),
    username: data.username.clone().unwrap(),
    message: data.message.clone().unwrap().trim().to_owned(),
    from_server: false
  });

  Ok(())
}

pub trait DataManager {
  fn deny_auth(&mut self) -> Result<()>;
  fn auth(&mut self, sygnal: String) -> Result<SygnalType>;
  fn remove_user(&mut self, username: String) -> Result<()>;
  fn process_messages_pool(&mut self, receiver: Receiver<()>) -> Result<()>;
}

impl DataManager for Manager {
  fn deny_auth(&mut self) -> Result<()> {
    let response = SygnalData::new(
      vec![SygnalHeader::AuthStatus(AuthStatus::DENIED)],
      None
    );

    self.send_data(&response.to_string())?;
    Ok(())
  }

  fn auth(&mut self, sygnal: String) -> Result<SygnalType> {
    let data = SygnalData::from_str(&sygnal)?;

    match data.sygnal_type.unwrap() {
        SygnalType::Connection => {
          if let None = data.username {
            return Err(AuthConnectionError.into());
          }
          let mut state = self.state.get();
          if state.users.contains_key(&data.username.clone().unwrap()) {
            return Err(AuthConnectionError.into())
          }
          state.users.insert(data.username.clone().unwrap().to_owned(), UserData {
            address: self.stream.peer_addr()?.to_string(),
          });
          self.messages_pool.lock().push(PoolMessage {
            id: Uuid::new_v4().to_string(),
            username: String::new(),
            message: format!("{} joined the chat!", data.username.clone().unwrap()),
            from_server: true
          });
        }
        _ => return Err(AuthConnectionError.into()),
    }

    self.connected_user_username = Some(data.username.unwrap());

    let response = SygnalData::new(
      vec![SygnalHeader::AuthStatus(AuthStatus::ACCEPTED)],
      None
    );

    self.send_data(&response.to_string())?;
    Ok(data.sygnal_type.unwrap())
  }

  fn remove_user(&mut self, username: String) -> Result<()> {
    let mut state = self.state.get();

    if state.users.contains_key(&username) {
      state.users.remove(&username);
      self.messages_pool.lock().push(PoolMessage {
        id: Uuid::new_v4().to_string(),
        username: String::new(),
        message: format!("{username} left the chat!"),
        from_server: true
      });
    }
    Ok(())
  }

  fn process_messages_pool(&mut self, receiver: Receiver<()>) -> Result<()> {
    loop {
      if let Ok(()) = receiver.try_recv() {
        break;
      };

      let lock_ref = self.messages_pool.clone();
      let pool_lock = lock_ref.lock();

      let messages = pool_lock.has_new(&self.last_read_message_id);
      if let Some(v) = messages {
        if let Some(last) = v.1 {
          self.last_read_message_id = last;
        }
        for message in v.0 {
          let mut syg_vec = vec![
            SygnalHeader::SygnalType(SygnalType::NewMessage),
            SygnalHeader::Username(message.username.clone()),
            SygnalHeader::WithMessage
          ];
          if message.from_server {
            syg_vec.push(SygnalHeader::ServerMessage);
          }
          let response = SygnalData::new(syg_vec,Some(&message.message));
          self.send_data(&response.to_string())?;
        }
      }
      thread::sleep(Duration::from_millis(10));
    }

    Ok(())
  }
}