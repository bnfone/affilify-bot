// src/commands/stats.rs
use serenity::all::{
    Command, CommandInteraction, 
    CreateCommand, CreateInteractionResponse, CreateInteractionResponseMessage,
    CreateEmbed, Colour,
    InstallationContext, InteractionContext,
};
use serenity::http::Http;
use serenity::prelude::*;
use rusqlite::params;
use super::super::db;

pub async fn register_commands(http: &Http) {
    let command = CreateCommand::new("stats")
        .description("Show link generation statistics")
        .dm_permission(false)
        // Nur im Server sichtbar machen:
        .integration_types(vec![InstallationContext::Guild])
        .contexts(vec![InteractionContext::Guild]);
    
    let _ = Command::create_global_command(http, command).await;
}

pub async fn run(ctx: &Context, cmd: &CommandInteraction) {
    let guild_id = cmd.guild_id.unwrap().get().to_string();
    let (global_count, guild_count, top_regions) = match db::with_connection(|conn| {
        let global: i64 = conn.query_row("SELECT COUNT(*) FROM link_stats", [], |r| r.get(0))?;
        let local: i64 = conn.query_row("SELECT COUNT(*) FROM link_stats WHERE guild_id = ?", params![guild_id], |r| r.get(0))?;
        
        // Get top 5 regions for this server
        let mut stmt = conn.prepare(
            "SELECT region, COUNT(*) as count FROM link_stats WHERE guild_id = ? GROUP BY region ORDER BY count DESC LIMIT 5"
        )?;
        let regions: Vec<(String, i64)> = stmt.query_map(params![guild_id], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })?.collect::<Result<Vec<_>, _>>()?;
        
        Ok((global, local, regions))
    }) {
        Ok(data) => data,
        Err(e) => {
            eprintln!("Database error in stats command: {}", e);
            let error_response = CreateInteractionResponse::Message(
                CreateInteractionResponseMessage::new()
                    .content("❌ Unable to fetch statistics. Please try again later.")
                    .ephemeral(true)
            );
            let _ = cmd.create_response(&ctx.http, error_response).await;
            return;
        }
    };

    // Build top regions field
    let regions_text = if top_regions.is_empty() {
        "No regions yet".to_string()
    } else {
        top_regions.iter()
            .map(|(region, count)| format!("🌍 **{}**: {} links", region.to_uppercase(), count))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let embed = CreateEmbed::new()
        .title("📊 Affilify Statistics")
        .description("Link generation statistics for this server")
        .field("🌐 Global Total", format!("{} links", global_count), true)
        .field("🏠 This Server", format!("{} links", guild_count), true)
        .field("📈 Top Regions", regions_text, false)
        .colour(Colour::from_rgb(52, 152, 219)) // Nice blue color
        .footer(serenity::all::CreateEmbedFooter::new("Keep sharing those affiliate links! 💰"));

    let response = CreateInteractionResponse::Message(
        CreateInteractionResponseMessage::new()
            .embed(embed)
    );
    let _ = cmd.create_response(&ctx.http, response).await;
}
