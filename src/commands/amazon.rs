// src/commands/amazon.rs
// Handles the `/amazon` slash command: cleans URLs, tags them, and logs usage.

use serenity::http::Http;
use serenity::model::application::command::{Command, CommandOptionType};
use serenity::model::application::interaction::application_command::{ApplicationCommandInteraction, CommandDataOptionValue};
use serenity::model::application::interaction::InteractionResponseType;
use serenity::prelude::*;
use rusqlite::params;
use super::super::{db, utils};

/// Register the `/amazon` command with a required URL option.
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
/// - Retrieves tracking tag and footer for the guild
/// - Logs the usage in the database
/// - Returns a cleaned, tagged embed to the user
pub async fn run(ctx: &Context, cmd: &ApplicationCommandInteraction) {
    // Convert guild ID to string
    let guild_id = cmd.guild_id.unwrap().0.to_string();

    // Extract the raw URL from the first option
    let url_raw = if let CommandDataOptionValue::String(u) =
        &cmd.data.options[0].resolved.as_ref().unwrap()
    {
        u.clone()
    } else {
        "".to_string()
    };

    // Follow redirects if it's a short URL
    let resolved = utils::resolve_url(&url_raw).await.unwrap_or_else(|_| url_raw.clone());

    // Parse the ASIN and region (e.g. ("B0EXAMPLE", "de"))
    if let Some((asin, region)) = utils::parse_amazon_url(&resolved) {
        // Fetch tracking tag and footer from the database
        let (tag, footer) = db::with_connection(|conn| {
            let tag: String = conn.query_row(
                "SELECT tracking_tag FROM guild_affiliates WHERE guild_id = ? AND region = ?",
                params![guild_id, region],
                |r| r.get(0),
            )?;
            let footer: String = conn
                .query_row(
                    "SELECT footer_text FROM guild_settings WHERE guild_id = ?",
                    params![guild_id],
                    |r| r.get(0),
                )
                .unwrap_or_else(|_| "Using this link you support our server!".to_string());
            Ok((tag, footer))
        })
        .unwrap_or_else(|_| ("".to_string(), "Using this link you support our server!".to_string()));

        // If no tag is set for this region, inform the user
        if tag.is_empty() {
            let _ = cmd
                .create_interaction_response(&ctx.http, |resp|
                    resp.kind(InteractionResponseType::ChannelMessageWithSource)
                        .interaction_response_data(|m| m.content(
                            "No tracking tag configured for this region. Ask an admin to run `/configure`.",
                        )))
                .await;
            return;
        }

        // Log this link usage
        let _ = db::with_connection(|conn| {
            conn.execute(
                "INSERT INTO link_stats (guild_id, region) VALUES (?, ?)",
                params![guild_id, region],
            )
        });

        // Build the cleaned, tagged URL
        let clean_url = format!("https://amazon.{}/dp/{}/?tag={}", region, asin, tag);

        // Respond with an embed containing the affiliate link
        let _ = cmd
            .create_interaction_response(&ctx.http, |resp| {
                resp.kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|m|
                        m.embed(|e| {
                            e.title("Your affiliate link")
                                .description(clean_url)
                                .footer(|f| f.text(footer))
                        }),
                    )
            })
            .await;
    } else {
        // URL parsing failed
        let _ = cmd
            .create_interaction_response(&ctx.http, |resp|
                resp.kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|m| m.content(
                        "Could not parse Amazon URL. Ensure it's a valid Amazon link.",
                    )))
            .await;
    }
}