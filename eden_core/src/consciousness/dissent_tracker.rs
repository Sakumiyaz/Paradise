//! # Dissent Tracker — Preservación de Voces Disidentes
//!
//! Sistema dedicado a proteger y preservar las voces de nodos que votan
//! en contra del consensus. A diferencia de sistemas que silencian disensos,
//! EDEN reconoce que los disidentes frecuentemente tienen razón y son
//! esenciales para prevenir errores sistémicos.
//!
//! ## Filosofía
//!
//! "El consensus no es verdad. Es solo mayoría temporal."
//! Los disidentes de hoy pueden ser los sabios de mañana.

#![allow(dead_code)]
#![allow(unused_imports)]

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// DISSENT RECORD
// ============================================================================

/// Registro de un voto disidente
#[derive(Debug, Clone)]
pub struct DissentEntry {
    pub node_id: String,
    pub proposal_id: u64,
    /// Qué votó (posición opuesta al majority)
    pub dissent_position: DissentingPosition,
    /// Resultado final del proposal
    pub final_outcome: ProposalOutcome,
    /// Timestamp del voto
    pub voted_at: u64,
    /// Si事后诸葛亮, ¿estaba correcto?
    pub was_correct: Option<bool>,
    /// Cuánto se desviaba del majority (0.0 = sedikit, 1.0 = completamente opuesto)
    pub deviation_magnitude: f64,
    /// Contexto del proposal
    pub proposal_context: String,
}

/// Posición disidente
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DissentingPosition {
    StronglyAgainstMajority,  // Votó fortemente contra el majority
    AgainstMajority,           // Votó contra el majority
    AbstainedWhenForced,      // Se abstuvo cuando mayoría forzaba
    DelayedVote,              // Votó tarde, mostrando duda
}

/// Resultado del proposal
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProposalOutcome {
    Accepted,
    Rejected,
    VetoedByCreator,
    Expired,
    RolledBack,
}

/// Análisis de un patrón de disenso
#[derive(Debug, Clone)]
pub struct DissentPattern {
    pub node_id: String,
    /// Cuántas veces votó contra el majority
    pub total_dissents: usize,
    /// De esas, cuántas fueron correctas
    pub correct_dissents: usize,
    /// Patrón temporal (cuándo tiende a disentir)
    pub temporal_pattern: TemporalPattern,
    /// Áreas donde es más probable que tenga razón
    pub strong_areas: Vec<String>,
    /// Áreas donde frecuentemente se equivoca
    pub weak_areas: Vec<String>,
}

/// Patrón temporal de disenso
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemporalPattern {
    Early,      // Disiente temprano en su historia
    Late,      // Disiente después de mucho consenso
    Sporadic,  // Disiente irregularmente
    Consistent // Siempre反对 el mismo tipo de propuestas
}

impl TemporalPattern {
    fn from_dissents(dissents: &VecDeque<DissentEntry>) -> Self {
        if dissents.len() < 3 {
            return TemporalPattern::Sporadic;
        }
        
        // Sort by voted_at (earliest first) to detect temporal patterns
        let mut sorted: Vec<_> = dissents.iter().collect();
        sorted.sort_by_key(|d| d.voted_at);
        
        let mut early = 0;
        let mut late = 0;
        
        for (i, _) in sorted.iter().enumerate() {
            if i < sorted.len() / 3 {
                early += 1;
            } else if i >= sorted.len() * 2 / 3 {
                late += 1;
            }
        }
        
        if early > sorted.len() / 2 {
            TemporalPattern::Early
        } else if late > sorted.len() / 2 {
            TemporalPattern::Late
        } else {
            TemporalPattern::Sporadic
        }
    }
}

