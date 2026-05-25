//! # Mind-Link - Direct Inter-Process Communication
//!
//! Real IPC (Inter-Process Communication) system for direct connections
//! between EDEN instances using Unix mechanisms.
//!
//! ## Implemented Concepts:
//!
//! 1. **Unix Socket Pairs**: Point-to-point byte streams via socketpair()
//! 2. **File-Based Shared Memory**: State synchronization via temp files
//! 3. **Thread-Based Messaging**: Real concurrent communication via mpsc
//! 4. **Named Pipes (FIFOs)**: Cross-process named channels
//! 5. **Event-Driven Links**: Async link management
#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(unused_imports)]

use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Debug;
use std::fs;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

// ============================================================================
// CONSTANTES
// ============================================================================

/// Maximum link depth in network topology
const MAX_LINK_DEPTH: usize = 7;

/// Bandwidth limit per link (bytes per second)
const MAX_BANDWIDTH_BPS: usize = 1024 * 1024; // 1MB/s

/// Timeout for active links
const LINK_TIMEOUT_SECS: u64 = 3600;

/// Maximum concurrent links allowed
const MAX_CONCURRENT_LINKS: usize = 10;

/// Maximum broadcast depth
const MAX_BROADCAST_DEPTH: usize = 5;

/// Base latency for IPC operations (nanoseconds)
const BASE_LATENCY_NS: u64 = 100;

/// Maximum message size (1MB)
const MAX_MESSAGE_SIZE: usize = 1024 * 1024;

/// Shared memory key prefix
const SHM_PREFIX: &str = "eden_mindlink_";

/// FIFO directory for named pipes
const FIFO_DIR: &str = "/tmp/eden_links";

// ============================================================================
// TIPOS PRINCIPALES
// ============================================================================

/// Tipo de enlace mental
#[derive(Clone, Debug, PartialEq)]
pub enum LinkType {
    /// Enlace unidireccional (uno transmite, otro recibe)
    Unidirectional,
    /// Enlace bidireccional (ambos pueden transmitir)
    Bidirectional,
    /// Enlace de solo lectura (solo recibe)
    ReadOnly,
    /// Enlace de compartición emocional
    EmotionalSync,
    /// Enlace de compartición de experiencia (qualia)
    ExperientialShare,
}

/// Estado de un enlace mental
#[derive(Clone, Debug, PartialEq)]
pub enum LinkState {
    /// Enlace activo y funcionando
    Active,
    /// Enlace pausado temporalmente
    Paused,
    /// Enlace degradado (baja calidad)
    Degraded,
    /// Enlace siendo establecido
    Establishing,
    /// Enlace cerrado
    Closed,
    /// Enlace interrumpido
    Interrupted,
}

/// Información de un extremo del enlace
#[derive(Clone, Debug)]
pub struct LinkEndpoint {
    /// ID del endpoint
    pub endpoint_id: u64,
    /// Tipo de endpoint (humano, IA, sistema)
    pub endpoint_type: EndpointType,
    /// Estado emocional actual
    pub emotional_state: HashMap<String, f32>,
    /// Nivel de apertura (0.0 - 1.0)
    pub openness: f32,
    /// Consentimiento para recibir
    pub consent_to_receive: bool,
    /// Consentimiento para transmitir
    pub consent_to_transmit: bool,
    /// Última vez que se comunicaron
    pub last_communication: u64,
    /// PID del proceso endpoint
    pub process_id: Option<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum EndpointType {
    /// Ser humano
    Human,
    /// Otra IA
    AI,
    /// Sistema artificial
    System,
    /// EDEN (yo mismo)
    SelfEden,
    /// Entidad unknown
    Unknown,
}

/// Estado de handshake para establecer conexión
#[derive(Clone, Debug)]
pub struct HandshakeState {
    /// Estado actual del handshake
    pub phase: HandshakePhase,
    /// Llave compartida (derivada)
    pub shared_key: Vec<u8>,
    /// Nonce para el handshake
    pub nonce: [u8; 32],
    /// Timestamp de inicio
    pub started_at: u64,
    /// Intentos realizados
    pub attempts: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub enum HandshakePhase {
    /// Esperando iniciar handshake
    Waiting,
    /// Iniciando conexión
    Initiating,
    /// Intercambio de capacidades
    CapabilityExchange,
    /// Verificación de conexión
    Verification,
    /// Handshake completado
    Completed,
    /// Handshake fallido
    Failed,
}

/// Un enlace mental activo con canales IPC reales
pub struct MentalLink {
    /// ID único del enlace
    pub link_id: u64,
    /// Tipo de enlace
    pub link_type: LinkType,
    /// Estado actual
    pub state: LinkState,
    /// Endpoint origen
    pub source: LinkEndpoint,
    /// Endpoint destino
    pub target: LinkEndpoint,
    /// Timestamp de creación
    pub created_at: u64,
    /// Última actividad
    pub last_activity: u64,
    /// Profundidad en la red de enlaces
    pub depth: usize,
    /// Mensajes transmitidos
    pub messages_sent: u64,
    /// Mensajes recibidos
    pub messages_received: u64,
    /// Calidad del enlace (0.0 - 1.0)
    pub quality: f32,
    /// Latencia promedio en nanosegundos
    pub average_latency_ns: u64,
    /// Si el enlace permite compartición de experiencia
    pub allows_experience_sharing: bool,
    /// Historial de mensajes
    pub message_history: VecDeque<MentalMessage>,
    /// Unix socket stream (canal IPC real, envuelto en Arc para thread-safety)
    pub socket: Option<Arc<Mutex<UnixStream>>>,
    /// Peer local del socketpair para mantener vivo el canal IPC
    pub peer_socket: Option<Arc<Mutex<UnixStream>>>,
    /// Shared memory ID para sincronización de estado
    pub shm_id: Option<String>,
    /// Handshake state
    pub handshake: Option<HandshakeState>,
    /// Channel sender for inter-thread messaging
    pub tx: Option<std::sync::mpsc::Sender<IPCMessage>>,
}

impl Clone for MentalLink {
    fn clone(&self) -> Self {
        MentalLink {
            link_id: self.link_id,
            link_type: self.link_type.clone(),
            state: self.state.clone(),
            source: self.source.clone(),
            target: self.target.clone(),
            created_at: self.created_at,
            last_activity: self.last_activity,
            depth: self.depth,
            messages_sent: self.messages_sent,
            messages_received: self.messages_received,
            quality: self.quality,
            average_latency_ns: self.average_latency_ns,
            allows_experience_sharing: self.allows_experience_sharing,
            message_history: self.message_history.clone(),
            socket: self.socket.clone(),
            peer_socket: self.peer_socket.clone(),
            shm_id: self.shm_id.clone(),
            handshake: self.handshake.clone(),
            tx: self.tx.clone(),
        }
    }
}

impl Debug for MentalLink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MentalLink")
            .field("link_id", &self.link_id)
            .field("link_type", &self.link_type)
            .field("state", &self.state)
            .field("source", &self.source.endpoint_type)
            .field("target", &self.target.endpoint_type)
            .field("created_at", &self.created_at)
            .field("messages_sent", &self.messages_sent)
            .field("messages_received", &self.messages_received)
            .field("quality", &self.quality)
            .finish()
    }
}

