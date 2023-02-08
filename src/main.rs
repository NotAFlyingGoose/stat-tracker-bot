mod handler;
mod plot;

use handler::Handler;
use std::{env, process::exit};
use serenity::{prelude::GatewayIntents, Client};

#[tokio::main]
async fn main() {
    if dotenv::dotenv().is_ok() {
        println!("found .env");
    }
    let token = env::var("DISCORD_TOKEN")
        .expect("Expected a token in this environment");
        
    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::MESSAGE_CONTENT;
  
    let mut client =
        Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Err creating client");
    
    
    if let Err(why) = client.start().await {
        println!("Client Error: {:?}", why);
        exit(1);
    }
}
