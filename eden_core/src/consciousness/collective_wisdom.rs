//! # Collective Wisdom — Sabiduría Colectiva sin Fusión
//!
//! Este módulo implementa la agregación de conocimiento colectivo
//! sin perder la diversidad de nodos individuales. A diferencia de
//! "fusión de identidad", aquí cada nodo mantiene su autonomía
//! mientras contribuye a una comprensión más amplia del sistema.
//!
//! ## Filosofía
//!
//! La mente global NO es "un cerebro" sino "un ecosistema de cerebros
//! coordinados". Cada nodo aporta su perspectiva única, y el sistema
//! agrega estas perspectivas para tomar decisiones más sabias.

#![allow(dead_code)]
#![allow(unused_imports)]

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// WISDOM AGGREGATION
// ============================================================================

/// Perspectiva de un nodo sobre el sistema
#[derive(Debug, Clone)]
pub struct NodePerspective {
    pub node_id: String,
    pub understanding: f64,           // Nivel de comprensión (0-1)
    pub confidence: f64,             // Confianza en su perspectiva
    pub bias_direction: Bias,        // Sesgo conocido
    pub expertise_areas: Vec<String>, // Áreas donde es experto
    pub last_update: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bias {
    Optimistic,
    Pessimistic,
    Neutral,
    Cautious,
    Aggressive,
}

/// Decisión agregada de múltiples perspectivas
#[derive(Debug, Clone)]
pub struct CollectiveDecision {
    pub proposal_id: u64,
    pub perspectives: Vec<WeightedPerspective>,
    pub aggregate_position: Position,
    pub confidence: f64,
    pub dissent_nodes: Vec<String>,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct WeightedPerspective {
    pub node_id: String,
    pub weight: f64,
    pub position: Position,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Position {
    StronglyFor,
    For,
    Neutral,
    Against,
    StronglyAgainst,
}

impl Position {
    pub fn as_f64(&self) -> f64 {
        match self {
            Position::StronglyAgainst => -2.0,
            Position::Against => -1.0,
            Position::Neutral => 0.0,
            Position::For => 1.0,
            Position::StronglyFor => 2.0,
        }
    }

    pub fn from_f64(val: f64) -> Position {
        if val <= -1.5 {
            Position::StronglyAgainst
        } else if val <= -0.5 {
            Position::Against
        } else if val <= 0.5 {
            Position::Neutral
        } else if val <= 1.5 {
            Position::For
        } else {
            Position::StronglyFor
        }
    }
}

// ============================================================================
// COLLECTIVE WISDOM MANAGER
// ============================================================================

/// Gestor de sabiduría colectiva
pub struct CollectiveWisdomManager {
    /// Perspectivas conocidas
    perspectives: HashMap<String, NodePerspective>,
    /// Decisiones agregadas históricas
    decision_history: Vec<CollectiveDecision>,
    /// Nodos disidentes preservados
    dissent_preservation: HashMap<String, DissentRecord>,
    /// Configuración
    config: WisdomConfig,
}

#[derive(Debug, Clone)]
pub struct WisdomConfig {
    /// Peso máximo de cualquier nodo
    pub max_node_weight: f64,
    /// Peso mínimo de cualquier nodo
    pub min_node_weight: f64,
    /// Habilitar preservación de disidentes
    pub preserve_dissent: bool,
    /// Umbral para consensus
    pub consensus_threshold: f64,
    /// Máximo de perspectivas a considerar
    pub max_perspectives: usize,
}

impl Default for WisdomConfig {
    fn default() -> Self {
        Self {
            max_node_weight: 3.0,
            min_node_weight: 0.1,
            preserve_dissent: true,
            consensus_threshold: 0.75,
            max_perspectives: 100,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DissentRecord {
    pub node_id: String,
    pub proposal_id: u64,
    pub minority_position: Position,
    pub majority_position: Position,
    pub timestamp: u64,
}

impl CollectiveWisdomManager {
    pub fn new() -> Self {
        Self {
            perspectives: HashMap::new(),
            decision_history: Vec::new(),
            dissent_preservation: HashMap::new(),
            config: WisdomConfig::default(),
        }
    }

    /// Registrar perspectiva de un nodo
    pub fn register_perspective(&mut self, perspective: NodePerspective) {
        self.perspectives.insert(perspective.node_id.clone(), perspective);
    }

    /// Obtener perspectiva de un nodo
    pub fn get_perspective(&self, node_id: &str) -> Option<&NodePerspective> {
        self.perspectives.get(node_id)
    }

    /// Calcular peso de un nodo basado en su perspectiva
    pub fn calculate_weight(&self, node_id: &str, proposal_area: &str) -> f64 {
        let perspective = match self.perspectives.get(node_id) {
            Some(p) => p,
            None => return 1.0, // Default weight
        };

        // Base weight from expertise in this area
        let expertise = if perspective.expertise_areas.iter().any(|a| a == proposal_area) {
            1.5
        } else {
            1.0
        };

        // Modify by confidence
        let confidence_mod = 0.5 + (perspective.confidence * 0.5);

        // Modify by understanding
        let understanding_mod = 0.5 + (perspective.understanding * 0.5);

        let weight = expertise * confidence_mod * understanding_mod;

        weight.clamp(self.config.min_node_weight, self.config.max_node_weight)
    }

    /// Agregar decisión colectiva
    pub fn aggregate_decision(
        &mut self,
        proposal_id: u64,
        votes: &[(String, Position, f64)], // (node_id, position, base_weight)
        proposal_area: &str,
    ) -> CollectiveDecision {
        let mut weighted_perspectives = Vec::new();
        let mut total_for = 0.0f64;
        let mut total_against = 0.0f64;
        let mut dissent_nodes = Vec::new();

        for (node_id, position, base_weight) in votes {
            let expertise_weight = self.calculate_weight(node_id, proposal_area);
            let effective_weight = base_weight * expertise_weight;

            weighted_perspectives.push(WeightedPerspective {
                node_id: node_id.clone(),
                weight: effective_weight,
                position: *position,
            });

            if position.as_f64() > 0.0 {
                total_for += effective_weight;
            } else if position.as_f64() < 0.0 {
                total_against += effective_weight;
            }
        }

        // Calcular posición agregada
        let total_weight = total_for + total_against;
        let aggregate_position = if total_weight > 0.0 {
            // Calculate the weighted average position (-1 to 1)
            let avg = (total_for - total_against) / total_weight;
            // Map to Position (0.6 majority should be "For")
            Position::from_f64(avg)
        } else {
            Position::Neutral
        };

        // Calcular confianza (basada en acuerdo)
        let confidence = if total_weight > 0.0 {
            let dominant = total_for.max(total_against);
            dominant / total_weight
        } else {
            0.0
        };

        // Identificar disidentes
        let majority_pos = if total_for > total_against {
            Position::For
        } else if total_against > total_for {
            Position::Against
        } else {
            Position::Neutral
        };

        for wp in &weighted_perspectives {
            // Si votó diferente al majority y hay diferencia significativa
            if wp.position != majority_pos && 
               (wp.position.as_f64() - majority_pos.as_f64()).abs() >= 0.5 {
                dissent_nodes.push(wp.node_id.clone());
                
                // Registrar disenso
                if self.config.preserve_dissent {
                    self.dissent_preservation.insert(
                        format!("{}_{}", wp.node_id, proposal_id),
                        DissentRecord {
                            node_id: wp.node_id.clone(),
                            proposal_id,
                            minority_position: wp.position,
                            majority_position: majority_pos,
                            timestamp: current_timestamp(),
                        },
                    );
                }
            }
        }

        let decision = CollectiveDecision {
            proposal_id,
            perspectives: weighted_perspectives,
            aggregate_position,
            confidence,
            dissent_nodes,
            timestamp: current_timestamp(),
        };

        self.decision_history.push(decision.clone());

        // Limitar historial
        if self.decision_history.len() > 1000 {
            self.decision_history.remove(0);
        }

        decision
    }

    /// Verificar si hay consenso suficiente
    pub fn has_consensus(&self, decision: &CollectiveDecision) -> bool {
        decision.confidence >= self.config.consensus_threshold
    }

    /// Obtener historial de decisiones
    pub fn get_history(&self, limit: usize) -> Vec<&CollectiveDecision> {
        self.decision_history.iter().rev().take(limit).collect()
    }

    /// Obtener disidentes preservados
    pub fn get_preserved_dissent(&self) -> Vec<&DissentRecord> {
        self.dissent_preservation.values().collect()
    }

    /// Aprender de decisiones pasadas
    pub fn learn_from_outcome(&mut self, proposal_id: u64, success: bool) {
        if let Some(decision) = self.decision_history.iter().find(|d| d.proposal_id == proposal_id) {
            // Ajustar pesos de nodos basado en resultado
            for perspective in &mut self.perspectives.values_mut() {
                let voted = decision.perspectives.iter()
                    .find(|p| p.node_id == perspective.node_id);

                if let Some(_voter) = voted {
                    // Si votó con el majority y el resultado fue bueno, +confianza
                    // Si votó con el majority y el resultado fue malo, -confianza
                    // Si fue disidente y resultado fue bueno, +mucho
                    // Si fue disidente y resultado fue malo, -poco
                    
                    let was_dissent = decision.dissent_nodes.contains(&perspective.node_id);
                    
                    if success {
                        if was_dissent {
                            perspective.confidence = (perspective.confidence + 0.1).min(1.0);
                        } else {
                            perspective.confidence = (perspective.confidence + 0.05).min(1.0);
                        }
                    } else {
                        if was_dissent {
                            perspective.confidence = (perspective.confidence + 0.02).min(1.0);
                        } else {
                            perspective.confidence = (perspective.confidence - 0.05).max(0.0);
                        }
                    }

                    perspective.last_update = current_timestamp();
                }
            }
        }
    }

    /// Obtener estadísticas de sabiduría
    pub fn get_stats(&self) -> WisdomStats {
        WisdomStats {
            registered_perspectives: self.perspectives.len(),
            total_decisions: self.decision_history.len(),
            preserved_dissents: self.dissent_preservation.len(),
            avg_confidence: if self.decision_history.is_empty() {
                0.0
            } else {
                self.decision_history.iter()
                    .map(|d| d.confidence)
                    .sum::<f64>() / self.decision_history.len() as f64
            },
        }
    }
}

impl Default for CollectiveWisdomManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Default)]
pub struct WisdomStats {
    pub registered_perspectives: usize,
    pub total_decisions: usize,
    pub preserved_dissents: usize,
    pub avg_confidence: f64,
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
    fn test_perspective_registration() {
        let mut wisdom = CollectiveWisdomManager::new();
        
        wisdom.register_perspective(NodePerspective {
            node_id: "node1".to_string(),
            understanding: 0.8,
            confidence: 0.9,
            bias_direction: Bias::Optimistic,
            expertise_areas: vec!["security".to_string()],
            last_update: current_timestamp(),
        });
        
        let p = wisdom.get_perspective("node1").unwrap();
        assert_eq!(p.node_id, "node1");
    }

    #[test]
    fn test_weight_calculation() {
        let mut wisdom = CollectiveWisdomManager::new();
        
        wisdom.register_perspective(NodePerspective {
            node_id: "expert".to_string(),
            understanding: 0.9,
            confidence: 0.95,
            bias_direction: Bias::Neutral,
            expertise_areas: vec!["protocol".to_string()],
            last_update: current_timestamp(),
        });
        
        let weight = wisdom.calculate_weight("expert", "protocol");
        assert!(weight > 1.0); // Expert should have higher weight
    }

    #[test]
    fn test_decision_aggregation() {
        let mut wisdom = CollectiveWisdomManager::new();
        
        // Test with 3 For vs 2 Against (60% majority)
        // With Position::For = 1.0 and Position::Against = -1.0
        // avg = (3*1 - 2*1) / 5 = 0.2
        // 0.2 maps to Neutral with current thresholds
        let votes = vec![
            ("node1".to_string(), Position::For, 1.0),
            ("node2".to_string(), Position::For, 1.0),
            ("node3".to_string(), Position::Against, 1.0),
            ("node4".to_string(), Position::Against, 1.0),
            ("node5".to_string(), Position::For, 1.0),
        ];
        
        let decision = wisdom.aggregate_decision(1, &votes, "general");
        
        // 3 For vs 2 Against = 0.2 average position = Neutral
        // This is mathematically correct - 60% is not overwhelming
        assert_eq!(decision.aggregate_position, Position::Neutral);
        
        // Test with stronger majority - 4 For vs 1 Against
        let votes2 = vec![
            ("node1".to_string(), Position::For, 1.0),
            ("node2".to_string(), Position::For, 1.0),
            ("node3".to_string(), Position::For, 1.0),
            ("node4".to_string(), Position::For, 1.0),
            ("node5".to_string(), Position::Against, 1.0),
        ];
        
        let decision2 = wisdom.aggregate_decision(2, &votes2, "general");
        
        // avg = (4*1 - 1*1) / 5 = 0.6, maps to For
        assert_eq!(decision2.aggregate_position, Position::For);
        
        // Dissidents should be node5
        assert!(decision2.dissent_nodes.contains(&"node5".to_string()));
    }

    #[test]
    fn test_consensus_check() {
        let mut wisdom = CollectiveWisdomManager::new();
        
        let decision = CollectiveDecision {
            proposal_id: 1,
            perspectives: Vec::new(),
            aggregate_position: Position::For,
            confidence: 0.8,
            dissent_nodes: vec!["dissenter".to_string()],
            timestamp: current_timestamp(),
        };
        
        assert!(wisdom.has_consensus(&decision));
        
        let low_confidence = CollectiveDecision {
            confidence: 0.5,
            ..decision
        };
        
        assert!(!wisdom.has_consensus(&low_confidence));
    }

    #[test]
    fn test_dissent_preservation() {
        let mut wisdom = CollectiveWisdomManager::new();
        wisdom.config.preserve_dissent = true;
        
        let votes = vec![
            ("majority".to_string(), Position::For, 1.0),
            ("majority".to_string(), Position::For, 1.0),
            ("majority".to_string(), Position::For, 1.0),
            ("dissenter".to_string(), Position::Against, 1.0),
        ];
        
        wisdom.aggregate_decision(1, &votes, "general");
        
        let dissents = wisdom.get_preserved_dissent();
        assert!(!dissents.is_empty());
    }

    #[test]
    fn test_learning_from_outcome() {
        let mut wisdom = CollectiveWisdomManager::new();
        
        // Register perspective
        wisdom.register_perspective(NodePerspective {
            node_id: "learner".to_string(),
            understanding: 0.5,
            confidence: 0.5,
            bias_direction: Bias::Neutral,
            expertise_areas: vec![],
            last_update: current_timestamp(),
        });
        
        // Make a decision
        let votes = vec![
            ("learner".to_string(), Position::For, 1.0),
        ];
        
        wisdom.aggregate_decision(1, &votes, "general");
        
        // Learn from outcome
        wisdom.learn_from_outcome(1, true);
        
        let p = wisdom.get_perspective("learner").unwrap();
        assert!(p.confidence > 0.5); // Should have increased
    }
}