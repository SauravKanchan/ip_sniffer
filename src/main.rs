use std::net::{IpAddr, TcpStream};
use std::{process, io};
use std::io::{Write}; 
use std::sync::mpsc::{channel, Receiver, Sender};

struct Arguments {
    flag: String,
    ip_addr: String,
    threads: u16

}

const MAX: u16 = 65535;

impl Arguments {
    fn new(args: &[String]) -> Result<Arguments, &'static str> {
        if args.len() < 2 {
            return Ok(Arguments { flag: "".to_string(), ip_addr: "127.0.0.1".to_string(), threads: 4 });
        } else if args.len() > 4 {
            return Err("Too many arguments");
        }

        let f = args[1].clone();

        if let Ok(ip_addr) = args[2].parse() {
            return Ok(Arguments { flag: f, ip_addr, threads: 4 });
        } else {
            let flag = args[1].clone();
            if flag.contains("-h") || flag.contains("--help") && args.len() == 2 {
                println!("Usage: -j to select how many threads you want
                \r\n -h or --help to show this help message");
                return Err("help");
            } else if flag.contains("-h") || flag.contains("--help") {
                return Err("Too many arguments");
            } else if flag.contains("-j") {
                let threads = match args[2].parse::<u16>() {
                    Ok(t) => t,
                    Err(_) => return Err("Failed to parse thread number")
                };
                let ip_addr = match args[3].parse::<String>() {
                    Ok(ip) => ip,
                    Err(_) => return Err("Failed to parse IP address")
                };
                return Ok(Arguments {threads, flag, ip_addr});
            } else {
                return Err("Invalid syntax");
            }
        }
    }
}

fn scan(tx: Sender<u16>, start_port: u16, addr: String, num_threads: u16) {
    let mut port: u16 = start_port + 1;
    loop {
        match TcpStream::connect((addr.as_str(), port)) {
            Ok(_) => {
                print!(".");
                io::stdout().flush().unwrap();
                tx.send(port);
            }
            Err(_) => {}
        }
        if (MAX - port) <= num_threads {
            break;
        }
        port += num_threads;
    }
}

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    let program = args[0].clone();
    let arguments = Arguments::new(&args).unwrap_or_else( |err| {
        if err.contains("help") {
            process::exit(0);
        } else {
            eprintln!("{} problem parsing arguments: {}", program, err);
            process::exit(0);
        }
    });

    let num_threads = arguments.threads;
    let (tx, rx): (Sender<u16>, Receiver<u16>) = channel(); // tx = transmitter, rx = receiver
    for i in 0..num_threads {
        let tx = tx.clone();
        let ip_addr_clone = arguments.ip_addr.clone(); // Clone `ip_addr` for each iteration
        std::thread::spawn(move || {
            scan(tx, i, ip_addr_clone, num_threads);
        });
    }

    let mut out = vec![];
    drop(tx);
    for p in rx {
        out.push(p);
    }   
    println!("");
    for v in out {
        println!("{} is open", v);
    }
}
