// src/commands/stats.rs
use serenity::model::application::command::Command;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::http::Http;
use serenity::prelude::*;
use rusqlite::params;
use super::super::db;

pub async fn register_commands(http: &Http) {
    let _ = Command::create_global_application_command(http, |cmd| {
        cmd.name("stats").description("Show link generation statistics")
    }).await;
}

pub async fn run(ctx: &Context, cmd: &ApplicationCommandInteraction) {
    let guild_id = cmd.guild_id.unwrap().0.to_string();
    let (global_count, guild_count) = db::with_connection(|conn| {
        let global: i64 = conn.query_row("SELECT COUNT(*) FROM link_stats", [], |r| r.get(0))?;
        let local: i64 = conn.query_row("SELECT COUNT(*) FROM link_stats WHERE guild_id = ?", params![guild_id], |r| r.get(0))?;
        Ok((global, local))
    }).unwrap();

    let content = format!(
        "Total links generated: {}\nThis server: {}",
        global_count, guild_count
    );
    let _ = cmd.create_interaction_response(&ctx.http, |resp| resp
        .kind(InteractionResponseType::ChannelMessageWithSource)
        .interaction_response_data(|m| m.content(content))
    ).await;
}
