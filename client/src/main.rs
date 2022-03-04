use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;
use std::thread;

fn main() {
    const REMOTE_HOST: &str = "SomeHost";
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
        let buf = Vec::from(msg);
        stream.write_all(&buf).unwrap();
    }
}