use anyhow::Result;

use service::Service;
use settings::Settings;
use state::State;

mod settings;
mod state;
mod service;
mod managers;
mod messages_pool;
mod reader;
mod types;

fn main() -> Result<()> {
  let settings = Settings::new();
  let state = State::new(settings);

  Service::run(state)?;
  
  Ok(())
}