//! # Emotion - Emotion Detection and Modeling
//!
//! Sistema de detección y modelado de emociones 100% original.
//! Sin dependencias.
//!
//! ## Características
//!
//! 1. **EmotionDetector**: Detecta emociones desde señales observables
//! 2. **EmotionalState**: Representa estado emocional completo
//! 3. **EmotionDynamics**: Modela la dinámica temporal de emociones
//! 4. **AffectiveModel**: Modelo afectivo completo
//!
//! ## Señales Emocionales
//!
//! - Expresiones faciales (conceptuales)
//! - Tono de voz (conceptual)
//! - Comportamiento
//! - Respuestas fisiológicas (modeladas)
//! - Interacción social
#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(unused_imports)]

use crate::theory_of_mind::{AgentId, EmotionType, TimePoint};
use std::collections::{HashMap, VecDeque};

// ============================================================================
// EMOTION DETECTOR
// ============================================================================

/// Detector de emociones desde señales
pub struct EmotionDetector {
    /// Configuración de detección
    config: DetectorConfig,
    /// Modelos emocionales por agente
    emotional_models: HashMap<AgentId, EmotionalState>,
    /// Historial de detecciones
    detection_history: Vec<EmotionDetection>,
}

/// Configuración del detector
#[derive(Clone, Debug)]
pub struct DetectorConfig {
    /// Sensibilidad mínima para detección
    pub min_sensitivity: f32,
    /// Peso de señales comportamentales
    pub behavioral_weight: f32,
    /// Peso de señales fisiológicas
    pub physiological_weight: f32,
    /// Peso de contexto social
    pub social_context_weight: f32,
    /// Decaimiento emocional
    pub emotion_decay: f32,
}

impl Default for DetectorConfig {
    fn default() -> Self {
        Self {
            min_sensitivity: 0.3,
            behavioral_weight: 0.4,
            physiological_weight: 0.3,
            social_context_weight: 0.3,
            emotion_decay: 0.95,
        }
    }
}

/// Detección de emoción
#[derive(Clone, Debug)]
pub struct EmotionDetection {
    pub agent_id: AgentId,
    pub emotion: EmotionType,
    pub intensity: f32,
    pub confidence: f32,
    pub signals: Vec<EmotionSignal>,
    pub timestamp: TimePoint,
}

/// Señal emocional observable
#[derive(Clone, Debug, PartialEq)]
pub enum EmotionSignal {
    /// Expresión facial
    FacialExpression {
        expression: FacialExpression,
        intensity: f32,
    },
    /// Tono de voz
    VocalTone { tone: VocalTone, intensity: f32 },
    /// Comportamiento específico
    Behavior { behavior: String, deviation: f32 },
    /// Respuesta fisiológica
    Physiological {
        response: PhysiologicalResponse,
        value: f32,
    },
    /// Elección en decisión
    Choice { choice: String, risk_level: f32 },
    /// Interacción social
    SocialInteraction { interaction: String, warmth: f32 },
}

/// Expresión facial
#[derive(Clone, Debug, PartialEq)]
pub enum FacialExpression {
    Smile,
    Frown,
    Surprise,
    Contempt,
    Anger,
    Fear,
    Disgust,
    Neutral,
}

/// Tono de voz
#[derive(Clone, Debug, PartialEq)]
pub enum VocalTone {
    Happy,
    Sad,
    Angry,
    Anxious,
    Calm,
    Excited,
    Flat,
}

/// Respuesta fisiológica
#[derive(Clone, Debug, PartialEq)]
pub enum PhysiologicalResponse {
    HeartRate,
    SkinConductance,
    PupilDilation,
    BreathingRate,
    MuscleTension,
}

impl EmotionDetector {
    pub fn new() -> Self {
        Self {
            config: DetectorConfig::default(),
            emotional_models: HashMap::new(),
            detection_history: Vec::new(),
        }
    }

