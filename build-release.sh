#!/bin/bash
set -e

# Get the directory this script is in
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(realpath "$SCRIPT_DIR/..")"

# Build the project
cd "$PROJECT_ROOT"
cargo clean
cargo build --release

# Copy files using absolute paths based on the project root
cp "$PROJECT_ROOT/.env" "$PROJECT_ROOT/package/etc/speedtest-statuspage/.env"
cp "$PROJECT_ROOT/target/release/speedtest-statuspage" "$PROJECT_ROOT/package/usr/local/bin/speedtest-statuspage"