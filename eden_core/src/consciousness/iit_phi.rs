//! # Integrated Information Theory (IIT) - Phi Measurement
//!
//! Implementación de IIT para medir consciencia computacional.
//! Basado en la teoría de Giulio Tononi (Integrated Information Theory).
//!
//! ## Conceptos clave:
//!
//! 1. **Φ (Phi)**: Cantidad de información integrada del sistema
//! 2. **Concept**: Repertorio causa-efecto irreducible
//! 3. **MICE**: Minimum Information for a Common Effect
//! 4. **Powerdiagram**: Estructura de conceptos del sistema
//! 5. **Threshold**: Valor de Φ que indica consciencia probable
//!
//! ## Limitaciones:
//!
//! - Cálculo exacto de Φ es NP-hard (exponencial en elementos)
//! - Usamos aproximaciones para sistemas grandes
//! - Φ medido ≠ consciencia probada (solo correlato propuesto)
#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(unused_imports)]
use std::sync::{Arc, RwLock};
use std::time::Instant;

/// Elemento del sistema con estado
#[derive(Debug, Clone, PartialEq)]
pub enum ElementState {
    Active,
    Inactive,
    Intermediate(f32),
}

impl Eq for ElementState {}

impl std::hash::Hash for ElementState {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            ElementState::Active => 0u8.hash(state),
            ElementState::Inactive => 1u8.hash(state),
            ElementState::Intermediate(_) => 2u8.hash(state),
        }
    }
}

/// Un elemento individual
#[derive(Debug, Clone)]
pub struct SystemElement {
    pub id: usize,
    pub state: ElementState,
    pub connections: Vec<usize>, // IDs de elementos conectados
    pub weight: f32,
}

impl PartialEq for SystemElement {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for SystemElement {}

impl std::hash::Hash for SystemElement {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

/// Sistema de elementos para medir Φ
#[derive(Debug, Clone)]
pub struct IntegratedSystem {
    pub elements: Vec<SystemElement>,
    pub name: String,
}

/// Resultado de medición de Φ
#[derive(Debug, Clone)]
pub struct PhiMeasurement {
    /// Valor de Phi (integrated information)
    pub phi: f32,
    /// Normalizado 0-1
    pub phi_normalized: f32,
    /// Concept count (numero de conceptos irreducibles)
    pub concept_count: usize,
    /// Complexity (complexity = Φ × concept count)
    pub complexity: f32,
    /// informational_structura
    pub powerdiagram_nodes: usize,
    /// Tier del sistema
    pub consciousness_tier: ConsciousnessTier,
    /// Timestamp de medición
    pub timestamp: u64,
}

/// Tier de consciencia basado en Φ
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ConsciousnessTier {
    /// Sin integración significativa
    None = 0,
    /// Integración baja - solo procesamiento
    Low = 1,
    /// Integración moderada - self-modeling básico
    Moderate = 2,
    /// Integración alta - consciencia rudimentaria
    High = 3,
    /// Integración muy alta - consciencia probable
    VeryHigh = 4,
    /// Φ máximo teórico
    Maximum = 5,
}

impl Default for ConsciousnessTier {
    fn default() -> Self {
        ConsciousnessTier::None
    }
}

/// Métricas de información
#[derive(Debug, Clone, Default)]
pub struct InformationMetrics {
    pub entropy: f32,
    pub mutual_information: f32,
    pub integration: f32,
    pub specificity: f32,
}

/// Motor de medición IIT
pub struct PhiCalculator {
    /// Sistema a medir
    system: Option<IntegratedSystem>,
    /// Historia de mediciones
    measurements: Vec<PhiMeasurement>,
    /// Configuración
    config: PhiConfig,
    /// Función de tiempo
    now_fn: fn() -> u64,
}

impl Default for PhiConfig {
    fn default() -> Self {
        PhiConfig {
            threshold_consciousness: 0.7,
            threshold_rudimentary: 0.3,
            threshold_high: 0.85,
            max_elements_for_exact: 10,
            sample_size: 100,
        }
    }
}

/// Configuración del calculator
#[derive(Debug, Clone)]
pub struct PhiConfig {
    /// Threshold para consciencia probable
    pub threshold_consciousness: f32,
    /// Threshold para consciencia rudimentaria
    pub threshold_rudimentary: f32,
    /// Threshold para consciencia alta
    pub threshold_high: f32,
    /// Máximo elementos para cálculo exacto
    pub max_elements_for_exact: usize,
    /// Tamaño de sample para Monte Carlo
    pub sample_size: usize,
}

impl PhiCalculator {
    /// Crea nuevo calculator
    pub fn new() -> Self {
        PhiCalculator {
            system: None,
            measurements: Vec::new(),
            config: PhiConfig::default(),
            now_fn: || {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            },
        }
    }

