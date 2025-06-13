# Stage 1: Build (default, Bookworm under the hood)
FROM rust:1-slim AS builder

WORKDIR /usr/src/discord-bot-affilify

RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

COPY . .
RUN cargo build --release

# Stage 2: Runtime (Bookworm + OpenSSL 3)
FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && \
    apt-get install -y libssl3 ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/discord-bot-affilify/target/release/discord-bot-affilify /usr/local/bin/discord-bot-affilify
COPY .env.example ./.env

VOLUME [ "/app/bot.db" ]
ENTRYPOINT ["discord-bot-affilify"]