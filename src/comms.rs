use once_cell::sync::Lazy;
use std::sync::RwLock;

pub static SHARED: Lazy<Comms> = Lazy::new(Comms::default);

#[derive(Debug)]
pub struct Comms {
    pub try_connect: RwLock<bool>,
    pub connected: RwLock<bool>,
    pub server_name: RwLock<String>,
}

impl Default for Comms {
    fn default() -> Self {
        Self {
            try_connect: RwLock::new(false),
            connected: RwLock::new(false),
            server_name: RwLock::new("catornot-test".to_string()),
        }
    }
}
