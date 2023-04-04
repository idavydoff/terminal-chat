use std::{net::TcpListener, thread, sync::Arc};
use anyhow::Result;
use parking_lot::Mutex;

use crate::{state::State, managers::Manager, messages_pool::MessagesPool};

pub struct Service;

impl Service {
  pub fn run(state: State) -> Result<()> {
    let listener = TcpListener::bind(format!("0.0.0.0:{}", state.get().settings.port))?;

    println!("Running!");

    let messages_pool = Arc::new(Mutex::new(MessagesPool::new()));

    for con in listener.incoming() {
      let cloned_state = state.clone();
      let cloned_messages_pool = messages_pool.clone();
      thread::spawn(move || -> Result<()> {
        Manager::new(con?, cloned_state, cloned_messages_pool)?;

        Ok(())
      });
    }

    Ok(())
  }
}