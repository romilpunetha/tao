#!/bin/bash

# TAO Database Demo Server and Frontend Runner

echo "🚀 Starting TAO Database Demo..."

# Set environment variables
export DATABASE_URL="${DATABASE_URL:-sqlite:data/tao_database.db}"
export SERVER_HOST="${SERVER_HOST:-127.0.0.1}"
export SERVER_PORT="${SERVER_PORT:-8000}"

echo "📊 Configuration:"
echo "  Server: $SERVER_HOST:$SERVER_PORT"
echo "  Frontend will be served at: http://$SERVER_HOST:$SERVER_PORT"
echo "  API available at: http://$SERVER_HOST:$SERVER_PORT/api"

# Build frontend if it doesn't exist
if [ ! -d "frontend/build" ]; then
    echo "🔨 Building frontend..."
    cd frontend
    npm install
    npm run build
    cd ..
else
    echo "✅ Frontend build exists"
fi

# Build and run the demo server
echo "🔧 Building demo server..."
cargo build --bin tao_server_simple

echo "🌐 Starting demo server..."
echo "👀 Visit http://127.0.0.1:8000 to see the social graph visualization!"
cargo run --bin tao_server_simple