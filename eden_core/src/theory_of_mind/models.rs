//! # Models - Mental State Representations
//!
//! Representación de estados mentales usando modelo BDI.
//! 100% original, sin dependencias.
//!
//! ## BDI (Belief-Desire-Intention)
//!
//! - **Belief**: Lo que el agente cree sobre el mundo
//! - **Desire**: Lo que el agente quiere lograr
//! - **Intention**: Lo que el agente está comprometído a hacer
//!
//! ## Conceptos
//!
//! 1. **Mental Model**: Modelo completo del estado mental de un agente
//! 2. **Belief**: Proposición sobre el mundo con confianza
//! 3. **Desire**: Objetivo con prioridad y urgencia
//! 4. **Intention**: Plan comprometido con ejecución activa
//! 5. **Mental Snapshot**: Captura temporal del estado mental
#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(unused_imports)]

use std::collections::HashMap;
// ============================================================================
// TIPOS BASE
// ============================================================================

/// Identificador de agente
pub type AgentId = u64;

/// Timestamp mental
pub type TimePoint = u64;

/// Confianza en una creencia [0.0 - 1.0]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Confidence(pub f32);

impl Confidence {
    pub fn certain() -> Self {
        Self(1.0)
    }
    pub fn very_high() -> Self {
        Self(0.9)
    }
    pub fn high() -> Self {
        Self(0.75)
    }
    pub fn medium() -> Self {
        Self(0.5)
    }
    pub fn low() -> Self {
        Self(0.25)
    }
    pub fn very_low() -> Self {
        Self(0.1)
    }
    pub fn impossible() -> Self {
        Self(0.0)
    }

    pub fn value(&self) -> f32 {
        self.0
    }

    pub fn combine(self, other: Confidence) -> Confidence {
        // Bayesian updating-like combination
        let combined = self.0 + other.0 - (self.0 * other.0);
        Confidence(combined.min(1.0).max(0.0))
    }

    pub fn weight(&self) -> f32 {
        // Convert confidence to weight for reasoning
        self.0.max(0.01)
    }
}

/// Valoración de un deseo
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Valuation {
    pub priority: f32,    // [0.0 - 1.0] cuán importante es
    pub urgency: f32,     // [0.0 - 1.0] cuán urgente es
    pub feasibility: f32, // [0.0 - 1.0] cuán alcanzable es
}

impl Valuation {
    pub fn new(priority: f32, urgency: f32, feasibility: f32) -> Self {
        Self {
            priority: priority.max(0.0).min(1.0),
            urgency: urgency.max(0.0).min(1.0),
            feasibility: feasibility.max(0.0).min(1.0),
        }
    }

    /// Valor total del deseo
    pub fn total_value(&self) -> f32 {
        self.priority * 0.4 + self.urgency * 0.3 + self.feasibility * 0.3
    }

    pub fn critical() -> Self {
        Self::new(1.0, 1.0, 0.8)
    }
    pub fn high() -> Self {
        Self::new(0.8, 0.6, 0.7)
    }
    pub fn medium() -> Self {
        Self::new(0.5, 0.5, 0.5)
    }
    pub fn low() -> Self {
        Self::new(0.3, 0.3, 0.4)
    }
}

/// Actitud mental
#[derive(Clone, Debug, PartialEq)]
pub enum MentalAttitude {
    /// Creencia (proposición sobre el mundo)
    Belief(Belief),
    /// Deseo (objetivo deseado)
    Desire(Desire),
    /// Intención (plan comprometido)
    Intention(Intention),
}

impl MentalAttitude {
    pub fn confidence(&self) -> f32 {
        match self {
            MentalAttitude::Belief(b) => b.confidence.0,
            MentalAttitude::Desire(d) => d.valuation.priority,
            MentalAttitude::Intention(i) => i.commitment.resistance_to_change(),
        }
    }
}

// ============================================================================
// BELIEF
// ============================================================================

/// Creencia: lo que el agente cree sobre el mundo
#[derive(Clone, Debug, PartialEq)]
pub struct Belief {
    /// Proposición (ej: "el objeto está en la mesa")
    pub proposition: String,
    /// Confianza en la creencia
    pub confidence: Confidence,
    /// Origen de la creencia (percepción, inferencia, testimonio)
    pub source: BeliefSource,
    /// Timestamp de cuando se formó
    pub formed_at: TimePoint,
    /// Evidencia supporting esta creencia
    pub evidence: Vec<String>,
    /// ¿Es esta creencia revisable?
    pub revisable: bool,
}

