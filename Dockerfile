# Stage 1: Dependencies (Caching layer for Cargo dependencies)
FROM rust:1-slim AS deps

WORKDIR /usr/src/affilify-bot

RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Copy only Cargo files to cache dependencies
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Stage 2: Build application (Uses cached dependencies)
FROM rust:1-slim AS builder

WORKDIR /usr/src/affilify-bot

RUN apt-get update && \
    apt-get install -y pkg-config libssl-dev && \
    rm -rf /var/lib/apt/lists/*

# Copy cached dependencies from deps stage
COPY --from=deps /usr/src/affilify-bot/target target

# Copy source code and build
COPY . .
RUN touch src/main.rs  # Force rebuild of main application
RUN cargo build --release

# Stage 3: Runtime (Bookworm + OpenSSL 3)
FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && \
    apt-get install -y libssl3 ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/affilify-bot/target/release/affilify-bot /usr/local/bin/affilify-bot
COPY .env.example ./.env

VOLUME [ "/app/data" ]
ENTRYPOINT ["affilify-bot"]