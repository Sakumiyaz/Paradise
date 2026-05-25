//! # Quorum Manager — Gestión de Threshold Signatures y Consenso
//!
//! Implementa el sistema de quorum para decisiones críticas:
//! - Threshold signatures (necesitan N firmas para ejecutar)
//! - Verificación de identidad
//! - Multi-quorum (diferentes thresholds para diferentes acciones)
//! - Time-locked recovery
#![allow(dead_code)]
#![allow(non_snake_case)]

use super::{current_timestamp, ThresholdSignature};
use std::collections::{HashMap, HashSet};

// ============================================================================
// QUORUM TYPES
// ============================================================================

/// Nivel de quorum requerido
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QuorumLevel {
    /// Operaciones menores (51% de participantes)
    Minor,
    /// Operaciones normales (66%)
    Standard,
    /// Operaciones importantes (75%)
    Important,
    /// Operaciones críticas (90%)
    Critical,
    /// Operaciones existenciales (100% + Creator)
    Existential,
}

impl QuorumLevel {
    /// Obtener threshold requerido
    pub fn threshold(&self) -> f64 {
        match self {
            QuorumLevel::Minor => 0.51,
            QuorumLevel::Standard => 0.66,
            QuorumLevel::Important => 0.75,
            QuorumLevel::Critical => 0.90,
            QuorumLevel::Existential => 1.00,
        }
    }

    /// Firmas requeridas
    pub fn required_signatures(&self, total_nodes: usize) -> usize {
        let threshold = self.threshold();
        (total_nodes as f64 * threshold).ceil() as usize
    }

    /// Requiere Creator approval
    pub fn requires_creator(&self) -> bool {
        matches!(self, QuorumLevel::Critical | QuorumLevel::Existential)
    }
}

/// Ação que requiere quorum
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum QuorumAction {
    /// Ejecutar propuesta aceptada
    ExecuteProposal,
    /// Modificar threshold de gobernanza
    ModifyThreshold,
    /// Agregar nodo
    AddNode,
    /// Remover nodo
    RemoveNode,
    /// Cambiar protocolo
    ChangeProtocol,
    /// Modificar laws (solo Creator)
    ModifyLaws,
    /// Hacer rollback
    Rollback,
    /// Desactivar sistema
    DisableSystem,
    /// Emergency action
    Emergency,
}

impl QuorumAction {
    /// Nivel de quorum requerido para esta acción
    pub fn required_level(&self) -> QuorumLevel {
        match self {
            QuorumAction::ExecuteProposal => QuorumLevel::Standard,
            QuorumAction::AddNode => QuorumLevel::Standard,
            QuorumAction::ModifyThreshold => QuorumLevel::Important,
            QuorumAction::RemoveNode => QuorumLevel::Important,
            QuorumAction::ChangeProtocol => QuorumLevel::Critical,
            QuorumAction::ModifyLaws => QuorumLevel::Existential,
            QuorumAction::Rollback => QuorumLevel::Standard,
            QuorumAction::DisableSystem => QuorumLevel::Existential,
            QuorumAction::Emergency => QuorumLevel::Minor,
        }
    }
}

// ============================================================================
// SIGNATURE SHARES
// ============================================================================

/// Share de firma (part of threshold signature)
#[derive(Debug, Clone)]
pub struct SignatureShare {
    pub node_id: String,
    pub share: [u8; 32],
    pub timestamp: u64,
}

/// Colección de shares para reconstruir firma
#[derive(Debug, Clone)]
pub struct ThresholdSignatureSet {
    pub action: QuorumAction,
    pub proposal_id: u64,
    pub shares: Vec<SignatureShare>,
    pub created_at: u64,
    pub expires_at: u64,
    /// Firma reconstruida (None hasta que haya enough shares)
    pub reconstructed_signature: Option<[u8; 64]>,
}

impl ThresholdSignatureSet {
    pub fn new(action: QuorumAction, proposal_id: u64, ttl_ms: u64) -> Self {
        let now = current_timestamp();
        Self {
            action,
            proposal_id,
            shares: Vec::new(),
            created_at: now,
            expires_at: now + ttl_ms,
            reconstructed_signature: None,
        }
    }

    /// Agregar share
    pub fn add_share(&mut self, node_id: String, share: [u8; 32]) -> Result<(), QuorumError> {
        if self.is_expired() {
            return Err(QuorumError::SignatureExpired);
        }

        if self.shares.iter().any(|s| s.node_id == node_id) {
            return Err(QuorumError::AlreadySigned);
        }

        self.shares.push(SignatureShare {
            node_id,
            share,
            timestamp: current_timestamp(),
        });

        Ok(())
    }

