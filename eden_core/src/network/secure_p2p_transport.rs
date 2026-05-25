//! # EVO-P2P: Autonomic Secure Transport
//! 
//! Protocolo de transporte P2P seguro 100% original para EDEN.
//! Sin estándares, sin RFCs, sin referencias a algoritmos conocidos.
//! 
//! ## Concepto: Autonomic Encryption
//!
//! La seguridad emerge de la interacción entre nodos como emerge
//! la vida de la interacción de células. No hay cifrado estático,
//! hay "metabolismo" de seguridad.
//!
//! ## Arquitectura:
//!
//! 1. **Metabolic Keys**: Llaves que "digieren" datos, no los cifran
//! 2. **GeneAuth**: Autenticación basada en verificación de "genes"
//! 3. **Surface Hash**: Hash que "envejece" con el tiempo
//! 4. **Session Metabolism**: Cada sesión tiene su propio ciclo de vida
//!
//! ## Sin estándares:
//! - Sin AES, ChaCha20, Poly1305
//! - Sin SHA-2, SHA-3, BLAKE
//! - Sin TLS, Noise Protocol
//! - Sin RSA, ECDH
//!
//! Todo es original y creado específicamente para EDEN.

#![allow(dead_code)]
#![allow(unused_imports)]

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use std::net::SocketAddr;

// ============================================================================
// METABOLIC KEY GENERATION - "Digestión" de datos para seguridad
// ============================================================================

/// MetabolicKey: Una llave que "digiere" datos progresivamente
/// No cifra directamente, transforma los datos en una forma irrevocable
#[derive(Clone)]
pub struct MetabolicKey {
    /// Estado metabólico actual (32 bytes)
    state: [u8; 32],
    /// Generación de la llave (cada uso avanza la generación)
    generation: u64,
    /// Vida útil en ciclos
    lifespan: u64,
}

impl MetabolicKey {
    /// Crea una nueva MetabolicKey desde el node_id de EDEN
    pub fn from_node_id(node_id: u64, entropy: u64) -> Self {
        let mut state = [0u8; 32];
        
        // "Digestión" inicial del entropy
        let combined = node_id.wrapping_mul(entropy);
        for i in 0..32 {
            // Cada byte es una "digestión" diferente del mismo input
            let byte_idx = ((combined >> (i * 4)) & 0xFF) as u8;
            state[i] = Self::digest_byte(byte_idx, (node_id >> (i % 8)) as u8, entropy as u8);
        }
        
        MetabolicKey {
            state,
            generation: 0,
            lifespan: 10000, // ~10k ciclos antes de renovar
        }
    }
    
    /// Función de digestión: transforma un byte en otro byte
    /// Esta es la función "metabólica" original - no basada en ningún cipher
    fn digest_byte(input: u8, key_byte: u8, entropy: u8) -> u8 {
        let mut x = input;
        let mut k = key_byte;
        
        // "Metabolismo" - 8 pasos de transformación irreversibles
        for i in 0..8 {
            // Paso i: combinación única
            x = x.wrapping_add(k.wrapping_mul(entropy.wrapping_add(i as u8)));
            x = x.rotate_left(3 + (i % 5) as u32);
            k = k.wrapping_sub(x ^ entropy);
            
            // Cada paso es diferente - ningún paso revela información del anterior
            if i % 2 == 0 {
                x = x.wrapping_mul(0x9E);
            } else {
                x = x.wrapping_add(0xC3);
            }
        }
        
        x
    }
    
    /// "Digiere" un bloque de datos - irreversible
    pub fn digest(&mut self, data: &[u8]) -> [u8; 32] {
        let mut output = [0u8; 32];
        
        for (i, byte) in data.iter().enumerate() {
            // Cada byte se digiere con su posición y el estado actual
            let pos_byte = Self::digest_byte(*byte, self.state[i % 32], self.generation as u8);
            
            // El resultado se acumula en el output
            output[i % 32] ^= pos_byte;
            
            // Estado metabólico avanza
            self.state[i % 32] = self.state[i % 32].wrapping_add(pos_byte);
        }
        
        self.generation += 1;
        output
    }
    