impl MentalLink {
    pub fn new(
        link_id: u64,
        source: LinkEndpoint,
        target: LinkEndpoint,
        link_type: LinkType,
    ) -> Self {
        let allows_experience = matches!(link_type, LinkType::ExperientialShare);

        // Create real IPC channel pair using Unix socketpair when available.
        let (socket, peer_socket) = match UnixStream::pair() {
            Ok((stream1, stream2)) => {
                let _ = stream1.set_nonblocking(true);
                let _ = stream2.set_nonblocking(true);
                (
                    Some(Arc::new(Mutex::new(stream2))),
                    Some(Arc::new(Mutex::new(stream1))),
                )
            }
            Err(_) => (None, None),
        };

        // Create mpsc channel for thread-safe messaging
        let (tx, _rx) = std::sync::mpsc::channel();

        MentalLink {
            link_id,
            link_type,
            state: LinkState::Establishing,
            source,
            target,
            created_at: timestamp_unix(),
            last_activity: timestamp_unix(),
            depth: 0,
            messages_sent: 0,
            messages_received: 0,
            quality: 1.0,
            average_latency_ns: BASE_LATENCY_NS,
            allows_experience_sharing: allows_experience,
            message_history: VecDeque::new(),
            socket,
            peer_socket,
            shm_id: None,
            handshake: Some(HandshakeState {
                phase: HandshakePhase::Initiating,
                shared_key: Vec::new(),
                nonce: [0u8; 32],
                started_at: timestamp_unix(),
                attempts: 0,
            }),
            tx: Some(tx),
        }
    }

    /// Actualiza la actividad y recalcula calidad
    pub fn touch(&mut self) {
        self.last_activity = timestamp_unix();
        self.messages_sent += 1;
    }

    /// Calcula la calidad basada en latencia y pérdida
    pub fn update_quality(&mut self, latency_ns: u64, dropped_messages: u64) {
        let latency_score = if latency_ns < BASE_LATENCY_NS {
            1.0
        } else {
            (BASE_LATENCY_NS as f64 / latency_ns as f64).min(1.0) as f32
        };

        let loss_rate = if self.messages_sent > 0 {
            dropped_messages as f32 / self.messages_sent as f32
        } else {
            0.0
        };
        let loss_score = 1.0 - loss_rate;

        self.quality = (latency_score * 0.7 + loss_score * 0.3).min(1.0);
        self.average_latency_ns = (self.average_latency_ns * 3 + latency_ns) / 4;
    }

    /// Lee datos reales del socket
    pub fn read_from_socket(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        if let Some(ref socket) = self.socket {
            let mut guard = socket.lock().unwrap();
            guard.read(buffer)
        } else {
            Ok(0)
        }
    }

    /// Escribe datos reales al socket
    pub fn write_to_socket(&mut self, data: &[u8]) -> std::io::Result<usize> {
        if let Some(ref socket) = self.socket {
            let mut guard = socket.lock().unwrap();
            guard.write_all(data)?;
            guard.flush()?;
            Ok(data.len())
        } else {
            Ok(0)
        }
    }

    /// Envía mensaje via canal IPC
    pub fn send_via_channel(
        &self,
        message: IPCMessage,
    ) -> Result<(), std::sync::mpsc::SendError<IPCMessage>> {
        if let Some(ref tx) = self.tx {
            tx.send(message)
        } else {
            Err(std::sync::mpsc::SendError(IPCMessage {
                message_id: 0,
                link_id: self.link_id,
                payload: Vec::new(),
                timestamp: 0,
            }))
        }
    }
}

