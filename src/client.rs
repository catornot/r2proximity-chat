use cpal::{
    traits::{DeviceTrait, HostTrait},
    InputCallbackInfo, Stream,
};
use rrplug::high::Handle;
use std::{
    io::{Read, Write},
    net::{SocketAddr, TcpStream},
    str::FromStr,
    sync::mpsc::{self, Receiver, Sender},
    time::Duration,
};

use crate::{
    bindings::parse_local_uid,
    shared::{
        AudioSampleType, AudioSampleVec, NetPacket, ProxiChatError, AUDIO_BUFFER_SIZE,
        DEFAULT_FILL_SAMPLE, READ_BUFFER_SIZE,
    },
};

pub struct Client {
    tcp_stream: Option<TcpStream>,
    ouput_stream: Option<Handle<Stream>>,
    input_stream: Option<Handle<Stream>>,
    send_audio: Sender<AudioSampleVec>,
    recv_audio: Receiver<AudioSampleVec>,
    read_buffer: Vec<u8>,
    audio_buffer: AudioSampleVec,
    auth_completed: bool,
    uid: i64,
}

impl Default for Client {
    fn default() -> Self {
        let (sender, recv) = mpsc::channel();

        Self {
            tcp_stream: Default::default(),
            ouput_stream: Default::default(),
            input_stream: Default::default(),
            send_audio: sender,
            recv_audio: recv,
            read_buffer: vec![0; READ_BUFFER_SIZE],
            audio_buffer: vec![DEFAULT_FILL_SAMPLE; AUDIO_BUFFER_SIZE * 4],
            auth_completed: false,
            uid: 0,
        }
    }
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("tcp_stream", &self.tcp_stream)
            .field("ouput_stream", &self.ouput_stream.is_some())
            .field("input_stream", &self.input_stream.is_some())
            .finish()
    }
}

impl Client {
    pub fn set_new_connection(&mut self, addr: String) {
        self.drop_stream();

        self.uid = parse_local_uid().expect("couldn't get local player uid");

        match TcpStream::connect_timeout(
            &SocketAddr::from_str(&addr)
                .expect("provided addrress was somehow invalid (impossible)"),
            Duration::from_secs(1),
        ) {
            Ok(stream) => {
                // stream
                //     .set_nonblocking(true)
                //     .expect("couldn't set non blocking stream");
                self.tcp_stream = stream.into();
            }
            Err(err) => return log::error!("couldn't connect to server: {err}"),
        }

        let host = cpal::default_host();

        // ouput
        let device = host
            .default_output_device()
            .expect("no output device available!");

        let supported_configs_range = device
            .supported_output_configs()
            .expect("error while querying configs");
        let supported_config = supported_configs_range
            .inspect(|c| {
                log::info!(
                    "sample format {}",
                    c.clone().with_max_sample_rate().sample_format()
                )
            })
            .last()
            .expect("no supported config?!")
            .with_max_sample_rate();
        let config = supported_config.config();

        let (sender, recv) = mpsc::channel();
        self.send_audio = sender;

        let ouput_stream = device
            .build_output_stream(
                &config,
                move |data: &mut [AudioSampleType], _: &cpal::OutputCallbackInfo| {
                    for (sample, new_sample) in
                        data.iter_mut().zip(recv.try_recv().unwrap_or_default())
                    {
                        *sample = new_sample;
                    }
                },
                |err| {
                    log::error!("output stream error: {err}");
                },
                None,
            )
            .expect("failed to create a output stream");
        self.ouput_stream = unsafe { Handle::new(ouput_stream) }.into();

        // input
        let device = host
            .default_input_device()
            .expect("no output device available!");

        let mut supported_configs_range = device
            .supported_input_configs()
            .expect("error while querying configs");
        let supported_config = supported_configs_range
            .next()
            .expect("no supported config?!")
            .with_max_sample_rate();
        let config = supported_config.config();

        let (sender, recv) = mpsc::channel();
        self.recv_audio = recv;

        let input_stream = device
            .build_input_stream(
                &config,
                move |data: &[AudioSampleType], _: &InputCallbackInfo| {
                    _ = sender.send(data.to_vec());
                },
                |err| {
                    log::error!("output stream error: {err}");
                },
                None,
            )
            .expect("failed to create a input stream");
        self.input_stream = unsafe { Handle::new(input_stream) }.into();
    }

