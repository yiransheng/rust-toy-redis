#[macro_use]
extern crate matches;
extern crate stringreader;

mod resp;

use resp::{read_protocol, ProtocolError, RespProtocol, Result, SimpleBytes};

use std::net::{Ipv4Addr, SocketAddrV4, TcpListener};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::thread;
use std::time::Duration;

fn sleep() {
    thread::sleep(Duration::from_millis(100));
}

fn run() -> io::Result<()> {
    let loopback = Ipv4Addr::new(127, 0, 0, 1);
    // Assigning port 0 requests the OS to assign a free port
    let socket = SocketAddrV4::new(loopback, 6379);
    let listener = TcpListener::bind(socket)?;
    let port = listener.local_addr()?;
    println!("Listening on {}, access this port to end the program", port);
    let (mut tcp_stream, addr) = listener.accept()?; //block  until requested
    println!("Connection received! {:?} is sending data.", addr);

    let mut buf_reader = tcp_stream.try_clone().map(BufReader::new)?;
    loop {
        let protocol = read_protocol(&mut buf_reader);

        match protocol {
            Ok(p) => {
                println!("{}", p.to_string());
                let response = RespProtocol::ok();

                let _ = tcp_stream.write_all(&response.into_bytes())?;
            }
            Err(err) => match err {
                ProtocolError::IoError(e) => return Err(e),
                _ => {
                    let simple_bytes = SimpleBytes::from_bytes("ERR".as_bytes());
                    let response = RespProtocol::Error(simple_bytes.unwrap());

                    let _ = tcp_stream.write_all(&response.into_bytes())?;
                }
            },
        }
        sleep();
    }
}

fn main() {
    if let Err(e) = run() {
        println!("ERROR: {:?}", e);
    }
}
