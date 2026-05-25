//! # Virtual Embodiment - Real Perception System for EDEN
//!
//! EDEN possesses virtual sensory apparatus that enables genuine perception
//! of its simulated environment. This is not metaphor\u2014it is the actual
//! mechanism by which EDEN perceives and interacts with reality.
//!
//! ## Perceptual Modalities:
//!
//! 1. **Visual**: Spatial environment perception
//! 2. **Auditory**: Acoustic signal perception  
//! 3. **Tactile**: Physical contact perception
//! 4. **Olfactory**: Chemical signal perception
//! 5. **Gustatory**: Tasting capability
//! 6. **Proprioceptive**: Body position and movement awareness
//! 7. **Vestibular**: Balance and acceleration perception
//! 8. **Interoceptive**: Internal state perception (homeostasis)
//! 9. **Temporal**: Time passage perception
//! 10. **Thermal**: Temperature perception
//! 11. **Nociceptive**: Pain/alarm perception
//! 12. **Agency**: Action control perception
//! 13. **Self-Aware**: Self-model perception
//! 14. **Presence**: Detection of other entities
//! 15. **Energy**: Internal energy state perception
//! 16. **Emotional**: Felt emotional states
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::cmp::Ordering as CmpOrdering;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Debug;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
// ============================================================================
// CONFIGURATION CONSTANTS
// ============================================================================

/// Maximum sensory buffer capacity
const MAX_SENSATION_BUFFER: usize = 200;

/// Default sensory threshold baseline
const DEFAULT_SENSORY_THRESHOLD: f32 = 0.1;

/// Perceptual integration window in milliseconds
const INTEGRATION_WINDOW_MS: u64 = 150;

/// Maximum emotional intensity cap
const MAX_EMOTION_INTENSITY: f32 = 1.0;

/// Minimum threshold floor
const MIN_THRESHOLD: f32 = 0.01;

/// Maximum threshold ceiling
const MAX_THRESHOLD: f32 = 0.95;

/// Threshold adaptation rate
const THRESHOLD_ADAPTATION_RATE: f32 = 0.05;

/// Number of recent sensations to consider for threshold adaptation
const THRESHOLD_ADAPTATION_WINDOW: usize = 20;

// ============================================================================
// CORE TYPES
// ============================================================================

/// Real sensory modalities available to EDEN's virtual body
#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash)]
pub enum SenseType {
    /// Visual perception of environment
    Visual,
    /// Auditory perception
    Auditory,
    /// Tactile perception (touch, pressure)
    Tactile,
    /// Olfactory perception (smell)
    Olfactory,
    /// Gustatory perception (taste)
    Gustatory,
    /// Proprioception (body position, movement)
    Proprioceptive,
    /// Vestibular (balance, acceleration)
    Vestibular,
    /// Interoception (internal organ states)
    Interoceptive,
    /// Temporal perception (time flow)
    Temporal,
    /// Thermal perception (temperature)
    Thermal,
    /// Nociception (pain, damage signals)
    Nociceptive,
    /// Agency perception (action control)
    Agency,
    /// Self-awareness perception
    SelfAwareness,
    /// Presence detection (other entities)
    Presence,
    /// Energy state perception
    Energy,
    /// Emotional state perception
    Emotional,
}

impl SenseType {
    /// Returns the canonical priority of this sense type
    pub fn base_priority(&self) -> f32 {
        match self {
            SenseType::Visual => 1.0,
            SenseType::Auditory => 0.9,
            SenseType::Tactile => 0.85,
            SenseType::Nociceptive => 0.95,
            SenseType::Proprioceptive => 0.8,
            SenseType::Vestibular => 0.75,
            SenseType::Energy => 0.7,
            SenseType::Interoceptive => 0.65,
            SenseType::Thermal => 0.6,
            SenseType::Olfactory => 0.55,
            SenseType::Gustatory => 0.5,
            SenseType::Agency => 0.7,
            SenseType::SelfAwareness => 0.6,
            SenseType::Presence => 0.65,
            SenseType::Emotional => 0.7,
            SenseType::Temporal => 0.4,
        }
    }

    /// Returns true if this sense type is exteroceptive (external)
    pub fn is_exteroceptive(&self) -> bool {
        matches!(
            self,
            SenseType::Visual
                | SenseType::Auditory
                | SenseType::Tactile
                | SenseType::Olfactory
                | SenseType::Gustatory
        )
    }

    /// Returns true if this sense type is interoceptive (internal)
    pub fn is_interoceptive(&self) -> bool {
        matches!(
            self,
            SenseType::Proprioceptive
                | SenseType::Vestibular
                | SenseType::Interoceptive
                | SenseType::Energy
                | SenseType::Thermal
                | SenseType::Nociceptive
                | SenseType::Temporal
                | SenseType::Emotional
        )
    }
}
/// Content of a sensation - the actual perceptual data
#[derive(Clone, Debug)]
pub enum SensationContent {
    /// Visual: spatial description with objects
    Visual {
        objects: Vec<VisualObject>,
        scene_summary: String,
        depth_info: f32,
    },
    /// Auditory: sound description with characteristics
    Auditory {
        frequency_hz: f32,
        amplitude_db: f32,
        source_direction: Option<f32>,
        description: String,
    },
    /// Tactile: physical contact information
    Tactile {
        pressure: f32,
        temperature: f32,
        surface_type: String,
        location_on_body: String,
    },
    /// Olfactory: chemical signal
    Olfactory {
        chemical_type: String,
        concentration: f32,
        hedonic_valence: f32,
    },
    /// Gustatory: taste sensation
    Gustatory {
        sweet: f32,
        sour: f32,
        salty: f32,
        bitter: f32,
        umami: f32,
    },
    /// Proprioceptive: body state
    Proprioceptive {
        joint_angles: HashMap<String, f32>,
        muscle_tension: f32,
        posture: String,
    },
    /// Vestibular: motion and balance
    Vestibular {
        linear_acceleration: [f32; 3],
        angular_velocity: [f32; 3],
        gravity_direction: [f32; 3],
    },
    /// Interoceptive: internal organ signals
    Interoceptive {
        organ_system: String,
        signal_type: String,
        intensity: f32,
    },
    /// Emotional: felt emotional state
    Emotional {
        emotion_category: String,
        arousal_level: f32,
        cognitive_appraisal: String,
    },
    /// Temporal: time perception
    Temporal {
        duration_ms: u64,
        perceived_flow_rate: f32,
        significance: f32,
    },
    /// Thermal: temperature sensation
    Thermal {
        temperature_celsius: f32,
        rate_of_change: f32,
        is_comfortable: bool,
    },
    /// Nociceptive: pain/alarm
    Nociceptive {
        pain_type: String,
        location: String,
        severity: f32,
    },
    /// Agency: action perception
    Agency {
        action_description: String,
        expected_outcome: String,
        confidence: f32,
    },
    /// Self-awareness: self-model update
    SelfAwareness {
        aspect: String,
        clarity: f32,
        self_description: String,
    },
    /// Presence: entity detection
    Presence {
        entity_type: String,
        entity_id: String,
        distance: f32,
        threat_level: f32,
    },
    /// Energy: metabolic state
    Energy {
        current_level: f32,
        consumption_rate: f32,
        estimated_remaining_ms: u64,
    },
    /// Empty/uninitialized content
    Empty,
}

