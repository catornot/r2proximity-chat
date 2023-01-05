use discord_game_sdk::{
    Cast, Comparison, Discord, EventHandler, LobbyKind, LobbyMemberTransaction, LobbyTransaction,
    SearchQuery,
};
use rrplug::wrappers::vector::Vector3;
use std::collections::HashMap;
use std::sync::RwLock;

use crate::comms::SHARED;
use crate::DISCORD;

const APP_ID: i64 = 1056631161276874824;
const NAME_KEY: &str = "tf_name";
const MAX_ATTEMPS: i32 = 5;

// oath url = https://discord.com/api/oauth2/authorize?client_id=1056631161276874824&redirect_uri=https%3A%2F%2Fcatornot.github.io%2Ftfproxichat&response_type=code&scope=rpc.voice.read%20rpc.activities.write%20gdm.join%20guilds.join%20activities.read%20applications.entitlements%20voice%20activities.write%20rpc.voice.write%20identify

pub struct DiscordClient {
    pub client: Discord<'static, DiscordEvent>,
    pub token: RwLock<Option<String>>,
    pub lobby_id: RwLock<Option<i64>>,
    pub members: RwLock<HashMap<i64, String>>,
}

impl DiscordClient {
    pub fn new() -> Self {
        let mut c = Discord::new(APP_ID).expect("failed to load discord rpc; Is your discord running?");
        
        *c.event_handler_mut() = Some(DiscordEvent::default());

        // c.clear_activity(|discord, _| {
        //     discord.update_activity(
        //         Activity::empty()
        //             .with_state("ProxiChat")
        //             .with_details("PAIN"),
        //         |_, _| {},
        //     )
        // });

        c.set_overlay_opened(true, |_, _| {});

        Self {
            client: c,
            token: RwLock::new(None::<String>),
            lobby_id: RwLock::new(None),
            members: RwLock::new(HashMap::new()),
        }
    }

    pub fn tick(&mut self) -> Result<(), ()> {
        match self.client.run_callbacks() {
            Ok(_) => Ok(()),
            Err(err) => Err(log::error!("unable to run callbacks because of {err}")),
        }
    }

    pub fn try_setup(&self) {
        self.client
            .oauth2_token(move |_discord, token| match token {
                Ok(token) => {
                    let mut tk = unsafe { DISCORD.token.try_write().unwrap() };
                    *tk = Some(token.access_token().to_string());
                }
                Err(error) => {
                    log::error!("failed to retrieve OAuth2 token: {}", error);
                    // panic!()
                }
            });
    }

