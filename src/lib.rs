use rrplug::prelude::*;
use std::sync::mpsc::{channel, Sender};

mod window_backup4;
mod comms;

use comms::Comms;
use crate::window_backup4::init_window;

#[derive(Debug)]
struct ProximityChat {
    send: Option<Sender<Comms>>,
}

impl Plugin for ProximityChat {
    fn new() -> Self {
        Self {
            send: None,
        }
    }

    fn initialize(&mut self, _plugin_data: &PluginData) {
        let (send, recv) = channel::<Comms>();
        self.send = Some(send);

        // init_window( recv );
        
        log::info!("starting a second window");
        std::thread::spawn( move || init_window( recv ) );
    }

    fn main(&self) {
        let send = self.send.as_ref().unwrap();

        let mut comms = Comms::default();

        loop {
            comms.x = 0;
            comms.y = 0;
            comms.z = 0;
            
            let _ = send.send(comms);

            wait(3000);
        }
    }
}

unsafe impl Sync for ProximityChat {
    
}

entry!(ProximityChat);

// goodies
// https://github.com/emma-miler/NorthstarPluginLibrary/blob/main/NorthstarPluginLibrary/lib/plugin_abi.h