/// Fuente de una creencia
#[derive(Clone, Debug, PartialEq)]
pub enum BeliefSource {
    ///来自感官
    Perception,
    /// Inferida de otras creencias
    Inference,
    ///来自 otro agente
    Testimony { agent_id: AgentId, reliability: f32 },
    /// Memoria
    Memory,
    /// Default (asunción inicial)
    Default,
}

impl Belief {
    pub fn new(proposition: &str, confidence: Confidence, source: BeliefSource) -> Self {
        Self {
            proposition: proposition.to_string(),
            confidence,
            source,
            formed_at: 0,
            evidence: Vec::new(),
            revisable: true,
        }
    }

    pub fn with_time(mut self, time: TimePoint) -> Self {
        self.formed_at = time;
        self
    }

    pub fn with_evidence(mut self, evidence: Vec<&str>) -> Self {
        self.evidence = evidence.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Revisa la confianza de la creencia
    pub fn revise(&mut self, new_confidence: Confidence, evidence: &str) {
        self.confidence = new_confidence;
        self.evidence.push(evidence.to_string());
    }

    /// Compatible con otra creencia?
    pub fn compatible_with(&self, other: &Belief) -> bool {
        // Simplified: check if propositions are contradictory
        // In real implementation, would need logical inference
        !self.contradicts(&other.proposition)
    }

    fn contradicts(&self, other_prop: &str) -> bool {
        // Check for negation markers
        let self_lower = self.proposition.to_lowercase();
        let other_lower = other_prop.to_lowercase();

        // Simple heuristics
        if self_lower.contains("no ") && !other_lower.contains("no ") {
            return true;
        }
        if other_lower.contains("no ") && !self_lower.contains("no ") {
            return true;
        }

        false
    }
}

// ============================================================================
// DESIRE
// ============================================================================

/// Deseo: lo que el agente quiere lograr
#[derive(Clone, Debug, PartialEq)]
pub struct Desire {
    /// Descripción del objetivo
    pub goal: String,
    /// Valoración (prioridad, urgencia, factibilidad)
    pub valuation: Valuation,
    /// Estado de satisfacción
    pub satisfaction: SatisfactionState,
    /// Dependencias con otros deseos
    pub dependencies: Vec<String>,
    /// Created at
    pub created_at: TimePoint,
}

/// Estado de satisfacción de un deseo
#[derive(Clone, Debug, PartialEq)]
pub enum SatisfactionState {
    /// Deseo aún no considerado
    Dormant,
    /// Deseo activo, siendo perseguido
    Active,
    /// Deseo en proceso de ser satisfecho
    Pursuing,
    /// Deseo satisfecho
    Fulfilled,
    /// Deseo abandonado
    Abandoned,
    /// Deseo frustado
    Frustrated,
}

impl Desire {
    pub fn new(goal: &str, valuation: Valuation) -> Self {
        Self {
            goal: goal.to_string(),
            valuation,
            satisfaction: SatisfactionState::Dormant,
            dependencies: Vec::new(),
            created_at: 0,
        }
    }

    pub fn with_time(mut self, time: TimePoint) -> Self {
        self.created_at = time;
        self
    }

