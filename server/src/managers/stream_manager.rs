use std::{ 
  io::{
    Write, BufReader
  }, 
  thread,
  sync::mpsc::{
    self, 
    Sender
  }
};
use anyhow::Result;

use crate::{managers::data_manager::DataManager, reader::StreamReader};

use super::manager::Manager;

pub trait StreamManager {
  fn process_connection(&mut self) -> Result<()>;
  fn process_disconnection(&mut self) -> Result<()>;
  fn send_data(&mut self, data: &str) -> Result<()>;
  fn process_signals(&mut self, sender: Sender<()>) -> Result<()>;
}

impl StreamManager for Manager {
  fn process_connection(&mut self) -> Result<()> {
    println!("Connection established - {}", self.connected_peer_addr);

    let auth_data = match BufReader::new(
      self.stream.try_clone()?
    ).read_signal() {
      Ok(v) => v,
      Err(_) => {
        self.process_disconnection()?;
        return Ok(())
      }
    };

    if self.auth(auth_data.clone()).is_err() {
      self.deny_auth()?;
      self.process_disconnection()?;
      return Ok(())
    }

    let (channel_sender, channel_receiver) = mpsc::channel::<()>();
    self.process_signals(channel_sender)?;
    
    self.process_messages_pool(channel_receiver)?;

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

  fn process_signals(&mut self, sender: Sender<()>) -> Result<()> {
    let cloned_stream = self.stream.try_clone()?;
    let cloned_messages_pool = self.messages_pool.clone();

    thread::spawn(move || -> Result<()> {
      let mut reader = BufReader::new(cloned_stream.try_clone()?);
      loop {
        let data_from_socket = match reader.read_signal() {
          Ok(s) => s,
          Err(_) => {
            break;
          }
        };

        match Self::process_incoming_message(cloned_messages_pool.clone(), data_from_socket) {
          Ok(_) => (),
          Err(_) => println!("invalid message")
        };
      }

      sender.send(())?;

      Ok(())
    });

    Ok(())
  }
}