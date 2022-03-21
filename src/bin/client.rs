use linda::{recv_message, tuple::Value, utils::*};
use std::{env, io, net};

fn main() {
    let server_socket = init();
    let (prev_client, next_client) = connect_to_server(server_socket);

    println!(
        "Previous in ring: {}, next in ring: {}",
        prev_client, next_client
    );
}

fn init() -> net::SocketAddr {
    let mut args = env::args();
    let prog_name = args.next().unwrap();

    if args.len() < 1 {
        error(&format!("Usage:\n{} $SERVER_ADDRESS", prog_name));
    }

    match args.next().unwrap().parse() {
        Ok(addr) => net::SocketAddr::new(addr, SERVER_PORT),
        Err(e) => error(&format!("Incorrect server address! {}", e)),
    }
}

fn connect_to_server(server_socket: net::SocketAddr) -> (net::SocketAddr, net::SocketAddr) {
    let client_socket = match net::TcpStream::connect(server_socket) {
        Ok(mut stream) => {
            if let Err(e) = io::Write::write(&mut stream, "hello".as_bytes()) {
                error(&format!("Failed to send to server! {}", e));
            }
            match stream.local_addr() {
                Ok(addr) => addr,
                Err(e) => error(&format!("Failed to obtain local address! {}", e)),
            }
        }
        Err(e) => error(&format!("Connection to {} failed! {}", server_socket, e)),
    };
    println!(
        "Connected from {} to server {}",
        client_socket, server_socket
    );

    let listener = match net::TcpListener::bind(client_socket) {
        Ok(list) => list,
        Err(e) => error(&format!("Failed to bind to {}! {}", client_socket, e)),
    };

    let (mut stream, _) = match listener.accept() {
        Ok(res) => res,
        Err(e) => error(&format!("Failed to accept incoming connection! {}", e)),
    };

    (get_socket(&mut stream), get_socket(&mut stream))
}

fn get_socket(stream: &mut net::TcpStream) -> net::SocketAddr {
    match recv_message::<Value>(stream) {
        Ok(msg) => msg.ip,
        Err(e) => error(&format!("Failed to obtain socket! {}", e)),
    }
}
