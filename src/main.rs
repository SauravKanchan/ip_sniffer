use std::net::{TcpStream, IpAddr};
use std::{process, io};
use std::io::Write; 
use std::sync::mpsc::{channel, Receiver, Sender};
use std::str::FromStr;

#[derive(Debug)]
struct Arguments {
    flag: String,
    ip_addr: IpAddr,
    threads: u16

}

const MAX: u16 = 65535;

impl Arguments {
    fn new(args: &[String]) -> Result<Arguments, &'static str> {
        if args.len() < 2 {
            // IpAddr from 127.0.0.1
            let ip_addr = IpAddr::from_str("127.0.0.1").unwrap();
            return Ok(Arguments { flag: "".to_string(), ip_addr: ip_addr, threads: 4 });
        } else if args.len() > 4 {
            return Err("Too many arguments");
        }

        let f = args[1].clone();

        if let Ok(ip_addr) = IpAddr::from_str(&f) {
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
                let ip_addr = match IpAddr::from_str(&args[3]) {
                    Ok(s) => s,
                    Err(_) => return Err("not a valid IPADDR; must be IPv4 or IPv6"),
                };
                return Ok(Arguments {threads, flag, ip_addr});
            } else {
                return Err("Invalid syntax");
            }
        }
    }
}

fn scan(tx: Sender<u16>, start_port: u16, addr: IpAddr, num_threads: u16) {
    let mut port: u16 = start_port + 1;
    loop {
        match TcpStream::connect((addr, port)) {
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
