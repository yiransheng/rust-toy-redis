extern crate bytes;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate matches;
extern crate stringreader;

extern crate futures;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_proto;
extern crate tokio_service;

#[macro_use]
mod macros;
mod redis_value;
mod commands;
mod store;
mod protocol;
mod service;

use std::sync::Arc;
use tokio_proto::TcpServer;

use protocol::RedisProto;
use store::Store;
use service::RedisService;

fn main() {
    // Specify the localhost address
    let addr = "127.0.0.1:6379".parse().unwrap();

    // The builder requires a protocol and an address
    let server = TcpServer::new(RedisProto, addr);
    let store = Arc::new(Store::new());

    server.serve(move || Ok(RedisService::new(store.clone())));
}