    /// Detecta emociones desde señales
    pub fn detect_emotion(
        &mut self,
        agent_id: AgentId,
        signals: &[EmotionSignal],
        context: &str,
        timestamp: TimePoint,
    ) -> Vec<EmotionDetection> {
        let mut detections = Vec::new();

        // Pre-compute context weight first (before any mutable borrows)
        let context_weight = self.weight_by_context(context);

        // Analyze each signal (collect results before getting entry borrow)
        let mut emotion_scores: HashMap<EmotionType, f32> = HashMap::new();

        for signal in signals {
            let signal_emotions = self.analyze_signal(signal);
            for (emotion, score) in signal_emotions {
                *emotion_scores.entry(emotion).or_insert(0.0) += score;
            }
        }

        // Get or create emotional state (this creates the mutable borrow)
        let state = self
            .emotional_models
            .entry(agent_id)
            .or_insert_with(|| EmotionalState::new());

        // Normalize and threshold
        let total: f32 = emotion_scores.values().sum();
        if total > 0.0 {
            for (emotion, raw_score) in emotion_scores {
                let normalized = raw_score / total;
                if normalized >= self.config.min_sensitivity {
                    let confidence = normalized * context_weight;
                    let detection = EmotionDetection {
                        agent_id,
                        emotion: emotion.clone(),
                        intensity: normalized,
                        confidence,
                        signals: signals.to_vec(),
                        timestamp,
                    };
                    detections.push(detection.clone());
                    state.set_emotion(emotion, normalized);
                    self.detection_history.push(detection);
                }
            }
        }

        detections
    }

    fn analyze_signal(&self, signal: &EmotionSignal) -> HashMap<EmotionType, f32> {
        let mut scores = HashMap::new();

        match signal {
            EmotionSignal::FacialExpression {
                expression,
                intensity,
            } => {
                let emotion_scores = match expression {
                    FacialExpression::Smile => vec![
                        (EmotionType::Joy, *intensity),
                        (EmotionType::Contentment, *intensity * 0.7),
                    ],
                    FacialExpression::Frown => vec![
                        (EmotionType::Sadness, *intensity),
                        (EmotionType::Despair, *intensity * 0.5),
                    ],
                    FacialExpression::Surprise => vec![
                        (EmotionType::Surprise, *intensity),
                        (EmotionType::Fear, *intensity * 0.4),
                    ],
                    FacialExpression::Contempt => vec![
                        (EmotionType::Contempt, *intensity),
                        (EmotionType::Disgust, *intensity * 0.6),
                    ],
                    FacialExpression::Anger => vec![(EmotionType::Anger, *intensity)],
                    FacialExpression::Fear => vec![(EmotionType::Fear, *intensity)],
                    FacialExpression::Disgust => vec![(EmotionType::Disgust, *intensity)],
                    FacialExpression::Neutral => vec![],
                };
                for (emotion, score) in emotion_scores {
                    scores.insert(emotion, score * self.config.behavioral_weight);
                }
            }
            EmotionSignal::VocalTone { tone, intensity } => {
                let emotion_scores = match tone {
                    VocalTone::Happy => vec![
                        (EmotionType::Joy, *intensity),
                        (EmotionType::Anticipation, *intensity * 0.6),
                    ],
                    VocalTone::Sad => vec![(EmotionType::Sadness, *intensity)],
                    VocalTone::Angry => vec![(EmotionType::Anger, *intensity)],
                    VocalTone::Anxious => vec![
                        (EmotionType::Fear, *intensity * 0.7),
                        (EmotionType::Anticipation, *intensity * 0.5),
                    ],
                    VocalTone::Calm => vec![(EmotionType::Contentment, *intensity * 0.8)],
                    VocalTone::Excited => vec![
                        (EmotionType::Joy, *intensity),
                        (EmotionType::Surprise, *intensity * 0.5),
                    ],
                    VocalTone::Flat => vec![],
                };
                for (emotion, score) in emotion_scores {
                    scores.insert(emotion, score * self.config.behavioral_weight);
                }
            }
            EmotionSignal::Behavior { deviation, .. } => {
                // Analyze behavior deviation for emotion
                if *deviation > 0.7 {
                    scores.insert(EmotionType::Anxiety, *deviation * 0.8);
                }
                if *deviation < -0.3 {
                    scores.insert(EmotionType::Contentment, -*deviation * 0.6);
                }
            }
            EmotionSignal::Physiological { response, value } => {
                let emotion_scores = match response {
                    PhysiologicalResponse::HeartRate => {
                        if *value > 0.7 {
                            vec![(EmotionType::Fear, 0.6), (EmotionType::Anger, 0.4)]
                        } else if *value < 0.3 {
                            vec![(EmotionType::Contentment, 0.5)]
                        } else {
                            vec![]
                        }
                    }
                    PhysiologicalResponse::SkinConductance => {
                        if *value > 0.6 {
                            vec![(EmotionType::Anxiety, 0.7)]
                        } else {
                            vec![]
                        }
                    }
                    PhysiologicalResponse::PupilDilation => {
                        if *value > 0.5 {
                            vec![(EmotionType::Interest, 0.6), (EmotionType::Fear, 0.3)]
                        } else {
                            vec![]
                        }
                    }
                    _ => vec![],
                };
                for (emotion, score) in emotion_scores {
                    scores.insert(emotion, score * self.config.physiological_weight);
                }
            }
            EmotionSignal::Choice { risk_level, .. } => {
                if *risk_level > 0.7 {
                    scores.insert(EmotionType::Fear, *risk_level * 0.5);
                    scores.insert(EmotionType::Hope, *risk_level * 0.4);
                }
            }
            EmotionSignal::SocialInteraction { warmth, .. } => {
                if *warmth > 0.7 {
                    scores.insert(EmotionType::Trust, *warmth * 0.8);
                    scores.insert(EmotionType::Love, *warmth * 0.5);
                } else if *warmth < 0.3 {
                    scores.insert(EmotionType::Disgust, (1.0 - warmth) * 0.6);
                }
            }
        }

        scores
    }

