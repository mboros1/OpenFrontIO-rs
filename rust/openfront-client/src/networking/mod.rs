use bevy::prelude::*;

pub mod transport;
pub mod schemas;

use schemas::ServerMessage;

pub struct NetworkingPlugin;

impl Plugin for NetworkingPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<transport::NetworkEvent>()
            .insert_resource(transport::WebSocketConfig::default())
            .add_systems(Startup, transport::setup_websocket)
            .add_systems(Update, (
                transport::handle_messages,
                handle_network_events,
            ).chain());
    }
}

fn handle_network_events(
    mut network_events: EventReader<transport::NetworkEvent>,
    mut menu_state: ResMut<crate::rendering::ui::main_menu::MainMenuState>,
) {
    let event_count = network_events.len();
    if event_count > 0 {
        info!("📬 Processing {} network event(s)", event_count);
    }
    
    for event in network_events.read() {
        info!("🎯 Processing network event: {:?}", event.message);
        
        match &event.message {
            ServerMessage::Prestart(msg) => {
                info!("🎮 PRESTART received!");
                info!("  📍 Map: {}", msg.game_map);
                info!("  🎯 Game Type: {:?}", msg.game_type);
                info!("  🚪 Hiding menu and preparing game...");
                // Hide menu when game is starting
                menu_state.show_menu = false;
            }
            ServerMessage::Start(msg) => {
                info!("🚀 GAME START received!");
                info!("  👥 Players: {}", msg.game_start_info.players.len());
                info!("  ⚙️ Config: {:?}", msg.game_start_info.config);
                for (i, player) in msg.game_start_info.players.iter().enumerate() {
                    info!("    Player #{}: {:?}", i, player);
                }
                info!("  🚪 Hiding menu, game is starting!");
                // Hide menu when game starts
                menu_state.show_menu = false;
            }
            ServerMessage::Update(msg) => {
                if msg.tick % 10 == 0 {  // Log every 10th tick to reduce spam
                    info!("🔄 Game update - tick: {} with {} updates", msg.tick, msg.updates.len());
                } else {
                    debug!("Game update - tick: {} with {} updates", msg.tick, msg.updates.len());
                }
            }
            ServerMessage::Error(msg) => {
                error!("❌ SERVER ERROR: {}", msg.error);
                warn!("  🔧 You may need to rejoin or check your connection");
            }
            ServerMessage::Pong(msg) => {
                debug!("🏓 Pong received for tick {}", msg.tick);
            }
        }
    }
}