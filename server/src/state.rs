use std::{
  sync::Arc, 
  collections::HashMap
};
use parking_lot::{Mutex, MutexGuard};

use crate::settings::Settings;

#[derive(Debug, Clone)]
pub struct UserData {
  pub producer_address: Option<String>,
  pub consumer_address: String,
}

#[derive(Debug, Clone)]
pub struct StateData {
  pub settings: Settings,
  pub users: HashMap<String, UserData>,
}

pub struct State(Arc<Mutex<StateData>>);

impl State {
  pub fn new(settings: Settings) -> State {
    State(
      Arc::new(Mutex::new(StateData { 
        settings, 
        users: HashMap::new()
      }))
    )
  }

  pub fn get(&self) -> MutexGuard<StateData> {
    self.0.lock()
  }
}

impl Clone for State {
  fn clone(&self) -> Self {
    State(Arc::clone(&self.0))
  }

  fn clone_from(&mut self, source: &Self) {
    *self = source.clone();
  }
}