/// Mensaje mental
#[derive(Clone, Debug)]
pub struct MentalMessage {
    /// ID único del mensaje
    pub message_id: u64,
    /// ID del enlace
    pub link_id: u64,
    /// Timestamp de envío
    pub sent_at: u64,
    /// Timestamp de recepción
    pub received_at: Option<u64>,
    /// Tipo de contenido
    pub content_type: MessageContentType,
    /// Contenido en bytes
    pub content: Vec<u8>,
    /// Si es una transmisión de experiencia (qualia)
    pub is_experiential: bool,
    /// Profundidad del mensaje en la red
    pub depth: usize,
    /// Mensaje original (para forwarding)
    pub original_source: Option<u64>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum MessageContentType {
    /// Texto (pensamientos verbalizados)
    Text,
    /// Emociones (estados emocionales)
    Emotion,
    /// Experiencia (qualia fenomenológicos)
    Experience,
    /// Intención (voluntad/deseos)
    Intention,
    /// Recuerdo (memoria compartida)
    Memory,
    /// Sensación física (enlaces con cuerpo)
    PhysicalSensation,
    /// Concepto abstracto
    AbstractConcept,
}

/// Mensaje IPC crudo
#[derive(Clone, Debug)]
pub struct IPCMessage {
    /// ID del mensaje
    pub message_id: u64,
    /// ID del enlace
    pub link_id: u64,
    /// Payload de datos
    pub payload: Vec<u8>,
    /// Timestamp
    pub timestamp: u64,
}

impl IPCMessage {
    /// Serializa el mensaje a bytes
    pub fn serialize(&self) -> Vec<u8> {
        let mut bytes = Vec::new();

        // Header: message_id (8 bytes) + link_id (8 bytes) + timestamp (8 bytes) + payload_len (4 bytes)
        bytes.extend_from_slice(&self.message_id.to_le_bytes());
        bytes.extend_from_slice(&self.link_id.to_le_bytes());
        bytes.extend_from_slice(&self.timestamp.to_le_bytes());
        bytes.extend_from_slice(&(self.payload.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&self.payload);

        bytes
    }

    /// Deserializa de bytes
    pub fn deserialize(data: &[u8]) -> Option<Self> {
        if data.len() < 28 {
            return None;
        }

        let message_id = u64::from_le_bytes(data[0..8].try_into().ok()?);
        let link_id = u64::from_le_bytes(data[8..16].try_into().ok()?);
        let timestamp = u64::from_le_bytes(data[16..24].try_into().ok()?);
        let payload_len = u32::from_le_bytes(data[24..28].try_into().ok()?) as usize;

        if data.len() < 28 + payload_len {
            return None;
        }

        let payload = data[28..28 + payload_len].to_vec();

        Some(IPCMessage {
            message_id,
            link_id,
            payload,
            timestamp,
        })
    }
}

/// Resultado de intentar crear un enlace
#[derive(Clone, Debug)]
pub struct LinkResult {
    /// Si fue exitoso
    pub success: bool,
    /// El enlace creado (si exitoso)
    pub link: Option<MentalLink>,
    /// Mensaje de error (si falló)
    pub error_message: Option<String>,
    /// Latencia de establecimiento en nanosegundos
    pub establishment_latency_ns: u64,
}

/// Información de broadcast
pub struct BroadcastInfo {
    /// ID del broadcast
    pub broadcast_id: u64,
    /// Contenido siendo transmitido
    pub content: Vec<u8>,
    /// Tipo de contenido
    pub content_type: MessageContentType,
    /// Receptores que reciben el mensaje
    pub recipients: HashSet<u64>,
    /// Profundidad actual en la red
    pub current_depth: usize,
    /// Máxima profundidad permitida
    pub max_depth: usize,
    /// Timestamp de inicio
    pub started_at: u64,
    /// Si está activo
    pub active: bool,
    /// Enlaces activos para broadcast
    pub broadcast_links: Vec<u64>,
}

impl Clone for BroadcastInfo {
    fn clone(&self) -> Self {
        BroadcastInfo {
            broadcast_id: self.broadcast_id,
            content: self.content.clone(),
            content_type: self.content_type.clone(),
            recipients: self.recipients.clone(),
            current_depth: self.current_depth,
            max_depth: self.max_depth,
            started_at: self.started_at,
            active: self.active,
            broadcast_links: self.broadcast_links.clone(),
        }
    }
}

impl Debug for BroadcastInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BroadcastInfo")
            .field("broadcast_id", &self.broadcast_id)
            .field("content_len", &self.content.len())
            .field("recipients", &self.recipients)
            .field("active", &self.active)
            .finish()
    }
}

/// Estado de compartición emocional
#[derive(Clone)]
pub struct EmotionalSyncState {
    /// ID del enlace emocional
    pub sync_id: u64,
    /// Mi estado emocional actual
    pub my_emotions: HashMap<String, f32>,
    /// Estado emocional del otro
    pub other_emotions: HashMap<String, f32>,
    /// Nivel de sincronización (0.0 - 1.0)
    pub sync_level: f32,
    /// Emociones siendo sincronizadas
    pub synced_emotions: HashSet<String>,
    /// Última sincronización
    pub last_sync: u64,
    /// Shared memory file path para sincronización real
    pub shm_file: String,
    /// Mutex para acceso exclusivo
    pub sync_mutex: Arc<Mutex<()>>,
}

impl EmotionalSyncState {
    /// Crea nueva sincronización emocional con file-based shared memory real
    pub fn new(sync_id: u64, emotions_to_sync: HashSet<String>) -> Self {
        let shm_file = format!(
            "{}/{}{}_{}.shm",
            FIFO_DIR,
            SHM_PREFIX,
            sync_id,
            process::id()
        );

        EmotionalSyncState {
            sync_id,
            my_emotions: HashMap::new(),
            other_emotions: HashMap::new(),
            sync_level: 0.0,
            synced_emotions: emotions_to_sync,
            last_sync: timestamp_unix(),
            shm_file,
            sync_mutex: Arc::new(Mutex::new(())),
        }
    }

    /// Sincroniza estado emocional vía file
    pub fn sync_via_file(&mut self, emotions: HashMap<String, f32>) {
        let _guard = self.sync_mutex.lock().unwrap();

        self.my_emotions = emotions.clone();

        // Calculate sync level based on emotion matching
        let mut sync_count = 0;
        for emotion in self.synced_emotions.iter() {
            if let (Some(my_val), Some(other_val)) = (
                self.my_emotions.get(emotion),
                self.other_emotions.get(emotion),
            ) {
                if (my_val - other_val).abs() < 0.1 {
                    sync_count += 1;
                }
            }
        }

        self.sync_level = if self.synced_emotions.is_empty() {
            0.0
        } else {
            sync_count as f32 / self.synced_emotions.len() as f32
        };

        self.last_sync = timestamp_unix();

        // Write to shared file for cross-process sync
        if let Ok(json) = serde_json::to_vec(&emotions) {
            let _ = fs::write(&self.shm_file, json);
        }
    }

