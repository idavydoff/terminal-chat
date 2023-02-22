use std::{ 
  io::{
    Error, 
    ErrorKind,
    Write, self, BufRead
  }, time::Duration
};
use anyhow::Result;

use crate::managers::{data_manager::DataManager, types::SygnalType};

use super::{manager::Manager, types::SygnalHeader};

pub trait StreamManager {
  fn process_connection(&mut self) -> Result<()>;
  fn process_disconnection(&mut self) -> Result<()>;
  fn process_sygnals(&mut self) -> Result<()>;
  fn read_sygnal(&mut self, max_read_try: Option<u8>) -> io::Result<String> ;
  fn send_data(&mut self, data: &str) -> Result<()>;
}

impl StreamManager for Manager {
  fn process_connection(&mut self) -> Result<()> {
    self.stream.set_read_timeout(Some(Duration::from_millis(1000)))?;
    println!("Connection established - {}", self.connected_peer_addr);

    let auth_data = match self.read_sygnal(Some(25)) {
      Ok(v) => v,
      Err(_) => {
        self.process_disconnection()?;
        return Ok(())
      }
    };

    let sygnal_type = match self.auth(auth_data.clone()) {
      Ok(v) => v,
      Err(_) => {
        self.deny_auth()?;
        self.process_disconnection()?;
        return Ok(())
      }
    };

    if let SygnalType::ConnectionProducer = sygnal_type {
      self.process_sygnals()?;
    }
    else if let SygnalType::ConnectionConsumer = sygnal_type {
      self.process_messages_pool()?;
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

  fn process_sygnals(&mut self) -> Result<()> {
    loop {
      let data_from_socket = match self.read_sygnal(None) {
        Ok(s) => s,
        Err(_) => {
          break;
        }
      };

      match self.process_incoming_message(data_from_socket) {
        Ok(_) => (),
        Err(_) => println!("invalid message")
      };
    }

    Ok(())
  }

  fn read_sygnal(&mut self, max_read_try: Option<u8>) -> io::Result<String> {
    let mut res_line = String::new();
    let mut headers_read = false;
    let mut fail_reads_count: u8 = 0;
    loop {
      let mut buf_line = String::new();
      match self.reader.read_line(&mut buf_line) {
        Err(e) => {
          match e.kind() {
            io::ErrorKind::WouldBlock => {
              if let Some(max_fails) = max_read_try {
                fail_reads_count += 1;
                if fail_reads_count == max_fails {
                  return Err(Error::new(ErrorKind::ConnectionAborted, "boom boom"))
                }
              }
              continue;
            },
            _ => return Err(Error::new(ErrorKind::ConnectionAborted, "boom boom"))
          }
        },
        Ok(m) => {
          if m == 0 {
            return Err(Error::new(ErrorKind::BrokenPipe, "boom boom"))
          }
          m
        },
      };
      res_line.push_str(&buf_line);

      if res_line.ends_with("\r\n\r\n"){
        if !res_line.contains(&SygnalHeader::WithMessage.to_string()) || headers_read {
          break;
        }
        headers_read = true;
      }
    }

    Ok(res_line)
  }

  fn send_data(&mut self, data: &str) -> Result<()> {
    self.stream.write(data.as_bytes())?;
    Ok(())
  }
}