use bevy::prelude::*;
use bevy::input::mouse::{MouseWheel, MouseButton};
use bevy::input::touch::TouchPhase;
use bevy::window::PrimaryWindow;

use crate::rendering::ui::UIState;
use crate::game::state::GameState;
use crate::networking::transport::{send_intent, send_join, WebSocketConnection};
use crate::networking::schemas::{Intent, TileRef};

#[derive(Event)]
pub enum GameInputEvent {
    // Map interactions
    TileClicked(i32, i32),
    TileRightClicked(i32, i32),
    MapPan(Vec2),
    MapZoom(f32),
    
    // Unit commands
    SpawnUnit(i32, i32),
    AttackCommand { target: Option<u32>, troops: u32 },
    BuildStructure { tile: (i32, i32), unit_type: String },
    
    // UI toggles
    ToggleLeaderboard,
    ToggleChat,
    ToggleBuildMenu,
    ToggleSettings,
    
    // Game actions
    SendEmoji(u32),
    QuickChat(String),
}

pub fn handle_keyboard_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut ui_state: ResMut<UIState>,
    mut input_events: EventWriter<GameInputEvent>,
    connection: Option<Res<WebSocketConnection>>,
) {
    // Join game on J key
    if keyboard.just_pressed(KeyCode::KeyJ) {
        if let Some(connection) = connection {
            info!("Joining game PM7jP0nv...");
            // Generate proper 8-char client ID
            use rand::Rng;
            let mut rng = rand::thread_rng();
            let client_id: String = (0..8)
                .map(|_| {
                    let idx = rng.gen_range(0..36);
                    if idx < 10 {
                        (b'0' + idx) as char
                    } else {
                        (b'A' + idx - 10) as char
                    }
                })
                .collect();
            
            // Generate UUID token
            let token = uuid::Uuid::new_v4().to_string();
            
            send_join(
                &connection,
                "PM7jP0nv".to_string(),
                client_id,
                "RustPlayer".to_string(),
                token, // Use proper UUID token
                "us".to_string(), // Must be lowercase - see countries.json
            );
        }
    }
    
    // UI toggles
    if keyboard.just_pressed(KeyCode::Tab) {
        ui_state.show_leaderboard = !ui_state.show_leaderboard;
        input_events.send(GameInputEvent::ToggleLeaderboard);
    }
    
    if keyboard.just_pressed(KeyCode::Enter) {
        ui_state.show_chat = !ui_state.show_chat;
        input_events.send(GameInputEvent::ToggleChat);
    }
    
    if keyboard.just_pressed(KeyCode::Escape) {
        ui_state.show_settings = !ui_state.show_settings;
        input_events.send(GameInputEvent::ToggleSettings);
    }
    
    if keyboard.just_pressed(KeyCode::KeyB) {
        ui_state.show_build_menu = !ui_state.show_build_menu;
        input_events.send(GameInputEvent::ToggleBuildMenu);
    }
    
    // Camera controls
    let mut pan_delta = Vec2::ZERO;
    let pan_speed = 10.0;
    
    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        pan_delta.y += pan_speed;
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        pan_delta.y -= pan_speed;
    }
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        pan_delta.x -= pan_speed;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        pan_delta.x += pan_speed;
    }
    
    if pan_delta != Vec2::ZERO {
        input_events.send(GameInputEvent::MapPan(pan_delta));
    }
    
    // Quick actions
    if keyboard.just_pressed(KeyCode::Space) {
        // Quick spawn at selected tile
        if let Some((q, r)) = ui_state.selected_tile {
            input_events.send(GameInputEvent::SpawnUnit(q, r));
        }
    }
}