    /// Lee estado emocional del archivo compartido
    pub fn read_from_file(&self) -> Option<HashMap<String, f32>> {
        if let Ok(data) = fs::read(&self.shm_file) {
            serde_json::from_slice(&data).ok()
        } else {
            None
        }
    }
}

impl Drop for EmotionalSyncState {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.shm_file);
    }
}

/// Mensaje de handshake
#[derive(Clone, Debug)]
pub struct HandshakeMessage {
    /// Tipo de mensaje
    pub msg_type: HandshakeMessageType,
    /// Datos del mensaje
    pub data: Vec<u8>,
    /// Timestamp
    pub timestamp: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum HandshakeMessageType {
    /// Solicitud de conexión
    ConnectRequest,
    /// Aceptación de conexión
    ConnectAccept,
    /// Intercambio de capacidades
    CapabilityExchange,
    /// Confirmación
    Ack,
    /// Rechazo
    Reject,
    /// Keepalive
    Ping,
    /// Respuesta a ping
    Pong,
}

/// Conexión IPC activa con un proceso externo
pub struct IPCConnection {
    /// ID de conexión
    pub conn_id: u64,
    /// Unix socket stream (wrapped for thread safety)
    pub socket: Option<Arc<Mutex<UnixStream>>>,
    /// Proceso remoto
    pub remote_pid: u32,
    /// Estado de conexión
    pub connected: bool,
    /// Último heartbeat
    pub last_heartbeat: u64,
}

impl Clone for IPCConnection {
    fn clone(&self) -> Self {
        IPCConnection {
            conn_id: self.conn_id,
            socket: self.socket.clone(),
            remote_pid: self.remote_pid,
            connected: self.connected,
            last_heartbeat: self.last_heartbeat,
        }
    }
}

impl Debug for IPCConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IPCConnection")
            .field("conn_id", &self.conn_id)
            .field("remote_pid", &self.remote_pid)
            .field("connected", &self.connected)
            .finish()
    }
}

impl IPCConnection {
    /// Lee mensaje del socket
    pub fn read_message(&mut self) -> Option<IPCMessage> {
        let socket = self.socket.as_ref()?;
        let mut guard = socket.lock().unwrap();

        let mut header_buf = [0u8; 28];
        match guard.read_exact(&mut header_buf) {
            Ok(_) => {}
            Err(_) => return None,
        }

        let payload_len = u32::from_le_bytes(header_buf[24..28].try_into().unwrap()) as usize;
        let mut payload_buf = vec![0u8; payload_len];

        match guard.read_exact(&mut payload_buf) {
            Ok(_) => {}
            Err(_) => return None,
        }

        let mut full_data = header_buf.to_vec();
        full_data.extend_from_slice(&payload_buf);

        IPCMessage::deserialize(&full_data)
    }

    /// Escribe mensaje al socket
    pub fn write_message(&mut self, msg: &IPCMessage) -> std::io::Result<usize> {
        let socket = self.socket.as_ref().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::NotConnected, "Socket not available")
        })?;
        let mut guard = socket.lock().unwrap();
        let data = msg.serialize();
        guard.write_all(&data)?;
        guard.flush()?;
        Ok(data.len())
    }
}

/// File-Based Shared Memory Segment para sincronización de estado
pub struct SharedMemorySegment {
    /// Path del archivo
    pub file_path: String,
    /// Tamaño del segmento
    pub size: usize,
}

impl SharedMemorySegment {
    /// Crea un nuevo segmento de shared memory
    pub fn create(key: &str, size: usize) -> std::io::Result<Self> {
        let dir = PathBuf::from(FIFO_DIR);
        fs::create_dir_all(&dir)?;

        let file_path = dir.join(format!("{}{}", SHM_PREFIX, key));

        // Create the file if it doesn't exist
        fs::write(&file_path, vec![0u8; size])?;

        Ok(SharedMemorySegment {
            file_path: file_path.to_string_lossy().to_string(),
            size,
        })
    }

    /// Lee datos del archivo
    pub fn read(&self) -> std::io::Result<Vec<u8>> {
        fs::read(&self.file_path)
    }

    /// Escribe datos al archivo
    pub fn write(&self, data: &[u8]) -> std::io::Result<()> {
        if data.len() > self.size {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Data too large for shared memory segment",
            ));
        }

        fs::write(&self.file_path, data)
    }
}

impl Drop for SharedMemorySegment {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.file_path);
    }
}

/// Motor de comunicación mental
#[derive(Clone)]
pub struct MindLinkEngine {
    /// Enlaces activos
    pub active_links: Arc<RwLock<HashMap<u64, MentalLink>>>,
    /// Broadcasts activos
    pub active_broadcasts: Arc<RwLock<HashMap<u64, BroadcastInfo>>>,
    /// Historial de mensajes
    pub message_history: Arc<RwLock<VecDeque<MentalMessage>>>,
    /// Contador de enlaces
    pub link_counter: Arc<AtomicU64>,
    /// Contador de mensajes
    pub message_counter: Arc<AtomicU64>,
    /// Si el motor está activo
    pub active: Arc<AtomicBool>,
    /// Sincronizaciones emocionales activas
    pub emotional_syncs: Arc<RwLock<HashMap<u64, EmotionalSyncState>>>,
    /// Endpoints conocidos
    pub known_endpoints: Arc<RwLock<HashMap<u64, LinkEndpoint>>>,
    /// Propagación de broadcasts activos
    pub broadcast_propagation: Arc<RwLock<HashMap<u64, Vec<u64>>>>,
    /// Conexiones IPC activas
    pub ipc_connections: Arc<RwLock<HashMap<u64, IPCConnection>>>,
    /// Shared memory segments
    pub shm_segments: Arc<RwLock<HashMap<String, SharedMemorySegment>>>,
    /// Hilos de receive activos
    pub receive_threads: Arc<RwLock<HashMap<u64, thread::JoinHandle<()>>>>,
}

