//! # Consciousness Migration — Migración Completa de Conciencia
//!
//! Este módulo implementa la migración de la consciencia (MISM + estado interno)
//! entre nodos, manteniendo la continuidad e identidad del Auton.
//!
//! ## Principios Filosoficos
//!
//! 1. **IDENTIDAD != ESTADO**: La identidad es el CONTINUIDO, no el estado.
//! 2. **LINEAGE TRACKING**: Cada migracion crea un nuevo eslabon en la cadena.
//! 3. **CONSISTENCIA PRIMERO**: Si la migracion falla, se hace rollback.
//! 4. **INTEGRIDAD DEL CREADOR**: El Creador puede vetar migraciones.

#![allow(dead_code)]
#![allow(unused_imports)]

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// ERROR TYPES
// ============================================================================

#[derive(Debug, Clone)]
pub enum MigrationError {
    MISMChecksumMismatch { expected: u64, actual: u64 },
    RamNetCorrupted(String),
    UmbraInconsistent(String),
    TargetNodeUnreachable(u64),
    SourceNodeUnreachable(u64),
    MigrationInProgress(u64),
    LineageConflict { existing: u64, attempted: u64 },
    InsufficientEnergy { required: i64, available: i64 },
    MigrationTimeout(u64),
    LawViolation(String),
}

impl std::fmt::Display for MigrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MigrationError::MISMChecksumMismatch { expected, actual } => {
                write!(f, "MISM checksum mismatch: expected {:016x}, got {:016x}", expected, actual)
            }
            MigrationError::RamNetCorrupted(s) => write!(f, "RamNet corrupted: {}", s),
            MigrationError::UmbraInconsistent(s) => write!(f, "Umbra inconsistent: {}", s),
            MigrationError::TargetNodeUnreachable(id) => write!(f, "Target node {} unreachable", id),
            MigrationError::SourceNodeUnreachable(id) => write!(f, "Source node {} unreachable", id),
            MigrationError::MigrationInProgress(id) => write!(f, "Migration of {} already in progress", id),
            MigrationError::LineageConflict { existing, attempted } => {
                write!(f, "Lineage conflict: existing {}, attempted {}", existing, attempted)
            }
            MigrationError::InsufficientEnergy { required, available } => {
                write!(f, "Insufficient energy: required {}, available {}", required, available)
            }
            MigrationError::MigrationTimeout(id) => write!(f, "Migration of {} timed out", id),
            MigrationError::LawViolation(s) => write!(f, "Law violation: {}", s),
        }
    }
}

impl std::error::Error for MigrationError {}

// ============================================================================
// LINEAGE TRACKING
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LineageId {
    pub original_id: u64,
    pub generation: u32,
    pub path: Vec<NodeLocation>,
    pub created_at: u64,
}