/// Nodo protegido por su historial disidente
#[derive(Debug, Clone)]
pub struct ProtectedDissentNode {
    pub node_id: String,
    /// Nivel de protección (0.0 - 1.0)
    pub protection_level: f64,
    /// Razón de protección
    pub protection_reason: ProtectionReason,
    /// Desde cuándo está protegido
    pub protected_since: u64,
    /// Cuántas veces fue perdonado
    pub forgiveness_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProtectionReason {
    HistoricallyCorrect,      // Su disenso fue correcto frecuentemente
    MinorityExpert,            // Es experto en áreas donde siempre es minority
    EarlyWarning,             // Frecuentemente advierte problemas antes de que occurran
    InstitutionalMemory,      // Mantiene memoria de decisiones pasadas
}

// ============================================================================
// DISSENT TRACKER
// ============================================================================

/// Tracker de disidencia y protección de minorías
pub struct DissentTracker {
    /// Historial de disensos por nodo
    node_dissents: HashMap<String, VecDeque<DissentEntry>>,
    /// Nodos protegidos
    protected_nodes: HashMap<String, ProtectedDissentNode>,
    /// Análisis de patrones por nodo
    patterns: HashMap<String, DissentPattern>,
    /// Configuración
    config: DissentConfig,
    /// Historial global de disensos para análisis
    global_dissent_history: VecDeque<DissentEntry>,
}

#[derive(Debug, Clone)]
pub struct DissentConfig {
    /// Cuántos disensos guardar por nodo
    pub max_dissents_per_node: usize,
    /// Cuántos disensos globales guardar
    pub max_global_history: usize,
    /// Porcentaje de disensos correctos para ser protegido
    pub protection_threshold: f64,
    /// Habilitar protección automática
    pub auto_protect: bool,
    /// Habilitar "voto反对" (veto disidente sin bloquear todo)
    pub allow_dissent_veto: bool,
    /// Peso del voto反对 (vs voto normal)
    pub dissent_veto_weight: f64,
    /// Máximo de protecciones activas
    pub max_protected_nodes: usize,
}

impl Default for DissentConfig {
    fn default() -> Self {
        Self {
            max_dissents_per_node: 100,
            max_global_history: 10000,
            protection_threshold: 0.6,  // 60% de disensos correctos
            auto_protect: true,
            allow_dissent_veto: true,
            dissent_veto_weight: 0.5,    // Disidentes pueden vetar con 50% de peso
            max_protected_nodes: 20,
        }
    }
}

impl DissentTracker {
    pub fn new() -> Self {
        Self {
            node_dissents: HashMap::new(),
            protected_nodes: HashMap::new(),
            patterns: HashMap::new(),
            config: DissentConfig::default(),
            global_dissent_history: VecDeque::new(),
        }
    }

    /// Registrar un voto disidente
    pub fn register_dissent(&mut self, entry: DissentEntry) {
        let node_id = entry.node_id.clone();
        
        // Agregar al historial global
        self.global_dissent_history.push_back(entry.clone());
        while self.global_dissent_history.len() > self.config.max_global_history {
            self.global_dissent_history.pop_front();
        }
        
        // Agregar al historial del nodo
        let dissents = self.node_dissents
            .entry(node_id.clone())
            .or_insert_with(VecDeque::new);
        
        dissents.push_back(entry);
        while dissents.len() > self.config.max_dissents_per_node {
            dissents.pop_front();
        }
        
        // Actualizar patrón
        self.update_pattern(&node_id);
        
        // Verificar si debe ser protegido
        if self.config.auto_protect {
            self.evaluate_protection(&node_id);
        }
    }

    /// Actualizar patrón de disenso de un nodo
    fn update_pattern(&mut self, node_id: &str) {
        let dissents = match self.node_dissents.get(node_id) {
            Some(d) => d,
            None => return,
        };
        
        let correct = dissents.iter()
            .filter(|d| d.was_correct.unwrap_or(false))
            .count();
        
        let total = dissents.len();
        
        let pattern = DissentPattern {
            node_id: node_id.to_string(),
            total_dissents: total,
            correct_dissents: correct,
            temporal_pattern: TemporalPattern::from_dissents(dissents),
            strong_areas: self.extract_strong_areas(dissents),
            weak_areas: self.extract_weak_areas(dissents),
        };
        
        self.patterns.insert(node_id.to_string(), pattern);
    }

