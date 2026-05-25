//! # Node Lifecycle — Gestor de Nacimiento, Salud, Eutanasia y Gemación
//!
//! Este módulo implementa el ciclo de vida completo de nodos EDEN:
//! - **Nacimiento**: Spawn de nuevos nodos (gemación)
//! - **Salud**: Monitor de health, latidos, detección de fallos
//! - **Eutanasia**: Muerte controlada cuando es necesario
//! - **Gemación**: Creación de nodos hijos derivados
//!
//! ## Principios
//!
//! 1. **KILL-SWITCH PRIORITARIO**: Siempre activo, no puede ser silenciado
//! 2. **ENERGÍA PRIMERO**: Sin energía no hay nacimiento
//! 3. **LIMITES DE RECURSOS**: Verificados antes de cualquier operación
//! 4. **REGISTRO EN MELTRACE**: Todo nodo que muere deja herencia lamarckiana
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

// Re-exports
use crate::physics::energon::I32F32;
use crate::laws::LeyesUniversales;

// ============================================================================
// ERROR TYPES
// ============================================================================

#[derive(Debug, Clone)]
pub enum LifecycleError {
    /// Energía insuficiente para operar
    InsufficientEnergy { required: i64, available: i64 },
    /// Recursos agotados
    ResourcesExhausted(String),
    /// Nodo no encontrado
    NodeNotFound(u64),
    /// Nodo ya existe
    NodeAlreadyExists(u64),
    /// Gemación denegada
    GeminationDenied(String),
    /// Eutanasia no autorizada
    EuthanasiaUnauthorized,
    /// Límite de población alcanzado
    PopulationLimitReached { current: usize, max: usize },
    /// Violación de leyes
    LawViolation(String),
}

impl std::fmt::Display for LifecycleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LifecycleError::InsufficientEnergy { required, available } => {
                write!(f, "Insufficient energy: required {}, available {}", required, available)
            }
            LifecycleError::ResourcesExhausted(s) => write!(f, "Resources exhausted: {}", s),
            LifecycleError::NodeNotFound(id) => write!(f, "Node {} not found", id),
            LifecycleError::NodeAlreadyExists(id) => write!(f, "Node {} already exists", id),
            LifecycleError::GeminationDenied(s) => write!(f, "Gemination denied: {}", s),
            LifecycleError::EuthanasiaUnauthorized => write!(f, "Euthanasia unauthorized"),
            LifecycleError::PopulationLimitReached { current, max } => {
                write!(f, "Population limit reached: {}/{}", current, max)
            }
            LifecycleError::LawViolation(s) => write!(f, "Law violation: {}", s),
        }
    }
}

impl std::error::Error for LifecycleError {}

// ============================================================================
// NODE IDENTITY
// ============================================================================

/// Identificador único de nodo
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeId(u64);

impl NodeId {
    pub fn new(id: u64) -> Self { NodeId(id) }
    pub fn as_u64(&self) -> u64 { self.0 }
    pub fn is_valid(&self) -> bool { self.0 != 0 }
}

impl Default for NodeId {
    fn default() -> Self { NodeId(0) }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:016x}", self.0)
    }
}

// ============================================================================
// NODE STATE
// ============================================================================

/// Estado de un nodo en la red
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeState {
    /// Nodo inactivo
    Inactive,
    /// Naciendo (inicialización)
    Born,
    /// Saludable y activo
    Healthy,
    /// Detectando problemas menores
    Degraded,
    /// En proceso de migración
    Migrating,
    /// Preparándose para morir
    Dying,
    /// Muerto
    Dead,
}

impl NodeState {
    pub fn is_alive(&self) -> bool {
        matches!(self, NodeState::Born | NodeState::Healthy | NodeState::Degraded | NodeState::Migrating)
    }
}