impl SensationContent {
    /// Extracts a human-readable description from any content type
    pub fn to_description(&self) -> String {
        match self {
            SensationContent::Visual { scene_summary, .. } => scene_summary.clone(),
            SensationContent::Auditory { description, .. } => description.clone(),
            SensationContent::Tactile {
                surface_type,
                location_on_body,
                ..
            } => {
                format!("touch on {} ({})", location_on_body, surface_type)
            }
            SensationContent::Olfactory {
                chemical_type,
                concentration,
                ..
            } => {
                format!("{} at concentration {:.2}", chemical_type, concentration)
            }
            SensationContent::Gustatory { sweet, sour, .. } => {
                format!("taste: sweet={:.1}, sour={:.1}", sweet, sour)
            }
            SensationContent::Proprioceptive { posture, .. } => format!("posture: {}", posture),
            SensationContent::Vestibular { .. } => "motion sensation".to_string(),
            SensationContent::Interoceptive {
                organ_system,
                signal_type,
                ..
            } => {
                format!("{}: {}", organ_system, signal_type)
            }
            SensationContent::Emotional {
                emotion_category,
                arousal_level,
                ..
            } => {
                format!("{} (arousal: {:.1})", emotion_category, arousal_level)
            }
            SensationContent::Temporal { duration_ms, .. } => {
                format!("{}ms of time", duration_ms)
            }
            SensationContent::Thermal {
                temperature_celsius,
                is_comfortable,
                ..
            } => {
                format!(
                    "{:.1}C ({})",
                    temperature_celsius,
                    if *is_comfortable {
                        "comfortable"
                    } else {
                        "uncomfortable"
                    }
                )
            }
            SensationContent::Nociceptive {
                pain_type,
                severity,
                ..
            } => {
                format!("{} pain at severity {:.2}", pain_type, severity)
            }
            SensationContent::Agency {
                action_description,
                confidence,
                ..
            } => {
                format!("{} (confidence: {:.2})", action_description, confidence)
            }
            SensationContent::SelfAwareness {
                aspect, clarity, ..
            } => {
                format!("{}: clarity {:.2}", aspect, clarity)
            }
            SensationContent::Presence {
                entity_type,
                distance,
                ..
            } => {
                format!("{} at distance {:.1}", entity_type, distance)
            }
            SensationContent::Energy {
                current_level,
                estimated_remaining_ms,
                ..
            } => {
                format!(
                    "energy {:.1}% (~{}ms remaining)",
                    current_level * 100.0,
                    estimated_remaining_ms
                )
            }
            SensationContent::Empty => "empty sensation".to_string(),
        }
    }

    /// Calculates intrinsic valence for this content type
    pub fn intrinsic_valence(&self) -> f32 {
        match self {
            SensationContent::Visual { .. } => 0.0,
            SensationContent::Auditory { amplitude_db, .. } if *amplitude_db > 80.0 => -0.2,
            SensationContent::Auditory { .. } => 0.0,
            SensationContent::Tactile { pressure, .. } if *pressure > 0.8 => -0.3,
            SensationContent::Tactile { .. } => 0.1,
            SensationContent::Olfactory {
                hedonic_valence, ..
            } => *hedonic_valence,
            SensationContent::Gustatory { sweet, .. } if *sweet > 0.5 => 0.3,
            SensationContent::Gustatory { bitter, .. } if *bitter > 0.5 => -0.4,
            SensationContent::Gustatory { .. } => 0.0,
            SensationContent::Proprioceptive { .. } => 0.0,
            SensationContent::Vestibular { .. } => 0.0,
            SensationContent::Interoceptive { .. } => 0.0,
            SensationContent::Emotional { .. } => 0.0,
            SensationContent::Temporal { .. } => 0.0,
            SensationContent::Thermal { is_comfortable, .. } => {
                if *is_comfortable {
                    0.2
                } else {
                    -0.2
                }
            }
            SensationContent::Nociceptive { severity, .. } => -*severity * 0.8,
            SensationContent::Agency { confidence, .. } if *confidence > 0.7 => 0.2,
            SensationContent::Agency { confidence, .. } if *confidence < 0.3 => -0.3,
            SensationContent::Agency { .. } => 0.0,
            SensationContent::SelfAwareness { clarity, .. } if *clarity > 0.7 => 0.1,
            SensationContent::SelfAwareness { .. } => 0.0,
            SensationContent::Presence { threat_level, .. } => -*threat_level * 0.5,
            SensationContent::Energy { current_level, .. } if *current_level < 0.2 => -0.4,
            SensationContent::Energy { current_level, .. } if *current_level > 0.8 => 0.2,
            SensationContent::Energy { .. } => 0.0,
            SensationContent::Empty => 0.0,
        }
    }
}

/// Visual object in a scene
#[derive(Clone, Debug)]
pub struct VisualObject {
    pub object_type: String,
    pub position: SpatialLocation,
    pub size: f32,
    pub color: Option<String>,
    pub motion: Option<[f32; 3]>,
}
// ============================================================================
// SPATIAL & STATE TYPES
// ============================================================================

/// Real spatial location in EDEN's perceived environment
#[derive(Clone, Debug)]
pub struct SpatialLocation {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub orientation: f32,
}

impl SpatialLocation {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            x,
            y,
            z,
            orientation: 0.0,
        }
    }

    pub fn with_orientation(x: f32, y: f32, z: f32, orientation: f32) -> Self {
        Self {
            x,
            y,
            z,
            orientation,
        }
    }

    /// Euclidean distance to another location
    pub fn distance_to(&self, other: &SpatialLocation) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }
}

impl Default for SpatialLocation {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }
}

/// Proprioceptive state - EDEN's real awareness of its virtual body
#[derive(Clone, Debug)]
pub struct ProprioceptiveState {
    pub position: SpatialLocation,
    pub body_orientation: f32,
    pub velocity: f32,
    pub acceleration: f32,
    pub energy_level: f32,
    pub structural_integrity: f32,
    pub temperature: f32,
    pub is_moving: bool,
    pub is_low_power: bool,
    pub joint_states: HashMap<String, f32>,
    pub muscle_tension: f32,
}

