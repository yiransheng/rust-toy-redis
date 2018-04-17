#![feature(conservative_impl_trait, universal_impl_trait)]

extern crate btoi;
extern crate bytes;
extern crate bytes_decoder;
#[macro_use]
extern crate matches;
#[macro_use]
extern crate lazy_static;
extern crate stringreader;

extern crate futures;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_proto;
extern crate tokio_service;

#[macro_use]
mod macros;
mod commands;
mod protocol;
mod redis_value;
mod resp;
mod service;
mod store;

use std::sync::Arc;
use tokio_proto::TcpServer;

use protocol::RedisProto;
// use service::RedisService;
use store::Store;

fn main() {
    /*
     *     // Specify the localhost address
     *     let addr = "127.0.0.1:6379".parse().unwrap();
     *
     *     // The builder requires a protocol and an address
     *     let server = TcpServer::new(RedisProto, addr);
     *     let store = Arc::new(Store::new());
     *
     *     server.serve(move || Ok(RedisService::new(store.clone())));
     */
}
