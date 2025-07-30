// src/commands/configure.rs
// Handles the `/configure` slash command: sets tracking tags and footer text per region.

use serenity::http::Http;
use serenity::model::application::command::Command;
use serenity::model::application::interaction::application_command::ApplicationCommandInteraction;
use serenity::model::application::interaction::InteractionResponseType;
use serenity::model::application::interaction::MessageFlags;
use serenity::model::permissions::Permissions;
use serenity::prelude::*;
use rusqlite::params;
use super::super::db;

/// Register the `/configure` command with autocomplete for regions.
pub async fn register_commands(http: &Http) {
    let _ = Command::create_global_application_command(http, |cmd| {
        cmd.name("configure")
            .description("🌍 Configure affiliate tracking for Amazon marketplaces")
            .create_option(|opt| {
                opt.name("region")
                    .description("Amazon region to configure")
                    .kind(serenity::model::application::command::CommandOptionType::String)
                    .required(true)
                    .set_autocomplete(true)
            })
    })
    .await;
}

/// Handler for `/configure` command - opens modal for selected region.
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

    // Get selected region
    let region = if let Some(option) = cmd.data.options.get(0) {
        if let Some(serenity::model::application::interaction::application_command::CommandDataOptionValue::String(r)) = &option.resolved {
            r.clone()
        } else {
            "global".to_string()
        }
    } else {
        "global".to_string()
    };

    // Get current configuration
    let current_config = get_current_config(guild_id_u64);
    let current_footer = get_current_footer(guild_id_u64);
    
    // Open configuration modal
    open_config_modal(ctx, cmd, &region, &current_config, &current_footer).await;
}

/// Handle autocomplete for region selection  
pub async fn handle_autocomplete(ctx: &Context, autocomplete: &serenity::model::application::interaction::autocomplete::AutocompleteInteraction) {
    let focused_option = autocomplete.data.options.iter()
        .find(|opt| opt.focused);
    
    if let Some(option) = focused_option {
        if option.name == "region" {
            let input = option.value.as_ref().and_then(|v| v.as_str()).unwrap_or("");
            let suggestions = get_region_suggestions(input);
            
            let _ = autocomplete.create_autocomplete_response(&ctx.http, |response| {
                for (value, name) in suggestions.into_iter().take(25) {
                    response.add_string_choice(&name, &value);
                }
                response
            }).await;
        }
    }
}

/// Get region suggestions for autocomplete
fn get_region_suggestions(input: &str) -> Vec<(String, String)> {
    let regions = vec![
        ("global".to_string(), "🌍 Global Settings (All Regions)".to_string()),
        ("com".to_string(), "🇺🇸 USA (amazon.com)".to_string()),
        ("ca".to_string(), "🇨🇦 Canada (amazon.ca)".to_string()),
        ("com.mx".to_string(), "🇲🇽 Mexico (amazon.com.mx)".to_string()),
        ("br".to_string(), "🇧🇷 Brazil (amazon.br)".to_string()),
        ("co.uk".to_string(), "🇬🇧 UK (amazon.co.uk)".to_string()),
        ("de".to_string(), "🇩🇪 Germany (amazon.de)".to_string()),
        ("fr".to_string(), "🇫🇷 France (amazon.fr)".to_string()),
        ("es".to_string(), "🇪🇸 Spain (amazon.es)".to_string()),
        ("it".to_string(), "🇮🇹 Italy (amazon.it)".to_string()),
        ("nl".to_string(), "🇳🇱 Netherlands (amazon.nl)".to_string()),
        ("se".to_string(), "🇸🇪 Sweden (amazon.se)".to_string()),
        ("pl".to_string(), "🇵🇱 Poland (amazon.pl)".to_string()),
        ("ae".to_string(), "🇦🇪 UAE (amazon.ae)".to_string()),
        ("sa".to_string(), "🇸🇦 Saudi Arabia (amazon.sa)".to_string()),
        ("in".to_string(), "🇮🇳 India (amazon.in)".to_string()),
        ("co.jp".to_string(), "🇯🇵 Japan (amazon.co.jp)".to_string()),
        ("sg".to_string(), "🇸🇬 Singapore (amazon.sg)".to_string()),
        ("cn".to_string(), "🇨🇳 China (amazon.cn)".to_string()),
        ("com.au".to_string(), "🇦🇺 Australia (amazon.com.au)".to_string()),
    ];
    
    let input_lower = input.to_lowercase();
    regions.into_iter()
        .filter(|(code, name)| {
            code.contains(&input_lower) || name.to_lowercase().contains(&input_lower)
        })
        .take(25)
        .collect()
}

