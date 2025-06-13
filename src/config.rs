// src/config.rs
use dotenvy::dotenv;
use std::env;

pub fn init() -> Result<(), dotenvy::Error> {
    dotenv().map(|_| ())
}

pub fn discord_token() -> String {
    env::var("DISCORD_TOKEN").expect("Expected DISCORD_TOKEN in .env")
}

pub fn database_url() -> String {
    // Get DATABASE_URL and strip sqlite:// prefix if present
    let raw = std::env::var("DATABASE_URL").unwrap_or_else(|_| "./bot.db".into());
    raw.strip_prefix("sqlite://").unwrap_or(&raw).to_string()
}