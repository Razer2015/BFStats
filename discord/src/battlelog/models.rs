use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_aux::prelude::*;

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
pub struct IngameMetadataResponse {
    pub club_rank: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub persona_id: u64,
    pub emblem_url: String,
    pub club_name: String,
    pub country_code: String,
}

impl IngameMetadataResponse {
    pub fn get_emblem_url(&self) -> Option<String> {
        if self.emblem_url.is_empty() {
            return None;
        }

        Some(self.emblem_url.replace(".dds", ".png"))
    }
}