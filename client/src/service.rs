use std::{
  thread, 
  io::{
    self, 
    Write
  },
  str::FromStr
};
use termion::{
  raw::IntoRawMode, 
  input::TermRead
};
use crate::{
  settings::Settings, 
  state::State, 
  connection::Connection, 
  types::{
    SygnalType, 
    SygnalData, 
    SygnalHeader
  }
};

pub struct Service {
  pub producer: Connection,
  pub settings: Settings,
  pub state: State,
}

impl Service {
  pub fn run(settings: Settings, state: State) -> io::Result<()> {
    let producer = Connection::new(
      &settings.server_address.to_owned(), 
      SygnalType::ConnectionProducer, 
      &state.username
    )?;

    let mut instance = Service {
      producer,
      settings,
      state
    }.enable_print();

    instance.proccess_incoming_messages();
    instance.read_inputs();

    Ok(())
  }

  pub fn proccess_incoming_messages(&self) {
    let messages = self.state.messages.clone();
    let username = self.state.username.clone();
    let tx = self.state.chat_reload_sender.clone();
    let server_address = self.settings.server_address.clone();
    thread::spawn(move || -> io::Result<()> {
      let mut consumer = Connection::new(
        &server_address, 
        SygnalType::ConnectionConsumer, 
        &username
      )?;
  
      loop {
        let data_from_socket = match consumer.read_signal(None) {
          Ok(v) => v,
          Err(_) => {
            break;
          }
        };
        let sygnal = SygnalData::from_str(&data_from_socket);
        let mut messages = messages.lock();
        if let Ok(s) = sygnal {
          if let Some(SygnalType::NewMessage) = s.sygnal_type {
            if s.server_message {
              messages.push(
                format!(
                  "{}{}{}{}",
                  termion::style::Faint,
                  termion::style::Bold,
                  s.message.unwrap(),
                  termion::style::Reset,
                )
              );
            }
            else {
              messages.push(format!("<{}> {}", s.username.unwrap(), s.message.unwrap()));
            }
          }
        }
        match tx.send(()) {
          Ok(_) => {},
          Err(_) => continue
        };
      }
  
      Ok(())
    });
  }

  pub fn enable_print(self) -> Service {
    let rx = self.state.chat_reload_receiver.unwrap();
    let messages = self.state.messages.clone();
    let user_input = self.state.user_input.clone();
    let username = self.state.username.clone();

    thread::spawn(move || -> io::Result<()> {
      loop {
        match rx.recv() {
          Ok(()) => {},
          Err(_) => break
        };
        print!("{}", termion::clear::All);
        for (index, m) in (&*messages.lock()).iter().enumerate() {
          if index == 0 {
            write!(
              std::io::stdout(), 
              "\r\n{m}\r\n"
            )?;
          }
          else {
            write!(
              std::io::stdout(), 
              "{m}\r\n"
            )?;
          }
        }
        let input = user_input.lock().clone();
        write!(
          std::io::stdout(), 
          "{}{}{} >{} {}", 
          termion::color::Bg(termion::color::White), 
          termion::color::Fg(termion::color::Black), 
          username, 
          termion::style::Reset,
          input
        )?;
        std::io::stdout().flush()?;
      }
      Ok(())
    });

    Service { 
      producer: self.producer,
      settings: self.settings, 
      state: State {
        username: self.state.username.clone(),
        chat_reload_receiver: None,
        chat_reload_sender: self.state.chat_reload_sender.clone(),
        user_input: self.state.user_input.clone(),
        messages: self.state.messages.clone(),
      }
    }
  }

  pub fn read_inputs(&mut self) {
    let stdout = io::stdout().into_raw_mode().unwrap(); // НЕЛЬЗЯ УБИРАТЬ
    let mut stdin = termion::async_stdin().keys();
  
    loop {
      let input = stdin.next();
  
      if let Some(Ok(key)) = input {
        match key {
          termion::event::Key::Ctrl('c') => break,
          termion::event::Key::Char('\n') => {
            let ms = self.state.user_input.lock().clone().trim().to_owned();
            if ms == "" {
              match self.state.chat_reload_sender.send(()) {
                Ok(_) => {},
                Err(_) => continue, 
              };
              continue;
            }
            self.state.user_input.lock().clear();
            let signal = SygnalData::new(
              vec![
                SygnalHeader::SygnalType(SygnalType::NewMessage),
                SygnalHeader::WithMessage,
                SygnalHeader::Username(self.state.username.to_owned())
              ],
              Some(&ms)
            );
  
            self.producer.stream.write_all(signal.to_string().as_bytes()).unwrap();
          },
          termion::event::Key::Backspace => {
            self.state.user_input.lock().pop();
            match self.state.chat_reload_sender.send(()) {
              Ok(_) => {},
              Err(_) => continue, 
            };
          }
          termion::event::Key::Char(k) => {
            println!("{k}");
            self.state.user_input.lock().push_str(&k.to_string());
            match self.state.chat_reload_sender.send(()) {
              Ok(_) => {},
              Err(_) => continue, 
            };
          },
          _ => {
            continue;
          }
        }
      }
    }
  }
}