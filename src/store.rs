use std::sync::RwLock;
use std::collections::HashMap;

use super::resp::{RespProtocol, StringValue};
use super::commands::Command;

pub struct Store {
    store: RwLock<HashMap<StringValue, StringValue>>,
}

impl Store {
    pub fn new() -> Self {
        Store {
            store: RwLock::new(HashMap::new()),
        }
    }
    pub fn run_command(&mut self, cmd: Command) -> RespProtocol {
        match cmd {
            Command::GET { key } => {
                let store = self.store.read().unwrap();
                let value = store.get(&key);
                RespProtocol::from(value)
            }
            Command::DEL { keys } => {
                let mut store = self.store.write().unwrap();
                let deleted: usize = (&keys)
                    .iter()
                    .map(|k| store.remove(k).map_or(0, |_| 1))
                    .sum();

                RespProtocol::from_integer(deleted as i64)
            }
            Command::SET { key, value } => {
                let mut store = self.store.write().unwrap();
                store.insert(key, value);

                RespProtocol::ok()
            }
        }
    }
}