    /// Verificar si se puede reconstruir
    pub fn can_reconstruct(&self, threshold: usize) -> bool {
        self.shares.len() >= threshold && self.reconstructed_signature.is_none()
    }

    /// Reconstruir firma (simplified)
    pub fn reconstruct(&mut self, threshold: usize) -> Result<[u8; 64], QuorumError> {
        if self.shares.len() < threshold {
            return Err(QuorumError::InsufficientShares {
                required: threshold,
                actual: self.shares.len(),
            });
        }

        // Simplified reconstruction - combine shares
        let mut signature = [0u8; 64];
        for (i, share) in self.shares.iter().take(threshold).enumerate() {
            for (j, &byte) in share.share.iter().enumerate() {
                let idx = (i * 32 + j).min(63);
                signature[idx] = signature[idx].wrapping_add(byte);
            }
        }

        self.reconstructed_signature = Some(signature);
        Ok(signature)
    }

    /// Verificar si expiró
    pub fn is_expired(&self) -> bool {
        current_timestamp() > self.expires_at
    }

    /// Obtener número de shares
    pub fn share_count(&self) -> usize {
        self.shares.len()
    }
}

// ============================================================================
// QUORUM MANAGER
// ============================================================================

/// Gestor de quorum y threshold signatures
pub struct QuorumManager {
    /// Nodos activos en la red
    active_nodes: HashSet<String>,
    /// Threshold signature sets pendientes
    pending_signatures: HashMap<u64, ThresholdSignatureSet>,
    /// Firmas completadas
    completed_signatures: HashMap<u64, CompletedSignature>,
    /// Configuración
    config: QuorumConfig,
    /// Historial de quorum
    history: Vec<QuorumRecord>,
}

#[derive(Debug, Clone)]
pub struct QuorumConfig {
    /// Tiempo de expiración de shares (ms)
    pub share_timeout_ms: u64,
    /// Firmas mínimas para quorum estándar
    pub min_signatures: usize,
    /// Habilitar time-locked recovery
    pub enable_timelock: bool,
    /// Tiempo de timelock para emergencia (ms)
    pub emergency_timelock_ms: u64,
    /// Permitir skip de quorum para emergency
    pub allow_emergency_skip: bool,
}

