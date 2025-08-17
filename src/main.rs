// src/main.rs
// Entry point for Affilify Discord bot in Rust (MIT License)
use serenity::{
    async_trait,
    all::{Ready, Interaction, Message, CreateMessage, Mentionable},
    prelude::*,
};

mod config;
mod db;
mod utils;
mod commands {
    pub mod amazon;
    pub mod configure;
    pub mod stats;
}

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    /// Called when the bot successfully connects to Discord.
    async fn ready(&self, ctx: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
        // Register slash commands at startup
        commands::configure::register_commands(&ctx.http).await;
        commands::amazon::register_commands(&ctx.http).await;
        commands::stats::register_commands(&ctx.http).await;
    }

    /// Handle incoming interactions (slash commands, autocomplete, modals).
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match &interaction {
            Interaction::Command(cmd) => {
                match cmd.data.name.as_str() {
                    "configure" => commands::configure::run(&ctx, cmd).await,
                    "amazon"    => commands::amazon::run(&ctx, cmd).await,
                    "stats"     => commands::stats::run(&ctx, cmd).await,
                    _            => {}
                }
            },
            Interaction::Autocomplete(autocomplete) => {
                match autocomplete.data.name.as_str() {
                    "configure" => commands::configure::handle_autocomplete(&ctx, &interaction).await,
                    _ => {}
                }
            },
            Interaction::Modal(modal) => {
                if modal.data.custom_id.starts_with("config_modal_") {
                    commands::configure::handle_modal(&ctx, &interaction).await;
                }
            },
            _ => {}
        }
    }

    /// Monitor all messages: delete raw Amazon links and send a temporary hint in-channel.
    async fn message(&self, ctx: Context, msg: Message) {
        // Ignore messages from bots
        if msg.author.bot {
            return;
        }

        // Skip deletion logic in DMs
        if msg.guild_id.is_none() {
            return;
        }

        let content = msg.content.trim();
        // Detect raw Amazon or short links
        if content.contains("amazon.") || content.contains("amzn.to") {
            // Delete original message
            let _ = msg.delete(&ctx.http).await;

            // Ping user in same channel with hint
            let mention = msg.author.id.mention();
            let message = CreateMessage::new()
                .content(format!(
                    "{}, please use `/amazon <link>` to clean and tag your URL.",
                    mention
                ));
                
            if let Ok(sent) = msg.channel_id.send_message(&ctx.http, message).await {
                // Auto-delete the hint after 10 seconds
                let http = ctx.http.clone();
                tokio::spawn(async move {
                    tokio::time::sleep(std::time::Duration::from_secs(10)).await;
                    let _ = sent.delete(&http).await;
                });
            }
        }
    }
}

#[tokio::main]
async fn main() {
    // Load .env configuration
    config::init().expect("Failed to load .env");
    // Initialize SQLite database
    db::init().expect("Failed to initialize database");
    // Retrieve Discord token from environment
    let token = config::discord_token();
    // Define the necessary gateway intents (including DM support)
    let intents = GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;
    // Build the client
    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client");
    // Start the bot
    if let Err(why) = client.start().await {
        eprintln!("Client error: {:?}", why);
    }
}