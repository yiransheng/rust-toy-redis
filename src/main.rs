extern crate bytes;
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
mod protocol;

use std::net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream};
use std::result;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::thread;
use std::sync::Arc;
use std::time::Duration;

fn main() {}
