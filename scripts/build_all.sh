#!/bin/bash
set -e

echo "🔨 BUILDING RCC SYSTEM"
echo "----------------------"

# 1. Targets
rustup target add wasm32-wasi

# 2. Compile WASM
echo "📦 Building WASM Game Module..."
cargo build -p rcc-wasm-game --release --target wasm32-wasi
mkdir -p games
cp target/wasm32-wasi/release/rcc_wasm_game.wasm games/game.wasm

# 3. Docker Build
echo "🐳 Building Docker Images..."
docker-compose build

# 4. Start Services
echo "🚀 Launching Coordinator & Executor..."
docker-compose up -d

# 5. Launch Client
echo "🎮 Starting Bevy Client..."
sleep 2 # Wait for socket to bind
cargo run -p rcc-client --release