/// Open configuration modal for selected region
async fn open_config_modal(
    ctx: &Context,
    cmd: &ApplicationCommandInteraction,
    region: &str,
    current_config: &std::collections::HashMap<String, String>,
    current_footer: &Option<String>
) {
    let modal_title = if region == "global" {
        "🌍 Global Amazon Configuration".to_string()
    } else {
        format!("🌍 Configure Amazon {}", region.to_uppercase())
    };
    
    let current_tag = current_config.get(region).cloned().unwrap_or_default();
    let current_footer_text = current_footer.clone().unwrap_or_default();
    
    let _ = cmd.create_interaction_response(&ctx.http, |response| {
        response.kind(InteractionResponseType::Modal)
            .interaction_response_data(|data| {
                data.custom_id(format!("config_modal_{}", region))
                    .title(&modal_title)
                    .components(|c| {
                        c.create_action_row(|row| {
                            row.create_input_text(|input| {
                                input.custom_id("tracking_tag")
                                    .label(format!("🏷️ Tracking Tag for {}", region.to_uppercase()))
                                    .style(serenity::model::application::component::InputTextStyle::Short)
                                    .placeholder(&format!("your-tag-{}", if region.contains("co.") { "21" } else { "20" }))
                                    .max_length(50)
                                    .required(false)
                                    .value(&current_tag)
                            })
                        })
                        .create_action_row(|row| {
                            row.create_input_text(|input| {
                                input.custom_id("footer_text")
                                    .label("💬 Custom Footer (optional)")
                                    .style(serenity::model::application::component::InputTextStyle::Paragraph)
                                    .placeholder("{{sender}} recommended this and supports our server!")
                                    .max_length(500)
                                    .required(false)
                                    .value(&current_footer_text)
                            })
                        })
                    })
            })
    }).await;
}