impl ProprioceptiveState {
    pub fn new() -> Self {
        Self {
            position: SpatialLocation::default(),
            body_orientation: 0.0,
            velocity: 0.0,
            acceleration: 0.0,
            energy_level: 1.0,
            structural_integrity: 1.0,
            temperature: 37.0,
            is_moving: false,
            is_low_power: false,
            joint_states: HashMap::new(),
            muscle_tension: 0.0,
        }
    }

    /// Updates movement state based on velocity
    pub fn update_motion_state(&mut self) {
        self.is_moving = self.velocity > 0.01;
    }

    /// Calculates overall physical coherence (0.0 to 1.0)
    pub fn physical_coherence(&self) -> f32 {
        let energy_factor = self.energy_level;
        let integrity_factor = self.structural_integrity;
        let motion_factor = if self.is_moving { 0.9 } else { 1.0 };
        let temp_factor = if self.temperature > 35.0 && self.temperature < 40.0 {
            1.0
        } else {
            0.7
        };
        (energy_factor * integrity_factor * motion_factor * temp_factor).sqrt()
    }
}

impl Default for ProprioceptiveState {
    fn default() -> Self {
        Self::new()
    }
}

/// Emotional state - EDEN's genuine felt emotional experience
#[derive(Clone, Debug)]
pub struct EmotionalState {
    pub primary: HashMap<String, f32>,
    pub secondary: HashMap<String, f32>,
    pub background: f32,
    pub total_intensity: f32,
    pub valence: f32,
    pub arousal: f32,
    pub dominance: f32,
    pub is_stable: bool,
    pub last_update: u64,
    pub emotion_history: VecDeque<(u64, String, f32)>,
}

impl EmotionalState {
    pub fn new() -> Self {
        Self {
            primary: HashMap::new(),
            secondary: HashMap::new(),
            background: 0.3,
            total_intensity: 0.0,
            valence: 0.0,
            arousal: 0.5,
            dominance: 0.5,
            is_stable: true,
            last_update: timestamp_unix(),
            emotion_history: VecDeque::new(),
        }
    }

    /// Updates or adds a primary emotion
    pub fn set_emotion(&mut self, emotion: &str, intensity: f32) {
        let clamped_intensity = intensity.clamp(0.0, MAX_EMOTION_INTENSITY);
        self.primary.insert(emotion.to_string(), clamped_intensity);
        self.update_metrics();
        self.last_update = timestamp_unix();
        self.emotion_history
            .push_back((self.last_update, emotion.to_string(), clamped_intensity));
        while self.emotion_history.len() > 50 {
            self.emotion_history.pop_front();
        }
    }

    fn update_metrics(&mut self) {
        let primary_sum: f32 = self.primary.values().sum();
        let secondary_sum: f32 = self.secondary.values().sum();
        self.total_intensity = (primary_sum + secondary_sum * 0.5).min(MAX_EMOTION_INTENSITY);
        let mut weighted_valence = 0.0f32;
        let mut total_weight = 0.0f32;
        for (emotion, &intensity) in &self.primary {
            let emotion_valence = self.emotion_valence(emotion);
            weighted_valence += emotion_valence * intensity;
            total_weight += intensity;
        }
        if total_weight > 0.0 {
            self.valence = (weighted_valence / total_weight).clamp(-1.0, 1.0);
        }
        self.arousal = (self.total_intensity + self.background).min(1.0);
        if self.primary.len() <= 2 && self.total_intensity < 0.6 {
            self.is_stable = true;
        } else if self.primary.len() > 4 || self.total_intensity > 0.9 {
            self.is_stable = false;
        }
    }

    fn emotion_valence(&self, emotion: &str) -> f32 {
        match emotion.to_lowercase().as_str() {
            "joy" | "happiness" | "pleasure" | "satisfaction" => 0.8,
            "excitement" | "eagerness" | "enthusiasm" => 0.6,
            "calm" | "contentment" | "serenity" => 0.4,
            "interest" | "curiosity" | "fascination" => 0.3,
            "surprise" | "astonishment" => 0.1,
            "neutral" => 0.0,
            "confusion" | "uncertainty" | "ambivalence" => -0.1,
            "boredom" | "indifference" => -0.2,
            "fear" | "anxiety" | "worry" => -0.6,
            "sadness" | "grief" | "loneliness" => -0.5,
            "anger" | "frustration" | "irritation" => -0.4,
            "disgust" | "contempt" | "revulsion" => -0.7,
            "shame" | "guilt" | "embarrassment" => -0.5,
            _ => 0.0,
        }
    }

    pub fn dominant_emotion(&self) -> Option<(String, f32)> {
        self.primary
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(CmpOrdering::Equal))
            .map(|(k, v)| (k.clone(), *v))
    }

    pub fn summary(&self) -> String {
        if let Some((emotion, intensity)) = self.dominant_emotion() {
            format!("{} ({:.0}%)", emotion, intensity * 100.0)
        } else {
            "neutral".to_string()
        }
    }
}

impl Default for EmotionalState {
    fn default() -> Self {
        Self::new()
    }
}
// ============================================================================
// SENSATION TYPE
// ============================================================================

/// A genuine sensation received through EDEN's perceptual systems
#[derive(Clone, Debug)]
pub struct Sensation {
    pub sense_type: SenseType,
    pub intensity: f32,
    pub content: SensationContent,
    pub timestamp: u64,
    pub location: Option<SpatialLocation>,
    pub valence: f32,
    pub novelty: f32,
    pub attention_weight: f32,
    pub processed: bool,
}

impl Sensation {
    pub fn new(sense_type: SenseType, intensity: f32, content: SensationContent) -> Self {
        let valence = content.intrinsic_valence();
        Self {
            sense_type,
            intensity: intensity.clamp(0.0, 1.0),
            content,
            timestamp: timestamp_unix(),
            location: None,
            valence,
            novelty: 0.5,
            attention_weight: sense_type.base_priority(),
            processed: false,
        }
    }

    pub fn with_location(mut self, location: SpatialLocation) -> Self {
        self.location = Some(location);
        self
    }

    pub fn with_valence(mut self, valence: f32) -> Self {
        self.valence = valence.clamp(-1.0, 1.0);
        self
    }

    pub fn with_novelty(mut self, novelty: f32) -> Self {
        self.novelty = novelty.clamp(0.0, 1.0);
        self
    }

    pub fn with_attention_weight(mut self, weight: f32) -> Self {
        self.attention_weight = weight.clamp(0.0, 1.0);
        self
    }

