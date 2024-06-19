use clap::{Parser, Subcommand};
use pnet::datalink::{self, interfaces, Channel::Ethernet, Config};
use std::fs::{create_dir, File, OpenOptions};
use std::io::{self, BufReader, Read, Write};
use std::num::ParseIntError;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;



#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(alias = "-u")]
    Usage,
    #[command(alias = "-s")]
    Speedtest,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file_path = PathBuf::from("/home/mateusz/.config/meter/data_backup.txt"); //path to file with saved data ( its for logging data usage when app is added to systemctl services (linux))
    let mut bytes: u64;
    let cli = Cli::parse();
    if let Some(command) = cli.command {
        match command {
            Commands::Usage => loop {
                match read_data(file_path.clone()) {
                    Ok(bytes) => {
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
                        println!("read err: {}", e);
                    }
                }
                thread::sleep(Duration::from_secs(5));
            },
            Commands::Speedtest => todo!(), //i will do it (maybe)
        }
    }

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

    let config = Config {
        read_buffer_size: 4096,
        ..Default::default()
    };

    let interfaces = interfaces();
    let used_interface = interfaces.iter().find(|e| e.is_up() && e.name == "wlan0"); //Use "iw dev" in bash, it will return ur interfaces, select one u use and paste it into e.name = ""

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
        // for dummies like me
        // this is raw packet "[0x00, 0x1a, 0x2b, 0x3c, 0x4d, 0x5e, 0x6f]"
        // every entry like 0x00  = 1byte  | this packet has 7 bytes in total,
        match rx.next() {
            Ok(packet) => {
                save_data_usage_info(bytes);
                bytes += packet.len() as u64;
            }
            Err(e) => {
                println!("packet read error {}", e);
            }
        }
    }
}

fn save_data_usage_info(bytes: u64) {
    let directory = PathBuf::from("/home/mateusz/.config/meter"); // set it as your  config dir
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

fn bytes_to_megabytes(bytes: u64) -> f64 {
    bytes as f64 / (1024.0 * 1024.0)
}

fn read_data(path: PathBuf) -> Result<u64, ParseIntError> {
    let file = File::open(path);
    let mut buf_reader = BufReader::new(file.unwrap());
    let mut contents = String::new();
    buf_reader.read_to_string(&mut contents).unwrap();

    contents.parse::<u64>()
}
