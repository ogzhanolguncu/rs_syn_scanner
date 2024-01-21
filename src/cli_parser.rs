use std::net::IpAddr;

use clap::{arg, command, Parser};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    // Target host
    #[arg(short, long)]
    target: String,

    // Target port
    #[arg(short, long)]
    port: Option<u16>,
}

pub fn parse_cli() -> (IpAddr, Option<u16>) {
    let args: Args = Args::parse();
    let addr = args.target.parse::<IpAddr>().unwrap();
    return (addr, args.port);
}
