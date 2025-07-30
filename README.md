![affilify logo](./.github/logo_cropped.png)


# Affilify — Discord Affiliate Bot in Rust

**Affilify** is a high-performance, multi-guild Discord bot written in Rust. It automatically cleans and tags Amazon links with your affiliate tag, tracks usage statistics, and supports custom footers per server.


![forthebadge](https://forthebadge.com/images/badges/license-mit.svg) ![forthebadge](https://forthebadge.com/images/badges/made-with-rust.svg) ![forthebadge](https://forthebadge.com/images/badges/contains-17-coffee-cups.svg) ![forthebadge](https://forthebadge.com/images/badges/it-works-why.svg)

---

## 📋 Features

* **Affiliate-Link Cleaning & Tagging**: Normalize any Amazon URL (including short links) to a clean `https://amazon.{region}/dp/{ASIN}/?tag={tracking_tag}` format.
* **Short-URL Resolution**: Follows redirects for `amzn.to`, `amzn.eu`, etc.
* **User Install Support**: Works both as server bot and user installation for DMs.
* **Per-Server Configuration**: Admins or server owners can set affiliate tags and custom footer templates via `/configure`.
* **Developer Fallback System**: Uses default developer tracking tags when no server configuration exists, ensuring fair compensation.
* **Custom Footer**: Supports a `{{sender}}` placeholder or defaults to `@user recommended this…`.
* **Usage Statistics**: Logs every `/amazon` invocation and exposes global & per-guild counts via `/stats`.
* **Automatic Hint**: Raw Amazon links in chat are deleted and the user is pinged with a temporary hint to use `/amazon`.
* **Multi-Arch Docker**: Run on x86\_64, ARM64, Raspberry Pi, Apple Silicon, etc.
* **Open Source** under the MIT License.

---

## 🚀 Quick Start

> [!NOTE] 
> If you become an Amazon millionaire thanks to this bot, a [small tip](https://donate.stripe.com/6oE2bm5Y76vG9A47sz) to the dev keeps the code flowing!

[![Invite](https://img.shields.io/badge/-Invite%20Bot-5865F2?logo=discord&logoColor=white)](https://discord.com/oauth2/authorize?client_id=1383091487293575270&permissions=274878000129&integration_type=0&scope=bot)



 

### 1. Clone & Build Locally

```bash
git clone https://github.com/bnfone/discord-bot-affilify.git
cd discord-bot-affilify

# Copy example env and configure your bot
cp .env.example .env
# Edit .env and set your DISCORD_TOKEN
# Configure default tracking IDs for each Amazon region
# Set your preferred DEFAULT_SIGNATURE for DMs

# Create the SQLite file before running
touch bot.db

# Run locally
cargo run
```

### 2. Bot Permissions & Intents

In the Discord Developer Portal, under your Bot settings:

* **Privileged Gateway Intents**:

  * **Message Content Intent** (to detect raw Amazon links)
* **OAuth2 Scopes**:

  * `bot`, `applications.commands`
* **Installation Types** (for User Install support):

  * **Guild Install** (traditional server installation)
  * **User Install** (personal bot access in DMs)
* **Bot Permissions** (when adding to your server):

  * **Read Messages & History**
  * **Send Messages**
  * **Embed Links** (optional, if you later switch back to embeds)
  * **Manage Messages** (to delete raw links)

### 3. Slash Commands

* `/configure region:<code> tag:<your-tag> [footer:<text>]` — Set your affiliate tag and optional custom footer (Server only)
* `/amazon url:<link>` — Clean & tag your Amazon link (Works in servers and DMs)
* `/stats` — Show total and per-server link counts

**Usage Examples:**

```
# Server configuration (admin only)
/configure region: de tag: mytag-21 footer: "{{sender}} recommended this and supports us!"

# Link cleaning (works everywhere)
/amazon https://amzn.to/xyz123

# Statistics
/stats
```

**DM Usage:** When used in Direct Messages, the bot automatically uses your configured default tracking tags and signature, ensuring you get compensated for unconfigured usage.

---

## 🐳 Docker & Docker Compose

[![CI - Build & Push Multi-Arch Docker Image](https://github.com/bnfone/discord-bot-affilify/actions/workflows/docker-multiarch.yml/badge.svg)](https://github.com/bnfone/discord-bot-affilify/actions/workflows/docker-multiarch.yml)

**Dockerfile** builds a multi-stage image:

```dockerfile
# Stage 1: Build with Rust 1.x (Debian Bullseye)
FROM rust:1-bullseye-slim AS builder
... (build steps)

# Stage 2: Runtime (Debian Bullseye)
FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y libssl1.1 ca-certificates
COPY --from=builder /usr/local/bin/discord-bot-affilify /usr/local/bin/
```

**docker-compose.yml** snippet:

```yaml
services:
  affilify:
    build: .
    restart: unless-stopped
    env_file: .env
    volumes:
      - ./bot.db:/app/bot.db
```

Before `docker compose up -d`, run:

```bash
touch bot.db
```

---

## 🤖 GitHub Actions Multi-Arch Build

A workflow triggers on new releases and builds/pushes Docker images for:

* `linux/amd64`
* `linux/arm64`
* `linux/arm/v7`

Register it under `.github/workflows/docker-multiarch.yml` to automatically publish to GitHub Container Registry.

---

## 🔐 Configuration & Data

### Environment Variables

The `.env` file supports the following configuration:

```env
# Required
DISCORD_TOKEN=your_bot_token_here
DATABASE_URL=sqlite://./bot.db

# Default tracking tags for developer compensation
DEFAULT_TRACKING_TAG_DE=your-tag-21      # Germany
DEFAULT_TRACKING_TAG_COM=your-tag-20     # United States
DEFAULT_TRACKING_TAG_CO_UK=your-tag-21   # United Kingdom
DEFAULT_TRACKING_TAG_FR=your-tag-21      # France
DEFAULT_TRACKING_TAG_IT=your-tag-21      # Italy
DEFAULT_TRACKING_TAG_ES=your-tag-21      # Spain
DEFAULT_TRACKING_TAG_CO_JP=your-tag-22   # Japan
DEFAULT_TRACKING_TAG_CA=your-tag-20      # Canada

# Default signature for DMs and fallback
DEFAULT_SIGNATURE="🤖 Powered by Affilify Bot - Supporting developers worldwide!"
```

### Database

* **SQLite DB**: the file referenced by `DATABASE_URL` **must exist** before the bot starts, or create it with `touch bot.db`.
* The bot automatically creates the necessary tables on first run.

---

## 📄 Changelog

### Version 2.0.0 - User Install & Developer Fallback Support

#### 🆕 New Features
- **User Install Support**: Bot now works as both server installation and user installation for DMs
- **Developer Fallback System**: Automatic fallback to developer tracking tags when no server configuration exists
- **DM Support**: Full functionality in Direct Messages with developer compensation
- **Configurable Defaults**: Environment-based default tracking tags and signatures for all Amazon regions
- **Smart Context Detection**: Bot automatically detects DM vs server context and adjusts behavior accordingly

#### 🔧 Technical Changes
- Added `DIRECT_MESSAGES` gateway intent for DM support
- Enhanced `amazon.rs` with DM detection and fallback logic
- New configuration functions in `config.rs` for default tracking tags and signatures
- Updated `.env.example` with comprehensive default tracking configuration for 8 Amazon regions
- Fixed emoji parsing in environment variables (proper quoting required)
- Improved error handling for missing configuration scenarios

#### 🐛 Bug Fixes
- Fixed `.env` parsing error with special characters in `DEFAULT_SIGNATURE`
- Added proper string quoting for environment variables containing emojis or special characters

#### 💡 Benefits
- **Fair Developer Compensation**: Ensures developers get paid even when bot isn't configured on servers
- **Enhanced User Experience**: Seamless functionality across servers and DMs
- **Reduced Configuration Overhead**: Works out-of-the-box with sensible defaults
- **Global Amazon Support**: Pre-configured for major Amazon marketplaces worldwide

#### 🔄 Migration Notes
- **Breaking Change**: `.env` format updated - check `.env.example` for new required variables
- Users upgrading should update their `.env` file with new `DEFAULT_TRACKING_TAG_*` and `DEFAULT_SIGNATURE` variables
- Existing server configurations remain unchanged and take priority over defaults

---

## 📄 License

This project is licensed under the **MIT License**. See [LICENSE](./LICENSE) for details.



---
![forthebadge](https://forthebadge.com/images/badges/code-written-by-chatgpt-ai-ftw.svg) ![forthebadge](https://forthebadge.com/images/badges/fake-it-make-it-1.svg)
