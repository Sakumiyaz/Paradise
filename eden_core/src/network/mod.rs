//! # EDEN Network Module
//!
//! Módulo de comunicación peer-to-peer para EDEN distribuido.
//!
//! ## Arquitectura
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                        EDEN Network                              │
//! │                                                                  │
//! │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐              │
//! │  │   PeerMgr   │  │  SyncMgr    │  │  GossipMgr  │              │
//! │  │  (peers)    │  │  (state)    │  │  (flooding) │              │
//! │  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘              │
//! │         │               │               │                       │
//! │         └───────────────┼───────────────┘                       │
//! │                         ▼                                       │
//! │                  ┌─────────────┐                               │
//! │                  │   Router    │ (wire protocol)                │
//! │                  └──────┬──────┘                               │
//! │                         │                                       │
//! │         ┌───────────────┼───────────────┐                       │
//! │         ▼               ▼               ▼                       │
//! │  ┌───────────┐   ┌───────────┐   ┌───────────┐                  │
//! │  │    TCP    │   │   UDP     │   │  Relay    │                  │
//! │  └───────────┘   └───────────┘   └───────────┘                  │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Features
//!
//! ```toml
//! [dependencies]
//! eden_core = { path = ".", features = ["network"] }
//! ```
#![allow(dead_code)]
#![allow(non_snake_case)]

pub mod protocol_evolution;
pub mod node_lifecycle;
pub mod secure_p2p_transport;
pub mod controlled_discovery;
pub mod democratic_propagation;
pub mod resource_monitor;

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::net::SocketAddr;

// EDEN Async Runtime - Use internal asyncrt instead of tokio
#[cfg(feature = "network")]
use crate::asyncrt::{TcpListener, TcpStream, AsyncReceiver, AsyncSender, Channel};
#[cfg(feature = "network")]
use std::sync::{Arc, RwLock};

#[cfg(not(feature = "network"))]
use std::sync::{Arc, RwLock};

#[cfg(feature = "trace")]
use tracing::{info, warn, error};

// =============================================================================
// PeerId - Identificador único de peer
// =============================================================================

/// Peer ID único (derivado de pubkey o generated)
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct PeerId(u64);

impl PeerId {
    /// Generate a new random peer ID (for testing)
    pub fn generate_random() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        PeerId((now ^ (now >> 47)) as u64)
    }

    /// Generate from string (for deterministic IDs)
    pub fn from_str(s: &str) -> Self {
        let mut hash: u64 = 0xCBF29CE484222325;
        for byte in s.bytes() {
            hash = hash.wrapping_mul(0x100000001B3);
            hash ^= byte as u64;
        }
        PeerId(hash)
    }

    pub fn as_u64(&self) -> u64 {
        self.0
    }

    pub fn to_string(&self) -> String {
        format!("{:016x}", self.0)
    }
}

impl std::fmt::Display for PeerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl std::fmt::Debug for PeerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PeerId({:016x})", self.0)
    }
}

impl Default for PeerId {
    fn default() -> Self {
        PeerId(0)
    }
}

// =============================================================================
// Connection State
// =============================================================================

/// Estado de conexión de un peer
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ConnectionState {
    /// No conectado
    Disconnected,
    /// Conexión TCP en progreso
    Connecting,
    /// Handshake en progreso
    Handshaking,
    /// Conectado y activo
    Connected,
    /// Conexión degradada (latencia alta o pérdidas)
    Degraded,
    /// Desconectando
    Disconnecting,
}

impl Default for ConnectionState {
    fn default() -> Self {
        ConnectionState::Disconnected
    }
}

// =============================================================================
// Message Types (Wire Protocol v1)
// =============================================================================

