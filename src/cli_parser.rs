use std::net::IpAddr;

use clap::{arg, command, Parser};
const MAX: u16 = 65535;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Target host
    #[arg(short, long, default_value_t = String::from("127.0.0.1"))]
    target: String,

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
        target: IpAddr,
        start_port: u16,
        end_port: u16,
    },
    WithTarget {
        target: IpAddr,
        port: u16,
    },
}

pub fn parse_cli() -> ParseCliOutput {
    let args: Args = Args::parse();
    let addr = args
        .target
        .parse::<IpAddr>()
        .expect("You must give an address!");

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
