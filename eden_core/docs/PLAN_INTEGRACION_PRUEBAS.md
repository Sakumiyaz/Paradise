# EDEN Distribuido: Plan de Integración y Pruebas

## Filosofía del Plan

> "Un sistema distribuido de vida artificial debe probarse como cualquier sistema distribuido crítico, pero validarse como un organismo vivo: no por lo que hace, sino por lo que emerge."

---

## FASE 0: Pruebas de Congelación (Semana 1-2)

### Objetivo: Validar que la serialización no rompe el determinismo

```text
┌────────────────────────────────────────────────────────────────────┐
│                    PRUEBA DE CONGELACIÓN                            │
│                                                                    │
│  Instancia A ──tick 0-100──> Snapshot ──> Instancia B              │
│                                                                    │
│  Validación: tick(A,100) == tick(B,100)                            │
│  Ambas corren con identical_seed = [0x42; 128]                      │
└────────────────────────────────────────────────────────────────────┘
```

### Tests de Congelación

| Test | Descripción | Criterio Aceptación |
|------|-------------|---------------------|
| `test_freeze_identity` | Serializar/deserialize estado 100 ticks | Estado final idéntico |
| `test_freeze_autons` | Verificar que todos los Autons preservan ID | Hash estable |
| `test_freeze_mar` | Validar grid del Mar Morfóseo | Densidades exactas |
| `test_freeze_ramnet` | Verificar memorias RamNet | Contenido bit-exacto |
| `test_freeze_umbra` | Validar DAG de decisiones | Arquitectura de arcos preservada |

### Implementación

```rust
// src/network/tests/freeze_tests.rs

#[test]
fn test_freeze_identity() {
    let seed = [0x42u8; 128];
    let mut universo_a = Universo::crear(seed).unwrap();
    
    // Correr 100 ticks en A
    for _ in 0..100 {
        universo_a.tick();
    }
    
    // Congelar
    let snapshot = universo_a.freeze().unwrap();
    
    // Descongelar en B
    let universo_b = Universo::thaw(snapshot).unwrap();
    
    // Validar determinismo
    let hash_a = universo_a.state_hash();
    let hash_b = universo_b.state_hash();
    
    assert_eq!(hash_a, hash_b, "Congelación violó determinismo");
}

// Verificación bit-exacto
fn verify_state_equivalence(a: &Universo, b: &Universo) -> bool {
    a.autons.len() == b.autons.len()
    && a.mar.energon_total() == b.mar.energon_total()
    && a.meltrace.len() == b.meltrace.len()
    // ... más comparaciones
}
```

---

## FASE 1: Pruebas de Conectividad Local (Semana 2-3)

### Objetivo: Dos instancias en la misma máquina intercambian mensajes

```
┌─────────────────────┐          ┌─────────────────────┐
│  EDEN Instance A    │◄─────────►│  EDEN Instance B    │
│  Puerto 9876        │   TCP     │  Puerto 9877        │
│  ID: peer_0xA1      │          │  ID: peer_0xB2      │
└─────────┬───────────┘          └─────────┬───────────┘
          │                                │
          └──────────┬─────────────────────┘
                     │
              ┌──────▼──────┐
              │  Wire Format│
              │  Protocol v1│
              └─────────────┘
```

### Tests de Conectividad

| Test | Descripción | Timeout |
|------|-------------|---------|
| `test_tcp_handshake` | Conexión TCP bidireccional | 5s |
| `test_peer_discovery` | Intercambio de IDs y addresses | 10s |
| `test_heartbeat` | Keepalive cada 5s | 30s |
| `test_flood_fill` | Bootstrap de peers conocidos | 15s |
| `test_address_reconnection` | Reconexión tras desconexión | 20s |

### Implementación

