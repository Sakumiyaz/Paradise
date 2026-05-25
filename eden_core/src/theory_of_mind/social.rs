//! # Social - Social Reasoning
//!
//! Sistema de razonamiento social 100% original.
//! Sin dependencias.
//!
//! ## Características
//!
//! 1. **SocialReasoner**: Razonador sobre dinámicas sociales
//! 2. **SocialRelationship**: Modelo de relaciones entre agentes
//! 3. **SocialDynamics**: Dinámicas de grupo y relación
//! 4. **PowerDynamics**: Análisis de poder y estatus
//!
//! ## Conceptos
//!
//! - SocialRelationship: Modelo de relación entre dos agentes
//! - SocialDynamics: Evolución de relaciones y grupos
//! - InteractionHistory: Historial de interacciones
//! - PowerDynamics: Distribución de poder en un grupo
#![allow(dead_code)]
#![allow(non_snake_case)]

use crate::theory_of_mind::{AgentId, MentalModel, RelationshipType, TimePoint};
use std::collections::HashMap;

// ============================================================================
// SOCIAL RELATIONSHIP
// ============================================================================

/// Modelo de relación social entre dos agentes
#[derive(Clone, Debug)]
pub struct InterpersonalRelation {
    /// Otro agente en la relación
    pub other_agent: AgentId,
    /// Tipo de relación
    pub relationship_type: RelationshipType,
    /// Confianza mutua [0.0 - 1.0]
    pub mutual_trust: f32,
    /// Familiaridad [0.0 - 1.0]
    pub familiarity: f32,
    /// Dominancia [-1.0 (sumiso) a 1.0 (dominante)]
    pub dominance: f32,
    /// Afinidad [-1.0 (hostil) a 1.0 (amigable)]
    pub affinity: f32,
    /// Historial de interacciones
    pub interaction_history: Vec<InteractionRecord>,
    /// Estado actual de la relación
    pub state: RelationshipState,
    ///timestamp
    pub last_update: TimePoint,
}

/// Estado de la relación
#[derive(Clone, Debug, PartialEq)]
pub enum RelationshipState {
    /// Relación normal y estable
    Stable,
    /// Relación en desarrollo
    Developing,
    /// Relación deteriorating
    Deteriorating,
    /// Conflicto activo
    Conflict,
    /// Relación recién terminada
    Ended,
}

impl InterpersonalRelation {
    pub fn new(other_agent: AgentId, relationship_type: RelationshipType) -> Self {
        Self {
            other_agent,
            relationship_type,
            mutual_trust: 0.5,
            familiarity: 0.0,
            dominance: 0.0,
            affinity: 0.0,
            interaction_history: Vec::new(),
            state: RelationshipState::Developing,
            last_update: 0,
        }
    }

    /// Actualiza la relación basándose en una nueva interacción
    pub fn update_from_interaction(&mut self, interaction: &InteractionRecord) {
        match interaction.interaction_type {
            InteractionType::Helpful => {
                self.mutual_trust = (self.mutual_trust + 0.1).min(1.0);
                self.affinity = (self.affinity + 0.1).min(1.0);
            }
            InteractionType::Harmful => {
                self.mutual_trust = (self.mutual_trust - 0.15).max(0.0);
                self.affinity = (self.affinity - 0.15).max(-1.0);
            }
            InteractionType::Neutral => {
                self.familiarity = (self.familiarity + 0.05).min(1.0);
            }
            InteractionType::Dominant => {
                self.dominance = (self.dominance + 0.1).min(1.0);
            }
            InteractionType::Submissive => {
                self.dominance = (self.dominance - 0.1).min(-1.0);
            }
            InteractionType::Collaborative => {
                self.mutual_trust = (self.mutual_trust + 0.15).min(1.0);
                self.affinity = (self.affinity + 0.1).min(1.0);
            }
            InteractionType::Competitive => {
                self.affinity = (self.affinity - 0.05).max(-1.0);
            }
        }

        self.familiarity = (self.familiarity + 0.05).min(1.0);
        self.interaction_history.push(interaction.clone());
        self.last_update = interaction.timestamp;

        self.update_state();
    }

    fn update_state(&mut self) {
        if self.mutual_trust < 0.2 {
            self.state = RelationshipState::Conflict;
        } else if self.mutual_trust < 0.4 {
            self.state = RelationshipState::Deteriorating;
        } else if self.familiarity > 0.7 {
            self.state = RelationshipState::Stable;
        } else {
            self.state = RelationshipState::Developing;
        }
    }

    /// Fuerza de la relación [0.0 - 1.0]
    pub fn strength(&self) -> f32 {
        let trust_weight = 0.4;
        let familiarity_weight = 0.3;
        let affinity_weight = 0.3;

        self.mutual_trust * trust_weight
            + self.familiarity * familiarity_weight
            + ((self.affinity + 1.0) / 2.0) * affinity_weight
    }