    fn weight_by_context(&self, context: &str) -> f32 {
        // Context affects detection confidence
        let base = match context {
            "social" => 1.2,
            "stressful" => 1.1,
            "relaxed" => 0.9,
            _ => 1.0,
        };
        base
    }

    /// Obtiene estado emocional de un agente
    pub fn get_emotional_state(&self, agent_id: AgentId) -> Option<&EmotionalState> {
        self.emotional_models.get(&agent_id)
    }
}

/// Emoción simple para función helper
impl From<&str> for EmotionType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "joy" | "felicidad" => EmotionType::Joy,
            "sadness" | "tristeza" => EmotionType::Sadness,
            "anger" | "ira" => EmotionType::Anger,
            "fear" | "miedo" => EmotionType::Fear,
            "surprise" | "sorpresa" => EmotionType::Surprise,
            "disgust" | "disgusto" => EmotionType::Disgust,
            "trust" | "confianza" => EmotionType::Trust,
            "anticipation" | "anticipacion" => EmotionType::Anticipation,
            "love" | "amor" => EmotionType::Love,
            _ => EmotionType::Contentment,
        }
    }
}

/// Función helper para detectar emoción
pub fn emotion_from_signal(signals: &[EmotionSignal]) -> HashMap<EmotionType, f32> {
    let detector = EmotionDetector::new();
    let mut scores = HashMap::new();

    for signal in signals {
        let signal_scores = detector.analyze_signal(signal);
        for (emotion, score) in signal_scores {
            *scores.entry(emotion).or_insert(0.0) += score;
        }
    }

    scores
}

/// Función helper para detectar emociones
pub fn detect_emotion(
    detector: &mut EmotionDetector,
    agent_id: AgentId,
    signals: &[EmotionSignal],
    context: &str,
    timestamp: TimePoint,
) -> Vec<EmotionDetection> {
    detector.detect_emotion(agent_id, signals, context, timestamp)
}

