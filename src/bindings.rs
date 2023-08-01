use std::ffi::CStr;

use rrplug::prelude::*;
use rrplug::{bindings::entity::CBaseClient, engine_functions};
use std::ffi::c_char;

const MAX_PLAYERS: usize = 28; // 32 is impossible to reach and I think I have seen 28 player lobbies

engine_functions! {
    ENGINE_FUNCTIONS + EngineFunctions for PluginLoadDLL::ENGINE => {
        client_array = *mut CBaseClient, at 0x12A53F90;
        local_player_user_id = *const c_char, at 0x13F8E688;
    }
}

pub fn uid_exits(uid: i32) -> bool {
    let uid = uid.to_string();
    let client_array = ENGINE_FUNCTIONS.wait().client_array;
    (0..MAX_PLAYERS)
        .filter_map(|i| unsafe { client_array.add(i).as_ref() })
        .any(|client| unsafe { CStr::from_ptr(client.uid.as_ptr()).to_string_lossy() == uid })
    // maybe not be sound since CBaseClient.uid might not have a terminator
}

pub fn parse_local_uid() -> Result<i32,std::num::ParseIntError> {
    unsafe { CStr::from_ptr(ENGINE_FUNCTIONS.wait().local_player_user_id).to_string_lossy().parse() }
}