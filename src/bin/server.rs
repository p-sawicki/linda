use linda::{message::Message, tuple::*, utils::*};
use std::{
    env,
    io::Read,
    net::{self},
};

fn main() {
    let num_clients = init();
    println!("Starting server for {} clients", num_clients);
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
                error(&format!(
                    "Expected positive integer as first argument, got: {}",
                    val
                ));
            }
        },
        None => error(&format!("Usage:\n{} $NUMBER_OF_CLIENTS", prog_name)),
    }
}

fn collect_clients(num: usize) -> Vec<net::SocketAddr> {
    let localhost = net::SocketAddrV4::new(net::Ipv4Addr::LOCALHOST, SERVER_PORT);
    let listener = match net::TcpListener::bind(localhost) {
        Ok(val) => val,
        Err(e) => error(&format!(
            "Bind to local address {} failed! {}",
            localhost, e
        )),
    };
    println!("Listening at {}", listener.local_addr().unwrap());

    let mut clients = Vec::new();
    clients.reserve(num);
    let mut index = 0;

    while index < num {
        match listener.accept() {
            Ok((mut stream, addr)) => {
                index += 1;
                let mut port = [0 as u8; 2];
                let port = match stream.read(&mut port) {
                    Ok(_) => match read_le_u16(&mut &port[..]) {
                        Some(val) => val,
                        None => {
                            eprintln!("Failed to parse port number - skipping!");
                            continue;
                        }
                    },
                    Err(e) => {
                        eprintln!(
                            "Failed to receive port number from client - skipping! {}",
                            e
                        );
                        continue;
                    }
                };
                let client_addr = net::SocketAddr::new(addr.ip(), port);
                println!("[{}/{}] Adding client {}.", index, num, client_addr);
                clients.push(client_addr);
            }
            Err(e) => eprintln!("Incoming connection failed - skipping client! {}", e),
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
            Err(e) => error(&format!("Connection to client {} failed - {}!", addr, e)),
        };

        let prev_ip = match prev {
            None => {
                prev = Some(clients.iter());
                clients.last().unwrap()
            }
            Some(ref mut addr) => addr.next().unwrap(),
        };
        let prev_msg = Message::<Value>::from_ip(prev_ip.clone());

        let next_ip = match next.next() {
            Some(addr) => addr,
            None => clients.first().unwrap(),
        };
        let next_msg = Message::<Value>::from_ip(next_ip.clone());

        for msg in [prev_msg, next_msg] {
            if let Err(e) = msg.send(&mut stream) {
                error(&format!("Write to client {} failed - {}!", addr, e));
            }
        }
    }
}
