use serde::{Deserialize, Serialize};
use std::collections::HashMap;

static mut STATE: Option<GameState> = None;
static mut OUTPUT_BUFFER: Vec<u8> = Vec::new();

#[derive(Serialize, Deserialize)]
struct GameState {
    players: HashMap<String, Player>,
    chat: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
struct Player {
    id: String,
    x: f32, y: f32, z: f32,
    score: i32,
}

#[no_mangle]
pub extern "C" fn init(_ptr: i32, _len: i32) -> i32 {
    unsafe {
        STATE = Some(GameState {
            players: HashMap::new(),
            chat: vec!["Server: System Initialized".to_string()],
        });
    }
    0 // Success
}

#[no_mangle]
pub extern "C" fn tick(dt_ms: f32) {
    // Simple logic: Gravity or world-wide updates
}

#[no_mangle]
pub extern "C" fn on_input(p_ptr: i32, p_len: i32, i_ptr: i32, i_len: i32) {
    unsafe {
        let state = STATE.as_mut().unwrap();
        let p_id = String::from_utf8(std::slice::from_raw_parts(p_ptr as *const u8, p_len as usize).to_vec()).unwrap();
        let input_json = std::slice::from_raw_parts(i_ptr as *const u8, i_len as usize);
        
        // Update player position based on input
        let player = state.players.entry(p_id.clone()).or_insert(Player { id: p_id, x: 0.0, y: 1.0, z: 0.0, score: 0 });
        // Simplified input parsing
        if input_json.len() > 0 { player.x += 0.1; } 
    }
}

#[no_mangle]
pub extern "C" fn serialize_state() -> *const u8 {
    unsafe {
        let state = STATE.as_ref().unwrap();
        OUTPUT_BUFFER = serde_json::to_vec(state).unwrap();
        OUTPUT_BUFFER.as_ptr()
    }
}

#[no_mangle]
pub extern "C" fn get_state_size() -> i32 {
    unsafe { OUTPUT_BUFFER.len() as i32 }
}