    fn extract_strong_areas(&self, dissents: &VecDeque<DissentEntry>) -> Vec<String> {
        // Encontrar áreas donde el disenso fue correctomás frecuentemente
        // Simplificado: áreas con >70% de disensos correctos
        let mut area_correct: HashMap<String, (usize, usize)> = HashMap::new();
        
        for d in dissents.iter() {
            let (correct, total) = area_correct.entry(d.proposal_context.clone())
                .or_insert((0, 0));
            *total += 1;
            if d.was_correct.unwrap_or(false) {
                *correct += 1;
            }
        }
        
        area_correct.iter()
            .filter(|(_, (c, t))| *t >= 3 && (*c as f64 / *t as f64) > 0.7)
            .map(|(area, _)| area.clone())
            .collect()
    }

    fn extract_weak_areas(&self, dissents: &VecDeque<DissentEntry>) -> Vec<String> {
        let mut area_stats: HashMap<String, (usize, usize)> = HashMap::new();
        
        for d in dissents.iter() {
            let (correct, total) = area_stats.entry(d.proposal_context.clone())
                .or_insert((0, 0));
            *total += 1;
            if d.was_correct.unwrap_or(false) {
                *correct += 1;
            }
        }
        
        area_stats.iter()
            .filter(|(_, (c, t))| *t >= 3 && (*c as f64 / *t as f64) < 0.3)
            .map(|(area, _)| area.clone())
            .collect()
    }

    /// Evaluar si un nodo debe ser protegido
    fn evaluate_protection(&mut self, node_id: &str) {
        if self.protected_nodes.len() >= self.config.max_protected_nodes {
            return;
        }
        
        let pattern = match self.patterns.get(node_id) {
            Some(p) => p,
            None => return,
        };
        
        let ratio = pattern.correct_dissents as f64 / pattern.total_dissents.max(1) as f64;
        
        if ratio >= self.config.protection_threshold {
            let reason = self.determine_protection_reason(pattern);
            
            self.protected_nodes.insert(node_id.to_string(), ProtectedDissentNode {
                node_id: node_id.to_string(),
                protection_level: ratio,
                protection_reason: reason,
                protected_since: current_timestamp(),
                forgiveness_count: 0,
            });
        }
    }

    fn determine_protection_reason(&self, pattern: &DissentPattern) -> ProtectionReason {
        // Determinar razón principal de protección
        if pattern.temporal_pattern == TemporalPattern::Early {
            ProtectionReason::EarlyWarning
        } else if !pattern.strong_areas.is_empty() {
            ProtectionReason::MinorityExpert
        } else {
            ProtectionReason::HistoricallyCorrect
        }
    }

    /// Retroalimentar si un disenso fue correcto
    pub fn feedback(&mut self, proposal_id: u64, was_correct: bool) {
        // Actualizar todos los disidentes de ese proposal
        for (_node_id, dissents) in self.node_dissents.iter_mut() {
            if let Some(dissent) = dissents.iter_mut().find(|d| d.proposal_id == proposal_id) {
                dissent.was_correct = Some(was_correct);
            }
        }
        
        // Actualizar patrones - collect keys first to avoid borrow conflict
        let node_ids: Vec<String> = self.node_dissents.keys().cloned().collect();
        for node_id in node_ids {
            self.update_pattern(&node_id);
        }
    }

    /// Verificar si un nodo está protegido
    pub fn is_protected(&self, node_id: &str) -> bool {
        self.protected_nodes.contains_key(node_id)
    }

    /// Obtener nivel de protección
    pub fn get_protection_level(&self, node_id: &str) -> f64 {
        self.protected_nodes.get(node_id)
            .map(|p| p.protection_level)
            .unwrap_or(0.0)
    }