// ============================================================================
// EMOTIONAL STATE
// ============================================================================

/// Estado emocional de un agente
#[derive(Clone, Debug)]
pub struct EmotionalState {
    /// Emociones actuales con intensidades
    pub emotions: HashMap<EmotionType, f32>,
    /// Emociones dominantes (top 3)
    pub dominant: Vec<(EmotionType, f32)>,
    /// Estado de ánimo base
    pub mood: Mood,
    /// Valence general (-1.0 a 1.0)
    pub valence: f32,
    /// Arousal general (0.0 a 1.0)
    pub arousal: f32,
    ///timestamp
    pub last_update: TimePoint,
}

/// Estado de ánimo base
#[derive(Clone, Debug, PartialEq)]
pub enum Mood {
    Positive,
    Negative,
    Neutral,
    Anxious,
    Calm,
    Excited,
}

impl EmotionalState {
    pub fn new() -> Self {
        Self {
            emotions: HashMap::new(),
            dominant: Vec::new(),
            mood: Mood::Neutral,
            valence: 0.0,
            arousal: 0.5,
            last_update: 0,
        }
    }

    pub fn set_emotion(&mut self, emotion: EmotionType, intensity: f32) {
        self.emotions.insert(emotion, intensity.max(0.0).min(1.0));
        self.update_dominant();
        self.update_valence_arousal();
        self.last_update += 1;
    }

    fn update_dominant(&mut self) {
        let mut sorted: Vec<_> = self.emotions.iter().collect();
        sorted.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());
        self.dominant = sorted
            .into_iter()
            .take(3)
            .map(|(k, v)| (k.clone(), *v))
            .collect();
    }

    fn update_valence_arousal(&mut self) {
        let mut total_valence = 0.0f32;
        let mut total_arousal = 0.0f32;
        let mut count = 0usize;

        for (emotion, &intensity) in &self.emotions {
            total_valence += emotion.valence() * intensity;
            total_arousal += emotion.arousal() * intensity;
            count += 1;
        }

        if count > 0 {
            self.valence = total_valence / count as f32;
            self.arousal = total_arousal / count as f32;
        }

        // Update mood based on valence/arousal
        self.mood = match (self.valence, self.arousal) {
            (v, _) if v > 0.5 && self.arousal > 0.6 => Mood::Excited,
            (v, _) if v > 0.3 => Mood::Positive,
            (v, _) if v < -0.3 => Mood::Negative,
            (_, a) if a > 0.7 => Mood::Anxious,
            (_, a) if a < 0.3 => Mood::Calm,
            _ => Mood::Neutral,
        };
    }

    pub fn decay(&mut self, factor: f32) {
        for intensity in self.emotions.values_mut() {
            *intensity *= factor;
        }
        self.update_dominant();
        self.update_valence_arousal();
    }
}

// ============================================================================
// EMOTION DYNAMICS
// ============================================================================

/// Modelo de dinámica emocional temporal
#[derive(Clone, Debug)]
pub struct EmotionDynamics {
    /// Historial de estados emocionales
    history: VecDeque<(TimePoint, EmotionalState)>,
    /// Máxima profundidad del historial
    max_history: usize,
    /// Constantes de tiempo por emoción
    time_constants: HashMap<EmotionType, f32>,
}

impl EmotionDynamics {
    pub fn new() -> Self {
        let mut time_constants = HashMap::new();
        time_constants.insert(EmotionType::Joy, 2.0);
        time_constants.insert(EmotionType::Sadness, 3.0);
        time_constants.insert(EmotionType::Anger, 1.5);
        time_constants.insert(EmotionType::Fear, 1.0);
        time_constants.insert(EmotionType::Surprise, 0.5);
        time_constants.insert(EmotionType::Disgust, 2.0);
        time_constants.insert(EmotionType::Hope, 2.5);
        time_constants.insert(EmotionType::Despair, 4.0);

        Self {
            history: VecDeque::new(),
            max_history: 100,
            time_constants,
        }
    }

