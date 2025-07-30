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

Add the bot to your user installed apps [here](https://discord.com/oauth2/authorize?client_id=1383091487293575270&integration_type=1&scope=applications.commands)!



 

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

* `/configure <region>` — Opens beautiful modal dialog with autocomplete for region selection (Server only)
* `/amazon url:<link>` — Clean & tag your Amazon link (Works in servers and DMs)
* `/stats` — Show total and per-server link counts

**Usage Examples:**

```
# Server configuration (admin only) - Opens beautiful modal with autocomplete
/configure Global Settings    # Configure all regions at once
/configure USA                # Configure just USA marketplace
/configure Germany            # Configure just German marketplace

# Link cleaning (works everywhere)  
/amazon https://amzn.to/xyz123

# Statistics
/stats
```

**🎨 Beautiful Configuration Experience:**
1. **Smart Autocomplete**: Type `/configure` and get instant suggestions with flag emojis
2. **Professional Modal**: Clean, intuitive popup dialog with pre-filled current settings
3. **Region-Specific**: Configure individual marketplaces or use "Global Settings" for all
4. **Visual Feedback**: Immediate confirmation with region count and success status

**Configuration Features:**
- 🏷️ **Tracking Tag Input**: Set your affiliate tag for the selected region
- 💬 **Custom Footer**: Optional personalized message with `{{sender}}` placeholder support
- 🔄 **Smart Pre-filling**: Shows current configuration for easy editing
- 🌍 **Global Mode**: Special option to apply settings to all regions simultaneously

**🌍 Supported Amazon Marketplaces:**
**North America**: 🇺🇸 USA • 🇨🇦 Canada • 🇲🇽 Mexico  
**South America**: 🇧🇷 Brazil  
**Europe**: 🇬🇧 UK • 🇩🇪 Germany • 🇫🇷 France • 🇪🇸 Spain • 🇮🇹 Italy • 🇳🇱 Netherlands • 🇸🇪 Sweden • 🇵🇱 Poland  
**Middle East**: 🇦🇪 UAE • 🇸🇦 Saudi Arabia  
**Asia**: 🇮🇳 India • 🇯🇵 Japan • 🇸🇬 Singapore • 🇨🇳 China  
**Oceania**: 🇦🇺 Australia

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

# Default tracking tags for developer compensation (19 Amazon marketplaces)
# North America
DEFAULT_TRACKING_TAG_COM=your-tag-20        # 🇺🇸 United States
DEFAULT_TRACKING_TAG_CA=your-tag-20         # 🇨🇦 Canada
DEFAULT_TRACKING_TAG_COM_MX=your-tag-20     # 🇲🇽 Mexico

# South America
DEFAULT_TRACKING_TAG_BR=your-tag-20         # 🇧🇷 Brazil

# Europe
DEFAULT_TRACKING_TAG_CO_UK=your-tag-21      # 🇬🇧 United Kingdom
DEFAULT_TRACKING_TAG_DE=your-tag-21         # 🇩🇪 Germany
DEFAULT_TRACKING_TAG_FR=your-tag-21         # 🇫🇷 France
DEFAULT_TRACKING_TAG_ES=your-tag-21         # 🇪🇸 Spain
DEFAULT_TRACKING_TAG_IT=your-tag-21         # 🇮🇹 Italy
DEFAULT_TRACKING_TAG_NL=your-tag-21         # 🇳🇱 Netherlands
DEFAULT_TRACKING_TAG_SE=your-tag-21         # 🇸🇪 Sweden
DEFAULT_TRACKING_TAG_PL=your-tag-21         # 🇵🇱 Poland

# Middle East
DEFAULT_TRACKING_TAG_AE=your-tag-21         # 🇦🇪 UAE
DEFAULT_TRACKING_TAG_SA=your-tag-21         # 🇸🇦 Saudi Arabia

# Asia
DEFAULT_TRACKING_TAG_IN=your-tag-21         # 🇮🇳 India
DEFAULT_TRACKING_TAG_CO_JP=your-tag-22      # 🇯🇵 Japan
DEFAULT_TRACKING_TAG_SG=your-tag-23         # 🇸🇬 Singapore
DEFAULT_TRACKING_TAG_CN=your-tag-23         # 🇨🇳 China

# Oceania
DEFAULT_TRACKING_TAG_COM_AU=your-tag-23     # 🇦🇺 Australia

# Default signature for DMs and fallback
DEFAULT_SIGNATURE="🤖 Powered by Affilify Bot - Supporting developers worldwide!"
```

### Database

* **SQLite DB**: the file referenced by `DATABASE_URL` **must exist** before the bot starts, or create it with `touch bot.db`.
* The bot automatically creates the necessary tables on first run.

---

## 📄 Changelog

### Version 3.0.0 - Professional Modal Interface & Global Amazon Support

#### 🎨 Revolutionary User Experience
- **Smart Autocomplete Interface**: Type `/configure` and get beautiful dropdown with flag emojis
- **Professional Modal Dialogs**: Clean, intuitive popup forms with pre-filled current settings
- **Visual Region Selection**: Instant suggestions with country flags and marketplace names
- **Global Configuration Mode**: Special "Global Settings" option to configure all regions at once
- **Real-time Feedback**: Immediate success confirmation with region counts

#### 🌍 Complete Amazon Marketplace Coverage (19 Regions)
- **North America**: 🇺🇸 USA, 🇨🇦 Canada, 🇲🇽 Mexico
- **South America**: 🇧🇷 Brazil  
- **Europe**: 🇬🇧 UK, 🇩🇪 Germany, 🇫🇷 France, 🇪🇸 Spain, 🇮🇹 Italy, 🇳🇱 Netherlands, 🇸🇪 Sweden, 🇵🇱 Poland
- **Middle East**: 🇦🇪 UAE, 🇸🇦 Saudi Arabia
- **Asia**: 🇮🇳 India, 🇯🇵 Japan, 🇸🇬 Singapore, 🇨🇳 China
- **Oceania**: 🇦🇺 Australia

#### 🔧 Technical Excellence
- **Advanced Autocomplete System**: Dynamic region filtering with fuzzy search
- **Modal Response Handling**: Proper Serenity 0.11 implementation with error handling
- **Smart Database Logic**: Efficient region-specific and global configuration updates
- **Enhanced Interaction Handlers**: Support for autocomplete, modals, and slash commands
- **Comprehensive Environment Setup**: All 19 marketplaces pre-configured in `.env.example`

#### 💡 Usability Improvements
- **Zero Learning Curve**: Intuitive interface that guides users through configuration
- **Context-Aware Inputs**: Smart placeholders and pre-filled values
- **Professional Polish**: Beautiful titles, icons, and visual feedback
- **Flexible Configuration**: Individual regions or global batch updates
- **Error Prevention**: Clear validation and helpful error messages

#### 🔄 Migration from v2.0.0
- **Enhanced Commands**: `/configure` now features professional autocomplete interface
- **Backward Compatible**: All existing configurations preserved and enhanced
- **Improved Environment**: Updated `.env` structure with organized regional groupings
- **Streamlined Setup**: Configuration is now a delightful, guided experience

### Version 2.0.0 - User Install & Developer Fallback Support

#### 🆕 Previous Features
- **User Install Support**: Bot works as both server and user installation
- **Developer Fallback System**: Automatic compensation for unconfigured usage
- **DM Support**: Full functionality in Direct Messages
- **Smart Context Detection**: Automatic DM vs server context handling

---

## 📄 License

This project is licensed under the **MIT License**. See [LICENSE](./LICENSE) for details.



---
![forthebadge](https://forthebadge.com/images/badges/code-written-by-chatgpt-ai-ftw.svg) ![forthebadge](https://forthebadge.com/images/badges/fake-it-make-it-1.svg)
