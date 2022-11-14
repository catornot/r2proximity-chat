use std::time::Duration;

pub const DEFAULT_WAIT: u64 = 100;

pub fn wait(milliseconds: u64) {
    std::thread::sleep(Duration::from_millis(milliseconds))
}