pub fn handle_mouse_input(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut mouse_wheel: EventReader<MouseWheel>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    mut input_events: EventWriter<GameInputEvent>,
    mut ui_state: ResMut<UIState>,
) {
    let window = windows.single();
    let (camera, camera_transform) = cameras.single();
    
    // Handle mouse clicks
    if mouse_button.just_pressed(MouseButton::Left) {
        if let Some(cursor_position) = window.cursor_position() {
            // Convert screen position to world position
            if let Some(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_position) {
                // Convert world position to hex coordinates
                let (q, r) = world_to_hex(world_pos);
                ui_state.selected_tile = Some((q, r));
                input_events.send(GameInputEvent::TileClicked(q, r));
            }
        }
    }
    
    if mouse_button.just_pressed(MouseButton::Right) {
        if let Some(cursor_position) = window.cursor_position() {
            if let Some(world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_position) {
                let (q, r) = world_to_hex(world_pos);
                input_events.send(GameInputEvent::TileRightClicked(q, r));
            }
        }
    }
    
    // Handle mouse wheel zoom
    for event in mouse_wheel.read() {
        let zoom_delta = event.y * 0.1;
        input_events.send(GameInputEvent::MapZoom(zoom_delta));
    }
    
    // Handle middle mouse button pan
    if mouse_button.pressed(MouseButton::Middle) {
        // TODO: Implement mouse dragging for map pan
    }
}

pub fn handle_touch_input(
    mut touch_events: EventReader<TouchInput>,
    mut input_events: EventWriter<GameInputEvent>,
) {
    for touch in touch_events.read() {
        match touch.phase {
            TouchPhase::Started => {
                // Handle touch start (similar to mouse click)
                let world_pos = Vec2::new(touch.position.x, touch.position.y);
                let (q, r) = world_to_hex(world_pos);
                input_events.send(GameInputEvent::TileClicked(q, r));
            }
            TouchPhase::Moved => {
                // Handle touch drag for panning
                // TODO: Implement touch pan with velocity
            }
            TouchPhase::Ended | TouchPhase::Canceled => {
                // Handle touch end
            }
        }
    }
}

pub fn process_input_events(
    mut input_events: EventReader<GameInputEvent>,
    mut cameras: Query<&mut Transform, With<Camera>>,
    game_state: Res<GameState>,
    connection: Option<Res<WebSocketConnection>>,
) {
    for event in input_events.read() {
        match event {
            GameInputEvent::TileClicked(q, r) => {
                info!("Tile clicked: ({}, {})", q, r);
                // Handle tile selection logic
            }
            
            GameInputEvent::TileRightClicked(q, r) => {
                info!("Tile right-clicked: ({}, {})", q, r);
                // Handle context menu or quick action
            }
            
            GameInputEvent::MapPan(delta) => {
                for mut transform in cameras.iter_mut() {
                    transform.translation.x += delta.x;
                    transform.translation.y += delta.y;
                }
            }
            
            GameInputEvent::MapZoom(delta) => {
                for mut transform in cameras.iter_mut() {
                    let scale = transform.scale.x;
                    let new_scale = (scale + delta).clamp(0.5, 3.0);
                    transform.scale = Vec3::splat(new_scale);
                }
            }
            
            GameInputEvent::SpawnUnit(q, r) => {
                if let Some(connection) = &connection {
                    send_intent(
                        connection,
                        game_state.tick,
                        Intent::Spawn {
                            tile: TileRef { q: *q, r: *r },
                        },
                    );
                }
            }
            
            GameInputEvent::AttackCommand { target, troops } => {
                if let Some(connection) = &connection {
                    send_intent(
                        connection,
                        game_state.tick,
                        Intent::Attack {
                            target_id: *target,
                            troops: *troops,
                        },
                    );
                }
            }
            
            _ => {
                // Handle other events
            }
        }
    }
}

// Convert world position to hex coordinates
fn world_to_hex(world_pos: Vec2) -> (i32, i32) {
    let radius = 30.0;
    let x = world_pos.x;
    let y = world_pos.y;
    
    // Axial coordinates conversion
    let q = (x * 3.0_f32.sqrt() / 3.0 - y / 3.0) / radius;
    let r = y * 2.0 / 3.0 / radius;
    
    // Round to nearest hex
    let q_round = q.round() as i32;
    let r_round = r.round() as i32;
    let s_round = (-q - r).round() as i32;
    
    let q_diff = (q_round as f32 - q).abs();
    let r_diff = (r_round as f32 - r).abs();
    let s_diff = (s_round as f32 - (-q - r)).abs();
    
    if q_diff > r_diff && q_diff > s_diff {
        (-r_round - s_round, r_round)
    } else if r_diff > s_diff {
        (q_round, -q_round - s_round)
    } else {
        (q_round, r_round)
    }
}