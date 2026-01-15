#!/bin/bash
set -e

echo "🚧 Creating Game Directory..."
mkdir -p games

echo "🛠️  Checking Dependencies..."
if ! command -v cargo &> /dev/null; then
    echo "Rust is required."
    exit 1
fi

echo "📦 Adding WASM Target..."
rustup target add wasm32-wasi

echo "🏗️  Building WASM Game Module..."
cargo build -p rcc-wasm-game --release --target wasm32-wasi
cp target/wasm32-wasi/release/rcc_wasm_game.wasm games/game.wasm

echo "🏗️  Building Backend Containers..."
docker-compose build

echo "🚀 Starting Backend Services..."
docker-compose up -d

echo "✅ Backend Running!"
echo "   - Coordinator: localhost:50051"
echo "   - Executor: localhost:8080"
echo "   - Game WASM Loaded"

echo "🎮 Starting Game Client (Native)..."
echo "   (Press Ctrl+C to stop client, then 'docker-compose down' to stop server)"
cargo run -p rcc-client
