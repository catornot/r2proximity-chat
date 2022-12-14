#![feature(is_some_and)]

use comms::SHARED;
use once_cell::sync::Lazy;
use rrplug::bindings::squirrelclasstypes::ScriptContext_CLIENT;
use rrplug::prelude::*;
use rrplug::wrappers::northstar::ScriptVmType;
use rrplug::wrappers::vector::Vector3;
use rrplug::{sq_return_null, sqfunction, to_sq_string};
use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver};
use std::sync::RwLock;

mod comms;
mod discord_client;
mod window;

use crate::comms::SendComms;
use crate::window::init_window;

use crate::discord_client::DiscordClient;

const BLACKLIST: [&str; 2] = ["Fragyeeter", "R8GOD"];

static PLAYER_POS: Lazy<RwLock<HashMap<String, Vector3>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));
static LOCAL_PLAYER: Lazy<RwLock<String>> = Lazy::new(|| RwLock::new("None".to_string()));

static mut DISCORD: Lazy<DiscordClient> = Lazy::new(DiscordClient::new);

#[derive(Debug)]
struct ProximityChat {
    valid_cl_vm: RwLock<bool>,
    recv: Option<Receiver<SendComms>>,
}

impl Plugin for ProximityChat {
    fn new() -> Self {
        Self {
            valid_cl_vm: RwLock::new(false),
            recv: None,
        }
    }

    fn initialize(&mut self, plugin_data: &PluginData) {
        _ = plugin_data.register_sq_functions(info_push_player_pos);
        _ = plugin_data.register_sq_functions(info_push_player_name);
        _ = plugin_data.register_sq_functions(info_push_server_name);

        let (send, recv) = channel::<SendComms>();

        self.recv = Some(recv);

        log::info!("starting a second window");
        std::thread::spawn(move || init_window(send));

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
        let recv = self.recv.as_ref().unwrap();

        loop {
            wait(1000);

            _ = client.tick();

            if let Ok(comms) = recv.try_recv() {
                match client.client.set_self_mute(comms.mute) {
                    Ok(_) => log::info!("set muted to {}", comms.mute),
                    Err(e) => log::error!(
                        "unable to set muted to {}; the window is now desynced; {e}",
                        comms.mute
                    ),
                }

                match client.client.set_self_deaf(comms.deaf) {
                    Ok(_) => log::info!("set deaf to {}", comms.deaf),
                    Err(e) => log::error!(
                        "unable to set deaf to {}; the window is now desynced; {e}",
                        comms.deaf
                    ),
                }

                if self.valid_cl_vm.try_read().is_ok_and(|x| !*x) {
                    continue;
                }

                if comms.reset_server_name {
                    let func_name = to_sq_string!("CodeCallback_PushServerName");
                    unsafe {
                        (sq_functions.sq_schedule_call_external)(
                            ScriptContext_CLIENT,
                            func_name.as_ptr(),
                            pop_function,
                        )
                    }
                } else if comms.name_overwrite.is_some()
                    && SHARED.connected.try_read().is_ok_and(|x| *x)
                {
                    let server_name = comms.name_overwrite.unwrap();
                    log::info!("got server name overwrite {server_name}");
                    log::warn!("spamming this may result in a crash");

                    if let Ok(mut lock) = SHARED.server_name.try_write() {
                        if *lock != server_name {
                            disconnect()
                        }

                        *lock = server_name;
                    }
                }
            }

            if SHARED.connected.try_read().is_ok_and(|x| !*x) {
                continue;
            }

            if let Ok(lock) = self.valid_cl_vm.try_read() {
                if !*lock {
                    client.reset_vc();
                    continue;
                }
            }

            match PLAYER_POS.try_read() {
                Ok(positions) => {
                    if let Ok(local_player) = LOCAL_PLAYER.try_read() {
                        if let Some(local) = positions.get(&*local_player) {
                            client.update_player_volumes(local, &positions);
                        }
                    }
                }
                Err(err) => log::error!("unable to acces player positions {err}"),
            }

            if let Ok(lock) = client.members.try_read() {
                if let Ok(mut lock_members) = SHARED.members.try_write() {
                    // expensive :(
                    let members: Vec<String> =
                        (*lock.values().cloned().collect::<Vec<_>>()).to_vec();

                    *lock_members = members
                }
            }

            let func_name = to_sq_string!("CodeCallback_GetPlayersPostion");

            unsafe {
                (sq_functions.sq_schedule_call_external)(
                    ScriptContext_CLIENT,
                    func_name.as_ptr(),
                    pop_function,
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
            if let Ok(mut lock) = self.valid_cl_vm.try_write() {
                *lock = true;
                break;
            }
        }

        if let Ok(lock) = &SHARED.server_name.try_read() {
            connect((*lock).clone());
        }
    }

    fn on_sqvm_destroyed(&self, _context: ScriptVmType) {
        loop {
            if let Ok(mut lock) = self.valid_cl_vm.try_write() {
                *lock = false;
                break;
            }
        }
    }
}

unsafe impl Sync for ProximityChat {}

entry!(ProximityChat);

pub fn connect(server_name: String) {
    if SHARED.connected.try_read().is_ok_and(|x| *x) {
        return;
    }

    match LOCAL_PLAYER.try_read() {
        Ok(local_player) => {
            if *local_player == "None" {
                log::warn!("player name isn't registered yet so connection is canceled");
            } else {
                log::info!("connecting to {}", server_name);
                let client = unsafe { &DISCORD };
                client.join(server_name, local_player.clone(), 0);
            }
        }
        Err(err) => log::error!("unable to get lock : {err:?}"),
    }
}

fn disconnect() {
    if SHARED.connected.try_read().is_ok_and(|x| !*x) {
        return;
    }

    log::info!("disconnecting from discord lobby");
    let client = unsafe { &DISCORD };
    client.leave();
}

#[sqfunction(VM=Client,ExportName=ProxiChatPushPlayerPositions)]
fn push_player_pos(name: String, pos: Vector3) {
    if let Ok(mut lock) = PLAYER_POS.try_write() {
        _ = lock.insert(name, pos);
    }
    sq_return_null!()
}

#[sqfunction(VM=Client,ExportName=ProxiChatPushPlayerName)]
fn push_player_name(name: String) {
    log::info!("name is set to {name}");

    if BLACKLIST.contains(&&name[..]) {
        sq_return_null!()
    }

    loop {
        if let Ok(mut lock) = LOCAL_PLAYER.try_write() {
            *lock = name;
            break;
        }
    }

    sq_return_null!()
}

#[sqfunction(VM=Client,ExportName=ProxiChatPushServerName)]
fn push_server_name(server_name: String) {
    log::info!("server name is set to {server_name}");

    loop {
        if let Ok(mut lock) = SHARED.server_name.try_write() {
            if *lock != server_name {
                disconnect()
            }
            connect(server_name.clone());

            *lock = server_name;
            break;
        }
    }

    sq_return_null!()
}

#[sqfunction(VM=Client)]
fn pop_function() {
    sq_return_null!()
}
