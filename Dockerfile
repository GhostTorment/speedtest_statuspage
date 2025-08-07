# -------- Stage 1: Build the binary with musl --------
FROM rust:latest AS builder

# Install musl target and build dependencies
RUN rustup target add x86_64-unknown-linux-musl && \
    apt-get update && apt-get install -y pkg-config libssl-dev musl-tools

WORKDIR /app

COPY . .

# Build with musl target for static linking
RUN cargo build --release --target x86_64-unknown-linux-musl

# -------- Stage 2: Create minimal runtime image --------
FROM alpine:latest

RUN apk add --no-cache \
    curl \
    ca-certificates \
    speedtest-cli \
    bash

RUN adduser -D -s /bin/bash speedtest

WORKDIR /app

# Copy the musl static binary
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/speedtest_statuspage /usr/local/bin/speedtest-statuspage

COPY .env /etc/speedtest-statuspage/.env

EXPOSE 8080

USER speedtest

CMD ["/usr/local/bin/speedtest-statuspage"]
