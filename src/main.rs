// src/main.rs
// Entry point for Affilify Discord bot in Rust (MIT License)
use serenity::{
    async_trait,
    all::{Ready, Interaction, Message, CreateMessage, Mentionable, CreateButton, CreateActionRow},
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
        const VERSION: &str = env!("CARGO_PKG_VERSION");
        println!("🤖 {} v{} is connected and ready!", ready.user.name, VERSION);
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

    /// Monitor all messages: smart handling of Amazon links based on message content.
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
        
        // Check if message contains Amazon links
        if content.contains("amazon.") || content.contains("amzn.to") {
            let amazon_urls = utils::extract_amazon_urls(content);
            
            if amazon_urls.is_empty() {
                return;
            }

            // Determine if this is a link-only message or mixed content
            if utils::is_amazon_link_only(content) {
                // Link-only message: delete and show hint (current behavior)
                let _ = msg.delete(&ctx.http).await;

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
            } else {
                // Mixed content: add button(s) with affiliate link(s)
                let guild_id = msg.guild_id.map(|id| id.get().to_string());
                let mut buttons = Vec::new();
                let mut footer_template = String::new();
                
                // Process all Amazon URLs in the message
                for (i, url) in amazon_urls.iter().enumerate() {
                    if let Some((clean_url, template)) = utils::process_amazon_url(url, guild_id.clone()).await {
                        // Use footer template from first successful processing
                        if footer_template.is_empty() {
                            footer_template = template;
                        }
                        
                        // Create button label based on number of links
                        let label = if amazon_urls.len() > 1 {
                            format!("🛒 View on Amazon ({})", i + 1)
                        } else {
                            "🛒 View on Amazon".to_string()
                        };
                        
                        let button = CreateButton::new_link(&clean_url).label(&label);
                        buttons.push(button);
                        
                        // Discord has a limit of 5 buttons per action row
                        if buttons.len() >= 5 {
                            break;
                        }
                    }
                }
                
                // Only send message if we have at least one button
                if !buttons.is_empty() {
                    // Construct footer with sender mention support
                    let sender_mention = format!("<@{}>", msg.author.id.get());
                    let footer = if footer_template.contains("{{sender}}") {
                        footer_template.replace("{{sender}}", &sender_mention)
                    } else {
                        format!("{} recommended this. {}", sender_mention, footer_template)
                    };
                    
                    let action_row = CreateActionRow::Buttons(buttons);
                    
                    let response_content = format!("-# {}", footer);
                    let message = CreateMessage::new()
                        .content(response_content)
                        .components(vec![action_row]);
                        
                    let _ = msg.channel_id.send_message(&ctx.http, message).await;
                }
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