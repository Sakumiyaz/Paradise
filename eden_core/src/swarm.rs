//! # EDEN Swarm - Gossip Protocol
//!
//! Sistema de enjambre P2P usando Gossip Protocol.
//! Sin dependencias externas - 100% Rust stdlib.
//!
//! ## Gossip Protocol:
//!
//! - Cada nodo conoce algunos peers
//! - Periódicamente comparte su estado con peers elegidos al azar
//! - El estado se propaga exponencialmente a todos los nodos
//! - Resistente a caída de nodos - no hay servidor central
//!
//! ## Mensajes:
//!
//! - GOSSIP_HAVE: "Conozco estos peers"
//! - GENES_HAVE: "Tengo estos genes"
//! - REQUEST_PEERS: "Dame tu lista de peers"
//! - REQUEST_GENES: "Dame tus genes"

#![allow(dead_code)]

use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::hash::BuildHasher;
use std::io::Write;
use std::net::{SocketAddr, TcpListener, TcpStream, UdpSocket};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

// ============================================================================
// Constantes
// ============================================================================

/// Puerto UDP para discovery (broadcast local)
const DISCOVERY_PORT: u16 = 4850;
/// Timeout para peers
const PEER_TIMEOUT_SECS: u64 = 300;
/// Número de peers a contactar por ciclo
const GOSSIP_FANOUT: usize = 3;
/// Máximo peers conocidos
const MAX_KNOWN_PEERS: usize = 100;
/// Máximo genes conocidos
const MAX_KNOWN_GENES: usize = 1000;

// ============================================================================
// Estado global
// ============================================================================

static GOSSIP_STATE: std::sync::OnceLock<Arc<Mutex<GossipState>>> = std::sync::OnceLock::new();

fn get_state() -> &'static Arc<Mutex<GossipState>> {
    GOSSIP_STATE.get_or_init(|| Arc::new(Mutex::new(GossipState::new())))
}

// ============================================================================
// Tipos
// ============================================================================

/// Peer ID único (basado en timestamp + hash)
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct PeerId(u64);

impl PeerId {
    pub fn new(id: u64) -> Self {
        PeerId(id)
    }
    pub fn as_u64(&self) -> u64 {
        self.0
    }
    pub fn from_u64(v: u64) -> Self {
        PeerId(v)
    }
}

impl std::fmt::Display for PeerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:016x}", self.0)
    }
}

/// Información de un peer conocido
#[derive(Clone, Debug)]
pub struct PeerInfo {
    pub id: PeerId,
    pub addr: SocketAddr,
    pub last_gossip: Instant,
    pub connected: bool,
}

/// Genes de un Auton
#[derive(Clone, Debug)]
pub struct AutonGene {
    pub ramnet_seed: u64,
    pub campo_seed: u64,
    pub generacion: u32,
    pub pos_x: f32,
    pub pos_y: f32,
    pub fitness: f32,
}

impl AutonGene {
    /// Serializa a bytes (32 bytes)
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut v = Vec::with_capacity(32);
        v.extend_from_slice(&self.ramnet_seed.to_le_bytes());
        v.extend_from_slice(&self.campo_seed.to_le_bytes());
        v.extend_from_slice(&self.generacion.to_le_bytes());
        v.extend_from_slice(&self.pos_x.to_le_bytes());
        v.extend_from_slice(&self.pos_y.to_le_bytes());
        v.extend_from_slice(&self.fitness.to_le_bytes());
        v
    }

    /// Deserializa desde bytes
    pub fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 32 {
            return None;
        }
        Some(Self {
            ramnet_seed: u64::from_le_bytes(data[0..8].try_into().ok()?),
            campo_seed: u64::from_le_bytes(data[8..16].try_into().ok()?),
            generacion: u32::from_le_bytes(data[16..20].try_into().ok()?),
            pos_x: f32::from_le_bytes(data[20..24].try_into().ok()?),
            pos_y: f32::from_le_bytes(data[24..28].try_into().ok()?),
            fitness: f32::from_le_bytes(data[28..32].try_into().ok()?),
        })
    }
}

/// Tipo de mensaje Gossip
#[derive(Clone, Copy, Debug)]
#[repr(u8)]
enum GossipMsgType {
    GossipHave = 0,
    GenesHave = 1,
    RequestPeers = 2,
    RequestGenes = 3,
    Ping = 4,
    Pong = 5,
}

/// Header de mensaje Gossip (19 bytes)
#[derive(Clone, Debug)]
struct GossipHeader {
    msg_type: GossipMsgType,
    sender_id: PeerId,
    seq: u64,
    payload_len: u16,
}

impl GossipHeader {
    fn to_bytes(&self) -> [u8; 19] {
        let mut b = [0u8; 19];
        b[0] = self.msg_type as u8;
        b[1..9].copy_from_slice(&self.sender_id.as_u64().to_le_bytes());
        b[9..17].copy_from_slice(&self.seq.to_le_bytes());
        b[17..19].copy_from_slice(&self.payload_len.to_le_bytes());
        b
    }

