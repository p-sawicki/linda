use std::{io, mem, net};

use crate::{tuple::*, utils::*};

#[derive(Debug, PartialEq)]
pub struct Message<T> {
    pub tuple: Tuple<T>,
    pub ip: net::SocketAddr,
}

impl<T> Message<T> {
    pub fn new(tuple: Tuple<T>, ip: net::SocketAddr) -> Message<T> {
        Message { tuple, ip }
    }

    pub fn from_ip(ip: net::SocketAddr) -> Message<T> {
        Message {
            tuple: Tuple::new(),
            ip,
        }
    }
}

impl<T: Serializable> Message<T> {
    pub fn send(&self, stream: &mut net::TcpStream) -> Result<(), String> {
        let bytes = self.to_bytes();
        let size = bytes.len().to_le_bytes();

        for buf in [&size[..], &bytes[..]] {
            if let Err(e) = io::Write::write(stream, buf) {
                return Err(e.to_string());
            }
        }
        Ok(())
    }

    pub fn recv(stream: &mut net::TcpStream) -> Result<Message<T>, String> {
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
}

impl<T: Serializable> Serializable for Message<T> {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = self.tuple.to_bytes();
        bytes.append(&mut ip_to_bytes(&self.ip));

        bytes
    }

    fn from_bytes(bytes: &mut &[u8]) -> Option<Message<T>> {
        let tuple = Tuple::from_bytes(bytes)?;
        let ip = bytes_to_ip(bytes)?;

        Some(Message { tuple, ip })
    }
}

fn ip_to_bytes(addr: &net::SocketAddr) -> Vec<u8> {
    let mut buffer = match addr {
        net::SocketAddr::V4(addr) => {
            let mut buffer = 4u8.to_le_bytes().to_vec();
            buffer.append(&mut addr.ip().octets().to_vec());
            buffer
        }
        net::SocketAddr::V6(addr) => {
            let mut buffer = 6u8.to_le_bytes().to_vec();
            buffer.append(&mut addr.ip().octets().to_vec());
            buffer
        }
    };
    buffer.append(&mut addr.port().to_le_bytes().to_vec());

    buffer
}

fn bytes_to_ip(bytes: &mut &[u8]) -> Option<net::SocketAddr> {
    let ip_ver = read_le_u8(bytes)?;
    let addr = match ip_ver {
        4 => {
            let (ip_bytes, rest) = bytes.split_at(IPV4_ADDR_LENGTH);
            *bytes = rest;
            let buffer: [u8; IPV4_ADDR_LENGTH] = ip_bytes.try_into().ok()?;
            net::IpAddr::V4(net::Ipv4Addr::from(buffer))
        }
        6 => {
            let (ip_bytes, rest) = bytes.split_at(IPV6_ADDR_LENGTH);
            *bytes = rest;
            let buffer: [u8; IPV6_ADDR_LENGTH] = ip_bytes.try_into().ok()?;
            net::IpAddr::V6(net::Ipv6Addr::from(buffer))
        }
        _ => return None,
    };
    let port = read_le_u16(bytes)?;

    Some(net::SocketAddr::new(addr, port))
}

mod tests {
    use super::*;
    use crate::utils;
    use std::fmt;

    fn check_message<T: Serializable + fmt::Debug + PartialEq>(message: Message<T>) {
        assert_eq!(
            message,
            Message::from_bytes(&mut &message.to_bytes()[..]).unwrap()
        );
    }

    #[test]
    fn serialize_message() {
        let mut tuple = Tuple::new();
        tuple.push(Request::new(Value::int(420), ComparisonOperator::LE));
        check_message(Message {
            tuple,
            ip: net::SocketAddr::new(
                net::IpAddr::V4(net::Ipv4Addr::LOCALHOST),
                crate::utils::SERVER_PORT,
            ),
        });

        check_message(Message::<Value>::from_ip(net::SocketAddr::new(
            net::IpAddr::V6(net::Ipv6Addr::LOCALHOST),
            utils::SERVER_PORT,
        )));
    }
}
