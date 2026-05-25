//! # G-FEP: Gene-First Emergence Protocol
//!
//! Protocolo P2P 100% original creado para EDEN.
//! Sin dependencias externas - Sin algoritmos conocidos - Solo genética.
//!
//! ## Concepto:
//!
//! Los genes son ciudadanos de primera clase en la red.
//! La red se construye orgánicamente según fitness de genes.
//!
//! ## Cómo funciona:
//!
//! 1. **Gene Broadcasting**: Cada nodo difunde sus genes top-K periódicamente
//! 2. **Gene Evaluation**: Nodos evalúan genes recibidos contra sus propios
//! 3. **Adaptive Connection**: Si un gen es mejor, me conecto a su fuente Y adopto el gen
//! 4. **Gene Death**: Genes con bajo fitness pierden conexiones y "mueren"
//! 5. **Mesh Emergence**: La topología de red emerge de la distribución de fitness
//!
//! ## Sin referencias a algoritmos conocidos:
//! - Sin Kademlia (no hay buckets)
//! - Sin DHT (no hay lookup por clave)
//! - Sin Gossip (no hay intercambio de peers)
//! - Solo genes fluyendo hacia donde son útiles
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream, UdpSocket};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

// ============================================================================
// Constantes G-FEP
// ============================================================================

/// Puerto UDP para broadcast de genes locales
const GENE_BROADCAST_PORT: u16 = 4860;
/// Puerto TCP para transferencia de genes
const GENE_TRANSFER_PORT: u16 = 4861;
/// Máximo de genes que un nodo puede tener como "top"
const TOP_GENES_COUNT: usize = 10;
/// Cada cuántos ciclos se hace broadcast de genes
const BROADCAST_INTERVAL_CYCLES: u64 = 1000;
/// Timeout para considerar un gen "muerto"
const GENE_DEATH_TIMEOUT_SECS: u64 = 300;
/// Timeout para conexiones
const CONNECTION_TIMEOUT_MS: u64 = 2000;
// ============================================================================
// Estado global
// ============================================================================

static GFEP_STATE: std::sync::OnceLock<Arc<Mutex<GFEPState>>> = std::sync::OnceLock::new();

fn get_state() -> &'static Arc<Mutex<GFEPState>> {
    GFEP_STATE.get_or_init(|| Arc::new(Mutex::new(GFEPState::new())))
}

// ============================================================================
// Tipos G-FEP
// ============================================================================

/// Gen con metadata de red
#[derive(Clone, Debug)]
pub struct GeneCell {
    /// Semilla del gen
    pub ramnet_seed: u64,
    /// Fitness actual
    pub fitness: f32,
    /// Cuántas veces ha sido adoptado por otros
    pub adoption_count: u32,
    /// Última vez que fue visto
    pub last_seen: Instant,
    /// источник (source) - IP del nodo que lo originó
    pub source_addr: Option<SocketAddr>,
    /// Número de ciclos desde que nació
    pub age_cycles: u64,
}

impl GeneCell {
    pub fn new(ramnet_seed: u64, fitness: f32) -> Self {
        GeneCell {
            ramnet_seed,
            fitness,
            adoption_count: 0,
            last_seen: Instant::now(),
            source_addr: None,
            age_cycles: 0,
        }
    }

    /// Signal strength = fitness * adoption_count (cuanto más adoptado, más fuerte)
    pub fn signal_strength(&self) -> f32 {
        self.fitness * (1.0 + self.adoption_count as f32 * 0.1)
    }

    /// Check si el gen está "vivo"
    pub fn is_alive(&self) -> bool {
        Instant::now().duration_since(self.last_seen).as_secs() < GENE_DEATH_TIMEOUT_SECS
    }
}

/// Conexión a otro nodo
#[derive(Debug)]
struct GeneLink {
    /// Dirección del peer
    addr: SocketAddr,
    /// Genes que compartimos con este peer
    shared_genes: Vec<u64>,
    /// Última vez que recibimos genes de él
    last_seen: Instant,
    /// ¿Está activo?
    active: bool,
}

