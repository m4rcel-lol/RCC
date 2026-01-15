# 🔴 RCC: Rust Cloud Coordinator
RCC is a complete, clean-room reimplementation of a cloud-orchestrated gaming system. Built from the ground up in Rust, it prioritizes security through WebAssembly (WASM) sandboxing, scalability via gRPC, and a modern 3D experience using the Bevy Engine.
## 🏗️ System Architecture
The RCC ecosystem is composed of four primary pillars, designed to be modular and Linux-first:
| Component | Responsibility | Technology |
|---|---|---|
| RCCService | Cloud Coordinator: Manages instances, database state, and executor registry. | Rust, gRPC (Tonic), SQLx, PostgreSQL |
| RCCSoap | Secure Executor: Hosts the WASM runtime and manages real-time client traffic. | Rust, Wasmtime (WASI), WebSockets |
| WASM Module | Authoritative Logic: The game rules, physics, and state transitions. | Rust (wasm32-wasi) |
| Bevy Client | Visualizer: Renders the world state and captures user input. | Bevy Engine, WebGPU |
## 🚀 Quick Start (One-Command Build)
This project is designed for local development with zero manual configuration.
1. Prerequisites
 * Rust Toolchain: rustup target add wasm32-wasi
 * Docker & Docker Compose
 * Native Build Essentials (for Bevy dependencies)
2. Execution:
   
Run the automated build and launch script from the root directory:
```
chmod +x scripts/build_all.sh
./scripts/build_all.sh
```
### What this does:
 * Compiles the game logic into a .wasm artifact.
 * Boots the PostgreSQL database and Coordinator service.
 * Initializes the RCCSoap executor with the WASM module.
 * Launches the Bevy 3D client and connects to the runtime.
🔐 Security & Sandboxing
Our "Clean-Room" philosophy extends deeply into the runtime security model.
 * Instruction Isolation: Game logic has zero access to the host machine's syscalls. All I/O is piped through WASI with strictly defined pre-opened directories.
 * Memory Guarding: The WASM module is confined to a 64MB linear memory segment. Any attempt to access memory outside this range results in an immediate instance trap.
 * Resource Throttling: Using Wasmtime's "Fuel" mechanism, the executor terminates modules that exceed defined CPU cycles per tick, preventing infinite loop exploits.
 * Identity: All service-to-service communication is gated by JWT (JSON Web Tokens).
🛠️ WASM Authoring Guide
To build a game module for the RCC system, your Rust code must export the following interface:
Required Exports
```
// Called once when the instance is spawned
#[no_mangle] pub extern "C" fn init(ptr: i32, len: i32) -> i32;

// Called every simulation frame (16ms)
#[no_mangle] pub extern "C" fn tick(dt_ms: f32);

// Processes player input buffers
#[no_mangle] pub extern "C" fn on_input(p_ptr: i32, p_len: i32, i_ptr: i32, i_len: i32);

// Signals the host to read the serialized state
#[no_mangle] pub extern "C" fn serialize_state() -> *const u8;
```
## 📊 Observability & Metrics
RCC provides native hooks for monitoring system health in production:
 * Structured Logging: All services output JSON logs via tracing-subscriber, ready for ELK or Loki ingestion.
 * Prometheus Metrics: RCCSoap exposes a /metrics endpoint (Port 9090) tracking:
   * player_count: Current active WebSocket connections.
   * wasm_tick_duration_ms: Latency of the game logic execution.
   * instance_memory_usage: Real-time heap usage of the WASM guest.
## 📁 Project Layout
```
.
├── rcc-proto/          # Shared gRPC & Protobuf definitions
├── rcc-service/        # Coordinator logic & SQLx database migrations
├── rcc-soap/           # WASM runtime host & WebSocket relay
├── rcc-wasm-game/      # Authoritative game logic source
├── rcc-client/         # 3D Bevy game client
├── games/              # Compiled WASM artifacts
├── scripts/            # Build and deployment automation
└── docker-compose.yml  # Local infrastructure stack
```
# ⚠️ Disclaimer
- This is a clean-room reimplementation. No proprietary code, assets, or reverse-engineered logic from existing platforms were used. It is an independent architecture built on open-source standards.