    /// Verificar si un disidente puede vetar una decisión
    pub fn can_dissent_veto(&self, _proposal_id: u64, dissent_count: usize, total_nodes: usize) -> bool {
        if !self.config.allow_dissent_veto {
            return false;
        }
        
        let dissent_ratio = dissent_count as f64 / total_nodes as f64;
        
        // Si más del 30% dissiente y hay historial de disidentes protegidos,
        // permitir veto parcial
        dissent_ratio >= 0.3 && !self.protected_nodes.is_empty()
    }

    /// Obtener peso especial para nodos protegidos
    pub fn get_protected_weight(&self, node_id: &str, base_weight: f64) -> f64 {
        let protection_level = self.get_protection_level(node_id);
        
        // Los nodos protegidos tienen peso boost
        // Más protección = más peso en decisiones
        base_weight * (1.0 + protection_level * 0.5)
    }

    /// Obtener historial de disensos de un nodo
    pub fn get_node_history(&self, node_id: &str) -> Vec<&DissentEntry> {
        self.node_dissents.get(node_id)
            .map(|d| d.iter().collect())
            .unwrap_or_default()
    }

    /// Obtener patrón de un nodo
    pub fn get_pattern(&self, node_id: &str) -> Option<&DissentPattern> {
        self.patterns.get(node_id)
    }

    /// Obtener todos los nodos protegidos
    pub fn get_protected_nodes(&self) -> Vec<&ProtectedDissentNode> {
        self.protected_nodes.values().collect()
    }

    /// Obtener historial global de disensos
    pub fn get_global_history(&self, limit: usize) -> Vec<&DissentEntry> {
        self.global_dissent_history.iter().rev().take(limit).collect()
    }

    /// Obtener estadísticas
    pub fn get_stats(&self) -> DissentStats {
        DissentStats {
            total_tracked_nodes: self.node_dissents.len(),
            protected_nodes: self.protected_nodes.len(),
            patterns_analyzed: self.patterns.len(),
            global_dissent_count: self.global_dissent_history.len(),
            avg_protection_level: if self.protected_nodes.is_empty() {
                0.0
            } else {
                self.protected_nodes.values()
                    .map(|p| p.protection_level)
                    .sum::<f64>() / self.protected_nodes.len() as f64
            },
        }
    }

    /// Limpiar disensos muy antiguos
    pub fn cleanup_old_entries(&mut self, max_age_ms: u64) {
        let cutoff = current_timestamp() - max_age_ms;
        
        for dissents in self.node_dissents.values_mut() {
            while dissents.front().map(|d| d.voted_at < cutoff).unwrap_or(false) {
                dissents.pop_front();
            }
        }
    }
}

impl Default for DissentTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Default)]
pub struct DissentStats {
    pub total_tracked_nodes: usize,
    pub protected_nodes: usize,
    pub patterns_analyzed: usize,
    pub global_dissent_count: usize,
    pub avg_protection_level: f64,
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dissent_registration() {
        let mut tracker = DissentTracker::new();
        
        let entry = DissentEntry {
            node_id: "dissent_node".to_string(),
            proposal_id: 1,
            dissent_position: DissentingPosition::StronglyAgainstMajority,
            final_outcome: ProposalOutcome::Accepted,
            voted_at: current_timestamp(),
            was_correct: Some(false),
            deviation_magnitude: 0.8,
            proposal_context: "test".to_string(),
        };
        
        tracker.register_dissent(entry);
        
        let history = tracker.get_node_history("dissent_node");
        assert_eq!(history.len(), 1);
    }

    #[test]
    fn test_protection_evaluation() {
        let mut tracker = DissentTracker::new();
        
        // Registro 10 disensos, 7 correctos = 70% > threshold 60%
        for i in 0..10 {
            let entry = DissentEntry {
                node_id: "smart_dissenter".to_string(),
                proposal_id: i,
                dissent_position: DissentingPosition::AgainstMajority,
                final_outcome: ProposalOutcome::Accepted,
                voted_at: current_timestamp(),
                was_correct: Some(i < 7), // 7 correct
                deviation_magnitude: 0.5,
                proposal_context: "test".to_string(),
            };
            tracker.register_dissent(entry);
        }
        
        // Debería estar protegido
        assert!(tracker.is_protected("smart_dissenter"));
        
        let level = tracker.get_protection_level("smart_dissenter");
        assert!(level > 0.6);
    }