```rust
// src/network/tests/connectivity_tests.rs

const TIMEOUT_HANDSHAKE: Duration = Duration::from_secs(5);
const TIMEOUT_DISCOVERY: Duration = Duration::from_secs(10);
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

#[tokio::test]
async fn test_tcp_handshake() {
    // Arrange:dos peers escuchando en puertos locales
    let (peer_a, peer_b) = spawn_local_peers().await;
    
    // Act: conectar A->B
    let result = peer_a.connect(peer_b.listen_addr()).await;
    
    // Assert: handshake completado
    assert!(result.is_ok());
    assert_eq!(peer_a.state(), PeerState::Connected);
    assert_eq!(peer_b.state(), PeerState::Connected);
}

#[tokio::test]
async fn test_heartbeat() {
    let (peer_a, peer_b) = spawn_local_peers().await;
    peer_a.connect(peer_b.listen_addr()).await.unwrap();
    
    // Enviar heartbeats por 30s
    let start = Instant::now();
    while start.elapsed() < Duration::from_secs(30) {
        peer_a.send_heartbeat();
        peer_b.send_heartbeat();
        tokio::time::sleep(HEARTBEAT_INTERVAL).await;
    }
    
    // Verificar que ambos siguen vivos
    assert!(peer_a.is_alive());
    assert!(peer_b.is_alive());
}
```

---

## FASE 2: Pruebas de Sincronización (Semana 3-4)

### Objetivo: Dos instancias convergen al mismo estado

```
Tick 0    Tick 50   Tick 100  Tick 150  Tick 200
  │         │         │         │         │
  ▼         ▼         ▼         ▼         ▼
[ A ]──────[ A ]──────[ A ]──────[ A ]──────[ A ]
  │         │         │         │         │
  │    ┌────┴─────────┴─────────┴────┐    │
  │    │  Intercambio de snapshots   │    │
  │    │  Merge de Autons nuevos     │    │
  │    └─────────────────────────────┘    │
  │         │         │         │         │
  ▼         ▼         ▼         ▼         ▼
[ B ]──────[ B ]──────[ B ]──────[ B ]──────[ B ]
```

### Tests de Sincronización

| Test | Descripción | Criterio Aceptación |
|------|-------------|---------------------|
| `test_sync_autons` | Intercambio de Autons nuevos | Sin duplicados |
| `test_sync_mar` | Merge de energon del Mar | Densidad converge |
| `test_sync_claims` | Resolución de conflictos de territorio | Un owner por celda |
| `test_divergence_recovery` | Reconvergencia tras split brain | Hash igual en 5 ticks |
| `test_concurrent_escision` | Dos peers escinden simultáneamente | No corruption |

### Conflict Resolution Strategy

```rust
// src/network/sync.rs

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ConflictResolution {
    /// Timestamp más antiguo gana (first-write-wins)
    TimestampAntiguo,
    /// Timestamp más nuevo gana (last-write-wins)
    TimestampReciente,
    /// Quien tiene más energía gana
    EnergiaMayor,
    /// Merge: ambos coexisten si es posible
    Merge,
}

impl UniversosSincronizados {
    /// Resuelve conflictos de claims de celdas del Mar
    fn resolver_conflicto_claim(
        claim_a: &CellClaim,
        claim_b: &CellClaim,
        modo: ConflictResolution,
    ) -> CellClaim {
        match modo {
            ConflictResolution::TimestampAntiguo => {
                if claim_a.timestamp < claim_b.timestamp {
                    *claim_a
                } else {
                    *claim_b
                }
            }
            ConflictResolution::EnergiaMayor => {
                if claim_a.energia >= claim_b.energia {
                    *claim_a
                } else {
                    *claim_b
                }
            }
            ConflictResolution::Merge => {
                // Combinar: tomar el max de cada campo
                CellClaim {
                    owner_id: claim_a.owner_id, // A gana por default
                    energia: claim_a.energia.max(claim_b.energia),
                    timestamp: claim_a.timestamp.min(claim_b.timestamp),
                    // ... merge resto
                }
            }
        }
    }
}
```

---

## FASE 3: Pruebas de Topología (Semana 4-6)

### Topología mesh pequeña (3-5 nodos)

```
    [A]
   / | \
  B  C  D
   \ | /
    [E]
```

### Tests de Topología

| Test | Descripción | Escala |
|------|-------------|--------|
| `test_mesh_partial_connectivity` | 3 peers, solo A conecta a B y C | 3 nodos |
| `test_flood_propagation` | Mensaje llega a todos desde cualquier origen | 5 nodos |
| `test_partition_heal` | Red recuperada tras split | 5 nodos |
| `test_byzantine_node` | Peer envía datos corruptos | 3 nodos |
| `test_node_departure` | Peer abandona la red graceful | 3 nodos |