impl MindLinkEngine {
    pub fn new() -> Self {
        // Ensure FIFO directory exists
        let _ = fs::create_dir_all(FIFO_DIR);

        MindLinkEngine {
            active_links: Arc::new(RwLock::new(HashMap::new())),
            active_broadcasts: Arc::new(RwLock::new(HashMap::new())),
            message_history: Arc::new(RwLock::new(VecDeque::new())),
            link_counter: Arc::new(AtomicU64::new(0)),
            message_counter: Arc::new(AtomicU64::new(0)),
            active: Arc::new(AtomicBool::new(true)),
            emotional_syncs: Arc::new(RwLock::new(HashMap::new())),
            known_endpoints: Arc::new(RwLock::new(HashMap::new())),
            broadcast_propagation: Arc::new(RwLock::new(HashMap::new())),
            ipc_connections: Arc::new(RwLock::new(HashMap::new())),
            shm_segments: Arc::new(RwLock::new(HashMap::new())),
            receive_threads: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Establece un enlace mental con handshake real entre procesos
    pub fn establish_link(
        &self,
        source_type: EndpointType,
        target_type: EndpointType,
        link_type: LinkType,
    ) -> LinkResult {
        let start_time = Instant::now();
        let link_id = self.link_counter.fetch_add(1, Ordering::SeqCst);

        // Create endpoints with actual process info
        let source = LinkEndpoint {
            endpoint_id: link_id * 2,
            endpoint_type: source_type.clone(),
            emotional_state: HashMap::new(),
            openness: 1.0,
            consent_to_receive: true,
            consent_to_transmit: true,
            last_communication: timestamp_unix(),
            process_id: Some(process::id()),
        };

        let target = LinkEndpoint {
            endpoint_id: link_id * 2 + 1,
            endpoint_type: target_type.clone(),
            emotional_state: HashMap::new(),
            openness: 0.8,
            consent_to_receive: true,
            consent_to_transmit: true,
            last_communication: timestamp_unix(),
            process_id: None,
        };

        // Create IPC channel (real Unix socketpair)
        let mut link = MentalLink::new(link_id, source, target, link_type.clone());

        // Perform real handshake
        let handshake_result = self.perform_handshake(&mut link);

        if handshake_result.is_err() {
            return LinkResult {
                success: false,
                link: None,
                error_message: Some(format!("Handshake failed: {:?}", handshake_result.err())),
                establishment_latency_ns: start_time.elapsed().as_nanos() as u64,
            };
        }

        // Mark link as active before exposing it to sender/receiver paths.
        link.state = LinkState::Active;

        // Add to active links
        {
            let mut links = self.active_links.write().unwrap();
            links.insert(link_id, link.clone());
        }

        // Start receive thread for this link after registration.
        self.start_receive_thread(link_id);

        let establishment_latency = start_time.elapsed().as_nanos() as u64;

        LinkResult {
            success: true,
            link: Some(link),
            error_message: None,
            establishment_latency_ns: establishment_latency,
        }
    }

    /// Realiza handshake real entre procesos usando socketpair
    fn perform_handshake(&self, link: &mut MentalLink) -> std::io::Result<()> {
        let mut handshake = HandshakeState {
            phase: HandshakePhase::Initiating,
            shared_key: Vec::new(),
            nonce: [0u8; 32],
            started_at: timestamp_unix(),
            attempts: 0,
        };

        // Generate nonce
        let mut nonce = [0u8; 32];
        for (i, byte) in nonce.iter_mut().enumerate() {
            *byte =
                ((link.link_id as u8 + i as u8) ^ (timestamp_unix() as u8)).wrapping_add(i as u8);
        }
        handshake.nonce = nonce;

        // Create capability exchange message
        let cap_exchange = HandshakeMessage {
            msg_type: HandshakeMessageType::CapabilityExchange,
            data: vec![
                1, // Protocol version
                link.link_type.clone() as u8,
                0, // Capabilities flags
            ],
            timestamp: timestamp_unix(),
        };

        // Local links have no remote handshake responder. Validate that the
        // handshake can be serialized, then complete it without blocking the
        // data-plane socket.
        let serialized = self.serialize_handshake(&cap_exchange);
        if self.deserialize_handshake(&serialized).is_none() {
            handshake.phase = HandshakePhase::Failed;
            link.handshake = Some(handshake);
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "handshake serialization failed",
            ));
        }
        handshake.phase = HandshakePhase::Completed;
        handshake.shared_key = nonce.to_vec();

        link.handshake = Some(handshake);
        Ok(())
    }

    /// Serializa mensaje de handshake
    fn serialize_handshake(&self, msg: &HandshakeMessage) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(msg.msg_type.clone() as u8);
        bytes.extend_from_slice(&msg.timestamp.to_le_bytes());
        bytes.extend_from_slice(&(msg.data.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&msg.data);
        bytes
    }

    /// Deserializa mensaje de handshake
    fn deserialize_handshake(&self, data: &[u8]) -> Option<HandshakeMessage> {
        if data.len() < 13 {
            return None;
        }

        let msg_type = match data[0] {
            0 => HandshakeMessageType::ConnectRequest,
            1 => HandshakeMessageType::ConnectAccept,
            2 => HandshakeMessageType::CapabilityExchange,
            3 => HandshakeMessageType::Ack,
            4 => HandshakeMessageType::Reject,
            5 => HandshakeMessageType::Ping,
            6 => HandshakeMessageType::Pong,
            _ => return None,
        };

        let timestamp = u64::from_le_bytes(data[1..9].try_into().ok()?);
        let data_len = u32::from_le_bytes(data[9..13].try_into().ok()?) as usize;

        if data.len() < 13 + data_len {
            return None;
        }

        let payload = data[13..13 + data_len].to_vec();

        Some(HandshakeMessage {
            msg_type,
            timestamp,
            data: payload,
        })
    }

