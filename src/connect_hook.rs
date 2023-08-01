use once_cell::sync::OnceCell;
use rrplug::{
    bindings::command::CCommand, mid::concommands::find_concommand, prelude::CCommandResult,
};

use crate::{exports::PLUGIN, shared::PROXICHAT_PORT};

static ORIGINAL_CONNECT_FUNC: OnceCell<unsafe extern "C" fn(*const CCommand)> = OnceCell::new();
static ORIGINAL_DISCONNECT_FUNC: OnceCell<unsafe extern "C" fn(*const CCommand)> = OnceCell::new();

pub fn setup_connect_hook() {
    let connect_command = match find_concommand("connect") {
        Some(c) => c,
        None => return log::error!("couldn't find connect command => proxi chat will not work"),
    };

    if let Some(org_func) = connect_command.m_pCommandCallback.replace(connect_hook) {
        _ = ORIGINAL_CONNECT_FUNC.set(org_func);

        log::info!("replaced connect callback");
    }

    let disconnect_command = match find_concommand("disconnect") {
        Some(c) => c,
        None => return log::error!("couldn't find disconnect command => proxi chat will not work"),
    };

    if let Some(org_func) = disconnect_command
        .m_pCommandCallback
        .replace(disconnect_hook)
    {
        _ = ORIGINAL_DISCONNECT_FUNC.set(org_func);

        log::info!("replaced disconnect callback");
    }
}

unsafe extern "C" fn connect_hook(ccommand: *const CCommand) {
    let parsed_ccommand = CCommandResult::new(ccommand);

    let ip = match parsed_ccommand.get_args().get(0) {
        Some(arg) => arg.split(':').collect::<Vec<&str>>()[0],
        None => return log::error!("connect didn't have any args"),
    };

    if ip == "localhost" {
        log::info!("found connection to local server; doing nothing");
        (ORIGINAL_CONNECT_FUNC.wait())(ccommand);
        return;
    }

    if let crate::shared::ProximityChatType::Client(client) = &PLUGIN.wait().proximity_chat {
        log::info!("found connection to {ip}");

        client
            .lock()
            .set_new_connection(format!("{ip}:{PROXICHAT_PORT}"));
    }

    (ORIGINAL_CONNECT_FUNC.wait())(ccommand);
}

unsafe extern "C" fn disconnect_hook(ccommand: *const CCommand) {
    if let crate::shared::ProximityChatType::Client(client) = &PLUGIN.wait().proximity_chat {
        client.lock().drop_stream();

        log::info!("found disconnect");
    }

    (ORIGINAL_DISCONNECT_FUNC.wait())(ccommand);
}