    /// Añade un nuevo estado
    pub fn add_state(&mut self, timestamp: TimePoint, state: EmotionalState) {
        if self.history.len() >= self.max_history {
            self.history.pop_front();
        }
        self.history.push_back((timestamp, state));
    }

    /// Predice siguiente estado emocional
    pub fn predict_next(&self, current: &EmotionalState, delta_time: f32) -> EmotionalState {
        let mut predicted = current.clone();

        for (emotion, current_intensity) in &mut predicted.emotions {
            if let Some(&tau) = self.time_constants.get(emotion) {
                // Exponential decay model
                let decay = (-delta_time / tau).exp();
                *current_intensity *= decay;
            }
        }

        predicted.update_dominant();
        predicted.update_valence_arousal();
        predicted
    }

    /// Detecta cambio emocional
    pub fn detect_change(&self, from_idx: usize, to_idx: usize) -> Option<(EmotionType, f32)> {
        if from_idx >= self.history.len() || to_idx >= self.history.len() {
            return None;
        }

        let from = &self.history[from_idx].1;
        let to = &self.history[to_idx].1;

        let mut max_change = 0.0f32;
        let mut changed_emotion = None;

        for (emotion, &intensity) in &to.emotions {
            let from_intensity = from.emotions.get(emotion).copied().unwrap_or(0.0);
            let change = (intensity - from_intensity).abs();
            if change > max_change {
                max_change = change;
                changed_emotion = Some(emotion.clone());
            }
        }

        changed_emotion.map(|e| (e, max_change))
    }
}

/// Tracking de dinámica emocional
pub fn track_emotion_dynamics(
    dynamics: &mut EmotionDynamics,
    state: &EmotionalState,
    timestamp: TimePoint,
) {
    dynamics.add_state(timestamp, state.clone());
}

// ============================================================================
// AFFECTIVE MODEL
// ============================================================================

/// Modelo afectivo completo (para empatía)
#[derive(Clone, Debug)]
pub struct AffectiveModel {
    /// Estado emocional actual
    pub current_state: EmotionalState,
    /// Tendencia emocional (rumbo)
    pub trend: EmotionTrend,
    /// Vulnerabilidades emocionales
    pub vulnerabilities: Vec<EmotionVulnerability>,
    /// Mecanismos de regulación
    pub regulation_mechanisms: Vec<RegulationMechanism>,
}

/// Tendencia emocional
#[derive(Clone, Debug)]
pub enum EmotionTrend {
    Rising,
    Stable,
    Falling,
    Fluctuating,
}

/// Vulnerabilidad emocional
#[derive(Clone, Debug)]
pub struct EmotionVulnerability {
    pub emotion: EmotionType,
    pub trigger: String,
    pub intensity: f32,
    pub recovery_time: f32,
}

/// Mecanismo de regulación emocional
#[derive(Clone, Debug)]
pub struct RegulationMechanism {
    pub name: String,
    pub description: String,
    pub effectiveness: f32,
    pub used_count: u32,
}

impl AffectiveModel {
    pub fn from_emotional_state(state: EmotionalState) -> Self {
        Self {
            current_state: state,
            trend: EmotionTrend::Stable,
            vulnerabilities: Vec::new(),
            regulation_mechanisms: Vec::new(),
        }
    }

    /// Simula empatía hacia el modelo
    pub fn simulate_empathy(&self, other: &AffectiveModel) -> f32 {
        // Calculate emotional resonance
        let val_diff = (self.current_state.valence - other.current_state.valence).abs();
        let arou_diff = (self.current_state.arousal - other.current_state.arousal).abs();

        let valence_similarity = 1.0 - val_diff;
        let arousal_similarity = 1.0 - arou_diff;

        (valence_similarity * 0.6 + arousal_similarity * 0.4).max(0.0)
    }
}