    pub fn with_dependencies(mut self, deps: Vec<&str>) -> Self {
        self.dependencies = deps.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Intensidad del deseo [0.0 - 1.0]
    pub fn intensity(&self) -> f32 {
        let state_factor = match self.satisfaction {
            SatisfactionState::Dormant => 0.1,
            SatisfactionState::Active => 0.6,
            SatisfactionState::Pursuing => 0.9,
            SatisfactionState::Fulfilled => 0.0,
            SatisfactionState::Abandoned => 0.0,
            SatisfactionState::Frustrated => 0.3,
        };

        self.valuation.total_value() * state_factor
    }
}

// ============================================================================
// INTENTION
// ============================================================================

/// Intención: plan que el agente está comprometido a ejecutar
#[derive(Clone, Debug, PartialEq)]
pub struct Intention {
    /// Plan de acción
    pub plan: Plan,
    /// Nivel de compromiso
    pub commitment: Commitment,
    /// Estado de ejecución
    pub execution_state: ExecutionState,
    /// Intenciones subordenadas
    pub sub_intentions: Vec<Box<Intention>>,
    /// Progress actual [0.0 - 1.0]
    pub progress: f32,
    /// Created at
    pub created_at: TimePoint,
}

/// Plan de acción
#[derive(Clone, Debug, PartialEq)]
pub struct Plan {
    /// Pasos del plan
    pub steps: Vec<PlanStep>,
    /// Goal final del plan
    pub goal: String,
    /// Plan alternativo (fallback)
    pub alternative: Option<Box<Plan>>,
}

/// Paso del plan
#[derive(Clone, Debug, PartialEq)]
pub struct PlanStep {
    /// Descripción del paso
    pub description: String,
    /// Precondiciones necesarias
    pub preconditions: Vec<String>,
    /// Efectos esperados
    pub effects: Vec<String>,
    /// Duración estimada
    pub estimated_duration: u64,
    /// ¿Completado?
    pub completed: bool,
}

/// Nivel de compromiso con una intención
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Commitment {
    /// Commitment debil - puede cambiar fácilmente
    Weak,
    /// Commitment medio
    Moderate,
    /// Commitment fuerte - difícil de desviar
    Strong,
    /// Commitment absoluto - inquebrantable
    Absolute,
}

impl Commitment {
    pub fn resistance_to_change(&self) -> f32 {
        match self {
            Commitment::Weak => 0.2,
            Commitment::Moderate => 0.5,
            Commitment::Strong => 0.8,
            Commitment::Absolute => 1.0,
        }
    }
}

/// Estado de ejecución
#[derive(Clone, Debug, PartialEq)]
pub enum ExecutionState {
    /// No ha iniciado
    NotStarted,
    /// En progreso
    InProgress { started_at: TimePoint },
    /// Pausado
    Paused { paused_at: TimePoint },
    /// Completado
    Completed { finished_at: TimePoint },
    /// Fallido
    Failed { reason: String },
    /// Abandonado
    Abandoned,
}

impl Intention {
    pub fn new(plan: Plan, commitment: Commitment) -> Self {
        Self {
            plan,
            commitment,
            execution_state: ExecutionState::NotStarted,
            sub_intentions: Vec::new(),
            progress: 0.0,
            created_at: 0,
        }
    }

    pub fn with_time(mut self, time: TimePoint) -> Self {
        self.created_at = time;
        self
    }

    /// Verifica si la intención sigue siendo relevante
    pub fn is_relevant(&self, current_beliefs: &[Belief]) -> bool {
        // Check if preconditions are still met
        for step in &self.plan.steps {
            for precondition in &step.preconditions {
                let mut satisfied = false;
                for belief in current_beliefs {
                    if belief.proposition.contains(precondition) {
                        satisfied = belief.confidence.0 > 0.5;
                        break;
                    }
                }
                if !satisfied {
                    return false;
                }
            }
        }
        true
    }

    /// Conflicto con otra intención?
    pub fn conflicts_with(&self, other: &Intention) -> bool {
        for step in &self.plan.steps {
            for effect in &step.effects {
                for other_step in &other.plan.steps {
                    for other_effect in &other_step.effects {
                        if effect == other_effect {
                            return true; // Effects conflict
                        }
                    }
                }
            }
        }
        false
    }
}

// ============================================================================
// MENTAL MODEL (BDI)
// ============================================================================

/// Modelo mental completo de un agente
#[derive(Clone, Debug)]
pub struct MentalModel {
    /// ID del agente
    pub agent_id: AgentId,
    /// Creencias actuales
    pub beliefs: HashMap<String, Belief>,
    /// Deseos actuales
    pub desires: Vec<Desire>,
    /// Intenciones actuales
    pub intentions: Vec<Intention>,
    /// Emociones actuales
    pub emotions: HashMap<EmotionType, f32>,
    /// Relationships con otros agentes
    pub relationships: HashMap<AgentId, SocialRelationship>,
    /// Mentalidad grupal (si pertenece a grupo)
    pub group_identity: Option<GroupIdentity>,
    /// Timestamp de última actualización
    pub last_update: TimePoint,
}

/// Tipo de emoción
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum EmotionType {
    Joy,
    Sadness,
    Anger,
    Fear,
    Surprise,
    Disgust,
    Trust,
    Anticipation,
    Love,
    Guilt,
    Shame,
    Pride,
    Hope,
    Despair,
    Contentment,
    Frustration,
    Anxiety,
    Interest,
    // Extended emotions for richer modeling
    Admiration,
    Contempt,
    Relief,
    Env,
    SelfConsciousEmotion, // e.g., embarrassment
}

impl EmotionType {
    pub fn valence(&self) -> f32 {
        // -1.0 (negative) to 1.0 (positive)
        match self {
            EmotionType::Joy => 0.9,
            EmotionType::Sadness => -0.8,
            EmotionType::Anger => -0.7,
            EmotionType::Fear => -0.8,
            EmotionType::Surprise => 0.2,
            EmotionType::Disgust => -0.7,
            EmotionType::Trust => 0.7,
            EmotionType::Anticipation => 0.5,
            EmotionType::Love => 0.9,
            EmotionType::Guilt => -0.6,
            EmotionType::Shame => -0.7,
            EmotionType::Pride => 0.7,
            EmotionType::Hope => 0.6,
            EmotionType::Despair => -0.9,
            EmotionType::Contentment => 0.8,
            EmotionType::Frustration => -0.5,
            EmotionType::Admiration => 0.7,
            EmotionType::Contempt => -0.6,
            EmotionType::Relief => 0.6,
            EmotionType::Env => -0.4,
            _ => 0.0,
        }
    }

