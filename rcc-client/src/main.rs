use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crossbeam_channel::{unbounded, Receiver, Sender};
use tokio_tungstenite::connect_async;
use futures_util::{StreamExt, SinkExt};
use url::Url;

// --- Components & Events ---
#[derive(Component)]
struct LocalPlayer;

#[derive(Component)]
struct RemotePlayer { id: String }

#[derive(Resource)]
struct NetworkChannels {
    tx: Sender<String>,
    rx: Receiver<String>,
}

#[derive(Resource)]
struct PlayerId(String);

#[derive(Deserialize, Debug)]
struct ServerState {
    players: HashMap<String, PlayerData>,
    chat: Vec<String>,
}

#[derive(Deserialize, Debug, Clone)]
struct PlayerData {
    x: f32, y: f32, z: f32,
}

#[derive(Serialize)]
struct PlayerInput {
    id: String,
    dx: f32,
    dz: f32,
}

fn main() {
    let id = format!("User{:03}", rand::random::<u8>());
    println!("🎮 Starting Client as {}", id);

    let (net_tx, thread_rx) = unbounded();
    let (thread_tx, net_rx) = unbounded();

    // Spawn Network Thread
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async move {
            let url = Url::parse("ws://localhost:8080").unwrap();
            let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
            let (mut write, mut read) = ws_stream.split();

            loop {
                tokio::select! {
                    msg = thread_rx.recv() => {
                        if let Ok(txt) = msg {
                            write.send(tokio_tungstenite::tungstenite::Message::Text(txt)).await.unwrap();
                        }
                    }
                    msg = read.next() => {
                        if let Some(Ok(tokio_tungstenite::tungstenite::Message::Text(txt))) = msg {
                            thread_tx.send(txt).unwrap();
                        }
                    }
                }
            }
        });
    });

    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(NetworkChannels { tx: net_tx, rx: net_rx })
        .insert_resource(PlayerId(id))
        .add_systems(Startup, setup)
        .add_systems(Update, (input_system, network_sync_system))
        .run();
}

fn setup(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    // Ground
    commands.spawn(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane::from_size(50.0))),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    });

    // Light
    commands.spawn(PointLightBundle {
        point_light: PointLight { intensity: 1500.0, shadows_enabled: true, ..default() },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    // Camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0.0, 10.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

fn input_system(
    input: Res<Input<KeyCode>>,
    net: Res<NetworkChannels>,
    pid: Res<PlayerId>,
) {
    let mut dx = 0.0;
    let mut dz = 0.0;

    if input.pressed(KeyCode::W) { dz -= 1.0; }
    if input.pressed(KeyCode::S) { dz += 1.0; }
    if input.pressed(KeyCode::A) { dx -= 1.0; }
    if input.pressed(KeyCode::D) { dx += 1.0; }

    if dx != 0.0 || dz != 0.0 {
        let payload = PlayerInput { id: pid.0.clone(), dx, dz };
        net.tx.send(serde_json::to_string(&payload).unwrap()).unwrap();
    }
}

fn network_sync_system(
    mut commands: Commands,
    net: Res<NetworkChannels>,
    mut meshes: ResMut<Assets<Mesh>>, 
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<(Entity, &mut Transform, &RemotePlayer)>,
    pid: Res<PlayerId>,
) {
    while let Ok(msg) = net.rx.try_recv() {
        if let Ok(state) = serde_json::from_str::<ServerState>(&msg) {
            
            // Collect existing entities
            let mut existing = HashMap::new();
            for (e, t, r) in query.iter_mut() {
                existing.insert(r.id.clone(), (e, t));
            }

            for (id, data) in state.players {
                if let Some((_, transform)) = existing.get_mut(&id) {
                    // Update Position (Simple Snap)
                    transform.translation = Vec3::new(data.x, data.y, data.z);
                } else {
                    // Spawn New
                    let color = if id == pid.0 { Color::BLUE } else { Color::RED };
                    commands.spawn((
                        PbrBundle {
                            mesh: meshes.add(Mesh::from(shape::Capsule::default())),
                            material: materials.add(color.into()),
                            transform: Transform::from_xyz(data.x, data.y, data.z),
                            ..default()
                        },
                        RemotePlayer { id },
                    ));
                }
            }
        }
    }
}
