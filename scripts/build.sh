#!/bin/bash

# Production build script for TAO Database
set -e

echo "🏗️  Building TAO Database for Production"

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Please run this script from the project root directory"
    exit 1
fi

# Clean previous builds
echo "🧹 Cleaning previous builds..."
cargo clean
rm -rf frontend/build

# Install frontend dependencies
echo "📦 Installing frontend dependencies..."
cd frontend
npm ci --only=production
cd ..

# Build the React frontend
echo "🎨 Building React frontend..."
cd frontend
npm run build
cd ..

# Copy built frontend to a location the Rust server can serve
echo "📁 Preparing static files..."
rm -rf frontend_dist
cp -r frontend/build frontend_dist

# Build the Rust server in release mode
echo "🔨 Building Rust backend (release mode)..."
cargo build --release --bin tao_database_server

echo ""
echo "✅ Production build complete!"
echo ""
echo "📦 Rust binary: target/release/tao_database_server"
echo "🎨 Frontend:    frontend_dist/"
echo ""
echo "To run in production:"
echo "  ./target/release/tao_database_server"
echo ""