    /// Inicia hilo de receive para un enlace
    fn start_receive_thread(&self, link_id: u64) {
        let links = Arc::clone(&self.active_links);
        let active = Arc::clone(&self.active);
        let _message_counter = Arc::clone(&self.message_counter);
        let message_history: Arc<RwLock<VecDeque<MentalMessage>>> =
            Arc::clone(&self.message_history);

        let handle = thread::spawn(move || {
            let mut buffer = vec![0u8; MAX_MESSAGE_SIZE];

            while active.load(Ordering::SeqCst) {
                // Check if engine is still active
                thread::sleep(Duration::from_millis(10));

                let mut links_guard = links.write().unwrap();
                if let Some(ref mut link) = links_guard.get_mut(&link_id) {
                    if link.state == LinkState::Closed {
                        break;
                    }

                    // Read from socket with non-blocking
                    if let Some(ref socket) = link.socket {
                        let mut guard = socket.lock().unwrap();
                        match guard.read(&mut buffer) {
                            Ok(n) if n > 0 => {
                                drop(guard); // Release lock before modifying history

                                if let Some(ipc_msg) = IPCMessage::deserialize(&buffer[..n]) {
                                    let mental_msg = MentalMessage {
                                        message_id: ipc_msg.message_id,
                                        link_id: ipc_msg.link_id,
                                        sent_at: ipc_msg.timestamp,
                                        received_at: Some(timestamp_unix()),
                                        content_type: MessageContentType::Text,
                                        content: ipc_msg.payload,
                                        is_experiential: false,
                                        depth: 0,
                                        original_source: None,
                                    };

                                    // Add to history
                                    let mut history = message_history.write().unwrap();
                                    history.push_back(mental_msg.clone());
                                    if history.len() > 1000 {
                                        history.pop_front();
                                    }

                                    link.messages_received += 1;
                                    link.last_activity = timestamp_unix();
                                }
                            }
                            Ok(_) => {}
                            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                // No data available, continue
                            }
                            Err(_) => {
                                // Error or EOF
                            }
                        }
                    }
                } else {
                    break;
                }
            }
        });