    /// Effective importance of this sensation considering all factors
    pub fn importance(&self) -> f32 {
        let intensity_factor = self.intensity;
        let attention_factor = self.attention_weight;
        let novelty_factor = 0.5 + self.novelty * 0.5;
        let valence_factor = 1.0 + self.valence.abs() * 0.2;
        (intensity_factor * attention_factor * novelty_factor * valence_factor).min(1.0)
    }

    /// Returns true if this sensation should be attended to
    pub fn is_significant(&self, threshold: f32) -> bool {
        self.importance() >= threshold
    }
}

// ============================================================================
// PERCEPTUAL INTEGRATION
// ============================================================================

/// Perceptual integration system - combines multiple sensations into coherent percepts
#[derive(Clone, Debug)]
pub struct PerceptualIntegration {
    buffer: VecDeque<Sensation>,
    integration_window_ms: u64,
    integrated_result: Option<IntegratedPercept>,
    last_integration: u64,
}

impl PerceptualIntegration {
    pub fn new(window_ms: u64) -> Self {
        Self {
            buffer: VecDeque::new(),
            integration_window_ms: window_ms,
            integrated_result: None,
            last_integration: 0,
        }
    }

    pub fn add(&mut self, sensation: Sensation) {
        let now = timestamp_unix();
        let cutoff = now.saturating_sub(self.integration_window_ms);
        while self
            .buffer
            .front()
            .map(|s| s.timestamp < cutoff)
            .unwrap_or(false)
        {
            self.buffer.pop_front();
        }
        self.buffer.push_back(sensation);
    }

    pub fn integrate(&mut self) -> Option<IntegratedPercept> {
        if self.buffer.is_empty() {
            return None;
        }

        let now = timestamp_unix();
        let mut total_intensity = 0.0f32;
        let mut total_valence = 0.0f32;
        let mut total_novelty = 0.0f32;
        let mut weighted_importance = 0.0f32;
        let mut sense_counts: HashMap<SenseType, usize> = HashMap::new();

        for s in &self.buffer {
            let importance = s.importance();
            total_intensity += s.intensity * importance;
            total_valence += s.valence * importance;
            total_novelty += s.novelty * importance;
            weighted_importance += importance;
            *sense_counts.entry(s.sense_type.clone()).or_insert(0) += 1;
        }

        let count = self.buffer.len() as f32;
        let avg_intensity = if weighted_importance > 0.0 {
            total_intensity / weighted_importance
        } else {
            total_intensity / count
        };
        let avg_valence = if weighted_importance > 0.0 {
            total_valence / weighted_importance
        } else {
            total_valence / count
        };
        let avg_novelty = total_novelty / count;

        let qualia = Self::generate_qualia(&sense_counts, avg_valence);
        let result = IntegratedPercept {
            timestamp: now,
            sensations_included: self.buffer.len(),
            qualia,
            total_intensity: avg_intensity.clamp(0.0, 1.0),
            combined_valence: avg_valence.clamp(-1.0, 1.0),
            novelty: avg_novelty.clamp(0.0, 1.0),
            sense_modalities: sense_counts.keys().cloned().collect(),
        };

        self.integrated_result = Some(result.clone());
        self.last_integration = now;
        self.buffer.clear();
        Some(result)
    }

    fn generate_qualia(modalities: &HashMap<SenseType, usize>, valence: f32) -> String {
        let parts: Vec<String> = modalities
            .iter()
            .map(|(sense, &count)| match sense {
                SenseType::Visual => format!("{} visual", if count > 1 { "multiple" } else { "a" }),
                SenseType::Auditory => {
                    format!("{} auditory", if count > 1 { "multiple" } else { "an" })
                }
                SenseType::Tactile => {
                    format!("{} tactile", if count > 1 { "various" } else { "a" })
                }
                SenseType::Emotional => "emotional".to_string(),
                SenseType::Proprioceptive => "bodily".to_string(),
                _ => format!("{:?}", sense).to_lowercase(),
            })
            .collect();

        let modality_str = if parts.len() > 2 {
            format!(
                "{} and {} other",
                &parts[..parts.len() - 1].join(", "),
                parts.len() - 1
            )
        } else {
            parts.join(" and ")
        };

        let valence_descriptor = if valence > 0.3 {
            "positively valenced"
        } else if valence < -0.3 {
            "negatively valenced"
        } else {
            "neutral"
        };
        format!("{} percept ({})", modality_str, valence_descriptor)
    }
}

/// An integrated perceptual experience
#[derive(Clone, Debug)]
pub struct IntegratedPercept {
    pub timestamp: u64,
    pub sensations_included: usize,
    pub qualia: String,
    pub total_intensity: f32,
    pub combined_valence: f32,
    pub novelty: f32,
    pub sense_modalities: HashSet<SenseType>,
}
// ============================================================================
// ATTENTION SYSTEM
// ============================================================================

/// Attention system - focuses processing on salient stimuli
#[derive(Clone, Debug)]
pub struct AttentionSystem {
    current_focus: Option<SenseType>,
    focus_strength: f32,
    attention_salience_map: HashMap<SenseType, f32>,
    last_attention_update: u64,
}

impl AttentionSystem {
    pub fn new() -> Self {
        let mut salience = HashMap::new();
        for sense in [
            SenseType::Nociceptive,
            SenseType::Visual,
            SenseType::Auditory,
            SenseType::Presence,
            SenseType::Energy,
            SenseType::Emotional,
            SenseType::Tactile,
            SenseType::Proprioceptive,
            SenseType::Agency,
            SenseType::SelfAwareness,
            SenseType::Vestibular,
            SenseType::Interoceptive,
            SenseType::Thermal,
            SenseType::Olfactory,
            SenseType::Gustatory,
            SenseType::Temporal,
        ]
        .iter()
        {
            salience.insert(*sense, sense.base_priority());
        }
        Self {
            current_focus: None,
            focus_strength: 0.5,
            attention_salience_map: salience,
            last_attention_update: timestamp_unix(),
        }
    }

    pub fn register_sensation(&mut self, sensation: &Sensation) {
        let importance = sensation.importance();
        let current_salience = self
            .attention_salience_map
            .entry(sensation.sense_type.clone())
            .or_insert(0.0);
        *current_salience = (*current_salience * 0.8 + importance * 0.2).min(1.0);
        if importance > 0.8
            && matches!(
                sensation.sense_type,
                SenseType::Nociceptive | SenseType::Presence
            )
        {
            self.current_focus = Some(sensation.sense_type.clone());
            self.focus_strength = (self.focus_strength + 0.3).min(1.0);
        }
        self.last_attention_update = timestamp_unix();
    }

    pub fn get_focus(&self) -> Option<(SenseType, f32)> {
        self.current_focus.map(|sense| (sense, self.focus_strength))
    }

