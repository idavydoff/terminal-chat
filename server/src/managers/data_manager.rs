use std::thread;
use std::time::Duration;
use std::str::FromStr;
use anyhow::Result;
use uuid::Uuid;

use crate::messages_pool::PoolMessage;
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

pub trait DataManager {
  fn deny_auth(&mut self) -> Result<()>;
  fn auth(&mut self, sygnal: String) -> Result<SygnalType>;
  fn remove_user(&mut self, username: String) -> Result<()>;
  fn process_messages_pool(&mut self) -> Result<()>;
  fn process_incoming_message(&mut self, sygnal: String) -> Result<()>;
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
        SygnalType::ConnectionConsumer => {
          if let None = data.username {
            return Err(AuthConnectionError.into());
          }
          let mut state = self.state.get();
          let user = match state.users.get(&data.username.clone().unwrap()) {
            Some(v) => v.clone(),
            None => return Err(AuthConnectionError.into()),
          };
          if user.producer_address.is_some() {
            return Err(AuthConnectionError.into())
          }
          state.users.insert(data.username.clone().unwrap(), UserData { 
            producer_address: Some(self.stream.peer_addr()?.to_string()), 
            consumer_address: user.consumer_address.clone() 
          });
        },
        SygnalType::ConnectionProducer => {
          if let None = data.username {
            return Err(AuthConnectionError.into());
          }
          let mut state = self.state.get();
          if state.users.contains_key(&data.username.clone().unwrap()) {
            return Err(AuthConnectionError.into())
          }
          state.users.insert(data.username.clone().unwrap().to_owned(), UserData {
            consumer_address: self.stream.peer_addr()?.to_string(),
            producer_address: None,
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

  fn process_messages_pool(&mut self) -> Result<()> {
    let mut timer: i16 = 0;
    loop {
      if timer >= 2500 {
        if !self.state.get().users.contains_key(&self.connected_user_username.clone().unwrap()) {
          break;
        }
        timer = 0
      }
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
      timer += 10;
    }

    Ok(())
  }

  fn process_incoming_message(&mut self, sygnal: String) -> Result<()> {
    let data = SygnalData::from_str(&sygnal)?;

    if !data.with_message || data.username.is_none() {
      return Err(IncomingMessageError.into())
    }

    self.messages_pool.lock().push(PoolMessage {
      id: Uuid::new_v4().to_string(),
      username: data.username.clone().unwrap(),
      message: data.message.clone().unwrap().trim().to_owned(),
      from_server: false
    });

    Ok(())
  }
}