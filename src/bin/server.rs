use linda::{message::*, tuple::*, utils::*};
use std::{env, net};

fn main() {
    let num_clients = init();
    println!("Starting server for {num_clients} clients");
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
                    "Expected positive integer as first argument, got: {val}"
                ));
            }
        },
        None => error(&format!("Usage:\n{prog_name} $NUMBER_OF_CLIENTS")),
    }
}

fn get_port(stream: &mut net::TcpStream) -> Result<u16, String> {
    let incorrect_msg_error = Err(String::from("No port in incoming message - skipping!"));
    match Message::recv(stream) {
        Ok(msg) => match msg.tuple {
            MessageType::Value(tuple) => match tuple.first() {
                Some(Value::Int(Some(val))) => Ok(*val as u16),
                _ => incorrect_msg_error,
            },
            _ => incorrect_msg_error,
        },
        Err(e) => Err(String::from(format!(
            "Failed to receive port - skipping: {e:?}!",
        ))),
    }
}

fn collect_clients(num: usize) -> Vec<net::SocketAddr> {
    let localhost = net::SocketAddrV4::new(net::Ipv4Addr::LOCALHOST, SERVER_PORT);
    let listener = match net::TcpListener::bind(localhost) {
        Ok(val) => val,
        Err(e) => error(&format!("Bind to local address {localhost} failed! {e}")),
    };
    println!("Listening at {}", listener.local_addr().unwrap());

    let mut clients = Vec::new();
    clients.reserve(num);
    let mut index = 0;

    while index < num {
        match listener.accept() {
            Ok((mut stream, addr)) => {
                index += 1;
                let port = match get_port(&mut stream) {
                    Ok(p) => p,
                    Err(e) => {
                        eprintln!("{e}");
                        continue;
                    }
                };
                let client_addr = net::SocketAddr::new(addr.ip(), port);
                println!("[{index}/{num}] Adding client {client_addr}.");
                clients.push(client_addr);
            }
            Err(e) => eprintln!("Incoming connection failed - skipping client! {e}"),
        }
    }

    clients
}

fn send_connection_info(clients: &[net::SocketAddr]) {
    if clients.is_empty() {
        return;
    }

    let mut next = clients.iter();
    if let None = next.next() {
        next = clients.iter();
    }

    for addr in clients.iter() {
        let mut stream = match net::TcpStream::connect(addr) {
            Ok(val) => val,
            Err(e) => error(&format!("Connection to client {addr} failed - {e}!")),
        };

        let next_ip = match next.next() {
            Some(addr) => addr,
            None => clients.first().unwrap(),
        };
        if let Err(e) = Message::from_ip(next_ip.clone()).send(&mut stream) {
            error(&format!("Write to client {addr} failed - {e:?}!"));
        }
    }
}
