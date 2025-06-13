![affilify logo](./.github/logo_cropped.png)


# Affilify — Discord Affiliate Bot in Rust

**Affilify** is a high-performance, multi-guild Discord bot written in Rust. It automatically cleans and tags Amazon links with your affiliate tag, tracks usage statistics, and supports custom footers per server.



![forthebadge](https://forthebadge.com/images/badges/license-mit.svg) ![forthebadge](https://forthebadge.com/images/badges/made-with-rust.svg) ![forthebadge](https://forthebadge.com/images/badges/contains-17-coffee-cups.svg) ![forthebadge](https://forthebadge.com/images/badges/it-works-why.svg)

---

## 📋 Features

* **Affiliate-Link Cleaning & Tagging**: Normalize any Amazon URL (including short links) to a clean `https://amazon.{region}/dp/{ASIN}/?tag={tracking_tag}` format.
* **Short-URL Resolution**: Follows redirects for `amzn.to`, `amzn.eu`, etc.
* **Per-Server Configuration**: Admins or server owners can set affiliate tags and custom footer templates via `/configure`.
* **Custom Footer**: Supports a `{{sender}}` placeholder or defaults to `@user recommended this…`.
* **Usage Statistics**: Logs every `/amazon` invocation and exposes global & per-guild counts via `/stats`.
* **Automatic Hint**: Raw Amazon links in chat are deleted and the user is pinged with a temporary hint to use `/amazon`.
* **Multi-Arch Docker**: Run on x86\_64, ARM64, Raspberry Pi, Apple Silicon, etc.
* **Open Source** under the MIT License.

---

## 🚀 Quick Start


Bot Invitation Link

Invite Affilify to your server:

> [!INFO] 
> If you become an Amazon millionaire thanks to this bot, a [small tip](https://donate.stripe.com/6oE2bm5Y76vG9A47sz) to the dev keeps the code flowing! 

### 1. Clone & Build Locally

```bash
git clone https://github.com/<your-org>/discord-bot-affilify.git
cd discord-bot-affilify

# Copy example env and fill in your token
cp .env.example .env
# Ensure DATABASE_URL points to a file path
# e.g. DATABASE_URL=./bot.db

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
* **Bot Permissions** (when adding to your server):

  * **Read Messages & History**
  * **Send Messages**
  * **Embed Links** (optional, if you later switch back to embeds)
  * **Manage Messages** (to delete raw links)

### 3. Slash Commands

* `/configure region:<code> tag:<your-tag> [footer:<text>]` — Set your affiliate tag and optional custom footer.
* `/amazon url:<link>` — Clean & tag your Amazon link.
* `/stats` — Show total and per-server link counts.

Example:

```
/configure region: de tag: mytag-21 footer: "{{sender}} recommended this and supports us!"
/amazon https://amzn.to/xyz123
/stats
```

---

## 🐳 Docker & Docker Compose

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

* **`.env`-file**: store `DISCORD_TOKEN` and `DATABASE_URL`.
* **SQLite DB**: the file referenced by `DATABASE_URL` **must exist** before the bot starts, or create it with `touch bot.db`.

---

## 📄 License

This project is licensed under the **MIT License**. See [LICENSE](./LICENSE) for details.



---
![forthebadge](https://forthebadge.com/images/badges/code-written-by-chatgpt-ai-ftw.svg) ![forthebadge](https://forthebadge.com/images/badges/fake-it-make-it-1.svg) ![forthebadge](data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIxMjguNjU2MjY5MDczNDg2MzMiIGhlaWdodD0iMzUiIHZpZXdCb3g9IjAgMCAxMjguNjU2MjY5MDczNDg2MzMgMzUiPjxyZWN0IHdpZHRoPSI1NS4zMzU5NDUxMjkzOTQ1MyIgaGVpZ2h0PSIzNSIgZmlsbD0iIzMxQzRGMyIvPjxyZWN0IHg9IjU1LjMzNTk0NTEyOTM5NDUzIiB3aWR0aD0iNzMuMzIwMzIzOTQ0MDkxOCIgaGVpZ2h0PSIzNSIgZmlsbD0iIzM4OUFENSIvPjx0ZXh0IHg9IjI3LjY2Nzk3MjU2NDY5NzI2NiIgeT0iMjEuNSIgZm9udC1zaXplPSIxMiIgZm9udC1mYW1pbHk9IidSb2JvdG8nLCBzYW5zLXNlcmlmIiBmaWxsPSIjRkZGRkZGIiB0ZXh0LWFuY2hvcj0ibWlkZGxlIiBsZXR0ZXItc3BhY2luZz0iMiI+VklCRTwvdGV4dD48dGV4dCB4PSI5MS45OTYxMDcxMDE0NDA0MyIgeT0iMjEuNSIgZm9udC1zaXplPSIxMiIgZm9udC1mYW1pbHk9IidNb250c2VycmF0Jywgc2Fucy1zZXJpZiIgZmlsbD0iI0ZGRkZGRiIgdGV4dC1hbmNob3I9Im1pZGRsZSIgZm9udC13ZWlnaHQ9IjkwMCIgbGV0dGVyLXNwYWNpbmc9IjIiPkNPREVEPC90ZXh0Pjwvc3ZnPg==) ![forthebadge](data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciIHdpZHRoPSIxODEuNzQyMjAyNzU4Nzg5MDYiIGhlaWdodD0iMzUiIHZpZXdCb3g9IjAgMCAxODEuNzQyMjAyNzU4Nzg5MDYgMzUiPjxyZWN0IHdpZHRoPSI4My4zMjAzMjAxMjkzOTQ1MyIgaGVpZ2h0PSIzNSIgZmlsbD0iI2Y1YTYyMyIvPjxyZWN0IHg9IjgzLjMyMDMyMDEyOTM5NDUzIiB3aWR0aD0iOTguNDIxODgyNjI5Mzk0NTMiIGhlaWdodD0iMzUiIGZpbGw9IiM0YTRhNGEiLz48dGV4dCB4PSI0MS42NjAxNjAwNjQ2OTcyNjYiIHk9IjIxLjUiIGZvbnQtc2l6ZT0iMTIiIGZvbnQtZmFtaWx5PSInUm9ib3RvJywgc2Fucy1zZXJpZiIgZmlsbD0iI0ZGRkZGRiIgdGV4dC1hbmNob3I9Im1pZGRsZSIgbGV0dGVyLXNwYWNpbmc9IjIiPkFNQVpPTjwvdGV4dD48dGV4dCB4PSIxMzIuNTMxMjYxNDQ0MDkxOCIgeT0iMjEuNSIgZm9udC1zaXplPSIxMiIgZm9udC1mYW1pbHk9IidNb250c2VycmF0Jywgc2Fucy1zZXJpZiIgZmlsbD0iI2ZmZmZmZiIgdGV4dC1hbmNob3I9Im1pZGRsZSIgZm9udC13ZWlnaHQ9IjkwMCIgbGV0dGVyLXNwYWNpbmc9IjIiPkFGRklMSUFURTwvdGV4dD48L3N2Zz4=)