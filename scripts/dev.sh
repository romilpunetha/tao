#!/bin/bash

# Development script for TAO Database
set -e

echo "🚀 Starting TAO Database Development Environment"

# Function to cleanup background processes
cleanup() {
    echo "🧹 Cleaning up..."
    if [ ! -z "$RUST_PID" ]; then
        kill $RUST_PID 2>/dev/null || true
    fi
    if [ ! -z "$REACT_PID" ]; then
        kill $REACT_PID 2>/dev/null || true
    fi
    exit 0
}

# Trap cleanup on script exit
trap cleanup EXIT INT TERM

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Please run this script from the project root directory"
    exit 1
fi

# Install frontend dependencies if needed
if [ ! -d "frontend/node_modules" ]; then
    echo "📦 Installing frontend dependencies..."
    cd frontend
    npm install
    cd ..
fi

# Build the Rust project
echo "🔨 Building Rust backend..."
cargo build --bin tao_database_server

# Start the Rust server in the background
echo "🚀 Starting Rust API server on port 3000..."
cargo run --bin tao_database_server &
RUST_PID=$!

# Give the server time to start
sleep 3

# Start the React development server
echo "🎨 Starting React development server on port 3001..."
cd frontend
BROWSER=none PORT=3001 npm start &
REACT_PID=$!
cd ..

# Wait a bit more for React to start
sleep 5

echo ""
echo "✅ Development environment is ready!"
echo ""
echo "🌐 Frontend: http://localhost:3001"
echo "🔧 API:      http://localhost:3000/api"
echo "💾 Health:   http://localhost:3000/api/health"
echo ""
echo "Press Ctrl+C to stop all servers"

# Wait for user interrupt
wait