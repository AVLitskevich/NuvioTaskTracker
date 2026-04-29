# Build stage
FROM rust:1.80-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .
RUN cargo build -p nuvio-bot --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/nuvio-bot /app/nuvio-bot

# Copy migrations if needed (assuming they are in backend/migrations)
COPY migrations /app/migrations

ENV APP_ENV=production
ENTRYPOINT ["./nuvio-bot"]
