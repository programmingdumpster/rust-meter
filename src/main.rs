use std::io::{self, Write};

use pnet::datalink::Channel::Ethernet;
use pnet::datalink::{self, interfaces, Config};

fn main() {
    let mut bytes: u64 = 0;
    let config = Config {
        read_buffer_size: 4096,
        ..Default::default()
    };

    let interfaces = interfaces();
    let used_interface = interfaces.iter().find(|e| e.is_up());

    let data_channel = match used_interface {
        Some(interface) => datalink::channel(interface, config),
        None => panic!("cannot find interface"),
    };

    let (_, mut rx) = match data_channel {
        Ok(Ethernet(_, rx)) => ((), rx),
        Ok(_) => panic!("channel error"),
        Err(e) => panic!("channel creation error {}", e),
    };

    loop {
        match rx.next() {
            Ok(packet) => {
                bytes += packet.len() as u64;
                let megabytes = bytes_to_megabytes(bytes);
                print!("\rData used: {:.3} MB", megabytes);
                io::stdout().flush().unwrap();
            }
            Err(e) => {
                println!("packet read error {}", e);
            }
        }
    }
}

fn bytes_to_megabytes(bytes: u64) -> f64 {
    return bytes as f64 / (1024.0 * 1024.0);
}
