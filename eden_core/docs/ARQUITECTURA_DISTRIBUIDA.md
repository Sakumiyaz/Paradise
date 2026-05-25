# EDEN Distribuido: Especificación de Arquitectura

> *"Diseño por Intent: Una red de vida artificial donde cada peer es un universo completo que puede comunicar-se con otros."*

---

## 1. Principios de Diseño

### 1.1 Filosofía Distribuida

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    ARQUITECTURA PEER-TO-PEER PURA                       │
│                                                                          │
│   ┌─────────┐     ┌─────────┐     ┌─────────┐     ┌─────────┐            │
│   │ EDEN A  │─────│ EDEN B  │─────│ EDEN C  │─────│ EDEN D  │            │
│   │Univ-0   │     │Univ-1   │     │Univ-2   │     │Univ-3   │            │
│   │         │     │         │     │         │     │         │            │
│   │ 10000   │     │  8000   │     │ 12000   │     │  9000   │            │
│   │ Autons  │     │ Autons  │     │ Autons  │     │ Autons  │            │
│   └─────────┘     └─────────┘     └─────────┘     └─────────┘            │
│                                                                          │
│   Cada peer tiene UN UNIVERSO COMPLETO.                                  │
│   La "distribución" es compartir estado, no fragmentar.                 │
└─────────────────────────────────────────────────────────────────────────┘
```

### 1.2 Invariantes Críticas

| Invariante | Descripción | Por qué importa |
|------------|-------------|-----------------|
| `DETERMINISM_COMMITMENT` | Mismo seed → mismo estado | Reproducibilidad científica |
| `AUTON_COMPLETENESS` | Todo Auton tiene toda su información | Autopoiesis local |
| `MAR_CONSISTENCY` | Grid del Mar refleja todos los cambios | Simetría espacial |
| `EVENTUAL_CONVERGENCE` | Todos los peers convergen eventual | Consistencia global |

### 1.3 Trade-offs Aceptados

| Trade-off | Aceptar | Razón |
|-----------|---------|-------|
| Latencia de sync | 50-500ms | Emergencia no requiere real-time |
| Eventual consistency | Sí | CP不适配, AP更适合 |
| Split brain | Temporal, okay | Autoresolución preferida a bloqueo |
| Network partition | Okay si <1% Autons perdidos | Agotamiento natural reemplaza |

---

## 2. Modelo de Consistencia

### 2.1 Modelo Eventual-Positivo (EPO)

```
┌─────────────────────────────────────────────────────────────────┐
│                 MODELO EPO (Eventual-Positivo)                  │
│                                                                  │
│  Cada tick, un peer:                                            │
│  1. Procesa su estado local                                      │
│  2. Genera "diff" de cambios                                     │
│  3. Comparte diff con peers                                     │
│  4. Aplica diffs recibidos cuando son beneficiosos             │
│                                                                  │
│  "Positivo" = El sistema tiende a aceptar cambios, no rechazarlos│
│  "Eventual" = No se garantiza instantaneidad                    │
└─────────────────────────────────────────────────────────────────┘
```

### 2.2 Vector de Estado (StateVector)

```rust
/// Identificador de estado con vector de ticks por peer
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct StateVector {
    /// tick local del peer
    pub tick: u64,
    /// vector de timestamps de última sincronización
    pub peer_ticks: HashMap<PeerId, u64>,
    /// hash del estado completo (para validación rápida)
    pub state_hash: u64,
}

impl StateVector {
    /// Compara dos StateVectors para determinar sync priority
    pub fn needs_sync(&self, other: &StateVector) -> bool {
        // Sync si:
        // 1. Mi tick es menor que el del otro → necesito actualizaciones
        // 2. Tengo gaps en peer_ticks → me faltan sincronizaciones
        // 3. Mi state_hash difiere → hay divergencia
        
        self.tick < other.tick 
        || !self.has_all_peer_timestamps(other)
        || self.state_hash != other.state_hash
    }
    
    /// Verifica si tengo todas las marcas de agua de otro
    fn has_all_peer_timestamps(&self, other: &StateVector) -> bool {
        other.peer_ticks.iter().all(|(peer, tick)| {
            self.peer_ticks.get(peer).map(|t| t >= tick).unwrap_or(false)
        })
    }
}
```

### 2.3 Sincronización Diferencial

```rust
/// Delta de estado entre dos StateVectors
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StateDiff {
    pub from_tick: u64,
    pub to_tick: u64,
    pub new_autons: Vec<AutonVivo>,
    pub dead_autons: Vec<AutonId>,
    pub mar_changes: Vec<MarChange>,
    pub meltrace_new: Vec<Grabado>,
    pub claims_update: Vec<CellClaim>,
}

