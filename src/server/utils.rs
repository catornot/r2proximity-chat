use std::time::Duration;

pub const DEFAULT_WAIT: u64 = 1;

pub fn wait(milliseconds: u64) {
    std::thread::sleep(Duration::from_millis(milliseconds))
}

pub fn log_err<T>(err: T)
where
    T: std::fmt::Debug,
{
    log::error!("err :{:?}", err);
}