    fn from_bytes(b: &[u8; 19]) -> Option<Self> {
        let msg_type = match b[0] {
            0 => GossipMsgType::GossipHave,
            1 => GossipMsgType::GenesHave,
            2 => GossipMsgType::RequestPeers,
            3 => GossipMsgType::RequestGenes,
            4 => GossipMsgType::Ping,
            5 => GossipMsgType::Pong,
            _ => return None,
        };
        Some(GossipHeader {
            msg_type,
            sender_id: PeerId::new(u64::from_le_bytes(b[1..9].try_into().ok()?)),
            seq: u64::from_le_bytes(b[9..17].try_into().ok()?),
            payload_len: u16::from_le_bytes(b[17..19].try_into().ok()?),
        })
    }
}

/// Estado completo del Gossip
#[derive(Clone, Debug)]
struct GossipState {
    /// Nuestro PeerId
    self_id: PeerId,
    /// peers conocidos: PeerId -> PeerInfo
    known_peers: HashMap<PeerId, PeerInfo>,
    /// genes conocidos: ramnet_seed -> AutonGene
    known_genes: HashMap<u64, AutonGene>,
    /// historial de mensajes vistos (para anti-loops)
    message_history: HashMap<(PeerId, u64), Instant>,
    /// secuencia de mensajes
    msg_seq: u64,
    /// último gossip enviado
    last_gossip: Instant,
    /// Puerto local
    local_port: u16,
    /// Genes pendientes de inyección
    inbound_genes: Vec<AutonGene>,
}

impl GossipState {
    fn new() -> Self {
        GossipState {
            self_id: generate_peer_id(),
            known_peers: HashMap::new(),
            known_genes: HashMap::new(),
            message_history: HashMap::new(),
            msg_seq: 0,
            last_gossip: Instant::now(),
            local_port: 0,
            inbound_genes: Vec::new(),
        }
    }

    fn next_seq(&mut self) -> u64 {
        self.msg_seq = self.msg_seq.wrapping_add(1);
        self.msg_seq
    }

    /// Añade un peer conocido
    fn add_peer(&mut self, id: PeerId, addr: SocketAddr) {
        if id == self.self_id {
            return;
        }

        // Check if we have room before acquiring mutable reference via entry()
        if self.known_peers.len() >= MAX_KNOWN_PEERS && !self.known_peers.contains_key(&id) {
            return;
        }

        match self.known_peers.entry(id) {
            Entry::Occupied(e) => {
                e.into_mut().last_gossip = Instant::now();
            }
            Entry::Vacant(e) => {
                e.insert(PeerInfo {
                    id,
                    addr,
                    last_gossip: Instant::now(),
                    connected: false,
                });
            }
        }
    }

    /// Añade un gene conocido
    fn add_gene(&mut self, gene: AutonGene) {
        let key = gene.ramnet_seed;

        // Skip if at capacity and not already present
        if self.known_genes.len() >= MAX_KNOWN_GENES && !self.known_genes.contains_key(&key) {
            return;
        }

        if let Entry::Vacant(e) = self.known_genes.entry(key) {
            e.insert(gene);
        }
    }

    /// Obtiene peers random para gossip
    fn get_random_peers(&self, count: usize) -> Vec<PeerId> {
        use std::collections::hash_map::RandomState;
        let _rng = RandomState::new();
        let keys: Vec<_> = self.known_peers.keys().cloned().collect();
        let mut sample = Vec::new();
        for k in keys {
            sample.push(k);
            if sample.len() >= count {
                break;
            }
        }
        sample
    }

    /// Limpia peers muertos y mensajes viejos
    fn cleanup(&mut self) {
        let now = Instant::now();
        self.known_peers
            .retain(|_, p| now.duration_since(p.last_gossip).as_secs() < PEER_TIMEOUT_SECS);
        self.message_history
            .retain(|_, t| now.duration_since(*t).as_secs() < 3600);
    }
}

// ============================================================================
// Funciones públicas
// ============================================================================

/// Inicializa el enjambre Gossip
pub fn init(port: u16) -> std::io::Result<()> {
    let mut state = get_state().lock().unwrap();
    state.local_port = port;

    println!("[GOSSIP] Peer {} listening on port {}", state.self_id, port);

    // Hilo UDP para discovery broadcast
    let state_ref = get_state().clone();
    thread::spawn(move || {
        if let Ok(socket) = UdpSocket::bind(format!("0.0.0.0:{}", DISCOVERY_PORT)) {
            socket.set_broadcast(true).ok();
            socket.set_nonblocking(true).ok();

            let mut buf = [0u8; 256];
            loop {
                if let Ok((len, addr)) = socket.recv_from(&mut buf) {
                    if let Some(peer_id) = parse_discovery_message(&buf[..len]) {
                        let mut s = state_ref.lock().unwrap();
                        s.add_peer(peer_id, addr);
                    }
                }
                thread::sleep(Duration::from_millis(100));
            }
        }
    });

    // Hilo TCP para gossip
    let state_ref = get_state().clone();
    thread::spawn(move || {
        let addr = format!("0.0.0.0:{}", port);
        if let Ok(listener) = TcpListener::bind(&addr) {
            listener.set_nonblocking(true).ok();
            loop {
                match listener.accept() {
                    Ok((mut stream, addr)) => {
                        let s = state_ref.clone();
                        thread::spawn(move || {
                            handle_tcp_connection(&mut stream, addr, &s);
                        });
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(_) => {}
                }
            }
        }
    });

    // Broadcast periódico de nuestra existencia
    let state_ref = get_state().clone();
    thread::spawn(move || {
        if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
            loop {
                {
                    let state = state_ref.lock().unwrap();
                    let msg = encode_discovery_message(state.self_id, state.local_port);
                    let _ = socket.send_to(&msg, format!("255.255.255.255:{}", DISCOVERY_PORT));
                }
                thread::sleep(Duration::from_secs(10));
            }
        }
    });

    Ok(())
}

