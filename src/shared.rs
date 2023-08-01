use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;
use thiserror::Error;

use crate::{client::Client, server::Server};

pub const PROXICHAT_PORT: usize = 8081;
pub const AUDIO_BUFFER_SIZE: usize = 512;
pub type AudioSampleType = u8;
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
            ProximityChatType::Client(c) => c.lock().run(),
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
        uid: i32,
    },
    AuthComfirm,
    #[serde(with = "BigArray")]
    NewAudio([AudioSampleType; AUDIO_BUFFER_SIZE]),
    #[serde(with = "BigArray")]
    ProccessedAudio([AudioSampleType; AUDIO_BUFFER_SIZE]),
}

#[derive(Error, Debug)]
pub enum ProxiChatError {
    #[error("a packet shouldn't be received as a client")]
    ImpossibleOnClient,

    #[error("a packet shouldn't be received as a server")]
    ImpossibleOnServer,

    #[error("a client tried to connect with a invalid uid: {0}")]
    InvalidUID(i32),

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
                matches!(err.kind(), std::io::ErrorKind::WouldBlock)
            }
            _ => false,
        }
    }
}