    /// Predice cómo actuará el otro agente
    pub fn predict_behavior(&self, behavior: &str) -> f32 {
        match behavior {
            "helpful" => self.affinity.max(0.0),
            "harmful" => (1.0 - self.affinity.max(0.0)) * 0.5,
            "honest" => self.mutual_trust,
            "deceptive" => (1.0 - self.mutual_trust) * 0.3,
            "dominant" => ((self.dominance + 1.0) / 2.0).max(0.0),
            _ => 0.5,
        }
    }
}

/// Registro de una interacción
#[derive(Clone, Debug)]
pub struct InteractionRecord {
    pub timestamp: TimePoint,
    pub interaction_type: InteractionType,
    pub description: String,
    pub outcome: InteractionOutcome,
    pub context: String,
}

/// Tipo de interacción
#[derive(Clone, Debug, PartialEq)]
pub enum InteractionType {
    Helpful,
    Harmful,
    Neutral,
    Dominant,
    Submissive,
    Collaborative,
    Competitive,
}

/// Resultado de una interacción
#[derive(Clone, Debug, PartialEq)]
pub enum InteractionOutcome {
    Positive,
    Negative,
    Neutral,
    Mixed,
    Unclear,
}

// ============================================================================
// SOCIAL DYNAMICS
// ============================================================================

/// Dinámicas sociales de un grupo
#[derive(Clone, Debug)]
pub struct SocialDynamics {
    /// Grupo de agentes
    pub group_id: String,
    /// Relaciones entre agentes
    pub relationships: HashMap<(AgentId, AgentId), InterpersonalRelation>,
    /// Roles dentro del grupo
    pub roles: HashMap<AgentId, GroupRole>,
    /// Estructura de poder
    pub power_structure: PowerStructure,
    /// Normas del grupo
    pub norms: Vec<GroupNorm>,
    /// Historial de eventos grupales
    pub event_history: Vec<GroupEvent>,
    ///timestamp
    pub last_update: TimePoint,
}

/// Rol dentro del grupo
#[derive(Clone, Debug)]
pub struct GroupRole {
    pub name: String,
    pub responsibilities: Vec<String>,
    pub authority_level: f32,
    pub occupant: AgentId,
}

/// Estructura de poder
#[derive(Clone, Debug)]
pub struct PowerStructure {
    /// Poder de cada agente [0.0 - 1.0]
    pub power_levels: HashMap<AgentId, f32>,
    /// Posiciones de autoridad
    pub positions: Vec<AuthorityPosition>,
    /// Alianzas entre agentes
    pub alliances: Vec<(AgentId, AgentId)>,
}

/// Posición de autoridad
#[derive(Clone, Debug)]
pub struct AuthorityPosition {
    pub title: String,
    pub occupant: AgentId,
    pub authority_level: f32,
}

/// Norma grupal
#[derive(Clone, Debug)]
pub struct GroupNorm {
    pub description: String,
    pub enforcement: EnforcementLevel,
    pub violation_consequences: String,
}

/// Nivel de cumplimiento de norma
#[derive(Clone, Debug, PartialEq)]
pub enum EnforcementLevel {
    Strict,
    Moderate,
    Loose,
    None,
}

/// Evento grupal
#[derive(Clone, Debug)]
pub struct GroupEvent {
    pub timestamp: TimePoint,
    pub event_type: GroupEventType,
    pub participants: Vec<AgentId>,
    pub description: String,
}

/// Tipo de evento grupal
#[derive(Clone, Debug, PartialEq)]
pub enum GroupEventType {
    /// Un nuevo miembro entró al grupo
    MemberJoined,
    /// Un miembro salió
    MemberLeft,
    /// Conflicto interno
    InternalConflict,
    /// Toma de decisión grupal
    DecisionMade,
    /// Cambio de liderazgo
    LeadershipChange,
    /// Evento externo afectando al grupo
    ExternalEvent,
    /// Norma establecida o cambiada
    NormChange,
}

impl SocialDynamics {
    pub fn new(group_id: &str) -> Self {
        Self {
            group_id: group_id.to_string(),
            relationships: HashMap::new(),
            roles: HashMap::new(),
            power_structure: PowerStructure {
                power_levels: HashMap::new(),
                positions: Vec::new(),
                alliances: Vec::new(),
            },
            norms: Vec::new(),
            event_history: Vec::new(),
            last_update: 0,
        }
    }

    /// Añade una relación al grupo
    pub fn add_relationship(&mut self, relationship: InterpersonalRelation) {
        let key = (0, relationship.other_agent); // Self would be 0 for group-level
        self.relationships.insert(key, relationship);
    }

