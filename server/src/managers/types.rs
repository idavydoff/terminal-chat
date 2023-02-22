use std::{
  str::FromStr, 
  fmt, 
  error::Error
};

/*
  Сигнал может содержать следующие хедеры
  USER:         USERNAME
  USER:         PASSWORD 
  USER:         KEY 
  SERVER:       AUTH_STATUS
  USER+SERVER:  WITH_MESSAGE
  USER+SERVER:  SYGNAL_TYPE
*/

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseSygnalDataError;
impl Error for ParseSygnalDataError {}
impl fmt::Display for ParseSygnalDataError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "invalid sygnal data")
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthConnectionError;
impl Error for AuthConnectionError {}
impl fmt::Display for AuthConnectionError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "auth connection error")
  }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IncomingMessageError;
impl Error for IncomingMessageError {}
impl fmt::Display for IncomingMessageError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "incoming message error")
  }
}


#[derive(Debug, Clone, Copy)]
pub enum SygnalType {
  ConnectionConsumer,
  ConnectionProducer,
  NewMessage,
}

impl FromStr for SygnalType {
  type Err = ParseSygnalDataError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "CONNECTION_CONSUMER" => Ok(SygnalType::ConnectionConsumer),
      "CONNECTION_PRODUCER" => Ok(SygnalType::ConnectionProducer),
      "NEW_MESSAGE" => Ok(SygnalType::NewMessage),
      _ => Err(ParseSygnalDataError)
    }
  }
}

impl ToString for SygnalType {
  fn to_string(&self) -> String {
    match self {
      SygnalType::ConnectionConsumer => "CONNECTION_CONSUMER".to_owned(),
      SygnalType::ConnectionProducer => "CONNECTION_PRODUCER".to_owned(),
      SygnalType::NewMessage => "NEW_MESSAGE".to_owned(),
    }
  }
}


#[derive(Debug, Clone, Copy)]
pub enum AuthStatus {
  ACCEPTED,
  DENIED
}

impl FromStr for AuthStatus {
  type Err = ParseSygnalDataError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "ACCEPTED" => Ok(AuthStatus::ACCEPTED),
      "DENIED" => Ok(AuthStatus::DENIED),
      _ => Err(ParseSygnalDataError)
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


pub enum SygnalHeader {
  Username(String),
  Password(String),
  Key(String),
  AuthStatus(AuthStatus),
  SygnalType(SygnalType),
  WithMessage,
  ServerMessage
}

impl FromStr for SygnalHeader {
  type Err = ParseSygnalDataError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let (header, value) = s.split_once(':').unwrap_or((s, s));

    match header {
      "USERNAME" => Ok(SygnalHeader::Username(value.trim().to_owned())),
      "PASSWORD" => Ok(SygnalHeader::Password(value.trim().to_owned())),
      "KEY" => Ok(SygnalHeader::Key(value.trim().to_owned())),
      "AUTH_STATUS" => {
        match AuthStatus::from_str(value.trim()) {
          Ok(v) => return Ok(SygnalHeader::AuthStatus(v)),
          Err(_) => Err(ParseSygnalDataError)
        }
      },
      "SYGNAL_TYPE" => {
        match SygnalType::from_str(value.trim()) {
          Ok(v) => return Ok(SygnalHeader::SygnalType(v)),
          Err(_) => Err(ParseSygnalDataError)
        }
      }
      "WITH_MESSAGE" => Ok(SygnalHeader::WithMessage),
      "SERVER_MESSAGE" => Ok(SygnalHeader::ServerMessage),
      _ => Err(ParseSygnalDataError)
    }
  }
}

impl ToString for SygnalHeader {
  fn to_string(&self) -> String {
    match self {
      SygnalHeader::Username(v) => format!("USERNAME: {v}\r\n"),
      SygnalHeader::Password(v) => format!("PASSWORD: {v}\r\n"),
      SygnalHeader::Key(v) => format!("KEY: {v}\r\n"),
      SygnalHeader::AuthStatus(v) => format!("AUTH_STATUS: {}\r\n", v.to_string()),
      SygnalHeader::SygnalType(v) => format!("SYGNAL_TYPE: {}\r\n", v.to_string()),
      SygnalHeader::WithMessage => "WITH_MESSAGE\r\n".to_owned(),
      SygnalHeader::ServerMessage => "SERVER_MESSAGE\r\n".to_owned()
    }
  }
}