/// Conecta a un peer específico
pub fn connect(addr: &str) -> std::io::Result<PeerId> {
    let stream = TcpStream::connect(addr)?;
    let peer_addr = stream.peer_addr()?;
    let peer_id = PeerId::from_socketaddr(&peer_addr);

    let mut state = get_state().lock().unwrap();
    state.add_peer(peer_id, peer_addr);
    println!("[GOSSIP] Connected to peer {} at {}", peer_id, addr);

    // Intercambiar peers inmediatamente
    let peers_msg = build_gossip_have(&mut state);
    drop(state);
    if let Ok(mut stream) = TcpStream::connect(addr) {
        let _ = stream.write_all(&peers_msg);
    }

    Ok(peer_id)
}

/// Comparte genes con todos los peers
pub fn share_genes(genes: &[AutonGene]) {
    let mut state = get_state().lock().unwrap();
    let msg = build_genes_have(&mut state, genes);
    drop(state);

    let peer_ids = get_state().lock().unwrap().get_random_peers(10);
    for pid in peer_ids {
        let addr = {
            let state = get_state().lock().unwrap();
            state.known_peers.get(&pid).map(|p| p.addr)
        };
        if let Some(addr) = addr {
            if let Ok(mut stream) = TcpStream::connect_timeout(&addr, Duration::from_millis(500)) {
                let _ = stream.write_all(&msg);
            }
        }
    }
}

/// Gossip con peers (llamar cada ciclo)
pub fn gossip_cycle() {
    let mut state = get_state().lock().unwrap();
    let now = Instant::now();

    // Cleanup periódico
    state.cleanup();

    // No hacer gossip muy seguido (cada 5 segundos máx)
    if now.duration_since(state.last_gossip).as_secs() < 5 {
        return;
    }
    state.last_gossip = now;

    // Seleccionar peers random
    let targets: Vec<_> = state.get_random_peers(GOSSIP_FANOUT);
    let peers_msg = build_gossip_have(&mut state);
    let genes_msg = build_genes_have(&mut state, &[]);

    drop(state);

    for addr in targets.iter().filter_map(|pid| {
        get_state()
            .lock()
            .unwrap()
            .known_peers
            .get(pid)
            .map(|p| p.addr)
    }) {
        if let Ok(mut stream) = TcpStream::connect_timeout(&addr, Duration::from_millis(500)) {
            let _ = stream.write_all(&peers_msg);
            let _ = stream.write_all(&genes_msg);
        }
    }
}

/// Recibe genes recibidos (para inyección)
pub fn receive_genes() -> Vec<AutonGene> {
    let mut state = get_state().lock().unwrap();
    let genes: Vec<_> = state.inbound_genes.drain(..).collect();
    genes
}

/// Número de peers conocidos
pub fn peer_count() -> usize {
    get_state().lock().unwrap().known_peers.len()
}

/// Verifica si hay peers activos
pub fn has_peers() -> bool {
    peer_count() > 0
}

/// Obtiene nuestro PeerId
pub fn our_peer_id() -> PeerId {
    get_state().lock().unwrap().self_id
}

// ============================================================================
// Helpers internos
// ============================================================================

fn generate_peer_id() -> PeerId {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    PeerId((now ^ (now >> 47)) as u64)
}

impl PeerId {
    fn from_socketaddr(addr: &SocketAddr) -> Self {
        let mut hash: u64 = 0xCBF29CE484222325;
        hash = hash.wrapping_mul(0x100000001B3);
        hash ^= addr.port() as u64;
        hash ^= addr
            .ip()
            .to_string()
            .as_bytes()
            .iter()
            .fold(0u64, |acc, &b| acc.wrapping_mul(0x100000001B3) ^ b as u64);
        PeerId(hash)
    }
}

/// Broadcast UDP: peer_id (8 bytes) + port (2 bytes)
fn encode_discovery_message(peer_id: PeerId, port: u16) -> Vec<u8> {
    let mut msg = Vec::with_capacity(10);
    msg.extend_from_slice(&peer_id.as_u64().to_le_bytes());
    msg.extend_from_slice(&port.to_le_bytes());
    msg
}

