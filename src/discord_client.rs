#[allow(unused_imports)]
use discord_game_sdk::{
    Action, Activity, Cast, Comparison, Discord, Entitlement, EventHandler, LobbyID, LobbyKind,
    LobbyMemberTransaction, LobbyTransaction, NetworkChannelID, NetworkPeerID, OAuth2Token,
    Relationship, SearchQuery, User, UserAchievement, UserID,
};
use std::sync::RwLock;

const APP_ID: i64 = 1056631161276874824;

// oath url = https://discord.com/api/oauth2/authorize?client_id=1056631161276874824&redirect_uri=https%3A%2F%2Fcatornot.github.io%2Ftfproxichat&response_type=code&scope=rpc.voice.read%20rpc.activities.write%20gdm.join%20guilds.join%20activities.read%20applications.entitlements%20voice%20activities.write%20rpc.voice.write%20identify

pub struct DiscordClient {
    pub client: Discord<'static, DiscordEvent>,
    pub token: RwLock<Option<OAuth2Token>>,
}

impl DiscordClient {
    pub fn new() -> Self {
        let c = Discord::new(APP_ID).expect("failed to load discord rpc; Is your discord running?");
        Self {
            client: c,
            token: RwLock::new(None::<OAuth2Token>),
        }
    }

    pub fn tick(&mut self) -> Result<(), ()> {
        match self.client.run_callbacks() {
            Ok(_) => Ok(()),
            Err(_) => Err(()),
        }
    }

    pub fn try_setup(&self) {
        // let mut tk = self.token.write().unwrap();
        self.client.oauth2_token(move |_discord, token| match token {
            Ok(_) => {
                // *tk = Some(token.clone());
            }
            Err(error) => {
                log::error!("failed to retrieve OAuth2 token: {}", error);
                // panic!()
            }
        });
    }

    pub fn join(&self, server_name: String) {
        self.client.lobby_search(
            SearchQuery::new()
                .filter(
                    "metadata.name".to_string(),
                    Comparison::Equal,
                    server_name,
                    Cast::String,
                )
                .limit(10),
            move |_discord, _result| {},
        )
    }
}
pub struct DiscordEvent;

impl EventHandler for DiscordEvent {}