#[derive(Debug, Clone)]
pub struct SygnalData {
  pub username: Option<String>,
  pub password: Option<String>,
  pub key: Option<String>,
  pub auth_status: Option<AuthStatus>,
  pub sygnal_type: Option<SygnalType>,
  pub with_message: bool,
  pub message: Option<String>,
  pub server_message: bool
}

impl SygnalData {
  pub fn new(headers: Vec<SygnalHeader>, message: Option<&str>) -> SygnalData {
    let mut data = SygnalData {
      username: None,
      password: None,
      key: None,
      auth_status: None,
      sygnal_type: None,
      with_message: false,
      message: None,
      server_message: false
    };

    for header in headers {
      match header {
        SygnalHeader::Username(v) => {
          data.username = Some(v);
        },
        SygnalHeader::Password(v) => {
          data.password = Some(v);
        },
        SygnalHeader::Key(v) => {
          data.key = Some(v);
        },
        SygnalHeader::AuthStatus(v) => {
          data.auth_status = Some(v);
        },
        SygnalHeader::SygnalType(v) => {
          data.sygnal_type = Some(v);
        },
        SygnalHeader::WithMessage => {
          data.with_message = true;
          data.message = Some(message.unwrap_or("").to_owned());
        },
        SygnalHeader::ServerMessage => {
          data.server_message = true;
        }
      }
    }

    data
  }
}

impl FromStr for SygnalData {
  type Err = ParseSygnalDataError;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut data = SygnalData { 
      username: None, 
      password: None, 
      key: None, 
      auth_status: None, 
      sygnal_type: None,
      with_message: false,
      message: None,
      server_message: false,
    };
    let splitted = s.split("\r\n");
    for string in splitted {
      let header = match SygnalHeader::from_str(string) {
        Ok(v) => v,
        Err(_) => continue
      };

      match header {
        SygnalHeader::Username(v) => {
          data.username = Some(v);
        },
        SygnalHeader::Password(v) => {
          data.password = Some(v);
        },
        SygnalHeader::Key(v) => {
          data.key = Some(v);
        },
        SygnalHeader::AuthStatus(v) => {
          data.auth_status = Some(v);
        },
        SygnalHeader::SygnalType(v) => {
          data.sygnal_type = Some(v);
        }
        SygnalHeader::WithMessage => {
          data.with_message = true;
        },
        SygnalHeader::ServerMessage => {
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
        return Err(ParseSygnalDataError);
      }
    }

    if let None = data.sygnal_type {
      return Err(ParseSygnalDataError)
    }

    Ok(data)
  }
}

impl ToString for SygnalData {
  fn to_string(&self) -> String {
    let mut res_str = String::new();

    if let Some(v) = &self.username {
      res_str.push_str(&SygnalHeader::Username(v.to_owned()).to_string());
    }
    if let Some(v) = &self.password {
      res_str.push_str(&SygnalHeader::Password(v.to_owned()).to_string());
    }
    if let Some(v) = &self.key {
      res_str.push_str(&SygnalHeader::Key(v.to_owned()).to_string());
    }
    if let Some(v) = &self.auth_status {
      res_str.push_str(&SygnalHeader::AuthStatus(v.clone()).to_string());
    }
    if let Some(v) = &self.sygnal_type {
      res_str.push_str(&SygnalHeader::SygnalType(v.clone()).to_string());
    }
    if self.server_message {
      res_str.push_str(&SygnalHeader::ServerMessage.to_string());
    }
    if self.with_message {
      if let Some(v) = &self.message {
        res_str.push_str(&SygnalHeader::WithMessage.to_string());
        res_str.push_str("\r\n");
        res_str.push_str(&v);
      }
    }
    res_str.push_str("\r\n\r\n");

    res_str
  }
}