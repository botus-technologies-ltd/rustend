//! WebSocket Module
//!
//! Generic WebSocket module for real-time communication.
//! Supports notifications, alerts, messaging, and live updates.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::broadcast;

/// WebSocket message type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageType {
    /// Text message
    Text,
    /// JSON message
    Json,
    /// Binary message
    Binary,
    /// Ping/pong
    Ping,
    Pong,
    /// Subscribe to channel
    Subscribe,
    /// Unsubscribe from channel
    Unsubscribe,
    /// Acknowledgment
    Ack,
    /// Error
    Error,
}

impl Default for MessageType {
    fn default() -> Self {
        Self::Text
    }
}

/// WebSocket message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsMessage {
    /// Message type
    pub msg_type: MessageType,
    /// Channel/topic
    pub channel: Option<String>,
    /// Payload
    pub payload: String,
    /// Message ID for acknowledgments
    pub id: Option<String>,
    /// Timestamp
    pub timestamp: i64,
}

impl WsMessage {
    pub fn new(msg_type: MessageType, payload: impl Into<String>) -> Self {
        Self {
            msg_type,
            channel: None,
            payload: payload.into(),
            id: None,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    pub fn text(payload: impl Into<String>) -> Self {
        Self::new(MessageType::Text, payload)
    }

    pub fn json(payload: impl Into<String>) -> Self {
        Self::new(MessageType::Json, payload)
    }

    pub fn channel(mut self, channel: impl Into<String>) -> Self {
        self.channel = Some(channel.into());
        self
    }

    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn ack(payload: impl Into<String>, original_id: impl Into<String>) -> Self {
        Self::new(MessageType::Ack, payload).with_id(original_id)
    }

    pub fn error(payload: impl Into<String>) -> Self {
        Self::new(MessageType::Error, payload)
    }
}

/// Connection info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub id: String,
    pub user_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub channels: Vec<String>,
    pub connected_at: i64,
}

impl ConnectionInfo {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            user_id: None,
            ip_address: None,
            user_agent: None,
            channels: Vec::new(),
            connected_at: chrono::Utc::now().timestamp(),
        }
    }

    pub fn with_user(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    pub fn with_ip(mut self, ip: impl Into<String>) -> Self {
        self.ip_address = Some(ip.into());
        self
    }

    pub fn with_agent(mut self, agent: impl Into<String>) -> Self {
        self.user_agent = Some(agent.into());
        self
    }

    pub fn subscribe(&mut self, channel: impl Into<String>) {
        let channel_str = channel.into();
        if !self.channels.contains(&channel_str) {
            self.channels.push(channel_str);
        }
    }

    pub fn unsubscribe(&mut self, channel: &str) {
        self.channels.retain(|c| c != channel);
    }
}

/// WebSocket events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event", content = "data")]
pub enum WsEvent {
    /// Client connected
    Connected(ConnectionInfo),
    /// Client disconnected
    Disconnected { id: String, reason: Option<String> },
    /// Message received
    Message { connection_id: String, message: WsMessage },
    /// Client subscribed to channel
    Subscribed { connection_id: String, channel: String },
    /// Client unsubscribed from channel
    Unsubscribed { connection_id: String, channel: String },
    /// Error
    Error { connection_id: String, error: String },
    /// Broadcast to channel
    Broadcast { channel: String, message: WsMessage },
    /// Send to specific user
    SendToUser { user_id: String, message: WsMessage },
    /// Send to specific connection
    SendToConnection { connection_id: String, message: WsMessage },
}

/// Connection handler trait
#[async_trait]
pub trait ConnectionHandler: Send + Sync {
    /// Handle new connection
    async fn on_connect(&self, _info: ConnectionInfo) {}
    
    /// Handle disconnection
    async fn on_disconnect(&self, _connection_id: &str, _reason: Option<String>) {}
    
    /// Handle incoming message
    async fn on_message(&self, _connection_id: &str, _message: WsMessage) {}
    
    /// Handle error
    async fn on_error(&self, _connection_id: &str, _error: String) {}
}

/// No-op handler
pub struct NoOpHandler;

#[async_trait]
impl ConnectionHandler for NoOpHandler {}

