use once_cell::sync::Lazy;
use rrplug::bindings::squirrelclasstypes::ScriptContext_CLIENT;
use rrplug::prelude::*;
use rrplug::wrappers::northstar::ScriptVmType;
use rrplug::wrappers::vector::Vector3;
use rrplug::{sq_return_null, sqfunction, to_sq_string};
use std::collections::HashMap;
use std::sync::mpsc::{channel, Sender};
use std::sync::RwLock;

mod comms;
mod discord_client;
mod window_backup4;

use crate::window_backup4::init_window;
use comms::Comms;

use crate::discord_client::DiscordClient;

static PLAYER_POS: Lazy<RwLock<HashMap<String, Vector3>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

static mut DISCORD: Lazy<DiscordClient> = Lazy::new(|| DiscordClient::new());

#[derive(Debug)]
struct ProximityChat {
    send: Option<Sender<Comms>>,
    valid_cl_vm: RwLock<bool>,
}

impl Plugin for ProximityChat {
    fn new() -> Self {
        Self {
            send: None,
            valid_cl_vm: RwLock::new(false),
        }
    }

    fn initialize(&mut self, plugin_data: &PluginData) {
        _ = plugin_data.register_sq_functions(info_push_player_pos);
        _ = plugin_data.register_sq_functions(info_nothing00909);

        let (send, recv) = channel::<Comms>();
        self.send = Some(send);

        log::info!("starting a second window");
        std::thread::spawn(move || init_window(recv));

        log::info!("setting up discord stuff");
        let client = unsafe { &DISCORD };
        client.try_setup();

        loop {
            if let Ok(lock) = client.token.read() {
                if lock.is_some() {
                    break;
                }
            }
            wait(1000)
        }
    }

    fn main(&self) {
        let sq_functions = loop {
            if let Some(sf) = unsafe { SQFUNCTIONS.client.as_ref() } {
                break sf;
            }
            wait(10000)
        };

        let client = unsafe { &mut DISCORD };

        let _send = self.send.as_ref().unwrap();

        // let mut comms = Comms::default();

        loop {
            _ = client.tick();

            if let Ok(lock) = self.valid_cl_vm.read() {
                if !*lock {
                    continue;
                }
            }

            if let Ok(lock) = PLAYER_POS.read() {
                log::info!("{lock:?}");
            }

            let func_name = to_sq_string!("CodeCallback_GetPlayersPostion");

            unsafe {
                (sq_functions.sq_schedule_call_external)(
                    ScriptContext_CLIENT,
                    func_name.as_ptr(),
                    nothing00909,
                )
            }

            wait(3000);
        }
    }

    fn on_sqvm_created(
        &self,
        context: northstar::ScriptVmType,
        _sqvm: &'static squirreldatatypes::CSquirrelVM,
    ) {
        if context != ScriptVmType::Client {
            return;
        }
        if let Ok(mut lock) = self.valid_cl_vm.write() {
            *lock = true
        }
    }

    fn on_sqvm_destroyed(&self, _context: ScriptVmType) {
        if let Ok(mut lock) = self.valid_cl_vm.write() {
            *lock = false
        }
    }
}

unsafe impl Sync for ProximityChat {}

entry!(ProximityChat);

#[sqfunction(VM=Client,ExportName=ProxiChatPushPlayersPositions)]
fn push_player_pos(name: String, pos: Vector3) {
    if let Ok(mut lock) = PLAYER_POS.write() {
        _ = lock.insert(name, pos);
    }
    sq_return_null!()
}

#[sqfunction(VM=Client)]
fn nothing00909() {
    sq_return_null!()
}
