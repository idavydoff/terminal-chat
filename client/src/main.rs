use std::io;

use service::Service;

use crate::{
  settings::Settings, 
  state::State
};

mod settings;
mod types;
mod connection;
mod state;
mod service;

fn main() -> io::Result<()> {
  let settings = Settings::new();
  let state = State::new()?;
  
  Service::run(settings, state)?;
  Ok(())
}