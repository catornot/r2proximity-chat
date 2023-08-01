use std::{
    io::{Read, Write},
    mem::size_of,
    net::{TcpListener, TcpStream},
    process::Command,
};

use crate::{
    bindings::uid_exits,
    shared::{AudioSampleVec, NetPacket, ProxiChatError, AUDIO_BUFFER_SIZE, PROXICHAT_PORT},
};

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug)]
enum UIDState {
    UID(i32),
    None,
    AuthReady(i32),
}

#[derive(Debug)]
struct ClientConnection {
    stream: TcpStream,
    audio_buffer: AudioSampleVec,
    send_audio_buffer: AudioSampleVec,
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
                        audio_buffer: vec![0; AUDIO_BUFFER_SIZE],
                        send_audio_buffer: vec![0; AUDIO_BUFFER_SIZE],
                        read_buffer: vec![0; size_of::<NetPacket>()],
                        player_uid: UIDState::None,
                    })
                }
                Err(err) => log::error!("failed to set nonblocking: {err}"),
            },
            Err(err) => {
                log::warn!("connection failed because of {err}");
            }
        }

        if let Some((i, err)) = self
            .connections
            .iter_mut()
            .enumerate()
            .map_while(|(i, conn)| Some((i, handle_collecting_packets(conn).err()?)))
            .filter(|(_, err)| !err.is_would_block_error())
            .last()
        {
            log::error!("{err}");
            log::info!("terminating the connection");
            self.connections.remove(i);
        }

        // TODO: proccess audio

        if let Some((i, err)) = self
            .connections
            .iter_mut()
            .enumerate()
            .map_while(|(i, conn)| Some((i, handle_sending_packets(conn).err()?)))
            .filter(|(_, err)| !err.is_would_block_error())
            .last()
        {
            log::error!("{err}");
            log::info!("terminating the connection");
            self.connections.remove(i);
        }
    }
}

fn handle_collecting_packets(client: &mut ClientConnection) -> Result<(), ProxiChatError> {
    client.read_buffer.clear();

    if client.stream.read(&mut client.read_buffer)? != size_of::<NetPacket>() {
        log::warn!("incorrect amount of bytes received!");
    }

    let packet: NetPacket = bincode::deserialize(&client.read_buffer)?;

    match packet {
        NetPacket::Auth { uid } => {
            if uid_exits(uid) {
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
    } else {
        let mut buf: Vec<u8> = vec![0; AUDIO_BUFFER_SIZE]; // can't avoid 2 or 1 clones/moves
        client.send_audio_buffer.clone_into(&mut buf);
        NetPacket::ProccessedAudio(
            buf.try_into()
                .map_err(|_| ProxiChatError::VecToArrayError)?,
        )
    };

    let buf = bincode::serialize(&packet)?;
    client.send_audio_buffer.write_all(&buf)?;

    Ok(())
}