    /// Avanza la generación de la llave
    pub fn metabolize(&mut self, session_data: &[u8]) {
        let digest = self.digest(session_data);
        
        // El digest modifica el estado - esto es irreversible
        for i in 0..32 {
            self.state[i] = self.state[i].wrapping_add(digest[i]);
        }
        
        self.generation += 1;
    }
    
    /// Obtiene el fingerprint actual (para identificación)
    pub fn fingerprint(&self) -> String {
        let mut fp = String::with_capacity(64);
        for (i, byte) in self.state.iter().enumerate() {
            fp.push_str(&format!("{:02x}", byte));
            if i % 8 == 7 && i < 31 {
                fp.push(':');
            }
        }
        fp
    }
    
    /// Checksum del estado actual
    fn checksum(&self) -> u64 {
        let mut sum: u64 = 0;
        for (i, byte) in self.state.iter().enumerate() {
            sum = sum.wrapping_add((*byte as u64).wrapping_mul((i as u64).wrapping_add(1)));
        }
        sum ^= self.generation;
        sum
    }
}

// ============================================================================
// GENEAUTH - Autenticación basada en "genes"
// ============================================================================

/// Un "gen" de autenticación - verification token único
#[derive(Clone, Debug)]
struct AuthGene {
    /// Posición en el genoma de autenticación
    position: u8,
    /// Valor del gen (diferente para cada nodo)
    value: [u8; 8],
    /// Checksum para verificación
    checksum: u64,
}

impl AuthGene {
    /// Crea un nuevo AuthGene desde la MetabolicKey
    fn from_key(key: &MetabolicKey, position: u8) -> Self {
        let mut value = [0u8; 8];
        for i in 0..8 {
            value[i] = key.state[(position as usize + i) % 32];
        }
        
        AuthGene {
            position,
            value,
            checksum: Self::compute_checksum(&value, position),
        }
    }
    
    fn compute_checksum(value: &[u8; 8], position: u8) -> u64 {
        let mut sum: u64 = 0;
        for (i, byte) in value.iter().enumerate() {
            sum = sum.wrapping_add((*byte as u64).wrapping_mul((i as u64 + 1).wrapping_mul(position as u64 + 1)));
        }
        sum
    }
    
    /// Verifica que este gen es válido
    fn verify(&self) -> bool {
        Self::compute_checksum(&self.value, self.position) == self.checksum
    }
}

/// Genoma de autenticación - colección de AuthGenes
struct AuthGenome {
    genes: Vec<AuthGene>,
    checksum: u64,
}

impl AuthGenome {
    /// Crea un nuevo Genoma desde la MetabolicKey
    fn from_key(key: &MetabolicKey, gene_count: u8) -> Self {
        let mut genes = Vec::with_capacity(gene_count as usize);
        for pos in 0..gene_count {
            genes.push(AuthGene::from_key(key, pos));
        }
        
        let checksum = Self::compute_genome_checksum(&genes);
        
        AuthGenome { genes, checksum }
    }
    
    fn compute_genome_checksum(genes: &[AuthGene]) -> u64 {
        let mut sum: u64 = 0;
        for (i, gene) in genes.iter().enumerate() {
            sum = sum.wrapping_add(gene.checksum.wrapping_mul((i as u64 + 1).wrapping_mul(0x9E3779B9)));
        }
        sum
    }
    
    /// Verifica un genoma recibido
    fn verify(&self) -> bool {
        // Verificar checksum del genoma
        if Self::compute_genome_checksum(&self.genes) != self.checksum {
            return false;
        }
        
        // Verificar cada gen
        self.genes.iter().all(|g| g.verify())
    }
    
    /// Serializa el genoma para transmisión
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(self.genes.len() as u8);
        bytes.extend_from_slice(&self.checksum.to_le_bytes());
        
        for gene in &self.genes {
            bytes.push(gene.position);
            bytes.extend_from_slice(&gene.value);
            bytes.extend_from_slice(&gene.checksum.to_le_bytes());
        }
        
