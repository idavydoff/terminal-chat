use std::{
  str::FromStr, 
  fmt, 
  error::Error
};

/*
  Сигнал может содержать следующие хедеры
  USER:         USERNAME
  SERVER:       AUTH_STATUS
  USER+SERVER:  WITH_MESSAGE
  USER+SERVER:  SIGNAL_TYPE
  SERVER:       SERVER_MESSAGE
*/

#[derive(Debug)]
pub struct ParseSignalDataError;
impl Error for ParseSignalDataError {}
impl fmt::Display for ParseSignalDataError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "invalid signal data")
  }
}

#[derive(Debug)]
pub struct AuthConnectionError;
impl Error for AuthConnectionError {}
impl fmt::Display for AuthConnectionError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "auth connection error")
  }
}

#[derive(Debug)]
pub struct IncomingMessageError;
impl Error for IncomingMessageError {}
impl fmt::Display for IncomingMessageError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "incoming message error")
  }
}


#[derive(Debug, Clone, Copy)]
pub enum SignalType {
  Connection,
  NewMessage,
}

impl FromStr for SignalType {
  type Err = ParseSignalDataError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "CONNECTION" => Ok(SignalType::Connection),
      "NEW_MESSAGE" => Ok(SignalType::NewMessage),
      _ => Err(ParseSignalDataError)
    }
  }
}

impl ToString for SignalType {
  fn to_string(&self) -> String {
    match self {
      SignalType::Connection => "CONNECTION".to_owned(),
      SignalType::NewMessage => "NEW_MESSAGE".to_owned(),
    }
  }
}


#[derive(Debug, Clone, Copy)]
pub enum AuthStatus {
  ACCEPTED,
  DENIED
}

impl FromStr for AuthStatus {
  type Err = ParseSignalDataError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "ACCEPTED" => Ok(AuthStatus::ACCEPTED),
      "DENIED" => Ok(AuthStatus::DENIED),
      _ => Err(ParseSignalDataError)
    }
  }
}

impl ToString for AuthStatus {
  fn to_string(&self) -> String {
    match self {
      AuthStatus::ACCEPTED => "ACCEPTED".to_owned(),
      AuthStatus::DENIED => "DENIED".to_owned()
    }
  }
}


pub enum SignalHeader {
  Username(String),
  AuthStatus(AuthStatus),
  SignalType(SignalType),
  WithMessage,
  ServerMessage
}

impl FromStr for SignalHeader {
  type Err = ParseSignalDataError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let (header, value) = s.split_once(':').unwrap_or((s, s));

    match header {
      "USERNAME" => Ok(SignalHeader::Username(value.trim().to_owned())),
      "AUTH_STATUS" => {
        match AuthStatus::from_str(value.trim()) {
          Ok(v) => return Ok(SignalHeader::AuthStatus(v)),
          Err(_) => Err(ParseSignalDataError)
        }
      },
      "SIGNAL_TYPE" => {
        match SignalType::from_str(value.trim()) {
          Ok(v) => return Ok(SignalHeader::SignalType(v)),
          Err(_) => Err(ParseSignalDataError)
        }
      }
      "WITH_MESSAGE" => Ok(SignalHeader::WithMessage),
      "SERVER_MESSAGE" => Ok(SignalHeader::ServerMessage),
      _ => Err(ParseSignalDataError)
    }
  }
}

impl ToString for SignalHeader {
  fn to_string(&self) -> String {
    match self {
      SignalHeader::Username(v) => format!("USERNAME: {v}\r\n"),
      SignalHeader::AuthStatus(v) => format!("AUTH_STATUS: {}\r\n", v.to_string()),
      SignalHeader::SignalType(v) => format!("SIGNAL_TYPE: {}\r\n", v.to_string()),
      SignalHeader::WithMessage => "WITH_MESSAGE\r\n".to_owned(),
      SignalHeader::ServerMessage => "SERVER_MESSAGE\r\n".to_owned()
    }
  }
}

#[derive(Debug, Clone)]
pub struct SignalData {
  pub username: Option<String>,
  pub password: Option<String>,
  pub key: Option<String>,
  pub auth_status: Option<AuthStatus>,
  pub signal_type: Option<SignalType>,
  pub with_message: bool,
  pub message: Option<String>,
  pub server_message: bool
}

impl SignalData {
  pub fn new(headers: Vec<SignalHeader>, message: Option<&str>) -> SignalData {
    let mut data = SignalData {
      username: None,
      password: None,
      key: None,
      auth_status: None,
      signal_type: None,
      with_message: false,
      message: None,
      server_message: false
    };

    for header in headers {
      match header {
        SignalHeader::Username(v) => {
          data.username = Some(v);
        },
        SignalHeader::AuthStatus(v) => {
          data.auth_status = Some(v);
        },
        SignalHeader::SignalType(v) => {
          data.signal_type = Some(v);
        },
        SignalHeader::WithMessage => {
          data.with_message = true;
          data.message = Some(message.unwrap_or("").to_owned());
        },
        SignalHeader::ServerMessage => {
          data.server_message = true;
        }
      }
    }

    data
  }
}

impl FromStr for SignalData {
  type Err = ParseSignalDataError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut data = SignalData { 
      username: None, 
      password: None, 
      key: None, 
      auth_status: None, 
      signal_type: None,
      with_message: false,
      message: None,
      server_message: false,
    };
    let splitted = s.split("\r\n");
    for string in splitted {
      let header = match SignalHeader::from_str(string) {
        Ok(v) => v,
        Err(_) => continue
      };

      match header {
        SignalHeader::Username(v) => {
          data.username = Some(v);
        },
        SignalHeader::AuthStatus(v) => {
          data.auth_status = Some(v);
        },
        SignalHeader::SignalType(v) => {
          data.signal_type = Some(v);
        }
        SignalHeader::WithMessage => {
          data.with_message = true;
        },
        SignalHeader::ServerMessage => {
          data.server_message = true;
        }
      }
    }

    if data.with_message {
      let splitted = s.split_once("\r\n\r\n");
      if let Some(v) = splitted {
        if v.1.ends_with("\r\n\r\n") {
          let string = v.1.to_owned();
          data.message = Some(string[..string.len() - 4].to_owned());
        }
        else {
          data.message = Some(v.1.to_owned());
        }
      }
      else {
        return Err(ParseSignalDataError);
      }
    }

    if let None = data.signal_type {
      return Err(ParseSignalDataError)
    }

    Ok(data)
  }
}

impl ToString for SignalData {
  fn to_string(&self) -> String {
    let mut res_str = String::new();

    if let Some(v) = &self.username {
      res_str.push_str(&SignalHeader::Username(v.to_owned()).to_string());
    }
    if let Some(v) = &self.auth_status {
      res_str.push_str(&SignalHeader::AuthStatus(v.clone()).to_string());
    }
    if let Some(v) = &self.signal_type {
      res_str.push_str(&SignalHeader::SignalType(v.clone()).to_string());
    }
    if self.server_message {
      res_str.push_str(&SignalHeader::ServerMessage.to_string());
    }
    if self.with_message {
      if let Some(v) = &self.message {
        res_str.push_str(&SignalHeader::WithMessage.to_string());
        res_str.push_str("\r\n");
        res_str.push_str(&v);
      }
    }
    res_str.push_str("\r\n\r\n");

    res_str
  }
}