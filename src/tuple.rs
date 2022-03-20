use std::{net, ops};

pub trait Serializable {
    fn to_bytes(&self) -> Vec<u8>;
    fn from_bytes(bytes: &mut &[u8]) -> Option<Self>
    where
        Self: Sized;
}

#[derive(PartialEq, Debug)]
pub enum Value {
    Int(Option<i32>),
    Float(Option<f64>),
    String(Option<String>),
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum ComparisonOperator {
    EQ,
    NEQ,
    GE,
    GT,
    LE,
    LT,
    ANY,
}

#[derive(PartialEq, Debug)]
pub struct Tuple<T>(Vec<T>);

#[derive(Debug, PartialEq)]
pub struct Request {
    value: Value,
    op: ComparisonOperator,
}

#[derive(Debug, PartialEq)]
pub struct Message<T> {
    tuple: Tuple<T>,
    ip: net::SocketAddr,
}

const INT_SIZE: i32 = -1;
const FLOAT_SIZE: i32 = -2;
const EMPTY_INT: i32 = -3;
const EMPTY_FLOAT: i32 = -4;
const EMPTY_STRING: i32 = -5;

impl Value {
    pub fn int(i: i32) -> Value {
        Value::Int(Some(i))
    }

    pub fn float(f: f64) -> Value {
        Value::Float(Some(f))
    }

    pub fn string(s: String) -> Value {
        Value::String(Some(s))
    }

    pub fn is_int(&self) -> bool {
        matches!(self, Value::Int(_))
    }

    pub fn is_float(&self) -> bool {
        matches!(self, Value::Float(_))
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }

    pub fn is_same_type(&self, other: &Value) -> bool {
        self.is_int() && other.is_int()
            || self.is_float() && other.is_float()
            || self.is_string() && other.is_string()
    }
}

impl Serializable for Value {
    fn to_bytes(&self) -> Vec<u8> {
        let mut result = Vec::new();
        match self {
            Value::Int(opt) => match opt {
                Some(i) => {
                    result.append(&mut INT_SIZE.to_le_bytes().to_vec());
                    result.append(&mut i.to_le_bytes().to_vec());
                }
                None => result.append(&mut EMPTY_INT.to_le_bytes().to_vec()),
            },
            Value::Float(opt) => match opt {
                Some(f) => {
                    result.append(&mut FLOAT_SIZE.to_le_bytes().to_vec());
                    result.append(&mut f.to_le_bytes().to_vec());
                }
                None => result.append(&mut EMPTY_FLOAT.to_le_bytes().to_vec()),
            },
            Value::String(opt) => match opt {
                Some(s) => {
                    result.append(&mut (s.as_bytes().len() as i32).to_le_bytes().to_vec());
                    result.append(&mut s.as_bytes().to_vec());
                }
                None => result.append(&mut EMPTY_STRING.to_le_bytes().to_vec()),
            },
        }

        result
    }

    fn from_bytes(bytes: &mut &[u8]) -> Option<Value> {
        let size = crate::utils::read_le_i32(bytes)?;
        match size {
            EMPTY_INT => Some(Value::Int(None)),
            EMPTY_FLOAT => Some(Value::Float(None)),
            EMPTY_STRING => Some(Value::String(None)),
            INT_SIZE => Some(Value::int(crate::utils::read_le_i32(bytes)?)),
            FLOAT_SIZE => Some(Value::float(crate::utils::read_le_f64(bytes)?)),
            s if (s as usize) <= bytes.len() => {
                let (string, rest) = bytes.split_at(s as usize); // Could be switched to split_at_unchecked() once it comes to stable Rust.
                *bytes = rest;
                Some(Value::string(String::from_utf8(string.to_vec()).ok()?))
            }
            _ => None,
        }
    }
}

impl Serializable for ComparisonOperator {
    fn to_bytes(&self) -> Vec<u8> {
        let value = match self {
            Self::EQ => 0i32,
            Self::NEQ => 1,
            Self::GE => 2,
            Self::GT => 3,
            Self::LE => 4,
            Self::LT => 5,
            Self::ANY => 6,
        };

        value.to_le_bytes().to_vec()
    }

    fn from_bytes(bytes: &mut &[u8]) -> Option<ComparisonOperator> {
        match crate::utils::read_le_i32(bytes)? {
            0 => Some(Self::EQ),
            1 => Some(Self::NEQ),
            2 => Some(Self::GE),
            3 => Some(Self::GT),
            4 => Some(Self::LE),
            5 => Some(Self::LT),
            6 => Some(Self::ANY),
            _ => None,
        }
    }
}

impl<T> Tuple<T> {
    pub fn new() -> Tuple<T> {
        Tuple(Vec::new())
    }
}

impl<T> ops::Deref for Tuple<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> ops::DerefMut for Tuple<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Serializable> Serializable for Tuple<T> {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = self.0.len().to_le_bytes().to_vec();
        for value in &self.0 {
            bytes.append(&mut value.to_bytes());
        }

