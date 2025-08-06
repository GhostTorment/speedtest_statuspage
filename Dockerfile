# -------- Stage 1: Build the binary --------
FROM rust:1.77 as builder

# Install build dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev

# Set the working directory
WORKDIR /app

# Copy all files to the container
COPY . .

# Build in release mode
RUN cargo build --release

# -------- Stage 2: Create minimal runtime image --------
FROM debian:bookworm-slim

# Install runtime dependencies: Python3 and speedtest-cli
RUN apt-get update && \
    apt-get install -y python3 python3-pip ca-certificates curl && \
    pip install speedtest-cli && \
    apt-get clean && rm -rf /var/lib/apt/lists/*

# Add non-root user
RUN useradd -m -s /bin/bash speedtest

# Set working directory
WORKDIR /app

# Copy built binary from builder stage
COPY --from=builder /app/target/release/speedtest-statuspage /usr/local/bin/speedtest-statuspage

# Copy optional .env and config
COPY .env /etc/speedtest-statuspage/.env

# Expose the port used by the app (default 8080)
EXPOSE 8080

# Switch to non-root user
USER speedtest

# Run the application
CMD ["/usr/local/bin/speedtest-statuspage"]
