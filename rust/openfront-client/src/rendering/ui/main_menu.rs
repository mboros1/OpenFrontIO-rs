use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use std::fs;
use std::path::PathBuf;

use crate::networking::transport::{send_join, WebSocketConnection};

// Get or create a persistent client ID (8 alphanumeric characters)
fn get_or_create_client_id() -> String {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("openfront-rust");
    
    let id_file = config_dir.join("client_id.txt");
    
    // Try to read existing ID
    if let Ok(id) = fs::read_to_string(&id_file) {
        let trimmed = id.trim();
        if !trimmed.is_empty() && trimmed.len() <= 8 && trimmed.chars().all(|c| c.is_alphanumeric()) {
            return trimmed.to_string();
        }
    }
    
    // Generate new 8-character alphanumeric ID
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let new_id: String = (0..8)
        .map(|_| {
            let idx = rng.gen_range(0..36);
            if idx < 10 {
                (b'0' + idx) as char
            } else {
                (b'A' + idx - 10) as char
            }
        })
        .collect();
    
    // Try to save it for next time
    let _ = fs::create_dir_all(&config_dir);
    let _ = fs::write(&id_file, &new_id);
    
    new_id
}

// Get or create a persistent UUID token for authentication
fn get_or_create_token() -> String {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("openfront-rust");
    
    let token_file = config_dir.join("token.txt");
    
    // Try to read existing token
    if let Ok(token) = fs::read_to_string(&token_file) {
        let trimmed = token.trim();
        // Validate it's a UUID format
        if uuid::Uuid::parse_str(trimmed).is_ok() {
            return trimmed.to_string();
        }
    }
    
    // Generate new UUID token
    let new_token = uuid::Uuid::new_v4().to_string();
    
    // Try to save it for next time
    let _ = fs::create_dir_all(&config_dir);
    let _ = fs::write(&token_file, &new_token);
    
    new_token
}

pub struct MainMenuPlugin;

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<MainMenuState>()
            .init_resource::<LobbyList>()
            .add_systems(Startup, fetch_lobbies)
            .add_systems(Update, (
                render_main_menu,
                update_lobbies,
            ));
    }
}

#[derive(Resource)]
pub struct MainMenuState {
    pub show_menu: bool,
    pub username: String,
    pub selected_flag: String,
    pub selected_pattern: Option<String>,
    pub show_help: bool,
    pub show_settings: bool,
    pub show_create_lobby: bool,
    pub show_join_private: bool,
    pub private_lobby_code: String,
}

impl Default for MainMenuState {
    fn default() -> Self {
        Self {
            show_menu: true,  // Show menu by default
            username: "Player".to_string(),
            selected_flag: "us".to_string(),  // Must be lowercase - see countries.json
            selected_pattern: None,
            show_help: false,
            show_settings: false,
            show_create_lobby: false,
            show_join_private: false,
            private_lobby_code: String::new(),
        }
    }
}

#[derive(Resource)]
pub struct LobbyList {
    pub lobbies: Vec<LobbyInfo>,
    pub last_update: f64,
    pub is_fetching: bool,
    pub lobby_fetch_times: std::collections::HashMap<String, f64>, // Track when each lobby was fetched
}