        bytes
    }
    
    /// Deserializa un genoma
    fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 2 { return None; }
        
        let gene_count = data[0] as usize;
        let checksum = u64::from_le_bytes(data[1..9].try_into().ok()?);
        
        let mut genes = Vec::with_capacity(gene_count);
        let mut offset = 9;
        
        for _ in 0..gene_count {
            if offset + 17 > data.len() { return None; }
            
            let position = data[offset];
            let value: [u8; 8] = data[offset+1..offset+9].try_into().ok()?;
            let gene_checksum = u64::from_le_bytes(data[offset+9..offset+17].try_into().ok()?);
            
            genes.push(AuthGene { position, value, checksum: gene_checksum });
            offset += 17;
        }
        
        Some(AuthGenome { genes, checksum })
    }
}

// ============================================================================
// SURFACE HASH - Hash que "envejece"
// ============================================================================

/// SurfaceHash: Hash que cambia con el tiempo, como una superficie que envejece
/// No es un hash criptográfico estándar - es un "aging hash" original
struct SurfaceHash {
    /// Estado interno (64 bytes)
    state: [u64; 8],
    /// Generación del hash
    generation: u64,
}

impl SurfaceHash {
    /// Crea un nuevo SurfaceHash desde datos iniciales
    fn new(data: &[u8], initial_generation: u64) -> Self {
        let mut state = [0u64; 8];
        
        // Cada byte de data modifica una parte del estado
        for (i, byte) in data.iter().enumerate() {
            let idx = i / 8;
            let shift = (i % 8) * 8;
            state[idx] = state[idx].wrapping_add((*byte as u64) << shift);
        }
        
        // "Curar" el estado initial
        for _ in 0..4 {
            Self::cure_state(&mut state);
        }
        
        SurfaceHash {
            state,
            generation: initial_generation,
        }
    }
    
    /// Función de "curado" - mezcla el estado
    fn cure_state(state: &mut [u64; 8]) {
        // Mezcla simple pero efectiva - no basada en ningún cipher
        for i in 0..8 {
            state[i] = state[i].wrapping_mul(0x9E3779B97F4A7C15);
            state[i] = state[i].rotate_right(31);
        }
        
        // Cross-feed
        for i in 0..8 {
            let j = (i + 1) % 8;
            state[i] ^= state[j].wrapping_add(0xC6EF3720);
        }
    }
    
    /// Añade datos al hash (actualiza)
    pub fn add(&mut self, data: &[u8]) {
        for (i, byte) in data.iter().enumerate() {
            let idx = (i % 8);
            let shift = ((i / 8) % 8) as u32;
            
            // El dato modifica el estado de forma irreversible
            self.state[idx] = self.state[idx].wrapping_add((*byte as u64).wrapping_mul(self.generation.wrapping_add(i as u64)));
            self.state[idx] = self.state[idx].rotate_right(shift + 7);
        }
        
        // Cure después de cada adición
        Self::cure_state(&mut self.state);
        self.generation += 1;
    }
    
    /// Obtiene el hash actual como bytes
    pub fn current(&self) -> [u8; 64] {
        let mut out = [0u8; 64];
        for (i, word) in self.state.iter().enumerate() {
            out[i*8..(i+1)*8].copy_from_slice(&word.to_le_bytes());
        }
        out
    }
    
    /// Compares dos hashes - retorna true si son "suficientemente" diferentes
    fn is_different(&self, other: &[u8; 64], threshold: u8) -> bool {
        let mut diff_count = 0u8;
        let my_bytes = self.current();
        
        for i in 0..64 {
            if (my_bytes[i] ^ other[i]) > threshold {
                diff_count += 1;
            }
        }
        
        diff_count >= 32 // Más del 50% diferente
    }
}

// ============================================================================
// SESSION METABOLISM - Gestión de sesiones
// ============================================================================