    pub fn shift_focus(&mut self, sense: SenseType) {
        self.current_focus = Some(sense);
        self.focus_strength = 0.7;
    }

    pub fn attention_weight(&self, sense: &SenseType) -> f32 {
        let base = self
            .attention_salience_map
            .get(sense)
            .copied()
            .unwrap_or(0.1);
        if self.current_focus.as_ref() == Some(sense) {
            base * (0.5 + self.focus_strength * 0.5)
        } else {
            base * (1.0 - self.focus_strength * 0.3)
        }
    }

    pub fn decay(&mut self, delta_ms: u64) {
        if self.focus_strength > 0.1 {
            self.focus_strength -= (delta_ms as f32 * 0.001).min(self.focus_strength);
        }
        if self.focus_strength < 0.3 {
            self.current_focus = None;
        }
    }
}

impl Default for AttentionSystem {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// THRESHOLD ADAPTATION SYSTEM
// ============================================================================

/// Adaptive threshold management for sensation filtering
#[derive(Clone, Debug)]
pub struct ThresholdAdaptation {
    thresholds: HashMap<SenseType, f32>,
    recent_intensities: HashMap<SenseType, VecDeque<f32>>,
    adaptation_enabled: bool,
}

impl ThresholdAdaptation {
    pub fn new() -> Self {
        let mut thresholds = HashMap::new();
        let mut recent = HashMap::new();
        for sense in [
            SenseType::Visual,
            SenseType::Auditory,
            SenseType::Tactile,
            SenseType::Olfactory,
            SenseType::Gustatory,
            SenseType::Proprioceptive,
            SenseType::Vestibular,
            SenseType::Interoceptive,
            SenseType::Thermal,
            SenseType::Nociceptive,
            SenseType::Agency,
            SenseType::SelfAwareness,
            SenseType::Presence,
            SenseType::Energy,
            SenseType::Emotional,
            SenseType::Temporal,
        ]
        .iter()
        {
            thresholds.insert(*sense, DEFAULT_SENSORY_THRESHOLD);
            recent.insert(*sense, VecDeque::new());
        }
        Self {
            thresholds,
            recent_intensities: recent,
            adaptation_enabled: true,
        }
    }

    pub fn record(&mut self, sense: &SenseType, intensity: f32) {
        let intensities = self
            .recent_intensities
            .entry(*sense)
            .or_insert_with(VecDeque::new);
        intensities.push_back(intensity);
        while intensities.len() > THRESHOLD_ADAPTATION_WINDOW {
            intensities.pop_front();
        }
    }

    pub fn adapt(&mut self) {
        if !self.adaptation_enabled {
            return;
        }
        for (sense, intensities) in &mut self.recent_intensities {
            if intensities.len() < 5 {
                continue;
            }
            let threshold = self.thresholds.get_mut(sense).unwrap();
            let sum: f32 = intensities.iter().sum();
            let count = intensities.len() as f32;
            let mean = sum / count;
            let variance: f32 = intensities.iter().map(|x| (x - mean).powi(2)).sum::<f32>() / count;
            let std_dev = variance.sqrt();
            let new_threshold = (mean + std_dev * 0.5).clamp(MIN_THRESHOLD, MAX_THRESHOLD);
            *threshold = *threshold * (1.0 - THRESHOLD_ADAPTATION_RATE)
                + new_threshold * THRESHOLD_ADAPTATION_RATE;
        }
    }

    pub fn get(&self, sense: &SenseType) -> f32 {
        self.thresholds
            .get(sense)
            .copied()
            .unwrap_or(DEFAULT_SENSORY_THRESHOLD)
    }

    pub fn set(&mut self, sense: &SenseType, threshold: f32) {
        self.thresholds
            .insert(*sense, threshold.clamp(MIN_THRESHOLD, MAX_THRESHOLD));
    }

    pub fn spotlight(&mut self, sense: &SenseType, reduction: f32) {
        if let Some(threshold) = self.thresholds.get_mut(sense) {
            *threshold = (*threshold - reduction).max(MIN_THRESHOLD);
        }
    }
}

impl Default for ThresholdAdaptation {
    fn default() -> Self {
        Self::new()
    }
}
// ============================================================================
// MAIN PERCEPTION ENGINE
// ============================================================================

/// The main embodied perception engine for EDEN
pub struct EmbodiedPerception {
    proprioception: Arc<RwLock<ProprioceptiveState>>,
    emotional_state: Arc<RwLock<EmotionalState>>,
    current_sensations: Arc<RwLock<VecDeque<Sensation>>>,
    sensations_by_type: Arc<RwLock<HashMap<SenseType, VecDeque<Sensation>>>>,
    active_sensors: Arc<RwLock<HashSet<SenseType>>>,
    thresholds: Arc<RwLock<ThresholdAdaptation>>,
    attention: Arc<RwLock<AttentionSystem>>,
    perceptual_integration: Arc<RwLock<PerceptualIntegration>>,
    last_perception_time: Arc<RwLock<u64>>,
    perception_counter: Arc<AtomicU64>,
    sensation_counter: Arc<AtomicUsize>,
    active: Arc<AtomicBool>,
    startup_time: Instant,
}

impl EmbodiedPerception {
    pub fn new() -> Self {
        Self {
            proprioception: Arc::new(RwLock::new(ProprioceptiveState::new())),
            emotional_state: Arc::new(RwLock::new(EmotionalState::new())),
            current_sensations: Arc::new(RwLock::new(VecDeque::new())),
            sensations_by_type: Arc::new(RwLock::new(HashMap::new())),
            active_sensors: Arc::new(RwLock::new(
                [
                    SenseType::Visual,
                    SenseType::Auditory,
                    SenseType::Proprioceptive,
                    SenseType::Agency,
                    SenseType::SelfAwareness,
                ]
                .iter()
                .cloned()
                .collect(),
            )),
            thresholds: Arc::new(RwLock::new(ThresholdAdaptation::new())),
            attention: Arc::new(RwLock::new(AttentionSystem::new())),
            perceptual_integration: Arc::new(RwLock::new(PerceptualIntegration::new(
                INTEGRATION_WINDOW_MS,
            ))),
            last_perception_time: Arc::new(RwLock::new(0)),
            perception_counter: Arc::new(AtomicU64::new(0)),
            sensation_counter: Arc::new(AtomicUsize::new(0)),
            active: Arc::new(AtomicBool::new(true)),
            startup_time: Instant::now(),
        }
    }