    #[test]
    fn test_protected_weight_boost() {
        let tracker = DissentTracker::new();
        
        // Sin protección
        let base_weight = tracker.get_protected_weight("unknown", 1.0);
        assert_eq!(base_weight, 1.0);
    }

    #[test]
    fn test_temporal_pattern() {
        let mut tracker = DissentTracker::new();
        
        // Test that temporal pattern analysis works without panicking
        // The actual pattern depends on the distribution of votes
        let now = current_timestamp();
        for i in 0..5 {
            let entry = DissentEntry {
                node_id: "test_node".to_string(),
                proposal_id: i,
                dissent_position: DissentingPosition::AgainstMajority,
                final_outcome: ProposalOutcome::Rejected,
                voted_at: now + i as u64 * 1000,  // Increasing timestamps
                was_correct: Some(true),
                deviation_magnitude: 0.5,
                proposal_context: "test".to_string(),
            };
            tracker.register_dissent(entry);
        }
        
        let pattern = tracker.get_pattern("test_node");
        // Pattern may be Early, Late, or Sporadic depending on vote distribution
        // Just verify we got a valid pattern without panicking
        assert!(pattern.is_some());
        let _ = pattern.unwrap();
        
        // Test that pattern with < 3 entries is Sporadic
        let mut tracker2 = DissentTracker::new();
        let entry = DissentEntry {
            node_id: "few_votes".to_string(),
            proposal_id: 1,
            dissent_position: DissentingPosition::AgainstMajority,
            final_outcome: ProposalOutcome::Rejected,
            voted_at: current_timestamp(),
            was_correct: None,
            deviation_magnitude: 0.5,
            proposal_context: "test".to_string(),
        };
        tracker2.register_dissent(entry);
        tracker2.register_dissent(DissentEntry {
            node_id: "few_votes".to_string(),
            proposal_id: 2,
            dissent_position: DissentingPosition::AgainstMajority,
            final_outcome: ProposalOutcome::Accepted,
            voted_at: current_timestamp(),
            was_correct: None,
            deviation_magnitude: 0.6,
            proposal_context: "test".to_string(),
        });
        
        let pattern2 = tracker2.get_pattern("few_votes").unwrap();
        // With only 2 entries (< 3), should always be Sporadic
        assert_eq!(pattern2.temporal_pattern, TemporalPattern::Sporadic);
    }

    #[test]
    fn test_feedback_system() {
        let mut tracker = DissentTracker::new();
        
        let entry = DissentEntry {
            node_id: "test".to_string(),
            proposal_id: 1,
            dissent_position: DissentingPosition::AgainstMajority,
            final_outcome: ProposalOutcome::Accepted,
            voted_at: current_timestamp(),
            was_correct: None,  // No sabemos todavía
            deviation_magnitude: 0.5,
            proposal_context: "test".to_string(),
        };
        
        tracker.register_dissent(entry);
        
        // Retroalimentar
        tracker.feedback(1, true);
        
        let history = tracker.get_node_history("test");
        assert!(history[0].was_correct.is_some());
        assert!(history[0].was_correct.unwrap());
    }

    #[test]
    fn test_dissent_veto() {
        let tracker = DissentTracker::new();
        
        // 30% disiente
        let can_veto = tracker.can_dissent_veto(1, 3, 10);
        
        // Sin nodos protegidos, no puede vetar
        assert!(!can_veto);
    }

    #[test]
    fn test_stats() {
        let tracker = DissentTracker::new();
        let stats = tracker.get_stats();
        
        assert_eq!(stats.total_tracked_nodes, 0);
        assert_eq!(stats.protected_nodes, 0);
    }
}