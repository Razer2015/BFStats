use serde::{Deserialize, Serialize};
use serde_aux::prelude::*;
use std::collections::HashMap;

pub struct Count {
    pub count: i64
}

#[derive(Debug, Serialize)]
pub struct PlayerVip {
    pub id: i32,
    pub gametype: String,
    pub servergroup: String,
    pub playername: Option<String>,
    pub timestamp: Option<i64>,
    pub status: String,
    pub admin: Option<String>,
    pub comment: Option<String>,
    pub guid: Option<String>,
    pub discord_id: Option<i64>,
    pub active: i32
}

#[derive(Debug, Serialize)]
pub struct PlayerScoreStats {
    pub position: Option<f64>,
    pub soldiername: Option<String>,
    pub score: Option<String>,
    pub global_rank: u16,
    pub kills: u32,
    pub deaths: u32,
    pub teamkills: u32,
    pub suicides: u32,
    pub kdr: Option<String>,
    pub playtime: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PlayerTeamkillStats {
    pub position: Option<f64>,
    pub soldiername: Option<String>,
    pub score: Option<String>,
    pub global_rank: u16,
    pub kills: u32,
    pub deaths: u32,
    pub teamkills: u32,
    pub suicides: u32,
    pub kdr: Option<String>,
    pub playtime: Option<String>,
    pub teamkills_by_hour: Option<String>
}

#[derive(Debug, Serialize)]
pub struct PlayerData {
    pub player_id: u32,
    pub clan_tag: Option<String>,
    pub soldiername: Option<String>,
    pub rank_score: u32,
    pub score: Option<String>,
    pub global_rank: u16,
    pub kills: u32,
    pub deaths: u32,
    pub teamkills: u32,
    pub suicides: u32,
    pub wins: u32,
    pub losses: u32,
    pub headshots: u32,
    pub highscore: Option<String>,
    pub killstreak: u16,
    pub deathstreak: u16,
    pub kdr: Option<String>,
    pub playtime: Option<String>,
    pub rounds: u32,
}

#[derive(Debug, Serialize)]
pub struct ServerScoreTemplate {
    pub base_path: String,
    pub players: Vec<PlayerScoreStats>
}

#[derive(Debug, Serialize)]
pub struct ServerTeamkillsTemplate {
    pub base_path: String,
    pub players: Vec<PlayerTeamkillStats>
}

#[derive(Debug, Serialize)]
pub struct ServerRankTemplate {
    pub base_path: String,
    pub servername: Option<String>,
    pub total_players: i64,
    pub bg_index: u8,
    pub clan_tag: Option<String>,
    pub soldiername: Option<String>,
    pub profile_image_url: String,
    pub rank_score: u32,
    pub score: Option<String>,
    pub global_rank: u16,
    pub kills: u32,
    pub deaths: u32,
    pub teamkills: u32,
    pub suicides: u32,
    pub wins: u32,
    pub losses: u32,
    pub headshots: u32,
    pub highscore: Option<String>,
    pub killstreak: u16,
    pub deathstreak: u16,
    pub kdr: Option<String>,
    pub playtime: Option<String>,
    pub rounds: u32,
}

#[derive(Debug, Serialize)]
pub struct Server {
    pub server_id: u16,
    pub server_name: Option<String>
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub username: Option<String>,
    pub gravatar_md5: Option<String>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub user_id: u64,
    pub created_at: u64,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Context {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub persona_id: u64,
    pub user: User,
}

// #[derive(Debug, Serialize, Deserialize)]
// #[serde(rename_all = "camelCase")]
// pub struct Game {
//     #[serde(deserialize_with = "deserialize_number_from_string")]
//     pub persona_id: u64,
//     pub user: User,
// }

/// # Example
/// ```ron
/// SearchResult {
///     picture: "",
///     user_id: 2955058489260500539,
///     user: User {
///         username: Some(
///             "PocketWolfy",
///         ),
///         gravatar_md5: Some(
///             "b97c726c98f9f615bd62088c9e4c5cb4",
///         ),
///         user_id: 2955058489260500539,
///         created_at: 1393081344,
///     },
///     persona_id: 994520424,
///     persona_name: "PocketWolfy",
///     namespace: "cem_ea_id",
///     games: {
///         1: "2050",
///     },
/// }
/// ```
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResult {
    pub picture: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub user_id: u64,
    pub user: User,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub persona_id: u64,
    pub persona_name: String,
    pub namespace: String,
    pub games: HashMap<i32, String>,
}


#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse {
    pub r#type: String,
    pub message: String,
    pub data: Vec<SearchResult>,
}
