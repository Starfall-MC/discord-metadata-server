use anyhow::anyhow;
use std::{ops::Deref, sync::Arc};

use axum::{extract::State, routing::get, Json, Router};
use serenity::all::*;

use crate::result::AppResult;

#[derive(Clone)]
pub struct AppState {
    pub cache: Arc<Cache>,
    pub http: Arc<Http>,
    pub guild_id: GuildId,
}

pub fn make_router(cache: Arc<Cache>, http: Arc<Http>, guild_id: GuildId) -> Router {
    let state = AppState {
        cache,
        http,
        guild_id,
    };
    Router::new()
        .route("/members", get(members_list))
        .with_state(state)
}

#[axum::debug_handler]
async fn members_list(State(state): State<AppState>) -> AppResult<Json<Vec<u64>>> {
    let gref = &state
        .cache
        .guild(state.guild_id)
        .ok_or(anyhow!("Guild in config not found on Discord"))?;

    let ids = gref
        .members
        .iter()
        .map(|(i, _)| i.get())
        .collect::<Vec<u64>>();

    Ok(Json(ids))
}
