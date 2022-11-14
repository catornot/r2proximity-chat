#[derive(Debug, Clone, Default)]
pub struct Comms {
    pub connected: Vec<String>
}

impl Comms {
    pub fn new() -> Self {
        Self {
            connected: Vec::new()
        }
    }
}