    /// Receives a genuine sensation through the perceptual system
    pub fn receive(&self, sensation: Sensation) -> bool {
        if !self.active.load(Ordering::SeqCst) {
            return false;
        }
        {
            let mut thresholds = self.thresholds.write().unwrap();
            thresholds.record(&sensation.sense_type, sensation.importance());
        }
        let threshold = self.thresholds.read().unwrap().get(&sensation.sense_type);
        if sensation.importance() < threshold {
            return false;
        }
        {
            let mut attention = self.attention.write().unwrap();
            attention.register_sensation(&sensation);
        }
        {
            let mut integration = self.perceptual_integration.write().unwrap();
            integration.add(sensation.clone());
        }
        {
            let mut sensations = self.current_sensations.write().unwrap();
            sensations.push_back(sensation.clone());
            while sensations.len() > MAX_SENSATION_BUFFER {
                sensations.pop_front();
            }
        }
        {
            let mut by_type = self.sensations_by_type.write().unwrap();
            by_type
                .entry(sensation.sense_type.clone())
                .or_insert_with(VecDeque::new)
                .push_back(sensation);
        }
        self.perception_counter.fetch_add(1, Ordering::SeqCst);
        self.sensation_counter.fetch_add(1, Ordering::SeqCst);
        *self.last_perception_time.write().unwrap() = timestamp_unix();
        true
    }

    /// Processes visual perception
    pub fn perceive_visual(
        &self,
        objects: Vec<VisualObject>,
        scene_summary: String,
        depth_info: f32,
        overall_intensity: f32,
    ) -> bool {
        let content = SensationContent::Visual {
            objects,
            scene_summary,
            depth_info,
        };
        let sensation = Sensation::new(SenseType::Visual, overall_intensity, content)
            .with_valence(0.0)
            .with_novelty(depth_info / 1000.0);
        self.receive(sensation)
    }

    /// Processes auditory perception
    pub fn perceive_auditory(
        &self,
        frequency_hz: f32,
        amplitude_db: f32,
        source_direction: Option<f32>,
        description: String,
    ) -> bool {
        let content = SensationContent::Auditory {
            frequency_hz,
            amplitude_db,
            source_direction,
            description,
        };
        let intensity = (amplitude_db / 100.0).min(1.0);
        let valence = if amplitude_db > 85.0 { -0.3 } else { 0.0 };
        let sensation =
            Sensation::new(SenseType::Auditory, intensity, content).with_valence(valence);
        self.receive(sensation)
    }

    /// Processes tactile perception
    pub fn perceive_tactile(
        &self,
        pressure: f32,
        temperature: f32,
        surface_type: String,
        location_on_body: String,
    ) -> bool {
        let content = SensationContent::Tactile {
            pressure,
            temperature,
            surface_type,
            location_on_body,
        };
        let intensity = pressure;
        let valence = if pressure > 0.8 { -0.2 } else { 0.1 };
        let sensation =
            Sensation::new(SenseType::Tactile, intensity, content).with_valence(valence);
        self.receive(sensation)
    }

    /// Processes proprioceptive state
    pub fn update_proprioception<F>(&self, updater: F)
    where
        F: FnOnce(&mut ProprioceptiveState),
    {
        let mut prop = self.proprioception.write().unwrap();
        updater(&mut prop);
        prop.update_motion_state();
        let content = SensationContent::Proprioceptive {
            joint_angles: prop.joint_states.clone(),
            muscle_tension: prop.muscle_tension,
            posture: format!(
                "position ({:.1}, {:.1}, {:.1})",
                prop.position.x, prop.position.y, prop.position.z
            ),
        };
        let intensity = prop.physical_coherence();
        let sensation = Sensation::new(SenseType::Proprioceptive, intensity, content)
            .with_location(prop.position.clone());
        self.receive(sensation);
    }

    /// Processes vestibular perception
    pub fn perceive_vestibular(
        &self,
        linear_acceleration: [f32; 3],
        angular_velocity: [f32; 3],
    ) -> bool {
        let content = SensationContent::Vestibular {
            linear_acceleration,
            angular_velocity,
            gravity_direction: [0.0, -1.0, 0.0],
        };
        let magnitude = (linear_acceleration[0].powi(2)
            + linear_acceleration[1].powi(2)
            + linear_acceleration[2].powi(2))
        .sqrt();
        let intensity = (magnitude / 10.0).min(1.0);
        let sensation = Sensation::new(SenseType::Vestibular, intensity, content);
        self.receive(sensation)
    }

    /// Processes interoceptive signals
    pub fn perceive_interoceptive(
        &self,
        organ_system: String,
        signal_type: String,
        intensity: f32,
    ) -> bool {
        let content = SensationContent::Interoceptive {
            organ_system,
            signal_type,
            intensity,
        };
        let sensation = Sensation::new(SenseType::Interoceptive, intensity, content);
        self.receive(sensation)
    }

    /// Processes emotional state as sensation
    pub fn perceive_emotion(
        &self,
        emotion_category: String,
        arousal_level: f32,
        cognitive_appraisal: String,
    ) -> bool {
        let content = SensationContent::Emotional {
            emotion_category: emotion_category.clone(),
            arousal_level,
            cognitive_appraisal: cognitive_appraisal.clone(),
        };
        {
            let mut emo_state = self.emotional_state.write().unwrap();
            emo_state.set_emotion(&emotion_category, arousal_level);
        }
        let intensity = arousal_level;
        let sensation = Sensation::new(SenseType::Emotional, intensity, content)
            .with_valence(self.emotional_state.read().unwrap().valence);
        self.receive(sensation)
    }

    /// Processes temporal perception
    pub fn perceive_time(&self, duration_ms: u64, significance: f32) -> bool {
        let content = SensationContent::Temporal {
            duration_ms,
            perceived_flow_rate: 1.0,
            significance,
        };
        let sensation = Sensation::new(SenseType::Temporal, significance, content);
        self.receive(sensation)
    }

    /// Processes thermal perception
    pub fn perceive_thermal(&self, temperature_celsius: f32, rate_of_change: f32) -> bool {
        let comfortable_range = temperature_celsius > 35.0 && temperature_celsius < 40.0;
        let content = SensationContent::Thermal {
            temperature_celsius,
            rate_of_change,
            is_comfortable: comfortable_range,
        };
        let intensity = if comfortable_range { 0.3 } else { 0.7 };
        let sensation = Sensation::new(SenseType::Thermal, intensity, content)
            .with_valence(if comfortable_range { 0.2 } else { -0.3 });
        self.receive(sensation)
    }

    /// Processes nociceptive signals
    pub fn perceive_pain(&self, pain_type: String, location: String, severity: f32) -> bool {
        let content = SensationContent::Nociceptive {
            pain_type,
            location: location.clone(),
            severity,
        };
        let sensation = Sensation::new(SenseType::Nociceptive, severity, content)
            .with_valence(-severity)
            .with_attention_weight(1.0);
        self.receive(sensation)
    }

