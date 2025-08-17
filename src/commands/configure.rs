// src/commands/configure.rs
// Handles the `/configure` slash command: sets tracking tags and footer text per region.

use serenity::all::{
    Command, CommandInteraction, CommandOptionType, 
    CreateCommand, CreateCommandOption,
    CreateInteractionResponse, CreateInteractionResponseMessage,
    CreateModal, CreateInputText, InputTextStyle, CreateActionRow,
    CreateAutocompleteResponse, Interaction,
    ActionRowComponent, Permissions,
    InstallationContext, InteractionContext,
};
use serenity::http::Http;
use serenity::prelude::*;
use rusqlite::params;
use super::super::db;

/// Register the `/configure` command with autocomplete for regions.
pub async fn register_commands(http: &Http) {
    let command = CreateCommand::new("configure")
        .description("🌍 Configure affiliate tracking for Amazon marketplaces")
        .add_option(
            CreateCommandOption::new(
                CommandOptionType::String,
                "region",
                "Amazon region to configure"
            )
            .required(true)
            .set_autocomplete(true)
        )
        .dm_permission(false)
        // Nur im Server sichtbar machen:
        .integration_types(vec![InstallationContext::Guild])
        .contexts(vec![InteractionContext::Guild]);
    
    let _ = Command::create_global_command(http, command).await;
}

/// Handler for `/configure` command - opens modal for selected region.
pub async fn run(ctx: &Context, cmd: &CommandInteraction) {
    // Ensure this is in a guild
    let guild_id_u64 = if let Some(guild_id) = cmd.guild_id {
        guild_id.get()
    } else {
        let response = CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new()
                .content("This command can only be used in a server.")
                .ephemeral(true)
        );
        let _ = cmd.create_response(&ctx.http, response).await;
        return;
    };

    // Permission check: only server owner or administrators
    let member = if let Some(member) = &cmd.member {
        member
    } else {
        return;
    };
    let perms = member.permissions.unwrap_or(Permissions::empty());
    let guild = match ctx.http.get_guild(guild_id_u64.into()).await {
        Ok(g) => g,
        Err(_) => return,
    };
    let is_owner = guild.owner_id.get() == cmd.user.id.get();
    if !is_owner && !perms.contains(Permissions::ADMINISTRATOR) {
        let response = CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new()
                .content("You must be a server administrator or the server owner to run this command.")
                .ephemeral(true)
        );
        let _ = cmd.create_response(&ctx.http, response).await;
        return;
    }

    // Get selected region
    let region = cmd.data.options.first()
        .and_then(|opt| opt.value.as_str())
        .unwrap_or("global")
        .to_string();

    // Get current configuration
    let current_config = get_current_config(guild_id_u64);
    let current_footer = get_current_footer(guild_id_u64);
    
    // Open configuration modal
    open_config_modal(ctx, cmd, &region, &current_config, &current_footer).await;
}

/// Handle autocomplete for region selection  
pub async fn handle_autocomplete(ctx: &Context, autocomplete: &Interaction) {
    if let Interaction::Autocomplete(auto) = autocomplete {
        // Find the focused option (the one being typed)
        let input = auto.data.options.first()
            .and_then(|opt| opt.value.as_str())
            .unwrap_or("");
        
        let suggestions = get_region_suggestions(input);
        
        let mut response = CreateAutocompleteResponse::new();
        for (value, name) in suggestions.into_iter().take(25) {
            response = response.add_string_choice(&name, &value);
        }
        
        let autocomplete_response = CreateInteractionResponse::Autocomplete(response);
        let _ = auto.create_response(&ctx.http, autocomplete_response).await;
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
    cmd: &CommandInteraction,
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
    
    let modal = CreateModal::new(
        format!("config_modal_{}", region),
        &modal_title
    )
    .components(vec![
        CreateActionRow::InputText(
            CreateInputText::new(
                InputTextStyle::Short,
                format!("🏷️ Tracking Tag for {}", region.to_uppercase()),
                "tracking_tag"
            )
            .placeholder(format!("your-tag-{}", if region.contains("co.") { "21" } else { "20" }))
            .max_length(50)
            .required(false)
            .value(&current_tag)
        ),
        CreateActionRow::InputText(
            CreateInputText::new(
                InputTextStyle::Paragraph,
                "💬 Custom Footer (optional)",
                "footer_text"
            )
            .placeholder("{{sender}} recommended this and supports our server!")
            .max_length(500)
            .required(false)
            .value(&current_footer_text)
        )
    ]);
    
    let response = CreateInteractionResponse::Modal(modal);
    let _ = cmd.create_response(&ctx.http, response).await;
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
pub async fn handle_modal(ctx: &Context, modal: &Interaction) {
    if let Interaction::Modal(modal_submit) = modal {
        let guild_id = if let Some(guild_id) = modal_submit.guild_id {
            guild_id.get()
        } else {
            return;
        };
        
        // Extract region from custom_id
        let region = modal_submit.data.custom_id.strip_prefix("config_modal_")
            .unwrap_or("unknown")
            .to_string();
        
        let guild_id_str = guild_id.to_string();
        
        // Extract form data
        let mut tracking_tag = None;
        let mut footer_text = None;
        
        for action_row in &modal_submit.data.components {
            for component in &action_row.components {
                if let ActionRowComponent::InputText(input) = component {
                    match input.custom_id.as_str() {
                        "tracking_tag" => {
                            if let Some(value) = &input.value {
                                let trimmed = value.trim();
                                if !trimmed.is_empty() {
                                    tracking_tag = Some(trimmed.to_string());
                                }
                            }
                        },
                        "footer_text" => {
                            if let Some(value) = &input.value {
                                let trimmed = value.trim();
                                if !trimmed.is_empty() {
                                    footer_text = Some(trimmed.to_string());
                                }
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
    
        let response = CreateInteractionResponse::Message(
            CreateInteractionResponseMessage::new()
                .content(content)
                .ephemeral(true)
        );
        let _ = modal_submit.create_response(&ctx.http, response).await;
    }
}