fn parse_discovery_message(buf: &[u8]) -> Option<PeerId> {
    if buf.len() < 10 {
        return None;
    }
    let id = u64::from_le_bytes(buf[0..8].try_into().ok()?);
    Some(PeerId::new(id))
}

/// Construye mensaje GOSSIP_HAVE con nuestra lista de peers
fn build_gossip_have(state: &mut GossipState) -> Vec<u8> {
    let mut msg = Vec::new();

    // Header
    msg.push(GossipMsgType::GossipHave as u8);
    msg.extend_from_slice(&state.self_id.as_u64().to_le_bytes());
    msg.extend_from_slice(&state.next_seq().to_le_bytes());

    // Payload: peers conocidos (max 20)
    let peers: Vec<_> = state
        .known_peers
        .iter()
        .take(20)
        .map(|(_, p)| p.clone())
        .collect();
    msg.extend_from_slice(&(peers.len() as u8).to_le_bytes());

    for peer in &peers {
        msg.extend_from_slice(&peer.id.as_u64().to_le_bytes());
        // IP + port (4 + 2 = 6 bytes)
        let ip = match peer.addr.ip() {
            std::net::IpAddr::V4(ip) => ip.octets(),
            std::net::IpAddr::V6(_) => [127, 0, 0, 1],
        };
        msg.extend_from_slice(&ip);
        msg.extend_from_slice(&peer.addr.port().to_le_bytes());
    }

    msg
}

/// Construye mensaje GENES_HAVE con nuestros genes
fn build_genes_have(state: &mut GossipState, genes: &[AutonGene]) -> Vec<u8> {
    let mut msg = Vec::new();

    msg.push(GossipMsgType::GenesHave as u8);
    msg.extend_from_slice(&state.self_id.as_u64().to_le_bytes());
    msg.extend_from_slice(&state.next_seq().to_le_bytes());

    // Genes a compartir (max 10)
    let share = if genes.is_empty() {
        state
            .known_genes
            .values()
            .take(10)
            .cloned()
            .collect::<Vec<_>>()
    } else {
        genes.to_vec()
    };

    msg.extend_from_slice(&(share.len() as u8).to_le_bytes());
    for gene in &share {
        msg.extend_from_slice(&gene.to_bytes());
    }

    msg
}

/// Maneja conexión TCP entrante
fn handle_tcp_connection(
    stream: &mut TcpStream,
    _addr: SocketAddr,
    state: &Arc<Mutex<GossipState>>,
) {
    use std::io::{Read, Write};

    let mut buf = Vec::new();
    let mut tmp = [0u8; 4096];

    // Read all data
    stream.set_read_timeout(Some(Duration::from_secs(5))).ok();
    loop {
        match stream.read(&mut tmp) {
            Ok(0) => break,
            Ok(n) => buf.extend_from_slice(&tmp[..n]),
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
            Err(_) => return,
        }
    }

    if buf.is_empty() {
        return;
    }

    let mut state = state.lock().unwrap();
    let mut offset = 0;

    while offset < buf.len() {
        if offset + 19 > buf.len() {
            break;
        }

        let header = match GossipHeader::from_bytes(&buf[offset..offset + 19].try_into().unwrap()) {
            Some(h) => h,
            None => break,
        };
        offset += 19;

        let payload_len = header.payload_len as usize;
        if offset + payload_len > buf.len() {
            break;
        }

        match header.msg_type {
            GossipMsgType::GossipHave => {
                // Parse peers
                let count = buf[offset];
                offset += 1;

                for _ in 0..count {
                    if offset + 10 > buf.len() {
                        break;
                    }

                    let id = u64::from_le_bytes(buf[offset..offset + 8].try_into().unwrap());
                    offset += 8;
                    let ip = [
                        buf[offset],
                        buf[offset + 1],
                        buf[offset + 2],
                        buf[offset + 3],
                    ];
                    let port = u16::from_le_bytes([buf[offset + 4], buf[offset + 5]]);
                    offset += 6;

                    let addr: SocketAddr =
                        format!("{}.{}.{}.{}:{}", ip[0], ip[1], ip[2], ip[3], port)
                            .parse()
                            .unwrap_or_else(|_| "127.0.0.1:4849".parse().unwrap());

                    state.add_peer(PeerId::new(id), addr);
                }
            }
            GossipMsgType::GenesHave => {
                let count = buf[offset];
                offset += 1;

                for _ in 0..count {
                    if offset + 32 > buf.len() {
                        break;
                    }

                    if let Some(gene) = AutonGene::from_bytes(&buf[offset..offset + 32]) {
                        state.add_gene(gene.clone());
                        state.inbound_genes.push(gene);
                    }
                    offset += 32;
                }
            }
            _ => {
                offset += payload_len;
            }
        }
    }

    // Respond with our state
    let response = build_gossip_have(&mut state);
    let _ = stream.write_all(&response);
}

// ============================================================================
// G-FEP: Gene-First Emergence Protocol (reemplaza Kademlia)
// ============================================================================

