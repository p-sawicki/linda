use std::{
    env::args,
    io::Write,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener, TcpStream},
    process::exit,
};

use linda::SERVER_PORT;

fn main() {
    let num_clients = init();
    let clients = collect_clients(num_clients);
    send_connection_info(&clients);
}

fn error(message: &str) -> ! {
    eprint!("{}", message);
    exit(1)
}

fn init() -> usize {
    let mut args = args();
    let prog_name = args.next().unwrap();
    match args.next() {
        Some(val) => match val.parse() {
            Ok(val) => val,
            Err(_) => {
                error(&format!(
                    "Expected positive integer as first argument, got: {}",
                    val
                ));
            }
        },
        None => error(&format!("Usage:\n{} $NUMBER_OF_CLIENTS", prog_name)),
    }
}

fn collect_clients(num: usize) -> Vec<SocketAddr> {
    let localhost = SocketAddrV4::new(Ipv4Addr::LOCALHOST, linda::SERVER_PORT);
    let listener = match TcpListener::bind(localhost) {
        Ok(val) => val,
        Err(e) => error(&format!(
            "Bind to local address 127.0.0.1:{} failed! {}",
            SERVER_PORT, e
        )),
    };

    let mut clients = Vec::new();
    clients.reserve(num);
    let mut index = 0;

    while index < num {
        match listener.accept() {
            Ok((_, addr)) => {
                index += 1;
                print!("[{}/{}] Adding client {}.", index, num, addr);
                clients.push(addr);
            }
            Err(e) => eprint!("Incoming connection failed - skipping client! {}", e),
        }
    }

    clients
}

fn send_connection_info(clients: &[SocketAddr]) {
    if clients.is_empty() {
        return;
    }

    let mut prev = None;
    let mut next = clients.iter();
    if let None = next.next() {
        next = clients.iter();
    }

    for addr in clients.iter() {
        let mut stream = match TcpStream::connect(addr) {
            Ok(val) => val,
            Err(e) => error(&format!("Connection to client {} failed - {}!", addr, e)),
        };

        let prev_buffer = to_bytes(match prev {
            None => {
                prev = Some(clients.iter());
                clients.last().unwrap()
            }
            Some(ref mut addr) => addr.next().unwrap(),
        });

        let next_buffer = to_bytes(match next.next() {
            Some(addr) => addr,
            None => clients.first().unwrap(),
        });

        for buffer in [prev_buffer, next_buffer] {
            if let Err(e) = stream.write(&buffer) {
                error(&format!("Write to client {} failed - {}!", addr, e));
            }
        }
    }
}

fn to_bytes(addr: &SocketAddr) -> Vec<u8> {
    let mut buffer = match addr {
        SocketAddr::V4(addr) => {
            let mut buffer = 4usize.to_le_bytes().to_vec();
            buffer.append(&mut addr.ip().octets().to_vec());
            buffer
        }
        SocketAddr::V6(addr) => {
            let mut buffer = 6usize.to_le_bytes().to_vec();
            buffer.append(&mut addr.ip().octets().to_vec());
            buffer
        }
    };
    buffer.append(&mut addr.port().to_le_bytes().to_vec());

    buffer
}