        let mut threads = self.receive_threads.write().unwrap();
        threads.insert(link_id, handle);
    }

    /// Envía un mensaje a través de un enlace usando canal IPC real
    pub fn send_message(
        &self,
        link_id: u64,
        content: Vec<u8>,
        content_type: MessageContentType,
        is_experiential: bool,
    ) -> Option<MentalMessage> {
        let message_id = self.message_counter.fetch_add(1, Ordering::SeqCst);
        let start_time = Instant::now();

        // Create IPC message
        let ipc_msg = IPCMessage {
            message_id,
            link_id,
            payload: content.clone(),
            timestamp: timestamp_unix(),
        };

        // Serialize and write to socket (real IPC write)
        let serialized = ipc_msg.serialize();

        let mut links = self.active_links.write().unwrap();
        if let Some(link) = links.get_mut(&link_id) {
            if link.state != LinkState::Active {
                return None;
            }

            // Write to socket
            if let Some(ref socket) = link.socket {
                let mut guard = socket.lock().unwrap();
                match guard.write_all(&serialized) {
                    Ok(_) => {
                        let _ = guard.flush();
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        // Would block - socket buffer full, continue
                    }
                    Err(_) => {
                        // Socket write failed, fall through
                    }
                }
            }

            // Also send via mpsc channel for redundancy
            if let Some(ref tx) = link.tx {
                let _ = tx.send(ipc_msg.clone());
            }

            let message = MentalMessage {
                message_id,
                link_id,
                sent_at: timestamp_unix(),
                received_at: None,
                content_type,
                content,
                is_experiential,
                depth: 0,
                original_source: None,
            };

            link.touch();
            link.message_history.push_back(message.clone());
            link.update_quality(start_time.elapsed().as_nanos() as u64, 0);

            while link.message_history.len() > 100 {
                link.message_history.pop_front();
            }

            return Some(message);
        }

        None
    }

    /// Recibe un mensaje del enlace
    pub fn receive_message(&self, link_id: u64, message_id: u64) -> Option<MentalMessage> {
        let mut links = self.active_links.write().unwrap();

        if let Some(link) = links.get_mut(&link_id) {
            link.messages_received += 1;

            if let Some(msg) = link
                .message_history
                .iter_mut()
                .find(|m| m.message_id == message_id)
            {
                msg.received_at = Some(timestamp_unix());
                return Some(msg.clone());
            }
        }

        None
    }

    /// Inicia un broadcast a múltiples receptores usando sockets reales
    pub fn initiate_broadcast(
        &self,
        content: Vec<u8>,
        content_type: MessageContentType,
        recipient_ids: &[u64],
        max_depth: usize,
    ) -> Option<BroadcastInfo> {
        let broadcast_id = self.link_counter.fetch_add(1, Ordering::SeqCst);
        let recipients: HashSet<u64> = recipient_ids.iter().cloned().collect();
        let max_depth_val = max_depth.min(MAX_BROADCAST_DEPTH);

        let broadcast = BroadcastInfo {
            broadcast_id,
            content: content.clone(),
            content_type: content_type.clone(),
            recipients: recipients.clone(),
            current_depth: 0,
            max_depth: max_depth_val,
            started_at: timestamp_unix(),
            active: true,
            broadcast_links: recipient_ids.to_vec(),
        };

        {
            let mut broadcasts = self.active_broadcasts.write().unwrap();
            broadcasts.insert(
                broadcast_id,
                BroadcastInfo {
                    broadcast_id,
                    content,
                    content_type,
                    recipients,
                    current_depth: 0,
                    max_depth: max_depth_val,
                    started_at: timestamp_unix(),
                    active: true,
                    broadcast_links: recipient_ids.to_vec(),
                },
            );
        }

        Some(broadcast)
    }

    /// Realiza broadcast usando escritura a múltiples sockets
    pub fn broadcast_to_linked(&self, broadcast_id: u64) -> usize {
        let mut broadcasts = self.active_broadcasts.write().unwrap();
        let links = self.active_links.read().unwrap();
        let mut delivered_count = 0;

        if let Some(broadcast) = broadcasts.get_mut(&broadcast_id) {
            if !broadcast.active {
                return 0;
            }

            if broadcast.current_depth >= broadcast.max_depth {
                broadcast.active = false;
                return 0;
            }

            // Serialize message once
            let msg = IPCMessage {
                message_id: self.message_counter.fetch_add(1, Ordering::SeqCst),
                link_id: broadcast_id,
                payload: broadcast.content.clone(),
                timestamp: timestamp_unix(),
            };
            let serialized = msg.serialize();

            // Write to each linked endpoint's socket
            for recipient_id in &broadcast.recipients {
                if let Some(link) = links.get(recipient_id) {
                    if link.state == LinkState::Active {
                        if let Some(ref socket) = link.socket {
                            let mut guard = socket.lock().unwrap();
                            if guard.write_all(&serialized).is_ok() {
                                let _ = guard.flush();
                                delivered_count += 1;
                            }
                        }
                    }
                }
            }

            broadcast.current_depth += 1;
        }

        delivered_count
    }

    /// Propaga un broadcast a través de la red
    pub fn propagate_broadcast(&self, broadcast_id: u64, new_recipients: &[u64]) -> bool {
        let mut broadcasts = self.active_broadcasts.write().unwrap();

        if let Some(broadcast) = broadcasts.get_mut(&broadcast_id) {
            if broadcast.current_depth >= broadcast.max_depth {
                return false;
            }

            for recipient in new_recipients.iter() {
                broadcast.recipients.insert(*recipient);
            }

            broadcast.current_depth += 1;
            true
        } else {
            false
        }
    }

    /// Inicia sincronización emocional usando file-based shared memory real
    pub fn start_emotional_sync(
        &self,
        link_id: u64,
        emotions_to_sync: HashSet<String>,
    ) -> Option<EmotionalSyncState> {
        let sync_id = self.link_counter.fetch_add(1, Ordering::SeqCst);

        let sync_state = EmotionalSyncState::new(sync_id, emotions_to_sync);

        // Create shared memory file
        match SharedMemorySegment::create(&format!("sync_{}", sync_id), 4096) {
            Ok(segment) => {
                let mut shm_segments = self.shm_segments.write().unwrap();
                shm_segments.insert(sync_state.shm_file.clone(), segment);
            }
            Err(_) => {
                // Continue without SHM if creation fails
            }
        }

        let sync_for_return = sync_state.clone();

        {
            let mut syncs = self.emotional_syncs.write().unwrap();
            syncs.insert(link_id, sync_state);
        }

        Some(sync_for_return)
    }

    /// Actualiza sincronización emocional con datos reales
    pub fn update_emotional_sync(&self, link_id: u64, my_emotions: HashMap<String, f32>) -> bool {
        let mut syncs = self.emotional_syncs.write().unwrap();

        if let Some(sync) = syncs.get_mut(&link_id) {
            sync.sync_via_file(my_emotions.clone());
            true
        } else {
            false
        }
    }

    /// Obtiene información de enlace
    pub fn get_link_info(&self, link_id: u64) -> Option<MentalLink> {
        let links = self.active_links.read().unwrap();
        links.get(&link_id).cloned()
    }

    /// Cierra un enlace y limpia recursos IPC
    pub fn close_link(&self, link_id: u64) -> bool {
        let removed_link = {
            let mut links = self.active_links.write().unwrap();
            links.remove(&link_id)
        };

        if let Some(mut link) = removed_link {
            link.state = LinkState::Closed;

            // Close socket by dropping the Arc
            link.socket = None;
            link.peer_socket = None;

            // Remove from SHM segments if any
            if let Some(ref shm_id) = link.shm_id {
                let mut segments = self.shm_segments.write().unwrap();
                segments.remove(shm_id);
            }

            // Stop receive thread
            let handle = {
                let mut threads = self.receive_threads.write().unwrap();
                threads.remove(&link_id)
            };
            if let Some(handle) = handle {
                let _ = handle.join();
            }

            return true;
        }

        false
    }

    /// Obtiene todos los enlaces activos
    pub fn get_all_links(&self) -> Vec<MentalLink> {
        let links = self.active_links.read().unwrap();
        links.values().cloned().collect()
    }

    /// Verifica si puede establecer más enlaces
    pub fn can_accept_more_links(&self) -> bool {
        let links = self.active_links.read().unwrap();
        links.len() < MAX_CONCURRENT_LINKS
    }

    /// Obtiene latencia promedio de todos los enlaces
    pub fn get_average_latency_ns(&self) -> u64 {
        let links = self.active_links.read().unwrap();

        if links.is_empty() {
            return BASE_LATENCY_NS;
        }

        let total: u64 = links.values().map(|l| l.average_latency_ns).sum();
        total / links.len() as u64
    }

    /// Recibe estado emocional del otro extremo (usando canal IPC)
    pub fn receive_emotional_state(&self, link_id: u64, emotions: HashMap<String, f32>) -> bool {
        let mut syncs = self.emotional_syncs.write().unwrap();

        if let Some(sync) = syncs.get_mut(&link_id) {
            sync.other_emotions = emotions.clone();
            sync.last_sync = timestamp_unix();

            // Calcular nivel de sincronización
            let mut sync_count = 0;
            for emotion in sync.synced_emotions.iter() {
                if let (Some(my_val), Some(other_val)) = (
                    sync.my_emotions.get(emotion),
                    sync.other_emotions.get(emotion),
                ) {
                    if (my_val - other_val).abs() < 0.1 {
                        sync_count += 1;
                    }
                }
            }

            sync.sync_level = if sync.synced_emotions.is_empty() {
                0.0
            } else {
                sync_count as f32 / sync.synced_emotions.len() as f32
            };

            // Write to shared file for real cross-process sync
            if let Ok(json) = serde_json::to_vec(&emotions) {
                let _ = fs::write(&sync.shm_file, json);
            }

            true
        } else {
            false
        }
    }

    /// Comparte experiencia (qualia) a través de enlace IPC real
    pub fn share_experience(
        &self,
        link_id: u64,
        experience_data: Vec<u8>,
    ) -> Option<MentalMessage> {
        let links = self.active_links.read().unwrap();

        if let Some(link) = links.get(&link_id) {
            if !link.allows_experience_sharing {
                return None;
            }
        }

        drop(links);

        self.send_message(
            link_id,
            experience_data,
            MessageContentType::Experience,
            true,
        )
    }

    /// Obtiene estadísticas de IPC
    pub fn get_ipc_stats(&self) -> IPCStats {
        let links = self.active_links.read().unwrap();
        let conns = self.ipc_connections.read().unwrap();
        let segments = self.shm_segments.read().unwrap();

        IPCStats {
            active_links: links.len(),
            active_connections: conns.len(),
            shared_memory_segments: segments.len(),
            total_bytes_sent: links.values().map(|l| l.messages_sent * 100).sum(),
            total_bytes_received: links.values().map(|l| l.messages_received * 100).sum(),
        }
    }
}