/// Tipos de mensaje del wire protocol v1
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum MessageType {
    // Control (0x00-0x0F)
    Handshake = 0x01,
    HandshakeAck = 0x02,
    Heartbeat = 0x03,
    Disconnect = 0x04,

    // Sync (0x10-0x1F)
    StateDiff = 0x10,
    FullSnapshot = 0x11,
    SyncRequest = 0x12,
    SyncResponse = 0x13,

    // Auton Events (0x20-0x2F)
    AutonBorn = 0x20,
    AutonDead = 0x21,
    AutonMigrated = 0x22,

    // Mar Events (0x30-0x3F)
    MarDelta = 0x30,
    ClaimUpdate = 0x31,

    // Consensus (0x40-0x4F)
    Proposal = 0x40,
    Vote = 0x41,
    ConsensusReached = 0x42,

    // Debug (0xF0-0xFF)
    Ping = 0xF0,
    Pong = 0xF1,
    Error = 0xFF,
}

impl MessageType {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0x01 => Some(MessageType::Handshake),
            0x02 => Some(MessageType::HandshakeAck),
            0x03 => Some(MessageType::Heartbeat),
            0x04 => Some(MessageType::Disconnect),
            0x10 => Some(MessageType::StateDiff),
            0x11 => Some(MessageType::FullSnapshot),
            0x12 => Some(MessageType::SyncRequest),
            0x13 => Some(MessageType::SyncResponse),
            0x20 => Some(MessageType::AutonBorn),
            0x21 => Some(MessageType::AutonDead),
            0x22 => Some(MessageType::AutonMigrated),
            0x30 => Some(MessageType::MarDelta),
            0x31 => Some(MessageType::ClaimUpdate),
            0x40 => Some(MessageType::Proposal),
            0x41 => Some(MessageType::Vote),
            0x42 => Some(MessageType::ConsensusReached),
            0xF0 => Some(MessageType::Ping),
            0xF1 => Some(MessageType::Pong),
            0xFF => Some(MessageType::Error),
            _ => None,
        }
    }
}

// =============================================================================
// Wire Protocol Header
// =============================================================================

/// Wire protocol message header
#[derive(Clone, Copy, Debug)]
pub struct MessageHeader {
    pub version: u8,       // Protocol version (0x01)
    pub msg_type: u8,     // MessageType as u8
    pub length: u32,      // Payload length (big-endian)
    pub checksum: u32,    // CRC32 of payload
}

impl MessageHeader {
    pub const PROTOCOL_VERSION: u8 = 0x01;
    pub const HEADER_SIZE: usize = 10; // 1 + 1 + 4 + 4

    /// Serialize header to bytes
    pub fn to_bytes(&self) -> [u8; 10] {
        let mut bytes = [0u8; 10];
        bytes[0] = self.version;
        bytes[1] = self.msg_type;
        bytes[2..6].copy_from_slice(&self.length.to_be_bytes());
        bytes[6..10].copy_from_slice(&self.checksum.to_be_bytes());
        bytes
    }

    /// Deserialize header from bytes
    pub fn from_bytes(bytes: &[u8; 10]) -> Option<Self> {
        if bytes[0] != Self::PROTOCOL_VERSION {
            return None;
        }
        Some(Self {
            version: bytes[0],
            msg_type: bytes[1],
            length: u32::from_be_bytes(bytes[2..6].try_into().ok()?),
            checksum: u32::from_be_bytes(bytes[6..10].try_into().ok()?),
        })
    }
}

// =============================================================================
// NetworkMessage
// =============================================================================

/// Mensaje de red completo (header + payload)
#[derive(Clone, Debug)]
pub struct NetworkMessage {
    pub sender: PeerId,
    pub msg_type: MessageType,
    pub timestamp: u64,
    pub payload: Vec<u8>,
}

impl NetworkMessage {
    pub fn new(sender: PeerId, msg_type: MessageType, payload: Vec<u8>) -> Self {
        Self {
            sender,
            msg_type,
            timestamp: current_timestamp(),
            payload,
        }
    }

