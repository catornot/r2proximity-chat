use once_cell::sync::Lazy;
use std::sync::RwLock;

pub static SHARED: Lazy<SharedComms> = Lazy::new(SharedComms::default);

#[derive(Debug)]
pub struct SharedComms {
    pub connected: RwLock<bool>,
    pub server_name: RwLock<String>,
    pub members: RwLock<Vec<String>>,
}

impl Default for SharedComms {
    fn default() -> Self {
        Self {
            connected: RwLock::new(false),
            server_name: RwLock::new("catornot-test".to_string()),
            members: RwLock::new(vec![]),
        }
    }
}

#[derive(Debug,Default)]
pub struct SendComms {
    pub mute: bool,
    pub deaf: bool
}