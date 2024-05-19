use std::fs::{create_dir, File, OpenOptions};
use std::io::{self, BufReader, Read, Write};
use std::num::ParseIntError;
use std::path::PathBuf;

use pnet::datalink::Channel::Ethernet;
use pnet::datalink::{self, interfaces, Config};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_path = PathBuf::from("/home/mateusz/.config/meter/data_backup.txt");
    let mut bytes: u64;

    if file_path.exists() {
        bytes = match read_data(file_path) {
            Ok(value) => value,
            Err(e) => {
                println!("File reading error: {}", e);
                0
            }
        };
    } else {
        bytes = 0;
    }

    print!("{}", bytes);

    let config = Config {
        read_buffer_size: 4096,
        ..Default::default()
    };

    let interfaces = interfaces();
    let used_interface = interfaces.iter().find(|e| e.is_up() && e.name == "wlan0"); //set it to your interface  iw dev

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
                save_data_usage_info(bytes);
                bytes += packet.len() as u64;
                let megabytes = bytes_to_megabytes(bytes);
                let gigabytes = megabytes / 1000.0;
                if megabytes > 999.9 {
                    print!("\rData used: {:.3} GB", gigabytes);
                } else {
                    print!("\rData used: {:.3} MB", megabytes);
                }

                io::stdout().flush()?;
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

fn save_data_usage_info(bytes: u64) {
    let directory = PathBuf::from("/home/mateusz/.config/meter");
    let file_path = directory.join("data_backup.txt");
    let data = bytes.to_string();

    if !directory.exists() {
        if let Err(e) = create_dir(&directory) {
            println!("Dir creating error: {}", e);
            return;
        }
    }

    let mut file = match OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&file_path)
    {
        Ok(file) => file,
        Err(e) => {
            println!("File opening error: {}", e);
            return;
        }
    };

    if let Err(e) = write!(file, "{}", data) {
        println!("File writing error: {}", e);
    }
}

fn read_data(path: PathBuf) -> Result<u64, ParseIntError> {
    let file = File::open(path);
    let mut buf_reader = BufReader::new(file.unwrap());
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents).unwrap();
    print!("{:?}", contents.trim());
    contents.parse::<u64>()
}