/// WebSocket server configuration
#[derive(Debug, Clone)]
pub struct WsServerConfig {
    /// Server host
    pub host: String,
    /// Server port
    pub port: u16,
    /// Path for WebSocket endpoint
    pub path: String,
    /// Max connections
    pub max_connections: usize,
    /// Message queue size
    pub message_queue_size: usize,
    /// Ping interval in seconds
    pub ping_interval_secs: u64,
    /// Pong timeout in seconds
    pub pong_timeout_secs: u64,
}

impl Default for WsServerConfig {
    fn default() -> Self {
        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            path: "/ws".to_string(),
            max_connections: 10000,
            message_queue_size: 100,
            ping_interval_secs: 30,
            pong_timeout_secs: 10,
        }
    }
}

impl WsServerConfig {
    pub fn new(host: impl Into<String>, port: u16) -> Self {
        Self { host: host.into(), port, ..Default::default() }
    }
    pub fn path(mut self, path: impl Into<String>) -> Self { self.path = path.into(); self }
    pub fn max_connections(mut self, max: usize) -> Self { self.max_connections = max; self }
    pub fn message_queue_size(mut self, size: usize) -> Self { self.message_queue_size = size; self }
    pub fn ping_interval(mut self, secs: u64) -> Self { self.ping_interval_secs = secs; self }
}

/// Channel subscriber
#[derive(Clone)]
pub struct ChannelSubscriber {
    sender: broadcast::Sender<WsMessage>,
}

impl ChannelSubscriber {
    pub fn new(sender: broadcast::Sender<WsMessage>) -> Self {
        Self { sender }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<WsMessage> {
        self.sender.subscribe()
    }

    pub fn broadcast(&self, message: WsMessage) -> Result<usize, WsError> {
        self.sender.send(message).map_err(|_| WsError::ChannelClosed)
    }
}

/// WebSocket hub - manages connections and channels
#[derive(Clone)]
pub struct WsHub {
    connections: Arc<parking_lot::RwLock<HashMap<String, ConnectionInfo>>>,
    channels: Arc<parking_lot::RwLock<HashMap<String, broadcast::Sender<WsMessage>>>>,
    config: WsServerConfig,
}

impl WsHub {
    pub fn new(config: WsServerConfig) -> Self {
        Self {
            connections: Arc::new(parking_lot::RwLock::new(HashMap::new())),
            channels: Arc::new(parking_lot::RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Register a new connection
    pub fn register_connection(&self, info: ConnectionInfo) {
        self.connections.write().insert(info.id.clone(), info);
    }

    /// Remove a connection
    pub fn remove_connection(&self, connection_id: &str) -> Option<ConnectionInfo> {
        self.connections.write().remove(connection_id)
    }

    /// Get connection info
    pub fn get_connection(&self, connection_id: &str) -> Option<ConnectionInfo> {
        self.connections.read().get(connection_id).cloned()
    }

    /// Get all connections for a user
    pub fn get_user_connections(&self, user_id: &str) -> Vec<String> {
        self.connections.read()
            .values()
            .filter(|c| c.user_id.as_deref() == Some(user_id))
            .map(|c| c.id.clone())
            .collect()
    }

    /// Create or get a channel
    pub fn get_or_create_channel(&self, name: &str) -> ChannelSubscriber {
        let mut channels = self.channels.write();
        if let Some(sender) = channels.get(name) {
            return ChannelSubscriber::new(sender.clone());
        }
        let (sender, _) = broadcast::channel(self.config.message_queue_size);
        channels.insert(name.to_string(), sender.clone());
        ChannelSubscriber::new(sender)
    }

    /// Subscribe connection to channel
    pub fn subscribe(&self, connection_id: &str, channel_name: &str) {
        {
            let mut conns = self.connections.write();
            if let Some(conn) = conns.get_mut(connection_id) {
                conn.subscribe(channel_name);
            }
        }
        self.get_or_create_channel(channel_name);
    }

    /// Unsubscribe connection from channel
    pub fn unsubscribe(&self, connection_id: &str, channel_name: &str) {
        let mut conns = self.connections.write();
        if let Some(conn) = conns.get_mut(connection_id) {
            conn.unsubscribe(channel_name);
        }
    }

    /// Broadcast to channel
    pub fn broadcast_to_channel(&self, channel: &str, message: WsMessage) -> Result<usize, WsError> {
        let channels = self.channels.read();
        if let Some(sender) = channels.get(channel) {
            sender.send(message).map_err(|_| WsError::ChannelClosed)
        } else {
            Ok(0)
        }
    }

    /// Send to specific user
    pub fn send_to_user(&self, user_id: &str, _message: WsMessage) -> usize {
        let connection_ids = self.get_user_connections(user_id);
        // In real implementation, send to each connection
        connection_ids.len()
    }

    /// Send to specific connection
    pub fn send_to_connection(&self, connection_id: &str, message: WsMessage) -> Result<(), WsError> {
        // In real implementation, would send to the actual WebSocket connection
        let _ = connection_id;
        let _ = message;
        Ok(())
    }

    /// Get connected users count
    pub fn connected_count(&self) -> usize {
        self.connections.read().len()
    }

    /// Get channel subscriber count
    pub fn channel_subscriber_count(&self, channel: &str) -> usize {
        self.channels.read().get(channel).map(|s| s.receiver_count()).unwrap_or(0)
    }
}

/// WebSocket errors
#[derive(Debug)]
pub enum WsError {
    ConnectionClosed,
    ChannelClosed,
    InvalidMessage,
    NotAuthenticated,
    RateLimited,
    Internal(String),
}

impl std::fmt::Display for WsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WsError::ConnectionClosed => write!(f, "Connection closed"),
            WsError::ChannelClosed => write!(f, "Channel closed"),
            WsError::InvalidMessage => write!(f, "Invalid message"),
            WsError::NotAuthenticated => write!(f, "Not authenticated"),
            WsError::RateLimited => write!(f, "Rate limited"),
            WsError::Internal(msg) => write!(f, "Internal: {}", msg),
        }
    }
}

impl std::error::Error for WsError {}

// ============================================
// Notification Types
// ============================================

/// Notification payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub id: String,
    pub title: String,
    pub body: String,
    pub icon: Option<String>,
    pub image: Option<String>,
    pub sound: Option<String>,
    pub data: Option<serde_json::Value>,
    pub timestamp: i64,
}

