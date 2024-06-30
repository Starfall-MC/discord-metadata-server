use anyhow::anyhow;
use std::sync::Arc;
use twilight_cache_inmemory::InMemoryCache;
use twilight_http::Client as DiscordClient;
use twilight_model::id::{marker::GuildMarker, Id};

use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};

use crate::result::AppResult;

#[derive(Clone)]
pub struct AppState {
    pub cache: Arc<InMemoryCache>,
    pub http: Arc<DiscordClient>,
    pub guild_id: Id<GuildMarker>,
}

pub fn make_router(
    cache: Arc<InMemoryCache>,
    http: Arc<DiscordClient>,
    guild_id: Id<GuildMarker>,
) -> Router {
    let state = AppState {
        cache,
        http,
        guild_id,
    };
    Router::new()
        .route("/members", get(members_list))
        .route("/enrich", post(crate::enrichment::enrich_token))
        .with_state(state)
}

#[axum::debug_handler]
async fn members_list(State(state): State<AppState>) -> AppResult<Json<Vec<u64>>> {
    let ids = state
        .cache
        .guild_members(state.guild_id)
        .ok_or(anyhow!("Guild in config not found on Discord cache"))?
        .iter()
        .map(|v| v.get())
        .collect();

    Ok(Json(ids))
}