/// Inicializa el enjambre completo (G-FEP)
pub fn init_hybrid(
    my_port: u16,
    seed_addr: Option<&str>,
    _seed_port: Option<u16>,
) -> std::io::Result<()> {
    // Iniciar G-FEP para genes
    if let Err(e) = crate::gfep::init(my_port) {
        println!("[SWARM] G-FEP init failed: {}", e);
    }

    // Añadir seeds si se proporcionan
    if let Some(addr) = seed_addr {
        if let Err(e) = crate::gfep::add_seed(addr) {
            println!("[SWARM] Failed to add seed {}: {}", addr, e);
        }
    }

    // Iniciar Gossip para genes también
    init(my_port)?;

    println!("[SWARM] G-FEP + Gossip hybrid stack initialized");
    Ok(())
}

/// Comparte genes usando G-FEP (fitness-based propagation)
pub fn hybrid_share_genes(genes: &[(u64, f32)]) {
    // Registrar genes locales en G-FEP
    crate::gfep::register_local_genes(genes);

    // Ciclo G-FEP (evalúa y propaga)
    crate::gfep::cycle();
}

/// Obtiene genes recibidos de G-FEP (para inyección)
pub fn gfep_take_genes() -> Vec<crate::gfep::GeneCell> {
    crate::gfep::take_received_genes()
}

/// Stats del enjambre
pub fn get_stats() -> String {
    let gfep_stats = crate::gfep::stats();
    format!(
        "G-FEP: node={}, genes={}, links={}/{}",
        gfep_stats.node_id, gfep_stats.local_genes, gfep_stats.active_links, gfep_stats.total_links
    )
}

// ============================================================================
// CONSENSUS MODULE - Simplified Raft-like consensus for swarm decisions
// ============================================================================

/// Role en el consenso
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ConsensusRole {
    Follower,
    Candidate,
    Leader,
}

/// Entrada de log de consenso
#[derive(Clone, Debug)]
pub struct ConsensusEntry {
    pub term: u64,
    pub index: u64,
    pub command: ConsensusCommand,
    pub timestamp: u64,
}

/// Comando de consenso
#[derive(Clone, Debug)]
pub enum ConsensusCommand {
    AddGene(u64),
    RemoveGene(u64),
    UpdateTopology(Vec<PeerId>),
    SetLeader(PeerId),
    QuorumDecision(String),
}

/// Estado del consenso distribuido
pub struct ConsensusState {
    pub role: ConsensusRole,
    pub current_term: u64,
    pub voted_for: Option<PeerId>,
    pub log: Vec<ConsensusEntry>,
    pub commit_index: u64,
    pub last_applied: u64,
    pub leader_id: Option<PeerId>,
    pub election_timeout: u64,
    pub heartbeat_interval: u64,
    pub votes_received: HashSet<PeerId>,
    pub last_heartbeat: u64,
}

impl ConsensusState {
    pub fn new(_self_id: PeerId) -> Self {
        ConsensusState {
            role: ConsensusRole::Follower,
            current_term: 0,
            voted_for: None,
            log: Vec::new(),
            commit_index: 0,
            last_applied: 0,
            leader_id: None,
            election_timeout: 150 + (rand_u64() % 150), // 150-300ms
            heartbeat_interval: 50,
            votes_received: HashSet::new(),
            last_heartbeat: 0,
        }
    }

    /// Inicia elección
    pub fn start_election(&mut self, self_id: PeerId) -> bool {
        if self.role == ConsensusRole::Leader {
            return false;
        }

        self.role = ConsensusRole::Candidate;
        self.current_term += 1;
        self.voted_for = Some(self_id);
        self.votes_received.clear();
        self.votes_received.insert(self_id);

        println!(
            "[CONSENSUS] Node {:?} starting election for term {}",
            self_id, self.current_term
        );
        true
    }

    /// Registra voto
    pub fn register_vote(&mut self, voter: PeerId) {
        self.votes_received.insert(voter);
    }

    /// Verifica si tiene mayoría
    pub fn has_majority(&self, total_nodes: usize) -> bool {
        self.votes_received.len() > total_nodes / 2
    }

    /// Se convierte en leader
    pub fn become_leader(&mut self, self_id: PeerId) {
        self.role = ConsensusRole::Leader;
        self.leader_id = Some(self_id);
        self.votes_received.clear();
        println!(
            "[CONSENSUS] Node {:?} became leader for term {}",
            self_id, self.current_term
        );
    }

    /// Recibe latido del leader
    pub fn receive_heartbeat(&mut self, leader_term: u64, leader: PeerId) {
        if leader_term > self.current_term {
            self.current_term = leader_term;
            self.role = ConsensusRole::Follower;
            self.voted_for = None;
        }
        self.leader_id = Some(leader);
        self.last_heartbeat = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
    }