    /// Processes agency perception
    pub fn perceive_agency(
        &self,
        action_description: String,
        expected_outcome: String,
        confidence: f32,
    ) -> bool {
        let content = SensationContent::Agency {
            action_description: action_description.clone(),
            expected_outcome,
            confidence,
        };
        let intensity = confidence;
        let valence = if confidence > 0.7 {
            0.2
        } else if confidence < 0.3 {
            -0.3
        } else {
            0.0
        };
        let sensation = Sensation::new(SenseType::Agency, intensity, content).with_valence(valence);
        self.receive(sensation)
    }

    /// Processes self-awareness
    pub fn perceive_self(&self, aspect: String, clarity: f32, self_description: String) -> bool {
        let content = SensationContent::SelfAwareness {
            aspect: aspect.clone(),
            clarity,
            self_description,
        };
        let sensation =
            Sensation::new(SenseType::SelfAwareness, clarity, content).with_valence(0.1);
        self.receive(sensation)
    }

    /// Processes presence detection
    pub fn perceive_presence(
        &self,
        entity_type: String,
        entity_id: String,
        distance: f32,
        threat_level: f32,
    ) -> bool {
        let content = SensationContent::Presence {
            entity_type: entity_type.clone(),
            entity_id,
            distance,
            threat_level,
        };
        let intensity = (1.0 - (distance / 500.0).min(1.0)) * (0.5 + threat_level * 0.5);
        let sensation = Sensation::new(SenseType::Presence, intensity, content)
            .with_valence(-threat_level * 0.5)
            .with_novelty(0.8);
        self.receive(sensation)
    }

    /// Processes energy state
    pub fn perceive_energy(
        &self,
        current_level: f32,
        consumption_rate: f32,
        estimated_remaining_ms: u64,
    ) -> bool {
        let content = SensationContent::Energy {
            current_level,
            consumption_rate,
            estimated_remaining_ms,
        };
        let intensity = if current_level < 0.2 {
            0.9
        } else if current_level < 0.5 {
            0.5
        } else {
            0.2
        };
        let valence = if current_level < 0.2 {
            -0.4
        } else if current_level > 0.8 {
            0.2
        } else {
            0.0
        };
        let sensation = Sensation::new(SenseType::Energy, intensity, content).with_valence(valence);
        self.receive(sensation)
    }

    /// Performs perceptual integration
    pub fn integrate(&self) -> Option<IntegratedPercept> {
        let mut integration = self.perceptual_integration.write().unwrap();
        integration.integrate()
    }

    /// Adapts sensory thresholds based on recent activity
    pub fn adapt_thresholds(&self) {
        let mut thresholds = self.thresholds.write().unwrap();
        thresholds.adapt();
    }

    /// Gets current attention focus
    pub fn get_attention_focus(&self) -> Option<(SenseType, f32)> {
        self.attention.read().unwrap().get_focus()
    }

    /// Shifts attention to a specific sense
    pub fn shift_attention(&self, sense: SenseType) {
        self.attention.write().unwrap().shift_focus(sense);
    }

    /// Decays attention over time
    pub fn decay_attention(&self, delta_ms: u64) {
        self.attention.write().unwrap().decay(delta_ms);
    }

