use std::collections::HashMap;
use std::sync::RwLock;

use super::resp::{Cmd, Value};

type Item = Vec<u8>;

pub struct Store {
    store: RwLock<HashMap<Item, Item>>,
}

impl Store {
    pub fn new() -> Self {
        Store {
            store: RwLock::new(HashMap::new()),
        }
    }
    pub fn run_command<T: AsRef<[u8]>>(&self, cmd: Cmd<T>) -> Value {
        match cmd {
            Cmd::GET { key } => {
                let store = self.store.read().unwrap();
                let value = store.get(key.as_ref());
                let value = value.map_or(Value::Nil, |s| Value::Data(s.to_vec()));

                value
            }
            Cmd::DEL { keys } => {
                let mut store = self.store.write().unwrap();
                let deleted: usize = (&keys)
                    .iter()
                    .map(|k| store.remove(k.as_ref()).map_or(0, |_| 1))
                    .sum();

                Value::Int(deleted as i64)
            }
            Cmd::SET { key, value } => {
                let mut store = self.store.write().unwrap();
                store.insert(key.as_ref().to_vec(), value.as_ref().to_vec());
                Value::Okay
            }
        }
    }
}