    pub fn drop_stream(&mut self) {
        _ = self.tcp_stream.take();
        _ = self.ouput_stream.take();
        _ = self.input_stream.take();
        self.auth_completed = false;
        self.audio_buffer.clear();
        self.read_buffer.clear();
    }

    pub fn run_thread(&mut self) {
        // assumptions : if TcpStream is Some then all other fields of Client are Some

        if let Some(stream) = self.tcp_stream.as_mut() {
            if let Ok(data) = self.recv_audio.try_recv() {
                self.audio_buffer.extend(data);
            }

            if let Err(err) = handle_sending(
                stream,
                &mut self.audio_buffer,
                self.auth_completed,
                self.uid,
            ) {
                if !err.is_would_block_error() {
                    log::error!("sending: {err}");
                    log::info!("terminating connection with server!");
                    self.drop_stream();
                    return;
                }
            }

            let audio =
                match handle_receiving(stream, &mut self.read_buffer, &mut self.auth_completed) {
                    Err(err) if !err.is_would_block_error() => {
                        log::error!("receiving: {err}");
                        log::info!("terminating connection with server!");
                        self.drop_stream();
                        return;
                    }
                    Ok(audio) => audio,
                    _ => return,
                };

            if let Some(audio) = audio {
                if let Err(err) = self.send_audio.send(audio) {
                    log::warn!("{err}");
                }
            }
        }
    }
}

fn handle_sending(
    stream: &mut TcpStream,
    audio_buffer: &mut AudioSampleVec,
    auth_completed: bool,
    uid: i64,
) -> Result<(), ProxiChatError> {
    let packet: NetPacket = if auth_completed {
        let mut buf = audio_buffer
            .drain(0..AUDIO_BUFFER_SIZE)
            .collect::<AudioSampleVec>();
        buf.resize(AUDIO_BUFFER_SIZE, DEFAULT_FILL_SAMPLE);

        NetPacket::NewAudio(
            buf.try_into()
                .map_err(|_| ProxiChatError::VecToArrayError)?,
        )
    } else {
        NetPacket::Auth { uid }
    };

    let buf = bincode::serialize(&packet)?; // not very effective -> new alloc each time for nothing

    stream.write_all(&buf)?;

    Ok(())
}

fn handle_receiving(
    stream: &mut TcpStream,
    read_buffer: &mut Vec<u8>,
    auth_completed: &mut bool,
) -> Result<Option<AudioSampleVec>, ProxiChatError> {
    read_buffer.clear();

    let size = stream.read(read_buffer)?;
    // let size = read_until_success(stream, read_buffer, &mut 10)?;
    if size > READ_BUFFER_SIZE {
        log::warn!("incorrect amount of bytes received! found {size} exepected {READ_BUFFER_SIZE}");
        return Ok(None);
    }

    // let packet: NetPacket = bincode::deserialize(read_buffer)?;
    let packet: NetPacket = match bincode::deserialize(read_buffer) {
        Ok(p) => p,
        Err(_) => return Ok(None), // ignore all bincode on decode since it can get garbage data by accident
    };

    log::info!("packet : {packet:?}");

    let audio = match packet {
        NetPacket::AuthComfirm => {
            log::info!("auth completed with server");
            *auth_completed = true;
            None
        }
        NetPacket::None => None,
        NetPacket::ProccessedAudio(audio) => Some(audio.to_vec()),
        _ => Err(ProxiChatError::ImpossibleOnClient)?,
    };

    Ok(audio)
}
