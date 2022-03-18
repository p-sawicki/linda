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

pub fn error(message: &str) -> ! {
    eprint!("{}", message);
    std::process::exit(1)
}