pub enum MarChange {
    /// (x, y, z, old_density, new_density)
    DensityChange(usize, usize, usize, I32F32, I32F32),
    /// (x, y, z) - celda colonizada
    Colonized(usize, usize, usize, AutonId),
    /// (x, y, z) - celda abandonada
    Abandoned(usize, usize, usize),
}

/// Snapshot completo para bootstrapping
#[derive(Clone, Serialize, Deserialize)]
pub struct FullSnapshot {
    pub version: u8,
    pub universos_id: UniversosId,
    pub state_vector: StateVector,
    pub timestamp: u64,
    pub autons: Vec<SerializedAuton>,
    pub mar_grid: Vec<SerializedCeldaMar>,
    pub meltrace: Vec<SerializedGrabado>,
    pub claims: Vec<CellClaim>,
}
```

---

## 3. Wire Protocol v1

### 3.1 Formato de Mensaje

```
┌─────────────────────────────────────────────────────────────────┐
│                    WIRE PROTOCOL v1                              │
│                                                                  │
│  ┌──────────┬──────────┬──────────┬──────────┬──────────┐        │
│  │ Version  │  Type    │ Length   │ Payload  │ Checksum│        │
│  │  (1 B)   │  (1 B)   │  (4 B)   │  (N B)   │  (4 B)  │        │
│  └──────────┴──────────┴──────────┴──────────┴──────────┘        │
│                                                                  │
│  Version: 0x01 (protocolo actual)                               │
│  Type: MESSAGE_TYPE enum                                         │
│  Length: payload length (big-endian u32)                         │
│  Payload: serialized protobuf/MessagePack/codegen                │
│  Checksum: CRC32 de payload para integridad                      │
└─────────────────────────────────────────────────────────────────┘
```

### 3.2 Tipos de Mensaje

```rust
#[repr(u8)]
pub enum MessageType {
    // Control
    Handshake     = 0x01,
    HandshakeAck   = 0x02,
    Heartbeat      = 0x03,
    Disconnect     = 0x04,
    
    // Sincronización de estado
    StateDiff      = 0x10,
    FullSnapshot   = 0x11,
    SyncRequest    = 0x12,
    SyncResponse   = 0x13,
    
    // Eventos de Autons
    AutonBorn      = 0x20, // Nuevo Auton nacido (escisión)
    AutonDead      = 0x21, // Auton murió
    AutonMigrated  = 0x22, // Auton se mudó a otro peer
    
    // Eventos del Mar
    MarDelta       = 0x30, // Cambios en densidades del Mar
    ClaimUpdate    = 0x31, // Nuevo/revisado claim territorial
    
    // Consenso
    Proposal       = 0x40, // Propuesta de bifurcación global
    Vote           = 0x41, // Voto sobre propuesta
    ConsensusReached = 0x42, // Consenso达成
    
    // Metadata
    Ping           = 0xF0,
    Pong           = 0xF1,
    Error          = 0xFF,
}
```

### 3.3 Handshake Protocol

```
Peer A                              Peer B
  │                                    │
  │──── Handshake (version, peer_id) ─►│
  │                                    │
  │◄── HandshakeAck (version, peer_id) ─│
  │      (si compatible)               │
  │                                    │
  │──── StateVector (tick, hash) ──────►│
  │                                    │
  │◄── StateDiff (deltas) ──────────────│
  │      (si A está desactualizado)    │
  │                                    │
  │──── StateVectorAck ────────────────►│
  │      (confirma sync)               │
  │                                    │
  ▼ Conexión establecida ▼
```

### 3.4 Payload Serialization

```rust
// Prioridad 1: wire-v1 local (determinista, auditado, sin dependencias)
// Prioridad 2: rkyv/protobuf solo si aparece un requisito externo concreto
// Evitar codecs binarios no versionados para gossip operativo.

#[cfg(feature = "use_rkyv")]
mod serialization {
    use rkyv::{Archive, Serialize, Deserialize};
    
    #[derive(Archive, Serialize, Deserialize)]
    pub struct NetworkMessage {
        pub version: u8,
        pub msg_type: MessageType,
        pub sender_id: PeerId,
        pub timestamp: u64,
        pub payload: Vec<u8>,
        pub checksum: u32,
    }
}
```

---

## 4. Gestión de peers

### 4.1 PeerTable

```rust
/// Estado de un peer conocido
#[derive(Clone)]
pub struct PeerEntry {
    pub peer_id: PeerId,
    pub address: SocketAddr,
    pub last_seen: u64,
    pub state_vector: StateVector,
    pub connection_state: ConnectionState,
    pub failure_count: u8,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Handshaking,
    Connected,
    Degraded, // Conexión lenta o con perdidas
}

