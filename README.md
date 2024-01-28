# SYN Port Scanner

This SYN Scan Tool is a Rust-based command-line utility for performing SYN scans on a specified range of IP addresses and ports. SYN scanning, also known as half-open scanning, is a popular technique used in network security to identify open ports on a network interface.

## Features

- SYN scan for detecting open ports
- Supports multiple target IP addresses
- Option to specify a range of ports or a single port
- Verbose mode for detailed output
- Customizable network interface selection
- Asyn for improved performance

## Usage

```bash
Usage: load-tester [OPTIONS] --url <URL> -n <NUMBER> -c <CONCURRENCY>

Options:
  -t, --target-ip <TARGET_IP>    Target host
  -p, --port <PORT>              Target port
  -s, --start-port <START_PORT>  Target start port [default: 1]
  -e, --end-port <END_PORT>      Target end port [default: 65535]
  -i, --interface <INTERFACE>    Target end port [default: en0]
  -v, --verbose                  Target end port
  -h, --help                     Print help
  -V, --version                  Print version


 ✘ sudo cargo run -- -t 13.107.42.14 -p 443;
```
