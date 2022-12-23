use std::io::{Error, ErrorKind, Read, Write};
use std::net::TcpStream;

const MAX_ATTEMPTS: u16 = 100;

#[derive(Debug, Default)]
pub struct Client {
    pub name: String,
    conn: Option<TcpStream>,
    attemps: u16,
}

impl Client {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn connect(&mut self, addr: &String) -> Result<(), Error> {
        if self.name.is_empty() {
            return Err(Error::new(ErrorKind::UnexpectedEof, "name is empty"));
        }

        let mut conn = TcpStream::connect(addr)?;

        _ = conn.write(format!("NAME:{}", self.name).as_bytes())?;
        conn.flush()?;

        self.conn = Some(conn);
        Ok(())
    }

    pub fn cancel(&mut self) {
        let conn = self.conn.as_mut();

        if let Some(conn) = conn {
            _ = conn.write(&[]);
            _ = conn.flush();
        }
        self.conn = None;
        self.attemps = 0;
    }

    pub fn has_stream(&self) -> bool {
        self.conn.is_some()
    }

    pub fn run(&mut self) {
        let mut buffer = [0; 1024];

        match self.conn.as_mut().unwrap().read(&mut buffer) {
            Ok(_) => {}
            Err(err) => {
                if self.attemps == u16::MAX {
                    self.cancel();
                }
                self.attemp(err);
                return;
            }
        }

        log::info!("read {:?}", buffer);

        match self.conn.as_mut().unwrap().write(&[1_u8]) {
            Ok(_) => {}
            Err(err) => {
                self.attemp(err);
                return;
            }
        }

        match self.conn.as_mut().unwrap().flush() {
            Ok(_) => {}
            Err(err) => {
                self.attemp(err);
            }
        }
    }

    fn attemp(&mut self, err: Error) {
        self.attemps += 1;

        if self.attemps >= MAX_ATTEMPTS {
            self.cancel()
        }

        log::info!(
            "WARNING: {:?} when reading data attemp: {}",
            err, self.attemps
        );
    }
}
