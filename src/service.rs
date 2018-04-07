use std::{io, str};

use bytes::BytesMut;
use futures::{future, Future};

use tokio_io::{AsyncRead, AsyncWrite};
use tokio_io::codec::{Decoder, Encoder, Framed};
use tokio_core::net::TcpStream;
use tokio_core::reactor::Handle;
use tokio_proto::{TcpClient, TcpServer};
use tokio_proto::pipeline::{ClientProto, ClientService, ServerProto};
use tokio_service::{NewService, Service};

use super::redis_value::RedisValue;

pub struct RedisService;

impl Service for RedisService {
    type Request = RedisValue;
    type Response = RedisValue;
    type Error = io::Error;
    // For simplicity, box the future.
    type Future = Box<Future<Item = RedisValue, Error = io::Error>>;

    fn call(&self, req: RedisValue) -> Self::Future {
        Box::new(future::ok(RedisValue::ok()))
    }
}
