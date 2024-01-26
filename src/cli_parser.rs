use std::net::IpAddr;

use clap::{arg, command, Parser};
const MAX: u16 = 65535;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Target host
    #[arg(short, long, value_delimiter = ',')]
    target: Vec<IpAddr>,

    /// Target port
    #[arg(short, long)]
    port: Option<u16>,

    /// Target start port
    #[arg(short, long, default_value_t = 1)]
    start_port: u16,

    /// Target end port
    #[arg(short, long, default_value_t = MAX)]
    end_port: u16,
}

pub enum ParseCliOutput {
    WithRange {
        target: Vec<IpAddr>,
        start_port: u16,
        end_port: u16,
    },
    WithTarget {
        target: Vec<IpAddr>,
        port: u16,
    },
}

pub fn parse_cli() -> ParseCliOutput {
    let args: Args = Args::parse();
    let addr = args.target;

    if addr.is_empty() {
        panic!("You must at least give one valid address!")
    }

    let start_port = args.start_port;
    let end_port = args.end_port;
    let port = args.port;

    if let Some(port) = port {
        return ParseCliOutput::WithTarget { target: addr, port };
    }

    return ParseCliOutput::WithRange {
        target: addr,
        start_port,
        end_port,
    };
}