impl Default for QuorumConfig {
    fn default() -> Self {
        Self {
            share_timeout_ms: 86400000, // 24 hours
            min_signatures: 3,
            enable_timelock: true,
            emergency_timelock_ms: 300000, // 5 minutes
            allow_emergency_skip: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompletedSignature {
    pub proposal_id: u64,
    pub action: QuorumAction,
    pub signature: [u8; 64],
    pub signers: Vec<String>,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct QuorumRecord {
    pub proposal_id: u64,
    pub action: QuorumAction,
    pub quorum_level: QuorumLevel,
    pub required_signatures: usize,
    pub actual_signatures: usize,
    pub success: bool,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub enum QuorumError {
    InsufficientShares { required: usize, actual: usize },
    SignatureExpired,
    AlreadySigned,
    ActionNotAllowed,
    QuorumNotReached,
    InvalidSignature,
    NodeNotActive,
}

impl std::fmt::Display for QuorumError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QuorumError::InsufficientShares { required, actual } => {
                write!(f, "Insufficient shares: need {}, got {}", required, actual)
            }
            QuorumError::SignatureExpired => write!(f, "Signature collection expired"),
            QuorumError::AlreadySigned => write!(f, "Node has already signed"),
            QuorumError::ActionNotAllowed => write!(f, "Action not allowed at this time"),
            QuorumError::QuorumNotReached => write!(f, "Quorum not reached"),
            QuorumError::InvalidSignature => write!(f, "Invalid signature"),
            QuorumError::NodeNotActive => write!(f, "Node is not active"),
        }
    }
}

impl std::error::Error for QuorumError {}

impl QuorumManager {
    /// Crear nuevo manager
    pub fn new() -> Self {
        Self {
            active_nodes: HashSet::new(),
            pending_signatures: HashMap::new(),
            completed_signatures: HashMap::new(),
            config: QuorumConfig::default(),
            history: Vec::new(),
        }
    }

    /// Agregar nodo activo
    pub fn add_node(&mut self, node_id: &str) {
        self.active_nodes.insert(node_id.to_string());
    }

    /// Remover nodo activo
    pub fn remove_node(&mut self, node_id: &str) {
        self.active_nodes.remove(node_id);
    }

    /// Verificar si nodo está activo
    pub fn is_node_active(&self, node_id: &str) -> bool {
        self.active_nodes.contains(node_id)
    }

    /// Obtener cantidad de nodos activos
    pub fn active_node_count(&self) -> usize {
        self.active_nodes.len()
    }

    /// Iniciar colección de firmas para una acción
    pub fn initiate_signature_collection(
        &mut self,
        proposal_id: u64,
        action: QuorumAction,
    ) -> Result<&ThresholdSignatureSet, QuorumError> {
        // Verificar que no exista ya
        if self.pending_signatures.contains_key(&proposal_id) {
            return Err(QuorumError::AlreadySigned);
        }

        let ttl = match action {
            QuorumAction::Emergency => self.config.emergency_timelock_ms,
            _ => self.config.share_timeout_ms,
        };

        let set = ThresholdSignatureSet::new(action, proposal_id, ttl);
        self.pending_signatures.insert(proposal_id, set);

        Ok(self.pending_signatures.get(&proposal_id).unwrap())
    }

    /// Agregar share a colección
    pub fn add_share(
        &mut self,
        proposal_id: u64,
        node_id: String,
        share: [u8; 32],
    ) -> Result<(), QuorumError> {
        // Verificar nodo activo
        if !self.is_node_active(&node_id) {
            return Err(QuorumError::NodeNotActive);
        }

        let set = self.pending_signatures.get_mut(&proposal_id)
            .ok_or(QuorumError::ActionNotAllowed)?;

        set.add_share(node_id, share)
    }

    /// Verificar si se alcanzó quorum
    pub fn check_quorum(&self, proposal_id: u64) -> bool {
        let set = match self.pending_signatures.get(&proposal_id) {
            Some(s) => s,
            None => return false,
        };

        let required = self.config.min_signatures;
        set.share_count() >= required
    }

    /// Reconstruir y completar firma
    pub fn complete_signature(
        &mut self,
        proposal_id: u64,
    ) -> Result<CompletedSignature, QuorumError> {
        let set = self.pending_signatures.remove(&proposal_id)
            .ok_or(QuorumError::ActionNotAllowed)?;

        if set.is_expired() {
            return Err(QuorumError::SignatureExpired);
        }

        let required = self.config.min_signatures;
        let signature = set.reconstruct(required)?;

        let signers: Vec<String> = set.shares.iter().map(|s| s.node_id.clone()).collect();

        let completed = CompletedSignature {
            proposal_id,
            action: set.action,
            signature,
            signers: signers.clone(),
            timestamp: current_timestamp(),
        };

        self.completed_signatures.insert(proposal_id, completed.clone());

        // Registrar en historial
        self.history.push(QuorumRecord {
            proposal_id,
            action: set.action,
            quorum_level: set.action.required_level(),
            required_signatures: required,
            actual_signatures: set.shares.len(),
            success: true,
            timestamp: current_timestamp(),
        });

        // Limitar tamaño del historial
        if self.history.len() > 10000 {
            self.history.remove(0);
        }

        Ok(completed)
    }

    /// Verificar si acción está permitida
    pub fn can_execute(&self, action: QuorumAction, proposal_id: u64) -> Result<(), QuorumError> {
        let level = action.required_level();
        let total_nodes = self.active_node_count();

        // Verificar que hay suficiente nodos
        let required = level.required_signatures(total_nodes);
        if required > total_nodes {
            return Err(QuorumError::InsufficientShares {
                required,
                actual: total_nodes,
            });
        }

        // Verificar signature completada
        if let Some(sig) = self.completed_signatures.get(&proposal_id) {
            if sig.action == action {
                return Ok(());
            }
        }

        // Verificar que tiene los shares necesarios
        if let Some(set) = self.pending_signatures.get(&proposal_id) {
            if set.share_count() >= required {
                return Ok(());
            }
        }

        Err(QuorumError::QuorumNotReached)
    }

    /// Obtener propuesta de firma pendiente
    pub fn get_pending(&self, proposal_id: u64) -> Option<&ThresholdSignatureSet> {
        self.pending_signatures.get(&proposal_id)
    }

    /// Obtener firma completada
    pub fn get_completed(&self, proposal_id: u64) -> Option<&CompletedSignature> {
        self.completed_signatures.get(&proposal_id)
    }

    /// Obtener historial
    pub fn get_history(&self) -> &[QuorumRecord] {
        &self.history
    }

    /// Limpiar firmas expiradas
    pub fn cleanup_expired(&mut self) {
        self.pending_signatures.retain(|_, set| !set.is_expired());
    }

    /// Obtener estadísticas
    pub fn get_stats(&self) -> QuorumStats {
        QuorumStats {
            active_nodes: self.active_nodes.len(),
            pending_signatures: self.pending_signatures.len(),
            completed_signatures: self.completed_signatures.len(),
            total_quorums: self.history.len(),
        }
    }
}

impl Default for QuorumManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Default)]
pub struct QuorumStats {
    pub active_nodes: usize,
    pub pending_signatures: usize,
    pub completed_signatures: usize,
    pub total_quorums: usize,
}

