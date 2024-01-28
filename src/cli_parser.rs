use core::panic;
use std::net::{IpAddr, Ipv4Addr, ToSocketAddrs};

use clap::{arg, command, Parser};
use pnet_datalink::NetworkInterface;
use rand::Rng;
const MAX: u16 = 65535;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Target host
    #[arg(short, long, value_delimiter = ',')]
    target_ip: Vec<Ipv4Addr>,

    /// Target port
    #[arg(short, long)]
    port: Option<u16>,

    /// Target start port
    #[arg(short, long, default_value_t = 1)]
    start_port: u16,

    /// Target end port
    #[arg(short, long, default_value_t = MAX)]
    end_port: u16,

    /// Target end port
    #[arg(short, long, default_value_t = String::from("en0"))]
    interface: String,

    /// Target end port
    #[arg(short, long, default_value_t = false)]
    verbose: bool,
}

pub enum ParseCliOutput {
    WithRange {
        target_ip: Vec<Ipv4Addr>,
        start_port: u16,
        end_port: u16,
        verbose: bool,
        interface: NetworkInterface,
        source_ip: Ipv4Addr,
        source_port: u16,
    },
    WithTarget {
        target_ip: Vec<Ipv4Addr>,
        port: u16,
        verbose: bool,
        interface: NetworkInterface,
        source_ip: Ipv4Addr,
        source_port: u16,
    },
}

pub fn parse_cli() -> ParseCliOutput {
    let args: Args = Args::parse();
    let addr = args.target_ip;
    let interface = args.interface;
    let verbose = args.verbose;

    if addr.is_empty() {
        panic!("You must at least give one valid address!")
    }

    let start_port = args.start_port;
    let end_port = args.end_port;
    let port = args.port;

    let (interface, source_ip, source_port) = match fun_name(&addr, interface, start_port, end_port)
    {
        Ok(value) => value,
        Err(value) => panic!("ERROR - {}", value),
    };

    if let Some(port) = port {
        return ParseCliOutput::WithTarget {
            target_ip: addr,
            port,
            interface,
            verbose,
            source_ip,
            source_port: source_port as u16,
        };
    }

    return ParseCliOutput::WithRange {
        target_ip: addr,
        start_port,
        end_port,
        interface,
        verbose,
        source_ip,
        source_port: source_port as u16,
    };
}

fn fun_name(
    addr: &[Ipv4Addr],
    interface: String,
    start_port: u16,
    end_port: u16,
) -> Result<(NetworkInterface, Ipv4Addr, i32), &str> {
    for addrs in addr.iter() {
        println!(
            "Scanning {} using SYN scan:\n*Interface: {}\n*Staring port: {}\n*Ending port: {}\n",
            addrs, interface, start_port, end_port
        );
    }
    let interface = match pnet_datalink::interfaces()
        .iter()
        .find(|iface| iface.name == interface)
    {
        Some(iface) => iface.to_owned(),
        None => return Err("Couldn't find interface with such name"),
    };
    let source_port = rand::thread_rng().gen_range(10000..=65535);
    let ip: Ipv4Addr =
        match interface.ips.iter().find(|&entry| entry.is_ipv4()) {
            Some(addr) => match addr.ip() {
                IpAddr::V4(ipv4_addr) => ipv4_addr,
                IpAddr::V6(_) => return Err(
                    "Error happened while looking for a valid Ipv4 address in the given interface",
                ),
            },
            None => return Err("No valid IPv4 addresses found for the given interface"),
        };
    Ok((interface, ip, source_port))
}

fn dns_lookup(hostname: &str) -> Result<Ipv4Addr, &'static str> {
    let mut lookup_results = match format!("{hostname}:8000").to_socket_addrs() {
        Ok(iter) => iter,
        Err(_) => return Err("LOOKUP_FAILED"),
    };
    let addr = match lookup_results.next() {
        Some(sock_addr) => match sock_addr.ip() {
            IpAddr::V4(addr) => addr,
            IpAddr::V6(_) => return Err("IPV6_ONLY"),
        },
        None => return Err("NO_ADDRS_FOUND"),
    };
    Ok(addr)
}
