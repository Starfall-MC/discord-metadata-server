use anyhow::anyhow;
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use twilight_model::id::{
    marker::{RoleMarker, UserMarker},
    Id,
};

use crate::{result::AppResult, svc::AppState};

#[derive(Debug, Serialize, Deserialize)]
pub struct MemberInfo {
    pub roles: Vec<Id<RoleMarker>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Token {
    #[serde(flatten)]
    pub other_fields: std::collections::HashMap<String, serde_json::Value>,

    #[serde(flatten, default, skip_serializing_if = "Option::is_none")]
    pub member_info: Option<MemberInfo>,
}

#[axum::debug_handler]
pub async fn enrich_token(
    State(state): State<AppState>,
    Json(src_token): Json<Token>,
) -> AppResult<Json<Token>> {
    tracing::debug!("Received for enrichment: {:?}", src_token);

    // If the discord_id field is empty or not a number, then we don't want to do anything
    let user_id = match src_token.other_fields.get("discord_id") {
        Some(discord_id) if discord_id.is_u64() => {
            Id::<UserMarker>::new(discord_id.as_u64().unwrap())
        }
        Some(v) if v.is_string() => {
            let maybe_discord_id = v.as_str().unwrap().parse::<u64>();
            if let Ok(discord_id) = maybe_discord_id {
                Id::<UserMarker>::new(discord_id)
            } else {
                // If the discord_id is a string,
                // and that string is not a number,
                // then we should replace it with a number,
                // so that the upstream app doesn't get confused
                tracing::warn!(
                    "Incoming token had a discord_id string field that is not a number: {:?}, replacing with 0",
                    v
                );
                let mut token = src_token;
                token.other_fields.insert(
                    "discord_id".to_string(),
                    serde_json::Value::String(String::from("0")),
                );
                return Ok(Json(token));
            }
        }
        _ => {
            tracing::warn!("Incoming token had no discord_id integer field, returning unchanged");
            return Ok(Json(src_token));
        }
    };

    let member = state
        .cache
        .member(state.guild_id, user_id)
        .ok_or(anyhow!("Member not found in guild"))?;

    tracing::debug!("Got member: {:?}", member);

    let info = MemberInfo {
        roles: member.roles().iter().copied().collect(),
    };

    Ok(Json(Token {
        other_fields: src_token.other_fields,
        member_info: Some(info),
    }))
}
