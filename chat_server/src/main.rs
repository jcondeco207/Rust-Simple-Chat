use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

const LOCAL: &str = "127.0.0.1:3000";
const MSG_SZ: usize = 32;

fn main() {
    let server = TcpListener::bind(LOCAL).expect("Listener failed to bind");
    server
        .set_nonblocking(true)
        .expect("Failed to initialize non-blocking");

    let mut clients = vec![];

    // Sender and Receiver for the app
    let (tx, rx) = mpsc::channel::<String>();
    println!("[ ATTENTION ]: Server accepting connections on {}", LOCAL);
    loop {
        // New Connection
        if let Ok((mut socket, addr)) = server.accept() {
            println!("[ ATTENTION ]: {} connected", addr);
            let tx = tx.clone();
            clients.push(
                socket
                    .try_clone()
                    .expect("[ ERROR ]: Failed to clone client"),
            );

            thread::spawn(move || loop {
                let mut buff = vec![0; MSG_SZ];
                match socket.read(&mut buff) {
                    Ok(bytes_read) => {
                        let msg = buff.into_iter().take(bytes_read).collect::<Vec<_>>();
                        if !msg.is_empty() {
                            let msg_str = String::from_utf8_lossy(&msg);
                            println!("{}: {:?}", addr, msg_str);
                            tx.send(msg_str.to_string())
                                .expect("[ ERROR ]: Failed to send message to rx");
                        }
                    }
                    Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                    Err(_) => {
                        println!("[ ATTENTION ]: Closing connection with {}", addr);
                        break;
                    }
                }
                sleep();
            });
        }

        // New Message
        if let Ok(msg) = rx.try_recv() {
            clients = clients
                .into_iter()
                .filter_map(|mut client| {
                    let mut buff = msg.clone().into_bytes();
                    buff.resize(MSG_SZ, 0);
                    client.write_all(&buff).map(|_| client).ok()
                })
                .collect::<Vec<_>>();
        }
        sleep();
    }
}

fn sleep() {
    thread::sleep(Duration::from_millis(100));
}
