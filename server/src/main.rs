use std::collections::HashMap;
use tokio::net::{tcp::OwnedWriteHalf, TcpListener};
use tokio::sync::mpsc;

enum Message {
    Connection(usize, OwnedWriteHalf),
    ConnectionReset(usize),
    Client(usize, String),
}

#[tokio::main]
async fn main() {
    const LOCAL_HOST: &str = "SomeIPAddr";
    //Initialzation
    let listener = TcpListener::bind(LOCAL_HOST)
        .await
        .expect("Fail to bind socket");
    println!("Listener up and running");
    let (tx, mut rx) = mpsc::channel(1);

    //Connection controller
    tokio::spawn(async move {
        let mut client_id = 0;
        let mut get_client_id = || -> usize {
            client_id += 1;
            client_id
        };
        if let Ok((stream, _)) = listener.accept().await {
            let (read_stream, write_stream) = stream.into_split();
            let client_id = get_client_id();
            if tx
                .send(Message::Connection(client_id, write_stream))
                .await
                .is_err()
            {
                panic!();
            }

            let client_tx = tx.clone();

            //TCP reader
            tokio::spawn(async move {
                loop {
                    read_stream.readable().await.unwrap();
                    let mut buf = vec![];
                    match read_stream.try_read(&mut buf) {
                        Err(_) | Ok(0) => {
                            if client_tx
                                .send(Message::ConnectionReset(client_id))
                                .await
                                .is_err()
                            {
                                panic!("Connection Reset");
                            };
                            break;
                        }
                        Ok(_) => {
                            if client_tx
                                .send(Message::Client(client_id, String::from_utf8(buf).unwrap()))
                                .await
                                .is_err()
                            {
                                panic!("Unreachable message");
                            }
                        }
                    };
                }
            });
        }
    });

    let mut client_map = HashMap::new();

    //Initialized
    loop {
        handle_message(
            if let Some(msg) = rx.recv().await {
                msg
            } else {
                panic!("");
            },
            &mut client_map,
        )
        .await;
    }
}

async fn handle_message(msg: Message, client_map: &mut HashMap<usize, OwnedWriteHalf>) {
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
                let (client_id, write_stream) = client;
                if *client_id == id {
                    continue;
                } else {
                    let msg = format!("client {} : ", id) + msg;
                    let mut buf = Vec::from(msg);
                    buf.push(0xa);
                    loop {
                        write_stream.writable().await.unwrap();
                        write_stream
                            .try_write(&buf)
                            .expect("Failed to write message");
                        println!("Message sent to client : {}", client_id);
                    }
                }
            }
        }
    }
}
