use std::{io, mem, net, sync};

use crate::{tuple::*, utils::*};

const VALUE_ID: u8 = 0;
const REQUEST_ID: u8 = 1;

#[derive(Clone, Debug, PartialEq)]
pub enum MessageType {
    Value(Tuple<Value>),
    Request(Tuple<Request>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Message {
    pub tuple: MessageType,
    pub ip: net::SocketAddr,
}

#[derive(Debug)]
pub enum LindaError {
    MutexLockFailure(String),
    IoFailure(io::Error),
    MessageParseFailure,
    ChannelSendFailure(sync::mpsc::SendError<Message>),
    NoTuple,
    Timeout,
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

    pub fn send<OutputStream: io::Write>(
        &self,
        stream: &mut OutputStream,
    ) -> Result<(), LindaError> {
        let bytes = self.to_bytes();
        let size = bytes.len().to_le_bytes();

        for buf in [&size[..], &bytes[..]] {
            if let Err(e) = stream.write(buf) {
                return Err(LindaError::IoFailure(e));
            }
        }
        Ok(())
    }

    pub fn recv<InputStream: io::Read>(stream: &mut InputStream) -> Result<Message, LindaError> {
        let mut size = [0u8; mem::size_of::<usize>()];
        if let Err(e) = stream.read(&mut size[..]) {
            return Err(LindaError::IoFailure(e));
        }

        let size = match read_le_usize(&mut &size[..]) {
            Some(val) => val,
            None => return Err(LindaError::MessageParseFailure),
        };

        let mut bytes = Vec::new();
        bytes.resize(size, 0);
        if let Err(e) = stream.read(&mut bytes[..]) {
            return Err(LindaError::IoFailure(e));
        }

        match Message::from_bytes(&mut &bytes[..]) {
            Some(msg) => Ok(msg),
            None => return Err(LindaError::MessageParseFailure),
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

#[cfg(test)]
mod tests {
    use core::time;
    use std::thread;

    use super::*;

    fn check_message(message: Message) {
        assert_eq!(
            message,
            Message::from_bytes(&mut &message.to_bytes()[..]).unwrap()
        );
    }

    #[test]
    fn serialize_message() {
        let ip: net::SocketAddr = "127.0.0.1:0".parse().unwrap();

        let mut tuple = Tuple::new();
        tuple.push(Request::new(Value::int(420), ComparisonOperator::LE));
        check_message(Message {
            tuple: MessageType::Request(tuple),
            ip: ip.clone(),
        });

        let mut tuple = Tuple::new();
        tuple.push(Value::int(69));
        tuple.push(Value::String(None));
        check_message(Message {
            tuple: MessageType::Value(tuple),
            ip,
        });

        check_message(Message::from_ip("[::1]:0".parse().unwrap()));
    }

    #[test]
    fn send_msg() {
        let mut tuple = Tuple::new();
        tuple.push(Request::new(Value::Float(None), ComparisonOperator::ANY));
        tuple.push(Request::new(
            Value::string(String::from("hello")),
            ComparisonOperator::EQ,
        ));
        tuple.push(Request::new(Value::int(36), ComparisonOperator::GE));

        let msg = Message::request(tuple, "[::1]:0".parse().unwrap());
        let msg_clone = msg.clone();

        thread::spawn(move || {
            let listener = net::TcpListener::bind("127.0.0.1:1999").unwrap();
            let (mut stream, _) = listener.accept().unwrap();
            assert_eq!(Message::recv(&mut stream).unwrap(), msg);
        });

        thread::sleep(time::Duration::from_millis(100));
        let mut stream = net::TcpStream::connect("127.0.0.1:1999").unwrap();
        msg_clone.send(&mut stream).unwrap();
    }
}