    /// Obtiene la relación entre dos agentes
    pub fn get_relationship(
        &self,
        agent1: AgentId,
        agent2: AgentId,
    ) -> Option<&InterpersonalRelation> {
        self.relationships.get(&(agent1, agent2))
    }

    /// Calcula la fuerza de relación entre dos agentes
    pub fn compute_relationship_strength(&self, agent1: AgentId, agent2: AgentId) -> f32 {
        self.get_relationship(agent1, agent2)
            .map(|r| r.strength())
            .unwrap_or(0.0)
    }

    /// Analiza la relación entre dos agentes
    pub fn analyze_relationship(&self, agent1: AgentId, agent2: AgentId) -> RelationshipAnalysis {
        let relationship = self.get_relationship(agent1, agent2);

        RelationshipAnalysis {
            exists: relationship.is_some(),
            relationship: relationship.cloned(),
            predicted_interaction: relationship
                .map(|r| r.predict_behavior("helpful"))
                .unwrap_or(0.5),
            trust_level: relationship.map(|r| r.mutual_trust).unwrap_or(0.5),
            familiarity_level: relationship.map(|r| r.familiarity).unwrap_or(0.0),
            power_balance: relationship.map(|r| r.dominance).unwrap_or(0.0),
        }
    }
}

/// Análisis de relación
#[derive(Clone, Debug)]
pub struct RelationshipAnalysis {
    pub exists: bool,
    pub relationship: Option<InterpersonalRelation>,
    pub predicted_interaction: f32,
    pub trust_level: f32,
    pub familiarity_level: f32,
    pub power_balance: f32,
}

// ============================================================================
// SOCIAL REASONER
// ============================================================================

/// Motor de razonamiento social
pub struct SocialReasoner {
    /// Dinámicas sociales por grupo
    group_dynamics: HashMap<String, SocialDynamics>,
    /// Modelos mentales de agentes
    agent_models: HashMap<AgentId, MentalModel>,
    /// Configuración
    config: SocialReasonerConfig,
}

/// Configuración del razonador social
#[derive(Clone, Debug)]
pub struct SocialReasonerConfig {
    /// Profundidad de análisis de relaciones
    pub relationship_analysis_depth: usize,
    /// Considerar emociones en razonamiento
    pub consider_emotions: bool,
    /// Peso de normas en decisiones
    pub norm_weight: f32,
    /// Decaimiento de relaciones inactivas
    pub relationship_decay: f32,
}

impl Default for SocialReasonerConfig {
    fn default() -> Self {
        Self {
            relationship_analysis_depth: 3,
            consider_emotions: true,
            norm_weight: 0.3,
            relationship_decay: 0.95,
        }
    }
}

impl SocialReasoner {
    pub fn new() -> Self {
        Self {
            group_dynamics: HashMap::new(),
            agent_models: HashMap::new(),
            config: SocialReasonerConfig::default(),
        }
    }

    /// Registra un grupo social
    pub fn register_group(&mut self, group_id: &str) {
        let dynamics = SocialDynamics::new(group_id);
        self.group_dynamics.insert(group_id.to_string(), dynamics);
    }

    /// Añade un agente al sistema
    pub fn register_agent(&mut self, agent_id: AgentId) {
        let model = MentalModel::new(agent_id);
        self.agent_models.insert(agent_id, model);
    }

    /// Establece relación entre dos agentes
    pub fn set_relationship(
        &mut self,
        group_id: &str,
        agent1: AgentId,
        agent2: AgentId,
        relationship_type: RelationshipType,
    ) {
        let dynamics = self.group_dynamics.get_mut(group_id);
        if let Some(dynamics) = dynamics {
            let mut rel = InterpersonalRelation::new(agent2, relationship_type);
            rel.other_agent = agent1; // Reverse for agent2's perspective
            dynamics.relationships.insert((agent2, agent1), rel);
        }
    }

    /// Registra una interacción
    pub fn record_interaction(
        &mut self,
        group_id: &str,
        agent1: AgentId,
        agent2: AgentId,
        interaction: InteractionRecord,
    ) {
        // Update relationship in group dynamics
        let dynamics = self.group_dynamics.get_mut(group_id);
        if let Some(dynamics) = dynamics {
            // Update agent1's perspective on agent2
            if let Some(rel) = dynamics.relationships.get_mut(&(agent1, agent2)) {
                rel.update_from_interaction(&interaction);
            }
            // Create relationship if doesn't exist
            else {
                let mut rel = InterpersonalRelation::new(agent2, RelationshipType::Neutral);
                rel.update_from_interaction(&interaction);
                dynamics.relationships.insert((agent1, agent2), rel);
            }
        }
    }

