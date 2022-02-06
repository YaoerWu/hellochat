use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;
use std::thread;

const REMOTE_HOST: &str = "127.0.0.1:8888";

fn main() {
    let mut stream = TcpStream::connect(REMOTE_HOST).unwrap();
    let mut reader = BufReader::new(stream.try_clone().unwrap());

    thread::spawn(move || loop {
        let mut buf = String::new();
        match reader.read_line(&mut buf) {
            Err(_) | Ok(0) => {
                println!("Connection reset by remote");
                break;
            }
            Ok(_) => {
                let msg = buf;
                println!("{}", msg.trim());
            }
        }
    });
    loop {
        let mut msg = String::new();
        io::stdin()
            .read_line(&mut msg)
            .expect("Failed to read line");
        msg = msg.trim().to_string();
        let mut buf = Vec::from(msg);
        buf.push(0xa);
        stream.write_all(&buf).unwrap();
    }
}
