use bevy::prelude::*;
use tokio::runtime::Runtime;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{StreamExt, SinkExt};
use tokio::sync::mpsc;

use super::schemas::{ClientMessage, ServerMessage};

#[derive(Resource)]
pub struct WebSocketConfig {
    pub url: String,
    pub auto_reconnect: bool,
    pub reconnect_delay_secs: u64,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        // This will be updated dynamically based on game ID
        Self {
            url: String::new(), // Will be set when joining a game
            auto_reconnect: true,
            reconnect_delay_secs: 5,
        }
    }
}

impl WebSocketConfig {
    pub fn for_game(game_id: &str) -> Self {
        // Calculate worker index from game ID hash
        let worker_index = calculate_worker_index(game_id);
        let url = format!("wss://openfront.io/w{}", worker_index);
        
        info!("🔧 Calculated WebSocket URL for game {}: {}", game_id, url);
        
        Self {
            url,
            auto_reconnect: true,
            reconnect_delay_secs: 5,
        }
    }
}

fn simple_hash(s: &str) -> u32 {
    let mut hash: i32 = 0;
    for ch in s.chars() {
        let char_code = ch as i32;
        hash = ((hash << 5).wrapping_sub(hash)).wrapping_add(char_code);
    }
    hash.unsigned_abs()
}

fn calculate_worker_index(game_id: &str) -> u32 {
    // OpenFront.io uses 4 workers in production
    const NUM_WORKERS: u32 = 4;
    simple_hash(game_id) % NUM_WORKERS
}

use std::sync::Mutex;
use once_cell::sync::Lazy;

// Global channel for game messages
static GAME_MESSAGE_SENDER: Lazy<Mutex<Option<mpsc::UnboundedSender<ServerMessage>>>> = 
    Lazy::new(|| Mutex::new(None));

#[derive(Resource)]
pub struct WebSocketConnection {
    pub sender: mpsc::UnboundedSender<ClientMessage>,
    pub receiver: mpsc::UnboundedReceiver<ServerMessage>,
}

#[derive(Event)]
pub struct NetworkEvent {
    pub message: ServerMessage,
}

#[derive(Event)]
pub struct ConnectionStatus {
    pub connected: bool,
    pub error: Option<String>,
}

pub fn setup_websocket(
    config: Res<WebSocketConfig>,
    mut commands: Commands,
) {
    // Don't connect immediately - connections will be made per game
    info!("🔌 WebSocket system initialized");
    
    // Create channels for game messages
    let (client_tx, client_rx) = mpsc::unbounded_channel::<ClientMessage>();
    let (server_tx, server_rx) = mpsc::unbounded_channel::<ServerMessage>();
    
    // Store the server sender globally so the WebSocket thread can send messages
    *GAME_MESSAGE_SENDER.lock().unwrap() = Some(server_tx);
    
    // Store the connection resource
    commands.insert_resource(WebSocketConnection {
        sender: client_tx,
        receiver: server_rx,
    });
}

