# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is **Affilify**, a high-performance Discord bot written in Rust using the Serenity framework. The bot automatically cleans and tags Amazon links with affiliate tracking codes, supports both server and user installations (DMs/group chats), and provides per-server configuration with usage statistics.

## Development Commands

### Build & Run
```bash
# Build the project
cargo build

# Run locally (requires .env configuration)
cargo run

# Check for compilation errors without building
cargo check

# Format code
cargo fmt

# Run linter
cargo clippy
```

### Environment Setup
```bash
# Copy and configure environment variables
cp .env.example .env
# Edit .env with your DISCORD_TOKEN and affiliate tags

# Initialize SQLite database (required before first run)
touch bot.db

# Verify compilation
cargo check

# Test run (requires valid DISCORD_TOKEN)
cargo run
```

### Docker Development
```bash
# Build Docker image
docker build -t affilify .

# Run with Docker Compose
docker compose up -d

# View logs
docker compose logs -f affilify
```

## Architecture

### Core Components

**Main Entry Point** (`src/main.rs:95-117`):
- Initializes configuration and database
- Sets up Discord client with necessary intents (GUILDS, GUILD_MESSAGES, DIRECT_MESSAGES, MESSAGE_CONTENT)
- Registers event handlers for interactions and message monitoring

**Event Handling** (`src/main.rs:20-93`):
- **Ready**: Registers all slash commands globally on bot startup
- **Interaction Create**: Routes slash commands, autocomplete, and modal interactions
- **Message**: Monitors guild channels for raw Amazon links and auto-deletes them with helpful hints

**Command Structure** (`src/commands/`):
- **Amazon** (`amazon.rs`): Core `/amazon` command for link processing, supports both guild and DM contexts
- **Configure** (`configure.rs`): Guild-only `/configure` command with autocomplete and modal interfaces
- **Stats** (`stats.rs`): Guild-only `/stats` command showing usage analytics

### Database Schema (`src/db.rs:10-27`)

**Tables**:
- `guild_affiliates`: Maps guild_id + region → tracking_tag 
- `guild_settings`: Maps guild_id → footer_text template
- `link_stats`: Tracks usage with guild_id, region, timestamp

**Connection Pattern**: Uses `with_connection()` wrapper for SQLite operations throughout codebase.

### Configuration System (`src/config.rs`)

**Environment Variables**:
- `DISCORD_TOKEN`: Required Discord bot token
- `DATABASE_URL`: SQLite database path (defaults to `./bot.db`)
- `DEFAULT_TRACKING_TAG_*`: Fallback affiliate tags for 19 Amazon regions
- `DEFAULT_SIGNATURE`: Default footer text for DMs/fallback

**Region Support**: 19 Amazon marketplaces across North America, Europe, Asia, Middle East, and Oceania.

### URL Processing (`src/utils.rs`)

**Key Functions**:
- `resolve_url()`: Follows redirects for short links (amzn.to, etc.) using reqwest with 10-redirect limit
- `parse_amazon_url()`: Extracts ASIN and region from any Amazon URL format using regex pattern `/dp/([A-Z0-9]+)/?`

### Discord Integration Features

**Installation Types**:
- **Guild Install**: Traditional server bot with full permissions
- **User Install**: Personal bot access for DMs and group chats

**Command Contexts**:
- `/amazon`: Works in guilds, DMs, and group chats
- `/configure` & `/stats`: Guild-only with admin permissions

**Interaction Types**:
- Slash commands with autocomplete (region selection)
- Modal dialogs for configuration input
- Automatic message deletion with temporary hints

## Development Guidelines

### Adding New Commands

1. Create new module in `src/commands/`
2. Implement `register_commands()` and `run()` functions
3. Add command registration to `main.rs:26-28`
4. Add interaction routing to `main.rs:34-40`
5. Consider Installation Context: Use `InstallationContext::Guild` for server-only, add `InstallationContext::User` for DM support
6. Set appropriate `InteractionContext` values: `Guild`, `BotDm`, `PrivateChannel`

### Database Operations

Use the `db::with_connection()` pattern for all database access:

```rust
db::with_connection(|conn| {
    conn.execute("INSERT INTO ...", params![...])?;
    Ok(())
})?;
```

### Configuration Access

Access environment variables through `config.rs` functions rather than direct `env::var()` calls:
- `discord_token()`: Gets DISCORD_TOKEN with error handling
- `database_url()`: Gets DATABASE_URL, strips `sqlite://` prefix, defaults to `./bot.db`
- `default_tracking_tag(region)`: Gets `DEFAULT_TRACKING_TAG_{REGION}` with proper formatting
- `default_signature()`: Gets DEFAULT_SIGNATURE with fallback

### Error Handling Patterns

Use `?` operator with proper error propagation:
```rust
// Database operations return rusqlite::Result
db::with_connection(|conn| {
    conn.execute("...", params![...])?;
    Ok(())
})?;

// HTTP requests return reqwest::Result
let resolved = utils::resolve_url(&input_url).await?;
```

### Testing Commands

Requires valid Discord application with proper bot permissions:
- Message Content Intent enabled
- Both Guild Install and User Install configured
- Bot permissions: Read Messages, Send Messages, Manage Messages, Embed Links

### Dependencies

**Core Framework**: Serenity v0.12 for Discord API integration
**Async Runtime**: Tokio with multi-threading support
**HTTP Client**: reqwest with JSON and gzip features for URL resolution
**Database**: rusqlite with bundled SQLite for cross-platform compatibility  
**Environment**: dotenvy for .env file loading
**Utilities**: url crate for parsing, regex for ASIN extraction, chrono for timestamps

### Message Deletion Logic

The bot automatically deletes raw Amazon links in guild channels only (`src/main.rs:58-93`):
- Detects messages containing "amazon." or "amzn.to"
- Skips deletion in DMs (`msg.guild_id.is_none()`)
- Posts temporary hint mentioning user with 10-second auto-delete
- Ignores bot messages to prevent loops