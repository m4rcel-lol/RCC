use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use futures_util::{StreamExt, SinkExt};
use wasmtime::*;
use wasmtime_wasi::sync::WasiCtxBuilder;

// --- Shared State ---
#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
struct GameState {
    players: std::collections::HashMap<String, Player>,
    chat: Vec<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
struct Player {
    id: String,
    x: f32,
    y: f32,
    z: f32,
    msg: Option<String>,
}

// --- WASM Host Setup ---
struct GameEngine {
    store: Store<WasiCtx>,
    instance: Instance,
    memory: Memory,
}

impl GameEngine {
    fn new(wasm_path: &str) -> Self {
        let engine = Engine::default();
        let mut linker = Linker::new(&engine);
        wasmtime_wasi::add_to_linker(&mut linker, |s| s).unwrap();

        let wasi = WasiCtxBuilder::new().inherit_stdio().build();
        let mut store = Store::new(&engine, wasi);
        
        let module = Module::from_file(&engine, wasm_path).expect("Failed to load WASM");
        let instance = linker.instantiate(&mut store, &module).expect("Failed to instantiate");
        let memory = instance.get_memory(&mut store, "memory").expect("No memory exported");

        Self { store, instance, memory }
    }

    fn tick(&mut self, dt: f32, input_json: String) -> String {
        // 1. Write Input to WASM Memory
        let input_bytes = input_json.as_bytes();
        let input_ptr = 0; // Simplified allocator for MVP (reuse start of memory)
        self.memory.write(&mut self.store, input_ptr, input_bytes).unwrap();

        // 2. Call Tick
        let tick_fn = self.instance.get_typed_func::<(i32, i32, f32), i32>(&mut self.store, "tick").unwrap();
        let output_ptr = tick_fn.call(&mut self.store, (input_ptr as i32, input_bytes.len() as i32, dt)).unwrap();

        // 3. Read Result (first 4 bytes is len, then data)
        let mut len_buf = [0u8; 4];
        self.memory.read(&mut self.store, output_ptr as usize, &mut len_buf).unwrap();
        let len = u32::from_le_bytes(len_buf) as usize;

        let mut data = vec![0u8; len];
        self.memory.read(&mut self.store, output_ptr as usize + 4, &mut data).unwrap();
        
        String::from_utf8(data).unwrap()
    }
}

#[tokio::main]
async fn main() {
    println!("🧼 RCCSoap Executor Starting...");
    
    // Simulate downloading WASM artifact
    let wasm_path = "games/game.wasm"; 
    
    // In a real app, we'd wait for the file to exist. 
    // For this build script, we assume it's mounted.
    
    // Start WebSocket Server
    let addr = "0.0.0.0:8080";
    let listener = TcpListener::bind(&addr).await.expect("Failed to bind");
    println!("🔌 WS Listening on {}", addr);

    let inputs = Arc::new(Mutex::new(Vec::new()));
    let inputs_clone = inputs.clone();
    
    // Broadcast channel for state updates
    let (tx, _rx) = tokio::sync::broadcast::channel(100);
    let tx_clone = tx.clone();

    // 🔄 Game Loop Thread
    std::thread::spawn(move || {
        // Wait for file to appear (build race condition safety)
        std::thread::sleep(std::time::Duration::from_secs(5)); 
        let mut engine = GameEngine::new(wasm_path);
        
        loop {
            let start = std::time::Instant::now();
            
            // Collect inputs
            let mut current_inputs = {
                let mut guard = inputs_clone.lock().unwrap();
                let data = guard.clone();
                guard.clear();
                data
            };
            
            let json_input = serde_json::to_string(&current_inputs).unwrap();
            let state_json = engine.tick(0.016, json_input);
            
            // Broadcast state
            let _ = tx_clone.send(state_json);
            
            // Cap at ~60 FPS
            let elapsed = start.elapsed();
            if elapsed < std::time::Duration::from_millis(16) {
                std::thread::sleep(std::time::Duration::from_millis(16) - elapsed);
            }
        }
    });

    // 🌐 Connection Handler
    while let Ok((stream, _)) = listener.accept().await {
        let inputs = inputs.clone();
        let mut rx = tx.subscribe();
        
        tokio::spawn(async move {
            let ws_stream = accept_async(stream).await.expect("Handshake failed");
            let (mut write, mut read) = ws_stream.split();

            loop {
                tokio::select! {
                    msg = read.next() => {
                        match msg {
                            Some(Ok(tokio_tungstenite::tungstenite::Message::Text(txt))) => {
                                // Parse basic input: {"id": "p1", "x": 1, "y": 0}
                                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&txt) {
                                    inputs.lock().unwrap().push(val);
                                }
                            }
                            _ => break,
                        }
                    }
                    Ok(state) = rx.recv() => {
                         write.send(tokio_tungstenite::tungstenite::Message::Text(state)).await.ok();
                    }
                }
            }
        });
    }
}
