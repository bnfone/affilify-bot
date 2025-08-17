// src/commands/amazon.rs
// Handles the `/amazon` slash command: cleans URLs, tags them, and logs usage.

use serenity::all::{
    Command, CommandInteraction, CommandOptionType,
    CreateCommand, CreateCommandOption,
    CreateInteractionResponse, CreateInteractionResponseMessage,
    InstallationContext, InteractionContext,
};
use serenity::http::Http;
use serenity::prelude::*;
use rusqlite::params;
use super::super::{db, utils, config};

/// Register the `/amazon` slash command with a URL option.
pub async fn register_commands(http: &Http) {
    let command = CreateCommand::new("amazon")
        .description("Clean and tag your Amazon link")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "url",
                "Your raw Amazon URL, e.g. https://amzn.to/..."
            )
            .required(true)
        )
        .dm_permission(true)
        // *** WICHTIG für DM-Verwendung: ***
        // 1) App darf als User-App installiert werden (für DMs):
        .integration_types(vec![
            InstallationContext::Guild,
            InstallationContext::User,      // nötig für DM-Verwendung
        ])
        // 2) In welchen Kontexten der Command erscheint:
        .contexts(vec![
            InteractionContext::Guild,                // Server
            InteractionContext::BotDm,                // 1:1 DMs mit dem Bot
            InteractionContext::PrivateChannel,       // Gruppen-DMs
        ]);

    let _ = Command::create_global_command(http, command).await;
}

/// Handler for the `/amazon` command.
/// - Resolves short URLs
/// - Parses ASIN and region
/// - Retrieves tracking tag and footer template
/// - Logs usage in the database
/// - Replies with a plain message: cleaned link + footer
pub async fn run(ctx: &Context, cmd: &CommandInteraction) {
    // Check if this is a DM or Guild interaction
    let is_dm = cmd.guild_id.is_none();
    let guild_id = cmd.guild_id.map(|id| id.get().to_string()).unwrap_or_else(|| "DM".to_string());

    // Extract raw URL argument
    let url_raw = cmd.data.options.first()
        .and_then(|opt| opt.value.as_str())
        .unwrap_or("")
        .to_string();

    // Resolve redirects
    let resolved = utils::resolve_url(&url_raw).await.unwrap_or_else(|_| url_raw.clone());

    // Parse ASIN and region
    if let Some((asin, region)) = utils::parse_amazon_url(&resolved) {
        // Determine tracking tag and footer based on context (DM vs Guild)
        let (tag, footer_template) = if is_dm {
            // Use default developer tags and signature for DMs
            let default_tag = config::default_tracking_tag(&region);
            let default_signature = config::default_signature();
            (default_tag, default_signature)
        } else {
            // Try to get guild-specific settings, fallback to defaults
            let default_template = "Using this link you support our server!".to_string();
            
            let (guild_tag, guild_footer) = db::with_connection(|conn| {
                let tag: String = conn.query_row(
                    "SELECT tracking_tag FROM guild_affiliates WHERE guild_id = ? AND region = ?",
                    params![guild_id, region],
                    |r| r.get(0),
                ).unwrap_or_else(|_| String::new());
                
                let footer: String = conn.query_row(
                        "SELECT footer_text FROM guild_settings WHERE guild_id = ?",
                        params![guild_id],
                        |r| r.get(0),
                    )
                    .unwrap_or_else(|_| default_template.clone());
                Ok((tag, footer))
            })
            .unwrap_or((String::new(), default_template.clone()));
            
            // If no guild tag configured, use default developer tag
            if guild_tag.is_empty() {
                let default_tag = config::default_tracking_tag(&region);
                let default_signature = config::default_signature();
                (default_tag, default_signature)
            } else {
                (guild_tag, guild_footer)
            }
        };

        // If still no tag available, inform user
        if tag.is_empty() {
            let response = CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content("No tracking tag available for this region.")
            );
            let _ = cmd.create_response(&ctx.http, response).await;
            return;
        }

        // Log usage
        let _ = db::with_connection(|conn| {
            conn.execute(
                "INSERT INTO link_stats (guild_id, region) VALUES (?, ?)",
                params![guild_id, region],
            )
        });

        // Build cleaned URL
        let clean_url = format!("https://amazon.{}/dp/{}/?tag={}", region, asin, tag);

        // Construct footer with sender mention support (only in guilds, not DMs)
        let footer = if is_dm {
            footer_template
        } else {
            let sender_mention = format!("<@{}>", cmd.user.id.get());
            if footer_template.contains("{{sender}}") {
                footer_template.replace("{{sender}}", &sender_mention)
            } else {
                format!("{} recommended this. {}", sender_mention, footer_template)
            }
        };

        // Send plain message: link + "-# footer"
        let response_content = format!("{}\n-# {}", clean_url, footer);
        let response = CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new()
                .content(response_content)
        );
        let _ = cmd.create_response(&ctx.http, response).await;
    } else {
        // Parsing failed
        let response = CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new()
                .content("Could not parse Amazon URL. Ensure it's valid.")
        );
        let _ = cmd.create_response(&ctx.http, response).await;
    }
}