impl Notification {
    pub fn new(title: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title: title.into(),
            body: body.into(),
            icon: None,
            image: None,
            sound: None,
            data: None,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    pub fn with_icon(mut self, icon: impl Into<String>) -> Self { self.icon = Some(icon.into()); self }
    pub fn with_image(mut self, image: impl Into<String>) -> Self { self.image = Some(image.into()); self }
    pub fn with_sound(mut self, sound: impl Into<String>) -> Self { self.sound = Some(sound.into()); self }
    pub fn with_data(mut self, data: serde_json::Value) -> Self { self.data = Some(data); self }

    pub fn to_message(&self) -> WsMessage {
        WsMessage::json(serde_json::to_string(self).unwrap_or_default())
    }
}

/// Alert payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub level: AlertLevel,
    pub title: String,
    pub message: String,
    pub dismissible: bool,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlertLevel {
    Info,
    Success,
    Warning,
    Error,
}

impl Alert {
    pub fn info(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(AlertLevel::Info, title, message)
    }

    pub fn success(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(AlertLevel::Success, title, message)
    }

    pub fn warning(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(AlertLevel::Warning, title, message)
    }

    pub fn error(title: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(AlertLevel::Error, title, message)
    }

    pub fn new(level: AlertLevel, title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            level,
            title: title.into(),
            message: message.into(),
            dismissible: true,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    pub fn to_message(&self) -> WsMessage {
        WsMessage::json(serde_json::to_string(self).unwrap_or_default())
    }
}

/// Chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub sender_id: String,
    pub sender_name: Option<String>,
    pub recipient_id: Option<String>,
    pub channel_id: Option<String>,
    pub content: String,
    pub timestamp: i64,
    pub read: bool,
}

impl ChatMessage {
    pub fn new(sender_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            sender_id: sender_id.into(),
            sender_name: None,
            recipient_id: None,
            channel_id: None,
            content: content.into(),
            timestamp: chrono::Utc::now().timestamp(),
            read: false,
        }
    }

