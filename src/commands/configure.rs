// src/commands/configure.rs
use serenity::model::application::command::{Command, CommandOptionType};
use serenity::model::application::interaction::application_command::{ApplicationCommandInteraction, CommandDataOptionValue};
use serenity::model::application::interaction::InteractionResponseType;
use serenity::http::Http;
use serenity::prelude::*;
use rusqlite::params;
use super::super::db;

pub async fn register_commands(http: &Http) {
    let _ = Command::create_global_application_command(http, |cmd| {
        cmd.name("configure").description("Configure affiliate tracking tags and footer text")
            .create_option(|opt| {
                opt.name("region").description("Amazon region code (de, com, co.uk, se)")
                    .kind(CommandOptionType::String)
                    .required(true)
            })
            .create_option(|opt| {
                opt.name("tag").description("Your affiliate tracking tag")
                    .kind(CommandOptionType::String)
                    .required(true)
            })
            .create_option(|opt| {
                opt.name("footer").description("Custom footer text")
                    .kind(CommandOptionType::String)
                    .required(false)
            })
    }).await;
}

pub async fn run(ctx: &Context, cmd: &ApplicationCommandInteraction) {
    let guild_id = cmd.guild_id.unwrap().0.to_string();
    let mut region = String::new();
    let mut tag = String::new();
    let mut footer = None;

    for option in &cmd.data.options {
        match option.name.as_str() {
            "region" => if let CommandDataOptionValue::String(r) = &option.resolved.as_ref().unwrap() { region = r.clone(); },
            "tag"    => if let CommandDataOptionValue::String(t) = &option.resolved.as_ref().unwrap() { tag = t.clone(); },
            "footer" => if let CommandDataOptionValue::String(f) = &option.resolved.as_ref().unwrap() { footer = Some(f.clone()); },
            _ => {}
        }
    }

    let res = db::with_connection(|conn| {
        conn.execute(
            "INSERT OR REPLACE INTO guild_affiliates (guild_id, region, tracking_tag) VALUES (?, ?, ?)",
            params![guild_id, region, tag]
        )?;
        if let Some(f) = footer {
            conn.execute(
                "INSERT OR REPLACE INTO guild_settings (guild_id, footer_text) VALUES (?, ?)",
                params![guild_id, f]
            )?;
        }
        Ok(())
    });

    let content = match res {
        Ok(_) => "Configuration saved!".to_string(),
        Err(e) => format!("Error saving configuration: {:?}", e),
    };

    let _ = cmd.create_interaction_response(&ctx.http, |resp| resp
        .kind(InteractionResponseType::ChannelMessageWithSource)
        .interaction_response_data(|msg| msg.content(content))
    ).await;
}