#[macro_use]
extern crate matches;
extern crate stringreader;

mod resp;
mod commands;

use resp::{read_protocol, ProtocolError, RespProtocol, Result, SimpleBytes};

use std::net::{Ipv4Addr, SocketAddrV4, TcpListener, TcpStream};
use std::result;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::thread;
use std::time::Duration;

fn sleep() {
    thread::sleep(Duration::from_millis(100));
}

fn handle_connection(mut tcp_stream: TcpStream) -> io::Result<()> {
    let mut buffered_stream = tcp_stream.try_clone().map(BufReader::new)?;

    loop {
        let protocol = read_protocol(&mut buffered_stream);

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

fn run() -> io::Result<()> {
    let loopback = Ipv4Addr::new(127, 0, 0, 1);
    let socket = SocketAddrV4::new(loopback, 6379);
    let listener = TcpListener::bind(socket)?;
    let port = listener.local_addr()?;

    println!("Listening on {}, access this port to end the program", port);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || {
                    let _ =
                        handle_connection(stream).map_err(|e| println!("Connection died: {}", e));
                });
            }
            Err(_) => (),
        }
    }
    Ok(())
}

fn main() {
    if let Err(e) = run() {
        println!("ERROR: {:?}", e);
    }
}