    pub fn to_user(mut self, recipient_id: impl Into<String>) -> Self { self.recipient_id = Some(recipient_id.into()); self }
    pub fn to_channel(mut self, channel_id: impl Into<String>) -> Self { self.channel_id = Some(channel_id.into()); self }
    pub fn from_name(mut self, name: impl Into<String>) -> Self { self.sender_name = Some(name.into()); self }

    pub fn to_message(&self) -> WsMessage {
        WsMessage::json(serde_json::to_string(self).unwrap_or_default())
    }
}

/// Live update (for real-time data sync)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveUpdate {
    pub entity_type: String,
    pub entity_id: String,
    pub action: UpdateAction,
    pub data: serde_json::Value,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UpdateAction {
    Create,
    Update,
    Delete,
}

impl LiveUpdate {
    pub fn created(entity_type: impl Into<String>, entity_id: impl Into<String>, data: serde_json::Value) -> Self {
        Self::new(entity_type, entity_id, UpdateAction::Create, data)
    }

    pub fn updated(entity_type: impl Into<String>, entity_id: impl Into<String>, data: serde_json::Value) -> Self {
        Self::new(entity_type, entity_id, UpdateAction::Update, data)
    }

    pub fn deleted(entity_type: impl Into<String>, entity_id: impl Into<String>) -> Self {
        Self::new(entity_type, entity_id, UpdateAction::Delete, serde_json::json!({}))
    }

    pub fn new(entity_type: impl Into<String>, entity_id: impl Into<String>, action: UpdateAction, data: serde_json::Value) -> Self {
        Self {
            entity_type: entity_type.into(),
            entity_id: entity_id.into(),
            action,
            data,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }

    pub fn to_message(&self) -> WsMessage {
        WsMessage::json(serde_json::to_string(self).unwrap_or_default())
    }
}

/// Presence (user online/offline status)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Presence {
    pub user_id: String,
    pub status: PresenceStatus,
    pub last_seen: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PresenceStatus {
    Online,
    Away,
    Busy,
    Offline,
}

impl Presence {
    pub fn online(user_id: impl Into<String>) -> Self {
        Self { user_id: user_id.into(), status: PresenceStatus::Online, last_seen: chrono::Utc::now().timestamp() }
    }

    pub fn offline(user_id: impl Into<String>) -> Self {
        Self { user_id: user_id.into(), status: PresenceStatus::Offline, last_seen: chrono::Utc::now().timestamp() }
    }

    pub fn to_message(&self) -> WsMessage {
        WsMessage::json(serde_json::to_string(self).unwrap_or_default())
    }
}

// ============================================
// WebSocket Service
// ============================================

/// WebSocket service for sending messages
pub struct WsService {
    hub: WsHub,
}

impl WsService {
    pub fn new(config: WsServerConfig) -> Self {
        Self { hub: WsHub::new(config) }
    }

    pub fn hub(&self) -> &WsHub {
        &self.hub
    }

    /// Send notification to user
    pub fn notify_user(&self, user_id: &str, notification: Notification) {
        let message = notification.to_message();
        self.hub.send_to_user(user_id, message);
    }

    /// Send alert to user
    pub fn alert_user(&self, user_id: &str, alert: Alert) {
        let message = alert.to_message();
        self.hub.send_to_user(user_id, message);
    }

    /// Send chat message
    pub fn send_chat(&self, recipient_id: &str, message: ChatMessage) {
        let ws_message = message.to_message();
        self.hub.send_to_user(recipient_id, ws_message);
    }

    /// Broadcast live update to channel
    pub fn broadcast_update(&self, channel: &str, update: LiveUpdate) {
        let message = update.to_message();
        let _ = self.hub.broadcast_to_channel(channel, message);
    }

    /// Update user presence
    pub fn update_presence(&self, channel: &str, presence: Presence) {
        let message = presence.to_message();
        let _ = self.hub.broadcast_to_channel(channel, message);
    }
}
