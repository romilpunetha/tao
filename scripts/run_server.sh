#!/bin/bash

# Meta TAO Database Server Runner

echo "🚀 Starting Meta TAO Database Server..."

# Create data directory if it doesn't exist
mkdir -p data

# Set environment variables with defaults
export DATABASE_URL="${DATABASE_URL:-sqlite:data/tao_database.db}"
export SERVER_HOST="${SERVER_HOST:-127.0.0.1}"
export SERVER_PORT="${SERVER_PORT:-3000}"
export CACHE_CAPACITY="${CACHE_CAPACITY:-1000}"

echo "📊 Configuration:"
echo "  Database: $DATABASE_URL"
echo "  Server: $SERVER_HOST:$SERVER_PORT"
echo "  Cache: $CACHE_CAPACITY items"

# Build and run
cargo build --release
cargo run --bin tao_server