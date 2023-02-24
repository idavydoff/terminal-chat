use std::{io::{BufReader, self, BufRead, Error, ErrorKind}, self, net::TcpStream};

use crate::managers::types::SignalHeader;

pub trait StreamReader {
  fn read_signal(&mut self, max_read_try: Option<u8>) -> io::Result<String>;
}

impl StreamReader for BufReader<TcpStream> {
  fn read_signal(&mut self, max_read_try: Option<u8>) -> io::Result<String> {
    let mut res_line = String::new();
    let mut headers_read = false;
    let mut fail_reads_count: u8 = 0;
    loop {
      let mut buf_line = String::new();
      match self.read_line(&mut buf_line) {
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
        if !res_line.contains(&SignalHeader::WithMessage.to_string()) || headers_read {
          break;
        }
        headers_read = true;
      }
    }
  
    Ok(res_line)
  }
}