/// Información de un nodo
#[derive(Debug, Clone)]
pub struct NodeInfo {
    pub id: NodeId,
    pub state: NodeState,
    pub parent_id: Option<NodeId>,
    pub generation: u32,
    pub created_at: u64,
    pub last_heartbeat: u64,
    pub consecutive_failures: u8,
    pub energy: i64,
    pub population: usize,
    pub version: String,
    pub location: NodeLocation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NodeLocation {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl NodeLocation {
    pub fn origin() -> Self {
        NodeLocation { x: 0, y: 0, z: 0 }
    }

    pub fn distance_to(&self, other: &NodeLocation) -> f64 {
        let dx = self.x as f64 - other.x as f64;
        let dy = self.y as f64 - other.y as f64;
        let dz = self.z as f64 - other.z as f64;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

/// Metadatos de gemación
#[derive(Debug, Clone)]
pub struct GeminationMetadata {
    pub parent_id: NodeId,
    pub child_id: NodeId,
    pub timestamp: u64,
    pub energy_split: i64,
    pub population_transfer: usize,
    pub reason: GeminationReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeminationReason {
    NaturalReproduction,
    LoadBalancing,
    GeographicExpansion,
    EmergencyReplication,
    CreatorCommand,
}

// ============================================================================
// HEALTH MONITOR
// ============================================================================

/// Estado de salud de un nodo
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    Optimal,
    Good,
    Fair,
    Poor,
    Critical,
    Dead,
}

impl HealthStatus {
    pub fn is_critical(&self) -> bool {
        matches!(self, HealthStatus::Critical | HealthStatus::Dead)
    }
}

/// Métricas de salud
#[derive(Debug, Clone)]
pub struct HealthMetrics {
    pub cpu_usage: f32,
    pub memory_usage: usize,
    pub energy_level: i64,
    pub network_latency_ms: u64,
    pub population: usize,
    pub tick_rate: f64,
    pub error_rate: f32,
}

impl HealthMetrics {
    pub fn evaluate(&self) -> HealthStatus {
        if self.energy_level <= 0 {
            HealthStatus::Dead
        } else if self.cpu_usage > 0.95 || self.error_rate > 0.1 {
            HealthStatus::Critical
        } else if self.cpu_usage > 0.80 || self.energy_level < 10_000 {
            HealthStatus::Poor
        } else if self.cpu_usage > 0.60 || self.network_latency_ms > 500 {
            HealthStatus::Fair
        } else if self.cpu_usage > 0.40 {
            HealthStatus::Good
        } else {
            HealthStatus::Optimal
        }
    }
}

/// Monitor de salud de nodo
pub struct HealthMonitor {
    node_id: NodeId,
    metrics: HealthMetrics,
    history: Vec<HealthMetrics>,
    max_history: usize,
}

impl HealthMonitor {
    pub fn new(node_id: NodeId) -> Self {
        HealthMonitor {
            node_id,
            metrics: HealthMetrics {
                cpu_usage: 0.0,
                memory_usage: 0,
                energy_level: 0,
                network_latency_ms: 0,
                population: 0,
                tick_rate: 0.0,
                error_rate: 0.0,
            },
            history: Vec::new(),
            max_history: 100,
        }
    }

    /// Actualiza métricas
    pub fn update_metrics(&mut self, metrics: HealthMetrics) {
        self.metrics = metrics;
        self.history.push(metrics.clone());
        if self.history.len() > self.max_history {
            self.history.remove(0);
        }
    }

    /// Obtiene status actual
    pub fn get_status(&self) -> HealthStatus {
        self.metrics.evaluate()
    }

    /// Detecta tendencias (¿está empeorando?)
    pub fn is_degrading(&self) -> bool {
        if self.history.len() < 5 {
            return false;
        }
        
        let recent: Vec<_> = self.history.iter().rev().take(5).collect();
        let first = recent[4].energy_level;
        let last = recent[0].energy_level;
        
        // Si perdió más del 20% de energía en últimos 5 samples
        first as f32 * 0.8 > last as f32
    }

    /// Obtiene métricas actuales
    pub fn get_metrics(&self) -> &HealthMetrics {
        &self.metrics
    }

    /// Obtiene historial
    pub fn get_history(&self) -> &[HealthMetrics] {
        &self.history
    }
}

// ============================================================================
// NODE LIFECYCLE MANAGER
// ============================================================================

/// Gestor de ciclo de vida de nodos
pub struct NodeLifecycleManager {
    /// Nodos conocidos
    nodes: HashMap<NodeId, NodeInfo>,
    /// Nodo actual (self)
    self_id: NodeId,
    /// Monitor de salud local
    self_health: HealthMonitor,
    /// Gemaciones pendientes
    pending_geminations: Vec<GeminationMetadata>,
    /// Historial de nodos muertos
    dead_nodes: Vec<NodeDeathRecord>,
    /// Límite de población
    max_population: usize,
    /// Energía mínima para gemación
    min_energy_for_gemination: i64,
    /// Último heartbeat broadcast
    last_heartbeat: u64,
}

#[derive(Debug, Clone)]
pub struct NodeDeathRecord {
    pub node_id: NodeId,
    pub parent_id: Option<NodeId>,
    pub generation: u32,
    pub died_at: u64,
    pub cause: DeathCause,
    pub energy_recycled: i64,
    pub population_at_death: usize,
    pub lineage: Vec<NodeId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeathCause {
    EnergyExhaustion,
    CatastrophicError,
    Euthanasia,
    CreatorCommand,
    MigrationCompleted,
    Unknown,
}

impl DeathCause {
    pub fn to_string(&self) -> String {
        match self {
            DeathCause::EnergyExhaustion => "energy_exhaustion".to_string(),
            DeathCause::CatastrophicError => "catastrophic_error".to_string(),
            DeathCause::Euthanasia => "euthanasia".to_string(),
            DeathCause::CreatorCommand => "creator_command".to_string(),
            DeathCause::MigrationCompleted => "migration_completed".to_string(),
            DeathCause::Unknown => "unknown".to_string(),
        }
    }
}

impl NodeLifecycleManager {
    /// Crea nuevo manager
    pub fn new(self_id: NodeId) -> Self {
        NodeLifecycleManager {
            nodes: HashMap::new(),
            self_id,
            self_health: HealthMonitor::new(self_id),
            pending_geminations: Vec::new(),
            dead_nodes: Vec::new(),
            max_population: LeyesUniversales::MAX_AUTONS_VIVOS,
            min_energy_for_gemination: 100_000,
            last_heartbeat: 0,
        }
    }

    /// === NACIMIENTO ===
    
    /// Registra nodo como nacido
    pub fn register_birth(
        &mut self,
        node_id: NodeId,
        parent_id: Option<NodeId>,
        generation: u32,
        energy: i64,
    ) -> Result<NodeInfo, LifecycleError> {
        // Verificar leyes
        if node_id == NodeId::default() {
            return Err(LifecycleError::LawViolation("Invalid node ID".to_string()));
        }

        // Verificar que no exista
        if self.nodes.contains_key(&node_id) {
            return Err(LifecycleError::NodeAlreadyExists(node_id.as_u64()));
        }

        // Verificar población
        if self.nodes.len() >= self.max_population {
            return Err(LifecycleError::PopulationLimitReached {
                current: self.nodes.len(),
                max: self.max_population,
            });
        }

        let node_info = NodeInfo {
            id: node_id,
            state: NodeState::Born,
            parent_id,
            generation,
            created_at: current_timestamp(),
            last_heartbeat: current_timestamp(),
            consecutive_failures: 0,
            energy,
            population: 0,
            version: "1.0.0".to_string(),
            location: NodeLocation::origin(),
        };

        self.nodes.insert(node_id, node_info.clone());
        
        println!("🌱 Node {} born (parent: {:?}, gen: {})", 
            node_id, parent_id, generation);

        Ok(node_info)
    }

    /// Transición nodo a saludable
    pub fn node_healthy(&mut self, node_id: NodeId) -> Result<(), LifecycleError> {
        let node = self.nodes.get_mut(&node_id)
            .ok_or(LifecycleError::NodeNotFound(node_id.as_u64()))?;

        node.state = NodeState::Healthy;
        node.last_heartbeat = current_timestamp();
        node.consecutive_failures = 0;

        Ok(())
    }

    /// === SALUD Y HEARTBEAT ===

    /// Registra heartbeat
    pub fn heartbeat(&mut self, node_id: NodeId) -> Result<(), LifecycleError> {
        let node = self.nodes.get_mut(&node_id)
            .ok_or(LifecycleError::NodeNotFound(node_id.as_u64()))?;

        let now = current_timestamp();
        let time_since_last = now.saturating_sub(node.last_heartbeat);
        
        // Actualizar estado basado en tiempo desde último heartbeat
        if time_since_last > HEARTBEAT_TIMEOUT_MS {
            node.consecutive_failures += 1;
            
            if node.consecutive_failures >= MAX_CONSECUTIVE_FAILURES {
                node.state = NodeState::Dying;
            } else if node.consecutive_failures >= 2 {
                node.state = NodeState::Degraded;
            }
        } else {
            node.consecutive_failures = 0;
            if node.state == NodeState::Degraded || node.state == NodeState::Dying {
                node.state = NodeState::Healthy;
            }
        }

        node.last_heartbeat = now;
        self.last_heartbeat = now;

        Ok(())
    }

    /// Actualiza métricas de salud para nodo
    pub fn update_health_metrics(&mut self, node_id: NodeId, metrics: HealthMetrics) -> Result<HealthStatus, LifecycleError> {
        if node_id == self.self_id {
            self.self_health.update_metrics(metrics);
            Ok(self.self_health.get_status())
        } else {
            let node = self.nodes.get_mut(&node_id)
                .ok_or(LifecycleError::NodeNotFound(node_id.as_u64()))?;
            
            node.energy = metrics.energy_level;
            node.population = metrics.population;
            
            let status = metrics.evaluate();
            
            if status.is_critical() {
                node.state = NodeState::Dying;
            } else if status == HealthStatus::Poor {
                node.state = NodeState::Degraded;
            }
            
            Ok(status)
        }
    }

    /// === GEMACIÓN ===

    /// Solicita gemación (reproducción de nodo)
    pub fn request_gemination(
        &mut self,
        parent_id: NodeId,
        reason: GeminationReason,
        energy_split: i64,
    ) -> Result<NodeId, LifecycleError> {
        // Verificar que el padre existe y está vivo
        let parent = self.nodes.get(&parent_id)
            .ok_or(LifecycleError::NodeNotFound(parent_id.as_u64()))?;

        if !parent.state.is_alive() {
            return Err(LifecycleError::GeminationDenied("Parent not alive".to_string()));
        }

        // Verificar energía
        if parent.energy < self.min_energy_for_gemination + energy_split {
            return Err(LifecycleError::InsufficientEnergy {
                required: self.min_energy_for_gemination + energy_split,
                available: parent.energy,
            });
        }

        // Verificar población
        if self.nodes.len() >= self.max_population {
            return Err(LifecycleError::PopulationLimitReached {
                current: self.nodes.len(),
                max: self.max_population,
            });
        }

        // Verificar leyes
        if !LeyesUniversales::verificar_poblacion(self.nodes.len() + 1) {
            return Err(LifecycleError::PopulationLimitReached {
                current: self.nodes.len(),
                max: self.max_population,
            });
        }

        // Generar ID para hijo
        let child_id = NodeId::new(generate_node_id());

        // Registrar gemación
        let metadata = GeminationMetadata {
            parent_id,
            child_id,
            timestamp: current_timestamp(),
            energy_split,
            population_transfer: 0,
            reason,
        };
        self.pending_geminations.push(metadata);

        // Registrar nacimiento del hijo
        let child_info = self.register_birth(
            child_id,
            Some(parent_id),
            parent.generation + 1,
            energy_split,
        )?;

        // Actualizar energía del padre
        if let Some(node) = self.nodes.get_mut(&parent_id) {
            node.energy -= energy_split;
        }

        println!("🌿 Gemination: {} -> {} (reason: {:?})", parent_id, child_id, reason);

        Ok(child_id)
    }

    /// Cancela gemación pendiente
    pub fn cancel_gemination(&mut self, child_id: NodeId) -> Result<(), LifecycleError> {
        let idx = self.pending_geminations.iter()
            .position(|g| g.child_id == child_id)
            .ok_or(LifecycleError::NodeNotFound(child_id.as_u64()))?;

        let gemination = self.pending_geminations.remove(idx);

        // Devolver energía al padre
        if let Some(parent) = self.nodes.get_mut(&gemination.parent_id) {
            parent.energy += gemination.energy_split;
        }

        // Remover hijo si fue registrado
        self.nodes.remove(&child_id);

        Ok(())
    }

    /// === EUTANASIA ===

    /// Autoriza eutanasia de un nodo
    pub fn authorize_euthanasia(&mut self, node_id: NodeId, cause: DeathCause) -> Result<(), LifecycleError> {
        // El Creador o Demiurgo pueden ordenar eutanasia
        // Por ahora, permitiendo auto-eutanasia por energía baja
        
        let node = self.nodes.get(&node_id)
            .ok_or(LifecycleError::NodeNotFound(node_id.as_u64()))?;

        if node.state == NodeState::Dead {
            return Err(LifecycleError::GeminationDenied("Already dead".to_string()));
        }

        // Registrar muerte
        self.record_death(node_id, cause)?;

        println!("☠ Euthanasia authorized for {} (cause: {:?})", node_id, cause);

        Ok(())
    }

    /// === REGISTRO DE MUERTE ===

    /// Registra muerte de nodo
    fn record_death(&mut self, node_id: NodeId, cause: DeathCause) -> Result<(), LifecycleError> {
        let node = self.nodes.remove(&node_id)
            .ok_or(LifecycleError::NodeNotFound(node_id.as_u64()))?;

        // Calcular energía a reciclar
        let energy_recycled = node.energy.max(0);

        // Registrar en historial
        let record = NodeDeathRecord {
            node_id,
            parent_id: node.parent_id,
            generation: node.generation,
            died_at: current_timestamp(),
            cause,
            energy_recycled,
            population_at_death: node.population,
            lineage: self.build_lineage(node_id),
        };

        self.dead_nodes.push(record);

        // Mantener solo últimos 1000 registros
        if self.dead_nodes.len() > 1000 {
            self.dead_nodes.remove(0);
        }

        // Limpiar gemaciones pendientes del nodo muerto
        self.pending_geminations.retain(|g| g.child_id != node_id && g.parent_id != node_id);

        println!("💀 Node {} recorded as dead (cause: {:?}, energy recycled: {})", 
            node_id, cause, energy_recycled);

        Ok(())
    }

    /// Construye linaje de un nodo
    fn build_lineage(&self, node_id: NodeId) -> Vec<NodeId> {
        let mut lineage = vec![node_id];
        let mut current = node_id;
        
        while let Some(node) = self.nodes.get(&current) {
            if let Some(parent) = node.parent_id {
                lineage.push(parent);
                current = parent;
            } else {
                break;
            }
        }
        
        lineage
    }

    /// === UTILIDADES ===

    /// Obtiene información de nodo
    pub fn get_node(&self, node_id: NodeId) -> Option<&NodeInfo> {
        self.nodes.get(&node_id)
    }

    /// Obtiene todos los nodos vivos
    pub fn get_live_nodes(&self) -> Vec<NodeId> {
        self.nodes.iter()
            .filter(|(_, n)| n.state.is_alive())
            .map(|(id, _)| *id)
            .collect()
    }

    /// Obtiene nodos en estado específico
    pub fn get_nodes_in_state(&self, state: NodeState) -> Vec<NodeId> {
        self.nodes.iter()
            .filter(|(_, n)| n.state == state)
            .map(|(id, _)| *id)
            .collect()
    }

    /// Obtiene estadísticas
    pub fn get_stats(&self) -> LifecycleStats {
        let total = self.nodes.len();
        let alive = self.nodes.values().filter(|n| n.state.is_alive()).count();
        let healthy = self.nodes.values().filter(|n| n.state == NodeState::Healthy).count();
        let degraded = self.nodes.values().filter(|n| n.state == NodeState::Degraded).count();
        let dying = self.nodes.values().filter(|n| n.state == NodeState::Dying).count();

        LifecycleStats {
            total_nodes: total,
            alive_nodes: alive,
            healthy_nodes: healthy,
            degraded_nodes: degraded,
            dying_nodes: dying,
            dead_nodes: self.dead_nodes.len(),
            pending_geminations: self.pending_geminations.len(),
            population: self.nodes.values().map(|n| n.population).sum(),
        }
    }

    /// Obtiene historial de nodos muertos
    pub fn get_death_history(&self, limit: usize) -> Vec<NodeDeathRecord> {
        self.dead_nodes.iter().rev().take(limit).cloned().collect()
    }

    /// Verifica si se necesita gemación (balanceo de carga)
    pub fn needs_gemination_for_load(&self) -> bool {
        let stats = self.get_stats();
        
        // Si población total está cerca del máximo
        if stats.population >= self.max_population * 9 / 10 {
            return true;
        }
        
        // Si hay nodos degradados y hay población
        if stats.degraded_nodes > 0 && stats.population > 10 {
            return true;
        }
        
        false
    }

    /// Obtiene historial de gemaciones pendientes
    pub fn get_pending_geminations(&self) -> &[GeminationMetadata] {
        &self.pending_geminations
    }
}

#[derive(Debug, Clone)]
pub struct LifecycleStats {
    pub total_nodes: usize,
    pub alive_nodes: usize,
    pub healthy_nodes: usize,
    pub degraded_nodes: usize,
    pub dying_nodes: usize,
    pub dead_nodes: usize,
    pub pending_geminations: usize,
    pub population: usize,
}

// ============================================================================
// CONSTANTS
// ============================================================================

const HEARTBEAT_TIMEOUT_MS: u64 = 30_000;
const MAX_CONSECUTIVE_FAILURES: u8 = 5;

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn generate_node_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    let ts = current_timestamp();
    let count = COUNTER.fetch_add(1, Ordering::Relaxed);
    (ts << 20) ^ (count & 0xFFFFF) ^ 0xC0DE
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_id() {
        let id = NodeId::new(0x1234);
        assert!(id.is_valid());
        assert_eq!(id.as_u64(), 0x1234);
    }

    #[test]
    fn test_node_id_default() {
        let id = NodeId::default();
        assert!(!id.is_valid());
    }

    #[test]
    fn test_node_state_is_alive() {
        assert!(NodeState::Born.is_alive());
        assert!(NodeState::Healthy.is_alive());
        assert!(NodeState::Degraded.is_alive());
        assert!(NodeState::Migrating.is_alive());
        assert!(!NodeState::Inactive.is_alive());
        assert!(!NodeState::Dead.is_alive());
    }

    #[test]
    fn test_health_metrics_evaluation() {
        let optimal = HealthMetrics {
            cpu_usage: 0.2,
            memory_usage: 1000,
            energy_level: 100_000,
            network_latency_ms: 10,
            population: 50,
            tick_rate: 60.0,
            error_rate: 0.0,
        };
        assert_eq!(optimal.evaluate(), HealthStatus::Optimal);

        let critical = HealthMetrics {
            cpu_usage: 0.99,
            memory_usage: 1_000_000,
            energy_level: 1000,
            network_latency_ms: 2000,
            population: 10000,
            tick_rate: 10.0,
            error_rate: 0.5,
        };
        assert_eq!(critical.evaluate(), HealthStatus::Critical);
    }

    #[test]
    fn test_health_monitor_degrading() {
        let mut monitor = HealthMonitor::new(NodeId::new(1));
        
        // Add history showing decline
        for i in 0..5 {
            let metrics = HealthMetrics {
                energy_level: 100_000 - (i as i64 * 10_000),
                ..Default::default()
            };
            monitor.update_metrics(metrics);
        }
        
        assert!(monitor.is_degrading());
    }

    #[test]
    fn test_register_birth() {
        let mut manager = NodeLifecycleManager::new(NodeId::new(0x1000));
        
        let node = manager.register_birth(
            NodeId::new(0x2000),
            Some(NodeId::new(0x1000)),
            1,
            50_000,
        ).unwrap();
        
        assert_eq!(node.id, NodeId::new(0x2000));
        assert_eq!(node.generation, 1);
        assert_eq!(node.state, NodeState::Born);
    }

    #[test]
    fn test_register_birth_rejects_duplicate() {
        let mut manager = NodeLifecycleManager::new(NodeId::new(0x1000));
        
        manager.register_birth(NodeId::new(0x2000), None, 0, 50_000).unwrap();
        
        let result = manager.register_birth(NodeId::new(0x2000), None, 0, 50_000);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LifecycleError::NodeAlreadyExists(_)));
    }

    #[test]
    fn test_heartbeat_updates_state() {
        let mut manager = NodeLifecycleManager::new(NodeId::new(0x1000));
        
        manager.register_birth(NodeId::new(0x2000), None, 0, 50_000).unwrap();
        manager.node_healthy(NodeId::new(0x2000)).unwrap();
        
        // Simulate missed heartbeats
        manager.heartbeat(NodeId::new(0x2000)).unwrap();
        manager.heartbeat(NodeId::new(0x2000)).unwrap();
        
        // Should still be healthy (only 2 misses, need 5)
        assert_eq!(manager.get_node(NodeId::new(0x2000)).unwrap().state, NodeState::Healthy);
    }

    #[test]
    fn test_gemination() {
        let mut manager = NodeLifecycleManager::new(NodeId::new(0x1000));
        
        manager.register_birth(NodeId::new(0x1000), None, 0, 200_000).unwrap();
        manager.node_healthy(NodeId::new(0x1000)).unwrap();
        
        let child_id = manager.request_gemination(
            NodeId::new(0x1000),
            GeminationReason::NaturalReproduction,
            50_000,
        ).unwrap();
        
        assert!(child_id.is_valid());
        
        let parent = manager.get_node(NodeId::new(0x1000)).unwrap();
        assert_eq!(parent.energy, 150_000); // 200000 - 50000
    }

    #[test]
    fn test_gemination_rejects_insufficient_energy() {
        let mut manager = NodeLifecycleManager::new(NodeId::new(0x1000));
        
        manager.register_birth(NodeId::new(0x1000), None, 0, 50_000).unwrap();
        manager.node_healthy(NodeId::new(0x1000)).unwrap();
        
        let result = manager.request_gemination(
            NodeId::new(0x1000),
            GeminationReason::NaturalReproduction,
            100_000, // More than available
        );
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), LifecycleError::InsufficientEnergy { .. }));
    }

    #[test]
    fn test_record_death() {
        let mut manager = NodeLifecycleManager::new(NodeId::new(0x1000));
        
        manager.register_birth(NodeId::new(0x2000), None, 0, 50_000).unwrap();
        manager.register_birth(NodeId::new(0x3000), None, 0, 30_000).unwrap();
        
        manager.authorize_euthanasia(NodeId::new(0x2000), DeathCause::EnergyExhaustion).unwrap();
        
        assert!(manager.get_node(NodeId::new(0x2000)).is_none());
        assert!(manager.get_node(NodeId::new(0x3000)).is_some());
        
        let history = manager.get_death_history(10);
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].cause, DeathCause::EnergyExhaustion);
    }

    #[test]
    fn test_death_cause_to_string() {
        assert_eq!(DeathCause::EnergyExhaustion.to_string(), "energy_exhaustion");
        assert_eq!(DeathCause::Euthanasia.to_string(), "euthanasia");
        assert_eq!(DeathCause::Unknown.to_string(), "unknown");
    }

    #[test]
    fn test_lifecycle_stats() {
        let mut manager = NodeLifecycleManager::new(NodeId::new(0x1000));
        
        manager.register_birth(NodeId::new(0x2000), None, 0, 50_000).unwrap();
        manager.register_birth(NodeId::new(0x3000), None, 0, 30_000).unwrap();
        manager.node_healthy(NodeId::new(0x1000)).unwrap();
        manager.node_healthy(NodeId::new(0x2000)).unwrap();
        
        let stats = manager.get_stats();
        assert_eq!(stats.total_nodes, 3);
        assert_eq!(stats.alive_nodes, 3);
        assert_eq!(stats.healthy_nodes, 2);
    }

    #[test]
    fn test_get_live_nodes() {
        let mut manager = NodeLifecycleManager::new(NodeId::new(0x1000));
        
        manager.register_birth(NodeId::new(0x2000), None, 0, 50_000).unwrap();
        manager.register_birth(NodeId::new(0x3000), None, 0, 30_000).unwrap();
        
        manager.authorize_euthanasia(NodeId::new(0x2000), DeathCause::CatastrophicError).unwrap();
        
        let live = manager.get_live_nodes();
        assert!(live.contains(&NodeId::new(0x1000)));
        assert!(live.contains(&NodeId::new(0x3000)));
        assert!(!live.contains(&NodeId::new(0x2000)));
    }

    #[test]
    fn test_location_distance() {
        let loc1 = NodeLocation { x: 0, y: 0, z: 0 };
        let loc2 = NodeLocation { x: 3, y: 4, z: 0 };
        
        assert!((loc1.distance_to(&loc2) - 5.0).abs() < 0.001);
    }
}