/// Estado del nodo G-FEP
struct GFEPState {
    /// Nuestro ID único
    node_id: u64,
    /// Nuestros genes locales (los que generamos nosotros)
    local_genes: HashMap<u64, GeneCell>,
    /// Genes conocidos de la red (fuera de nosotros)
    network_genes: HashMap<u64, GeneCell>,
    /// Conexiones activas a otros nodos
    links: HashMap<SocketAddr, GeneLink>,
    /// Genes pendientes de solicitar (de otros nodos)
    pending_requests: Vec<u64>,
    /// Último broadcast
    last_broadcast: Instant,
    /// Ciclo actual
    current_cycle: u64,
    /// Puerto local
    local_port: u16,
    /// Genes recibidos esta sesión
    received_genes: Vec<GeneCell>,
}

impl GFEPState {
    fn new() -> Self {
        GFEPState {
            node_id: generate_node_id(),
            local_genes: HashMap::new(),
            network_genes: HashMap::new(),
            links: HashMap::new(),
            pending_requests: Vec::new(),
            last_broadcast: Instant::now(),
            current_cycle: 0,
            local_port: 0,
            received_genes: Vec::new(),
        }
    }

    /// Registra un gen local
    fn register_gene(&mut self, ramnet_seed: u64, fitness: f32) {
        if let Entry::Vacant(e) = self.local_genes.entry(ramnet_seed) {
            e.insert(GeneCell::new(ramnet_seed, fitness));
        } else {
            // Update fitness
            if let Some(gene) = self.local_genes.get_mut(&ramnet_seed) {
                gene.fitness = fitness;
                gene.last_seen = Instant::now();
            }
        }
    }