/// Get current configuration for pre-filling the modal
fn get_current_config(guild_id: u64) -> std::collections::HashMap<String, String> {
    let guild_id_str = guild_id.to_string();
    
    let mut config = std::collections::HashMap::new();
    
    // Get all tracking tags for this guild
    let _ = db::with_connection(|conn| {
        let mut stmt = conn.prepare("SELECT region, tracking_tag FROM guild_affiliates WHERE guild_id = ?")?;
        let rows = stmt.query_map(params![guild_id_str], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        
        for row in rows {
            if let Ok((region, tag)) = row {
                config.insert(region, tag);
            }
        }
        
        Ok(())
    });
    
    config
}

/// Format current configuration as text for the modal
fn format_current_tags(config: &std::collections::HashMap<String, String>) -> String {
    if config.is_empty() {
        return String::new();
    }
    
    let regions = [
        ("com", "🇺🇸 USA"),
        ("ca", "🇨🇦 Canada"), 
        ("com.mx", "🇲🇽 Mexico"),
        ("br", "🇧🇷 Brazil"),
        ("co.uk", "🇬🇧 UK"),
        ("de", "🇩🇪 Germany"),
        ("fr", "🇫🇷 France"),
        ("es", "🇪🇸 Spain"),
        ("it", "🇮🇹 Italy"),
        ("nl", "🇳🇱 Netherlands"),
        ("se", "🇸🇪 Sweden"),
        ("pl", "🇵🇱 Poland"),
        ("ae", "🇦🇪 UAE"),
        ("sa", "🇸🇦 Saudi Arabia"),
        ("in", "🇮🇳 India"),
        ("co.jp", "🇯🇵 Japan"),
        ("sg", "🇸🇬 Singapore"),
        ("cn", "🇨🇳 China"),
        ("com.au", "🇦🇺 Australia"),
    ];
    
    let mut result = Vec::new();
    for (region_code, region_name) in regions.iter() {
        if let Some(tag) = config.get(*region_code) {
            result.push(format!("{}:{}  # {}", region_code, tag, region_name));
        }
    }
    
    result.join("\n")
}

/// Get current footer text
fn get_current_footer(guild_id: u64) -> Option<String> {
    let guild_id_str = guild_id.to_string();
    
    db::with_connection(|conn| {
        Ok(conn.query_row(
            "SELECT footer_text FROM guild_settings WHERE guild_id = ?",
            params![guild_id_str],
            |r| r.get::<_, String>(0),
        ).ok())
    }).unwrap_or(None)
}

/// Handle modal submission for configuration
pub async fn handle_modal(ctx: &Context, modal: &serenity::model::application::interaction::modal::ModalSubmitInteraction) {
    let guild_id = if let Some(guild_id) = modal.guild_id {
        guild_id.0
    } else {
        return;
    };
    
    // Extract region from custom_id
    let region = modal.data.custom_id.strip_prefix("config_modal_")
        .unwrap_or("unknown")
        .to_string();
    
    let guild_id_str = guild_id.to_string();
    
    // Extract form data
    let mut tracking_tag = None;
    let mut footer_text = None;
    
    for action_row in &modal.data.components {
        for component in &action_row.components {
            if let serenity::model::application::component::ActionRowComponent::InputText(input) = component {
                match input.custom_id.as_str() {
                    "tracking_tag" => {
                        let value = input.value.trim();
                        if !value.is_empty() {
                            tracking_tag = Some(value.to_string());
                        }
                    },
                    "footer_text" => {
                        let value = input.value.trim();
                        if !value.is_empty() {
                            footer_text = Some(value.to_string());
                        }
                    },
                    _ => {}
                }
            }
        }
    }
    
    // Update database
    let res = db::with_connection(|conn| {
        let mut updates = 0;
        
        // Handle tracking tag
        if let Some(tag) = tracking_tag {
            if region == "global" {
                // Update all existing regions with the same tag
                let mut stmt = conn.prepare("SELECT DISTINCT region FROM guild_affiliates WHERE guild_id = ?")?;
                let regions: Vec<String> = stmt.query_map(params![guild_id_str], |row| {
                    Ok(row.get::<_, String>(0)?)
                })?.collect::<Result<Vec<_>, _>>()?;
                
                for r in regions {
                    conn.execute(
                        "UPDATE guild_affiliates SET tracking_tag = ? WHERE guild_id = ? AND region = ?",
                        params![tag, guild_id_str, r]
                    )?;
                    updates += 1;
                }
                
                if updates == 0 {
                    // No existing regions, add popular ones
                    for default_region in ["com", "de", "co.uk", "fr"] {
                        conn.execute(
                            "INSERT OR REPLACE INTO guild_affiliates (guild_id, region, tracking_tag) VALUES (?, ?, ?)",
                            params![guild_id_str, default_region, tag]
                        )?;
                        updates += 1;
                    }
                }
            } else {
                conn.execute(
                    "INSERT OR REPLACE INTO guild_affiliates (guild_id, region, tracking_tag) VALUES (?, ?, ?)",
                    params![guild_id_str, region, tag]
                )?;
                updates += 1;
            }
        }
        
        // Handle footer
        if let Some(footer) = footer_text {
            conn.execute(
                "INSERT OR REPLACE INTO guild_settings (guild_id, footer_text) VALUES (?, ?)",
                params![guild_id_str, footer]
            )?;
            updates += 1;
        }
        
        Ok(updates)
    });
    
    // Send response
    let content = match res {
        Ok(updates) if updates > 0 => {
            if region == "global" {
                format!("✅ Global configuration updated!\n🌍 {} regions configured", updates)
            } else {
                format!("✅ Configuration updated for {}!\n🌍 {} items configured", region.to_uppercase(), updates)
            }
        },
        Ok(_) => "ℹ️ No changes made.".to_string(),
        Err(e) => format!("❌ Error saving configuration: {:?}", e),
    };
    
    let _ = modal.create_interaction_response(&ctx.http, |r| {
        r.kind(InteractionResponseType::ChannelMessageWithSource)
            .interaction_response_data(|m| {
                m.content(content)
                    .flags(MessageFlags::EPHEMERAL)
            })
    }).await;
}