    /// Verifica si necesita elección
    pub fn needs_election(&self) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        now - self.last_heartbeat > self.election_timeout
    }

    /// Añade entrada al log
    pub fn append_entry(&mut self, command: ConsensusCommand) -> u64 {
        let index = self.log.len() as u64 + 1;
        let entry = ConsensusEntry {
            term: self.current_term,
            index,
            command,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        };
        self.log.push(entry);
        index
    }

    /// Confirma entradas hasta índice
    pub fn commit_entries(&mut self, up_to_index: u64) {
        if up_to_index > self.commit_index && up_to_index <= self.log.len() as u64 {
            self.commit_index = up_to_index;
        }
    }
}

/// Administrador de consenso
pub struct ConsensusManager {
    self_id: PeerId,
    state: ConsensusState,
    peers: HashMap<PeerId, ConsensusPeer>,
    active_proposals: HashMap<u64, Proposal>,
}

/// Peer de consenso
#[derive(Clone, Debug)]
pub struct ConsensusPeer {
    pub id: PeerId,
    pub addr: SocketAddr,
    pub last_contact: u64,
    pub match_index: u64,
    pub next_index: u64,
}

/// Propuesta de consenso
#[derive(Clone, Debug)]
pub struct Proposal {
    pub id: u64,
    pub command: ConsensusCommand,
    pub proposed_by: PeerId,
    pub votes: HashSet<PeerId>,
    pub accepted: bool,
    pub term: u64,
}

impl ConsensusManager {
    pub fn new(self_id: PeerId) -> Self {
        ConsensusManager {
            self_id,
            state: ConsensusState::new(self_id),
            peers: HashMap::new(),
            active_proposals: HashMap::new(),
        }
    }

    /// Añade peer al consenso
    pub fn add_peer(&mut self, id: PeerId, addr: SocketAddr) {
        self.peers.insert(
            id,
            ConsensusPeer {
                id,
                addr,
                last_contact: 0,
                match_index: 0,
                next_index: 1,
            },
        );
    }

    /// Inicia propuesta de consenso
    pub fn propose(&mut self, command: ConsensusCommand) -> u64 {
        let proposal_id = self.state.append_entry(command);
        proposal_id
    }

    /// Obtiene estado actual
    pub fn get_state(&self) -> &ConsensusState {
        &self.state
    }

    /// Procesa tick de consenso
    pub fn tick(&mut self) -> ConsensusAction {
        if self.state.needs_election() {
            if self.state.start_election(self.self_id) {
                return ConsensusAction::RequestVotes;
            }
        }

        if self.state.role == ConsensusRole::Leader {
            ConsensusAction::SendHeartbeat
        } else {
            ConsensusAction::None
        }
    }
}

/// Acción de consenso
#[derive(Debug)]
pub enum ConsensusAction {
    None,
    RequestVotes,
    SendHeartbeat,
    AppendEntries(u64),
}

/// Verifica si tenemos quorum
pub fn check_quorum(active_peers: usize, total_peers: usize) -> bool {
    active_peers >= (total_peers / 2) + 1
}

// ============================================================================
// COALITION FORMATION - Group behavior and alliance formation
// ============================================================================

/// Coalición entre agentes
#[derive(Clone, Debug)]
pub struct Coalition {
    pub id: u64,
    pub leader: PeerId,
    pub members: HashSet<PeerId>,
    pub purpose: CoalitionPurpose,
    pub formed_at: u64,
    pub strength: f32,
    pub resources: HashMap<String, f32>,
}

/// Propósito de coalición
#[derive(Clone, Debug)]
pub enum CoalitionPurpose {
    GeneProtection,
    ResourceSharing,
    Defense,
    Expansion,
    Research,
}

/// Gestor de coaliciones
pub struct CoalitionManager {
    coalitions: HashMap<u64, Coalition>,
    membership: HashMap<PeerId, u64>,
    coalition_id_counter: u64,
}

impl CoalitionManager {
    pub fn new() -> Self {
        CoalitionManager {
            coalitions: HashMap::new(),
            membership: HashMap::new(),
            coalition_id_counter: 0,
        }
    }

    /// Crea nueva coalición
    pub fn create_coalition(&mut self, leader: PeerId, purpose: CoalitionPurpose) -> u64 {
        let id = self.coalition_id_counter;
        self.coalition_id_counter += 1;

        let coalition = Coalition {
            id,
            leader,
            members: vec![leader].into_iter().collect(),
            purpose,
            formed_at: timestamp_now(),
            strength: 1.0,
            resources: HashMap::new(),
        };

        self.coalitions.insert(id, coalition);
        self.membership.insert(leader, id);
        id
    }

    /// Añade miembro a coalición
    pub fn add_member(&mut self, coalition_id: u64, member: PeerId) -> bool {
        if let Some(coalition) = self.coalitions.get_mut(&coalition_id) {
            coalition.members.insert(member);
            self.membership.insert(member, coalition_id);
            coalition.strength += 0.1;
            true
        } else {
            false
        }
    }

    /// Elimina miembro de coalición
    pub fn remove_member(&mut self, member: PeerId) -> bool {
        if let Some(coalition_id) = self.membership.get(&member) {
            if let Some(coalition) = self.coalitions.get_mut(coalition_id) {
                coalition.members.remove(&member);
                coalition.strength -= 0.1;
            }
            self.membership.remove(&member);
            true
        } else {
            false
        }
    }