async fn websocket_loop_with_join(
    url: String,
    mut client_rx: mpsc::UnboundedReceiver<ClientMessage>,
    server_tx: mpsc::UnboundedSender<ServerMessage>,
    client_tx: mpsc::UnboundedSender<ClientMessage>,
    conn_info: ConnectionInfo,
) {
    let mut connection_attempts = 0;
    
    loop {
        connection_attempts += 1;
        info!("🔗 Connection attempt #{} to {}", connection_attempts, url);
        
        match connect_async(&url).await {
            Ok((ws_stream, response)) => {
                info!("✅ WebSocket connected successfully!");
                info!("📡 Server response status: {}", response.status());
                if !response.headers().is_empty() {
                    info!("📋 Response headers: {:?}", response.headers());
                }
                
                let (mut write, mut read) = ws_stream.split();
                info!("🔀 WebSocket stream split, ready for bidirectional communication");
                
                // Send join message immediately after connection
                let join_msg = ClientMessage::Join(super::schemas::ClientJoinMessage {
                    game_id: conn_info.game_id.clone(),
                    client_id: conn_info.client_id.clone(),
                    last_turn: 0,
                    token: conn_info.token.clone(),
                    username: conn_info.username.clone(),
                    flag: conn_info.flag.clone(),
                    pattern: None,
                });
                
                let join_json = serde_json::to_string(&join_msg).unwrap();
                info!("📤 Sending join message after connection: {}", join_json);
                
                match write.send(Message::Text(join_json.clone())).await {
                    Ok(_) => {
                        info!("✅ Join message sent successfully on connection");
                    }
                    Err(e) => {
                        error!("❌ Failed to send join message: {}", e);
                        continue; // Retry connection
                    }
                }
                
                // Start ping interval
                let mut ping_interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
                ping_interval.tick().await; // Skip first immediate tick
                
                loop {
                    tokio::select! {
                        // Send ping every 5 seconds
                        _ = ping_interval.tick() => {
                            // For now, use tick 0 - in a real game this would be the current game tick
                            let ping_msg = ClientMessage::Ping(super::schemas::ClientPingMessage { tick: 0 });
                            let ping_json = serde_json::to_string(&ping_msg).unwrap();
                            debug!("🏓 Sending ping to keep connection alive");
                            
                            match write.send(Message::Text(ping_json)).await {
                                Ok(_) => {
                                    debug!("✅ Ping sent");
                                }
                                Err(e) => {
                                    error!("❌ Failed to send ping: {}", e);
                                    break;
                                }
                            }
                        }
                        
                        // Handle outgoing messages
                        Some(msg) = client_rx.recv() => {
                            let json = serde_json::to_string(&msg).unwrap();
                            info!("📤 Sending to server: {}", json);
                            
                            match write.send(Message::Text(json.clone())).await {
                                Ok(_) => {
                                    info!("✅ Message sent successfully");
                                }
                                Err(e) => {
                                    error!("❌ Failed to send message: {}", e);
                                    error!("   Message was: {}", json);
                                    break;
                                }
                            }
                        }
                        
                        // Handle incoming messages
                        Some(result) = read.next() => {
                            info!("📨 Received WebSocket frame: {:?}", result);
                            match result {
                                Ok(Message::Text(text)) => {
                                    info!("📥 Text message from server ({}bytes): {}", text.len(), 
                                        if text.len() > 500 { 
                                            format!("{}...", &text[..500]) 
                                        } else { 
                                            text.clone() 
                                        }
                                    );
                                    
                                    // Try to parse as JSON first to see structure
                                    match serde_json::from_str::<serde_json::Value>(&text) {
                                        Ok(json_value) => {
                                            info!("📄 JSON structure: {}", serde_json::to_string_pretty(&json_value).unwrap_or_else(|_| "failed to pretty print".to_string()));
                                        }
                                        Err(e) => {
                                            warn!("⚠️ Not valid JSON: {}", e);
                                        }
                                    }
                                    
                                    match serde_json::from_str::<ServerMessage>(&text) {
                                        Ok(msg) => {
                                            info!("✅ Parsed server message type: {:?}", msg);
                                            if server_tx.send(msg).is_err() {
                                                error!("❌ Failed to forward server message to game");
                                            } else {
                                                info!("📨 Message forwarded to game systems");
                                            }
                                        }
                                        Err(e) => {
                                            error!("❌ Failed to deserialize as ServerMessage: {}", e);
                                            error!("   Raw message was: {}", text);
                                        }
                                    }
                                }
                                Ok(Message::Binary(data)) => {
                                    info!("📥 Binary message received ({} bytes)", data.len());
                                }
                                Ok(Message::Ping(data)) => {
                                    debug!("🏓 Ping received ({} bytes)", data.len());
                                }
                                Ok(Message::Pong(data)) => {
                                    debug!("🏓 Pong received ({} bytes)", data.len());
                                }
                                Ok(Message::Close(frame)) => {
                                    if let Some(close_frame) = frame {
                                        error!("🚪 WebSocket closed by server with code {} and reason: {}", 
                                            close_frame.code, close_frame.reason);
                                    } else {
                                        error!("🚪 WebSocket closed by server without close frame");
                                    }
                                    break;
                                }
                                Ok(Message::Frame(_)) => {
                                    debug!("📦 Raw frame received");
                                }
                                Err(e) => {
                                    error!("❌ WebSocket error: {}", e);
                                    break;
                                }
                            }
                        }
                    }
                }
                
                warn!("⚠️ WebSocket connection lost, will attempt reconnection");
            }
            Err(e) => {
                error!("❌ Failed to connect to WebSocket: {}", e);
                error!("   URL: {}", url);
            }
        }
        
        // Reconnect delay
        warn!("⏳ Waiting 5 seconds before reconnection attempt...");
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}

pub fn handle_messages(
    mut connection: ResMut<WebSocketConnection>,
    mut network_events: EventWriter<NetworkEvent>,
) {
    // Process all pending messages
    let mut message_count = 0;
    while let Ok(message) = connection.receiver.try_recv() {
        message_count += 1;
        info!("📨 Forwarding message #{} from WebSocket to game: {:?}", message_count, message);
        network_events.send(NetworkEvent { message });
    }
    
    if message_count > 0 {
        info!("📊 Processed {} message(s) from WebSocket", message_count);
    }
}

// Store connection info for reconnection
#[derive(Clone)]
struct ConnectionInfo {
    game_id: String,
    client_id: String,
    username: String,
    token: String,
    flag: String,
}

// Helper function to establish connection and send join
pub fn connect_and_join(
    game_id: String,
    client_id: String,
    username: String,
    token: String,
    flag: String,
) {
    info!("🎮 === CONNECTING TO GAME ===");
    info!("  📝 Game ID: {}", game_id);
    
    // Calculate the correct worker URL
    let config = WebSocketConfig::for_game(&game_id);
    let url = config.url.clone();
    
    info!("  🌐 Connecting to: {}", url);
    
    // Get the global server sender
    let server_tx = GAME_MESSAGE_SENDER.lock().unwrap().clone();
    if server_tx.is_none() {
        error!("❌ WebSocket system not initialized!");
        return;
    }
    let server_tx = server_tx.unwrap();
    
    // Create client channel for this connection
    let (client_tx, client_rx) = mpsc::unbounded_channel::<ClientMessage>();
    
    // Store connection info for reconnection
    let conn_info = ConnectionInfo {
        game_id: game_id.clone(),
        client_id: client_id.clone(),
        username: username.clone(),
        token: token.clone(),
        flag: flag.clone(),
    };
    
    info!("  🆔 Client ID: {}", client_id);
    info!("  👤 Username: {}", username);
    info!("  🔑 Token: {}", token);
    info!("  🏳️ Flag: {}", flag);
    
    info!("✅ Starting WebSocket connection...");
    
    // Start the WebSocket connection in a new thread
    std::thread::spawn(move || {
        info!("🧵 WebSocket thread started for game");
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            websocket_loop_with_join(url, client_rx, server_tx, client_tx, conn_info).await;
        });
    });
}