### Simulador de Red

```rust
// src/network/tests/topology_sim.rs

/// Simula condiciones de red adversas
pub struct NetworkSimulator {
    /// Probabilidad de drop (0.0 = sin drops, 1.0 = todo drop)
    pub packet_loss_rate: f32,
    /// Latencia base en ms
    pub base_latency_ms: u64,
    /// Variación de latencia (±jitter)
    pub jitter_ms: u64,
    /// partición activa
    pub partition: Option<(usize, usize)>,
}

impl NetworkSimulator {
    /// Simula partición de red entre dos subconjuntos
    pub fn partition(&mut self, subset_a: &[PeerId], subset_b: &[PeerId]) {
        self.partition = Some((subset_a.len(), subset_b.len()));
    }
    
    /// Simula recuperación de partición
    pub fn heal(&mut self) {
        self.partition = None;
    }
}
```

---

## FASE 4: Pruebas de Estrés (Semana 6-8)

### Escenario: 10 instancias, 10,000 Autons, 1,000,000 ticks

```
Instancias: 10
Autons por instancia: ~1,000 (distribución no uniforme)
Ticks totales: 1,000,000
Mar grid: 64x64x8 (32,768 celdas)
Eventos de red: ~50,000
```

### Tests de Estrés

| Test | Descripción | Métrica Objetivo |
|------|-------------|------------------|
| `test_10_instances_10k_autons` | Carga real de producción | <5% packet loss |
| `test_100k_ticks_convergence` | 100k ticks con sync | T convergence < 50 ticks |
| `test_memory_stability` | Sin memory leaks en 1M ticks | <1MB growth |
| `test_cpu_scaling` | Uso CPU escalona linealmente | <80% por instancia |
| `test_network_bandwidth` | Ancho de banda estable | <10MB/min por peer |

### Stress Test Implementation

```rust
// src/network/tests/stress_tests.rs

const TARGET_TICKS: u64 = 100_000;
const TARGET_INSTANCES: usize = 10;
const TARGET_AUTONS: usize = 10_000;

#[tokio::test(flavor = "multi_thread", workers = 12)]
async fn test_10_instances_10k_autons() {
    // Arrange: crear 10 instancias con IDs únicos
    let instances: Vec<_> = (0..TARGET_INSTANCES)
        .map(|i| {
            let mut seed = [0u8; 128];
            seed[0] = i as u8; // Semilla única por instancia
            let universo = Universo::crear(seed).unwrap();
            let peer_id = PeerId::generate(format!("peer_{}", i));
            (universo, peer_id)
        })
        .collect();
    
    // Spawn todas las instancias en tasks concurrentes
    let handles: Vec<_> = instances
        .into_iter()
        .map(|(universo, peer_id)| {
            tokio::spawn(async move {
                run_instance(universo, peer_id).await
            })
        })
        .collect();
    
    // Esperar y validar
    let results = futures::future::join_all(handles).await;
    
    // Verificar que no hubo crashes
    assert!(results.iter().all(|r| r.is_ok()));
    
    // Verificar convergencia
    verify_global_convergence().await;
}

async fn verify_global_convergence() {
    // Obtener hash de estado de cada peer
    let hashes: Vec<u64> = get_all_peer_states().await;
    
    // Verificar que divergen poco (50% de hashes iguales = éxito)
    let unique_hashes: HashSet<_> = hashes.iter().collect();
    let convergence_ratio = unique_hashes.len() as f32 / hashes.len() as f32;
    
    assert!(
        convergence_ratio > 0.5,
        "Convergencia insuficiente: {:.1}%",
        convergence_ratio * 100.0
    );
}
```

---

## FASE 5: Pruebas de Emergencia (Semana 8-10)

### Objetivo: Validar que la distribución NO mata la emergencia

```
Hipótesis: La distribución debe ser transparente al comportamiento emergente.
Si la distribución rompe la emergencia, el diseño está mal.
```

### Tests de Emergencia

