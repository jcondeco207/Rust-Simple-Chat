use std::io::{self, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc::{self, TryRecvError};
use std::thread;
use std::time::Duration;

const LOCAL: &str = "127.0.0.1:3000";
const MSG_SZ: usize = 32;

fn main() {
    let mut client = TcpStream::connect(LOCAL).expect("Failed to connect to server");
    client.set_nonblocking(true).expect("Failed to initiate non-blocking");
    
    let (tx, rx) = mpsc::channel::<String>();

    thread::spawn(move || loop{
        let mut buff = vec![0; MSG_SZ];
        match client.read_exact(&mut buff){
            Ok(_) => {
                let msg = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                println!("message: {:?}", msg);

            },
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_)=> {
                println!("Connection lost");
                break;
            }
        }

        match rx.try_recv(){
            Ok(msg)=>{
                let mut buff = msg.clone().into_bytes();
                buff.resize(MSG_SZ, 0);
                client.write_all(&buff).expect("Writing to socket failed");
            },
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => break
        }
        thread::sleep(Duration::from_millis(100));
    });

    println!("Your message:");
    loop{
        let mut buff = String::new();
        io::stdin().read_line(&mut buff).expect("Failed to read your msg");
        let msg = buff.trim().to_string();
        if msg == "/Q" || tx.send(msg).is_err(){break}
    }
    println!("Shutting down")
}