// Keep the old function for compatibility but mark it as deprecated
pub fn send_join(
    connection: &WebSocketConnection,
    game_id: String,
    client_id: String,
    username: String,
    token: String,
    flag: String,
) {
    warn!("⚠️ Using deprecated send_join - should use connect_and_join instead");
    connect_and_join(game_id, client_id, username, token, flag);
}

// Original websocket loop without auto-join
async fn websocket_loop(
    url: String,
    mut client_rx: mpsc::UnboundedReceiver<ClientMessage>,
    server_tx: mpsc::UnboundedSender<ServerMessage>,
) {
    let mut connection_attempts = 0;
    
    loop {
        connection_attempts += 1;
        info!("🔗 Connection attempt #{} to {}", connection_attempts, url);
        
        match connect_async(&url).await {
            Ok((ws_stream, response)) => {
                info!("✅ WebSocket connected successfully!");
                info!("📡 Server response status: {}", response.status());
                
                let (mut write, mut read) = ws_stream.split();
                info!("🔀 WebSocket stream split, ready for bidirectional communication");
                
                loop {
                    tokio::select! {
                        // Handle outgoing messages
                        Some(msg) = client_rx.recv() => {
                            let json = serde_json::to_string(&msg).unwrap();
                            info!("📤 Sending to server: {}", json);
                            
                            match write.send(Message::Text(json.clone())).await {
                                Ok(_) => {
                                    info!("✅ Message sent successfully");
                                }
                                Err(e) => {
                                    error!("❌ Failed to send message: {}", e);
                                    break;
                                }
                            }
                        }
                        
                        // Handle incoming messages
                        Some(result) = read.next() => {
                            match result {
                                Ok(Message::Text(text)) => {
                                    info!("📥 Raw message from server: {}", text);
                                    
                                    match serde_json::from_str::<ServerMessage>(&text) {
                                        Ok(msg) => {
                                            info!("✅ Parsed server message: {:?}", msg);
                                            if server_tx.send(msg).is_err() {
                                                error!("❌ Failed to forward server message to game");
                                            }
                                        }
                                        Err(e) => {
                                            error!("❌ Failed to deserialize message: {}", e);
                                        }
                                    }
                                }
                                Ok(Message::Close(frame)) => {
                                    if let Some(close_frame) = frame {
                                        error!("🚪 WebSocket closed by server with code {} and reason: {}", 
                                            close_frame.code, close_frame.reason);
                                    } else {
                                        error!("🚪 WebSocket closed by server without close frame");
                                    }
                                    break;
                                }
                                Err(e) => {
                                    error!("❌ WebSocket error: {}", e);
                                    break;
                                }
                                _ => {}
                            }
                        }
                    }
                }
                
                warn!("⚠️ WebSocket connection lost, will attempt reconnection");
            }
            Err(e) => {
                error!("❌ Failed to connect to WebSocket: {}", e);
            }
        }
        
        // Reconnect delay
        warn!("⏳ Waiting 5 seconds before reconnection attempt...");
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    }
}

pub fn send_intent(
    connection: &WebSocketConnection,
    tick: u32,
    intent: super::schemas::Intent,
) {
    let message = ClientMessage::Intent(super::schemas::ClientIntentMessage {
        tick,
        intent,
    });
    
    if let Err(e) = connection.sender.send(message) {
        error!("Failed to send intent: {}", e);
    }
}