        bytes
    }

    fn from_bytes(bytes: &mut &[u8]) -> Option<Tuple<T>> {
        let size = crate::utils::read_le_usize(bytes)?;
        let mut elements = Vec::new();
        for _ in 0..size {
            elements.push(T::from_bytes(bytes)?);
        }

        Some(Tuple(elements))
    }
}

impl Request {
    pub fn new(value: Value, op: ComparisonOperator) -> Request {
        Request { value, op }
    }
}

impl Serializable for Request {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = self.value.to_bytes();
        bytes.append(&mut self.op.to_bytes());

        bytes
    }

    fn from_bytes(bytes: &mut &[u8]) -> Option<Request> {
        let value = Value::from_bytes(bytes)?;
        let op = ComparisonOperator::from_bytes(bytes)?;

        Some(Request { value, op })
    }
}

impl<T: Serializable> Serializable for Message<T> {
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = self.tuple.to_bytes();
        bytes.append(&mut crate::utils::ip_to_bytes(&self.ip));

        bytes
    }

    fn from_bytes(bytes: &mut &[u8]) -> Option<Message<T>> {
        let tuple = Tuple::from_bytes(bytes)?;
        let ip = crate::utils::bytes_to_ip(bytes)?;

        Some(Message { tuple, ip })
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fmt,
        net::{Ipv4Addr, Ipv6Addr},
    };

    use super::*;

    fn check_value(value: Value) {
        assert_eq!(
            value,
            Value::from_bytes(&mut &value.to_bytes()[..]).unwrap()
        );
    }

    fn check_request(request: Request) {
        assert_eq!(
            request,
            Request::from_bytes(&mut &request.to_bytes()[..]).unwrap()
        );
    }

    fn check_tuple<T: Serializable + fmt::Debug + PartialEq>(tuple: Tuple<T>) {
        assert_eq!(
            tuple,
            Tuple::from_bytes(&mut &tuple.to_bytes()[..]).unwrap()
        );
    }

    fn check_message<T: Serializable + fmt::Debug + PartialEq>(message: Message<T>) {
        assert_eq!(
            message,
            Message::from_bytes(&mut &message.to_bytes()[..]).unwrap()
        );
    }

    #[test]
    fn serialize_value() {
        check_value(Value::int(1));
        check_value(Value::float(3.14));
        check_value(Value::string(String::new()));
        check_value(Value::string(String::from("test")));
        check_value(Value::Int(None));
        check_value(Value::Float(None));
        check_value(Value::String(None));
    }

    #[test]
    fn serialize_request() {
        check_request(Request::new(Value::Int(None), ComparisonOperator::ANY));
        check_request(Request::new(Value::float(3.14), ComparisonOperator::EQ));
        check_request(Request::new(
            Value::string(String::new()),
            ComparisonOperator::NEQ,
        ));
        check_request(Request::new(Value::int(1), ComparisonOperator::LT));
        check_request(Request::new(Value::float(3.14), ComparisonOperator::GT));
        check_request(Request::new(
            Value::string(String::from("test")),
            ComparisonOperator::LE,
        ));
    }

    #[test]
    fn serialize_tuple() {
        check_tuple(Tuple(vec![
            Value::int(1),
            Value::string(String::new()),
            Value::string(String::from("test")),
            Value::Int(None),
            Value::String(None),
        ]));

        check_tuple(Tuple(vec![
            Request::new(Value::float(3.14), ComparisonOperator::GE),
            Request::new(Value::Float(None), ComparisonOperator::ANY),
            Request::new(Value::String(None), ComparisonOperator::ANY),
        ]))
    }

    #[test]
    fn serialize_message() {
        check_message(Message {
            tuple: Tuple(vec![Request::new(Value::int(420), ComparisonOperator::LE)]),
            ip: net::SocketAddr::new(
                net::IpAddr::V4(Ipv4Addr::LOCALHOST),
                crate::utils::SERVER_PORT,
            ),
        });

        check_message(Message {
            tuple: Tuple::<Value>(vec![]),
            ip: net::SocketAddr::new(
                net::IpAddr::V6(Ipv6Addr::LOCALHOST),
                crate::utils::SERVER_PORT,
            ),
        });
    }
}
