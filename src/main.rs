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
            for port in start_port..end_port {
                let tx = tx.clone();

                task::spawn(async move { scan(tx, port, target).await });
            }
        }
        cli_parser::ParseCliOutput::WithTarget { target, port } => {
            let tx = tx.clone();
            scan(tx, port, target).await
        }
    }

    let mut out: Vec<u16> = vec![];
    drop(tx);

    for port in rx {
        out.push(port)
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

async fn scan(tx: Sender<u16>, start_port: u16, addr: IpAddr) {
    match TcpStream::connect_timeout(
        &SocketAddr::new(addr, start_port),
        Duration::from_millis(400),
    ) {
        Ok(_) => {
            io::stdout().flush().unwrap();
            tx.send(start_port).unwrap()
        }
        // If the connection is unsuccessful, do nothing. Means port is not open.
        Err(_) => {}
    }
}
