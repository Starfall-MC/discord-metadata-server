mod enrichment;
mod result;
mod svc;

use std::sync::Arc;

use svc::make_router;
use tokio::net::TcpListener;
use twilight_cache_inmemory::InMemoryCache;
use twilight_gateway::{Intents, Shard, ShardId};
use twilight_model::id::{marker::GuildMarker, Id};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let _ = dotenvy::dotenv(); // ignoring if there is no .env file

    let discord_token =
        std::env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN must be the Discord bot token");

    let guild_id = Id::<GuildMarker>::new(
        std::env::var("GUILD_ID")
            .expect("GUILD_ID must be the Discord guild ID to keep track of")
            .parse::<u64>()
            .expect("GUILD_ID must be an integer"),
    );

    // Set up the shard and cache
    let mut shard = Shard::new(ShardId::ONE, discord_token.clone(), Intents::all());
    let http = Arc::new(twilight_http::Client::new(discord_token));
    let cache = Arc::new(InMemoryCache::builder().build());

    // Keep the Cache and Http, so we can use that in server operations
    let router = make_router(cache.clone(), http.clone(), guild_id);

    // Set up the Axum server
    let http_task = tokio::task::spawn(async move {
        axum::serve(
            TcpListener::bind("0.0.0.0:8080")
                .await
                .expect("Failed to bind to port 8080 for HTTP client"),
            router.into_make_service(),
        )
        .await
        .expect("failed serving HTTP");
    });

    // Start the shard
    let gateway_task = tokio::task::spawn(async move {
        loop {
            let item = match shard.next_event().await {
                Ok(v) => v,
                Err(e) => {
                    tracing::error!("Error in shard: {}", e);
                    break;
                }
            };

            tracing::debug!("Received event: {:?}", item);
            cache.update(&item);
        }
    });

    tokio::select! {
        _ = http_task => tracing::info!("HTTP server exited first"),
        _ = gateway_task => tracing::info!("Gateway exited first"),
    }
}
