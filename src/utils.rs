use std::{
    mem,
    net::{self, Ipv4Addr, Ipv6Addr},
};

use crate::tuple::{Request, Tuple, Value};

pub type Timeout = usize;

#[derive(PartialEq, Debug)]
pub enum Command {
    Out(Tuple<Value>),
    In(Tuple<Request>, Timeout),
    Rd(Tuple<Request>, Timeout),
    Inp(Tuple<Request>),
    Rdp(Tuple<Request>),
    Help,
    Exit,
}

pub const SERVER_PORT: u16 = 1999;
pub const IP_ADDR_LENGTH: usize = mem::size_of::<u8>();
pub const IPV4_ADDR_LENGTH: usize = net::Ipv4Addr::LOCALHOST.octets().len();
pub const IPV6_ADDR_LENGTH: usize = net::Ipv6Addr::LOCALHOST.octets().len();
pub const PORT_LENGTH: usize = mem::size_of::<u16>();

pub fn error(message: &str) -> ! {
    eprint!("{}", message);
    std::process::exit(1)
}

pub fn read_le_u8(input: &mut &[u8]) -> Option<u8> {
    let (byte, rest) = input.split_at(mem::size_of::<u8>());
    *input = rest;
    Some(u8::from_le_bytes(byte.try_into().ok()?))
}

pub fn read_le_i32(input: &mut &[u8]) -> Option<i32> {
    let (int_bytes, rest) = input.split_at(mem::size_of::<i32>());
    *input = rest;
    Some(i32::from_le_bytes(int_bytes.try_into().ok()?))
}

pub fn read_le_u16(input: &mut &[u8]) -> Option<u16> {
    let (int_bytes, rest) = input.split_at(mem::size_of::<u16>());
    *input = rest;
    Some(u16::from_le_bytes(int_bytes.try_into().ok()?))
}

pub fn read_le_f64(input: &mut &[u8]) -> Option<f64> {
    let (float_bytes, rest) = input.split_at(mem::size_of::<f64>());
    *input = rest;
    Some(f64::from_le_bytes(float_bytes.try_into().ok()?))
}

pub fn read_le_usize(input: &mut &[u8]) -> Option<usize> {
    let (usize_bytes, rest) = input.split_at(mem::size_of::<usize>());
    *input = rest;
    Some(usize::from_le_bytes(usize_bytes.try_into().ok()?))
}

pub fn ip_to_bytes(addr: &net::SocketAddr) -> Vec<u8> {
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

pub fn bytes_to_ip(bytes: &mut &[u8]) -> Option<net::SocketAddr> {
    let ip_ver = read_le_u8(bytes)?;
    let addr = match ip_ver {
        4 => {
            let (ip_bytes, rest) = bytes.split_at(IPV4_ADDR_LENGTH);
            *bytes = rest;
            let buffer: [u8; IPV4_ADDR_LENGTH] = ip_bytes.try_into().ok()?;
            net::IpAddr::V4(Ipv4Addr::from(buffer))
        }
        6 => {
            let (ip_bytes, rest) = bytes.split_at(IPV6_ADDR_LENGTH);
            *bytes = rest;
            let buffer: [u8; IPV6_ADDR_LENGTH] = ip_bytes.try_into().ok()?;
            net::IpAddr::V6(Ipv6Addr::from(buffer))
        }
        _ => return None,
    };
    let port = read_le_u16(bytes)?;

    Some(net::SocketAddr::new(addr, port))
}
