use std::{
  net::TcpStream, 
  io::BufReader, 
  sync::Arc
};
use parking_lot::Mutex;
use anyhow::Result;

use crate::{state::State, messages_pool::MessagesPool};
use super::stream_manager::StreamManager;

pub struct Manager {
  pub stream: TcpStream,
  pub reader: BufReader<TcpStream>,
  pub state: State,
  pub messages_pool: Arc<Mutex<MessagesPool>>,
  pub last_read_message_id: String,
  pub connected_user_username: Option<String>,
  pub connected_peer_addr: String
}

impl Manager {
  pub fn new(stream: TcpStream, state: State, messages_pool: Arc<Mutex<MessagesPool>>) -> Result<()> {
    let mut manager = Manager {
      stream: stream.try_clone()?,
      reader: BufReader::new(stream.try_clone()?),
      state,
      messages_pool,
      last_read_message_id: String::new(),
      connected_user_username: None,
      connected_peer_addr: stream.try_clone()?.peer_addr()?.to_string()
    };

    manager.process_connection()?;
    Ok(())
  }
}