// src/commands/configure.rs
// Handles the `/configure` slash command: sets tracking tags and footer text per region.

use serenity::http::Http;
use serenity::model::application::command::Command;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::application::interaction::modal::ModalSubmitInteraction;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::application::interaction::MessageFlags;
use serenity::model::permissions::Permissions;
use serenity::prelude::*;
use rusqlite::params;
use super::super::db;

/// Register the `/configure` command - now opens a modal dialog.
pub async fn register_commands(http: &Http) {
    let _ = Command::create_global_application_command(http, |cmd| {
        cmd.name("configure")
            .description("Configure affiliate tracking tags and footer text")
    })
    .await;
}

/// Handler for `/configure`. Opens a modal dialog for multi-region configuration.
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

    // Get current configuration for pre-filling
    let current_config = get_current_config(guild_id_u64);

    // Show modal dialog
    let _ = cmd.create_interaction_response(&ctx.http, |r| {
        r.kind(InteractionResponseType::Modal)
            .interaction_response_data(|m| {
                m.custom_id("configure_modal")
                    .title("Configure Affiliate Settings")
                    .components(|c| {
                        c.create_action_row(|row| {
                            row.create_input_text(|input| {
                                input.custom_id("tag_de")
                                    .label("Germany (amazon.de) - Tracking Tag")
                                    .style(serenity::model::application::component::InputTextStyle::Short)
                                    .placeholder("your-tag-21")
                                    .max_length(50)
                                    .required(false)
                                    .value(&current_config.0.unwrap_or_default())
                            })
                        })
                        .create_action_row(|row| {
                            row.create_input_text(|input| {
                                input.custom_id("tag_com")
                                    .label("USA (amazon.com) - Tracking Tag")
                                    .style(serenity::model::application::component::InputTextStyle::Short)
                                    .placeholder("your-tag-20")
                                    .max_length(50)
                                    .required(false)
                                    .value(&current_config.1.unwrap_or_default())
                            })
                        })
                        .create_action_row(|row| {
                            row.create_input_text(|input| {
                                input.custom_id("tag_couk")
                                    .label("UK (amazon.co.uk) - Tracking Tag")
                                    .style(serenity::model::application::component::InputTextStyle::Short)
                                    .placeholder("your-tag-21")
                                    .max_length(50)
                                    .required(false)
                                    .value(&current_config.2.unwrap_or_default())
                            })
                        })
                        .create_action_row(|row| {
                            row.create_input_text(|input| {
                                input.custom_id("tag_fr")
                                    .label("France (amazon.fr) - Tracking Tag")
                                    .style(serenity::model::application::component::InputTextStyle::Short)
                                    .placeholder("your-tag-21")
                                    .max_length(50)
                                    .required(false)
                                    .value(&current_config.3.unwrap_or_default())
                            })
                        })
                        .create_action_row(|row| {
                            row.create_input_text(|input| {
                                input.custom_id("footer_text")
                                    .label("Custom Footer (optional)")
                                    .style(serenity::model::application::component::InputTextStyle::Paragraph)
                                    .placeholder("{{sender}} recommended this and supports our server!")
                                    .max_length(200)
                                    .required(false)
                                    .value(&current_config.4.unwrap_or_default())
                            })
                        })
                    })
            })
    }).await;
}

/// Get current configuration for pre-filling the modal
fn get_current_config(guild_id: u64) -> (Option<String>, Option<String>, Option<String>, Option<String>, Option<String>) {
    let guild_id_str = guild_id.to_string();
    
    let tags = db::with_connection(|conn| {
        let mut de_tag = None;
        let mut com_tag = None;
        let mut couk_tag = None;
        let mut fr_tag = None;
        
        // Get tracking tags for each region
        if let Ok(tag) = conn.query_row(
            "SELECT tracking_tag FROM guild_affiliates WHERE guild_id = ? AND region = ?",
            params![guild_id_str, "de"],
            |r| r.get::<_, String>(0),
        ) { de_tag = Some(tag); }
        
        if let Ok(tag) = conn.query_row(
            "SELECT tracking_tag FROM guild_affiliates WHERE guild_id = ? AND region = ?",
            params![guild_id_str, "com"],
            |r| r.get::<_, String>(0),
        ) { com_tag = Some(tag); }
        
        if let Ok(tag) = conn.query_row(
            "SELECT tracking_tag FROM guild_affiliates WHERE guild_id = ? AND region = ?",
            params![guild_id_str, "co.uk"],
            |r| r.get::<_, String>(0),
        ) { couk_tag = Some(tag); }
        
        if let Ok(tag) = conn.query_row(
            "SELECT tracking_tag FROM guild_affiliates WHERE guild_id = ? AND region = ?",
            params![guild_id_str, "fr"],
            |r| r.get::<_, String>(0),
        ) { fr_tag = Some(tag); }
        
        Ok((de_tag, com_tag, couk_tag, fr_tag))
    }).unwrap_or((None, None, None, None));
    
    // Get footer text
    let footer = db::with_connection(|conn| {
        Ok(conn.query_row(
            "SELECT footer_text FROM guild_settings WHERE guild_id = ?",
            params![guild_id_str],
            |r| r.get::<_, String>(0),
        ).ok())
    }).unwrap_or(None);
    
    (tags.0, tags.1, tags.2, tags.3, footer)
}

/// Handle modal submission
pub async fn handle_modal(ctx: &Context, modal: &ModalSubmitInteraction) {
    let guild_id = if let Some(guild_id) = modal.guild_id {
        guild_id.0
    } else {
        return;
    };
    
    let guild_id_str = guild_id.to_string();
    
    // Extract form data
    let mut updates = Vec::new();
    let mut footer_text = None;
    
    for row in &modal.data.components {
        for component in &row.components {
            if let serenity::model::application::component::ActionRowComponent::InputText(input) = component {
                let value = input.value.trim();
                if !value.is_empty() {
                    match input.custom_id.as_str() {
                        "tag_de" => updates.push(("de".to_string(), value.to_string())),
                        "tag_com" => updates.push(("com".to_string(), value.to_string())),
                        "tag_couk" => updates.push(("co.uk".to_string(), value.to_string())),
                        "tag_fr" => updates.push(("fr".to_string(), value.to_string())),
                        "footer_text" => footer_text = Some(value.to_string()),
                        _ => {}
                    }
                }
            }
        }
    }
    
    // Update database
    let res = db::with_connection(|conn| {
        // Update tracking tags
        for (region, tag) in updates {
            conn.execute(
                "INSERT OR REPLACE INTO guild_affiliates (guild_id, region, tracking_tag) VALUES (?, ?, ?)",
                params![guild_id_str, region, tag]
            )?;
        }
        
        // Update footer if provided
        if let Some(footer) = footer_text {
            conn.execute(
                "INSERT OR REPLACE INTO guild_settings (guild_id, footer_text) VALUES (?, ?)",
                params![guild_id_str, footer]
            )?;
        }
        
        Ok(())
    });
    
    // Send response
    let content = match res {
        Ok(_) => "✅ Configuration updated successfully!".to_string(),
        Err(e) => format!("❌ Error saving configuration: {:?}", e),
    };
    
    let _ = modal.create_interaction_response(&ctx.http, |r| {
        r.kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|m| m
                .content(content)
                .flags(MessageFlags::EPHEMERAL)
            )
    }).await;
}
