use rrplug::prelude::*;
use std::env;

mod bindings;
mod client;
mod connect_hook;
mod server;
mod shared;

use crate::{
    bindings::{EngineFunctions, ENGINE_FUNCTIONS},
    connect_hook::setup_connect_hook,
    shared::ProximityChatType,
};

#[derive(Debug)]
pub struct ProximityChat {
    proximity_chat: ProximityChatType,
}

impl Plugin for ProximityChat {
    fn new(_plugin_data: &PluginData) -> Self {
        // log::info!("starting a second window");
        // std::thread::spawn(move || init_window(send));

        Self {
            proximity_chat: env::args()
                .filter(|cmd| cmd == "-dedicated")
                .last()
                .is_some()
                .into(),
        }
    }

    fn main(&self) {
        loop {
            self.proximity_chat.run_thread();
        }
    }

    fn on_engine_load(&self, engine: &EngineLoadType, dll_ptr: DLLPointer) {
        unsafe { EngineFunctions::try_init(&dll_ptr, &ENGINE_FUNCTIONS) };

        match *engine {
            EngineLoadType::Engine(_) => {}
            EngineLoadType::Client if !self.proximity_chat.is_server() => setup_connect_hook(),
            _ => {}
        }
    }

    fn runframe(&self) {
        self.proximity_chat.run();
    }
}

entry!(ProximityChat);
