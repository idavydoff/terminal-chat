use std::{ 
  io::{
    Write, BufReader
  }, 
  time::Duration, 
  thread, 
  net::TcpStream, 
  sync::{
    Arc, 
    mpsc::{
      self, 
      Sender
    }
  }
};
use anyhow::Result;
use parking_lot::Mutex;

use crate::{managers::{data_manager::DataManager, types::SignalType}, messages_pool::MessagesPool, reader::StreamReader};

use super::{manager::Manager, data_manager::process_incoming_message};

fn process_signals(stream: TcpStream, messages_pool: Arc<Mutex<MessagesPool>>, sender: Sender<()>) -> Result<()> {
  let mut reader = BufReader::new(stream.try_clone()?);
  loop {
    let data_from_socket = match reader.read_signal(None) {
      Ok(s) => s,
      Err(_) => {
        break;
      }
    };

    match process_incoming_message(messages_pool.clone(), data_from_socket) {
      Ok(_) => (),
      Err(_) => println!("invalid message")
    };
  }

  sender.send(())?;

  Ok(())
}

pub trait StreamManager {
  fn process_connection(&mut self) -> Result<()>;
  fn process_disconnection(&mut self) -> Result<()>;
  fn send_data(&mut self, data: &str) -> Result<()>;
}

impl StreamManager for Manager {
  fn process_connection(&mut self) -> Result<()> {
    self.stream.set_read_timeout(Some(Duration::from_millis(1000)))?;
    println!("Connection established - {}", self.connected_peer_addr);

    let auth_data = match BufReader::new(
      self.stream.try_clone()?
    ).read_signal(Some(25)) {
      Ok(v) => v,
      Err(_) => {
        self.process_disconnection()?;
        return Ok(())
      }
    };

    let signal_type = match self.auth(auth_data.clone()) {
      Ok(v) => v,
      Err(_) => {
        self.deny_auth()?;
        self.process_disconnection()?;
        return Ok(())
      }
    };

    if let SignalType::Connection = signal_type {
      let cloned_stream = self.stream.try_clone()?;
      let cloned_messages_pool = self.messages_pool.clone();
      let (channel_sender, channel_receiver) = mpsc::channel::<()>();
      thread::spawn(move || -> Result<()> {
        process_signals(cloned_stream, cloned_messages_pool, channel_sender)?;

        Ok(())
      });
      
      self.process_messages_pool(channel_receiver)?;
    }

    self.process_disconnection()?;
    Ok(())
  }

  fn process_disconnection(&mut self) -> Result<()> {
    if self.connected_user_username.is_some() {
      self.remove_user(self.connected_user_username.clone().unwrap())?;
    }
    println!("Connection closed - {}", self.connected_peer_addr);
    Ok(())
  }

  fn send_data(&mut self, data: &str) -> Result<()> {
    self.stream.write(data.as_bytes())?;
    Ok(())
  }
}