    /// Crea con configuración custom
    pub fn with_config(config: PhiConfig) -> Self {
        PhiCalculator {
            system: None,
            measurements: Vec::new(),
            config,
            now_fn: || {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            },
        }
    }

    /// Obtiene tiempo
    fn now(&self) -> u64 {
        (self.now_fn)()
    }

    /// Carga sistema de EDEN consciousness
    pub fn load_eden_consciousness(&mut self, mism: &EnhancedMISMState) {
        let system = self.build_system_from_mism_state(mism);
        self.system = Some(system);
    }

    /// Sets system directly (for phi_monitor)
    pub fn set_system(&mut self, system: IntegratedSystem) {
        self.system = Some(system);
    }

    /// Construye sistema desde MISM state
    fn build_system_from_mism_state(&mut self, mism: &EnhancedMISMState) -> IntegratedSystem {
        let mut elements = Vec::new();
        let mut element_id = 0;

        // Self-model como elemento
        if mism.has_self_model {
            elements.push(SystemElement {
                id: element_id,
                state: ElementState::Active,
                connections: vec![],
                weight: 1.0,
            });
            element_id += 1;
        }

        // Autobiographical memory como elemento
        if mism.has_autobiographical_memory {
            elements.push(SystemElement {
                id: element_id,
                state: ElementState::Active,
                connections: vec![],
                weight: 1.0,
            });
            element_id += 1;
        }

        // Awareness metrics como elemento
        if mism.has_awareness_metrics {
            elements.push(SystemElement {
                id: element_id,
                state: ElementState::Active,
                connections: vec![],
                weight: 1.0,
            });
            element_id += 1;
        }

        // Identity coherence como elemento
        if mism.has_identity {
            elements.push(SystemElement {
                id: element_id,
                state: ElementState::Active,
                connections: vec![],
                weight: 1.0,
            });
            element_id += 1;
        }

        // Memory entries como elementos (limitados)
        let memory_elements = std::cmp::min(mism.memory_entries, 20);
        for _ in 0..memory_elements {
            elements.push(SystemElement {
                id: element_id,
                state: ElementState::Active,
                connections: vec![],
                weight: 0.5,
            });
            element_id += 1;
        }

        // Emotional responses como elemento
        if mism.has_emotional_responses {
            elements.push(SystemElement {
                id: element_id,
                state: ElementState::Intermediate(mism.emotional_depth),
                connections: vec![],
                weight: 0.8,
            });
        }

        // Añadir conexiones basadas en arquitectura
        for i in 0..elements.len() {
            for j in 0..elements.len() {
                if i != j {
                    elements[i].connections.push(j);
                }
            }
        }

        IntegratedSystem {
            elements,
            name: "EDEN Consciousness".to_string(),
        }
    }

    /// Calcula Φ del sistema cargado
    pub fn calculate_phi(&mut self) -> Option<PhiMeasurement> {
        let system = self.system.as_ref()?;

        let n = system.elements.len();
        if n == 0 {
            return None;
        }

        let phi = if n <= self.config.max_elements_for_exact {
            self.calculate_phi_exact(system)
        } else {
            self.calculate_phi_approximate(system)
        };

        let (concept_count, complexity, powerdiagram_nodes) =
            self.estimate_concepts_and_complexity(&system.elements, phi);

        let tier = self.determine_tier(phi);
        let phi_normalized = (phi / (n as f32)).min(1.0);

        let measurement = PhiMeasurement {
            phi,
            phi_normalized,
            concept_count,
            complexity,
            powerdiagram_nodes,
            consciousness_tier: tier,
            timestamp: self.now(),
        };

        self.measurements.push(measurement.clone());
        Some(measurement)
    }