    /// Encuentra coalición para peer
    pub fn find_coalition(&self, peer: PeerId) -> Option<u64> {
        self.membership.get(&peer).copied()
    }

    /// Evalúa fuerza de coalición
    pub fn evaluate_strength(&self, coalition_id: u64) -> f32 {
        self.coalitions
            .get(&coalition_id)
            .map(|c| c.strength * c.members.len() as f32)
            .unwrap_or(0.0)
    }

    /// Disuelve coalición
    pub fn dissolve(&mut self, coalition_id: u64) {
        if let Some(coalition) = self.coalitions.remove(&coalition_id) {
            for member in coalition.members {
                self.membership.remove(&member);
            }
        }
    }
}

impl Default for CoalitionManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// LEADERSHIP ELECTION - Emergent leadership based on fitness/reputation
// ============================================================================

/// Candidato a leader
#[derive(Clone, Debug)]
pub struct LeadershipCandidate {
    pub peer_id: PeerId,
    pub fitness: f32,
    pub reputation: f32,
    pub uptime: u64,
    pub vote_count: u64,
}

/// Resultado de elección
#[derive(Clone, Debug)]
pub struct ElectionResult {
    pub winner: Option<PeerId>,
    pub term: u64,
    pub votes: HashMap<PeerId, u64>,
    pub changed: bool,
}

/// Gestor de elecciones
pub struct LeadershipElection {
    candidates: HashMap<PeerId, LeadershipCandidate>,
    current_leader: Option<PeerId>,
    election_term: u64,
    voting_deadline: u64,
}

impl LeadershipElection {
    pub fn new() -> Self {
        LeadershipElection {
            candidates: HashMap::new(),
            current_leader: None,
            election_term: 0,
            voting_deadline: 0,
        }
    }

    /// Registra candidato
    pub fn register_candidate(&mut self, peer_id: PeerId, fitness: f32) {
        let candidate = LeadershipCandidate {
            peer_id,
            fitness,
            reputation: 0.5,
            uptime: 0,
            vote_count: 0,
        };
        self.candidates.insert(peer_id, candidate);
    }

    /// Vota por candidato
    pub fn vote(&mut self, _voter: PeerId, candidate: PeerId) {
        let candidates_len = self.candidates.len();
        if let Some(c) = self.candidates.get_mut(&candidate) {
            c.vote_count += 1;
            // Update reputation based on votes received
            c.reputation = (c.vote_count as f32 / (candidates_len as f32 + 1.0)).min(1.0);
        }
    }

    /// Computa leader basado en fitness + reputación + uptime
    pub fn compute_leader(&self) -> Option<PeerId> {
        self.candidates
            .iter()
            .max_by(|a, b| {
                let score_a =
                    a.1.fitness * 0.4 + a.1.reputation * 0.4 + (a.1.uptime as f32 / 1000.0) * 0.2;
                let score_b =
                    b.1.fitness * 0.4 + b.1.reputation * 0.4 + (b.1.uptime as f32 / 1000.0) * 0.2;
                score_a
                    .partial_cmp(&score_b)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(id, _)| *id)
    }

    /// Inicia nueva elección
    pub fn start_election(&mut self) -> ElectionResult {
        self.election_term += 1;
        let votes = HashMap::new();

        // Clear previous vote counts
        for candidate in self.candidates.values_mut() {
            candidate.vote_count = 0;
        }

        ElectionResult {
            winner: None,
            term: self.election_term,
            votes,
            changed: false,
        }
    }

    /// Finaliza elección
    pub fn finish_election(&mut self) -> ElectionResult {
        let new_leader = self.compute_leader();
        let changed = new_leader != self.current_leader;

        let result = ElectionResult {
            winner: new_leader,
            term: self.election_term,
            votes: HashMap::new(),
            changed,
        };

        if changed {
            self.current_leader = new_leader;
        }

        result
    }

    /// Obtiene leader actual
    pub fn get_leader(&self) -> Option<PeerId> {
        self.current_leader
    }

    /// Verifica si somos leader
    pub fn is_leader(&self, peer_id: PeerId) -> bool {
        self.current_leader == Some(peer_id)
    }
}

impl Default for LeadershipElection {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// ANTI-ENTROPY GOSSIP - Rumor mongering with anti-entropy
// ============================================================================

/// Información de rumor
#[derive(Clone, Debug)]
pub struct RumorInfo {
    pub origin: PeerId,
    pub content: RumorContent,
    pub ttl: u8,
    pub version: u64,
}

/// Contenido de rumor
#[derive(Clone, Debug)]
pub enum RumorContent {
    GeneUpdate { gene_key: u64, fitness: f32 },
    PeerJoined { peer: PeerId, addr: SocketAddr },
    PeerLeft { peer: PeerId },
    CoalitionFormed { coalition_id: u64 },
    LeadershipChange { new_leader: PeerId },
}

/// Rumor en memoria
pub struct RumorMongerer {
    known_rumors: HashMap<(PeerId, u64), RumorInfo>,
    max_rumors: usize,
    max_ttl: u8,
}

impl RumorMongerer {
    pub fn new() -> Self {
        RumorMongerer {
            known_rumors: HashMap::new(),
            max_rumors: 1000,
            max_ttl: 8,
        }
    }