// ============================================================================
// MULTI-QUORUM (para diferentes niveles)
// ============================================================================

/// Sistema de multi-quorum (diferentes grupos de nodos)
pub struct MultiQuorum {
    /// Grupos de quorum
    groups: HashMap<String, QuorumGroup>,
    /// Grupo por defecto
    default_group: String,
}

#[derive(Debug, Clone)]
pub struct QuorumGroup {
    pub name: String,
    pub members: HashSet<String>,
    pub threshold: usize,
}

impl MultiQuorum {
    pub fn new(default_name: &str) -> Self {
        let mut groups = HashMap::new();
        groups.insert(default_name.to_string(), QuorumGroup {
            name: default_name.to_string(),
            members: HashSet::new(),
            threshold: 3,
        });

        Self {
            groups,
            default_group: default_name.to_string(),
        }
    }

    /// Agregar grupo
    pub fn add_group(&mut self, name: &str, threshold: usize) {
        self.groups.insert(name.to_string(), QuorumGroup {
            name: name.to_string(),
            members: HashSet::new(),
            threshold,
        });
    }

    /// Agregar miembro a grupo
    pub fn add_member(&mut self, group_name: &str, node_id: &str) -> Result<(), QuorumError> {
        let group = self.groups.get_mut(group_name)
            .ok_or(QuorumError::NodeNotActive)?;
        
        group.members.insert(node_id.to_string());
        Ok(())
    }

    /// Verificar quorum en grupo
    pub fn check_group_quorum(&self, group_name: &str, shares: usize) -> bool {
        self.groups
            .get(group_name)
            .map(|g| shares >= g.threshold)
            .unwrap_or(false)
    }

    /// Obtener grupo de un nodo
    pub fn get_node_group(&self, node_id: &str) -> Option<&str> {
        for (name, group) in &self.groups {
            if group.members.contains(node_id) {
                return Some(name);
            }
        }
        None
    }
}

// ============================================================================
// TIMELOCK RECOVERY
// ============================================================================

/// Sistema de timelock para recovery de emergencia
pub struct TimelockRecovery {
    /// Locked actions esperando
    locked_actions: HashMap<u64, TimelockEntry>,
    /// Duración default del lock (ms)
    default_lock_ms: u64,
}

#[derive(Debug, Clone)]
pub struct TimelockEntry {
    pub proposal_id: u64,
    pub action: QuorumAction,
    pub initiator: String,
    pub created_at: u64,
    pub unlock_at: u64,
    pub status: TimelockStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TimelockStatus {
    Locked,
    Unlocked,
    Cancelled,
    Executed,
}

impl TimelockRecovery {
    pub fn new(default_lock_ms: u64) -> Self {
        Self {
            locked_actions: HashMap::new(),
            default_lock_ms,
        }
    }

    /// Crear timelock (para acciones que requieren delay)
    pub fn create_lock(
        &mut self,
        proposal_id: u64,
        action: QuorumAction,
        initiator: String,
        duration_ms: Option<u64>,
    ) -> Result<u64, QuorumError> {
        let now = current_timestamp();
        let duration = duration_ms.unwrap_or(self.default_lock_ms);

        let unlock_at = now + duration;

        let entry = TimelockEntry {
            proposal_id,
            action,
            initiator,
            created_at: now,
            unlock_at,
            status: TimelockStatus::Locked,
        };

        self.locked_actions.insert(proposal_id, entry);
        Ok(unlock_at)
    }

