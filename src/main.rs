mod handler;
mod result;
mod svc;

use serenity::all::{GatewayIntents, GuildId};
use svc::make_router;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let _ = dotenvy::dotenv(); // ignoring if there is no .env file

    let discord_token =
        std::env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN must be the Discord bot token");

    let guild_id = GuildId::from(
        std::env::var("GUILD_ID")
            .expect("GUILD_ID must be the Discord guild ID to keep track of")
            .parse::<u64>()
            .expect("GUILD_ID must be an integer"),
    );

    let intents = GatewayIntents::all();

    let mut client = serenity::Client::builder(&discord_token, intents)
        .event_handler(crate::handler::Handler {})
        .await
        .expect("Failed to create client");

    // Keep the Cache and Http, so we can use that in server operations
    let cache = client.cache.clone();
    let http = client.http.clone();

    tokio::task::spawn(async move {
        if let Err(why) = client.start().await {
            println!("Client error: {:?}", why);
        }
    });

    // Set up the Axum server
    axum::serve(
        TcpListener::bind("0.0.0.0:8080")
            .await
            .expect("Failed to bind to port 8080 for HTTP client"),
        make_router(cache, http, guild_id).into_make_service(),
    )
    .await
    .expect("failed serving HTTP");
}