    pub fn join(&self, server_name: String, nickname: String, attemp: i32) {

        if attemp >= MAX_ATTEMPS {
            log::warn!("reached max connection attemps");
            return;
        }

        self.client.lobby_search(
            SearchQuery::new()
                .filter(
                    "metadata.name".to_string(),
                    Comparison::Equal,
                    server_name.clone(),
                    Cast::String,
                )
                .limit(1),
            move |discord, result| 
            match result {
                Ok(_) if discord.lobby_count() == 0 => {
                    log::info!("no lobbies found creating a new one");

                    discord.create_lobby(
                        LobbyTransaction::new()
                            .capacity(1000)
                            .kind(LobbyKind::Public)
                            .add_metadata("name".to_string(), server_name.clone()),
                        move |discord, result| 
                        match result {
                            Ok(lobby) => {
                                log::info!("yay we have new lobby");

                                let lobby_id = lobby.id();

                                discord.connect_lobby_voice(lobby_id, |_discord, result|
                                    match result {
                                        Ok(_) => {
                                            log::info!("connected to a vc");
                                            update_connection_status(true)
                                        },
                                        Err(err) => {
                                            log::error!("failed to connected to vc {err}");
                                            update_connection_status(false)
                                        }
                                    }
                                );

                                discord.update_lobby(lobby_id, LobbyTransaction::new()
                                    .add_metadata("secret".to_string(), lobby.secret().to_string()), |_discord, result| {
                                        match result {
                                            Ok(_) => {},
                                            Err(err) => log::error!("failed to set lobby secret people won't be able to connect D: {err}"),
                                        }
                                    }
                                );

                                discord.update_member(
                                    lobby_id,
                                    Self::get_id(),
                                    LobbyMemberTransaction::new().add_metadata(NAME_KEY.to_string(), nickname),
                                    |_,result| {
                                        match result {
                                            Ok(_) => log::info!("nickname updated in the lobby :)"),
                                            Err(err) => {
                                                log::error!("couldn't set nickname on join; people won't hear you; pls report this issue to catornot :D");
                                                log::error!("{err}")
                                            }
                                        }
                                    },
                                );

                                Self::write_id(lobby_id);
                                Self::add_connected_members(discord,lobby_id);
                            }
                            Err(err) => {
                                log::error!("lobby creation failed {err}");
                                rrplug::prelude::wait(10000);
                                unsafe { DISCORD.join(server_name, nickname, attemp + 1) }
                            }
                        }
                    )
                },
                Ok(_) => {
                    log::info!("found a lobby for this server joining it");

                    let lobby_id = discord.lobby_id_at(0).unwrap();

                    let secret = match discord.lobby_metadata(lobby_id, "secret"){
                        Ok(secret) => secret,
                        Err(err) => {
                            log::error!("unable to get secret from the lobby : {err}");
                            return;
                        },
                    }; 

                    discord.connect_lobby(lobby_id, secret, move |discord, result| 
                        match result {
                            Ok(_) => {
                                log::info!("we joined a lobby yupppppppie");

                                    discord.connect_lobby_voice(lobby_id, |_discord, result| 
                                        match result {
                                            Ok(_) => {
                                                log::info!("connected to a vc");
                                                update_connection_status(true)
                                            },
                                            Err(err) => {
                                                log::error!("failed to connected to a vc {err}");
                                                update_connection_status(false)
                                            }
                                        }
                                    );
                                    
                                    discord.update_member(
                                        lobby_id,
                                        Self::get_id(),
                                        LobbyMemberTransaction::new().add_metadata(NAME_KEY.to_string(), nickname),
                                        |_,result| {
                                            if let Err(err) = result {
                                                log::error!("couldn't set nickname on join; people won't hear you; pls report this issue to catornot :D");
                                                log::error!("{err}")
                                            }
                                        },
                                    );

                                    Self::write_id(lobby_id);
                                    Self::add_connected_members(discord,lobby_id);
                            }
                            Err(err) => {
                                log::info!("couldn't connect to the lobby: {err}");
                                rrplug::prelude::wait(10000);
                                unsafe { DISCORD.join(server_name, nickname, attemp + 1) }
                            }
                        }
                    );
                },
                Err(_) => {
                    log::info!("failed to get lobbies; retrying in 10 seconds");
                    rrplug::prelude::wait(10000);
                    unsafe { DISCORD.join(server_name, nickname, attemp + 1) }
                }
            },
        )
    }

    pub fn leave(&self) {
        loop {
            if let Ok(mut lock) = self.members.try_write() {
                lock.clear();
                break;
            }
        }

        let lobby_id = match self.lobby_id.try_read() {
            Ok(lock) => match *lock {
                Some(lobby_id) => lobby_id,
                None => return log::warn!("disconnected failed : no lobby"),
            },
            Err(err) => return log::warn!("disconnected failed : {err}"),
        };

        self.client.disconnect_lobby_voice(lobby_id, |_,result| { 
            match result {
                Ok(_) => log::info!("left a vc"),
                Err(err) => log::error!("failed to leave a vc : {err}"),
            }
        });

        self.client.disconnect_lobby(lobby_id, |_,result| { 
            match result {
                Ok(_) => {
                    log::info!("left the lobby");
                    update_connection_status(false);
                    
                    loop {
                        if let Ok(lock) = SHARED.server_name.try_read() {
                            crate::connect((*lock).clone()); // crashes :cluless:
                            break
                        }
                    }
                },
                Err(err) => log::error!("failed to leave the lobby : {err}"),
            }
        })
    }

    pub fn reset_vc(&self) {
        let members = match self.members.try_read() {
            Ok(m) => m,
            _ => return,
        };

        for id in members.keys() {
            _ = self.client.set_local_volume(*id, 100);
        }
    }

