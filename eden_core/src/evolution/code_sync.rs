//! # Code Sync — Sincronización y Evolución Distribuida del Código
//!
//! Este módulo implementa EdenFS distribuido: la capacidad de sincronizar
//! parches de código entre nodos mientras se mantiene la integridad y se
//! permite rollback.
//!
//! ## Principios
//!
//! - Solo código en `.eden_patchable` puede syncarse
//! - Cada cambio requiere checksum y validación
//! - Rollback automático si la validación falla
//! - Logs en Meltrace para audit trail

#![allow(dead_code)]
#![allow(unused_imports)]

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// ERROR TYPES
// ============================================================================

#[derive(Debug, Clone)]
pub enum CodeSyncError {
    RegionNotPatchable(String),
    ChecksumMismatch { expected: u64, actual: u64 },
    NodeUnreachable(u64),
    RollbackFailed(String),
    ValidationFailed(String),
    IncompatibleVersion { expected: u8, actual: u8 },
    ResourceLimitExceeded(String),
}

impl std::fmt::Display for CodeSyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodeSyncError::RegionNotPatchable(r) => write!(f, "Region {} is not patchable", r),
            CodeSyncError::ChecksumMismatch { expected, actual } => {
                write!(f, "Checksum mismatch: expected {:016x}, got {:016x}", expected, actual)
            }
            CodeSyncError::NodeUnreachable(id) => write!(f, "Node {} unreachable", id),
            CodeSyncError::RollbackFailed(s) => write!(f, "Rollback failed: {}", s),
            CodeSyncError::ValidationFailed(s) => write!(f, "Validation failed: {}", s),
            CodeSyncError::IncompatibleVersion { expected, actual } => {
                write!(f, "Incompatible version: expected {}, got {}", expected, actual)
            }
            CodeSyncError::ResourceLimitExceeded(s) => write!(f, "Resource limit: {}", s),
        }
    }
}

impl std::error::Error for CodeSyncError {}

// ============================================================================
// GLOBAL STATE
// ============================================================================

/// Estado global de sincronizacion de codigo
pub struct CodeSyncState {
    pub protocol_version: u8,
    local_patches: HashMap<usize, AppliedPatch>,
    pending_patches: Vec<Parche>,
    patch_history: Vec<PatchRecord>,
    my_patches: HashSet<u64>,
}

