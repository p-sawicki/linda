use std::{env, io, mem, net};

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
        linda::error(&format!("Usage:\n{} $SERVER_ADDRESS", prog_name));
    }

    match args.next().unwrap().parse() {
        Ok(addr) => net::SocketAddr::new(addr, linda::SERVER_PORT),
        Err(e) => linda::error(&format!("Incorrect server address! {}", e)),
    }
}

fn connect_to_server(server_socket: net::SocketAddr) -> (net::SocketAddr, net::SocketAddr) {
    let client_socket = match net::TcpStream::connect(server_socket) {
        Ok(mut stream) => {
            if let Err(e) = io::Write::write(&mut stream, "hello".as_bytes()) {
                linda::error(&format!("Failed to send to server! {}", e));
            }
            match stream.local_addr() {
                Ok(addr) => addr,
                Err(e) => linda::error(&format!("Failed to obtain local address! {}", e)),
            }
        }
        Err(e) => linda::error(&format!("Connection to {} failed! {}", server_socket, e)),
    };
    println!(
        "Connected from {} to server {}",
        client_socket, server_socket
    );

    let listener = match net::TcpListener::bind(client_socket) {
        Ok(list) => list,
        Err(e) => linda::error(&format!("Failed to bind to {}! {}", client_socket, e)),
    };

    let (mut stream, _) = match listener.accept() {
        Ok(res) => res,
        Err(e) => linda::error(&format!("Failed to accept incoming connection! {}", e)),
    };

    (get_socket(&mut stream), get_socket(&mut stream))
}

fn get_socket(stream: &mut net::TcpStream) -> net::SocketAddr {
    const IP_ADDR_LENGTH: usize = mem::size_of::<u8>();
    const IPV4_ADDR_LENGTH: usize = net::Ipv4Addr::LOCALHOST.octets().len();
    const IPV6_ADDR_LENGTH: usize = net::Ipv6Addr::LOCALHOST.octets().len();
    const PORT_LENGTH: usize = mem::size_of::<u16>();

    let mut buffer = [0u8; IP_ADDR_LENGTH];
    let ip_ver = match io::Read::read(stream, &mut buffer) {
        Ok(len) if len == IP_ADDR_LENGTH => u8::from_le_bytes(buffer),
        _ => linda::error(&format!("Failed to obtain IP version from server!")),
    };

    let ip_addr = match ip_ver {
        4 => {
            let mut buffer = [0u8; IPV4_ADDR_LENGTH];
            match io::Read::read(stream, &mut buffer) {
                Ok(len) if len == IPV4_ADDR_LENGTH => net::IpAddr::V4(net::Ipv4Addr::from(buffer)),
                _ => linda::error(&format!("Failed to obtain IPv4 address from server!")),
            }
        }
        6 => {
            let mut buffer = [0u8; IPV6_ADDR_LENGTH];
            match io::Read::read(stream, &mut buffer) {
                Ok(len) if len == IPV6_ADDR_LENGTH => net::IpAddr::V6(net::Ipv6Addr::from(buffer)),
                _ => linda::error(&format!("Failed to obtain IPv6 address from server!")),
            }
        }
        _ => linda::error(&format!("Invalid IP version from server!")),
    };

    let mut buffer = [0u8; PORT_LENGTH];
    let port = match io::Read::read(stream, &mut buffer) {
        Ok(len) if len == PORT_LENGTH => u16::from_le_bytes(buffer),
        _ => linda::error(&format!("Failed to obtain port from server!")),
    };

    net::SocketAddr::new(ip_addr, port)
}