    pub fn update_player_volumes(&self, local_pos: &Vector3, positions: &HashMap<String, Vector3>) {
        let members = match self.members.try_read() {
            Ok(m) => m,
            _ => return,
        };

        // let lobby_id = match self.lobby_id.try_read() {
        //     Ok(id) => match *id {
        //         Some(id) => id,
        //         None => return,
        //     },
        //     _ => return,
        // };

        let lx = local_pos.x;
        let ly = local_pos.y;

        for (id, player_name) in members.iter()
        {
            let player_pos = match positions.get(player_name) {
                Some(pos) => pos,
                None => continue,
            };

            if player_pos == local_pos {
                continue;
            }

            let px = player_pos.x;
            let py = player_pos.y;

            let x = (lx - px).abs();
            let y = (ly - py).abs();

            let dis = (y.powi(2) + x.powi(2)).sqrt() as i32;
            let volume = ((-dis / 2) + 500).clamp(0, 200);

            _ = self.client.set_local_volume(*id, volume.try_into().unwrap()); // how could it possible fail?
        }
    }

    fn write_id(id: i64) {
        let client = unsafe { &mut *DISCORD };
        loop {
            if let Ok(mut lock) = client.lobby_id.try_write() {
                *lock = Some(id);
                break;
            }
        }
    }

    fn get_id() -> i64 {
        let client = unsafe { &mut *DISCORD };
        client.client.current_user().unwrap().id()
    }

    fn add_connected_members(discord: &Discord<'_, DiscordEvent>, lobby_id: i64) {

        for index in 0..discord.lobby_member_count(lobby_id).unwrap() {
            let member_id = match discord.lobby_member_id_at(lobby_id, index) {
                Ok(i) => i,
                Err(err) => {
                    log::error!("a member couldn't be fetched; pls report to catornot; {err}");
                    return;
                },
            };

            let name = discord.lobby_member_metadata(lobby_id, member_id, NAME_KEY.to_string());
            match name {
                Ok(name) => {
                    let members = unsafe { &mut DISCORD.members };

                    loop {
                        if let Ok(mut members) = members.try_write() {
                            log::info!("{} is in the vc :)", name);
                            members.insert(member_id, name);
                            break;
                        }
                        rrplug::prelude::wait(10);
                    }
                }
                Err(err) => log::warn!(
                    "the player that just connected to the lobby doesn't have a name whar : {err}"
                ),
            }
        }
    }
}

#[derive(Default)]
pub struct DiscordEvent;

impl EventHandler for DiscordEvent {
    fn on_member_update(&mut self, discord: &Discord<'_, Self>, lobby_id: i64, member_id: i64) {
        log::info!("member update callled");

        let name = discord.lobby_member_metadata(lobby_id, member_id, NAME_KEY.to_string());
        match name {
            Ok(name) => {
                let members = unsafe { &mut DISCORD.members };

                loop {
                    if let Ok(mut members) = members.try_write() {
                        log::info!("{} joined the lobby :)", name);
                        members.insert(member_id, name);
                        break;
                    }
                    rrplug::prelude::wait(10);
                }
            }
            Err(err) => log::warn!(
                "the player that just connected to the lobby doesn't have a name whar : {err}"
            ),
        }
    }

    fn on_member_disconnect(
        &mut self,
        _discord: &Discord<'_, Self>,
        _lobby_id: i64,
        member_id: i64,
    ) {
        log::info!("member disconnect callled");
        let members = unsafe { &mut DISCORD.members };

        loop {
            if let Ok(mut members) = members.try_write() {
                match members.remove(&member_id) { 
                    Some(name) => log::info!("{name} left the lobby :("),
                    None => log::info!("someone left the lobby"), 
                }
                break;
            }
            rrplug::prelude::wait(10);
        }
    }

    fn on_lobby_delete(&mut self, _discord: &Discord<'_, Self>, _lobby_id: i64, _reason: u32) {
        log::info!("lobby got destroyed");

        update_connection_status(false)
    }
    
    // todo: add this
    // fn on_speaking(
    //         &mut self,
    //         discord: &Discord<'_, Self>,
    //         lobby_id: i64,
    //         member_id: i64,
    //         speaking: bool,
    //     ) {
        
    // }
}

fn update_connection_status(is_connected: bool) {
    loop {
        if let Ok(mut lock) = SHARED.connected.try_write() {
            *lock = is_connected;
            break;
        }
    }
}