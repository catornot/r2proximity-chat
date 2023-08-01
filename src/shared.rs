use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;
use std::mem::size_of;
use thiserror::Error;

use crate::{client::Client, server::Server};

pub const PROXICHAT_PORT: usize = 8081;
pub const AUDIO_BUFFER_SIZE: usize = 128;
pub const READ_BUFFER_SIZE: usize = size_of::<NetPacket>();
pub const DEFAULT_FILL_SAMPLE: AudioSampleType = 0.;
pub type AudioSampleType = f32;
pub type AudioSampleVec = Vec<AudioSampleType>;

#[derive(Debug)]
pub enum ProximityChatType {
    Server(Mutex<Server>),
    Client(Mutex<Client>),
}

impl ProximityChatType {
    pub fn is_server(&self) -> bool {
        matches!(self, Self::Server(_))
    }

    pub fn run(&self) {
        match self {
            ProximityChatType::Server(s) => s.lock().run(),
            ProximityChatType::Client(_) => {},
        }
    }

    pub fn run_thread(&self) {
        match self {
            ProximityChatType::Server(_) => {},
            ProximityChatType::Client(c) => c.lock().run_thread(),
        }
    }
}

impl From<bool> for ProximityChatType {
    fn from(is_server: bool) -> Self {
        if is_server {
            Self::Server(Mutex::new(Server::default()))
        } else {
            Self::Client(Mutex::new(Client::default()))
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum NetPacket {
    Auth {
        uid: i64,
    },
    AuthComfirm,
    #[serde(with = "BigArray")]
    NewAudio([AudioSampleType; AUDIO_BUFFER_SIZE]),
    #[serde(with = "BigArray")]
    ProccessedAudio([AudioSampleType; AUDIO_BUFFER_SIZE]),
    None,
}

#[derive(Error, Debug)]
pub enum ProxiChatError {
    #[error("a packet shouldn't be received as a client")]
    ImpossibleOnClient,

    #[error("a packet shouldn't be received as a server")]
    ImpossibleOnServer,

    #[error("a client tried to connect with a invalid uid: {0}")]
    InvalidUID(i64),

    #[error("a vec wasn't converted to an array")]
    VecToArrayError,

    #[error(transparent)]
    SocketError(#[from] std::io::Error),

    #[error(transparent)]
    BindCodeError(#[from] bincode::Error),

    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),
}

impl ProxiChatError {
    pub fn is_would_block_error(&self) -> bool {
        match self {
            ProxiChatError::SocketError(err) => {
                if matches!(err.kind(), std::io::ErrorKind::WouldBlock) {
                    log::info!("std::io::ErrorKind::WouldBlock");
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

pub fn log_mark_error<T: std::error::Error + core::fmt::Display>(msg: &str, err: T) -> T {
    log::info!("{msg}: {err}");
    err
}