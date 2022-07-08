use linda::{
    message::*,
    parser::*,
    tuple::{Tuple, Value},
    utils::*,
    *,
};
use std::{env, io, net, time};

fn main() {
    let server_socket = init();
    let (prev_client, next_client) = connect_to_server(server_socket);

    client_loop(prev_client, next_client)
}

fn init() -> net::SocketAddr {
    let mut args = env::args();
    let prog_name = args.next().unwrap();

    if args.len() < 1 {
        error(&format!("Usage:\n{prog_name} $SERVER_ADDRESS"));
    }

    match args.next().unwrap().parse() {
        Ok(addr) => net::SocketAddr::new(addr, SERVER_PORT),
        Err(e) => error(&format!("Incorrect server address! {e}")),
    }
}

fn connect_to_server(server_socket: net::SocketAddr) -> (net::TcpListener, net::SocketAddr) {
    let listener = match net::TcpListener::bind("127.0.0.1:0") {
        Ok(list) => list,
        Err(e) => error(&format!("Failed to bind! {e}")),
    };
    let port = match listener.local_addr() {
        Ok(addr) => addr.port(),
        Err(e) => error(&format!("Failed to receive local address! {e}")),
    };
    println!("Client listening on port {port}");

    let client_socket = match net::TcpStream::connect(server_socket) {
        Ok(mut stream) => {
            let msg = Message::value(
                Tuple::from_vec(vec![Value::int(port as i32)]),
                server_socket,
            );
            if let Err(e) = msg.send(&mut stream) {
                error(&format!("Failed to send to server! {e:?}"));
            }
            match stream.local_addr() {
                Ok(addr) => addr,
                Err(e) => error(&format!("Failed to obtain local address! {e}")),
            }
        }
        Err(e) => error(&format!("Connection to {server_socket} failed! {e}")),
    };
    println!("Connected from {client_socket} to server {server_socket}");

    let (mut stream, _) = match listener.accept() {
        Ok(res) => res,
        Err(e) => error(&format!("Failed to accept incoming connection! {e}")),
    };

    (listener, get_socket(&mut stream))
}

fn get_socket(stream: &mut net::TcpStream) -> net::SocketAddr {
    match Message::recv(stream) {
        Ok(msg) => msg.ip,
        Err(e) => error(&format!("Failed to obtain socket! {e:?}")),
    }
}

fn client_loop(local: net::TcpListener, next: net::SocketAddr) {
    let next = match net::TcpStream::connect(next) {
        Ok(str) => str,
        Err(e) => error(&format!("Failed to connect to next in ring! {e}")),
    };
    let (prev, _) = match local.accept() {
        Ok(str) => str,
        Err(e) => error(&format!("Failed to accept incoming stream! {e}")),
    };

    let linda = Linda::new(prev, next, local.local_addr().unwrap());
    loop {
        let command = match get_command() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("{e}");
                continue;
            }
        };

        println!("Got command: {:?}", command);

        match match command {
            Command::Exit => break,
            Command::Help => {
                println!("Under construction!");
                break;
            }
            Command::In(tuple, timeout) => {
                linda.input(tuple, time::Duration::from_secs(timeout as u64))
            }
            Command::Inp(tuple) => linda.inp(&tuple),
            Command::Out(tuple) => {
                if let Err(e) = linda.out(tuple) {
                    eprintln!("{e:?}");
                }
                continue;
            }
            Command::Rd(tuple, timeout) => {
                linda.read(tuple, time::Duration::from_secs(timeout as u64))
            }
            Command::Rdp(tuple) => linda.rdp(&tuple),
        } {
            Ok(tuple) => println!("Received: {tuple:?}"),
            Err(e) => eprintln!("Error: {e:?}"),
        };
    }
}

fn get_command() -> Result<Command, &'static str> {
    let mut input = String::new();
    match io::stdin().read_line(&mut input) {
        Ok(_) => {
            let mut parser = Parser::new(&input);
            parser.parse()
        }
        Err(_) => Err("Failed to read command - try again!"),
    }
}