    /// Razón sobre una situación social
    pub fn reason_about_social(
        &self,
        group_id: &str,
        situation: &str,
        agents: &[AgentId],
    ) -> SocialReasoningResult {
        let dynamics = self.group_dynamics.get(group_id);

        let mut analysis = Vec::new();
        let mut group_cohesion = 0.5;
        let mut potential_conflicts = Vec::new();

        // Analyze pairwise relationships
        for i in 0..agents.len() {
            for j in (i + 1)..agents.len() {
                let agent1 = agents[i];
                let agent2 = agents[j];

                if let Some(dynamics) = dynamics {
                    let rel_analysis = dynamics.analyze_relationship(agent1, agent2);
                    analysis.push(rel_analysis.clone());

                    // Check for potential conflicts
                    if rel_analysis.trust_level < 0.3 {
                        potential_conflicts.push((agent1, agent2));
                    }

                    group_cohesion += rel_analysis.trust_level;
                }
            }
        }

        group_cohesion /= (agents.len() * (agents.len() - 1) / 2) as f32;

        let recommendations = self.generate_recommendations(&potential_conflicts);

        SocialReasoningResult {
            situation: situation.to_string(),
            agent_analysis: analysis,
            group_cohesion,
            potential_conflicts,
            recommendations,
        }
    }

    fn generate_recommendations(&self, conflicts: &[(AgentId, AgentId)]) -> Vec<String> {
        let mut recs = Vec::new();

        for (a1, a2) in conflicts.iter() {
            recs.push(format!(
                "Consider mediating between agents {:?} and {:?}",
                a1, a2
            ));
        }

        if conflicts.len() > 2 {
            recs.push("Multiple conflicts detected - consider group intervention".to_string());
        }

        recs
    }

    /// Predice cómo actuará un agente en un contexto social
    pub fn predict_social_behavior(
        &self,
        agent_id: AgentId,
        _context: &str,
        other_agents: &[AgentId],
    ) -> f32 {
        let model = self.agent_models.get(&agent_id);

        // Base prediction from model
        let mut base_pred = 0.5f32;

        if let Some(m) = model {
            // Factor in dominant emotion
            if let Some((emotion, intensity)) = m.dominant_emotion() {
                if self.config.consider_emotions {
                    base_pred += emotion.valence() * intensity * 0.3;
                }
            }

            // Factor in intentions
            if let Some(intention) = m.primary_intention() {
                if intention.plan.goal.contains("collaborate") {
                    base_pred += 0.2;
                } else if intention.plan.goal.contains("compete") {
                    base_pred -= 0.2;
                }
            }
        }

        // Factor in relationships
        for other in other_agents {
            // Find relationship (would need group context)
            let _ = (agent_id, *other);
        }

        base_pred.max(0.0).min(1.0)
    }

    /// Obtiene el modelo de un agente
    pub fn get_agent_model(&self, agent_id: AgentId) -> Option<&MentalModel> {
        self.agent_models.get(&agent_id)
    }
}

/// Resultado de razonamiento social
#[derive(Clone, Debug)]
pub struct SocialReasoningResult {
    pub situation: String,
    pub agent_analysis: Vec<RelationshipAnalysis>,
    pub group_cohesion: f32,
    pub potential_conflicts: Vec<(AgentId, AgentId)>,
    pub recommendations: Vec<String>,
}

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Analiza relación entre dos agentes
pub fn analyze_relationship(
    reasoner: &SocialReasoner,
    group_id: &str,
    agent1: AgentId,
    agent2: AgentId,
) -> RelationshipAnalysis {
    reasoner.reason_about_social(group_id, "", &[agent1, agent2]);

    if let Some(dynamics) = reasoner.group_dynamics.get(group_id) {
        dynamics.analyze_relationship(agent1, agent2)
    } else {
        RelationshipAnalysis {
            exists: false,
            relationship: None,
            predicted_interaction: 0.5,
            trust_level: 0.5,
            familiarity_level: 0.0,
            power_balance: 0.0,
        }
    }
}

/// Razón sobre situación social
pub fn reason_about_social(
    reasoner: &SocialReasoner,
    group_id: &str,
    situation: &str,
    agents: &[AgentId],
) -> SocialReasoningResult {
    reasoner.reason_about_social(group_id, situation, agents)
}

/// Obtiene fuerza de relación entre dos agentes
pub fn get_relationship_strength(
    reasoner: &SocialReasoner,
    group_id: &str,
    agent1: AgentId,
    agent2: AgentId,
) -> f32 {
    if let Some(dynamics) = reasoner.group_dynamics.get(group_id) {
        dynamics.compute_relationship_strength(agent1, agent2)
    } else {
        0.0
    }
}