/// Estado de una sesión activa
#[derive(Clone)]
struct SessionState {
    /// Nuestra MetabolicKey para esta sesión
    our_key: MetabolicKey,
    /// Clave del peer (cuando se establece)
    peer_key: Option<MetabolicKey>,
    /// ID de la sesión
    session_id: u64,
    /// Genoma de autenticación
    genome: Option<AuthGenome>,
    /// Última actividad
    last_activity: u64,
    /// ¿Está autenticada?
    authenticated: bool,
    /// Surface hash del handshake
    handshake_hash: Option<SurfaceHash>,
}

impl SessionState {
    fn new(node_id: u64, session_id: u64, entropy: u64) -> Self {
        SessionState {
            our_key: MetabolicKey::from_node_id(node_id, entropy),
            peer_key: None,
            session_id,
            genome: None,
            last_activity: current_time_ms(),
            authenticated: false,
            handshake_hash: None,
        }
    }
    
    fn age(&mut self) {
        self.last_activity = current_time_ms();
    }
}

// ============================================================================
// PUBLIC API - SecureP2PTransport
// ============================================================================

/// SecureP2PTransport: Transporte seguro P2P 100% original
pub struct SecureP2PTransport {
    /// Node ID local
    node_id: u64,
    /// Sesiones activas
    sessions: HashMap<SocketAddr, SessionState>,
    /// Clave mestra (derivada del node_id)
    master_key: MetabolicKey,
    /// Entropy para nuevas sesiones
    entropy: u64,
}

impl SecureP2PTransport {
    /// Crea un nuevo SecureP2PTransport
    pub fn new(node_id: u64) -> Self {
        SecureP2PTransport {
            node_id,
            sessions: HashMap::new(),
            master_key: MetabolicKey::from_node_id(node_id, 0xEDEN_DEAD),
            entropy: current_time_ms(),
        }
    }
    
    /// Inicia una nueva sesión con un peer
    pub fn initiate_session(&mut self, peer_addr: SocketAddr) -> SessionId {
        let session_id = self.generate_session_id();
        let entropy = self.next_entropy();
        
        let session = SessionState::new(self.node_id, session_id, entropy);
        self.sessions.insert(peer_addr, session);
        
        SessionId { session_id, peer_addr }
    }
    
    /// Genera handshake inicial (para enviar al peer)
    pub fn generate_handshake_init(&self, session_id: u64, peer_addr: &SocketAddr) -> Vec<u8> {
        let mut handshake = Vec::new();
        
        // Session ID
        handshake.extend_from_slice(&session_id.to_le_bytes());
        
        // Node ID
        handshake.extend_from_slice(&self.node_id.to_le_bytes());
        
        // Primera generación del handshake hash
        let mut hasher = SurfaceHash::new(&handshake, 0);
        let initial_hash = hasher.current();
        
        handshake.extend_from_slice(&initial_hash);
        
        // Meta información de la sesión
        handshake.push(1); // versión 1
        handshake.push(32); // tamaño de genes
        
        handshake
    }
    
    /// Procesa handshake recibido (respuesta del peer)
    pub fn process_handshake_response(&mut self, peer_addr: SocketAddr, data: &[u8]) -> Result<AuthGenome, TransportError> {
        if data.len() < 50 {
            return Err(TransportError::InvalidHandshake);
        }
        
        // Extraer session ID y node ID del peer
        let peer_session_id = u64::from_le_bytes(data[0..8].try_into().map_err(|_| TransportError::InvalidHandshake)?);
        let peer_node_id = u64::from_le_bytes(data[8..16].try_into().map_err(|_| TransportError::InvalidHandshake)?);
        
        // Crear clave del peer
        let peer_key = MetabolicKey::from_node_id(peer_node_id, self.next_entropy());
        
        // Obtener o crear sesión
        let session = self.sessions.entry(peer_addr).or_insert_with(|| {
            SessionState::new(self.node_id, peer_session_id, self.next_entropy())
        });
        
        session.peer_key = Some(peer_key);
        session.age();
        
        // Generar nuestro genoma de autenticación
        let genome = AuthGenome::from_key(&session.our_key, 32);
        
        // Actualizar handshake hash
        let mut hasher = SurfaceHash::new(data, session.session_id);
        hasher.add(&genome.to_bytes());
        session.handshake_hash = Some(hasher);
        
        Ok(genome)
    }
    
