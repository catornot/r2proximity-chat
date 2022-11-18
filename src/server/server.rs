use comms::Comms;
pub use eframe;
use log::{debug, error, warn};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::{sync::mpsc::channel, thread::spawn};
use terminal::init_terminal;

mod comms;
mod terminal;
mod utils;

use self::utils::*;

#[derive(Debug, Clone)]
struct Locks {
    connected: Arc<Mutex<Vec<String>>>,
}
// TODO: rewrete this like this https://stackoverflow.com/questions/60678078/rust-tcp-socket-server-only-working-with-one-connection

fn main() {
    let (send, recv) = channel::<Comms>();

    spawn(move || init_terminal(recv));

    let listener: TcpListener = match TcpListener::bind("localhost:7888") {
        // "127.0.0.1:7888"
        Err(err) => {
            error!("port isn't available {}", err);
            return;
        }
        Ok(listener) => listener,
    };

    let locks = Locks {
        connected: Arc::new(Mutex::new(vec![String::from("a")])),
    };

    let l = locks.clone();
    spawn(move || manager(send, l));

    for stream in listener.incoming() {
        let stream = match stream {
            Ok(stream) => stream,
            Err(err) => {
                warn!("somone attempted to join but got the following error");
                warn!("{:?}", err);
                continue;
            }
        };

        let l = locks.clone();
        spawn(move || handle_connection(stream, l));
    }
}

fn manager(send: Sender<Comms>, locks: Locks) {
    let mut comms = Comms::new();
    loop {
        if let Ok(connected) = locks.connected.try_lock() {
            comms.connected = connected.to_vec();
            // debug!("connected {:?}",connected)
        }

        send.send(comms.clone()).expect("guys server is down");

        wait(DEFAULT_WAIT)
    }
}

fn handle_connection(mut stream: TcpStream, locks: Locks) {
    debug!("CONNECTION ESTABLISHED");
    loop {
        let mut buffer = [0; 1024];

        _ = match stream.write(&[0_u8]) {
            Ok(u) => u,
            Err(err) => {
                log_err(err);
                return;
            }
        };
        match stream.flush() {
            Ok(_) => {}
            Err(err) => {
                log_err(err);
                return;
            }
        }

        let _read = match stream.read(&mut buffer) {
            Ok(u) => u,
            Err(err) => {
                log_err(err);
                return;
            }
        };

        if let Ok(s) = String::from_utf8(buffer.to_vec()) {
            if s.starts_with("NAME") {
                loop {
                    if let Ok(mut connected) = locks.connected.try_lock() {
                        let s = s[5..].to_string();
                        let string = match s.split_once('\0') {
                            Some(s) => s.0.to_string(),
                            None => s,
                        };
                        debug!("name added {:?}", &string);
                        connected.push(string);
                        break;
                    }
                    wait(DEFAULT_WAIT)
                }
            }
        }

        wait(DEFAULT_WAIT)
    }
}
