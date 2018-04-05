extern crate stringreader;

mod resp;

use resp::RespProtocol;

use std::net::{Ipv4Addr, SocketAddrV4, TcpListener};
use std::io::{Read, Write};
use std::io;

fn run() -> io::Result<()> {
    let loopback = Ipv4Addr::new(127, 0, 0, 1);
    // Assigning port 0 requests the OS to assign a free port
    let socket = SocketAddrV4::new(loopback, 5500);
    let listener = TcpListener::bind(socket)?;
    let port = listener.local_addr()?;
    println!("Listening on {}, access this port to end the program", port);
    let (mut tcp_stream, addr) = listener.accept()?; //block  until requested
    println!("Connection received! {:?} is sending data.", addr);
    let mut input = String::new();
    // read from the socket until connection closed by client, discard byte count.
    let _ = tcp_stream.read_to_string(&mut input)?;
    let _ = tcp_stream.write_all(input.as_bytes())?;
    Ok(())
}

fn main() {
    run();
}
