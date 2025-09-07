use serde::{Deserialize, Serialize};

// Type aliases for clarity
pub type ClientID = String;
pub type GameID = String;
pub type PlayerID = u32;
pub type Tick = u32;

// Client -> Server Messages
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    #[serde(rename = "join")]
    Join(ClientJoinMessage),
    
    #[serde(rename = "intent")]
    Intent(ClientIntentMessage),
    
    #[serde(rename = "ping")]
    Ping(ClientPingMessage),
    
    #[serde(rename = "hash")]
    Hash(ClientHashMessage),
    
    #[serde(rename = "winner")]
    Winner(ClientWinnerMessage),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientJoinMessage {
    #[serde(rename = "gameID")]
    pub game_id: GameID,
    #[serde(rename = "clientID")]
    pub client_id: ClientID,
    #[serde(rename = "lastTurn")]
    pub last_turn: u32,
    pub token: String,
    pub username: String,
    pub flag: String,
    pub pattern: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientIntentMessage {
    pub tick: Tick,
    pub intent: Intent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientPingMessage {
    pub tick: Tick,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientHashMessage {
    pub tick: Tick,
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientWinnerMessage {
    pub winner: Winner,
    pub stats: AllPlayersStats,
}

// Server -> Client Messages
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    #[serde(rename = "prestart")]
    Prestart(PrestartMessage),
    
    #[serde(rename = "start")]
    Start(StartMessage),
    
    #[serde(rename = "update")]
    Update(UpdateMessage),
    
    #[serde(rename = "pong")]
    Pong(PongMessage),
    
    #[serde(rename = "error")]
    Error(ErrorMessage),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrestartMessage {
    pub game_map: String,
    pub game_type: GameType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartMessage {
    pub game_start_info: GameStartInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMessage {
    pub tick: Tick,
    pub updates: Vec<GameUpdate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PongMessage {
    pub tick: Tick,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMessage {
    #[serde(alias = "message", alias = "error")]
    pub error: String,
}

// Game Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameType {
    #[serde(rename = "1v1")]
    OneVsOne,
    #[serde(rename = "ffa")]
    FreeForAll,
    #[serde(rename = "teams")]
    Teams,
}

// Intent Types (Player Actions)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Intent {
    #[serde(rename = "spawn")]
    Spawn { tile: TileRef },
    
    #[serde(rename = "attack")]
    Attack {
        target_id: Option<PlayerID>,
        troops: u32,
    },
    
    #[serde(rename = "boat_attack")]
    BoatAttack {
        target_id: Option<PlayerID>,
        dst: TileRef,
        troops: u32,
        src: Option<TileRef>,
    },
    
    #[serde(rename = "build")]
    Build {
        unit: UnitType,
        tile: TileRef,
    },
    
    #[serde(rename = "upgrade")]
    Upgrade {
        unit_id: u32,
        unit_type: UnitType,
    },
    
    #[serde(rename = "delete_unit")]
    DeleteUnit {
        unit_id: u32,
    },
    
    #[serde(rename = "cancel_attack")]
    CancelAttack {
        attack_id: String,
    },
    
    #[serde(rename = "alliance_request")]
    AllianceRequest {
        recipient: PlayerID,
    },
    
    #[serde(rename = "alliance_reply")]
    AllianceReply {
        requestor: PlayerID,
        accepted: bool,
    },
    
    #[serde(rename = "break_alliance")]
    BreakAlliance {
        recipient: PlayerID,
    },
    
    #[serde(rename = "emoji")]
    Emoji {
        recipient: EmojiRecipient,
        emoji: u32,
    },
    
    #[serde(rename = "donate_gold")]
    DonateGold {
        recipient: PlayerID,
        gold: Option<u32>,
    },
    
    #[serde(rename = "donate_troops")]
    DonateTroops {
        recipient: PlayerID,
        troops: Option<u32>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EmojiRecipient {
    Player(PlayerID),
    All(String), // "all"
}

// Basic Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TileRef {
    pub q: i32,
    pub r: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerCosmetics {
    pub flag: String,
    pub pattern: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameStartInfo {
    pub config: GameConfig,
    pub players: Vec<PlayerInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    pub map_name: String,
    pub max_players: u32,
    pub game_speed: f32,
    // Add other config fields as needed
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerInfo {
    pub id: PlayerID,
    pub name: String,
    pub cosmetics: PlayerCosmetics,
    pub team: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameUpdate {
    // Add game update types as needed
    TerritoryUpdate,
    UnitUpdate,
    PlayerUpdate,
    HashUpdate,
    WinUpdate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Winner {
    pub player_id: Option<PlayerID>,
    pub team_id: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllPlayersStats {
    // Placeholder for stats structure
}