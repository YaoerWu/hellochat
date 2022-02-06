use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::thread;

const LOCAL_HOST: &str = "127.0.0.1:8888";

enum Message {
    Connection(usize, TcpStream),
    ConnectionReset(usize),
    Client(usize, String),
}

fn main() {
    //Initialzation
    let listener = TcpListener::bind(LOCAL_HOST).expect("Fail to bind socket");
    println!("Listener up and running");
    let (tx, rx) = mpsc::channel();

    //Connection controller
    thread::spawn(move || {
        let mut client_id = 0;
        let mut get_client_id = || -> usize {
            client_id += 1;
            client_id
        };
        for stream in listener.incoming() {
            match stream {
                Err(e) => println!("Connection failed: {}", e),
                Ok(stream) => {
                    let client_id = get_client_id();
                    tx.send(Message::Connection(
                        client_id,
                        stream.try_clone().expect("Failed to clone stream"),
                    ))
                    .expect("Failed to send message");
                    let client_tx = tx.clone();

                    //TCP reader
                    thread::spawn(move || {
                        let mut reader = BufReader::new(stream);
                        loop {
                            let mut buf = String::new();
                            match reader.read_line(&mut buf) {
                                Err(_) | Ok(0) => {
                                    client_tx
                                        .send(Message::ConnectionReset(client_id))
                                        .expect("Unreachable message");
                                    break;
                                }
                                Ok(_) => client_tx
                                    .send(Message::Client(client_id, buf))
                                    .expect("Unreachable message"),
                            };
                        }
                    });
                }
            }
        }
    });

    let mut client_map = HashMap::new();

    //Initialized
    loop {
        handle_message(
            rx.recv().expect("Failed to receive message"),
            &mut client_map,
        );
    }
}

fn handle_message(msg: Message, client_map: &mut HashMap<usize, TcpStream>) {
    match msg {
        Message::Connection(client_id, stream) => {
            client_map.insert(client_id, stream);
            println!("client {} connected", client_id);
        }
        Message::ConnectionReset(client_id) => {
            client_map.remove(&client_id);
            println!("client {} disconnected", client_id);
        }
        Message::Client(id, msg) => {
            let msg = msg.trim();
            println!("Message from client {} : {}", id, msg);
            for client in client_map.iter() {
                let (client_id, mut client_stream) = client;
                if *client_id == id {
                    continue;
                } else {
                    let msg = format!("client {} : ", id) + msg;
                    let mut buf = Vec::from(msg);
                    buf.push(0xa);
                    client_stream
                        .write_all(&buf)
                        .expect("Failed to write message");
                    println!("Message sent to client : {}", client_id);
                }
            }
        }
    }
}