pub struct PeerTable {
    /// Myself
    pub self_id: PeerId,
    
    /// Todos los peers conocidos (incluye seeds)
    entries: HashMap<PeerId, PeerEntry>,
    
    /// Conexiones activas
    active_connections: HashMap<PeerId, Connection>,
    
    /// Semillas iniciales (peers hardcoded)
    seeds: Vec<SocketAddr>,
    
    /// Configuración
    config: PeerTableConfig,
}

impl PeerTable {
    /// Connecta a un peer known (dirección conocida)
    pub async fn connect(&mut self, peer_id: PeerId) -> Result<()> {
        let entry = self.entries.get(&peer_id)
            .ok_or(Error::PeerUnknown)?;
        
        if entry.connection_state == ConnectionState::Connected {
            return Ok(()); // Ya conectado
        }
        
        let conn = TcpStream::connect(entry.address).await?;
        self.initiate_handshake(&mut conn, peer_id).await?;
        
        Ok(())
    }
    
    /// Conecta a un nuevo peer (address nuevo)
    pub async fn add_peer(&mut self, address: SocketAddr) -> Result<PeerId> {
        // Generar ID temporal hasta handshake
        let temp_id = PeerId::generate_temp();
        
        let conn = TcpStream::connect(address).await?;
        let (peer_id, state_vector) = self.complete_handshake(conn, temp_id).await?;
        
        self.entries.insert(peer_id, PeerEntry {
            peer_id,
            address,
            last_seen: now(),
            state_vector,
            connection_state: ConnectionState::Connected,
            failure_count: 0,
        });
        
        Ok(peer_id)
    }
    
    /// Bootstrap desde semillas
    pub async fn bootstrap(&mut self) -> Result<()> {
        for seed_addr in &self.seeds {
            match self.add_peer(*seed_addr).await {
                Ok(_) => break, // Una semilla够了
                Err(e) => {
                    eprintln!("Seed {} failed: {}", seed_addr, e);
                    continue;
                }
            }
        }
        Ok(())
    }
}
```

### 4.2 Flooding (Gossip Protocol)

```rust
/// Simple gossip para propagar eventos
pub struct GossipLayer {
    /// Mensajes ya vistos (evitar loops)
    seen_messages: LruCache<u64, ()>,
    
    /// Fan-out para flooding
    fan_out: usize,
    
    /// TTL de mensajes
    ttl: u8,
}

impl GossipLayer {
    /// Envía mensaje a todos los peers (excepto origen)
    pub fn flood(&mut self, msg: &NetworkMessage, exclude: Option<PeerId>) {
        let msg_id = msg.message_id();
        
        // Evitar loops
        if self.seen_messages.contains_key(&msg_id) {
            return;
        }
        self.seen_messages.insert(msg_id, ());
        
        // Enviar a todos los peers activos
        for (peer_id, _) in self.active_peers() {
            if Some(peer_id) != exclude {
                self.send_to(peer_id, msg.clone());
            }
        }
        
        // Reducir TTL
        if self.ttl > 0 {
            self.ttl -= 1;
        }
    }
}
```

---

## 5. Sincronización de Estado

### 5.1 Sync Manager

```rust
/// Orchestrates synchronization between peers
pub struct SyncManager {
    /// Peer table
    peer_table: Arc<RwLock<PeerTable>>,
    
    /// Local state
    local_state: Arc<RwLock<Universo>>,
    
    /// Pending diffs por aplicar
    pending_diffs: VecDeque<StateDiff>,
    
    /// Sync configuration
    config: SyncConfig,
}

impl SyncManager {
    /// Called each tick to sync state
    pub async fn tick_sync(&mut self) -> Result<()> {
        // 1. Recolectar state vectors de peers
        let peer_vectors = self.gather_peer_vectors().await?;
        
        // 2. Determinar quién necesita sync
        let local_tick = self.local_state.read().unwrap().contador_ciclos;
        
        for (peer_id, peer_vector) in peer_vectors {
            if peer_vector.tick > local_tick {
                // Peer está adelante, necesito su diff
                self.request_sync(peer_id, local_tick).await?;
            } else if peer_vector.tick < local_tick {
                // Yo estoy adelante, enviar diff
                self.send_diff(peer_id).await?;
            }
            
            // 3. Merge de diffs pendientes
            self.apply_pending_diffs()?;
        }
        
        Ok(())
    }
    
