mod cli_parser;
use spinners::{Spinner, Spinners};

use std::io::{self, Write};
use std::net::{IpAddr, SocketAddr, TcpStream};
use std::sync::mpsc::{channel, Sender};
use std::time::Duration;

use cli_parser::ParseCliOutput;
use tokio::task;

#[tokio::main]
async fn main() {
    let cli_inputs: ParseCliOutput = cli_parser::parse_cli();
    let (tx, rx) = channel();
    let mut sp = Spinner::new(Spinners::Dots9, "🔎 Scanning ports... 🔎".into());

    match cli_inputs {
        cli_parser::ParseCliOutput::WithRange {
            target,
            start_port,
            end_port,
        } => {
            for addr in target {
                for port in start_port..end_port {
                    let tx: Sender<(IpAddr, u16)> = tx.clone();

                    task::spawn(async move { scan(tx, port, addr).await });
                }
            }
        }
        cli_parser::ParseCliOutput::WithTarget { target, port } => {
            let tx = tx.clone();
            for addr in target {
                let tx: Sender<(IpAddr, u16)> = tx.clone();

                task::spawn(async move { scan(tx, port, addr).await });
            }
        }
    }

    let mut out: Vec<String> = vec![];
    drop(tx);

    for (ip_addr, u16) in rx {
        out.push(format!("{}:{}", ip_addr, u16))
    }

    out.sort();
    sp.stop();
    //Empty space to split loader from output
    println!("\n");

    if out.is_empty() {
        println!("Couldn't find any open port ")
    } else {
        for v in out {
            println!("{} is open", v)
        }
    }
}

async fn scan(tx: Sender<(IpAddr, u16)>, start_port: u16, addr: IpAddr) {
    match TcpStream::connect_timeout(
        &SocketAddr::new(addr, start_port),
        Duration::from_millis(400),
    ) {
        Ok(_) => {
            io::stdout().flush().unwrap();
            tx.send((addr, start_port)).unwrap()
        }
        // If the connection is unsuccessful, do nothing. Means port is not open.
        Err(_) => {}
    }
}