impl Default for LobbyList {
    fn default() -> Self {
        Self {
            lobbies: Vec::new(),
            last_update: 0.0,
            is_fetching: false,
            lobby_fetch_times: std::collections::HashMap::new(),
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct LobbyInfo {
    #[serde(rename = "gameID")]
    pub game_id: String,
    #[serde(rename = "numClients")]
    pub num_clients: u32,
    #[serde(rename = "gameConfig")]
    pub game_config: GameConfig,
    #[serde(rename = "msUntilStart")]
    pub ms_until_start: u32,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GameConfig {
    #[serde(rename = "gameMap")]
    pub game_map: String,
    #[serde(rename = "gameType")]
    pub game_type: String,
    pub difficulty: String,
    #[serde(rename = "gameMode")]
    pub game_mode: String,
    #[serde(rename = "maxPlayers")]
    pub max_players: u32,
    pub bots: Option<u32>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ApiPublicLobbiesResponse {
    pub lobbies: Vec<LobbyInfo>,
}

fn fetch_lobbies(lobby_list: ResMut<LobbyList>) {
    // Start fetching lobbies from the API
    start_lobby_fetch(lobby_list);
}

fn start_lobby_fetch(mut lobby_list: ResMut<LobbyList>) {
    if lobby_list.is_fetching {
        return;
    }
    
    lobby_list.is_fetching = true;
    
    // Spawn async task to fetch lobbies
    std::thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            match fetch_lobbies_async().await {
                Ok(fetched_lobbies) => {
                    // Store in a temporary file so main thread can read it
                    if let Ok(json) = serde_json::to_string(&fetched_lobbies) {
                        let _ = std::fs::write("/tmp/openfront_lobbies.json", json);
                    }
                }
                Err(e) => {
                    error!("Failed to fetch lobbies: {}", e);
                }
            }
        });
    });
}

async fn fetch_lobbies_async() -> Result<Vec<LobbyInfo>, Box<dyn std::error::Error>> {
    // Fetch from OpenFront.io API
    let client = reqwest::Client::new();
    let response = client
        .get("https://openfront.io/api/public_lobbies")
        .send()
        .await?;
    
    let json = response.text().await?;
    let data: ApiPublicLobbiesResponse = serde_json::from_str(&json)?;
    
    info!("Fetched {} lobbies from API", data.lobbies.len());
    Ok(data.lobbies)
}

fn update_lobbies(
    mut lobby_list: ResMut<LobbyList>,
    time: Res<Time>,
    menu_state: Res<MainMenuState>,
) {
    // Only update lobbies if the menu is actually showing
    if !menu_state.show_menu {
        return;
    }
    
    // Check for fetched lobbies from async task
    if let Ok(content) = std::fs::read_to_string("/tmp/openfront_lobbies.json") {
        if let Ok(lobbies) = serde_json::from_str::<Vec<LobbyInfo>>(&content) {
            let current_time = time.elapsed_seconds_f64();
            
            // Store when each lobby was fetched for countdown calculation
            for lobby in &lobbies {
                lobby_list.lobby_fetch_times.insert(lobby.game_id.clone(), current_time);
            }
            
            lobby_list.lobbies = lobbies;
            lobby_list.is_fetching = false;
            lobby_list.last_update = current_time;
            // Clean up temp file
            let _ = std::fs::remove_file("/tmp/openfront_lobbies.json");
        }
    }
    
    // Refetch every 5 seconds (only if menu is showing)
    if menu_state.show_menu 
        && time.elapsed_seconds_f64() - lobby_list.last_update > 5.0 
        && !lobby_list.is_fetching {
        start_lobby_fetch(lobby_list);
    }
}

fn render_main_menu(
    mut contexts: EguiContexts,
    mut menu_state: ResMut<MainMenuState>,
    lobby_list: Res<LobbyList>,
    connection: Option<Res<WebSocketConnection>>,
    time: Res<Time>,
) {
    // Always show the main menu initially
    if !menu_state.show_menu && menu_state.username.is_empty() {
        menu_state.show_menu = true;
        menu_state.username = "Player".to_string();
    }

    if !menu_state.show_menu {
        return;
    }

    // Main menu window with semi-transparent background
    egui::CentralPanel::default()
        .show(contexts.ctx_mut(), |ui| {
            // Background styling - dark semi-transparent
            let frame = egui::Frame::default()
                .fill(egui::Color32::from_rgba_unmultiplied(20, 25, 40, 220))
                .inner_margin(egui::Margin::same(20.0));
                
            frame.show(ui, |ui| {
                ui.visuals_mut().override_text_color = Some(egui::Color32::WHITE);
            
            // OpenFront title
            ui.vertical_centered(|ui| {
                ui.add_space(20.0);
                
                // Large title
                ui.heading(egui::RichText::new("OPENFRONT").size(48.0).color(egui::Color32::from_rgb(37, 99, 235)));
                ui.label(egui::RichText::new("ALPHA").size(16.0).color(egui::Color32::from_rgb(200, 200, 200)));
                
                ui.add_space(30.0);
                
                // Username input
                ui.horizontal(|ui| {
                    ui.label("Username:");
                    ui.text_edit_singleline(&mut menu_state.username);
                });
                
                ui.add_space(20.0);
                
                // Public lobbies section
                ui.separator();
                ui.heading("Public Lobbies");
                ui.separator();
                
                // Lobby list
                egui::ScrollArea::vertical()
                    .max_height(200.0)
                    .show(ui, |ui| {
                        if lobby_list.is_fetching && lobby_list.lobbies.is_empty() {
                            ui.horizontal(|ui| {
                                ui.spinner();
                                ui.label("Loading lobbies...");
                            });
                        } else if lobby_list.lobbies.is_empty() {
                            ui.vertical_centered(|ui| {
                                ui.label("No public lobbies available");
                                ui.label("Try creating your own or joining a private lobby");
                            });
                        } else {
                            // Show available lobbies
                            for lobby in &lobby_list.lobbies {
                                ui.group(|ui| {
                                    ui.horizontal(|ui| {
                                        // Show map name and game mode
                                        ui.label(format!("{} - {}", 
                                            lobby.game_config.game_map, 
                                            lobby.game_config.game_mode));
                                        
                                        // Show player count
                                        ui.label(format!("{}/{} players", 
                                            lobby.num_clients, 
                                            lobby.game_config.max_players));
                                        
                                        // Calculate real-time countdown
                                        let fetch_time = lobby_list.lobby_fetch_times
                                            .get(&lobby.game_id)
                                            .copied()
                                            .unwrap_or(time.elapsed_seconds_f64());
                                        let elapsed_since_fetch = time.elapsed_seconds_f64() - fetch_time;
                                        let ms_remaining = lobby.ms_until_start.saturating_sub((elapsed_since_fetch * 1000.0) as u32);
                                        let seconds_remaining = ms_remaining / 1000;
                                        
                                        // Show time until start with real-time update
                                        if seconds_remaining > 0 {
                                            ui.label(format!("Starting in {}s", seconds_remaining));
                                            // Request UI update for smooth countdown
                                            ui.ctx().request_repaint();
                                        } else {
                                            ui.label("Starting soon...");
                                        }
                                        
                                        // Join button
                                        if ui.button("Join").clicked() {
                                            if let Some(conn) = &connection {
                                                info!("Joining lobby {}", lobby.game_id);
                                                let client_id = get_or_create_client_id();
                                                let token = get_or_create_token();
                                                send_join(
                                                    conn,
                                                    lobby.game_id.clone(),
                                                    client_id,
                                                    menu_state.username.clone(),
                                                    token,  // Use proper UUID token
                                                    menu_state.selected_flag.clone(),
                                                );
                                                menu_state.show_menu = false;
                                            }
                                        }
                                    });
                                    
                                    // Show additional info on second line
                                    ui.horizontal(|ui| {
                                        ui.label(format!("Type: {} | Difficulty: {} | Bots: {}", 
                                            lobby.game_config.game_type,
                                            lobby.game_config.difficulty,
                                            lobby.game_config.bots.unwrap_or(0)));
                                    });
                                });
                            }
                        }
                    });
                
                ui.add_space(20.0);
                ui.separator();
                
                // Buttons row
                ui.horizontal(|ui| {
                    if ui.button("Create Lobby").clicked() {
                        menu_state.show_create_lobby = true;
                    }
                    
                    if ui.button("Join Private Lobby").clicked() {
                        menu_state.show_join_private = true;
                    }
                });
                
                ui.add_space(10.0);
                
                if ui.button("Single Player").clicked() {
                    info!("Single player mode not yet implemented");
                }
                
                ui.add_space(10.0);
                
                ui.horizontal(|ui| {
                    if ui.button("Instructions").clicked() {
                        menu_state.show_help = true;
                    }
                    
                    if ui.button("Settings").clicked() {
                        menu_state.show_settings = true;
                    }
                });
                
                ui.add_space(20.0);
                
                // Footer links
                ui.separator();
                ui.horizontal(|ui| {
                    ui.hyperlink_to("Discord", "https://discord.gg/jRpxXvG42t");
                    ui.separator();
                    ui.hyperlink_to("Wiki", "https://openfront.miraheze.org/wiki/Main_Page");
                    ui.separator();
                    ui.hyperlink_to("GitHub", "https://github.com/openfrontio/OpenFrontIO");
                });
            });  // End of frame content
        });  // End of frame
    });  // End of central panel
    
    // Help modal
    if menu_state.show_help {
        egui::Window::new("Instructions")
            .collapsible(false)
            .show(contexts.ctx_mut(), |ui| {
                ui.heading("How to Play OpenFront");
                ui.separator();
                
                ui.label("Basic Controls:");
                ui.label("• Click to select territory");
                ui.label("• Right-click to attack");
                ui.label("• WASD/Arrow keys to pan");
                ui.label("• Scroll to zoom");
                
                ui.separator();
                ui.label("Game Objective:");
                ui.label("Expand your territory and eliminate other players!");
                
                ui.separator();
                if ui.button("Close").clicked() {
                    menu_state.show_help = false;
                }
            });
    }
    
    // Settings modal
    if menu_state.show_settings {
        egui::Window::new("Settings")
            .collapsible(false)
            .show(contexts.ctx_mut(), |ui| {
                ui.heading("Game Settings");
                ui.separator();
                
                ui.checkbox(&mut true, "Enable sound");
                ui.checkbox(&mut true, "Show FPS");
                ui.checkbox(&mut true, "Territory patterns");
                
                ui.separator();
                if ui.button("Close").clicked() {
                    menu_state.show_settings = false;
                }
            });
    }
    
    // Create lobby modal
    if menu_state.show_create_lobby {
        egui::Window::new("Create Lobby")
            .collapsible(false)
            .show(contexts.ctx_mut(), |ui| {
                ui.label("Creating lobbies not yet implemented");
                
                if ui.button("Close").clicked() {
                    menu_state.show_create_lobby = false;
                }
            });
    }
    
    // Join private lobby modal
    if menu_state.show_join_private {
        egui::Window::new("Join Private Lobby")
            .collapsible(false)
            .show(contexts.ctx_mut(), |ui| {
                ui.horizontal(|ui| {
                    ui.label("Game ID:");
                    ui.text_edit_singleline(&mut menu_state.private_lobby_code);
                });
                
                ui.label("Example: PM7jP0nv");
                
                ui.horizontal(|ui| {
                    if ui.button("Join").clicked() && !menu_state.private_lobby_code.is_empty() {
                        if let Some(conn) = &connection {
                            info!("Joining private lobby: {}", menu_state.private_lobby_code);
                            let client_id = get_or_create_client_id();
                            let token = get_or_create_token();
                            send_join(
                                conn,
                                menu_state.private_lobby_code.clone(),
                                client_id,
                                menu_state.username.clone(),
                                token,  // Use proper UUID token
                                menu_state.selected_flag.clone(),
                            );
                            menu_state.show_join_private = false;
                            menu_state.show_menu = false;
                        }
                    }
                    
                    if ui.button("Cancel").clicked() {
                        menu_state.show_join_private = false;
                    }
                });
            });
    }
}