    /// Genera diff desde tick_up to current
    fn generate_diff(&self, since_tick: u64) -> StateDiff {
        let local = self.local_state.read().unwrap();
        
        StateDiff {
            from_tick: since_tick,
            to_tick: local.contador_ciclos,
            new_autons: local.autons.iter()
                .filter(|a| a.campo.tick > since_tick)
                .cloned()
                .collect(),
            dead_autons: vec![], // De local tracking
            mar_changes: vec![], // De tracking del Mar
            meltrace_new: local.meltrace.grabados_since(since_tick),
            claims_update: vec![], // De tracking
        }
    }
}
```

### 5.2 Conflict Resolution

```rust
/// Conflict resolver para datos distribuidos
pub struct ConflictResolver {
    strategy: ConflictStrategy,
}

pub enum ConflictStrategy {
    /// Last-write-wins con timestamp vector
    LWW,
    /// Merge automático (máximo de energías, unión de Autons)
    Merge,
    /// Consenso entre peers (para decisiones críticas)
    Consensus,
}

impl ConflictResolver {
    /// Resuelve conflicto de Auton (dos peers tienen el mismo ID)
    pub fn resolve_auton(&self, local: &AutonVivo, remote: &AutonVivo) -> AutonVivo {
        match self.strategy {
            ConflictStrategy::LWW => {
                // El que tiene mayor tick gana
                if local.campo.tick >= remote.campo.tick {
                    local.clone()
                } else {
                    remote.clone()
                }
            }
            ConflictStrategy::Merge => {
                // Merge: tomar lo mejor de cada uno
                AutonVivo {
                    campo: if local.campo.tick >= remote.campo.tick {
                        local.campo.clone()
                    } else {
                        remote.campo.clone()
                    },
                    energia: local.energia.max(remote.energia),
                    // RamNet y Umbra: merge de memorias
                    ramnet: local.ramnet.merge(&remote.ramnet),
                    umbra: local.umbra.merge(&remote.umbra),
                    // ID del más reciente
                    id: if local.campo.tick >= remote.campo.tick {
                        local.id
                    } else {
                        remote.id
                    },
                }
            }
            ConflictStrategy::Consensus => todo!(),
        }
    }
    
    /// Resuelve conflicto de claim territorial
    pub fn resolve_claim(&self, local: CellClaim, remote: CellClaim) -> CellClaim {
        // Energía mayor gana (más stake en el territorio)
        if local.energia >= remote.energia {
            local
        } else {
            remote
        }
    }
}
```

---

## 6. NAT Traversal

### 6.1 NAT Strategy Selection

```
┌─────────────────────────────────────────────────────────────────┐
│                    NAT TRAVERSAL STRATEGY                       │
│                                                                  │
│  Detectar tipo de NAT:                                          │
│  ┌───────────────────────────────────────────────────────┐      │
│  │ 1. STUN server (si disponible) → determinar tipo       │      │
│  │                                                      │      │
│  │ 2. Si open internet → directo TCP                    │      │
│  │ 3. Si symmetric → necesitan relay                    │      │
│  │ 4. Si port-restricted → intentar_hole_punching       │      │
│  └───────────────────────────────────────────────────────┘      │
│                                                                  │
│  Relay como fallback:                                           │
│  - Un peer con IP pública actúa como relay                      │
│  - Traffic relay through relay peer                             │
│  - Relay peer puede ser cualquier peer, no dedicado            │
└─────────────────────────────────────────────────────────────────┘
```

### 6.2 Relay Protocol

```rust
/// Peer que actúa como relay
pub struct RelayPeer {
    pub peer_id: PeerId,
    pub public_addr: SocketAddr,
    pub relayed_peers: HashSet<PeerId>,
}

impl RelayPeer {
    /// Forward mensaje a peer tras NAT
    pub async fn relay_message(&mut self, to: PeerId, msg: NetworkMessage) -> Result<()> {
        // Guardar mensaje para retrieval
        self.pending_messages.insert(to, msg);
        
        // Notificar al peer que tiene mensajes pendientes
        // El peer tras NAT pollingará o recibirá callback
        Ok(())
    }
    
    /// Retrieve mensajes pendientes
    pub fn retrieve_messages(&mut self, peer_id: &PeerId) -> Vec<NetworkMessage> {
        self.pending_messages.remove(peer_id).unwrap_or_default()
    }
}
```

---

## 7. Modelo de Fault Tolerance

### 7.1 Fallo Detection

```rust
/// Tracking de health de peers
pub struct PeerHealth {
    pub peer_id: PeerId,
    pub consecutive_failures: u8,
    pub last_heartbeat: u64,
    pub is_dead: bool,
}

