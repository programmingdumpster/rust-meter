use std::fs::{create_dir, File};
use std::io::{self, BufReader, Read, Write};
use std::num::ParseIntError;
use std::path::PathBuf;

use pnet::datalink::Channel::Ethernet;
use pnet::datalink::{self, interfaces, Config};

fn main() {
    let file_path = PathBuf::from("/home/mateusz/.config/meter/data_backup.txt");
    let readed = match read_data(file_path) {
        Ok(data) => data,
        Err(e) => {
            println!("parsing error: {}", e);
            0
        }
    };
    let mut bytes: u64 = readed;

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

fn save_data_usage_info(bytes: u64) {
    let directory = PathBuf::from("/home/mateusz/.config/meter");
    let file_path = PathBuf::from("/home/mateusz/.config/meter/data_backup.txt");
    let data = bytes.to_string();

    if directory.exists() {
    } else {
        match create_dir(directory) {
            Ok(_) => (),
            Err(e) => println!("Dir creating error: {}", e),
        }
    }
    if file_path.exists() {
    } else {
        match File::create(&file_path) {
            Ok(_) => (),
            Err(e) => println!("File creation error: {}", e),
        }
    }

    let mut file = match File::create(&file_path) {
        Ok(file) => file,
        Err(e) => {
            println!("Błąd podczas otwierania pliku: {}", e);
            return;
        }
    };

    if let Err(e) = writeln!(file, "{}", data) {
        println!("File writing error: {}", e);
        return;
    }
}

fn read_data(path: PathBuf) -> Result<u64, ParseIntError> {
    let file = File::open(path);
    let mut buf_reader = BufReader::new(file.unwrap());
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents).unwrap();
    contents.parse::<u64>()
}
