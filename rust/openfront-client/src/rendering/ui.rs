use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

pub mod main_menu;

pub struct UIRenderingPlugin;

impl Plugin for UIRenderingPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(main_menu::MainMenuPlugin)
            .init_resource::<UIState>()
            .add_systems(Update, (
                render_game_ui,
                render_leaderboard,
                render_chat,
                render_build_menu,
            ));
    }
}

#[derive(Resource, Default)]
pub struct UIState {
    pub show_leaderboard: bool,
    pub show_chat: bool,
    pub show_build_menu: bool,
    pub show_settings: bool,
    pub selected_tile: Option<(i32, i32)>,
    pub fps: f32,
    pub ping: u32,
}

fn render_game_ui(
    mut contexts: EguiContexts,
    mut ui_state: ResMut<UIState>,
    time: Res<Time>,
    menu_state: Res<main_menu::MainMenuState>,
) {
    // Don't show game UI if main menu is open
    if menu_state.show_menu {
        return;
    }
    
    // Top bar with game info
    egui::TopBottomPanel::top("top_panel").show(contexts.ctx_mut(), |ui| {
        ui.horizontal(|ui| {
            ui.label("OpenFront.io");
            ui.separator();
            
            // FPS display
            let fps = 1.0 / time.delta_seconds();
            ui_state.fps = ui_state.fps * 0.9 + fps * 0.1; // Smooth FPS
            
            let fps_color = if ui_state.fps >= 55.0 {
                egui::Color32::GREEN
            } else if ui_state.fps >= 30.0 {
                egui::Color32::YELLOW
            } else {
                egui::Color32::RED
            };
            
            ui.colored_label(fps_color, format!("FPS: {:.0}", ui_state.fps));
            ui.separator();
            
            // Ping display
            ui.label(format!("Ping: {}ms", ui_state.ping));
            ui.separator();
            
            // Toggle buttons
            if ui.button("Leaderboard (Tab)").clicked() {
                ui_state.show_leaderboard = !ui_state.show_leaderboard;
            }
            
            if ui.button("Chat (Enter)").clicked() {
                ui_state.show_chat = !ui_state.show_chat;
            }
            
            if ui.button("Settings (Esc)").clicked() {
                ui_state.show_settings = !ui_state.show_settings;
            }
        });
    });
    
    // Settings modal
    if ui_state.show_settings {
        egui::Window::new("Settings")
            .collapsible(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(contexts.ctx_mut(), |ui| {
                ui.heading("Game Settings");
                ui.separator();
                
                ui.horizontal(|ui| {
                    ui.label("Master Volume:");
                    ui.add(egui::Slider::new(&mut 1.0, 0.0..=1.0));
                });
                
                ui.horizontal(|ui| {
                    ui.label("Graphics Quality:");
                    ui.radio_value(&mut "high", "low", "Low");
                    ui.radio_value(&mut "high", "medium", "Medium");
                    ui.radio_value(&mut "high", "high", "High");
                });
                
                ui.checkbox(&mut true, "Show FPS");
                ui.checkbox(&mut true, "Show Ping");
                ui.checkbox(&mut true, "Enable Animations");
                
                ui.separator();
                
                if ui.button("Close").clicked() {
                    ui_state.show_settings = false;
                }
            });
    }
}

fn render_leaderboard(
    mut contexts: EguiContexts,
    ui_state: Res<UIState>,
) {
    if !ui_state.show_leaderboard {
        return;
    }
    
    egui::SidePanel::right("leaderboard")
        .min_width(200.0)
        .show(contexts.ctx_mut(), |ui| {
            ui.heading("Leaderboard");
            ui.separator();
            
            // Mock leaderboard data
            let players = vec![
                ("Player1", 1500, 25),
                ("Player2", 1200, 20),
                ("Player3", 800, 15),
                ("Player4", 600, 12),
                ("Player5", 400, 8),
            ];
            
            egui::ScrollArea::vertical().show(ui, |ui| {
                for (i, (name, score, territories)) in players.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(format!("{}.", i + 1));
                        ui.label(*name);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(format!("{} tiles", territories));
                            ui.label(format!("{} pts", score));
                        });
                    });
                    ui.separator();
                }
            });
        });
}

fn render_chat(
    mut contexts: EguiContexts,
    ui_state: Res<UIState>,
) {
    if !ui_state.show_chat {
        return;
    }
    
    egui::Window::new("Chat")
        .anchor(egui::Align2::LEFT_BOTTOM, [10.0, -10.0])
        .default_width(300.0)
        .default_height(200.0)
        .show(contexts.ctx_mut(), |ui| {
            egui::ScrollArea::vertical()
                .max_height(150.0)
                .show(ui, |ui| {
                    // Mock chat messages
                    ui.label("Player1: Good luck!");
                    ui.label("Player2: Let's team up");
                    ui.label("System: Player3 has joined");
                    ui.label("Player4: Attack Player1!");
                });
            
            ui.separator();
            
            // Chat input
            let mut message = String::new();
            let response = ui.text_edit_singleline(&mut message);
            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                // Send chat message
                info!("Chat message: {}", message);
            }
        });
}

fn render_build_menu(
    mut contexts: EguiContexts,
    ui_state: Res<UIState>,
) {
    if !ui_state.show_build_menu || ui_state.selected_tile.is_none() {
        return;
    }
    
    let (q, r) = ui_state.selected_tile.unwrap();
    
    egui::Window::new("Build Menu")
        .anchor(egui::Align2::CENTER_BOTTOM, [0.0, -50.0])
        .show(contexts.ctx_mut(), |ui| {
            ui.label(format!("Selected tile: ({}, {})", q, r));
            ui.separator();
            
            ui.horizontal(|ui| {
                // Building options
                if ui.button("🏰 Capital\n500g").clicked() {
                    info!("Build capital");
                }
                if ui.button("🏘️ City\n200g").clicked() {
                    info!("Build city");
                }
                if ui.button("⛺ Camp\n50g").clicked() {
                    info!("Build camp");
                }
                if ui.button("🗼 Tower\n100g").clicked() {
                    info!("Build tower");
                }
            });
            
            ui.horizontal(|ui| {
                if ui.button("🏰 Fort\n300g").clicked() {
                    info!("Build fort");
                }
                if ui.button("⚓ Port\n150g").clicked() {
                    info!("Build port");
                }
                if ui.button("✈️ Airport\n400g").clicked() {
                    info!("Build airport");
                }
                if ui.button("📡 Radar\n250g").clicked() {
                    info!("Build radar");
                }
            });
        });
}

// Radial menu for quick actions
pub fn render_radial_menu(
    contexts: &mut EguiContexts,
    center: Vec2,
    options: Vec<(&str, impl Fn())>,
) {
    let radius = 80.0;
    let angle_step = 360.0 / options.len() as f32;
    
    for (i, (label, _action)) in options.iter().enumerate() {
        let angle = (angle_step * i as f32).to_radians();
        let x = center.x + radius * angle.cos();
        let y = center.y + radius * angle.sin();
        
        egui::Window::new(format!("radial_{}", i))
            .id(egui::Id::new(format!("radial_{}", i)))
            .fixed_pos([x, y])
            .title_bar(false)
            .resizable(false)
            .show(contexts.ctx_mut(), |ui| {
                if ui.button(*label).clicked() {
                    // action();
                }
            });
    }
}