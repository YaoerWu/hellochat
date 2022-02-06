use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;
use std::thread;

const REMOTE_HOST: &str = "127.0.0.1:8888";

fn main() {
    let mut stream = TcpStream::connect(REMOTE_HOST).unwrap();
    let mut reader = BufReader::new(stream.try_clone().unwrap());

    thread::spawn(move || loop {
        let mut buf = vec![];
        match reader.read_until(b'}', &mut buf) {
            Err(_) | Ok(0) => {
                println!("Connection reset by remote");
                break;
            }
            Ok(_) => {
                let mut msg = String::from_utf8(buf).expect("Convert failed");
                msg.pop();
                println!("{}", msg);
            }
        }
    });
    loop {
        let mut msg = String::new();
        io::stdin()
            .read_line(&mut msg)
            .expect("Failed to read line");
        msg = msg.trim().to_string();
        msg.push('}');
        stream.write_all(msg.as_bytes()).unwrap();
    }
}
