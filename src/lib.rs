mod parser;
mod tuple;

pub const SERVER_PORT: u16 = 1999;

pub fn error(message: &str) -> ! {
    eprint!("{}", message);
    std::process::exit(1)
}
