extern crate stringreader;

mod resp;

use resp::{read_protocol, RespProtocol, Result, SimpleBytes};

use std::net::{Ipv4Addr, SocketAddrV4, TcpListener};
use std::io::{BufRead, BufReader, Read, Write};
use std::io;

fn run() -> Result<()> {
    let loopback = Ipv4Addr::new(127, 0, 0, 1);
    // Assigning port 0 requests the OS to assign a free port
    let socket = SocketAddrV4::new(loopback, 6379);
    let listener = TcpListener::bind(socket)?;
    let port = listener.local_addr()?;
    println!("Listening on {}, access this port to end the program", port);
    let (mut tcp_stream, addr) = listener.accept()?; //block  until requested
    println!("Connection received! {:?} is sending data.", addr);

    let mut buf_reader = tcp_stream.try_clone().map(BufReader::new)?;
    let protocol = read_protocol(&mut buf_reader);

    match protocol {
        Ok(_) => {
            let simple_bytes = SimpleBytes::from_bytes("Ok".as_bytes());
            let response = RespProtocol::SimpleString(simple_bytes.unwrap());

            let _ = tcp_stream.write_all(response.to_string().as_bytes())?;
        }
        Err(_) => {
            let simple_bytes = SimpleBytes::from_bytes("ERR".as_bytes());
            let response = RespProtocol::Error(simple_bytes.unwrap());

            let _ = tcp_stream.write_all(response.to_string().as_bytes())?;
        }
    }

    Ok(())
}

fn main() {
    run();
}
