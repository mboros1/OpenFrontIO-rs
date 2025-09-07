use bevy::prelude::*;

pub mod state;
pub mod view;
pub mod map;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(state::GameState::default())
            .add_systems(Startup, setup_test_map)
            .add_systems(Update, (
                state::update_game_state,
                view::sync_view_with_state,
            ));
    }
}

fn setup_test_map(mut commands: Commands) {
    // Create a test map with some territories
    let width = 100;
    let height = 100;
    
    // Generate simple terrain (land with some water)
    let mut terrain_data = Vec::with_capacity(width * height);
    for y in 0..height {
        for x in 0..width {
            // Create some islands
            let is_land = 
                (x > 10 && x < 30 && y > 10 && y < 30) ||
                (x > 40 && x < 70 && y > 20 && y < 60) ||
                (x > 60 && x < 90 && y > 60 && y < 90);
            
            let mut byte = 0u8;
            if is_land {
                byte |= 1 << 7; // IS_LAND_BIT
            }
            terrain_data.push(byte);
        }
    }
    
    let mut game_map = map::GameMap::new(width, height, terrain_data);
    
    // Add some test territories
    for y in 15..25 {
        for x in 15..25 {
            game_map.set_owner_id(game_map.ref_from_xy(x, y), 1);
        }
    }
    
    for y in 30..50 {
        for x in 45..65 {
            game_map.set_owner_id(game_map.ref_from_xy(x, y), 2);
        }
    }
    
    commands.insert_resource(game_map);
    info!("Test map created: {}x{}", width, height);
}