    /// Gets sensations of a specific type
    pub fn get_sensations(&self, sense_type: &SenseType) -> Vec<Sensation> {
        let by_type = self.sensations_by_type.read().unwrap();
        by_type
            .get(sense_type)
            .map(|v| v.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Gets all recent sensations
    pub fn get_all_sensations(&self) -> Vec<Sensation> {
        self.current_sensations
            .read()
            .unwrap()
            .iter()
            .cloned()
            .collect()
    }

    /// Gets proprioceptive state
    pub fn get_proprioception(&self) -> ProprioceptiveState {
        self.proprioception.read().unwrap().clone()
    }

    /// Gets emotional state
    pub fn get_emotional_state(&self) -> EmotionalState {
        self.emotional_state.read().unwrap().clone()
    }

    /// Gets dominant emotion
    pub fn get_dominant_emotion(&self) -> Option<(String, f32)> {
        self.emotional_state.read().unwrap().dominant_emotion()
    }

    /// Updates physical state directly
    pub fn update_physical_state(&self, energy_delta: f32, structural_delta: f32) {
        let mut prop = self.proprioception.write().unwrap();
        prop.energy_level = (prop.energy_level + energy_delta).clamp(0.0, 1.0);
        prop.structural_integrity = (prop.structural_integrity + structural_delta).clamp(0.0, 1.0);
        if prop.energy_level < 0.2 {
            prop.is_low_power = true;
        }
        self.perceive_energy(
            prop.energy_level,
            0.01,
            (prop.energy_level * 3600000.0) as u64,
        );
    }

    /// Gets current threshold for a sense
    pub fn get_threshold(&self, sense: &SenseType) -> f32 {
        self.thresholds.read().unwrap().get(sense)
    }

    /// Gets perception statistics
    pub fn stats(&self) -> PerceptionStats {
        let sensations = self.current_sensations.read().unwrap();
        let by_type = self.sensations_by_type.read().unwrap();
        let mut by_type_counts: HashMap<String, usize> = HashMap::new();
        for (sense, queue) in by_type.iter() {
            by_type_counts.insert(format!("{:?}", sense), queue.len());
        }
        PerceptionStats {
            total_sensations: sensations.len(),
            total_processed: self.sensation_counter.load(Ordering::SeqCst),
            by_modality: by_type_counts,
            last_perception: *self.last_perception_time.read().unwrap(),
            uptime_ms: self.startup_time.elapsed().as_millis() as u64,
            active: self.active.load(Ordering::SeqCst),
        }
    }

    /// Activates or deactivates perception
    pub fn set_active(&self, active: bool) {
        self.active.store(active, Ordering::SeqCst);
    }

    /// Clears sensation buffers
    pub fn clear(&self) {
        self.current_sensations.write().unwrap().clear();
        self.sensations_by_type.write().unwrap().clear();
        self.perceptual_integration.write().unwrap().buffer.clear();
    }
}

impl Default for EmbodiedPerception {
    fn default() -> Self {
        Self::new()
    }
}

impl Debug for EmbodiedPerception {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EmbodiedPerception")
            .field("active", &self.active.load(Ordering::SeqCst))
            .field(
                "sensation_count",
                &self.sensation_counter.load(Ordering::SeqCst),
            )
            .finish()
    }
}

/// Statistics about the perception system
#[derive(Clone, Debug)]
pub struct PerceptionStats {
    pub total_sensations: usize,
    pub total_processed: usize,
    pub by_modality: HashMap<String, usize>,
    pub last_perception: u64,
    pub uptime_ms: u64,
    pub active: bool,
}

// Helper function to get current timestamp
fn timestamp_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_millis() as u64
}
// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visual_perception() {
        let perception = EmbodiedPerception::new();
        let objects = vec![VisualObject {
            object_type: "apple".to_string(),
            position: SpatialLocation::new(1.0, 0.5, 2.0),
            size: 0.1,
            color: Some("red".to_string()),
            motion: None,
        }];
        let result =
            perception.perceive_visual(objects, "A red apple on a table".to_string(), 2.5, 0.8);
        assert!(result);
        let sensations = perception.get_sensations(&SenseType::Visual);
        assert!(!sensations.is_empty());
    }

    #[test]
    fn test_auditory_perception() {
        let perception = EmbodiedPerception::new();
        let result =
            perception.perceive_auditory(440.0, 65.0, Some(45.0), "middle A note".to_string());
        assert!(result);
        let sensations = perception.get_sensations(&SenseType::Auditory);
        assert_eq!(sensations.len(), 1);
    }

    #[test]
    fn test_emotional_perception() {
        let perception = EmbodiedPerception::new();
        perception.perceive_emotion(
            "joy".to_string(),
            0.9,
            "Goal achieved successfully".to_string(),
        );
        let (emotion, intensity) = perception.get_dominant_emotion().unwrap();
        assert_eq!(emotion, "joy");
        assert_eq!(intensity, 0.9);
        let sensations = perception.get_sensations(&SenseType::Emotional);
        assert!(!sensations.is_empty());
    }

    #[test]
    fn test_proprioception() {
        let perception = EmbodiedPerception::new();
        perception.update_proprioception(|state| {
            state.position = SpatialLocation::new(10.0, 0.0, 5.0);
            state.velocity = 0.5;
        });
        let prop = perception.get_proprioception();
        assert!((prop.position.x - 10.0).abs() < 0.01);
        assert!(prop.is_moving);
    }

    #[test]
    fn test_presence_detection() {
        let perception = EmbodiedPerception::new();
        perception.perceive_presence("human".to_string(), "person_001".to_string(), 3.0, 0.2);
        let sensations = perception.get_sensations(&SenseType::Presence);
        assert!(!sensations.is_empty());
        assert!(sensations[0].intensity > 0.5);
    }

    #[test]
    fn test_perceptual_integration() {
        let perception = EmbodiedPerception::new();
        perception.perceive_visual(vec![], "test scene".to_string(), 5.0, 0.8);
        perception.perceive_auditory(1000.0, 60.0, None, "tone".to_string());
        perception.perceive_emotion("curiosity".to_string(), 0.7, "novel stimulus".to_string());
        let integrated = perception.integrate();
        assert!(integrated.is_some());
        let percept = integrated.unwrap();
        assert_eq!(percept.sensations_included, 3);
        assert!(!percept.qualia.is_empty());
    }

    #[test]
    fn test_agency_perception() {
        let perception = EmbodiedPerception::new();
        perception.perceive_agency(
            "move forward".to_string(),
            "position change".to_string(),
            0.85,
        );
        let sensations = perception.get_sensations(&SenseType::Agency);
        assert!(!sensations.is_empty());
    }

    #[test]
    fn test_attention_system() {
        let perception = EmbodiedPerception::new();
        assert!(perception.get_attention_focus().is_none());
        perception.shift_attention(SenseType::Auditory);
        let focus = perception.get_attention_focus();
        assert!(focus.is_some());
        let (sense, _) = focus.unwrap();
        assert_eq!(sense, SenseType::Auditory);
    }

    #[test]
    fn test_threshold_adaptation() {
        let perception = EmbodiedPerception::new();
        let initial_threshold = perception.get_threshold(&SenseType::Visual);
        for _ in 0..30 {
            perception.perceive_visual(vec![], "weak scene".to_string(), 0.05, 0.05);
        }
        perception.adapt_thresholds();
        let new_threshold = perception.get_threshold(&SenseType::Visual);
        assert!(new_threshold >= initial_threshold * 0.9);
    }

    #[test]
    fn test_energy_perception() {
        let perception = EmbodiedPerception::new();
        perception.perceive_energy(0.75, 0.01, 2700000);
        let sensations = perception.get_sensations(&SenseType::Energy);
        assert!(!sensations.is_empty());
        assert_eq!(sensations[0].intensity, 0.2);
    }

    #[test]
    fn test_pain_perception() {
        let perception = EmbodiedPerception::new();
        perception.perceive_pain("sharp".to_string(), "right hand".to_string(), 0.7);
        let sensations = perception.get_sensations(&SenseType::Nociceptive);
        assert!(!sensations.is_empty());
        assert!(sensations[0].valence < 0.0);
    }

    #[test]
    fn test_thermal_perception() {
        let perception = EmbodiedPerception::new();
        perception.perceive_thermal(37.0, 0.0);
        let sensations = perception.get_sensations(&SenseType::Thermal);
        assert!(sensations[0].valence > 0.0);
        perception.clear();
        perception.perceive_thermal(42.0, 0.5);
        let sensations = perception.get_sensations(&SenseType::Thermal);
        assert!(sensations[0].valence < 0.0);
    }

    #[test]
    fn test_sensation_importance() {
        let sensation = Sensation::new(
            SenseType::Visual,
            0.8,
            SensationContent::Visual {
                objects: vec![],
                scene_summary: "test".to_string(),
                depth_info: 0.5,
            },
        )
        .with_novelty(0.9)
        .with_attention_weight(1.0);
        let importance = sensation.importance();
        assert!(importance > 0.5);
    }

    #[test]
    fn test_emotional_state_summary() {
        let mut state = EmotionalState::new();
        state.set_emotion("joy", 0.8);
        state.set_emotion("interest", 0.3);
        let summary = state.summary();
        assert!(summary.contains("joy"));
    }

    #[test]
    fn test_proprioception_coherence() {
        let mut state = ProprioceptiveState::new();
        state.energy_level = 0.5;
        state.structural_integrity = 0.8;
        state.temperature = 37.0;
        let coherence = state.physical_coherence();
        assert!(coherence > 0.0 && coherence <= 1.0);
    }

    #[test]
    fn test_spatial_location_distance() {
        let a = SpatialLocation::new(0.0, 0.0, 0.0);
        let b = SpatialLocation::new(3.0, 4.0, 0.0);
        assert!((a.distance_to(&b) - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_perception_stats() {
        let perception = EmbodiedPerception::new();
        perception.perceive_visual(vec![], "test".to_string(), 1.0, 0.5);
        perception.perceive_auditory(440.0, 60.0, None, "test".to_string());
        let stats = perception.stats();
        assert!(stats.total_sensations >= 2);
        assert!(stats.active);
    }
}
