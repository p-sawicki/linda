use linda::{message::*, tuple::Value, utils::*};
use std::{
    env,
    io::Write,
    net::{self},
};

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
    let listener = match net::TcpListener::bind("127.0.0.1:0") {
        Ok(list) => list,
        Err(e) => error(&format!("Failed to bind! {}", e)),
    };
    let port = match listener.local_addr() {
        Ok(addr) => addr.port(),
        Err(e) => error(&format!("Failed to receive local address! {}", e)),
    };
    println!("Client listening on port {}", port);

    let client_socket = match net::TcpStream::connect(server_socket) {
        Ok(mut stream) => {
            if let Err(e) = stream.write(&port.to_le_bytes()) {
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

    let (mut stream, _) = match listener.accept() {
        Ok(res) => res,
        Err(e) => error(&format!("Failed to accept incoming connection! {}", e)),
    };

    (get_socket(&mut stream), get_socket(&mut stream))
}

fn get_socket(stream: &mut net::TcpStream) -> net::SocketAddr {
    match Message::<Value>::recv(stream) {
        Ok(msg) => msg.ip,
        Err(e) => error(&format!("Failed to obtain socket! {}", e)),
    }
}