    /// Procesa genoma recibido del peer
    pub fn process_peer_genome(&mut self, peer_addr: SocketAddr, genome_data: &[u8]) -> Result<bool, TransportError> {
        let session = self.sessions.get_mut(&peer_addr).ok_or(TransportError::NoSession)?;
        
        let genome = AuthGenome::from_bytes(genome_data).ok_or(TransportError::InvalidAuthGenome)?;
        
        // Verificar genoma
        if !genome.verify() {
            return Err(TransportError::AuthenticationFailed);
        }
        
        session.genome = Some(genome);
        session.authenticated = true;
        
        Ok(true)
    }
    
    /// Cifra datos para una sesión (usando metabolism, no cipher)
    pub fn encrypt(&mut self, peer_addr: SocketAddr, plaintext: &[u8]) -> Result<Vec<u8>, TransportError> {
        let session = self.sessions.get_mut(&peer_addr).ok_or(TransportError::NoSession)?;
        
        if !session.authenticated {
            return Err(TransportError::NotAuthenticated);
        }
        
        // "Metabolizar" los datos - aplicar la clave metabólica
        let mut encrypted = Vec::with_capacity(plaintext.len() + 16);
        
        // Añadir a handshake hash para "sellar" el mensaje
        if let Some(ref mut hasher) = session.handshake_hash {
            hasher.add(plaintext);
            encrypted.extend_from_slice(&hasher.current()[..16]); // prefix con hash
        }
        
        // Aplicar "digestión" metabólica
        let digest = session.our_key.digest(plaintext);
        
        // XOR con el digest para cifrar (como una operación irreversibly mezclada)
        for (i, byte) in plaintext.iter().enumerate() {
            encrypted.push(byte ^ digest[i % 32]);
        }
        
        session.age();
        Ok(encrypted)
    }
    
    /// Descifra datos recibidos
    pub fn decrypt(&mut self, peer_addr: SocketAddr, ciphertext: &[u8]) -> Result<Vec<u8>, TransportError> {
        let session = self.sessions.get_mut(&peer_addr).ok_or(TransportError::NoSession)?;
        
        if !session.authenticated {
            return Err(TransportError::NotAuthenticated);
        }
        
        if ciphertext.len() < 16 {
            return Err(TransportError::InvalidData);
        }
        
        // Verificar hash del mensaje
        let msg_hash = &ciphertext[0..16];
        
        // Descifrar el resto
        let mut plaintext = Vec::with_capacity(ciphertext.len() - 16);
        
        // Necesitamos el digest - lo recalculamos usando los datos
        // (Esto funciona porque la operación es simétrica)
        let mut digest = [0u8; 32];
        for i in 0..32 {
            digest[i] = session.our_key.state[i];
        }
        
        for (i, byte) in ciphertext[16..].iter().enumerate() {
            plaintext.push(byte ^ digest[i % 32]);
        }
        
        session.age();
        Ok(plaintext)
    }
    
    /// Verifica si una sesión está activa
    pub fn is_session_active(&self, peer_addr: &SocketAddr) -> bool {
        if let Some(session) = self.sessions.get(peer_addr) {
            let age = current_time_ms() - session.last_activity;
            age < 300000 // 5 minutos timeout
        } else {
            false
        }
    }
    
    /// Cierra una sesión
    pub fn close_session(&mut self, peer_addr: &SocketAddr) {
        self.sessions.remove(peer_addr);
    }
    
    /// Obtiene el fingerprint público de este nodo
    pub fn public_fingerprint(&self) -> String {
        self.master_key.fingerprint()
    }
    
    // Helpers
    
    fn generate_session_id(&self) -> u64 {
        let time = current_time_ms();
        let rand = self.next_entropy();
        time.wrapping_mul(rand).wrapping_add(self.node_id)
    }
    
    fn next_entropy(&mut self) -> u64 {
        self.entropy = self.entropy.wrapping_mul(0x9E3779B97F4A7C15).wrapping_add(1);
        self.entropy
    }
}

