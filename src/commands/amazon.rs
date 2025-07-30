// src/commands/amazon.rs
// Handles the `/amazon` slash command: cleans URLs, tags them, and logs usage.

use serenity::http::Http;
use serenity::model::application::command::{Command, CommandOptionType};
use serenity::model::application::interaction::application_command::{ApplicationCommandInteraction, CommandDataOptionValue};
use serenity::model::application::interaction::InteractionResponseType;
use serenity::prelude::*;
use rusqlite::params;
use super::super::{db, utils, config};

/// Register the `/amazon` slash command with a URL option.
pub async fn register_commands(http: &Http) {
    let _ = Command::create_global_application_command(http, |cmd| {
        cmd.name("amazon")
            .description("Clean and tag your Amazon link")
            .create_option(|opt| {
                opt.name("url")
                    .description("Your raw Amazon URL, e.g. https://amzn.to/...")
                    .kind(CommandOptionType::String)
                    .required(true)
            })
    })
    .await;
}

/// Handler for the `/amazon` command.
/// - Resolves short URLs
/// - Parses ASIN and region
/// - Retrieves tracking tag and footer template
/// - Logs usage in the database
/// - Replies with a plain message: cleaned link + footer
pub async fn run(ctx: &Context, cmd: &ApplicationCommandInteraction) {
    // Check if this is a DM or Guild interaction
    let is_dm = cmd.guild_id.is_none();
    let guild_id = cmd.guild_id.map(|id| id.0.to_string()).unwrap_or_else(|| "DM".to_string());

    // Extract raw URL argument
    let url_raw = if let CommandDataOptionValue::String(u) = &cmd.data.options[0].resolved.as_ref().unwrap() {
        u.clone()
    } else {
        String::new()
    };

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
            let _ = cmd
                .create_interaction_response(&ctx.http, |resp|
                    resp.kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|m| m.content(
                            "No tracking tag available for this region."
                        )))
                .await;
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
            let sender_mention = format!("<@{}>", cmd.user.id.0);
            if footer_template.contains("{{sender}}") {
                footer_template.replace("{{sender}}", &sender_mention)
            } else {
                format!("{} recommended this. {}", sender_mention, footer_template)
            }
        };

        // Send plain message: link + "-# footer"
        let response = format!("{}\n-# {}", clean_url, footer);
        let _ = cmd
            .create_interaction_response(&ctx.http, |resp|
                resp.kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|m| m.content(response))
            )
            .await;
    } else {
        // Parsing failed
        let _ = cmd
            .create_interaction_response(&ctx.http, |resp|
                resp.kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|m| m.content(
                        "Could not parse Amazon URL. Ensure it's valid."
                    )))
            .await;
    }
}