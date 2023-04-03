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
    SignalType, 
    SignalData, 
    SignalHeader
  }
};

pub struct Service {
  pub connection: Connection,
  pub settings: Settings,
  pub state: State,
}

impl Service {
  pub fn run(settings: Settings, state: State) -> io::Result<()> {
    let connection = Connection::new(
      &settings.server_address.to_owned(), 
      &state.username
    )?;

    let mut instance = Service {
      connection,
      settings,
      state
    }.enable_print();

    instance.proccess_incoming_messages();
    instance.read_inputs();

    Ok(())
  }

  pub fn proccess_incoming_messages(&self) {
    let messages = self.state.messages.clone();
    let tx = self.state.chat_reload_sender.clone();
    let mut connection = self.connection.clone();
    thread::spawn(move || -> io::Result<()> {
      loop {
        let data_from_socket = match connection.read_signal() {
          Ok(v) => v,
          Err(_) => break
        };
        let signal = SignalData::from_str(&data_from_socket);
        let mut messages = messages.lock();
        if let Ok(s) = signal {
          if let Some(SignalType::NewMessage) = s.signal_type {
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
              messages.push(
                format!(
                  "<{}> {}", 
                  s.username.unwrap(), 
                  s.message.unwrap()
                )
              );
            }
          }
        }
        match tx.send(()) {
          Ok(_) => {},
          Err(_) => break
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
        for (index, m) in messages.lock().iter().enumerate() {
          if index == 0 {
            print!("\r\n{m}\r\n");
          }
          else {
            print!("{m}\r\n");
          }
        }
        let input = user_input.lock().clone();
        print!(
          "{}{}{} >{} {}", 
          termion::color::Bg(termion::color::White), 
          termion::color::Fg(termion::color::Black), 
          username, 
          termion::style::Reset,
          input
        );

        std::io::stdout().flush()?;
      }
      Ok(())
    });

    Service { 
      connection: self.connection,
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
    let mut stdin = io::stdin().keys();

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
                Err(_) => break, 
              };
              continue;
            }
            self.state.user_input.lock().clear();
            let signal = SignalData::new(
              vec![
                SignalHeader::SignalType(SignalType::NewMessage),
                SignalHeader::WithMessage,
                SignalHeader::Username(self.state.username.to_owned())
              ],
              Some(&ms)
            );
  
            self.connection.stream.write_all(signal.to_string().as_bytes()).unwrap();
          },
          termion::event::Key::Backspace => {
            self.state.user_input.lock().pop();
            match self.state.chat_reload_sender.send(()) {
              Ok(_) => {},
              Err(_) => break, 
            };
          }
          termion::event::Key::Char(k) => {
            println!("{k}");
            self.state.user_input.lock().push_str(&k.to_string());
            match self.state.chat_reload_sender.send(()) {
              Ok(_) => {},
              Err(_) => break, 
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