    pub fn arousal(&self) -> f32 {
        // 0.0 (calm) to 1.0 (excited)
        match self {
            EmotionType::Joy => 0.7,
            EmotionType::Sadness => 0.3,
            EmotionType::Anger => 0.9,
            EmotionType::Fear => 0.8,
            EmotionType::Surprise => 0.9,
            EmotionType::Disgust => 0.7,
            EmotionType::Trust => 0.4,
            EmotionType::Anticipation => 0.6,
            EmotionType::Love => 0.6,
            EmotionType::Guilt => 0.5,
            EmotionType::Shame => 0.6,
            EmotionType::Pride => 0.7,
            EmotionType::Hope => 0.5,
            EmotionType::Despair => 0.4,
            EmotionType::Contentment => 0.2,
            EmotionType::Frustration => 0.7,
            EmotionType::Admiration => 0.5,
            EmotionType::Contempt => 0.4,
            EmotionType::Relief => 0.3,
            EmotionType::Env => 0.5,
            _ => 0.5,
        }
    }
}

/// Relación social con otro agente
#[derive(Clone, Debug)]
pub struct SocialRelationship {
    pub other_agent: AgentId,
    pub relationship_type: RelationshipType,
    pub trust: f32,
    pub familiarity: f32,
    pub dominance: f32, // -1.0 (submissive) to 1.0 (dominant)
    pub affinity: f32,  // -1.0 (hostile) to 1.0 (friendly)
}

/// Tipo de relación
#[derive(Clone, Debug, PartialEq)]
pub enum RelationshipType {
    Friend,
    Enemy,
    Neutral,
    Family,
    Colleague,
    Superior,
    Subordinate,
    Romantic,
    Rival,
}

/// Identidad grupal
#[derive(Clone, Debug)]
pub struct GroupIdentity {
    pub group_id: String,
    pub role: String,
    pub loyalty: f32,
    pub group_goals: Vec<String>,
}

impl MentalModel {
    pub fn new(agent_id: AgentId) -> Self {
        Self {
            agent_id,
            beliefs: HashMap::new(),
            desires: Vec::new(),
            intentions: Vec::new(),
            emotions: HashMap::new(),
            relationships: HashMap::new(),
            group_identity: None,
            last_update: 0,
        }
    }

    /// Añade una creencia
    pub fn add_belief(&mut self, belief: Belief) {
        self.beliefs.insert(belief.proposition.clone(), belief);
        self.last_update += 1;
    }

    /// Añade un deseo
    pub fn add_desire(&mut self, desire: Desire) {
        self.desires.push(desire);
        self.last_update += 1;
    }

    /// Añade una intención
    pub fn add_intention(&mut self, intention: Intention) {
        self.intentions.push(intention);
        self.last_update += 1;
    }

    /// Obtiene creencia sobre una proposición
    pub fn get_belief(&self, proposition: &str) -> Option<&Belief> {
        self.beliefs.get(proposition)
    }

    /// Actualiza emoción
    pub fn set_emotion(&mut self, emotion: EmotionType, intensity: f32) {
        self.emotions.insert(emotion, intensity.max(0.0).min(1.0));
        self.last_update += 1;
    }

