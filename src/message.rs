use std::{io, mem, net};

use crate::{tuple::*, utils::*};

const VALUE_ID: u8 = 0;
const REQUEST_ID: u8 = 1;

#[derive(Debug, PartialEq)]
pub enum MessageType {
    Value(Tuple<Value>),
    Request(Tuple<Request>),
}

#[derive(Debug, PartialEq)]
pub struct Message {
    pub tuple: MessageType,
    pub ip: net::SocketAddr,
}

impl Message {
    pub fn value(tuple: Tuple<Value>, ip: net::SocketAddr) -> Message {
        Message {
            tuple: MessageType::Value(tuple),
            ip,
        }
    }

    pub fn request(tuple: Tuple<Request>, ip: net::SocketAddr) -> Message {
        Message {
            tuple: MessageType::Request(tuple),
            ip,
        }
    }

    pub fn from_ip(ip: net::SocketAddr) -> Message {
        Message {
            tuple: MessageType::Value(Tuple::new()),
            ip,
        }
    }

    pub fn send<OutputStream: io::Write>(&self, stream: &mut OutputStream) -> Result<(), String> {
        let bytes = self.to_bytes();
        let size = bytes.len().to_le_bytes();

        for buf in [&size[..], &bytes[..]] {
            if let Err(e) = stream.write(buf) {
                return Err(e.to_string());
            }
        }
        Ok(())
    }

    pub fn recv<InputStream: io::Read>(stream: &mut InputStream) -> Result<Message, String> {
        let mut size = [0u8; mem::size_of::<usize>()];
        if let Err(e) = stream.read(&mut size[..]) {
            return Err(e.to_string());
        }

        let size = match read_le_usize(&mut &size[..]) {
            Some(val) => val,
            None => return Err(String::from("Failed to receive message size!")),
        };

        let mut bytes = Vec::new();
        bytes.resize(size, 0);
        if let Err(e) = stream.read(&mut bytes[..]) {
            return Err(e.to_string());
        }

        match Message::from_bytes(&mut &bytes[..]) {
            Some(msg) => Ok(msg),
            None => return Err(String::from("Failed to parse message!")),
        }
    }
}

impl Serializable for Message {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        match &self.tuple {
            MessageType::Value(tuple) => {
                bytes.append(&mut VALUE_ID.to_le_bytes().to_vec());
                bytes.append(&mut tuple.to_bytes());
            }
            MessageType::Request(tuple) => {
                bytes.append(&mut REQUEST_ID.to_le_bytes().to_vec());
                bytes.append(&mut tuple.to_bytes());
            }
        };
        bytes.append(&mut ip_to_bytes(&self.ip));

        bytes
    }

    fn from_bytes(bytes: &mut &[u8]) -> Option<Message> {
        match read_le_u8(bytes) {
            Some(VALUE_ID) => Some(Message::value(
                Tuple::<Value>::from_bytes(bytes)?,
                bytes_to_ip(bytes)?,
            )),
            Some(REQUEST_ID) => Some(Message::request(
                Tuple::<Request>::from_bytes(bytes)?,
                bytes_to_ip(bytes)?,
            )),
            _ => None,
        }
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

    fn check_message(message: Message) {
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
            tuple: MessageType::Request(tuple),
            ip: net::SocketAddr::new(
                net::IpAddr::V4(net::Ipv4Addr::LOCALHOST),
                crate::utils::SERVER_PORT,
            ),
        });

        check_message(Message::from_ip(net::SocketAddr::new(
            net::IpAddr::V6(net::Ipv6Addr::LOCALHOST),
            utils::SERVER_PORT,
        )));
    }
}
