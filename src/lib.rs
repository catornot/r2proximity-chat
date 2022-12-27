#![feature(is_some_and)]

use comms::SHARED;
use once_cell::sync::Lazy;
use rrplug::bindings::squirrelclasstypes::ScriptContext_CLIENT;
use rrplug::prelude::*;
use rrplug::wrappers::northstar::ScriptVmType;
use rrplug::wrappers::vector::Vector3;
use rrplug::{sq_return_null, sqfunction, to_sq_string};
use std::collections::HashMap;
use std::sync::RwLock;

mod comms;
mod discord_client;
mod window_backup4;

use crate::window_backup4::init_window;

use crate::discord_client::DiscordClient;

static PLAYER_POS: Lazy<RwLock<HashMap<String, Vector3>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));
static LOCAL_PLAYER: Lazy<RwLock<String>> = Lazy::new(|| RwLock::new("None".to_string()));

static mut DISCORD: Lazy<DiscordClient> = Lazy::new(DiscordClient::new);

#[derive(Debug)]
struct ProximityChat {
    valid_cl_vm: RwLock<bool>,
    // current_server: Option<String>, // how do I get this?
}

impl Plugin for ProximityChat {
    fn new() -> Self {
        Self {
            valid_cl_vm: RwLock::new(false),
            // current_server: None,
        }
    }

    fn initialize(&mut self, plugin_data: &PluginData) {
        _ = plugin_data.register_sq_functions(info_push_player_pos);
        _ = plugin_data.register_sq_functions(info_push_player_name);
        _ = plugin_data.register_sq_functions(info_nothing00909);

        log::info!("starting a second window");
        std::thread::spawn(init_window);

        log::info!("setting up discord stuff");
        let client = unsafe { &DISCORD };
        client.try_setup();
    }

    fn main(&self) {
        let sq_functions = loop {
            if let Some(sf) = unsafe { SQFUNCTIONS.client.as_ref() } {
                break sf;
            }
            wait(10000)
        };

        let client = unsafe { &mut *DISCORD };

        loop {
            wait(1000);

            _ = client.tick();

            if let Ok(lock) = self.valid_cl_vm.read() {
                if !*lock {
                    // log::info!("reseting vc volume");
                    client.reset_vc();
                    continue;
                }
            }

            if SHARED.connected.read().is_ok_and(|x| !*x) {
                continue;
            }

            if let Ok(positions) = PLAYER_POS.read() {
                log::info!("{positions:?}");

                if let Ok(local_player) = LOCAL_PLAYER.read() {
                    if let Some(local) = positions.get(&*local_player) {
                        client.update_player_volumes(local, &positions);
                        log::info!("updating volume");
                    }
                }
            }

            let func_name = to_sq_string!("CodeCallback_GetPlayersPostion");

            unsafe {
                (sq_functions.sq_schedule_call_external)(
                    ScriptContext_CLIENT,
                    func_name.as_ptr(),
                    nothing00909,
                )
            }
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

        loop {
            if let Ok(mut lock) = self.valid_cl_vm.write() {
                *lock = true;
                break;
            }
        }

        match LOCAL_PLAYER.read() {
            Ok(local_player) => {
                if *local_player == "None" {
                    log::error!("player name isn't registered yet so connection is canceled");
                } else {
                    let client = unsafe { &DISCORD };
                    client.join("catornot-test".to_owned(), local_player.clone());
                }
            }
            Err(err) => log::error!("unable to get lock : {err:?}"),
        }

        if let Ok(mut lock) = self.valid_cl_vm.write() {
            *lock = true
        }

        let sq_functions = unsafe { SQFUNCTIONS.client.as_ref().unwrap() };

        let func_name = to_sq_string!("CodeCallback_GetPlayerName");

        // whar why is this not getting called?
        unsafe {
            (sq_functions.sq_schedule_call_external)(
                ScriptContext_CLIENT,
                func_name.as_ptr(),
                nothing00909,
            )
        }
    }

    fn on_sqvm_destroyed(&self, _context: ScriptVmType) {
        log::info!( "sqvm destroyed for proxichat {_context}" );
        loop {
            if let Ok(mut lock) = self.valid_cl_vm.write() {
                *lock = false;
                break;
            }
        }
    }
}

unsafe impl Sync for ProximityChat {}

entry!(ProximityChat);

#[sqfunction(VM=Client,ExportName=ProxiChatPushPlayerPositions)]
fn push_player_pos(name: String, pos: Vector3) {
    if let Ok(mut lock) = PLAYER_POS.write() {
        _ = lock.insert(name, pos);
    }
    sq_return_null!()
}

#[sqfunction(VM=Client,ExportName=ProxiChatPushPlayerName)]
fn push_player_name(name: String) {
    loop {
        if let Ok(mut lock) = LOCAL_PLAYER.write() {
            *lock = name.clone();
            break;
        }
    }
    log::error!("name is set to {name}");
    sq_return_null!()
}

#[sqfunction(VM=Client)]
fn nothing00909() {
    sq_return_null!()
}
