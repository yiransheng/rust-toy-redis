use bytes::{BufMut, Bytes, BytesMut};
use std::io;
use std::sync::Arc;

use futures::future;
use tokio_service::Service;

use super::resp::{Arguments, Cmd, Value};
use super::store::Store;

pub struct RedisService {
    store: Arc<Store>,
}
impl RedisService {
    pub fn new(store: Arc<Store>) -> Self {
        RedisService { store }
    }
}

impl Service for RedisService {
    type Request = Arguments<Bytes>;
    type Response = Value;
    type Error = io::Error;

    type Future = future::FutureResult<Value, io::Error>;

    fn call(&self, req: Arguments<Bytes>) -> Self::Future {
        let cmd = Cmd::from_args(req);

        let response = cmd.map(|cmd| self.store.run_command(cmd))
            .unwrap_or_else(|| Value::Status("Unknown Command".to_string()));

        future::ok(response)
    }
}
