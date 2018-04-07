use std::io;
use std::sync::Arc;

use futures::future;
use tokio_service::Service;

use super::redis_value::{RedisValue, Value};
use super::store::Store;
use super::commands::parse_command;

pub struct RedisService {
    store: Arc<Store>,
}
impl RedisService {
    pub fn new(store: Arc<Store>) -> Self {
        RedisService { store }
    }
}

impl Service for RedisService {
    type Request = RedisValue;
    type Response = RedisValue;
    type Error = io::Error;
    // For simplicity, box the future.
    type Future = future::FutureResult<RedisValue, io::Error>;

    fn call(&self, req: RedisValue) -> Self::Future {
        let response = parse_command(req.nodes)
            .map(|cmd| self.store.run_command(cmd))
            .unwrap_or_else(|_| {
                let value = Value::from_error("Error ocurred");
                RedisValue::from_value(value)
            });

        future::ok(response)
    }
}
