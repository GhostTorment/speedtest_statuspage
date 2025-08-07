# -------- Stage 1: Build the binary --------
FROM rust:latest AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev

# Set the working directory
WORKDIR /app

# Copy all files to the container
COPY . .

# Build in release mode
RUN cargo build --release

# -------- Stage 2: Create minimal runtime image --------
FROM alpine:latest

# Install runtime dependencies including speedtest-cli from community repo
RUN apk add --no-cache \
    curl \
    ca-certificates \
    speedtest-cli \
    bash

# Add non-root user
RUN adduser -D -s /bin/bash speedtest

# Set working directory
WORKDIR /app

# Copy built binary from builder stage
COPY --from=builder /app/target/release/speedtest_statuspage /usr/local/bin/speedtest-statuspage

# Copy optional .env and config
COPY .env /etc/speedtest-statuspage/.env

# Expose the port used by the app (default 8080)
EXPOSE 8080

# Switch to non-root user
USER speedtest

# Run the application
CMD ["/usr/local/bin/speedtest-statuspage"]
