// src/utils.rs
use regex::Regex;
use reqwest::Client;
use url::Url;
use rusqlite::params;

pub async fn resolve_url(input: &str) -> reqwest::Result<String> {
    let client = Client::builder().redirect(reqwest::redirect::Policy::limited(10)).build()?;
    let resp = client.get(input).send().await?;
    Ok(resp.url().to_string())
}

/// Parse an Amazon URL and return (ASIN, region)
/// region will be everything after "amazon.", e.g. "de", "co.uk", "com"
pub fn parse_amazon_url(url_str: &str) -> Option<(String, String)> {
    let url = Url::parse(url_str).ok()?;
    let host = url.host_str()?; // e.g. "www.amazon.co.uk" or "smile.amazon.com"
    // strip "www." or any subdomain before "amazon."
    let domain = host
        .trim_start_matches("www.")
        .trim_start_matches("smile.")
        .trim_start_matches("smile-redirect.")
        ; // now "amazon.de", "amazon.co.uk", "amazon.com"...
    // take what comes after "amazon."
    let region = domain
        .strip_prefix("amazon.")
        .unwrap_or("de") // default to "de" if something odd happens
        .to_string();

    // capture the ASIN from a "/dp/ASIN/" path segment
    let re = Regex::new(r"/dp/([A-Z0-9]+)/?").unwrap();
    re.captures(url.path())
      .and_then(|cap| cap.get(1))
      .map(|m| (m.as_str().to_string(), region))
}

/// Extract all Amazon URLs from a message content
pub fn extract_amazon_urls(content: &str) -> Vec<String> {
    let mut urls = Vec::new();
    
    // First find URLs with protocols (http:// or https://)
    // Using a more robust regex that handles complex URLs
    let url_regex = Regex::new(r"https?://[^\s]+").unwrap();
    
    for url_match in url_regex.find_iter(content) {
        let url = url_match.as_str();
        if url.contains("amazon.") || url.contains("amzn.to") {
            urls.push(url.to_string());
        }
    }
    
    // Then find URLs without protocols (amazon.* or amzn.to)
    let amazon_regex = Regex::new(r"(?:^|[\s])((?:amazon\.[a-z.]+|amzn\.to)[^\s]*)").unwrap();
    
    for cap in amazon_regex.captures_iter(content) {
        if let Some(url_match) = cap.get(1) {
            let url = url_match.as_str();
            // Add https:// prefix if not already present
            let full_url = if url.starts_with("http") {
                url.to_string()
            } else {
                format!("https://{}", url)
            };
            
            // Avoid duplicates from the first regex
            if !urls.contains(&full_url) {
                urls.push(full_url);
            }
        }
    }
    
    urls
}

/// Check if a message contains only Amazon links (and whitespace)
pub fn is_amazon_link_only(content: &str) -> bool {
    let trimmed = content.trim();
    
    // Check if the message contains Amazon links
    if !trimmed.contains("amazon.") && !trimmed.contains("amzn.to") {
        return false;
    }
    
    // Remove all URLs (with and without protocols) and check if anything meaningful remains
    let mut remaining = trimmed.to_string();
    
    // Remove URLs with protocols
    let url_regex = Regex::new(r"https?://[^\s]+").unwrap();
    remaining = url_regex.replace_all(&remaining, "").to_string();
    
    // Remove URLs without protocols (amazon.* or amzn.to)
    let amazon_regex = Regex::new(r"(?:^|[\s])((?:amazon\.[a-z.]+|amzn\.to)[^\s]*)").unwrap();
    remaining = amazon_regex.replace_all(&remaining, "").to_string();
    
    // If after removing URLs there's only whitespace, it's a link-only message
    remaining.trim().is_empty()
}

/// Process an Amazon URL and return (clean_url, footer_text)
/// Similar to the amazon command logic but as a utility function
pub async fn process_amazon_url(url: &str, guild_id: Option<String>) -> Option<(String, String)> {
    // Resolve redirects
    let resolved = resolve_url(url).await.unwrap_or_else(|_| url.to_string());
    
    // Parse ASIN and region
    if let Some((asin, region)) = parse_amazon_url(&resolved) {
        let is_dm = guild_id.is_none();
        let guild_id_str = guild_id.unwrap_or_else(|| "DM".to_string());
        
        // Determine tracking tag and footer based on context (same logic as amazon command)
        let (tag, footer_template) = if is_dm {
            // Use default developer tags and signature for DMs
            let default_tag = super::config::default_tracking_tag(&region);
            let default_signature = super::config::default_signature();
            (default_tag, default_signature)
        } else {
            // Try to get guild-specific settings, fallback to defaults
            let default_template = "Using this link you support our server!".to_string();
            
            let (guild_tag, guild_footer) = super::db::with_connection(|conn| {
                let tag: String = conn.query_row(
                    "SELECT tracking_tag FROM guild_affiliates WHERE guild_id = ? AND region = ?",
                    params![guild_id_str, region],
                    |r| r.get(0),
                ).unwrap_or_else(|_| String::new());
                
                let footer: String = conn.query_row(
                        "SELECT footer_text FROM guild_settings WHERE guild_id = ?",
                        params![guild_id_str],
                        |r| r.get(0),
                    )
                    .unwrap_or_else(|_| default_template.clone());
                Ok((tag, footer))
            })
            .unwrap_or((String::new(), default_template.clone()));
            
            // If no guild tag configured, use default developer tag
            if guild_tag.is_empty() {
                let default_tag = super::config::default_tracking_tag(&region);
                let default_signature = super::config::default_signature();
                (default_tag, default_signature)
            } else {
                (guild_tag, guild_footer)
            }
        };
        
        // If still no tag available, return None
        if tag.is_empty() {
            return None;
        }
        
        // Log usage
        let _ = super::db::with_connection(|conn| {
            conn.execute(
                "INSERT INTO link_stats (guild_id, region) VALUES (?, ?)",
                params![guild_id_str, region],
            )
        });
        
        // Build cleaned URL
        let clean_url = format!("https://amazon.{}/dp/{}/?tag={}", region, asin, tag);
        
        Some((clean_url, footer_template))
    } else {
        None
    }
}