    /// Verificar si acción está unlocked
    pub fn is_unlocked(&self, proposal_id: u64) -> bool {
        self.locked_actions
            .get(&proposal_id)
            .map(|e| {
                current_timestamp() >= e.unlock_at && e.status == TimelockStatus::Locked
            })
            .unwrap_or(true) // Si no existe, está unlocked
    }

    /// Unlock manualmente (para cancelaciones)
    pub fn unlock(&mut self, proposal_id: u64) -> Result<(), QuorumError> {
        let entry = self.locked_actions.get_mut(&proposal_id)
            .ok_or(QuorumError::ActionNotAllowed)?;

        entry.status = TimelockStatus::Unlocked;
        Ok(())
    }

    /// Cancelar timelock
    pub fn cancel(&mut self, proposal_id: u64) -> Result<(), QuorumError> {
        let entry = self.locked_actions.get_mut(&proposal_id)
            .ok_or(QuorumError::ActionNotAllowed)?;

        entry.status = TimelockStatus::Cancelled;
        Ok(())
    }

    /// Ejecutar acción después del lock
    pub fn execute_after_lock(
        &mut self,
        proposal_id: u64,
    ) -> Result<TimelockEntry, QuorumError> {
        let entry = self.locked_actions.get_mut(&proposal_id)
            .ok_or(QuorumError::ActionNotAllowed)?;

        if !self.is_unlocked(proposal_id) {
            return Err(QuorumError::ActionNotAllowed);
        }

        entry.status = TimelockStatus::Executed;
        Ok(entry.clone())
    }

    /// Obtener tiempo restante hasta unlock
    pub fn time_remaining(&self, proposal_id: u64) -> Option<u64> {
        self.locked_actions
            .get(&proposal_id)
            .map(|e| {
                e.unlock_at.saturating_sub(current_timestamp())
            })
    }
}

impl Default for TimelockRecovery {
    fn default() -> Self {
        Self::new(86400000) // 24 hours default
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quorum_levels() {
        assert_eq!(QuorumLevel::Minor.threshold(), 0.51);
        assert_eq!(QuorumLevel::Standard.threshold(), 0.66);
        assert_eq!(QuorumLevel::Existential.threshold(), 1.00);
    }

    #[test]
    fn test_action_requirements() {
        assert_eq!(QuorumAction::ExecuteProposal.required_level(), QuorumLevel::Standard);
        assert_eq!(QuorumAction::ModifyLaws.required_level(), QuorumLevel::Existential);
    }

    #[test]
    fn test_signature_collection() {
        let mut manager = QuorumManager::new();
        
        manager.add_node("node1");
        manager.add_node("node2");
        manager.add_node("node3");
        manager.add_node("node4");
        
        // Iniciar colección
        manager.initiate_signature_collection(1, QuorumAction::ExecuteProposal).unwrap();
        
        // Agregar shares
        manager.add_share(1, "node1".to_string(), [1u8; 32]).unwrap();
        manager.add_share(1, "node2".to_string(), [2u8; 32]).unwrap();
        manager.add_share(1, "node3".to_string(), [3u8; 32]).unwrap();
        
        // Verificar quorum
        assert!(manager.check_quorum(1));
        
        // Completar firma
        let completed = manager.complete_signature(1).unwrap();
        assert_eq!(completed.signers.len(), 3);
    }

    #[test]
    fn test_timelock() {
        let mut recovery = TimelockRecovery::new(1000); // 1 second
        
        // Crear lock
        let unlock_at = recovery.create_lock(
            1,
            QuorumAction::ChangeProtocol,
            "creator".to_string(),
            Some(100),
        ).unwrap();
        
        // Debería estar locked todavía
        assert!(!recovery.is_unlocked(1));
        
        // Wait and check
        std::thread::sleep(std::time::Duration::from_millis(150));
        assert!(recovery.is_unlocked(1));
    }

    #[test]
    fn test_multi_quorum() {
        let mut mq = MultiQuorum::new("default");
        
        mq.add_group("critical", 2).unwrap();
        mq.add_group("standard", 3).unwrap();
        
        mq.add_member("critical", "node1").unwrap();
        mq.add_member("critical", "node2").unwrap();
        
        mq.add_member("standard", "node3").unwrap();
        mq.add_member("standard", "node4").unwrap();
        mq.add_member("standard", "node5").unwrap();
        
        assert!(mq.check_group_quorum("critical", 2));
        assert!(!mq.check_group_quorum("standard", 2)); // Need 3
        
        assert_eq!(mq.get_node_group("node1"), Some("critical"));
    }
}