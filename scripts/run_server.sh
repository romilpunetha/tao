#!/bin/bash

# Meta TAO Database Server Runner

echo "🚀 Starting Meta TAO Database Server..."

# Create data directory if it doesn't exist
mkdir -p data

# Build and run
cargo build --release
cargo run --bin tao_web_server