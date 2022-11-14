use comms::Comms;
pub use eframe;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::{sync::mpsc::channel, thread::spawn};
use window::init_window;

mod comms;
mod utils;
mod window;

use self::utils::*;

struct Locks {
    connected: Arc<Mutex<Vec<String>>>,
}

fn main() {
    let (send, recv) = channel::<Comms>();

    spawn(move || init_window(recv));

    let listener: TcpListener = match TcpListener::bind("localhost:7878") {
        // "127.0.0.1:7878"
        Err(err) => {
            println!("port isn't available {}", err);
            return;
        }
        Ok(listener) => listener,
    };

    let locks = Locks {
        connected: Arc::new(Mutex::new(Vec::new())),
    };

    spawn(move || manager(send, locks));

    for stream in listener.incoming() {
        let stream = match stream {
            Ok(stream) => stream,
            Err(err) => {
                println!("somone attempted to join but got the following error");
                println!("{:?}", err);
                continue;
            }
        };

        spawn(move || handle_connection(stream));
    }
}

fn manager(send: Sender<Comms>, locks: Locks) {
    let mut comms = Comms::new();
    loop {
        if let Ok(connected) = locks.connected.try_lock() {
            comms.connected = connected.to_vec()
        }

        send.send(comms.clone()).expect("guys server is down");

        wait(DEFAULT_WAIT)
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];

    match stream.read(&mut buffer) {
        Ok(_) => {}
        Err(_) => {
            _ = stream.write(&[]).unwrap();
            stream.flush().unwrap();
            return;
        }
    }

    if !buffer.is_empty() && buffer[0] == 8_u8 {}

    loop {
        _ = stream.write(&[0_u8]).unwrap();
        stream.flush().unwrap();

        wait(DEFAULT_WAIT)
    }
}