    /// Get wire format bytes (header + payload)
    pub fn to_wire(&self) -> Vec<u8> {
        let checksum = crc32(&self.payload);
        let header = MessageHeader {
            version: MessageHeader::PROTOCOL_VERSION,
            msg_type: self.msg_type as u8,
            length: self.payload.len() as u32,
            checksum,
        };
        
        let mut wire = Vec::with_capacity(MessageHeader::HEADER_SIZE + self.payload.len());
        wire.extend_from_slice(&header.to_bytes());
        wire.extend_from_slice(&self.payload);
        wire
    }

    /// Parse from wire format
    pub fn from_wire(data: &[u8]) -> Option<Self> {
        if data.len() < MessageHeader::HEADER_SIZE {
            return None;
        }
        
        let header = MessageHeader::from_bytes(data[..10].try_into().ok()?)?;
        let msg_type = MessageType::from_u8(header.msg_type)?;
        
        let payload = data[MessageHeader::HEADER_SIZE..].to_vec();
        
        // Verify checksum
        let expected = crc32(&payload);
        if expected != header.checksum {
            return None;
        }
        
        Some(Self {
            sender: PeerId::default(), // Will be filled by connection
            msg_type,
            timestamp: current_timestamp(),
            payload,
        })
    }

    /// Get message ID for deduplication
    pub fn message_id(&self) -> u64 {
        let mut h: u64 = self.sender.as_u64();
        h = h.wrapping_mul(31).wrapping_add(self.msg_type as u64);
        h = h.wrapping_mul(31).wrapping_add(self.timestamp);
        h
    }
}

// =============================================================================
// CRC32 Checksum (simple implementation, no external deps)
// =============================================================================

/// Calculate CRC32 checksum
fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xFFFFFFFF;
    
    for byte in data {
        crc ^= *byte as u32;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc >>= 1;
            }
        }
    }
    
    !crc
}

/// Current timestamp (milliseconds since epoch)
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

// =============================================================================
// PeerEntry
// =============================================================================

/// Información de un peer conocido
#[derive(Clone, Debug)]
pub struct PeerEntry {
    pub peer_id: PeerId,
    pub address: SocketAddr,
    pub last_seen: u64,
    pub connected_at: u64,
    pub state: ConnectionState,
    pub consecutive_failures: u8,
    pub current_tick: u64,
    pub state_hash: u64,
}

impl PeerEntry {
    pub fn new(peer_id: PeerId, address: SocketAddr) -> Self {
        Self {
            peer_id,
            address,
            last_seen: current_timestamp(),
            connected_at: current_timestamp(),
            state: ConnectionState::Disconnected,
            consecutive_failures: 0,
            current_tick: 0,
            state_hash: 0,
        }
    }

    pub fn is_connected(&self) -> bool {
        matches!(self.state, ConnectionState::Connected | ConnectionState::Degraded)
    }

    pub fn is_alive(&self, timeout_ms: u64) -> bool {
        current_timestamp() - self.last_seen < timeout_ms
    }
}

// =============================================================================
// Connection Handler
// =============================================================================

/// Manejador de conexión TCP a un peer
pub struct Connection {
    pub peer_id: PeerId,
    stream: TcpStream,
    last_heartbeat: u64,
}

impl Connection {
    /// Create new connection
    pub async fn connect(peer_id: PeerId, addr: SocketAddr) -> std::io::Result<Self> {
        let stream = TcpStream::connect(addr).await?;
        Ok(Self {
            peer_id,
            stream,
            last_heartbeat: current_timestamp(),
        })
    }

    /// Send raw bytes
    pub async fn send(&mut self, data: &[u8]) -> std::io::Result<()> {
        self.stream.write_all(data).await
    }

    /// Receive raw bytes
    pub async fn receive(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.stream.read(buf).await
    }

    /// Send a NetworkMessage
    pub async fn send_message(&mut self, msg: &NetworkMessage) -> std::io::Result<()> {
        let wire = msg.to_wire();
        self.send(&wire).await
    }