| Test | Descripción | Validación |
|------|-------------|------------|
| `test_escision_rate` | Tasa de escisión en distribuido vs single | ±10% |
| `test_auton_lifespan` | Distribución de lifespan de Autons | Distribución similar |
| `test_meltrace_content` | Contenido de Meltrace (grabados) | Calidad similar |
| `test_emergence_patterns` | Patrones emergentes (clusters, ondas) | Visualmente igual |
| `test_economic_cycles` | Ciclos de energía (auge/colapso) | Frecuencia similar |

### Validación Visual (Manual)

```bash
# Correr instancia única y guardar screenshot tick 1000
cargo run --release -- --screenshot single_tick1000.png

# Correr 5 instancias distribuidas y guardar screenshot tick 1000
cargo run --release --dist --peers 5 --screenshot dist_tick1000.png

# Comparar visualmente con diff
diff -u single_tick1000.png dist_tick1000.png
```

---

## FASE 6: Pruebas de Consenso (Semana 10-12)

### Sistema de Consenso para Bifurcaciones

```
┌─────────────────────────────────────────────────────────────┐
│                    PROTOCOLO DE CONSENSO                    │
│                                                              │
│  1. Peer propone bifurcación (via Broadcast)                │
│  2. Todos los peers votan (accept/reject)                   │
│  3. Si >50% acceptan → ejecutar bifurcación                 │
│  4. Garantía: misma bifurcación en todos los peers          │
└─────────────────────────────────────────────────────────────┘
```

### Tests de Consenso

| Test | Descripción | Criterio |
|------|-------------|----------|
| `test_consensus_basic` | 3 peers达成 consenso | Unánime |
| `test_consensus_partition` | Consenso durante partición | Timeout aceptable |
| `test_consensus_60_nodes` | Consenso en 60 nodos | <5s |
| `test_consensus_fork` | Resolución de fork | No fork permanent |
| `test_consensus_byzantine` | 1/3 nodos byzantine | Sistema sobrevive |

---

## Checklist de Integración

```
□ FASE 0: Congelación
  □ test_freeze_identity
  □ test_freeze_autons
  □ test_freeze_mar
  □ test_freeze_ramnet
  □ test_freeze_umbra

□ FASE 1: Conectividad Local
  □ test_tcp_handshake
  □ test_peer_discovery
  □ test_heartbeat
  □ test_flood_fill
  □ test_address_reconnection

□ FASE 2: Sincronización
  □ test_sync_autons
  □ test_sync_mar
  □ test_sync_claims
  □ test_divergence_recovery
  □ test_concurrent_escision

□ FASE 3: Topología
  □ test_mesh_partial_connectivity
  □ test_flood_propagation
  □ test_partition_heal
  □ test_byzantine_node
  □ test_node_departure

□ FASE 4: Estrés
  □ test_10_instances_10k_autons
  □ test_100k_ticks_convergence
  □ test_memory_stability
  □ test_cpu_scaling
  □ test_network_bandwidth

□ FASE 5: Emergencia
  □ test_escision_rate
  □ test_auton_lifespan
  □ test_meltrace_content
  □ test_emergence_patterns
  □ test_economic_cycles

□ FASE 6: Consenso
  □ test_consensus_basic
  □ test_consensus_partition
  □ test_consensus_60_nodes
  □ test_consensus_fork
  □ test_consensus_byzantine
```

---

## Criterios de Aceptación Finales

| Métrica | Target | Minimum |
|---------|--------|---------|
| Convergence time | <50 ticks | <100 ticks |
| Packet loss | <1% | <5% |
| Memory growth | <100KB/tick | <1MB/tick |
| Network overhead | <5% CPU | <15% CPU |
| Escision rate delta | ±5% | ±10% |
| Tiempo de Consenso | <3s | <10s |

---

## Rollback Plan

Si cualquier fase falla:

1. **Fase 0-2**: Commitear último estado funcional, investigar serialización
2. **Fase 3-4**: Reducir número de peers, investigar cuellos de botella
3. **Fase 5**: Retroceder a arquitectura single-instance, re-diseñar distribución
4. **Fase 6**: Simplificar consenso, evaluar si es necesario para v1.0

---

*Documento vivo: actualizar según resultados de cada fase.*