#[derive(Debug, Clone)]
pub struct AppliedPatch {
    pub patch_id: u64,
    pub target: usize,
    pub applied_at: u64,
    pub source_node: Option<u64>,
    pub hash: u64,
    pub rollback_data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Parche {
    pub id: u64,
    pub nombre_funcion: String,
    pub destino: usize,
    pub codigo: Vec<u8>,
    pub estado: PatchState,
    pub hash_parche: u64,
    pub tick_propuesto: u64,
    pub tick_aplicado: u64,
    pub id_auton_iluminado: Option<u64>,
    pub descripcion: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatchState {
    Propuesto,
    Verificado,
    Aplicado,
    Rechazado,
    Revertido,
}

#[derive(Debug, Clone)]
pub struct PatchRecord {
    pub patch_id: u64,
    pub timestamp: u64,
    pub target: String,
    pub action: PatchAction,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum PatchAction {
    Applied,
    Reverted,
    Rejected,
    RollbackFailed,
}

const CURRENT_SYNC_VERSION: u8 = 0x01;
const MAX_PENDING_PATCHES: usize = 100;
const MAX_PATCH_HISTORY: usize = 1000;

impl CodeSyncState {
    pub fn new() -> Self {
        CodeSyncState {
            protocol_version: CURRENT_SYNC_VERSION,
            local_patches: HashMap::new(),
            pending_patches: Vec::new(),
            patch_history: Vec::new(),
            my_patches: HashSet::new(),
        }
    }

    pub fn record_patch(&mut self, record: PatchRecord) {
        self.patch_history.push(record);
        if self.patch_history.len() > MAX_PATCH_HISTORY {
            self.patch_history.remove(0);
        }
    }
}

impl Default for CodeSyncState {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// CODE DIFF
// ============================================================================

/// Representa un diff de codigo entre versiones
#[derive(Debug, Clone)]
pub struct CodeDiff {
    pub target: usize,
    pub function_name: String,
    pub original_hash: u64,
    pub new_hash: u64,
    pub new_code: Vec<u8>,
    pub diff_size: usize,
    pub checksum: u64,
}

impl CodeDiff {
    pub fn new(
        target: usize,
        function_name: &str,
        original_hash: u64,
        new_code: Vec<u8>,
    ) -> Result<Self, CodeSyncError> {
        let new_hash = compute_hash(&new_code);
        let checksum = compute_checksum(&new_code);
        let diff_size = new_code.len();

        Ok(CodeDiff {
            target,
            function_name: function_name.to_string(),
            original_hash,
            new_hash,
            new_code,
            diff_size,
            checksum,
        })
    }

    pub fn verify(&self) -> bool {
        compute_hash(&self.new_code) == self.new_hash && compute_checksum(&self.new_code) == self.checksum
    }
}

// ============================================================================
// ROLLBACK MANAGER
// ============================================================================

pub struct RollbackManager {
    rollback_data: HashMap<usize, Vec<u8>>,
    patch_order: Vec<(usize, u64)>,
    max_depth: usize,
}

impl RollbackManager {
    pub fn new(max_depth: usize) -> Self {
        RollbackManager {
            rollback_data: HashMap::new(),
            patch_order: Vec::new(),
            max_depth,
        }
    }

    pub fn save_rollback(&mut self, target: usize, original_code: &[u8]) {
        self.rollback_data.insert(target, original_code.to_vec());
        self.patch_order.push((target, current_timestamp()));
        
        while self.patch_order.len() > self.max_depth {
            if let Some((old_target, _)) = self.patch_order.first() {
                self.rollback_data.remove(old_target);
                self.patch_order.remove(0);
            }
        }
    }

    pub fn get_rollback(&self, target: &usize) -> Option<Vec<u8>> {
        self.rollback_data.get(target).cloned()
    }

    pub fn rollback(&mut self, target: &usize) -> Result<Vec<u8>, CodeSyncError> {
        self.rollback_data
            .remove(target)
            .ok_or_else(|| CodeSyncError::RollbackFailed(format!("No rollback data for {:?}", target)))
    }

    pub fn clear_rollback(&mut self, target: &usize) {
        self.rollback_data.remove(target);
        self.patch_order.retain(|(t, _)| t != target);
    }
}

// ============================================================================
// SYNC PROTOCOL
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncMode {
    FullSync,
    DeltaSync,
    EmergencySync,
}

impl SyncMode {
    pub fn to_byte(&self) -> u8 {
        match self {
            SyncMode::FullSync => 0x01,
            SyncMode::DeltaSync => 0x02,
            SyncMode::EmergencySync => 0xFF,
        }
    }

    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0x01 => Some(SyncMode::FullSync),
            0x02 => Some(SyncMode::DeltaSync),
            0xFF => Some(SyncMode::EmergencySync),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SyncMessage {
    pub mode: SyncMode,
    pub patches: Vec<CodeDiff>,
    pub request_sync: bool,
    pub from_node: u64,
    pub timestamp: u64,
}

impl SyncMessage {
    pub fn new(mode: SyncMode, from_node: u64) -> Self {
        SyncMessage {
            mode,
            patches: Vec::new(),
            request_sync: false,
            from_node,
            timestamp: current_timestamp(),
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(self.mode.to_byte());
        bytes.push(self.from_node as u8);
        bytes.extend_from_slice(&self.timestamp.to_le_bytes());
        bytes.push(if self.request_sync { 1 } else { 0 });
        bytes.extend_from_slice(&(self.patches.len() as u32).to_le_bytes());
        for patch in &self.patches {
            bytes.extend_from_slice(&patch.target.to_le_bytes());
            bytes.extend_from_slice(&patch.new_hash.to_le_bytes());
            bytes.extend_from_slice(&patch.checksum.to_le_bytes());
            bytes.extend_from_slice(&(patch.new_code.len() as u32).to_le_bytes());
            bytes.extend_from_slice(&patch.new_code);
        }
        bytes
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 12 { return None; }
        
        let mode = SyncMode::from_byte(bytes[0])?;
        let from_node = bytes[1] as u64;
        let timestamp = u64::from_le_bytes(bytes[2..10].try_into().ok()?);
        let request_sync = bytes[10] != 0;
        let num_patches = u32::from_le_bytes(bytes[11..15].try_into().ok()?) as usize;
        
        let mut patches = Vec::new();
        let mut pos = 15;
        
        for _ in 0..num_patches {
            if pos + 20 > bytes.len() { return None; }
            let target = usize::from_le_bytes(bytes[pos..pos+8].try_into().ok()?);
            let new_hash = u64::from_le_bytes(bytes[pos+8..pos+16].try_into().ok()?);
            let checksum = u64::from_le_bytes(bytes[pos+16..pos+24].try_into().ok()?);
            let code_len = u32::from_le_bytes(bytes[pos+24..pos+28].try_into().ok()?) as usize;
            pos += 28;
            
            if pos + code_len > bytes.len() { return None; }
            let new_code = bytes[pos..pos+code_len].to_vec();
            pos += code_len;
            
            patches.push(CodeDiff {
                target,
                function_name: String::new(),
                original_hash: 0,
                new_hash,
                new_code,
                diff_size: code_len,
                checksum,
            });
        }
        
        Some(SyncMessage {
            mode,
            patches,
            request_sync,
            from_node,
            timestamp,
        })
    }
}

#[derive(Debug, Clone)]
pub struct NodeSyncState {
    pub node_id: u64,
    pub last_sync: u64,
    pub pending_diffs: Vec<CodeDiff>,
    pub acknowledged_patches: HashSet<u64>,
    pub version: u8,
}

impl NodeSyncState {
    pub fn new(node_id: u64) -> Self {
        NodeSyncState {
            node_id,
            last_sync: 0,
            pending_diffs: Vec::new(),
            acknowledged_patches: HashSet::new(),
            version: CURRENT_SYNC_VERSION,
        }
    }

    pub fn needs_sync(&self) -> bool {
        !self.pending_diffs.is_empty() || current_timestamp() - self.last_sync > SYNC_INTERVAL_MS
    }
}

const SYNC_INTERVAL_MS: u64 = 5000;

// ============================================================================
// CODE SYNC MANAGER
// ============================================================================

pub struct CodeSyncManager {
    state: Arc<RwLock<CodeSyncState>>,
    rollback: Arc<RwLock<RollbackManager>>,
    peer_states: HashMap<u64, NodeSyncState>,
    patchables: HashMap<usize, PatchableInfo>,
    apply_callback: Option<Box<dyn Fn(usize, &[u8]) -> Result<(), String> + Send + Sync>>,
}

#[derive(Debug, Clone)]
pub struct PatchableInfo {
    pub nombre: String,
    pub tamano: usize,
    pub hash_original: u64,
}

impl CodeSyncManager {
    pub fn new() -> Self {
        CodeSyncManager {
            state: Arc::new(RwLock::new(CodeSyncState::new())),
            rollback: Arc::new(RwLock::new(RollbackManager::new(100))),
            peer_states: HashMap::new(),
            patchables: HashMap::new(),
            apply_callback: None,
        }
    }

    pub fn register_patchable(&mut self, addr: usize, info: PatchableInfo) {
        self.patchables.insert(addr, info);
    }

    pub fn set_apply_callback<F>(&mut self, callback: F)
    where
        F: Fn(usize, &[u8]) -> Result<(), String> + Send + Sync + 'static,
    {
        self.apply_callback = Some(Box::new(callback));
    }

    pub fn receive_diff(&self, diff: CodeDiff, _source_node: u64) -> Result<(), CodeSyncError> {
        let computed = compute_checksum(&diff.new_code);
        if computed != diff.checksum {
            return Err(CodeSyncError::ChecksumMismatch {
                expected: diff.checksum,
                actual: computed,
            });
        }

        let original_code = self.rollback
            .read()
            .ok()
            .and_then(|r| r.get_rollback(&diff.target))
            .unwrap_or_default();

        if let Some(ref callback) = self.apply_callback {
            callback(diff.target, &diff.new_code)
                .map_err(|e| CodeSyncError::ValidationFailed(e))?;
        }

        if let Some(mut rollback) = self.rollback.write().ok() {
            rollback.save_rollback(diff.target, &original_code);
        }

        let record = PatchRecord {
            patch_id: diff.new_hash,
            timestamp: current_timestamp(),
            target: diff.function_name.clone(),
            action: PatchAction::Applied,
            success: true,
            error: None,
        };

        if let Some(mut state) = self.state.write().ok() {
            state.record_patch(record);
        }

        Ok(())
    }

    pub fn request_sync(&mut self, node_id: u64) {
        if let Some(state) = self.peer_states.get_mut(&node_id) {
            state.last_sync = current_timestamp();
        }
    }

    pub fn generate_sync_message(&self, target_node: u64) -> Option<SyncMessage> {
        let state = self.state.read().ok()?;
        let peer_state = self.peer_states.get(&target_node)?;

        let mut msg = SyncMessage::new(SyncMode::DeltaSync, 0);
        
        for (addr, patch) in &state.local_patches {
            if !peer_state.acknowledged_patches.contains(&patch.patch_id) {
                if let Some(patchable) = self.patchables.get(addr) {
                    let diff = CodeDiff {
                        target: *addr,
                        function_name: patchable.nombre.clone(),
                        original_hash: patchable.hash_original,
                        new_hash: patch.hash,
                        new_code: Vec::new(),
                        diff_size: patchable.tamano,
                        checksum: patch.hash,
                    };
                    msg.patches.push(diff);
                }
            }
        }

        msg.request_sync = peer_state.needs_sync();
        Some(msg)
    }

    pub fn acknowledge_patch(&mut self, node_id: u64, patch_id: u64) {
        if let Some(state) = self.peer_states.get_mut(&node_id) {
            state.acknowledged_patches.insert(patch_id);
            state.pending_diffs.retain(|d| d.new_hash != patch_id);
        }
    }

    pub fn rollback_patch(&mut self, target: &usize) -> Result<(), CodeSyncError> {
        let original_code = self.rollback
            .write()
            .map_err(|_| CodeSyncError::RollbackFailed("Lock failed".to_string()))?
            .rollback(target)?;

        if let Some(ref callback) = self.apply_callback {
            callback(*target, &original_code)
                .map_err(|e| CodeSyncError::ValidationFailed(e))?;
        }

        let record = PatchRecord {
            patch_id: 0,
            timestamp: current_timestamp(),
            target: format!("{:?}", target),
            action: PatchAction::Reverted,
            success: true,
            error: None,
        };

        self.state.write().map(|mut s| s.record_patch(record)).ok();

        Ok(())
    }

    pub fn can_apply(&self, diff: &CodeDiff) -> bool {
        diff.diff_size <= MAX_MEMORY_FOR_PATCH
    }
}

impl Default for CodeSyncManager {
    fn default() -> Self {
        Self::new()
    }
}

const MAX_MEMORY_FOR_PATCH: usize = 1_048_576;

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn compute_hash(data: &[u8]) -> u64 {
    let mut h: u64 = 0xC0DE;
    for (i, &b) in data.iter().enumerate() {
        h = h.wrapping_mul(0x100000001B3);
        h ^= b as u64;
        h = h.rotate_left((i % 64) as u32);
    }
    h
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

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_diff_creation() {
        let diff = CodeDiff::new(
            0x1000,
            "test_function",
            0xABCDEF,
            vec![0x90, 0x91, 0x92],
        );
        
        assert!(diff.is_ok());
        let diff = diff.unwrap();
        assert_eq!(diff.target, 0x1000);
        assert!(diff.verify());
    }

    #[test]
    fn test_sync_message_serialization() {
        let mut msg = SyncMessage::new(SyncMode::DeltaSync, 42);
        msg.request_sync = true;
        
        let bytes = msg.to_bytes();
        let parsed = SyncMessage::from_bytes(&bytes).unwrap();
        
        assert_eq!(parsed.mode, SyncMode::DeltaSync);
        assert_eq!(parsed.from_node, 42);
        assert!(parsed.request_sync);
    }

    #[test]
    fn test_rollback_manager() {
        let mut manager = RollbackManager::new(5);
        
        manager.save_rollback(0x1000, &[1, 2, 3]);
        manager.save_rollback(0x2000, &[4, 5, 6]);
        
        let rollback = manager.rollback(&0x1000);
        assert!(rollback.is_ok());
        assert_eq!(rollback.unwrap(), vec![1, 2, 3]);
        
        let rollback2 = manager.rollback(&0x1000);
        assert!(rollback2.is_err());
    }

    #[test]
    fn test_checksum() {
        let data = [0xDE, 0xAD, 0xBE, 0xEF];
        let checksum = compute_checksum(&data);
        assert_ne!(checksum, 0);
        
        let data2 = [0xFF, 0xEE, 0xDD, 0xCC];
        let checksum2 = compute_checksum(&data2);
        assert_ne!(checksum, checksum2);
    }

    #[test]
    fn test_sync_mode_bytes() {
        assert_eq!(SyncMode::FullSync.to_byte(), 0x01);
        assert_eq!(SyncMode::DeltaSync.to_byte(), 0x02);
        assert_eq!(SyncMode::EmergencySync.to_byte(), 0xFF);
        
        assert_eq!(SyncMode::from_byte(0x01), Some(SyncMode::FullSync));
        assert_eq!(SyncMode::from_byte(0x02), Some(SyncMode::DeltaSync));
        assert_eq!(SyncMode::from_byte(0xFF), Some(SyncMode::EmergencySync));
        assert_eq!(SyncMode::from_byte(0x99), None);
    }

    #[test]
    fn test_node_sync_state() {
        let state = NodeSyncState::new(0x1234);
        assert!(state.needs_sync());
        
        let mut state = state;
        state.last_sync = current_timestamp();
        assert!(!state.needs_sync());
    }
}