mod cli_parser;

use pnet;
use pnet::packet::tcp::MutableTcpPacket;
use pnet::packet::Packet;
use pnet_datalink;
use pnet_datalink::NetworkInterface;
use pnet_transport::{self};
use rand::{self};
use std::panic;
use std::sync::Mutex;
use std::time::Duration;
use std::{
    net::{IpAddr, Ipv4Addr},
    sync::Arc,
};
use tokio::sync::{mpsc, Semaphore};
use tokio::task;
use tokio::time::sleep;
const MAX: u16 = u16::MAX;

/// Strcture for holding all the needed options/parameters

struct Config {
    target_ip: Ipv4Addr,
    interface: NetworkInterface,
    source_ip: Ipv4Addr,
    source_port: u16,
    verbose: bool,
}

/// Creates tcp packet with the SYN flag and given options
fn create_tcp_packet(
    buff: &mut [u8],
    source_port: u16,
    destination_port: u16,
    source_ip: Ipv4Addr,
    target_ip: Ipv4Addr,
) -> MutableTcpPacket {
    let mut packet = MutableTcpPacket::new(buff).unwrap();
    packet.set_flags(pnet::packet::tcp::TcpFlags::SYN);
    packet.set_source(source_port);
    packet.set_destination(destination_port);
    packet.set_window(1024);
    packet.set_data_offset(6);
    packet.set_sequence(rand::random::<u32>());
    packet.set_options(&[pnet::packet::tcp::TcpOption::mss(1460)]);
    packet.set_checksum(pnet::packet::tcp::ipv4_checksum(
        &packet.to_immutable(),
        &source_ip,
        &target_ip,
    ));
    packet
}

async fn send_syn_packets(config: Arc<Config>, start_port: u16) -> Result<(), std::io::Error> {
    let channel_type = pnet_transport::TransportChannelType::Layer4(
        pnet_transport::TransportProtocol::Ipv4(pnet::packet::ip::IpNextHeaderProtocols::Tcp),
    );
    let (mut tx, _) = match pnet_transport::transport_channel(4096, channel_type) {
        Ok((tx, rx)) => (tx, rx),
        Err(e) => return Err(e),
    };
    let destination_port = start_port;
    let source_port: u16 = config.source_port;
    let source_ip = config.source_ip;
    let target_ip = config.target_ip;
    let mut buff: [u8; 24] = [0; 24];

    let packet = create_tcp_packet(
        &mut buff,
        source_port,
        destination_port,
        source_ip,
        target_ip,
    );
    match tx.send_to(packet, IpAddr::V4(target_ip)) {
        Ok(_) => (),
        Err(e) => return Err(e),
    };
    sleep(Duration::from_millis(1)).await;

    Ok(())
}

/// Checks all incoming packets and finds one's that are related to the scan
async fn handle_scan_responses(
    tx: mpsc::Sender<u16>,
    config: Arc<Config>,
    is_finished: Arc<Mutex<bool>>,
) {
    let (_, mut receiver) = match pnet_datalink::channel(&config.interface, Default::default()) {
        Ok(pnet_datalink::Channel::Ethernet(tx, rx)) => (tx, rx),
        Ok(_) => panic!("Wrong channel type"),
        Err(e) => panic!("Error while creating a channel on given interface: {}", e),
    };
    let source_port = config.source_port;
    let target_ip = config.target_ip;
    loop {
        if *is_finished.lock().unwrap() {
            break;
        }
        let buf = receiver.next().unwrap();
        let ethernet = pnet::packet::ethernet::EthernetPacket::new(buf).unwrap();
        let ipv4 = pnet::packet::ipv4::Ipv4Packet::new(ethernet.payload()).unwrap();
        if ipv4.get_next_level_protocol() == pnet::packet::ip::IpNextHeaderProtocols::Tcp {
            let tcp = pnet::packet::tcp::TcpPacket::new(ipv4.payload()).unwrap();
            if !(tcp.get_destination() == source_port && ipv4.get_source() == target_ip) {
                continue;
            } else {
                if tcp.get_flags()
                    == pnet::packet::tcp::TcpFlags::SYN + pnet::packet::tcp::TcpFlags::ACK
                {
                    let open_port = tcp.get_source();
                    if config.verbose {
                        println!("Discovered open port: {}/tcp", &open_port);
                    };
                    match tx.send(open_port).await {
                        Ok(_) => (),
                        Err(e) => panic!("Error while writing to the Sender channel: {e}"),
                    };
                };
            };
        };
    }
}

/// Displays scan results
fn print_results(open_ports: &[u16], target_ip: Ipv4Addr) {
    println!(
        "Stats: {} filtered/closed port(s) (RST or no response), {} open port(s) for {}",
        MAX as usize - open_ports.len(),
        open_ports.len(),
        target_ip
    );
    println!("PORT\tSTATUS");
    for port in open_ports {
        println!("{port}\tOpen");
    }
}

#[tokio::main]
async fn main() {
    let parsed_inputs = cli_parser::parse_cli();
    match parsed_inputs {
        cli_parser::ParseCliOutput::WithRange {
            target_ip,
            start_port,
            end_port,
            verbose,
            interface,
            source_ip,
            source_port,
        } => {
            for target in target_ip {
                sleep(Duration::from_millis(3000)).await;
                scan(
                    Config {
                        target_ip: target,
                        interface: interface.clone(),
                        source_ip,
                        source_port,
                        verbose,
                    },
                    start_port,
                    end_port,
                    false,
                )
                .await
            }
        }
        cli_parser::ParseCliOutput::WithTarget {
            target_ip,
            port,
            verbose,
            interface,
            source_ip,
            source_port,
        } => {
            for target in target_ip {
                sleep(Duration::from_millis(3000)).await;
                scan(
                    Config {
                        target_ip: target,
                        interface: interface.clone(),
                        source_ip,
                        source_port,
                        verbose,
                    },
                    1,
                    port,
                    true,
                )
                .await
            }
        }
    }
}

async fn scan(config: Config, start_port: u16, end_port: u16, single_port: bool) {
    let config: Arc<Config> = Arc::new(config);

    let conf_copy = Arc::clone(&config);
    let (tx, mut rx) = mpsc::channel::<u16>(MAX as usize);
    let scan_finished = Arc::new(Mutex::new(false));
    let scan_finished_copy = Arc::clone(&scan_finished);

    let tx_clone = tx.clone();
    task::spawn(
        async move { handle_scan_responses(tx_clone, conf_copy, scan_finished_copy).await },
    );

    let semaphore = Arc::new(Semaphore::new(20));

    if single_port {
        let conf: Arc<Config> = Arc::clone(&config);
        send_syn_packets(conf, end_port).await.unwrap();
    } else {
        for i in start_port..end_port {
            let conf: Arc<Config> = Arc::clone(&config);
            let permit = Arc::clone(&semaphore).acquire_owned().await.unwrap();

            task::spawn(async move {
                send_syn_packets(conf, i).await.unwrap();
                drop(permit);
            });
        }
    }

    *scan_finished.lock().unwrap() = true;

    let mut open_ports = Vec::new();
    drop(tx);

    while let Some(port) = rx.recv().await {
        open_ports.push(port);
    }

    open_ports.sort();
    print_results(&open_ports, config.target_ip);
}