impl PeerHealth {
    /// Check if peer should be considered dead
    pub fn is_unreachable(&self, now: u64, timeout: u64) -> bool {
        self.is_dead 
        || self.consecutive_failures >= MAX_FAILURES
        || now - self.last_heartbeat > timeout
    }
}

/// Configuración de fault tolerance
pub struct FaultConfig {
    /// Máximo de fallos consecutivos antes de marcar dead
    pub max_failures: u8,
    /// Timeout de heartbeat en ms
    pub heartbeat_timeout_ms: u64,
    /// Intervalo de heartbeat en ms
    pub heartbeat_interval_ms: u64,
    /// Intentos de reconexión
    pub max_reconnect_attempts: u8,
}
```

### 7.2 Recovery

```rust
impl SyncManager {
    /// Maneja peer marcado como dead
    pub async fn handle_peer_death(&mut self, peer_id: PeerId) {
        // 1. Limpiar conexión
        self.peer_table.write().unwrap().remove_peer(peer_id);
        
        // 2. El estado del peer dead se preserva en snapshots
        // No se pierde información, solo se redistribuye el trabajo
        
        // 3. Notificar a otros peers (gossip)
        let msg = NetworkMessage::peer_departed(peer_id);
        self.gossip_layer.flood(&msg, None);
        
        // 4. Opcional: re-balancear carga si hay imbalance
    }
}
```

---

## 8. Testing Infrastructure

### 8.1 Mock Network

```rust
/// Simula red con propiedades controladas
pub struct MockNetwork {
    pub latency_ms: u64,
    pub packet_loss: f32,
    pub partitions: Vec<HashSet<PeerId>>,
}

impl MockNetwork {
    /// Simula partición de red
    pub fn partition(&mut self, group_a: &[PeerId], group_b: &[PeerId]) {
        self.partitions.push(group_a.iter().cloned().collect());
        self.partitions.push(group_b.iter().cloned().collect());
    }
    
    /// Check si dos peers pueden comunicarse
    pub fn can_send(&self, from: PeerId, to: PeerId) -> bool {
        // Check partitions
        for partition in &self.partitions {
            if partition.contains(&from) && !partition.contains(&to) {
                return false;
            }
        }
        // Check packet loss
        random() < self.packet_loss
    }
}
```

### 8.2 Chaos Testing

```rust
/// Inyecta fallos para probar resilience
pub struct ChaosMonkey {
    pub kill_probability: f32,
    pub network_partition_probability: f32,
    pub latency_injection_probability: f32,
}

impl ChaosMonkey {
    /// Decide si caos ocurre este tick
    pub fn should_chaos(&self) -> bool {
        random::<f32>() < self.kill_probability
    }
    
    /// Aplica caos
    pub async fn inject_chaos(&mut self, peers: &mut [Peer]) {
        if random::<f32>() < self.network_partition_probability {
            // Particionar red
            let midpoint = peers.len() / 2;
            // ... partition logic
        }
        
        if random::<f32>() < self.latency_injection_probability {
            // Injectar latency
            // ... latency logic
        }
    }
}
```

---

## 9. API de Usuario

### 9.1 CLI

```bash
# Modo single-instance (comportamiento actual)
eden --seed [SEED] --ticks [N]

# Modo distribuido
eden --dist --peers 5 --port 9876

# Modo peer (se conecta a otros)
eden --peer --connect localhost:9876,localhost:9877

# Modo seed (solo bootstrapping)
eden --seed-mode --port 9876

# Hybrid (single + listening for peers)
eden --seed [SEED] --listen 9876 --connect [PEERS]
```

### 9.2 IPC Commands

```rust
// Comandos vía Unix socket
pub enum IpCommand {
    GetState,
    GetAutons,
    GetPeerList,
    Connect(String), // address
    Disconnect(PeerId),
    BroadcastSnapshot,
    Shutdown,
}
```

---

## 10. Milestones de Implementación

| Fase | Descripción | Criteria | Timeline |
|------|-------------|----------|----------|
| P0 | Serialización básica | freeze/thaw works | Week 1 |
| P1 | TCP básico | peer A connects to B | Week 2 |
| P2 | Handshake | peers exchange state vectors | Week 3 |
| P3 | Sync básico | diffs propagate | Week 4 |
| P4 | Conflict resolution | LWW works | Week 5 |
| P5 | Gossip | events flood | Week 6 |
| P6 | NAT traversal | symmetric NAT works | Week 7 |
| P7 | Consensus | basic voting | Week 8 |
| P8 | Stress test | 10 peers, 100k ticks | Week 9-10 |
| P9 | Production hardening | docs, error handling | Week 11-12 |

---

*Especificación sujetos a cambios según feedback de implementación.*
