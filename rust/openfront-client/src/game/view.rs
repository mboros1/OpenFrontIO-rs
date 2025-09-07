use bevy::prelude::*;
use super::state::GameState;
use crate::rendering::map::HexTile;
use crate::rendering::units::Unit as RenderUnit;

// Syncs the game state with the visual representation
pub fn sync_view_with_state(
    game_state: Res<GameState>,
    mut hex_tiles: Query<&mut HexTile>,
    render_units: Query<&mut RenderUnit>,
    commands: Commands,
) {
    if !game_state.is_changed() {
        return;
    }
    
    // Update territory ownership
    for mut tile in hex_tiles.iter_mut() {
        let position = (tile.q, tile.r);
        if let Some(territory) = game_state.territories.get(&position) {
            tile.owner = territory.owner;
        }
    }
    
    // Update units
    // TODO: Implement unit spawning and despawning based on game state
    // This would involve:
    // 1. Checking which units exist in game state but not in render
    // 2. Spawning new unit entities
    // 3. Removing units that no longer exist
    // 4. Updating positions and stats of existing units
}

// Convert game coordinates to world position
pub fn game_to_world_pos(q: i32, r: i32) -> Vec3 {
    crate::rendering::map::hex_to_world(q, r, 30.0)
}

// Camera controller for following action
#[derive(Component)]
pub struct GameCamera {
    pub target: Option<Vec3>,
    pub zoom: f32,
    pub min_zoom: f32,
    pub max_zoom: f32,
}

impl Default for GameCamera {
    fn default() -> Self {
        Self {
            target: None,
            zoom: 1.0,
            min_zoom: 0.5,
            max_zoom: 3.0,
        }
    }
}

pub fn update_camera(
    mut cameras: Query<(&mut Transform, &mut OrthographicProjection, &GameCamera)>,
    time: Res<Time>,
) {
    for (mut transform, mut projection, camera) in cameras.iter_mut() {
        // Smooth camera movement to target
        if let Some(target) = camera.target {
            let current = transform.translation;
            let new_pos = current.lerp(target, 5.0 * time.delta_seconds());
            transform.translation = new_pos;
        }
        
        // Apply zoom
        projection.scale = camera.zoom;
    }
}

// Helper to focus camera on a position
pub fn focus_camera_on(
    position: Vec3,
    mut cameras: Query<&mut GameCamera>,
) {
    for mut camera in cameras.iter_mut() {
        camera.target = Some(position);
    }
}