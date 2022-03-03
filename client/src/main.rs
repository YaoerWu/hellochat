use std::io;
use tokio::net::TcpStream;

const REMOTE_HOST: &str = "127.0.0.1:8888";

#[tokio::main]
async fn main() {
    let stream = TcpStream::connect(REMOTE_HOST).await.unwrap();
    let (reader, writer) = stream.into_split();

    tokio::spawn(async move {
        loop {
            let mut buf = vec![];
            match reader.try_read(&mut buf) {
                Err(_) | Ok(0) => {
                    println!("Connection reset by remote");
                    break;
                }
                Ok(_) => {
                    let msg = buf;
                    println!("{}", String::from_utf8(msg).unwrap().trim());
                }
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
        writer.try_write(&buf).unwrap();
    }
}
