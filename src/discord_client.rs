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

// oath url = https://discord.com/api/oauth2/authorize?client_id=1056631161276874824&redirect_uri=https%3A%2F%2Fcatornot.github.io%2Ftfproxichat&response_type=code&scope=rpc.voice.read%20rpc.activities.write%20gdm.join%20guilds.join%20activities.read%20applications.entitlements%20voice%20activities.write%20rpc.voice.write%20identify

pub struct DiscordClient {
    pub client: Discord<'static, DiscordEvent>,
    pub token: RwLock<Option<String>>,
    pub lobby_id: RwLock<Option<i64>>,
    pub members: RwLock<HashMap<i64, String>>,
}

impl DiscordClient {
    pub fn new() -> Self {
        let c = Discord::new(APP_ID).expect("failed to load discord rpc; Is your discord running?");

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
                    let mut tk = unsafe { DISCORD.token.write().unwrap() };
                    *tk = Some(token.access_token().to_string());
                }
                Err(error) => {
                    log::error!("failed to retrieve OAuth2 token: {}", error);
                    // panic!()
                }
            });
    }

    pub fn join(&self, server_name: String, nickname: String) {
        self.client.lobby_search(
            SearchQuery::new()
                .filter(
                    "metadata.name".to_string(),
                    Comparison::Equal,
                    server_name.clone(),
                    Cast::String,
                )
                .limit(1),
            move |discord, result| match result {
                Ok(_) => {
                    if discord.lobby_count() == 0 {
                        log::info!("no lobbies found creating a new one");

                        discord.create_lobby(
                            LobbyTransaction::new()
                                .capacity(1000)
                                .kind(LobbyKind::Public)
                                .add_metadata("name".to_string(), server_name.clone()),
                            |discord, result| match result {
                                Ok(lobby) => {
                                    log::info!("aya we have new lobby");
                                    let lobby_id = lobby.id();
                                    discord.update_member(
                                        lobby_id,
                                        Self::get_id(),
                                        LobbyMemberTransaction::new().add_metadata(NAME_KEY.to_string(), nickname),
                                        |_,result| {
                                            if let Err(err) = result {
                                                log::error!("couldn't set nickname on join; people won't hear; pls report this issue to catornot :D");
                                                log::error!("{err}")
                                            }
                                        },
                                    );
                                    Self::write_id(lobby_id);
                                    Self::update_connection_status(true)
                                }
                                Err(err) => {
                                    log::error!("I would die : {err}");
                                    rrplug::prelude::wait(10000);
                                    unsafe { DISCORD.join(server_name, nickname) }
                                }
                            },
                        )
                    } else {
                        log::info!("found a lobby for this server joining it");

                        let lobby_id = discord.lobby_id_at(0).unwrap();
                        discord.connect_lobby_voice(lobby_id, move |discord, result| match result {
                            Ok(_) => {
                                log::info!("we joined a lobby yupppppppie");
                                    discord.update_member(
                                        lobby_id,
                                        Self::get_id(),
                                        LobbyMemberTransaction::new().add_metadata(NAME_KEY.to_string(), nickname),
                                        |_,result| {
                                            if let Err(err) = result {
                                                log::error!("couldn't set nickname on join; people won't hear; pls report this issue to catornot :D");
                                                log::error!("{err}")
                                            }
                                        },
                                    );
                                    Self::write_id(lobby_id);
                                    Self::update_connection_status(true)
                            }
                            Err(err) => {
                                log::info!("everythuingghs bikw: {err}");
                                rrplug::prelude::wait(10000);
                                unsafe { DISCORD.join(server_name, nickname) }
                            }
                        });
                    }
                }
                Err(_) => {
                    log::info!("failed to get lobbies; retrying in 10 seconds");
                    rrplug::prelude::wait(10000);
                    unsafe { DISCORD.join(server_name, nickname) }
                }
            },
        )
    }

    pub fn reset_vc(&self) {
        if let Ok(lock) = self.lobby_id.read() {
            if let Some(lobby_id) = *lock {
                for id in self
                    .client
                    .iter_lobby_member_ids(lobby_id)
                    .unwrap()
                    .filter_map(|i| i.ok())
                {
                    _ = self.client.set_local_volume(id, 100);
                }
            }
        }
    }

    pub fn update_player_volumes(&self, local_pos: &Vector3, positions: &HashMap<String, Vector3>) {
        let members = match self.members.read() {
            Ok(m) => m,
            _ => return,
        };

        let lobby_id = match self.lobby_id.read() {
            Ok(id) => match *id {
                Some(id) => id,
                None => return,
            },
            _ => return,
        };

        let lx = local_pos.x;
        let ly = local_pos.y;

        for id in self
            .client
            .iter_lobby_member_ids(lobby_id)
            .unwrap()
            .filter_map(|i| i.ok())
        {
            let player_name = match members.get(&id) {
                Some(name) => name,
                None => continue,
            };

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
            let volume = ((-dis / 2) + 200).clamp(200, 0);
            _ = self.client.set_local_volume(id, volume.try_into().unwrap()); // how could it possible fail?
        }
    }

    fn write_id(id: i64) {
        let client = unsafe { &mut *DISCORD };
        loop {
            if let Ok(mut lock) = client.lobby_id.write() {
                *lock = Some(id);
                break;
            }
        }
    }

    fn get_id() -> i64 {
        let client = unsafe { &mut *DISCORD };
        client.client.current_user().unwrap().id()
    }

    fn update_connection_status(is_connected: bool) {
        loop {
            if let Ok(mut lock) = SHARED.connected.write() {
                *lock = is_connected;
                break;
            }
        }
    }
}
pub struct DiscordEvent;

impl EventHandler for DiscordEvent {
    fn on_member_update(&mut self, discord: &Discord<'_, Self>, lobby_id: i64, member_id: i64) {
        log::info!("member update callled");

        let name = discord.lobby_member_metadata(lobby_id, member_id, NAME_KEY.to_string());
        match name {
            Ok(name) => {
                let members = unsafe { &mut DISCORD.members };

                loop {
                    if let Ok(mut members) = members.write() {
                        log::info!("{} joined the lobby :)", name.clone());
                        members.insert(member_id, name.clone());
                    }
                    rrplug::prelude::wait(10);
                }
            }
            Err(err) => log::error!(
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
            if let Ok(mut members) = members.write() {
                if let Some(name) = members.remove(&member_id) {
                    log::info!("{name} left the lobby :(")
                }
                break;
            }
            rrplug::prelude::wait(10);
        }
    }

    // maybe use this instead of on_member_update, more testing needed :|
    // fn on_member_connect(
    //         &mut self,
    //         discord: &Discord<'_, Self>,
    //         lobby_id: discord_game_sdk::LobbyID,
    //         member_id: discord_game_sdk::UserID,
    //     ) {

    // }
}

// fn brute_force_remove() {} // todo
