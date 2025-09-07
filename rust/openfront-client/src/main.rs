use bevy::prelude::*;
use bevy_egui::EguiPlugin;

mod networking;
mod rendering;
mod game;
mod input;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "OpenFront.io".to_string(),
                resolution: (1280.0, 720.0).into(),
                canvas: Some("#game-canvas".to_string()), // For WASM
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin)
        // Game systems
        .add_plugins(networking::NetworkingPlugin)
        .add_plugins(rendering::RenderingPlugin)
        .add_plugins(game::GamePlugin)
        .add_plugins(input::InputPlugin)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    // Spawn 2D camera
    commands.spawn(Camera2dBundle::default());
    
    info!("OpenFront Rust client initialized");
    info!("Connecting to OpenFront.io...");
    info!("To join game PM7jP0nv, press J");
}