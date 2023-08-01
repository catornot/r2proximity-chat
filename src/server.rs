use std::{
    io::{self, Read, Write},
    net::{TcpListener, TcpStream},
    process::Command,
};

use crate::{
    bindings::uid_exits,
    shared::{
        AudioSampleVec, NetPacket, ProxiChatError, AUDIO_BUFFER_SIZE, DEFAULT_FILL_SAMPLE,
        PROXICHAT_PORT, READ_BUFFER_SIZE, log_mark_error,
    },
};

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, PartialEq)]
enum UIDState {
    UID(i64),
    None,
    AuthReady(i64),
}

#[derive(Debug)]
struct ClientConnection {
    stream: TcpStream,
    audio_buffer: AudioSampleVec,
    send_audio_buffer: AudioSampleVec, // AudioNode<Sample = Float<f32>, Inputs = Size<AUDIO_BUFFER_SIZE>, Outputs = Size<AUDIO_BUFFER_SIZE>, Setting = Type>
    read_buffer: Vec<u8>,
    player_uid: UIDState,
}

#[derive(Debug)]
pub struct Server {
    server: TcpListener,
    connections: Vec<ClientConnection>,
}

impl Default for Server {
    fn default() -> Self {
        let cmd_result = Command::new("ipconfig")
            .output()
            .expect("failed to get ipconfig")
            .stdout;
        let cmd_result = String::from_utf8_lossy(&cmd_result).to_string();
        let addr = cmd_result
            .split('\n')
            .filter(|line| line.contains("  IPv4 Address"))
            .filter_map(|line| line.split(':').nth(1))
            .map(|addr| addr.trim().trim_end())
            .last()
            .expect("couldn't find the machine's ip address");

        let server = TcpListener::bind(format!("{addr}:{PROXICHAT_PORT}"))
            .expect("failed to bind to address : {addr}");

        server
            .set_nonblocking(true)
            .expect("failed to set non_blocking");

        Self {
            server,
            connections: Vec::new(),
        }
    }
}

impl Server {
    /// has to be ran on the tf2 thread aka runframe since it accesses player array
    pub fn run(&mut self) {
        match self.server.accept() {
            Ok((conn, addr)) => match conn.set_nonblocking(true) {
                Ok(_) => {
                    log::info!("connection created with {addr:?}");
                    self.connections.push(ClientConnection {
                        stream: conn,
                        audio_buffer: vec![DEFAULT_FILL_SAMPLE; AUDIO_BUFFER_SIZE],
                        send_audio_buffer: vec![DEFAULT_FILL_SAMPLE; AUDIO_BUFFER_SIZE],
                        read_buffer: vec![0; READ_BUFFER_SIZE],
                        player_uid: UIDState::None,
                    })
                }
                Err(err) => log::error!("failed to connect to a stream: {err}"),
            },
            Err(err) if !matches!(err.kind(), io::ErrorKind::WouldBlock) => {
                log::warn!("connection failed because of {err}");
            }
            _ => {}
        }

        if let Some((i, err)) = self
            .connections
            .iter_mut()
            .enumerate()
            .map_while(|(i, conn)| Some((i, handle_collecting_packets(conn).err()?)))
            .filter(|(_, err)| !err.is_would_block_error())
            .last()
        {
            log::error!("receiving: {err}");
            log::info!("terminating the connection");
            self.connections.remove(i);
        }

        // TODO: proccess audio

        self.connections
            .iter_mut()
            .for_each(|c| c.send_audio_buffer.clear());
        self.connections
            .iter_mut()
            .for_each(|c| c.send_audio_buffer.extend_from_slice(&c.audio_buffer));

        if let Some((i, err)) = self
            .connections
            .iter_mut()
            .enumerate()
            .map_while(|(i, conn)| Some((i, handle_sending_packets(conn).err()?)))
            .filter(|(_, err)| !err.is_would_block_error())
            .last()
        {
            log::error!("sending: {err}");
            log::info!("terminating the connection");
            self.connections.remove(i);
        }
    }
}

fn handle_collecting_packets(client: &mut ClientConnection) -> Result<(), ProxiChatError> {
    client.read_buffer.clear();

    if client.stream.read(&mut client.read_buffer).map_err(|err| log_mark_error("read sv", err))? > READ_BUFFER_SIZE {
        log::warn!("incorrect amount of bytes received!");
        return Ok(());
    }

    let packet: NetPacket = bincode::deserialize(&client.read_buffer).map_err(|err| log_mark_error("deserialize sv", err))?;
    // let packet: NetPacket = match bincode::deserialize(&client.read_buffer).map_err(|err| log_mark_error("deserialize sv", err)) {
    //     Ok(p) => p,
    //     Err(_) => return Ok(()), // ignore all bincode on decode since it can get garbage data by accident
    // };

    match packet {
        NetPacket::Auth { uid } => {
            if uid_exits(uid) {
                log::info!("auth completed with client");
                client.player_uid = UIDState::AuthReady(uid)
            } else {
                Err(ProxiChatError::InvalidUID(uid))?
            }
        }
        NetPacket::NewAudio(audio) => audio
            .into_iter()
            .enumerate()
            .for_each(|(i, e)| client.audio_buffer[i] = e), // not sure if there is a better way; perhaps should be change later
        _ => Err(ProxiChatError::ImpossibleOnServer)?,
    }

    Ok(())
}

fn handle_sending_packets(client: &mut ClientConnection) -> Result<(), ProxiChatError> {
    let packet = if let UIDState::AuthReady(uid) = client.player_uid {
        client.player_uid = UIDState::UID(uid);
        NetPacket::AuthComfirm
    } else if client.player_uid != UIDState::None {
        let mut buf: AudioSampleVec = vec![DEFAULT_FILL_SAMPLE; AUDIO_BUFFER_SIZE]; // can't avoid 2 or 1 clones/moves
        client.send_audio_buffer.clone_into(&mut buf);
        NetPacket::ProccessedAudio(
            buf.try_into()
                .map_err(|_| ProxiChatError::VecToArrayError)?,
        )
    } else {
        NetPacket::None
    };

    let buf = bincode::serialize(&packet)?;
    log::info!(
        "sending buffer of size {} when max size is {}",
        buf.len(),
        READ_BUFFER_SIZE
    );

    client.stream.write_all(&buf)?;
    // let mut size_to_write = buf.len() as i32;
    // while size_to_write <= 0 {
    //     size_to_write -= client.stream.write(&buf)? as i32;
    // }

    Ok(())
}
