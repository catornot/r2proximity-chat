use once_cell::sync::Lazy;
use std::sync::RwLock;

pub static SHARED: Lazy<SharedComms> = Lazy::new(SharedComms::default);

#[derive(Debug)]
pub struct SharedComms {
    pub try_connect: RwLock<bool>,
    pub connected: RwLock<bool>,
    pub server_name: RwLock<String>,
}

impl Default for SharedComms {
    fn default() -> Self {
        Self {
            try_connect: RwLock::new(false),
            connected: RwLock::new(false),
            server_name: RwLock::new("catornot-test".to_string()),
        }
    }
}

#[derive(Debug,Default)]
pub struct SendComms {
    pub mute: bool,
    pub deaf: bool
}