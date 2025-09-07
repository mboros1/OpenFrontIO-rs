use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Resource, Default)]
pub struct GameState {
    pub tick: u32,
    pub players: HashMap<u32, Player>,
    pub territories: HashMap<(i32, i32), Territory>,
    pub units: HashMap<u32, Unit>,
    pub my_player_id: Option<u32>,
    pub game_speed: f32,
    pub game_started: bool,
}

#[derive(Debug, Clone)]
pub struct Player {
    pub id: u32,
    pub name: String,
    pub color: Color,
    pub gold: u32,
    pub troops: u32,
    pub territory_count: u32,
    pub is_alive: bool,
    pub team: Option<u32>,
    pub cosmetics: PlayerCosmetics,
}

#[derive(Debug, Clone)]
pub struct PlayerCosmetics {
    pub flag: String,
    pub pattern: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Territory {
    pub position: (i32, i32), // Hex coordinates (q, r)
    pub owner: Option<u32>,   // Player ID
    pub troops: u32,
    pub terrain_type: TerrainType,
}

#[derive(Debug, Clone, Copy)]
pub enum TerrainType {
    Land,
    Water,
    Mountain,
    Forest,
    Desert,
}

#[derive(Debug, Clone)]
pub struct Unit {
    pub id: u32,
    pub unit_type: UnitType,
    pub owner: u32,
    pub position: (i32, i32),
    pub health: f32,
    pub max_health: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnitType {
    Capital,
    City,
    Camp,
    Tower,
    Fort,
    Port,
    Airport,
    Radar,
    Bridge,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            tick: 0,
            players: HashMap::new(),
            territories: HashMap::new(),
            units: HashMap::new(),
            my_player_id: None,
            game_speed: 1.0,
            game_started: false,
        }
    }
    
    pub fn add_player(&mut self, player: Player) {
        self.players.insert(player.id, player);
    }
    
    pub fn update_territory(&mut self, position: (i32, i32), owner: Option<u32>, troops: u32) {
        if let Some(territory) = self.territories.get_mut(&position) {
            territory.owner = owner;
            territory.troops = troops;
        }
    }
    
    pub fn add_unit(&mut self, unit: Unit) {
        self.units.insert(unit.id, unit);
    }
    
    pub fn remove_unit(&mut self, unit_id: u32) {
        self.units.remove(&unit_id);
    }
    
    pub fn get_my_player(&self) -> Option<&Player> {
        self.my_player_id.and_then(|id| self.players.get(&id))
    }
    
    pub fn is_my_territory(&self, position: (i32, i32)) -> bool {
        self.territories
            .get(&position)
            .and_then(|t| t.owner)
            .map(|owner| Some(owner) == self.my_player_id)
            .unwrap_or(false)
    }
    
    pub fn can_build_at(&self, position: (i32, i32)) -> bool {
        if !self.is_my_territory(position) {
            return false;
        }
        
        // Check if there's already a unit at this position
        !self.units.values().any(|u| u.position == position)
    }
}

pub fn update_game_state(
    mut game_state: ResMut<GameState>,
    time: Res<Time>,
) {
    if !game_state.game_started {
        return;
    }
    
    // Update game tick based on game speed
    let tick_duration = 1.0 / game_state.game_speed;
    let ticks_elapsed = (time.delta_seconds() / tick_duration) as u32;
    game_state.tick += ticks_elapsed;
    
    // Process game updates (this would come from server messages)
}