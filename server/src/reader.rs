use std::{io::{BufReader, self, BufRead, Error, ErrorKind}, self, net::TcpStream};

use crate::types::SignalHeader;

pub trait StreamReader {
  fn read_signal(&mut self) -> io::Result<String>;
}

impl StreamReader for BufReader<TcpStream> {
  fn read_signal(&mut self) -> io::Result<String> {
    let mut res_line = String::new();
    let mut headers_read = false;
    loop {
      let mut buf_line = String::new();
      match self.read_line(&mut buf_line) {
        Err(_) => return Err(Error::new(ErrorKind::ConnectionAborted, "boom boom")),
        Ok(0) => return Err(Error::new(ErrorKind::BrokenPipe, "boom boom")),
        Ok(m) => m,
      };
      res_line.push_str(&buf_line);
  
      if res_line.ends_with("\r\n\r\n"){
        if !res_line.contains(&SignalHeader::WithMessage.to_string()) || headers_read {
          break;
        }
        headers_read = true;
      }
    }
  
    Ok(res_line)
  }
}