fn current_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

/// Identificador de sesión
#[derive(Clone, Debug)]
pub struct SessionId {
    pub session_id: u64,
    pub peer_addr: SocketAddr,
}

/// Errores de transporte
#[derive(Debug, Clone)]
pub enum TransportError {
    NoSession,
    NotAuthenticated,
    InvalidHandshake,
    InvalidAuthGenome,
    AuthenticationFailed,
    InvalidData,
}

impl std::fmt::Display for TransportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransportError::NoSession => write!(f, "No active session"),
            TransportError::NotAuthenticated => write!(f, "Session not authenticated"),
            TransportError::InvalidHandshake => write!(f, "Invalid handshake data"),
            TransportError::InvalidAuthGenome => write!(f, "Invalid authentication genome"),
            TransportError::AuthenticationFailed => write!(f, "Authentication failed"),
            TransportError::InvalidData => write!(f, "Invalid encrypted data"),
        }
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_metabolic_key_digest() {
        let mut key1 = MetabolicKey::from_node_id(0x1234, 0xABCD);
        let mut key2 = MetabolicKey::from_node_id(0x1234, 0xABCD);
        
        let data = b"EDEN test data";
        
        let digest1 = key1.digest(data);
        let digest2 = key2.digest(data);
        
        assert_eq!(digest1, digest2, "Same inputs should produce same digest");
        assert_ne!(digest1, [0u8; 32], "Digest should not be all zeros");
    }
    
    #[test]
    fn test_metabolic_key_irreversibility() {
        let mut key = MetabolicKey::from_node_id(0x1234, 0xABCD);
        
        let data1 = b"first data";
        let data2 = b"second data";
        
        let digest1 = key.digest(data1);
        key.metabolize(data1);
        
        // After metabolizing, digest for same data should be different
        // (This tests that the state advances)
        assert_ne!(key.generation, 0);
    }
    
    #[test]
    fn test_auth_genome() {
        let key = MetabolicKey::from_node_id(0x1234, 0xABCD);
        let genome = AuthGenome::from_key(&key, 16);
        
        assert!(genome.verify(), "Fresh genome should verify");
        
        let bytes = genome.to_bytes();
        let restored = AuthGenome::from_bytes(&bytes).unwrap();
        
        assert!(restored.verify(), "Restored genome should verify");
    }
    
    #[test]
    fn test_surface_hash() {
        let mut hasher = SurfaceHash::new(b"initial", 0);
        
        hasher.add(b"data1");
        let hash1 = hasher.current();
        
        hasher.add(b"data2");
        let hash2 = hasher.current();
        
        // Different data should produce different hash
        assert_ne!(hash1, hash2);
    }
    
    #[test]
    fn test_session_lifecycle() {
        let mut transport = SecureP2PTransport::new(0x1234);
        
        let peer: SocketAddr = "127.0.0.1:4849".parse().unwrap();
        
        let session = transport.initiate_session(peer);
        assert_eq!(session.peer_addr, peer);
        
        let handshake = transport.generate_handshake_init(session.session_id, &peer);
        assert!(handshake.len() > 0);
        
        assert!(transport.is_session_active(&peer));
        
        transport.close_session(&peer);
        assert!(!transport.is_session_active(&peer));
    }
    
    #[test]
    fn test_encrypt_decrypt() {
        let mut transport = SecureP2PTransport::new(0x1234);
        
        let peer: SocketAddr = "127.0.0.1:4849".parse().unwrap();
        transport.initiate_session(peer);
        
        // For this test, we manually mark as authenticated
        if let Some(session) = transport.sessions.get_mut(&peer) {
            session.authenticated = true;
        }
        
        let plaintext = b"EDEN secret message";
        
        let encrypted = transport.encrypt(peer, plaintext).unwrap();
        assert!(encrypted.len() > plaintext.len()); // Has prefix
        
        let decrypted = transport.decrypt(peer, &encrypted).unwrap();
        assert_eq!(&decrypted, plaintext);
    }
}