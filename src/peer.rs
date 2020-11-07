use std::net::Ipv4Addr;
use std::net::SocketAddr;
use std::net::TcpStream;
use std::time::Duration;

use crate::torrent::*;

#[derive(Debug)]
pub struct Peer {
    pub ip: Ipv4Addr,
    pub port: u16,
    pub torrent: Torrent,
    pub peer_id: String,
}

impl Peer {
    pub fn handshake(self) {
        let timeout = Duration::new(5, 0);
        let sock_addr = SocketAddr::from((self.ip, self.port));
        
        match TcpStream::connect_timeout(&sock_addr, timeout) {
            Ok(_) => println!("connected to: {:?}", self),
            Err(_) => {
                println!("cannot connect to: {:?}", self);
                return;
            }
        }; 
    }
}