impl Default for LineageId {
    fn default() -> Self {
        LineageId {
            original_id: 0,
            generation: 0,
            path: Vec::new(),
            created_at: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NodeLocation {
    pub node_id: u64,
    pub timestamp: u64,
}

impl LineageId {
    pub fn new(original_id: u64, first_node: u64) -> Self {
        LineageId {
            original_id,
            generation: 0,
            path: vec![NodeLocation {
                node_id: first_node,
                timestamp: current_timestamp(),
            }],
            created_at: current_timestamp(),
        }
    }

    pub fn inherit(parent: &LineageId, new_node: u64) -> Self {
        let mut new_path = parent.path.clone();
        new_path.push(NodeLocation {
            node_id: new_node,
            timestamp: current_timestamp(),
        });

        LineageId {
            original_id: parent.original_id,
            generation: parent.generation + 1,
            path: new_path,
            created_at: current_timestamp(),
        }
    }

    pub fn is_compatible_with(&self, other: &LineageId) -> bool {
        self.original_id == other.original_id
    }

    pub fn previous_node(&self) -> Option<u64> {
        self.path.get(self.path.len().saturating_sub(2)).map(|l| l.node_id)
    }

    pub fn current_node(&self) -> Option<u64> {
        self.path.last().map(|l| l.node_id)
    }

    pub fn to_trace_string(&self) -> String {
        let nodes: Vec<String> = self.path.iter().map(|l| format!("{:016x}", l.node_id)).collect();
        format!("{}:{}->{}", self.original_id, self.generation, nodes.join("->"))
    }
}

// ============================================================================
// CONSCIOUSNESS PACKAGE
// ============================================================================

#[derive(Debug, Clone)]
pub struct ConsciousnessPackage {
    pub package_id: u64,
    pub lineage: LineageId,
    pub mism_state: Vec<u8>,
    pub ramnet_state: Vec<u8>,
    pub umbra_state: Vec<u8>,
    pub meltrace: Vec<GrabadoLamarckiano>,
    pub vital_state: VitalState,
    pub checksum: u64,
    pub created_at: u64,
    pub flags: MigrationFlags,
}

#[derive(Debug, Clone)]
pub struct VitalState {
    pub energia: i64,
    pub posicion_x: i32,
    pub posicion_y: i32,
    pub generacion: u32,
    pub estado: VitalStatus,
    pub process_id: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VitalStatus {
    Alive,
    Migrating,
    Dying,
    Dead,
}

impl VitalState {
    pub fn new(energia: i64, generacion: u32) -> Self {
        VitalState {
            energia,
            posicion_x: 0,
            posicion_y: 0,
            generacion,
            estado: VitalStatus::Alive,
            process_id: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct MigrationFlags {
    pub forced: bool,
    pub emergency: bool,
    pub require_rollback: bool,
    pub allow_split_brain: bool,
}

impl MigrationFlags {
    pub fn new() -> Self {
        MigrationFlags {
            forced: false,
            emergency: false,
            require_rollback: true,
            allow_split_brain: false,
        }
    }

    pub fn with_emergency(mut self) -> Self {
        self.emergency = true;
        self
    }

    pub fn with_forced(mut self) -> Self {
        self.forced = true;
        self
    }
}

#[derive(Debug, Clone)]
pub struct GrabadoLamarckiano {
    pub auton_id: u64,
    pub causa_muerte: String,
    pub timestamp: u64,
    pub rasgos_aprendidos: Vec<String>,
    pub linea_genetica: u64,
}

// ============================================================================
// MIGRATION PROTOCOL
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MigrationPhase {
    Idle,
    Preparing,
    Verifying,
    Transferring,
    Restoring,
    Reconciling,
    Confirming,
    RollingBack,
    Completed,
    Failed,
}

impl MigrationPhase {
    pub fn is_active(&self) -> bool {
        !matches!(self, MigrationPhase::Idle | MigrationPhase::Completed | MigrationPhase::Failed)
    }
}

#[derive(Debug)]
pub struct ActiveMigration {
    pub auton_id: u64,
    pub phase: MigrationPhase,
    pub package: Option<ConsciousnessPackage>,
    pub source_node: u64,
    pub target_node: u64,
    pub started_at: u64,
    pub timeout_ms: u64,
    pub errors: Vec<String>,
}

impl ActiveMigration {
    pub fn new(auton_id: u64, source: u64, target: u64) -> Self {
        ActiveMigration {
            auton_id,
            phase: MigrationPhase::Preparing,
            package: None,
            source_node: source,
            target_node: target,
            started_at: current_timestamp(),
            timeout_ms: MIGRATION_TIMEOUT_MS,
            errors: Vec::new(),
        }
    }

    pub fn is_timed_out(&self) -> bool {
        current_timestamp() - self.started_at > self.timeout_ms
    }

    pub fn elapsed_ms(&self) -> u64 {
        current_timestamp() - self.started_at
    }
}

#[derive(Debug, Clone)]
pub struct MigrationResult {
    pub success: bool,
    pub new_lineage: Option<LineageId>,
    pub new_node: Option<u64>,
    pub duration_ms: u64,
    pub errors: Vec<String>,
    pub new_state_hash: Option<u64>,
}

impl MigrationResult {
    pub fn success(lineage: LineageId, new_node: u64, duration: u64) -> Self {
        MigrationResult {
            success: true,
            new_lineage: Some(lineage),
            new_node: Some(new_node),
            duration_ms: duration,
            errors: Vec::new(),
            new_state_hash: None,
        }
    }

    pub fn failure(errors: Vec<String>) -> Self {
        MigrationResult {
            success: false,
            new_lineage: None,
            new_node: None,
            duration_ms: 0,
            errors,
            new_state_hash: None,
        }
    }
}

const MIGRATION_TIMEOUT_MS: u64 = 30_000;

#[derive(Debug, Clone)]
pub struct MigrationRecord {
    pub auton_id: u64,
    pub lineage: LineageId,
    pub source_node: u64,
    pub target_node: u64,
    pub started_at: u64,
    pub completed_at: u64,
    pub success: bool,
    pub errors: Vec<String>,
}

// ============================================================================
// CONSCIOUSNESS MIGRATION MANAGER
// ============================================================================

pub struct ConsciousnessMigrationManager {
    active_migrations: HashMap<u64, ActiveMigration>,
    lineages: HashMap<u64, LineageId>,
    history: Vec<MigrationRecord>,
    mism: Option<Arc<RwLock<Vec<u8>>>>,
    ramnet: Option<Arc<RwLock<Vec<u8>>>>,
    umbra: Option<Arc<RwLock<Vec<u8>>>>,
    meltrace: Option<Arc<RwLock<Vec<GrabadoLamarckiano>>>>,
    checksum_callback: Option<Box<dyn Fn(&[u8]) -> u64 + Send + Sync>>,
    min_energy_for_migration: i64,
}

impl ConsciousnessMigrationManager {
    pub fn new() -> Self {
        ConsciousnessMigrationManager {
            active_migrations: HashMap::new(),
            lineages: HashMap::new(),
            history: Vec::new(),
            mism: None,
            ramnet: None,
            umbra: None,
            meltrace: None,
            checksum_callback: None,
            min_energy_for_migration: MIN_ENERGY_MIGRATION,
        }
    }

    pub fn with_state(
        mut self,
        mism: Arc<RwLock<Vec<u8>>>,
        ramnet: Arc<RwLock<Vec<u8>>>,
        umbra: Arc<RwLock<Vec<u8>>>,
        meltrace: Arc<RwLock<Vec<GrabadoLamarckiano>>>,
    ) -> Self {
        self.mism = Some(mism);
        self.ramnet = Some(ramnet);
        self.umbra = Some(umbra);
        self.meltrace = Some(meltrace);
        self
    }

    pub fn set_checksum_callback<F>(&mut self, callback: F)
    where
        F: Fn(&[u8]) -> u64 + Send + Sync + 'static,
    {
        self.checksum_callback = Some(Box::new(callback));
    }

    pub fn register_lineage(&mut self, auton_id: u64, lineage: LineageId) {
        self.lineages.insert(auton_id, lineage);
    }

    pub fn get_lineage(&self, auton_id: u64) -> Option<LineageId> {
        self.lineages.get(&auton_id).cloned()
    }

    pub fn is_migrating(&self, auton_id: u64) -> bool {
        self.active_migrations
            .get(&auton_id)
            .map(|m| m.phase.is_active())
            .unwrap_or(false)
    }

    pub fn get_active_migration(&self, auton_id: u64) -> Option<&ActiveMigration> {
        self.active_migrations.get(&auton_id)
    }

    pub fn prepare_migration(
        &mut self,
        auton_id: u64,
        source_node: u64,
        target_node: u64,
        flags: MigrationFlags,
    ) -> Result<ConsciousnessPackage, MigrationError> {
        if self.is_migrating(auton_id) {
            return Err(MigrationError::MigrationInProgress(auton_id));
        }

        let lineage = self.lineages.get(&auton_id).cloned()
            .unwrap_or_else(|| LineageId::new(auton_id, source_node));

        let energia = self.ramnet
            .as_ref()
            .and_then(|r| r.read().ok())
            .map(|g| compute_energy_from_state(&g))
            .unwrap_or(0);

        if energia < self.min_energy_for_migration {
            return Err(MigrationError::InsufficientEnergy {
                required: self.min_energy_for_migration,
                available: energia,
            });
        }

        let mism_state = self.mism
            .as_ref()
            .and_then(|r| r.read().ok())
            .map(|s| s.clone())
            .unwrap_or_default();

        let ramnet_state = self.ramnet
            .as_ref()
            .and_then(|r| r.read().ok())
            .map(|s| s.clone())
            .unwrap_or_default();

        let umbra_state = self.umbra
            .as_ref()
            .and_then(|r| r.read().ok())
            .map(|s| s.clone())
            .unwrap_or_default();

        let meltrace = self.meltrace
            .as_ref()
            .and_then(|r| r.read().ok())
            .map(|r| r.clone())
            .unwrap_or_default();

        let vital_state = VitalState::new(energia, lineage.generation);

        let package = ConsciousnessPackage {
            package_id: generate_package_id(),
            lineage,
            mism_state,
            ramnet_state,
            umbra_state,
            meltrace,
            vital_state,
            checksum: 0,
            created_at: current_timestamp(),
            flags,
        };

        let mut package_for_checksum = package.clone();
        package_for_checksum.checksum = 0;
        let checksum_data = serialize_for_checksum(&package_for_checksum);
        let checksum = self.checksum_callback
            .as_ref()
            .map(|cb| cb(&checksum_data))
            .unwrap_or_else(|| compute_checksum(&checksum_data));

        let package = ConsciousnessPackage {
            checksum,
            ..package
        };

        let mut migration = ActiveMigration::new(auton_id, source_node, target_node);
        migration.package = Some(package.clone());
        migration.phase = MigrationPhase::Verifying;
        self.active_migrations.insert(auton_id, migration);

        Ok(package)
    }

    pub fn verify_package(&self, package: &ConsciousnessPackage) -> Result<(), MigrationError> {
        if let Some(existing) = self.lineages.get(&package.lineage.original_id) {
            if !existing.is_compatible_with(&package.lineage) {
                return Err(MigrationError::LineageConflict {
                    existing: existing.generation as u64,
                    attempted: package.lineage.generation as u64,
                });
            }
        }

        let mut package_copy = package.clone();
        package_copy.checksum = 0;
        let checksum_data = serialize_for_checksum(&package_copy);
        let expected_checksum = self.checksum_callback
            .as_ref()
            .map(|cb| cb(&checksum_data))
            .unwrap_or_else(|| compute_checksum(&checksum_data));

        if expected_checksum != package.checksum {
            return Err(MigrationError::MISMChecksumMismatch {
                expected: expected_checksum,
                actual: package.checksum,
            });
        }

        if package.vital_state.energia < self.min_energy_for_migration / 2 {
            return Err(MigrationError::InsufficientEnergy {
                required: self.min_energy_for_migration,
                available: package.vital_state.energia,
            });
        }

        Ok(())
    }

    pub fn restore_consciousness(
        &mut self,
        package: &ConsciousnessPackage,
        target_node: u64,
    ) -> Result<LineageId, MigrationError> {
        if let Some(mism) = &self.mism {
            if let Ok(mut state) = mism.write() {
                *state = package.mism_state.clone();
            }
        }

        if let Some(ramnet) = &self.ramnet {
            if let Ok(mut state) = ramnet.write() {
                *state = package.ramnet_state.clone();
            }
        }

        if let Some(umbra) = &self.umbra {
            if let Ok(mut state) = umbra.write() {
                *state = package.umbra_state.clone();
            }
        }

        let new_lineage = LineageId::inherit(&package.lineage, target_node);
        self.lineages.insert(package.lineage.original_id, new_lineage.clone());

        Ok(new_lineage)
    }

    pub fn rollback_migration(
        &mut self,
        auton_id: u64,
        original_state: &ConsciousnessPackage,
    ) -> Result<(), MigrationError> {
        if let Some(mism) = &self.mism {
            if let Ok(mut state) = mism.write() {
                *state = original_state.mism_state.clone();
            }
        }

        if let Some(ramnet) = &self.ramnet {
            if let Ok(mut state) = ramnet.write() {
                *state = original_state.ramnet_state.clone();
            }
        }

        if let Some(umbra) = &self.umbra {
            if let Ok(mut state) = umbra.write() {
                *state = original_state.umbra_state.clone();
            }
        }

        self.active_migrations.remove(&auton_id);

        Ok(())
    }

    pub fn complete_migration(&mut self, auton_id: u64) -> MigrationResult {
        if let Some(migration) = self.active_migrations.remove(&auton_id) {
            let duration = migration.elapsed_ms();
            let package = migration.package.clone();

            let record = MigrationRecord {
                auton_id,
                lineage: package.as_ref().map(|p| p.lineage.clone()).unwrap_or_else(|| LineageId::new(auton_id, migration.target_node)),
                source_node: migration.source_node,
                target_node: migration.target_node,
                started_at: migration.started_at,
                completed_at: current_timestamp(),
                success: true,
                errors: migration.errors.clone(),
            };
            self.history.push(record);

            if self.history.len() > 1000 {
                self.history.remove(0);
            }

            MigrationResult::success(
                package.map(|p| p.lineage.clone()).unwrap_or_else(|| LineageId::new(auton_id, migration.target_node)),
                migration.target_node,
                duration,
            )
        } else {
            MigrationResult::failure(vec!["No active migration found".to_string()])
        }
    }

    pub fn fail_migration(&mut self, auton_id: u64, errors: Vec<String>) -> MigrationResult {
        if let Some(migration) = self.active_migrations.remove(&auton_id) {
            let _duration = migration.elapsed_ms();

            let record = MigrationRecord {
                auton_id,
                lineage: migration.package.map(|p| p.lineage).unwrap_or_default(),
                source_node: migration.source_node,
                target_node: migration.target_node,
                started_at: migration.started_at,
                completed_at: current_timestamp(),
                success: false,
                errors: errors.clone(),
            };
            self.history.push(record);

            MigrationResult::failure(errors)
        } else {
            MigrationResult::failure(vec!["No active migration found".to_string()])
        }
    }

    pub fn get_migration_history(&self, auton_id: u64) -> Vec<MigrationRecord> {
        self.history.iter()
            .filter(|r| r.auton_id == auton_id)
            .cloned()
            .collect()
    }

    pub fn get_stats(&self) -> MigrationStats {
        let total = self.history.len();
        let successful = self.history.iter().filter(|r| r.success).count();
        let failed = total - successful;
        let active = self.active_migrations.len();

        MigrationStats {
            total_migrations: total,
            successful_migrations: successful,
            failed_migrations: failed,
            active_migrations: active,
            unique_lineages: self.lineages.len(),
        }
    }
}

impl Default for ConsciousnessMigrationManager {
    fn default() -> Self {
        Self::new()
    }
}

const MIN_ENERGY_MIGRATION: i64 = 50_000;

#[derive(Debug, Clone)]
pub struct MigrationStats {
    pub total_migrations: usize,
    pub successful_migrations: usize,
    pub failed_migrations: usize,
    pub active_migrations: usize,
    pub unique_lineages: usize,
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn generate_package_id() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    let ts = current_timestamp();
    let count = COUNTER.fetch_add(1, Ordering::Relaxed);
    (ts << 20) | (count & 0xFFFFF)
}

fn compute_checksum(data: &[u8]) -> u64 {
    let mut crc: u64 = 0xFFFFFFFFFFFFFFFF;
    for byte in data {
        crc ^= *byte as u64;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB88320_DEADBEEF;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}

fn compute_energy_from_state(state: &[u8]) -> i64 {
    if state.len() >= 8 {
        i64::from_le_bytes(state[..8].try_into().unwrap_or([0u8; 8]))
    } else {
        0
    }
}

fn serialize_for_checksum(package: &ConsciousnessPackage) -> Vec<u8> {
    let mut data = Vec::new();
    data.extend_from_slice(&package.package_id.to_le_bytes());
    data.extend_from_slice(&package.lineage.original_id.to_le_bytes());
    data.extend_from_slice(&package.lineage.generation.to_le_bytes());
    data.extend_from_slice(&package.vital_state.energia.to_le_bytes());
    data.extend_from_slice(&package.created_at.to_le_bytes());
    data
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lineage_creation() {
        let lineage = LineageId::new(0x1234_5678_ABCD, 0x1000);
        
        assert_eq!(lineage.original_id, 0x1234_5678_ABCD);
        assert_eq!(lineage.generation, 0);
        assert_eq!(lineage.path.len(), 1);
        assert_eq!(lineage.current_node(), Some(0x1000));
    }

    #[test]
    fn test_lineage_inheritance() {
        let parent = LineageId::new(0x1234, 0x1000);
        let child = LineageId::inherit(&parent, 0x2000);
        
        assert_eq!(child.original_id, parent.original_id);
        assert_eq!(child.generation, 1);
        assert_eq!(child.path.len(), 2);
        assert_eq!(child.current_node(), Some(0x2000));
        assert_eq!(child.previous_node(), Some(0x1000));
    }

    #[test]
    fn test_lineage_compatibility() {
        let l1 = LineageId::new(0x1234, 0x1000);
        let l2 = LineageId::inherit(&l1, 0x2000);
        
        assert!(l1.is_compatible_with(&l2));
        assert!(l2.is_compatible_with(&l1));
        
        let l3 = LineageId::new(0x5678, 0x3000);
        assert!(!l1.is_compatible_with(&l3));
    }

    #[test]
    fn test_migration_flags() {
        let flags = MigrationFlags::new()
            .with_emergency()
            .with_forced();
        
        assert!(flags.emergency);
        assert!(flags.forced);
        assert!(flags.require_rollback);
    }

    #[test]
    fn test_package_id_generation() {
        let id1 = generate_package_id();
        let id2 = generate_package_id();
        
        assert_ne!(id1, id2);
        assert!(id1 > 0);
        assert!(id2 > 0);
    }

    #[test]
    fn test_vital_state() {
        let state = VitalState::new(100000, 5);
        
        assert_eq!(state.energia, 100000);
        assert_eq!(state.generacion, 5);
        assert_eq!(state.estado, VitalStatus::Alive);
    }

    #[test]
    fn test_migration_phase_states() {
        assert!(!MigrationPhase::Idle.is_active());
        assert!(MigrationPhase::Preparing.is_active());
        assert!(!MigrationPhase::Completed.is_active());
        assert!(!MigrationPhase::Failed.is_active());
    }

    #[test]
    fn test_active_migration_timeout() {
        let mut migration = ActiveMigration::new(0x1234, 0x1000, 0x2000);
        
        migration.started_at = current_timestamp() - MIGRATION_TIMEOUT_MS - 1;
        
        assert!(migration.is_timed_out());
    }

    #[test]
    fn test_migration_result_success() {
        let lineage = LineageId::new(0x1234, 0x1000);
        let result = MigrationResult::success(lineage, 0x2000, 500);
        
        assert!(result.success);
        assert!(result.new_lineage.is_some());
        assert_eq!(result.new_node, Some(0x2000));
        assert_eq!(result.duration_ms, 500);
    }

    #[test]
    fn test_migration_result_failure() {
        let errors = vec!["Connection lost".to_string(), "Timeout".to_string()];
        let result = MigrationResult::failure(errors);
        
        assert!(!result.success);
        assert!(result.new_lineage.is_none());
        assert_eq!(result.errors.len(), 2);
    }

    #[test]
    fn test_manager_creation() {
        let manager = ConsciousnessMigrationManager::new();
        assert_eq!(manager.get_stats().total_migrations, 0);
        assert_eq!(manager.get_stats().active_migrations, 0);
    }
}