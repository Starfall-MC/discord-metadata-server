use async_trait::async_trait;
use serenity::all::*;

pub struct Handler {}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: serenity::client::Context, ready: Ready) {
        println!("{} is connected!", ctx.cache.current_user().name);

        println!("info: {ready:?}");
    }
}