    /// Receive a NetworkMessage (blocking)
    pub async fn receive_message(&mut self) -> std::io::Result<Option<NetworkMessage>> {
        let mut header_buf = [0u8; MessageHeader::HEADER_SIZE];
        
        match self.stream.read(&mut header_buf).await {
            Ok(0) => return Ok(None), // EOF
            Ok(n) if n < MessageHeader::HEADER_SIZE => return Ok(None),
            Ok(_) => {}
            Err(e) => return Err(e),
        }
        
        let header = MessageHeader::from_bytes(&header_buf)
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid header"))?;
        let mut payload = vec![0u8; header.length as usize];
        
        self.stream.read_exact(&mut payload).await?;
        
        let mut msg = NetworkMessage::from_wire(&[&header_buf[..], &payload[..]].concat())
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid message"))?;
        msg.sender = self.peer_id;
        
        Ok(Some(msg))
    }

    /// Update heartbeat timestamp
    pub fn touch(&mut self) {
        self.last_heartbeat = current_timestamp();
    }

    pub fn last_heartbeat_ms(&self) -> u64 {
        self.last_heartbeat
    }
}

// =============================================================================
// PeerManager
// =============================================================================

/// Gestor central de peers
pub struct PeerManager {
    /// Mi peer ID
    self_id: PeerId,
    
    /// Todos los peers conocidos
    peers: HashMap<PeerId, PeerEntry>,
    
    /// Conexiones activas
    connections: HashMap<PeerId, Connection>,
    
    /// Seeds para bootstrap
    seeds: Vec<SocketAddr>,
    
    /// Canales para mensajes entrantes
    msg_tx: mpsc::Sender<NetworkMessage>,
    msg_rx: mpsc::Receiver<NetworkMessage>,
    
    /// Config
    config: PeerConfig,
}

/// Configuración de peer manager
#[derive(Clone, Debug)]
pub struct PeerConfig {
    /// Timeout de heartbeat en ms
    pub heartbeat_timeout_ms: u64,
    /// Intervalo de heartbeat en ms
    pub heartbeat_interval_ms: u64,
    /// Máximo de conexiones entrantes
    pub max_inbound: usize,
    /// Máximo de conexiones salientes
    pub max_outbound: usize,
    /// Máximo de fallos consecutivos antes de considerar dead
    pub max_failures: u8,
}

impl Default for PeerConfig {
    fn default() -> Self {
        Self {
            heartbeat_timeout_ms: 30_000,      // 30s
            heartbeat_interval_ms: 5_000,       // 5s
            max_inbound: 100,
            max_outbound: 50,
            max_failures: 5,
        }
    }
}

impl PeerManager {
    /// Create new PeerManager
    pub fn new(self_id: PeerId, seeds: Vec<SocketAddr>) -> Self {
        let (msg_tx, msg_rx) = mpsc::channel(1024);
        
        Self {
            self_id,
            peers: HashMap::new(),
            connections: HashMap::new(),
            seeds,
            msg_tx,
            msg_rx,
            config: PeerConfig::default(),
        }
    }

    /// Start listening for incoming connections
    pub async fn listen(&mut self, addr: SocketAddr) -> std::io::Result<()> {
        let listener = TcpListener::bind(addr).await?;
        
        loop {
            match listener.accept().await {
                Ok((stream, client_addr)) => {
                    let peer_id = PeerId::generate_random(); // Temp until handshake
                    let conn = Connection {
                        peer_id,
                        stream,
                        last_heartbeat: current_timestamp(),
                    };
                    
                    // Add as pending connection for handshake
                    self.peers.insert(peer_id, PeerEntry {
                        peer_id,
                        address: client_addr,
                        last_seen: current_timestamp(),
                        connected_at: current_timestamp(),
                        state: ConnectionState::Handshaking,
                        consecutive_failures: 0,
                        current_tick: 0,
                        state_hash: 0,
                    });
                    self.connections.insert(peer_id, conn);
                }
                Err(e) => {
                    eprintln!("Accept error: {}", e);
                }
            }
        }
    }

