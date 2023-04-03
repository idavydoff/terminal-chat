use std::{collections::{HashMap, VecDeque}, iter};

#[derive(Debug, Clone)]
pub struct PoolMessage {
  pub id: String,
  pub username: String,
  pub message: String,
  pub from_server: bool,
}

impl PoolMessage {
  fn new() -> PoolMessage {
    PoolMessage {
      id: String::new(),
      username: String::new(),
      message: String::new(),
      from_server: false,
    }
  }
}

pub struct MessagesPool {
  pool: VecDeque<PoolMessage>,
  indexes: HashMap<String, u8>,
  length: u16,
}

impl MessagesPool {
  pub fn new() -> MessagesPool {
    let arr: VecDeque<PoolMessage> = iter::repeat_with(|| PoolMessage::new())
      .take(256)
      .collect();
    MessagesPool { 
      pool: arr, 
      indexes: HashMap::new(),
      length: 0
    }
  }

  pub fn push(&mut self, v: PoolMessage) {
    if self.length == 256 {
      self.pool.pop_front();
      self.pool.push_back(v);

      let mut new_indexes: HashMap<String, u8> = HashMap::new();
      for (index, message) in self.pool.iter().enumerate() {
        new_indexes.insert(message.id.clone(), index as u8);
      }
      self.indexes = new_indexes;
    }
    else {
      let index = self.length as u8;
      self.pool[index as usize] = v.clone();
      self.length += 1;
      self.indexes.insert(v.id.clone(), index);
    }
  }

  fn read_from(&self, id: &str) -> (Vec<PoolMessage>, Option<String>) {
    let found_index = self.indexes.get(id);
    match found_index {
      Some(v) => {
        let index: u16 = v.to_owned() as u16 + 1;
        let sliced_pool = &Vec::from(self.pool.clone())[index.into()..self.length.into()];
        let sliced_pool_last = {
          if sliced_pool.len() == 0 {
            None
          }
          else {
            Some(sliced_pool.last().unwrap().clone().id)
          }
        };
        return (sliced_pool.clone().into(), sliced_pool_last)
      },
      None => {
        let last_el = self.last();
        let index = match last_el {
          Some(v) => Some(v.id.clone()),
          None => None
        };
        let sliced_pool = &Vec::from(self.pool.clone())[..self.length.into()];
        return (sliced_pool.into(), index)
      }
    }
  }

  pub fn has_new(&self, id: &str) -> Option<(Vec<PoolMessage>, Option<String>)> {
    let last_el = self.last();
    match last_el {
      Some(_) => Some(self.read_from(id)),
      None => None,
    }
  }

  fn last(&self) -> Option<PoolMessage> {
    let last_index = {
      if self.length > 0 {
        self.length - 1
      } else {
        self.length
      }
    };

    let last_el = &self.pool[last_index.into()];
    if last_el.id == "".to_owned() {
      None
    } else {
      Some(last_el.to_owned())
    }
  }
}