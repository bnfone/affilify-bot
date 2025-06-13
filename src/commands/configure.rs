// src/commands/configure.rs
// Handles the `/configure` slash command: sets tracking tags and footer text per region.

use serenity::http::Http;
use serenity::model::application::command::{Command, CommandOptionType};
use serenity::model::application::interaction::application_command::{ApplicationCommandInteraction, CommandDataOptionValue};
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::application::interaction::MessageFlags; // use new MessageFlags for ephemerals
use serenity::model::permissions::Permissions;
use serenity::prelude::*;
use rusqlite::params;
use super::super::db;

/// Register the `/configure` command with region, tag, and optional footer.
pub async fn register_commands(http: &Http) {
    let _ = Command::create_global_application_command(http, |cmd| {
        cmd.name("configure")
            .description("Configure affiliate tracking tags and footer text")
            .create_option(|opt| {
                opt.name("region")
                    .description("Amazon region code (de, com, co.uk, se)")
                    .kind(CommandOptionType::String)
                    .required(true)
            })
            .create_option(|opt| {
                opt.name("tag")
                    .description("Your affiliate tracking tag")
                    .kind(CommandOptionType::String)
                    .required(true)
            })
            .create_option(|opt| {
                opt.name("footer")
                    .description("Custom footer text with optional {{sender}} placeholder")
                    .kind(CommandOptionType::String)
                    .required(false)
            })
    })
    .await;
}

/// Handler for `/configure`. Only guild admins or the server owner can run this.
pub async fn run(ctx: &Context, cmd: &ApplicationCommandInteraction) {
    // Ensure this is in a guild
    let guild_id_u64 = if let Some(guild_id) = cmd.guild_id {
        guild_id.0
    } else {
        let _ = cmd.create_interaction_response(&ctx.http, |r| {
            r.kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|m| m
                    .content("This command can only be used in a server.")
                    .flags(MessageFlags::EPHEMERAL)
                )
        }).await;
        return;
    };

    // Permission check: only server owner or administrators
    let member = if let Some(member) = &cmd.member {
        member
    } else {
        return;
    };
    let perms = member.permissions.unwrap_or(Permissions::empty());
    // Fetch guild to check owner
    let guild = match ctx.http.get_guild(guild_id_u64).await {
        Ok(g) => g,
        Err(_) => return,
    };
    let is_owner = guild.owner_id.0 == cmd.user.id.0;
    if !is_owner && !perms.contains(Permissions::ADMINISTRATOR) {
        let _ = cmd.create_interaction_response(&ctx.http, |r| {
            r.kind(InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|m| m
                    .content("You must be a server administrator or the server owner to run this command.")
                    .flags(MessageFlags::EPHEMERAL)
                )
        }).await;
        return;
    }

    // Parse command options
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

    // Persist to database
    let res = db::with_connection(|conn| {
        conn.execute(
            "INSERT OR REPLACE INTO guild_affiliates (guild_id, region, tracking_tag) VALUES (?, ?, ?)",
            params![guild_id_u64.to_string(), region, tag]
        )?;
        if let Some(f) = footer {
            conn.execute(
                "INSERT OR REPLACE INTO guild_settings (guild_id, footer_text) VALUES (?, ?)",
                params![guild_id_u64.to_string(), f]
            )?;
        }
        Ok(())
    });

    // Feedback
    let content = match res {
        Ok(_) => "Configuration saved!".to_string(),
        Err(e) => format!("Error saving configuration: {:?}", e),
    };
    let _ = cmd.create_interaction_response(&ctx.http, |r| {
        r.kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|m| m
                .content(content)
                .flags(MessageFlags::EPHEMERAL)
            )
    }).await;
}