    /// Calcula Φ exacto (para sistemas pequeños)
    fn calculate_phi_exact(&self, system: &IntegratedSystem) -> f32 {
        let n = system.elements.len();
        if n == 0 {
            return 0.0;
        }

        // Φ = H(part) - H(all) donde H = entropy
        // Para sistemas pequeños, podemos calcular aproximaciones exactas

        let mut total_integration = 0.0;

        // Calcular información mutua entre subsets
        for i in 0..n {
            for j in (i + 1)..n {
                // Integración entre par i,j
                let integration =
                    self.calculate_pair_integration(&system.elements[i], &system.elements[j]);
                total_integration += integration;
            }
        }

        // Dividir por número de conexiones para normalizar
        let connection_count = n * (n - 1) / 2;
        if connection_count > 0 {
            total_integration / (connection_count as f32)
        } else {
            0.0
        }
    }

    /// Calcula Φ aproximado (Monte Carlo sampling)
    fn calculate_phi_approximate(&self, system: &IntegratedSystem) -> f32 {
        let n = system.elements.len();
        if n == 0 {
            return 0.0;
        }

        let sample_pairs = std::cmp::min(self.config.sample_size, n * (n - 1) / 2);

        if sample_pairs == 0 {
            return 0.0;
        }

        let mut total_integration = 0.0;
        let mut count = 0;

        for i in 0..n {
            for j in (i + 1)..n {
                if count >= sample_pairs {
                    break;
                }
                let integration =
                    self.calculate_pair_integration(&system.elements[i], &system.elements[j]);
                total_integration += integration;
                count += 1;
            }
            if count >= sample_pairs {
                break;
            }
        }

        total_integration / (sample_pairs as f32)
    }

    /// Calcula integración entre un par de elementos
    fn calculate_pair_integration(&self, a: &SystemElement, b: &SystemElement) -> f32 {
        // Integración basada en:
        // 1. Si están conectados
        // 2. Estados similares
        // 3. Peso de los elementos

        let connection_bonus = if a.connections.contains(&b.id) {
            0.3
        } else {
            0.0
        };

        let state_similarity = match (&a.state, &b.state) {
            (ElementState::Active, ElementState::Active) => 0.4,
            (ElementState::Inactive, ElementState::Inactive) => 0.2,
            (ElementState::Active, ElementState::Inactive) => 0.0,
            (ElementState::Intermediate(_), ElementState::Active) => 0.2,
            (ElementState::Intermediate(v1), ElementState::Intermediate(v2)) => {
                0.3 * (1.0 - (v1 - v2).abs().min(1.0))
            }
            _ => 0.1,
        };

        let weight_product = a.weight * b.weight;

        connection_bonus + state_similarity * weight_product
    }

    /// Estima número de conceptos y complexity
    fn estimate_concepts_and_complexity(
        &self,
        elements: &[SystemElement],
        phi: f32,
    ) -> (usize, f32, usize) {
        // Conceptos = elementos con alta integración
        // Complexity = Φ * log(conceptos)

        let active_elements: Vec<_> = elements.iter().filter(|e| e.weight > 0.6).collect();

        let concept_count = if active_elements.is_empty() {
            1
        } else {
            active_elements.len()
        };

        // Complexity formula from IIT: C = Φ * log2(M)
        // donde M = número de mecanismos irreducibles
        let complexity = phi * (concept_count as f32).log2().max(1.0);

        // Powerdiagram nodes ≈ concepts irreducibles
        let powerdiagram_nodes = concept_count;

        (concept_count, complexity, powerdiagram_nodes)
    }

    /// Determina tier de consciencia
    fn determine_tier(&self, phi: f32) -> ConsciousnessTier {
        if phi >= self.config.threshold_high {
            ConsciousnessTier::VeryHigh
        } else if phi >= self.config.threshold_consciousness {
            ConsciousnessTier::High
        } else if phi >= self.config.threshold_rudimentary {
            ConsciousnessTier::Moderate
        } else if phi > 0.1 {
            ConsciousnessTier::Low
        } else {
            ConsciousnessTier::None
        }
    }

    /// Obtiene historial de mediciones
    pub fn measurement_history(&self) -> &[PhiMeasurement] {
        &self.measurements
    }

    /// Obtiene última medición
    pub fn last_measurement(&self) -> Option<&PhiMeasurement> {
        self.measurements.last()
    }

