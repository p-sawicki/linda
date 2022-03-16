use std::{env, io, net};

fn main() {
    let num_clients = init();
    let clients = collect_clients(num_clients);
    send_connection_info(&clients);
}

fn init() -> usize {
    let mut args = env::args();
    let prog_name = args.next().unwrap();
    match args.next() {
        Some(val) => match val.parse() {
            Ok(val) => val,
            Err(_) => {
                linda::error(&format!(
                    "Expected positive integer as first argument, got: {}",
                    val
                ));
            }
        },
        None => linda::error(&format!("Usage:\n{} $NUMBER_OF_CLIENTS", prog_name)),
    }
}

fn collect_clients(num: usize) -> Vec<net::SocketAddr> {
    let localhost = net::SocketAddrV4::new(net::Ipv4Addr::LOCALHOST, linda::SERVER_PORT);
    let listener = match net::TcpListener::bind(localhost) {
        Ok(val) => val,
        Err(e) => linda::error(&format!(
            "Bind to local address {} failed! {}",
            localhost, e
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

fn send_connection_info(clients: &[net::SocketAddr]) {
    if clients.is_empty() {
        return;
    }

    let mut prev = None;
    let mut next = clients.iter();
    if let None = next.next() {
        next = clients.iter();
    }

    for addr in clients.iter() {
        let mut stream = match net::TcpStream::connect(addr) {
            Ok(val) => val,
            Err(e) => linda::error(&format!("Connection to client {} failed - {}!", addr, e)),
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
            if let Err(e) = io::Write::write(&mut stream, &buffer[..]) {
                linda::error(&format!("Write to client {} failed - {}!", addr, e));
            }
        }
    }
}

fn to_bytes(addr: &net::SocketAddr) -> Vec<u8> {
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
