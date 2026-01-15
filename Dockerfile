# Build Stage
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
# Build WASM game
RUN rustup target add wasm32-wasi
RUN cd rcc-wasm-game && cargo build --release --target wasm32-wasi

# Build Binaries
RUN cargo build --release --bin rcc-service
RUN cargo build --release --bin rcc-soap

# Service Image
FROM debian:bookworm-slim as service
COPY --from=builder /app/target/release/rcc-service /usr/local/bin/
CMD ["rcc-service"]

# Soap Image
FROM debian:bookworm-slim as soap
WORKDIR /app
COPY --from=builder /app/target/release/rcc-soap /usr/local/bin/
# Copy the WASM game to a shared location
COPY --from=builder /app/target/wasm32-wasi/release/rcc_wasm_game.wasm /app/games/game.wasm
CMD ["rcc-soap"]
