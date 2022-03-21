use std::{io, mem, net};
use utils::*;

use tuple::*;

pub mod parser;
pub mod tuple;
pub mod utils;

pub fn send_message<T: Serializable>(
    stream: &mut net::TcpStream,
    msg: Message<T>,
) -> Result<(), String> {
    let bytes = msg.to_bytes();
    let size = bytes.len().to_le_bytes();

    for buf in [&size[..], &bytes[..]] {
        if let Err(e) = io::Write::write(stream, buf) {
            return Err(e.to_string());
        }
    }
    Ok(())
}

pub fn recv_message<T: Serializable>(stream: &mut net::TcpStream) -> Result<Message<T>, String> {
    let mut size = [0u8; mem::size_of::<usize>()];
    if let Err(e) = io::Read::read(stream, &mut size[..]) {
        return Err(e.to_string());
    }

    let size = match read_le_usize(&mut &size[..]) {
        Some(val) => val,
        None => return Err(String::from("Failed to receive message size!")),
    };

    let mut bytes = Vec::new();
    bytes.resize(size, 0);
    if let Err(e) = io::Read::read(stream, &mut bytes[..]) {
        return Err(e.to_string());
    }

    match Message::from_bytes(&mut &bytes[..]) {
        Some(msg) => Ok(msg),
        None => return Err(String::from("Failed to parse message!")),
    }
}
