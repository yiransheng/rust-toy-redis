use std::io;

use futures::future;
use tokio_service::Service;

use super::redis_value::RedisValue;

pub struct RedisService;

impl Service for RedisService {
    type Request = RedisValue;
    type Response = RedisValue;
    type Error = io::Error;
    // For simplicity, box the future.
    type Future = future::FutureResult<RedisValue, io::Error>;

    fn call(&self, req: RedisValue) -> Self::Future {
        future::ok(RedisValue::ok())
    }
}
