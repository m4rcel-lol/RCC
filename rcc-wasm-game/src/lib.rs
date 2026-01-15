use std::collections::HashMap;
use std::sync::Mutex;
use serde::{Deserialize, Serialize};

#[macro_use]
extern crate lazy_static;

#[derive(Serialize, Deserialize, Clone)]
struct Player {
    id: String,
    x: f32, 
    y: f32, 
    z: f32,
    msg: Option<String>
}

#[derive(Serialize, Deserialize)]
struct State {
    players: HashMap<String, Player>,
    chat: Vec<String>,
}

#[derive(Deserialize)]
struct Input {
    id: String,
    dx: f32,
    dz: f32,
    msg: Option<String>
}

// Basic static storage for the game state (single threaded WASM)
static mut STATE: Option<State> = None;
static mut BUFFER: Vec<u8> = Vec::new();

#[no_mangle]
pub extern "C" fn tick(ptr: i32, len: i32, _dt: f32) -> i32 {
    unsafe {
        if STATE.is_none() {
            STATE = Some(State { players: HashMap::new(), chat: Vec::new() });
        }
        let state = STATE.as_mut().unwrap();

        // 1. Read Input JSON
        let slice = std::slice::from_raw_parts(ptr as *const u8, len as usize);
        let input_str = std::str::from_utf8(slice).unwrap_or("[]");
        let inputs: Vec<Input> = serde_json::from_str(input_str).unwrap_or_default();

        // 2. Apply Logic
        for input in inputs {
            let player = state.players.entry(input.id.clone()).or_insert(Player {
                id: input.id, x: 0.0, y: 1.0, z: 0.0, msg: None
            });
            
            player.x += input.dx * 0.5; // Speed
            player.z += input.dz * 0.5;
            
            // Simple bound checking
            if player.x > 50.0 { player.x = 50.0; }
            if player.x < -50.0 { player.x = -50.0; }
            
            if let Some(msg) = input.msg {
                state.chat.push(format!("{}: {}", player.id, msg));
                if state.chat.len() > 10 { state.chat.remove(0); }
            }
        }

        // 3. Serialize State
        let json = serde_json::to_string(&state).unwrap();
        let bytes = json.as_bytes();
        
        // Write to buffer (Length header + Payload)
        BUFFER.clear();
        BUFFER.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
        BUFFER.extend_from_slice(bytes);
        
        BUFFER.as_ptr() as i32
    }
}