    /// Obtiene los genes top-K por signal strength
    fn get_top_genes(&self, count: usize) -> Vec<GeneCell> {
        let mut all_genes: Vec<_> = self.local_genes.values().collect();
        all_genes.sort_by(|a, b| {
            b.signal_strength()
                .partial_cmp(&a.signal_strength())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        all_genes.into_iter().take(count).cloned().collect()
    }

    /// Evalúa un gen recibido y decide si adoptarlo
    fn evaluate_gene(&mut self, gene: GeneCell) -> bool {
        let ramnet_seed = gene.ramnet_seed;

        // ¿Ya tenemos este gen?
        if let Some(existing) = self.local_genes.get(&ramnet_seed) {
            // ¿El nuevo es mejor?
            if gene.fitness > existing.fitness * 1.1 {
                // 10% mejor mínimo
                // Adoptar - actualizar
                let mut updated = gene.clone();
                updated.adoption_count = existing.adoption_count + 1;
                updated.source_addr = gene.source_addr;
                self.local_genes.insert(ramnet_seed, updated);
                self.received_genes.push(gene);
                return true;
            }
        } else {
            // Nuevo gen - adoptar si tiene fitness decente
            if gene.fitness > 0.0 {
                let mut adopted = gene.clone();
                adopted.adoption_count = 1;
                self.local_genes.insert(ramnet_seed, adopted);
                self.received_genes.push(gene);
                return true;
            }
        }
        false
    }

    /// Añade un link a otro nodo
    fn add_link(&mut self, addr: SocketAddr) {
        if self.links.len() >= 50 {
            // Máximo 50 connections
            // Remover link inactivo más antiguo
            if let Some((oldest_addr, _)) = self
                .links
                .iter()
                .find(|(_, l)| !l.active)
                .map(|(k, v)| (*k, v.last_seen))
            {
                self.links.remove(&oldest_addr);
            }
        }

        self.links.entry(addr).or_insert(GeneLink {
            addr,
            shared_genes: Vec::new(),
            last_seen: Instant::now(),
            active: true,
        });
    }

    /// Actualiza un link
    fn touch_link(&mut self, addr: &SocketAddr) {
        if let Some(link) = self.links.get_mut(addr) {
            link.last_seen = Instant::now();
            link.active = true;
        }
    }

    /// Evalúa links y marca inactivos
    fn eval_links(&mut self) {
        let now = Instant::now();
        for link in self.links.values_mut() {
            if now.duration_since(link.last_seen).as_secs() > 60 {
                link.active = false;
            }
        }
    }

    /// Cleanup genes muertos
    fn cleanup_genes(&mut self) {
        self.network_genes.retain(|_, g| g.is_alive());
        self.local_genes.retain(|_, g| g.is_alive());
    }
}

// ============================================================================
// Funciones públicas
// ============================================================================

/// Inicializa G-FEP
pub fn init(port: u16) -> std::io::Result<()> {
    let mut state = get_state().lock().unwrap();
    state.local_port = port;
    println!(
        "[G-FEP] Node {} initialized on port {}",
        state.node_id, port
    );

    // Hilo: broadcast de genes locales via UDP
    let state_ref = get_state().clone();
    thread::spawn(move || {
        gene_broadcast_loop(state_ref);
    });

    // Hilo: receptor TCP para recibir genes
    let state_ref = get_state().clone();
    thread::spawn(move || {
        gene_transfer_listener(state_ref, port);
    });

    // Hilo: evaluación periódica
    let state_ref = get_state().clone();
    thread::spawn(move || {
        evaluation_loop(state_ref);
    });

    Ok(())
}

/// Registra genes locales (llamar cada ciclo con genes actuales)
pub fn register_local_genes(genes: &[(u64, f32)]) {
    let mut state = get_state().lock().unwrap();
    for (seed, fitness) in genes {
        state.register_gene(*seed, *fitness);
    }
    state.current_cycle += 1;
}

/// Obtiene genes recibidos (para inyección)
pub fn take_received_genes() -> Vec<GeneCell> {
    let mut state = get_state().lock().unwrap();
    let genes: Vec<_> = state.received_genes.drain(..).collect();
    genes
}

/// Conecta a un peer específico
pub fn connect(addr: &str) -> std::io::Result<()> {
    let peer_addr: SocketAddr = addr
        .parse()
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid address"))?;

    let mut state = get_state().lock().unwrap();
    state.add_link(peer_addr);

    // Enviar solicitud de genes
    drop(state);
    send_gene_request(peer_addr)?;

    Ok(())
}

/// Añade un seed/bootstrap node
pub fn add_seed(addr: &str) -> std::io::Result<()> {
    let seed_addr: SocketAddr = addr
        .parse()
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid address"))?;
    let mut state = get_state().lock().unwrap();
    state.add_link(seed_addr);
    println!("[G-FEP] Seed added: {}", addr);
    Ok(())
}

/// Número de links activos
pub fn active_links() -> usize {
    get_state()
        .lock()
        .unwrap()
        .links
        .values()
        .filter(|l| l.active)
        .count()
}

/// Genes locales actuales
pub fn local_gene_count() -> usize {
    get_state().lock().unwrap().local_genes.len()
}

/// Obtiene stats
pub fn stats() -> GFEPStats {
    let state = get_state().lock().unwrap();
    GFEPStats {
        node_id: state.node_id,
        local_genes: state.local_genes.len(),
        network_genes: state.network_genes.len(),
        active_links: state.links.values().filter(|l| l.active).count(),
        total_links: state.links.len(),
    }
}

#[derive(Debug, Clone)]
pub struct GFEPStats {
    pub node_id: u64,
    pub local_genes: usize,
    pub network_genes: usize,
    pub active_links: usize,
    pub total_links: usize,
}

/// Ciclo G-FEP (llamar cada ciclo de simulación)
pub fn cycle() {
    let mut state = get_state().lock().unwrap();
    state.eval_links();
    state.cleanup_genes();

    // Cada BROADCAST_INTERVAL_CYCLES, hacer broadcast
    if state.current_cycle % BROADCAST_INTERVAL_CYCLES == 0 {
        let top_genes = state.get_top_genes(TOP_GENES_COUNT);
        drop(state);
        broadcast_genes(top_genes);
    }
}

// ============================================================================
// Hilos internos
// ============================================================================

fn gene_broadcast_loop(state_ref: Arc<Mutex<GFEPState>>) {
    // Broadcast UDP de genes locales cada 5 segundos
    if let Ok(socket) = UdpSocket::bind(format!("0.0.0.0:{}", GENE_BROADCAST_PORT)) {
        socket.set_broadcast(true).ok();
        socket.set_nonblocking(true).ok();

        let mut buf = [0u8; 1024];

        loop {
            // Recibir broadcasts de otros
            if let Ok((len, addr)) = socket.recv_from(&mut buf) {
                if let Some(genes) = parse_gene_broadcast(&buf[..len]) {
                    let mut state = state_ref.lock().unwrap();
                    for gene in genes {
                        let mut g = gene;
                        g.source_addr = Some(addr);
                        if state.evaluate_gene(g) {
                            // Si adoptamos, conectar a la fuente
                            state.add_link(addr);
                        }
                    }
                }
            }
            thread::sleep(Duration::from_millis(100));
        }
    }
}

fn gene_transfer_listener(state_ref: Arc<Mutex<GFEPState>>, port: u16) {
    let addr = format!("0.0.0.0:{}", port);
    if let Ok(listener) = TcpListener::bind(&addr) {
        listener.set_nonblocking(true).ok();

        loop {
            match listener.accept() {
                Ok((mut stream, addr)) => {
                    let s = state_ref.clone();
                    thread::spawn(move || {
                        handle_gene_transfer(&mut stream, addr, &s);
                    });
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(10));
                }
                Err(_) => {}
            }
        }
    }
}

fn evaluation_loop(state_ref: Arc<Mutex<GFEPState>>) {
    loop {
        {
            let mut state = state_ref.lock().unwrap();
            state.eval_links();
            state.cleanup_genes();
        }
        thread::sleep(Duration::from_secs(1));
    }
}

// ============================================================================
// Protocolo de mensajes
// ============================================================================

/// Mensaje de broadcast de genes (UDP)
/// Formato: [count:u8][gene:32bytes x count]
fn encode_gene_broadcast(genes: &[GeneCell]) -> Vec<u8> {
    let mut msg = Vec::new();
    msg.push(genes.len() as u8);

    for gene in genes {
        msg.extend_from_slice(&gene.ramnet_seed.to_le_bytes());
        msg.extend_from_slice(&gene.fitness.to_le_bytes());
        msg.extend_from_slice(&(gene.adoption_count as u32).to_le_bytes());
    }

    msg
}

fn parse_gene_broadcast(data: &[u8]) -> Option<Vec<GeneCell>> {
    if data.is_empty() {
        return None;
    }

    let count = data[0] as usize;
    let mut offset = 1;
    let mut genes = Vec::new();

    for _ in 0..count {
        if offset + 32 > data.len() {
            break;
        }

        let ramnet_seed = u64::from_le_bytes(data[offset..offset + 8].try_into().ok()?);
        let fitness = f32::from_le_bytes(data[offset + 8..offset + 12].try_into().ok()?);
        let adoption_count = u32::from_le_bytes(data[offset + 12..offset + 16].try_into().ok()?);
        offset += 32;

        genes.push(GeneCell {
            ramnet_seed,
            fitness,
            adoption_count,
            last_seen: Instant::now(),
            source_addr: None,
            age_cycles: 0,
        });
    }

    Some(genes)
}

/// Mensaje de transferencia de genes (TCP)
/// Formato: [type:u8][payload...]
/// type: 0 = REQUEST, 1 = RESPONSE
fn encode_gene_request() -> Vec<u8> {
    vec![0x01]
}

fn encode_gene_response(genes: &[GeneCell]) -> Vec<u8> {
    let mut msg = Vec::new();
    msg.push(0x02);
    msg.push(genes.len() as u8);

    for gene in genes {
        msg.extend_from_slice(&gene.ramnet_seed.to_le_bytes());
        msg.extend_from_slice(&gene.fitness.to_le_bytes());
        msg.extend_from_slice(&(gene.adoption_count as u32).to_le_bytes());
    }

    msg
}

fn parse_gene_response(data: &[u8]) -> Option<Vec<GeneCell>> {
    if data.len() < 2 || data[0] != 0x02 {
        return None;
    }

    let count = data[1] as usize;
    let mut offset = 2;
    let mut genes = Vec::new();

    for _ in 0..count {
        if offset + 32 > data.len() {
            break;
        }

        let ramnet_seed = u64::from_le_bytes(data[offset..offset + 8].try_into().ok()?);
        let fitness = f32::from_le_bytes(data[offset + 8..offset + 12].try_into().ok()?);
        let adoption_count = u32::from_le_bytes(data[offset + 12..offset + 16].try_into().ok()?);
        offset += 32;

        genes.push(GeneCell {
            ramnet_seed,
            fitness,
            adoption_count,
            last_seen: Instant::now(),
            source_addr: None,
            age_cycles: 0,
        });
    }

    Some(genes)
}

// ============================================================================
// Handlers
// ============================================================================

fn handle_gene_transfer(stream: &mut TcpStream, addr: SocketAddr, state: &Arc<Mutex<GFEPState>>) {
    use std::io::Read;

    let mut buf = [0u8; 4096];
    stream
        .set_read_timeout(Some(Duration::from_millis(CONNECTION_TIMEOUT_MS)))
        .ok();

    let n = match stream.read(&mut buf) {
        Ok(n) => n,
        _ => return,
    };

    if n == 0 {
        return;
    }

    let msg_type = buf[0];
    let mut state = state.lock().unwrap();

    match msg_type {
        0x01 => {
            // REQUEST - alguien pide nuestros genes
            state.touch_link(&addr);
            let top_genes = state.get_top_genes(TOP_GENES_COUNT);
            drop(state);
            let response = encode_gene_response(&top_genes);
            let _ = stream.write_all(&response);
        }
        0x02 => {
            // RESPONSE - alguien nos responde con genes
            if let Some(genes) = parse_gene_response(&buf[..n]) {
                for gene in genes {
                    let mut g = gene;
                    g.source_addr = Some(addr);
                    if state.evaluate_gene(g) {
                        state.add_link(addr);
                    }
                }
            }
        }
        _ => {}
    }
}

fn send_gene_request(addr: SocketAddr) -> std::io::Result<()> {
    let mut stream =
        TcpStream::connect_timeout(&addr, Duration::from_millis(CONNECTION_TIMEOUT_MS))?;
    stream.set_write_timeout(Some(Duration::from_millis(CONNECTION_TIMEOUT_MS)))?;
    let request = encode_gene_request();
    stream.write_all(&request)?;

    // Esperar respuesta
    let mut response = [0u8; 4096];
    let n = stream.read(&mut response)?;

    if n > 0 {
        if let Some(genes) = parse_gene_response(&response[..n]) {
            let mut state = get_state().lock().unwrap();
            for gene in genes {
                let mut g = gene;
                g.source_addr = Some(addr);
                if state.evaluate_gene(g) {
                    state.add_link(addr);
                }
            }
        }
    }

    Ok(())
}

fn broadcast_genes(genes: Vec<GeneCell>) {
    if let Ok(socket) = UdpSocket::bind("0.0.0.0:0") {
        socket.set_broadcast(true).ok();
        let msg = encode_gene_broadcast(&genes);
        let _ = socket.send_to(&msg, format!("255.255.255.255:{}", GENE_BROADCAST_PORT));
    }
}

// ============================================================================
// Helpers
// ============================================================================

fn generate_node_id() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    (now ^ (now >> 47)) as u64
}