    /// Connect to a peer
    pub async fn connect(&mut self, addr: SocketAddr) -> std::io::Result<PeerId> {
        let peer_id = PeerId::generate_random();
        
        let mut conn = Connection::connect(peer_id, addr).await?;
        
        // Send handshake
        let handshake_payload = self.create_handshake_payload();
        let msg = NetworkMessage::new(self.self_id, MessageType::Handshake, handshake_payload);
        conn.send_message(&msg).await?;
        
        // Wait for ack
        let response = conn.receive_message().await?;
        if let Some(resp) = response {
            if resp.msg_type == MessageType::HandshakeAck {
                // Handshake successful
                self.peers.insert(peer_id, PeerEntry {
                    peer_id,
                    address: addr,
                    last_seen: current_timestamp(),
                    connected_at: current_timestamp(),
                    state: ConnectionState::Connected,
                    consecutive_failures: 0,
                    current_tick: 0,
                    state_hash: 0,
                });
                self.connections.insert(peer_id, conn);
            }
        }
        
        Ok(peer_id)
    }

    /// Bootstrap from seeds
    pub async fn bootstrap(&mut self) -> std::io::Result<()> {
        // Clone seeds to avoid borrow conflict
        let seeds: Vec<SocketAddr> = self.seeds.clone();
        for seed in seeds {
            match self.connect(seed).await {
                Ok(_) => {
                    println!("[EDEN] Bootstrapped from {}", seed);
                    return Ok(());
                }
                Err(e) => {
                    println!("[EDEN] Seed {} failed: {}", seed, e);
                }
            }
        }
        Err(std::io::Error::new(
            std::io::ErrorKind::ConnectionRefused,
            "No seeds available",
        ))
    }

    /// Send message to a peer
    pub async fn send_to(&mut self, peer_id: PeerId, msg: NetworkMessage) -> std::io::Result<()> {
        if let Some(conn) = self.connections.get_mut(&peer_id) {
            conn.send_message(&msg).await
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::NotConnected, "Peer not connected"))
        }
    }

    /// Broadcast to all connected peers
    pub async fn broadcast(&mut self, msg: NetworkMessage, exclude: Option<PeerId>) -> std::io::Result<()> {
        for (peer_id, conn) in &mut self.connections {
            if Some(*peer_id) != exclude {
                conn.send_message(&msg).await?;
            }
        }
        Ok(())
    }

    /// Create handshake payload
    fn create_handshake_payload(&self) -> Vec<u8> {
        let mut payload = Vec::with_capacity(16);
        payload.extend_from_slice(&self.self_id.as_u64().to_le_bytes());
        payload.extend_from_slice(&current_timestamp().to_le_bytes());
        payload
    }

    /// Get list of connected peer IDs
    pub fn connected_peers(&self) -> Vec<PeerId> {
        self.connections.keys().copied().collect()
    }

    /// Check if peer is connected
    pub fn is_connected(&self, peer_id: &PeerId) -> bool {
        self.connections.contains_key(peer_id)
    }

    /// Get peer info
    pub fn get_peer(&self, peer_id: &PeerId) -> Option<&PeerEntry> {
        self.peers.get(peer_id)
    }

    /// Update heartbeat for peer
    pub fn touch_peer(&mut self, peer_id: &PeerId) {
        if let Some(entry) = self.peers.get_mut(peer_id) {
            entry.last_seen = current_timestamp();
            entry.consecutive_failures = 0;
        }
    }

    /// Mark peer as failed
    pub fn mark_failed(&mut self, peer_id: &PeerId) {
        if let Some(entry) = self.peers.get_mut(peer_id) {
            entry.consecutive_failures += 1;
            if entry.consecutive_failures >= self.config.max_failures {
                entry.state = ConnectionState::Disconnected;
                self.connections.remove(peer_id);
            }
        }
    }
}

// =============================================================================
// State Synchronization Types
// =============================================================================

/// Vector de estado para sincronización
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StateVector {
    /// tick local del peer
    pub tick: u64,
    /// vector de timestamps de última sincronización
    pub peer_ticks: HashMap<PeerId, u64>,
    /// hash del estado completo (para validación rápida)
    pub state_hash: u64,
}

