mod cli_parser;

use rayon::prelude::*;
use std::io::{Read, Write};
use std::net::{IpAddr, SocketAddr, TcpStream};
use std::time::Duration;

fn main() {
    let (host, port) = cli_parser::parse_cli();

    if port.is_some() {
        let is_port_alive = check_if_port_is_alive(host, port.unwrap_or_default());
        if is_port_alive.is_ok() {
            println!("Port: {} is open", port.unwrap_or_default())
        }
    } else {
        let ports: Vec<u16> = (1..=3000).collect();

        let alive_ports: Vec<String> = ports
            .par_iter()
            .filter_map(|&port| {
                if check_if_port_is_alive(host.clone(), port).is_ok() {
                    Some(format!("Port: {} is open", port))
                } else {
                    None
                }
            })
            .collect();
        println!("{:?}", alive_ports)
    }
}

fn check_if_port_is_alive(host: IpAddr, port: u16) -> Result<(), ()> {
    let timeout = Duration::from_millis(100);

    match TcpStream::connect_timeout(&SocketAddr::new(host, port), timeout) {
        Ok(mut stream) => {
            //Pinging the server
            let msg = "Ping";
            stream.write(msg.as_bytes()).unwrap();

            let mut buffer = [0; 1024];
            match stream.read(&mut buffer) {
                Ok(_) => Ok(()),
                Err(_) => Err(()),
            }
        }
        Err(_) => Err(()),
    }
}