    /// Obtiene intensidad de emoción
    pub fn get_emotion(&self, emotion: &EmotionType) -> f32 {
        self.emotions.get(emotion).copied().unwrap_or(0.0)
    }

    /// Obtiene el deseo más prioritario
    pub fn top_desire(&self) -> Option<&Desire> {
        self.desires.iter().max_by(|a, b| {
            a.valuation
                .total_value()
                .partial_cmp(&b.valuation.total_value())
                .unwrap()
        })
    }

    /// Obtiene la intención más comprometida
    pub fn primary_intention(&self) -> Option<&Intention> {
        self.intentions.iter().max_by(|a, b| {
            a.commitment
                .resistance_to_change()
                .partial_cmp(&b.commitment.resistance_to_change())
                .unwrap()
        })
    }

    /// Estado emocional dominante
    pub fn dominant_emotion(&self) -> Option<(EmotionType, f32)> {
        self.emotions
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(k, v)| (k.clone(), *v))
    }

    /// Serializa el modelo mental a string
    pub fn to_string(&self) -> String {
        let mut s = format!("MentalModel(agent={})\n", self.agent_id);
        s += &format!("  Beliefs ({}):\n", self.beliefs.len());
        for (prop, belief) in &self.beliefs {
            s += &format!("    - {} (conf: {:.2})\n", prop, belief.confidence.0);
        }
        s += &format!("  Desires ({}):\n", self.desires.len());
        for desire in &self.desires {
            s += &format!(
                "    - {} (value: {:.2})\n",
                desire.goal,
                desire.valuation.total_value()
            );
        }
        s += &format!("  Intentions ({}):\n", self.intentions.len());
        for intent in &self.intentions {
            s += &format!(
                "    - {} (commitment: {:?})\n",
                intent.plan.goal, intent.commitment
            );
        }
        s
    }
}

// ============================================================================
// MENTAL SNAPSHOT
// ============================================================================

/// Captura temporal del estado mental
#[derive(Clone, Debug)]
pub struct MentalSnapshot {
    pub agent_id: AgentId,
    pub timestamp: TimePoint,
    pub model: MentalModel,
    pub context: String, // Context in which this snapshot was taken
}

impl MentalSnapshot {
    pub fn capture(model: &MentalModel, context: &str) -> Self {
        Self {
            agent_id: model.agent_id,
            timestamp: model.last_update,
            model: model.clone(),
            context: context.to_string(),
        }
    }

    /// Compara con otra instantánea para detectar cambios
    pub fn diff(&self, other: &MentalSnapshot) -> MentalDiff {
        let mut changed_beliefs = Vec::new();
        let mut changed_desires = Vec::new();
        let changed_intentions = Vec::new();

        for (prop, belief) in &self.model.beliefs {
            if let Some(other_belief) = other.model.beliefs.get(prop) {
                if (belief.confidence.0 - other_belief.confidence.0).abs() > 0.1 {
                    changed_beliefs.push(prop.clone());
                }
            }
        }

        for desire in &self.model.desires {
            if let Some(other_desire) = other.model.desires.iter().find(|d| d.goal == desire.goal) {
                if (desire.valuation.total_value() - other_desire.valuation.total_value()).abs()
                    > 0.1
                {
                    changed_desires.push(desire.goal.clone());
                }
            }
        }

        MentalDiff {
            timestamp_diff: other.timestamp - self.timestamp,
            changed_beliefs,
            changed_desires,
            changed_intentions,
            emotional_shifts: self.compute_emotional_shift(other),
        }
    }

    fn compute_emotional_shift(&self, other: &MentalSnapshot) -> Vec<(EmotionType, f32)> {
        let mut shifts = Vec::new();
        for (emotion, intensity) in &self.model.emotions {
            let other_intensity = other.model.emotions.get(emotion).unwrap_or(&0.0);
            let diff = intensity - other_intensity;
            if diff.abs() > 0.1 {
                shifts.push((emotion.clone(), diff));
            }
        }
        shifts
    }
}

/// Diferencia entre instantáneas
#[derive(Clone, Debug)]
pub struct MentalDiff {
    pub timestamp_diff: u64,
    pub changed_beliefs: Vec<String>,
    pub changed_desires: Vec<String>,
    pub changed_intentions: Vec<String>,
    pub emotional_shifts: Vec<(EmotionType, f32)>,
}