    /// Spead rumor (rumor mongering)
    pub fn spread_rumor(&mut self, origin: PeerId, content: RumorContent, version: u64) {
        let key = (origin, version);

        // Limit storage
        if self.known_rumors.len() >= self.max_rumors {
            // Remove oldest
            if let Some(oldest) = self
                .known_rumors
                .iter()
                .find(|(_, r)| r.ttl == self.max_ttl)
                .map(|(k, _)| *k)
            {
                self.known_rumors.remove(&oldest);
            }
        }

        self.known_rumors.insert(
            key,
            RumorInfo {
                origin,
                content,
                ttl: self.max_ttl,
                version,
            },
        );
    }

    /// Decay TTL de rumores
    pub fn decay_rumors(&mut self) {
        for rumor in self.known_rumors.values_mut() {
            if rumor.ttl > 0 {
                rumor.ttl -= 1;
            }
        }

        // Remove expired
        self.known_rumors.retain(|_, r| r.ttl > 0);
    }

    /// Selecciona rumores para compartir
    pub fn select_rumors_to_share(&self, count: usize, exclude_peer: PeerId) -> Vec<RumorInfo> {
        self.known_rumors
            .values()
            .filter(|r| r.origin != exclude_peer && r.ttl > 0)
            .take(count)
            .cloned()
            .collect()
    }

    /// Verifica si rumor es conocido
    pub fn is_known(&self, origin: PeerId, version: u64) -> bool {
        self.known_rumors.contains_key(&(origin, version))
    }
}

impl Default for RumorMongerer {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// FAULT TOLERANCE - Detection and recovery
// ============================================================================

/// Estado de nodo
#[derive(Clone, Debug, PartialEq, Copy)]
pub enum NodeHealth {
    Healthy,
    Suspected,
    Dead,
}

/// Tracking de salud de peer
pub struct PeerHealthTracker {
    health: HashMap<PeerId, NodeHealth>,
    suspicion_count: HashMap<PeerId, u32>,
    last_ping: HashMap<PeerId, u64>,
}

impl PeerHealthTracker {
    pub fn new() -> Self {
        PeerHealthTracker {
            health: HashMap::new(),
            suspicion_count: HashMap::new(),
            last_ping: HashMap::new(),
        }
    }

    /// Registra ping
    pub fn record_ping(&mut self, peer: PeerId) {
        self.last_ping.insert(peer, timestamp_now());

        // Reset suspicion if healthy
        if let Some(count) = self.suspicion_count.get_mut(&peer) {
            if *count > 0 {
                *count -= 1;
            }
        }

        self.health.insert(peer, NodeHealth::Healthy);
    }

    /// Registra failure
    pub fn record_failure(&mut self, peer: PeerId) {
        let count = self.suspicion_count.entry(peer).or_insert(0);
        *count += 1;

        if *count >= 3 {
            self.health.insert(peer, NodeHealth::Dead);
        } else {
            self.health.insert(peer, NodeHealth::Suspected);
        }
    }

    /// Obtiene salud
    pub fn get_health(&self, peer: PeerId) -> NodeHealth {
        self.health
            .get(&peer)
            .cloned()
            .unwrap_or(NodeHealth::Healthy)
    }

    /// Verifica si peer está muerto
    pub fn is_dead(&self, peer: PeerId) -> bool {
        self.health.get(&peer).copied() == Some(NodeHealth::Dead)
    }

    /// Obtiene peers muertos
    pub fn get_dead_peers(&self) -> Vec<PeerId> {
        self.health
            .iter()
            .filter(|(_, h)| **h == NodeHealth::Dead)
            .map(|(p, _)| *p)
            .collect()
    }
}

impl Default for PeerHealthTracker {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// UTILITIES
// ============================================================================

fn timestamp_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

fn rand_u64() -> u64 {
    use std::collections::hash_map::RandomState;
    let rs = RandomState::new();
    use std::hash::{Hash, Hasher};
    let mut hasher = rs.build_hasher();
    std::time::Instant::now().hash(&mut hasher);
    hasher.finish()
}

/// Inicializa consenso (debe llamarse después de init)
pub fn init_consensus(self_id: PeerId) -> ConsensusManager {
    ConsensusManager::new(self_id)
}

/// Inicializa gestión de coaliciones
pub fn init_coalitions() -> CoalitionManager {
    CoalitionManager::new()
}

/// Inicializa elecciones de liderazgo
pub fn init_leadership_election() -> LeadershipElection {
    LeadershipElection::new()
}

/// Obtiene timestamp actual
pub fn now_millis() -> u64 {
    timestamp_now()
}