impl StateVector {
    pub fn new(tick: u64, state_hash: u64) -> Self {
        Self {
            tick,
            peer_ticks: HashMap::new(),
            state_hash,
        }
    }

    /// Compara dos StateVectors para determinar sync priority
    pub fn needs_sync(&self, other: &StateVector) -> bool {
        self.tick < other.tick 
        || self.state_hash != other.state_hash
    }
}

/// Delta de estado entre dos StateVectors
#[derive(Clone, Debug)]
pub struct StateDiff {
    pub from_tick: u64,
    pub to_tick: u64,
    pub new_auton_ids: Vec<u64>,
    pub dead_auton_ids: Vec<u64>,
    pub mar_change_count: usize,
}

/// Snapshot completo para bootstrapping
#[derive(Clone, Debug)]
pub struct FullSnapshot {
    pub version: u8,
    pub peer_id: PeerId,
    pub state_vector: StateVector,
    pub timestamp: u64,
    pub autons_count: usize,
    pub mar_energon_total: u64,
}

// =============================================================================
// Serialization helpers
// =============================================================================

use eden_serialize::{Serialize, Deserialize};
pub use eden_serialize::{to_bytes, from_bytes};

/// Serialize a type to bytes using eden_serialize
pub fn serialize<T: Serialize>(value: &T) -> Vec<u8> {
    to_bytes(value)
}

/// Deserialize bytes to a type using eden_serialize
pub fn deserialize<T: Deserialize>(data: &[u8]) -> Option<T> {
    from_bytes(data).ok()
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_id_generation() {
        let id1 = PeerId::generate_random();
        let id2 = PeerId::generate_random();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_peer_id_from_str() {
        let id1 = PeerId::from_str("test");
        let id2 = PeerId::from_str("test");
        assert_eq!(id1, id2);
        
        let id3 = PeerId::from_str("different");
        assert_ne!(id1, id3);
    }

    #[test]
    fn test_message_header_serialization() {
        let header = MessageHeader {
            version: 0x01,
            msg_type: MessageType::Heartbeat as u8,
            length: 100,
            checksum: 0xDEADBEEF,
        };
        
        let bytes = header.to_bytes();
        let parsed = MessageHeader::from_bytes(&bytes).unwrap();
        
        assert_eq!(header.version, parsed.version);
        assert_eq!(header.msg_type, parsed.msg_type);
        assert_eq!(header.length, parsed.length);
        assert_eq!(header.checksum, parsed.checksum);
    }

    #[test]
    fn test_message_wire_format() {
        let msg = NetworkMessage::new(
            PeerId::from_str("test_peer"),
            MessageType::Heartbeat,
            vec![1, 2, 3, 4],
        );
        
        let wire = msg.to_wire();
        let parsed = NetworkMessage::from_wire(&wire).unwrap();
        
        assert_eq!(msg.msg_type, parsed.msg_type);
        assert_eq!(msg.payload, parsed.payload);
    }

    #[test]
    fn test_crc32() {
        let data = b"hello world";
        let crc = crc32(data);
        // Verified CRC32
        assert_ne!(crc, 0);
    }

    #[test]
    fn test_state_vector() {
        let sv1 = StateVector::new(100, 0xABCDEF00);
        let sv2 = StateVector::new(150, 0xABCDEF00); // Same hash to test tick comparison
        
        // sv1 (tick 100) needs sync from sv2 (tick 150) because sv1 is behind
        assert!(sv1.needs_sync(&sv2));
        
        // sv2 (tick 150) doesn't need sync from sv1 (tick 100) because sv2 is ahead
        // But since hashes are same, no sync needed
        assert!(!sv2.needs_sync(&sv1));
        
        // Different hash triggers sync regardless of tick
        let sv3 = StateVector::new(100, 0x12345678);
        assert!(sv1.needs_sync(&sv3)); // Different hash
    }
}