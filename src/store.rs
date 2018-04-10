use std::collections::HashMap;
use std::sync::RwLock;

use super::commands::Cmd;
use super::redis_value::{RedisValue, Value};

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
    pub fn run_command<T: AsRef<[u8]>>(&self, cmd: Cmd<Value<T>>) -> RedisValue {
        match cmd {
            Cmd::GET { key } => {
                let store = self.store.read().unwrap();
                let value = store.get(key.as_slice());
                let value = value.map_or(Value::Nil, |s| Value::from_slice(s));
                RedisValue::from_value(value)
            }
            Cmd::DEL { keys } => {
                let mut store = self.store.write().unwrap();
                let deleted: usize = (&keys)
                    .iter()
                    .map(|k| store.remove(k.as_slice()).map_or(0, |_| 1))
                    .sum();

                RedisValue::from_value(Value::from_integer(deleted as i64))
            }
            Cmd::SET { key, value } => {
                let mut store = self.store.write().unwrap();
                store.insert(key.as_slice().to_vec(), value.as_slice().to_vec());
                RedisValue::ok()
            }
        }
    }
}
