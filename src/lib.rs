use rrplug::prelude::*;
use std::sync::mpsc::{channel, Sender};

mod window_backup4;
mod comms;

use comms::Comms;
use crate::window_backup4::init_window;
struct ProximityChat {
    gamestate: Option<GameState>,
    serverinfo: Option<ServerInfo>,
    playerinfo: Option<PlayerInfo>,
    send: Option<Sender<Comms>>,
}

impl Plugin for ProximityChat {
    fn new() -> Self {
        Self {
            gamestate: None,
            serverinfo: None,
            playerinfo: None,
            send: None,
        }
    }

    fn initialize(&mut self, external_plugin_data: ExternalPluginData) {
        self.gamestate = external_plugin_data.get_game_state_struct();
        self.serverinfo = external_plugin_data.get_server_info_struct();
        self.playerinfo = external_plugin_data.get_player_info_struct();

        println!("rust plugin initialized");

        let (send, recv) = channel::<Comms>();
        self.send = Some(send);

        // init_window( recv );
        
        println!("starting a second window");
        std::thread::spawn( move || init_window( recv ) );
    }

    fn main(&self) {
        let gamestate = self.gamestate.as_ref().unwrap();
        let _playerinfo = self.playerinfo.as_ref().unwrap();
        let _serverinfo = self.serverinfo.as_ref().unwrap();
        let send = self.send.as_ref().unwrap();

        let mut comms = Comms::default();

        loop {
            comms.x = gamestate.our_score();
            comms.y = gamestate.highest_score();
            comms.z = gamestate.second_highest_score();
            
            let _ = send.send(comms);

            wait(3000)
        }
    }
}

entry!(ProximityChat);

// goodies
// https://github.com/emma-miler/NorthstarPluginLibrary/blob/main/NorthstarPluginLibrary/lib/plugin_abi.h