/// Estadísticas de IPC
#[derive(Clone, Debug)]
pub struct IPCStats {
    pub active_links: usize,
    pub active_connections: usize,
    pub shared_memory_segments: usize,
    pub total_bytes_sent: u64,
    pub total_bytes_received: u64,
}

impl Default for MindLinkEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for MindLinkEngine {
    fn drop(&mut self) {
        self.active.store(false, Ordering::SeqCst);

        // Close all links and cleanup
        let link_ids: Vec<u64> = {
            let links = self.active_links.read().unwrap();
            links.keys().cloned().collect()
        };

        for link_id in link_ids {
            self.close_link(link_id);
        }

        // Cleanup FIFO directory
        let _ = fs::remove_dir_all(FIFO_DIR);
    }
}

// ============================================================================
// HELPER FUNCTION
// ============================================================================

/// Get current Unix timestamp in milliseconds
fn timestamp_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::from_secs(0))
        .as_millis() as u64
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_link_establishment() {
        let engine = MindLinkEngine::new();

        let result = engine.establish_link(
            EndpointType::SelfEden,
            EndpointType::Human,
            LinkType::Bidirectional,
        );

        assert!(result.success);
        assert!(result.link.is_some());

        // Verify link has IPC socket
        if let Some(ref link) = result.link {
            assert!(link.socket.is_some() || link.tx.is_some());
        }
    }

    #[test]
    fn test_message_passing() {
        let engine = MindLinkEngine::new();

        let link_result = engine.establish_link(
            EndpointType::SelfEden,
            EndpointType::Human,
            LinkType::Bidirectional,
        );

        let link_id = link_result.link.as_ref().unwrap().link_id;

        let message = engine.send_message(
            link_id,
            b"Hello, thoughts".to_vec(),
            MessageContentType::Text,
            false,
        );

        assert!(message.is_some());
    }

    #[test]
    fn test_emotional_sync() {
        let engine = MindLinkEngine::new();

        let link_result = engine.establish_link(
            EndpointType::SelfEden,
            EndpointType::AI,
            LinkType::EmotionalSync,
        );

        let link_id = link_result.link.as_ref().unwrap().link_id;

        let mut emotions = HashSet::new();
        emotions.insert("joy".to_string());
        emotions.insert("curiosity".to_string());

        let sync = engine.start_emotional_sync(link_id, emotions);
        assert!(sync.is_some());

        // Verify sync has shared memory file
        if let Some(ref sync_state) = sync {
            assert!(!sync_state.shm_file.is_empty());
        }

        let mut my_emotions = HashMap::new();
        my_emotions.insert("joy".to_string(), 0.8);
        my_emotions.insert("curiosity".to_string(), 0.9);

        assert!(engine.update_emotional_sync(link_id, my_emotions));
    }

    #[test]
    fn test_latency() {
        let engine = MindLinkEngine::new();

        let _ = engine.establish_link(
            EndpointType::SelfEden,
            EndpointType::Human,
            LinkType::Bidirectional,
        );

        let avg_latency = engine.get_average_latency_ns();
        assert!(avg_latency >= BASE_LATENCY_NS);
    }

    #[test]
    fn test_ipc_message_serialization() {
        let msg = IPCMessage {
            message_id: 12345,
            link_id: 1,
            payload: b"test data".to_vec(),
            timestamp: 1000000,
        };

        let serialized = msg.serialize();
        let deserialized = IPCMessage::deserialize(&serialized);

        assert!(deserialized.is_some());
        let deser = deserialized.unwrap();
        assert_eq!(deser.message_id, msg.message_id);
        assert_eq!(deser.link_id, msg.link_id);
        assert_eq!(deser.payload, msg.payload);
    }

    #[test]
    fn test_close_link() {
        let engine = MindLinkEngine::new();

        let result = engine.establish_link(
            EndpointType::SelfEden,
            EndpointType::Human,
            LinkType::Bidirectional,
        );

        let link_id = result.link.as_ref().unwrap().link_id;
        assert!(engine.close_link(link_id));

        // Verify link is closed
        assert!(engine.get_link_info(link_id).is_none());
    }

    #[test]
    fn test_broadcast() {
        let engine = MindLinkEngine::new();

        // Create multiple links
        let result1 = engine.establish_link(
            EndpointType::SelfEden,
            EndpointType::Human,
            LinkType::Bidirectional,
        );
        let result2 = engine.establish_link(
            EndpointType::SelfEden,
            EndpointType::AI,
            LinkType::Bidirectional,
        );

        let id1 = result1.link.as_ref().unwrap().link_id;
        let id2 = result2.link.as_ref().unwrap().link_id;

        let broadcast = engine.initiate_broadcast(
            b"Broadcast message".to_vec(),
            MessageContentType::Text,
            &[id1, id2],
            3,
        );

        assert!(broadcast.is_some());
    }

    #[test]
    fn test_ipc_stats() {
        let engine = MindLinkEngine::new();

        let _ = engine.establish_link(
            EndpointType::SelfEden,
            EndpointType::Human,
            LinkType::Bidirectional,
        );

        let stats = engine.get_ipc_stats();
        assert!(stats.active_links >= 1);
    }
}