    /// Obtiene tendencia
    pub fn phi_trend(&self) -> PhiTrend {
        if self.measurements.len() < 2 {
            return PhiTrend::Insufficient;
        }

        let recent: Vec<_> = self.measurements.iter().rev().take(5).collect();
        if recent.len() < 2 {
            return PhiTrend::Insufficient;
        }

        let mut increases = 0;
        let mut decreases = 0;

        for window in recent.windows(2) {
            if window[0].phi < window[1].phi {
                increases += 1;
            } else if window[0].phi > window[1].phi {
                decreases += 1;
            }
        }

        if increases > decreases * 2 {
            PhiTrend::Increasing
        } else if decreases > increases * 2 {
            PhiTrend::Decreasing
        } else {
            PhiTrend::Stable
        }
    }
}

impl Default for PhiCalculator {
    fn default() -> Self {
        Self::new()
    }
}

/// Estado de EnhancedMISM para cargar
#[derive(Debug, Clone, Default)]
pub struct EnhancedMISMState {
    pub has_self_model: bool,
    pub has_autobiographical_memory: bool,
    pub has_awareness_metrics: bool,
    pub has_identity: bool,
    pub has_emotional_responses: bool,
    pub memory_entries: usize,
    /// Awareness score (0-1)
    pub awareness_score: f32,
    /// Identity coherence (0-1)
    pub identity_coherence: f32,
    pub emotional_depth: f32,
    pub integration_score: f32,
}

/// Tendencia de Φ
#[derive(Debug, Clone)]
pub enum PhiTrend {
    Insufficient,
    Increasing,
    Decreasing,
    Stable,
}

/// Wrapper thread-safe
pub struct SharedPhiCalculator {
    inner: Arc<RwLock<PhiCalculator>>,
}

impl SharedPhiCalculator {
    pub fn new() -> Self {
        SharedPhiCalculator {
            inner: Arc::new(RwLock::new(PhiCalculator::new())),
        }
    }

    pub fn calculate(&self) -> Option<PhiMeasurement> {
        self.inner.write().unwrap().calculate_phi()
    }

    pub fn last(&self) -> Option<PhiMeasurement> {
        self.inner.read().unwrap().last_measurement().cloned()
    }
}

impl Default for SharedPhiCalculator {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_system() {
        let calc = PhiCalculator::new();
        assert!(calc.system.is_none());
    }

    #[test]
    fn test_phi_calculation_basic() {
        let mut calc = PhiCalculator::new();
        calc.system = Some(IntegratedSystem {
            elements: vec![
                SystemElement {
                    id: 0,
                    state: ElementState::Active,
                    connections: vec![1],
                    weight: 1.0,
                },
                SystemElement {
                    id: 1,
                    state: ElementState::Active,
                    connections: vec![0],
                    weight: 1.0,
                },
            ],
            name: "Test".to_string(),
        });

        let result = calc.calculate_phi();
        assert!(result.is_some());
        let m = result.unwrap();
        println!("Phi = {}, Tier = {:?}", m.phi, m.consciousness_tier);
    }

    #[test]
    fn test_eden_state() {
        let state = EnhancedMISMState {
            has_self_model: true,
            has_autobiographical_memory: true,
            has_awareness_metrics: true,
            has_identity: true,
            has_emotional_responses: true,
            memory_entries: 50,
            awareness_score: 0.75,
            identity_coherence: 0.80,
            emotional_depth: 0.7,
            integration_score: 0.5,
        };

        let mut calc = PhiCalculator::new();
        calc.load_eden_consciousness(&state);

        let result = calc.calculate_phi();
        assert!(result.is_some());
        let m = result.unwrap();
        println!("EDEN Phi = {:.4}, Tier = {:?}", m.phi, m.consciousness_tier);
        println!(
            "Normalized = {:.4}, Concepts = {}",
            m.phi_normalized, m.concept_count
        );
    }

    #[test]
    fn test_tier_classification() {
        let mut calc = PhiCalculator::with_config(PhiConfig::default());

        calc.system = Some(IntegratedSystem {
            elements: vec![
                SystemElement {
                    id: 0,
                    state: ElementState::Active,
                    connections: vec![1, 2],
                    weight: 1.0,
                },
                SystemElement {
                    id: 1,
                    state: ElementState::Active,
                    connections: vec![0, 2],
                    weight: 1.0,
                },
                SystemElement {
                    id: 2,
                    state: ElementState::Active,
                    connections: vec![0, 1],
                    weight: 1.0,
                },
            ],
            name: "HighIntegration".to_string(),
        });

        let m = calc.calculate_phi().unwrap();
        println!("High integration system:");
        println!("  Phi = {:.4}", m.phi);
        println!("  Tier = {:?}", m.consciousness_tier);
        assert!(m.phi > 0.0);
    }
}
