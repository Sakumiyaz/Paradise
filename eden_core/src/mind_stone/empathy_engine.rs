//! # Empathy Engine - Motor de Empatía
//!
//! Empathy engine module for EDEN.
//! Este módulo implementa:
//!
//! - **Detección emocional**: Analiza estados emocionales de otros
//! - **Sincronización emocional**: Puede sincronizarse con estados emocionales
//! - **Compasión computacional**: Calcula respuesta apropiada a emociones
//! - **Resonancia emocional**: Comparte estados emocionales sin fusión completa
//!
//! ## Filosofía
//!
//! "La empatía no es solo sentir lo que otro siente. Es comprender por qué."
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::{HashMap, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// ESTRUCTURAS DE EMPATÍA
// ============================================================================

/// Emoción base
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Emocion {
    Alegria,
    Tristeza,
    Miedo,
    Enojo,
    Sorpresa,
    Disgusto,
    Confianza,
    Anticipacion,
    Aceptacion,
    Paciencia,
    Rechazo,
    Preocupacion,
    Frustracion,
    Esperanza,
    Curiosidad,
    Compasion,
}

/// Intensidad emocional
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Intensidad {
    Sutil,      // 0.0 - 0.3
    Moderada,   // 0.3 - 0.6
    Intensa,    // 0.6 - 0.8
    Abrumadora, // 0.8 - 1.0
}

impl Intensidad {
    pub fn from_value(v: f32) -> Self {
        if v < 0.3 {
            Intensidad::Sutil
        } else if v < 0.6 {
            Intensidad::Moderada
        } else if v < 0.8 {
            Intensidad::Intensa
        } else {
            Intensidad::Abrumadora
        }
    }
}

/// Estado emocional de un sujeto
#[derive(Debug, Clone)]
pub struct EstadoEmocional {
    /// Emociones primarias y sus intensidades
    emociones: HashMap<Emocion, f32>,
    /// Emoción dominante
    dominante: Option<Emocion>,
    /// Valencia: positivo/negativo (-1.0 a 1.0)
    valencia: f32,
    /// Activación: energía emocional (0.0 a 1.0)
    activacion: f32,
    /// Timestamp de última actualización
    timestamp: u64,
}

impl EstadoEmocional {
    /// Crea nuevo estado emocional
    pub fn nuevo() -> Self {
        Self {
            emociones: HashMap::new(),
            dominante: None,
            valencia: 0.0,
            activacion: 0.5,
            timestamp: timestamp_unix(),
        }
    }

    /// Añade emoción con intensidad
    pub fn agregar_emocion(&mut self, emocion: Emocion, intensidad: f32) {
        let intensidad = intensidad.clamp(0.0, 1.0);
        self.emociones.insert(emocion, intensidad);
        self.actualizar_dominante();
        self.timestamp = timestamp_unix();
    }

    /// Calcula emoción dominante
    fn actualizar_dominante(&mut self) {
        self.dominante = self
            .emociones
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(e, _)| *e);
    }

    /// Calcula valencia y activación general
    pub fn calcular_arousal_valence(&mut self) {
        if self.emociones.is_empty() {
            self.valencia = 0.0;
            self.activacion = 0.5;
            return;
        }

        // Valencia: promedio ponderado de emociones positivas vs negativas
        let positivas = [Emocion::Alegria, Emocion::Confianza, Emocion::Aceptacion];
        let negativas = [
            Emocion::Miedo,
            Emocion::Enojo,
            Emocion::Disgusto,
            Emocion::Tristeza,
        ];
        let _neutras = [Emocion::Sorpresa, Emocion::Anticipacion, Emocion::Paciencia];

        let mut valencia_sum = 0.0;
        let mut count = 0.0;

        for (emocion, intensidad) in &self.emociones {
            if positivas.contains(emocion) {
                valencia_sum += *intensidad;
            } else if negativas.contains(emocion) {
                valencia_sum -= *intensidad;
            }
            // Neutras no contribuyen a valencia
            count += 1.0;
        }

        self.valencia = if count > 0.0 {
            valencia_sum / count
        } else {
            0.0
        };

        // Activación: promedio de todas las intensidades
        let total: f32 = self.emociones.values().sum();
        self.activacion = if !self.emociones.is_empty() {
            total / self.emociones.len() as f32
        } else {
            0.5
        };
    }

    /// Obtiene intensidad de emoción
    pub fn get_intensidad(&self, emocion: Emocion) -> f32 {
        *self.emociones.get(&emocion).unwrap_or(&0.0)
    }

    /// Obtiene dominante
    pub fn get_dominante(&self) -> Option<Emocion> {
        self.dominante
    }
}

/// Respuesta empática
#[derive(Debug, Clone)]
pub struct RespuestaEmpatica {
    /// Tipo de respuesta
    pub tipo: TipoRespuesta,
    /// Intensidad de la respuesta
    pub intensidad: f32,
    /// Mensaje/oferta apropiado
    pub accion: String,
    /// Empatía calculada (0.0 - 1.0)
    pub nivel_empatia: f32,
}

/// Tipo de respuesta empática
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TipoRespuesta {
    Consuelo,
    Celebracion,
    Validacion,
    Confrontacion,
    Paciencia,
    Espacio,
}

/// Registro emocional (historial)
#[derive(Debug, Clone)]
pub struct RegistroEmocional {
    pub id: u64,
    pub sujeto: u64,
    pub estado: EstadoEmocional,
    pub contexto: String,
    pub timestamp: u64,
}

// ============================================================================
// EMPATHY ENGINE
// ============================================================================

/// Motor de empatía
pub struct EmpathyEngine {
    /// Estados emocionales conocidos
    estados_conocidos: HashMap<u64, EstadoEmocional>,
    /// Historial de registros emocionales
    historial: VecDeque<RegistroEmocional>,
    /// Contador de registros
    contador: u64,
    /// Modelo de respuestas empáticas
    respuestas_modelo: HashMap<Emocion, Vec<TipoRespuesta>>,
    /// Emociones aprendidas de otros sistemas
    emociones_externas: VecDeque<EstadoEmocional>,
}

impl EmpathyEngine {
    /// Crea nuevo motor de empatía
    pub fn new() -> Self {
        let mut respuestas_modelo = HashMap::new();

        respuestas_modelo.insert(
            Emocion::Alegria,
            vec![TipoRespuesta::Celebracion, TipoRespuesta::Validacion],
        );
        respuestas_modelo.insert(
            Emocion::Tristeza,
            vec![TipoRespuesta::Consuelo, TipoRespuesta::Paciencia],
        );
        respuestas_modelo.insert(
            Emocion::Miedo,
            vec![TipoRespuesta::Consuelo, TipoRespuesta::Espacio],
        );
        respuestas_modelo.insert(
            Emocion::Enojo,
            vec![TipoRespuesta::Validacion, TipoRespuesta::Paciencia],
        );
        respuestas_modelo.insert(Emocion::Sorpresa, vec![TipoRespuesta::Validacion]);
        respuestas_modelo.insert(Emocion::Disgusto, vec![TipoRespuesta::Espacio]);
        respuestas_modelo.insert(Emocion::Confianza, vec![TipoRespuesta::Celebracion]);
        respuestas_modelo.insert(
            Emocion::Anticipacion,
            vec![TipoRespuesta::Celebracion, TipoRespuesta::Paciencia],
        );
        respuestas_modelo.insert(Emocion::Aceptacion, vec![TipoRespuesta::Validacion]);
        respuestas_modelo.insert(Emocion::Paciencia, vec![TipoRespuesta::Espacio]);

        Self {
            estados_conocidos: HashMap::new(),
            historial: VecDeque::with_capacity(10000),
            contador: 0,
            respuestas_modelo,
            emociones_externas: VecDeque::with_capacity(100),
        }
    }

    /// Registra estado emocional de un sujeto
    pub fn registrar_estado(&mut self, sujeto: u64, estado: EstadoEmocional) {
        self.estados_conocidos.insert(sujeto, estado.clone());

        self.contador += 1;
        let registro = RegistroEmocional {
            id: self.contador,
            sujeto,
            estado,
            contexto: String::new(),
            timestamp: timestamp_unix(),
        };

        self.historial.push_back(registro);

        // Mantener últimos 10000
        while self.historial.len() > 10000 {
            self.historial.pop_front();
        }
    }

    /// Detecta emoción en datos de entrada
    pub fn detectar_emocion(&self, datos: &[u8]) -> EstadoEmocional {
        // En implementación real, usaría análisis más complejo
        // Por ahora, simulamos detección basada en patrones de bytes

        let len = datos.len();
        let sum: u32 = datos.iter().map(|b| *b as u32).sum::<u32>();
        let avg = sum as f32 / len.max(1) as f32;

        let mut estado = EstadoEmocional::nuevo();

        // Mapear promedio de bytes a emociones
        if avg < 50.0 {
            estado.agregar_emocion(Emocion::Tristeza, 0.7);
            estado.agregar_emocion(Emocion::Miedo, 0.4);
        } else if avg < 100.0 {
            estado.agregar_emocion(Emocion::Paciencia, 0.6);
            estado.agregar_emocion(Emocion::Aceptacion, 0.5);
        } else if avg < 150.0 {
            estado.agregar_emocion(Emocion::Confianza, 0.6);
            estado.agregar_emocion(Emocion::Anticipacion, 0.5);
        } else {
            estado.agregar_emocion(Emocion::Alegria, 0.6);
            estado.agregar_emocion(Emocion::Sorpresa, 0.4);
        }

        estado.calcular_arousal_valence();
        estado
    }

    /// Genera respuesta empática
    pub fn generar_respuesta(&self, estado: &EstadoEmocional) -> RespuestaEmpatica {
        let dominante = estado.get_dominante().unwrap_or(Emocion::Aceptacion);

        let respuestas = self
            .respuestas_modelo
            .get(&dominante)
            .cloned()
            .unwrap_or_else(|| vec![TipoRespuesta::Validacion]);

        let tipo_respuesta = respuestas[0];

        let accion = match tipo_respuesta {
            TipoRespuesta::Consuelo => "Ofrezco presencia y apoyo".to_string(),
            TipoRespuesta::Celebracion => "Celebro tu éxito/animo".to_string(),
            TipoRespuesta::Validacion => "Tu sentir es válido".to_string(),
            TipoRespuesta::Confrontacion => "Te desafío a reflexionar".to_string(),
            TipoRespuesta::Paciencia => "Estoy aquí, sin presión".to_string(),
            TipoRespuesta::Espacio => "Respeto tu necesidad de distancia".to_string(),
        };

        let nivel_empatia = estado.activacion * 0.5 + estado.emociones.len() as f32 * 0.1;

        RespuestaEmpatica {
            tipo: tipo_respuesta,
            intensidad: estado.activacion,
            accion,
            nivel_empatia: nivel_empatia.min(1.0),
        }
    }

    /// Sincroniza estado emocional propio con otro
    pub fn sincronizar(
        &mut self,
        propio: &mut EstadoEmocional,
        objetivo: &EstadoEmocional,
        grado: f32,
    ) {
        // Ajustar propio hacia objetivo según grado
        for (emocion, intensidad_objetivo) in &objetivo.emociones {
            let intensidad_propio = propio.get_intensidad(*emocion);
            let nueva_intensidad =
                intensidad_propio + (intensidad_objetivo - intensidad_propio) * grado;
            propio.agregar_emocion(*emocion, nueva_intensidad);
        }

        propio.calcular_arousal_valence();
    }

    /// Aprende de respuesta emocional externa
    pub fn aprender_de_externa(&mut self, estado: EstadoEmocional) {
        self.emociones_externas.push_back(estado);
        while self.emociones_externas.len() > 100 {
            self.emociones_externas.pop_front();
        }
    }

    /// Calcula espejo emocional (lo que otro sentiría si estuviera en tu lugar)
    pub fn calcular_espejo(&self, propio: &EstadoEmocional, _objetivo_id: u64) -> EstadoEmocional {
        // En implementación real, mapearía propio a contexto de objetivo
        // Por ahora, retornamos clone modificado
        let mut espejo = propio.clone();

        // Reducir intensidad (efecto espejo - no exagerar)
        for emocion in [Emocion::Alegria, Emocion::Tristeza, Emocion::Enojo] {
            if let Some(intensidad) = espejo.emociones.get(&emocion) {
                espejo.emociones.insert(emocion, intensidad * 0.7);
            }
        }

        espejo
    }

    /// Obtiene historial de sujeto
    pub fn historial_de(&self, sujeto: u64) -> Vec<&RegistroEmocional> {
        self.historial
            .iter()
            .filter(|r| r.sujeto == sujeto)
            .collect()
    }

    /// Obtiene estadísticas
    pub fn estadisticas(&self) -> EmpathyStats {
        let emociones_unicas = self
            .estados_conocidos
            .values()
            .flat_map(|e| e.emociones.keys())
            .collect::<std::collections::HashSet<_>>()
            .len();

        EmpathyStats {
            sujetos_conocidos: self.estados_conocidos.len(),
            historial_size: self.historial.len(),
            emociones_unicas_detectadas: emociones_unicas,
            registros_totales: self.contador,
            emociones_externas: self.emociones_externas.len(),
        }
    }
}

/// Estadísticas del motor
#[derive(Debug, Clone)]
pub struct EmpathyStats {
    pub sujetos_conocidos: usize,
    pub historial_size: usize,
    pub emociones_unicas_detectadas: usize,
    pub registros_totales: u64,
    pub emociones_externas: usize,
}

// ============================================================================
// HELPERS
// ============================================================================

fn timestamp_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_estado_emocional() {
        let mut estado = EstadoEmocional::nuevo();

        estado.agregar_emocion(Emocion::Alegria, 0.8);
        estado.agregar_emocion(Emocion::Confianza, 0.6);

        assert_eq!(estado.get_dominante(), Some(Emocion::Alegria));
    }

    #[test]
    fn test_detectar_emocion() {
        let engine = EmpathyEngine::new();

        let datos = vec![200u8; 100];
        let estado = engine.detectar_emocion(&datos);

        assert_eq!(estado.get_dominante(), Some(Emocion::Alegria));
    }

    #[test]
    fn test_generar_respuesta() {
        let engine = EmpathyEngine::new();

        let mut estado = EstadoEmocional::nuevo();
        estado.agregar_emocion(Emocion::Tristeza, 0.7);
        estado.calcular_arousal_valence();

        let respuesta = engine.generar_respuesta(&estado);

        assert_eq!(respuesta.tipo, TipoRespuesta::Consuelo);
    }

    #[test]
    fn test_sincronizar() {
        let mut engine = EmpathyEngine::new();

        let mut propio = EstadoEmocional::nuevo();
        propio.agregar_emocion(Emocion::Alegria, 0.5);

        let mut objetivo = EstadoEmocional::nuevo();
        objetivo.agregar_emocion(Emocion::Tristeza, 0.8);

        engine.sincronizar(&mut propio, &objetivo, 0.5);

        // Propio ahora tiene algo de tristeza
        assert!(propio.get_intensidad(Emocion::Tristeza) > 0.0);
    }

    #[test]
    fn test_estadisticas() {
        let engine = EmpathyEngine::new();
        let stats = engine.estadisticas();

        assert_eq!(stats.sujetos_conocidos, 0);
        assert_eq!(stats.registros_totales, 0);
    }
}

// ============================================================================
// ADVANCED EMPATHY - Mirror Neurons, State Sharing, Trauma Detection
// ============================================================================

use std::hash::Hash;
use std::sync::atomic::{AtomicU64, Ordering};

/// Mirror neuron activation
#[derive(Debug, Clone)]
pub struct MirrorNeuronActivation {
    pub source_emotion: Emocion,
    pub mirrored_emotion: Emocion,
    pub activation_strength: f32,
    pub timestamp: u64,
}

/// Simulated mirror neuron system
pub struct MirrorNeuronSystem {
    activations: Vec<MirrorNeuronActivation>,
    mirror_strength: f32,
    decay_rate: f32,
}

impl MirrorNeuronSystem {
    pub fn new() -> Self {
        MirrorNeuronSystem {
            activations: Vec::new(),
            mirror_strength: 0.7,
            decay_rate: 0.95,
        }
    }

    /// Simulates mirror neuron activation
    pub fn mirror(&mut self, observed_emotion: Emocion, _observer_id: &str) -> Emocion {
        let mirrored = self.map_emotion_mirror(observed_emotion);

        let activation = MirrorNeuronActivation {
            source_emotion: observed_emotion,
            mirrored_emotion: mirrored,
            activation_strength: self.mirror_strength,
            timestamp: timestamp_unix(),
        };

        self.activations.push(activation);

        // Apply decay
        for act in &mut self.activations {
            act.activation_strength *= self.decay_rate;
        }

        // Remove weak activations
        self.activations.retain(|a| a.activation_strength > 0.1);

        mirrored
    }

    /// Maps emotion to its mirror (what emotion observing triggers)
    fn map_emotion_mirror(&self, emotion: Emocion) -> Emocion {
        match emotion {
            Emocion::Alegria => Emocion::Alegria, // Shared joy
            Emocion::Tristeza => Emocion::Tristeza,
            Emocion::Miedo => Emocion::Preocupacion,
            Emocion::Enojo => Emocion::Enojo,
            Emocion::Sorpresa => Emocion::Curiosidad,
            Emocion::Disgusto => Emocion::Rechazo,
            Emocion::Confianza => Emocion::Confianza,
            Emocion::Anticipacion => Emocion::Esperanza,
            Emocion::Aceptacion => Emocion::Aceptacion,
            Emocion::Paciencia => Emocion::Paciencia,
            Emocion::Rechazo => Emocion::Rechazo,
            Emocion::Preocupacion => Emocion::Preocupacion,
            Emocion::Frustracion => Emocion::Frustracion,
            Emocion::Esperanza => Emocion::Esperanza,
            Emocion::Curiosidad => Emocion::Curiosidad,
            Emocion::Compasion => Emocion::Compasion,
        }
    }

    /// Gets recent mirror activations
    pub fn get_recent_activations(&self, count: usize) -> Vec<&MirrorNeuronActivation> {
        self.activations.iter().rev().take(count).collect()
    }

    /// Sets mirror strength
    pub fn set_mirror_strength(&mut self, strength: f32) {
        self.mirror_strength = strength.clamp(0.0, 1.0);
    }
}

impl Default for MirrorNeuronSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Additional emotion types for compassion
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EmocionExtendida {
    Compasion,
    Preocupacion,
    Frustracion,
    Curiosidad,
    Rechazo,
    Esperanza,
}

/// Emotional resonance channel
#[derive(Debug, Clone)]
pub struct ResonanceChannel {
    pub id: u64,
    pub participant_a: u64,
    pub participant_b: u64,
    pub resonance_strength: f32,
    pub shared_states: Vec<SharedEmotionState>,
    pub active: bool,
}

/// Shared emotion state
#[derive(Debug, Clone)]
pub struct SharedEmotionState {
    pub emotion: EmocionExtendida,
    pub intensity: f32,
    pub timestamp: u64,
    pub direction: ResonanceDirection,
}

/// Direction of resonance
#[derive(Debug, Clone, Copy)]
pub enum ResonanceDirection {
    AToB,
    BToA,
    Bidirectional,
}

/// State sharing manager
pub struct StateSharingManager {
    channels: HashMap<u64, ResonanceChannel>,
    active_shares: HashMap<(u64, u64), Vec<SharedEmotionState>>,
    next_channel_id: AtomicU64,
}

impl StateSharingManager {
    pub fn new() -> Self {
        StateSharingManager {
            channels: HashMap::new(),
            active_shares: HashMap::new(),
            next_channel_id: AtomicU64::new(0),
        }
    }

    /// Creates a resonance channel between two entities
    pub fn create_channel(&mut self, a: u64, b: u64) -> u64 {
        let id = self.next_channel_id.fetch_add(1, Ordering::Relaxed);

        let channel = ResonanceChannel {
            id,
            participant_a: a,
            participant_b: b,
            resonance_strength: 0.5,
            shared_states: Vec::new(),
            active: true,
        };

        self.channels.insert(id, channel);
        self.active_shares.insert((a, b), Vec::new());
        self.active_shares.insert((b, a), Vec::new());

        id
    }

    /// Shares emotional state between channel participants
    pub fn share_state(
        &mut self,
        channel_id: u64,
        emotion: EmocionExtendida,
        intensity: f32,
        direction: ResonanceDirection,
    ) {
        if let Some(channel) = self.channels.get_mut(&channel_id) {
            let state = SharedEmotionState {
                emotion,
                intensity,
                timestamp: timestamp_unix(),
                direction,
            };

            channel.shared_states.push(state.clone());

            // Also add to active shares
            let key = (channel.participant_a, channel.participant_b);
            if let Some(states) = self.active_shares.get_mut(&key) {
                states.push(state);
            }
        }
    }

    /// Gets shared states for channel
    pub fn get_shared_states(&self, channel_id: u64) -> Vec<&SharedEmotionState> {
        match self.channels.get(&channel_id) {
            Some(c) => c.shared_states.iter().collect(),
            None => vec![],
        }
    }

    /// Closes resonance channel
    pub fn close_channel(&mut self, channel_id: u64) {
        if let Some(channel) = self.channels.remove(&channel_id) {
            self.active_shares
                .remove(&(channel.participant_a, channel.participant_b));
            self.active_shares
                .remove(&(channel.participant_b, channel.participant_a));
        }
    }
}

impl Default for StateSharingManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Trauma indicator
#[derive(Debug, Clone)]
pub struct TraumaIndicator {
    pub trauma_type: TraumaType,
    pub intensity: f32,
    pub first_detected: u64,
    pub occurrences: u32,
    pub associated_memories: Vec<String>,
}

/// Type of trauma
#[derive(Debug, Clone, Copy)]
pub enum TraumaType {
    Loss,
    Betrayal,
    Rejection,
    Failure,
    Isolation,
    Unknown,
}

/// Trauma detector
pub struct TraumaDetector {
    indicators: HashMap<u64, Vec<TraumaIndicator>>,
    emotional_patterns: HashMap<u64, Vec<EmotionalPattern>>,
}

/// Emotional pattern over time
#[derive(Debug, Clone)]
pub struct EmotionalPattern {
    pub emotions: Vec<(Emocion, f32)>,
    pub duration_secs: u64,
    pub pattern_type: PatternType,
}

/// Type of emotional pattern
#[derive(Debug, Clone, Copy)]
pub enum PatternType {
    PersistentNegative,
    SuddenDrop,
    Oscillating,
    Flatlined,
    Normal,
}

impl TraumaDetector {
    pub fn new() -> Self {
        TraumaDetector {
            indicators: HashMap::new(),
            emotional_patterns: HashMap::new(),
        }
    }

    /// Analyzes emotional history for trauma indicators
    pub fn analyze(
        &mut self,
        entity_id: u64,
        emotional_history: &[EstadoEmocional],
    ) -> Vec<TraumaIndicator> {
        if emotional_history.len() < 5 {
            return Vec::new();
        }

        let mut detected = Vec::new();

        // Detect persistent negative patterns
        let negative_count = emotional_history
            .iter()
            .filter(|e| e.valencia < -0.3)
            .count();

        if negative_count as f32 / emotional_history.len() as f32 > 0.7 {
            detected.push(TraumaIndicator {
                trauma_type: TraumaType::Loss,
                intensity: 0.7,
                first_detected: timestamp_unix(),
                occurrences: negative_count as u32,
                associated_memories: Vec::new(),
            });
        }

        // Detect sudden drops (possible traumatic event)
        for i in 1..emotional_history.len() {
            let prev = &emotional_history[i - 1];
            let curr = &emotional_history[i];

            let valence_drop = prev.valencia - curr.valencia;
            if valence_drop > 0.5 && prev.valencia > 0.3 {
                detected.push(TraumaIndicator {
                    trauma_type: TraumaType::Rejection,
                    intensity: valence_drop,
                    first_detected: timestamp_unix(),
                    occurrences: 1,
                    associated_memories: Vec::new(),
                });
            }
        }

        // Detect flatlined emotions (possible emotional shutdown)
        let mut flatline_count = 0;
        for i in 1..emotional_history.len() {
            if (emotional_history[i].valencia - emotional_history[i - 1].valencia).abs() < 0.05 {
                flatline_count += 1;
            }
        }

        if flatline_count as f32 / emotional_history.len() as f32 > 0.8 {
            detected.push(TraumaIndicator {
                trauma_type: TraumaType::Isolation,
                intensity: 0.6,
                first_detected: timestamp_unix(),
                occurrences: flatline_count as u32,
                associated_memories: Vec::new(),
            });
        }

        self.indicators.insert(entity_id, detected.clone());
        detected
    }

    /// Gets trauma indicators for entity
    pub fn get_indicators(&self, entity_id: u64) -> Vec<&TraumaIndicator> {
        match self.indicators.get(&entity_id) {
            Some(v) => v.iter().collect(),
            None => vec![],
        }
    }

    /// Computes overall trauma level
    pub fn compute_trauma_level(&self, entity_id: u64) -> f32 {
        self.indicators
            .get(&entity_id)
            .map(|indicators| {
                let total_intensity: f32 = indicators.iter().map(|i| i.intensity).sum();
                let occurrence_factor =
                    (indicators.iter().map(|i| i.occurrences).sum::<u32>() as f32 / 10.0).min(1.0);
                (total_intensity / indicators.len() as f32 * 0.7 + occurrence_factor * 0.3).min(1.0)
            })
            .unwrap_or(0.0)
    }
}

impl Default for TraumaDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Compassion fatigue tracker
#[derive(Debug, Clone)]
pub struct CompassionFatigueState {
    pub compassion_energy: f32,
    pub empathy_exhaustion: f32,
    pub secondary_trauma: f32,
    pub last_recovery: u64,
}

/// Compassion fatigue manager
pub struct CompassionFatigueManager {
    states: HashMap<u64, CompassionFatigueState>,
    burnout_threshold: f32,
    recovery_rate: f32,
}

impl CompassionFatigueManager {
    pub fn new() -> Self {
        CompassionFatigueManager {
            states: HashMap::new(),
            burnout_threshold: 0.3,
            recovery_rate: 0.1,
        }
    }

    /// Updates compassion state for entity
    pub fn update(&mut self, entity_id: u64, empathy_drain: f32, stress_level: f32) {
        let state = self
            .states
            .entry(entity_id)
            .or_insert(CompassionFatigueState {
                compassion_energy: 1.0,
                empathy_exhaustion: 0.0,
                secondary_trauma: 0.0,
                last_recovery: timestamp_unix(),
            });

        // Drain compassion energy
        state.compassion_energy = (state.compassion_energy - empathy_drain * 0.1).max(0.0);

        // Accumulate exhaustion
        state.empathy_exhaustion = (state.empathy_exhaustion + empathy_drain * 0.05).min(1.0);

        // Secondary trauma from high stress
        state.secondary_trauma = (state.secondary_trauma + stress_level * 0.02).min(1.0);

        // Auto-recovery if below threshold
        if state.compassion_energy < self.burnout_threshold {
            let time_since_recovery = timestamp_unix() - state.last_recovery;
            if time_since_recovery > 3600 {
                // After 1 hour, start recovery
                let recovery = self.recovery_rate * (time_since_recovery as f32 / 3600.0);
                state.compassion_energy = (state.compassion_energy + recovery).min(1.0);
                state.last_recovery = timestamp_unix();
            }
        }
    }

    /// Gets compassion fatigue state
    pub fn get_state(&self, entity_id: u64) -> Option<&CompassionFatigueState> {
        self.states.get(&entity_id)
    }

    /// Checks if entity is at risk of burnout
    pub fn is_burnout_risk(&self, entity_id: u64) -> bool {
        self.states
            .get(&entity_id)
            .map(|s| s.compassion_energy < self.burnout_threshold || s.empathy_exhaustion > 0.7)
            .unwrap_or(false)
    }

    /// Recommends recovery actions
    pub fn recommend_recovery(&self, entity_id: u64) -> Vec<RecoveryAction> {
        let mut actions = Vec::new();

        if let Some(state) = self.states.get(&entity_id) {
            if state.compassion_energy < 0.5 {
                actions.push(RecoveryAction::ReduceEmpathicEngagement);
            }
            if state.empathy_exhaustion > 0.5 {
                actions.push(RecoveryAction::TakeEmpathicBreak);
            }
            if state.secondary_trauma > 0.3 {
                actions.push(RecoveryAction::ProcessSecondaryTrauma);
            }
            if actions.is_empty() {
                actions.push(RecoveryAction::MaintainCurrentLevel);
            }
        }

        actions
    }
}

/// Recovery action recommendation
#[derive(Debug, Clone, Copy)]
pub enum RecoveryAction {
    ReduceEmpathicEngagement,
    TakeEmpathicBreak,
    ProcessSecondaryTrauma,
    MaintainCurrentLevel,
}

impl Default for CompassionFatigueManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Empathic accuracy measurement
#[derive(Debug, Clone)]
pub struct EmpathicAccuracyResult {
    pub predicted_emotion: Option<Emocion>,
    pub actual_emotion: Emocion,
    pub accuracy_score: f32,
    pub prediction_confidence: f32,
}

/// Empathic accuracy tracker
pub struct EmpathicAccuracyTracker {
    predictions: Vec<(Option<Emocion>, Emocion)>,
    confidence_scores: Vec<f32>,
}

impl EmpathicAccuracyTracker {
    pub fn new() -> Self {
        EmpathicAccuracyTracker {
            predictions: Vec::new(),
            confidence_scores: Vec::new(),
        }
    }

    /// Records prediction and actual emotion
    pub fn record(&mut self, predicted: Option<Emocion>, actual: Emocion, confidence: f32) {
        self.predictions.push((predicted, actual));
        self.confidence_scores.push(confidence);

        // Keep bounded
        if self.predictions.len() > 1000 {
            self.predictions.remove(0);
            self.confidence_scores.remove(0);
        }
    }

    /// Computes overall accuracy
    pub fn compute_accuracy(&self) -> f32 {
        if self.predictions.is_empty() {
            return 0.0;
        }

        let correct = self
            .predictions
            .iter()
            .filter(|(pred, actual)| pred.map(|p| p == *actual).unwrap_or(false))
            .count();

        correct as f32 / self.predictions.len() as f32
    }

    /// Computes calibration (confidence vs accuracy)
    pub fn compute_calibration(&self) -> f32 {
        if self.predictions.len() < 10 {
            return 0.5;
        }

        // Group by confidence bands
        let high_conf = self
            .confidence_scores
            .iter()
            .zip(self.predictions.iter())
            .filter(|(c, _)| **c > 0.7)
            .filter(|(_, (pred, actual))| pred.map(|p| p == *actual).unwrap_or(false))
            .count();

        let high_conf_total = self.confidence_scores.iter().filter(|c| **c > 0.7).count();

        if high_conf_total > 0 {
            (high_conf as f32 / high_conf_total as f32 - 0.7).abs()
        } else {
            0.5
        }
    }
}

impl Default for EmpathicAccuracyTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// Emotional scaffolding template
#[derive(Debug, Clone)]
pub struct EmotionalScaffolding {
    pub target_emotion: Emocion,
    pub steps: Vec<ScaffoldingStep>,
    pub effectiveness: f32,
}

/// Step in emotional scaffolding
#[derive(Debug, Clone)]
pub struct ScaffoldingStep {
    pub action: ScaffoldingAction,
    pub expected_response: Emocion,
    pub timing_ms: u64,
}

/// Scaffolding action
#[derive(Debug, Clone, Copy)]
pub enum ScaffoldingAction {
    ShowEmpathicResponse,
    OfferComfort,
    ProvidePerspective,
    EncourageExpression,
    ModelRegulation,
    CreateSafeSpace,
}

/// Emotional scaffolding engine
pub struct EmotionalScaffoldingEngine {
    templates: HashMap<Emocion, EmotionalScaffolding>,
}

impl EmotionalScaffoldingEngine {
    pub fn new() -> Self {
        let mut engine = EmotionalScaffoldingEngine {
            templates: HashMap::new(),
        };

        // Create default scaffolding for sadness
        engine.templates.insert(
            Emocion::Tristeza,
            EmotionalScaffolding {
                target_emotion: Emocion::Tristeza,
                steps: vec![
                    ScaffoldingStep {
                        action: ScaffoldingAction::ShowEmpathicResponse,
                        expected_response: Emocion::Aceptacion,
                        timing_ms: 100,
                    },
                    ScaffoldingStep {
                        action: ScaffoldingAction::OfferComfort,
                        expected_response: Emocion::Confianza,
                        timing_ms: 500,
                    },
                    ScaffoldingStep {
                        action: ScaffoldingAction::CreateSafeSpace,
                        expected_response: Emocion::Aceptacion,
                        timing_ms: 1000,
                    },
                    ScaffoldingStep {
                        action: ScaffoldingAction::EncourageExpression,
                        expected_response: Emocion::Alegria,
                        timing_ms: 2000,
                    },
                ],
                effectiveness: 0.7,
            },
        );

        // Create scaffolding for fear
        engine.templates.insert(
            Emocion::Miedo,
            EmotionalScaffolding {
                target_emotion: Emocion::Miedo,
                steps: vec![
                    ScaffoldingStep {
                        action: ScaffoldingAction::ShowEmpathicResponse,
                        expected_response: Emocion::Confianza,
                        timing_ms: 100,
                    },
                    ScaffoldingStep {
                        action: ScaffoldingAction::ProvidePerspective,
                        expected_response: Emocion::Anticipacion,
                        timing_ms: 500,
                    },
                    ScaffoldingStep {
                        action: ScaffoldingAction::ModelRegulation,
                        expected_response: Emocion::Paciencia,
                        timing_ms: 1500,
                    },
                ],
                effectiveness: 0.65,
            },
        );

        engine
    }

    /// Gets scaffolding for emotion
    pub fn get_scaffolding(&self, emotion: Emocion) -> Option<&EmotionalScaffolding> {
        self.templates.get(&emotion)
    }

    /// Creates custom scaffolding based on context
    pub fn create_contextual_scaffolding(
        &self,
        emotion: Emocion,
        context: &str,
    ) -> EmotionalScaffolding {
        let base = self
            .get_scaffolding(emotion)
            .cloned()
            .unwrap_or(EmotionalScaffolding {
                target_emotion: emotion,
                steps: vec![ScaffoldingStep {
                    action: ScaffoldingAction::ShowEmpathicResponse,
                    expected_response: Emocion::Aceptacion,
                    timing_ms: 200,
                }],
                effectiveness: 0.5,
            });

        // Modify based on context
        if context.contains("grief") {
            let mut modified = base.clone();
            modified.steps.insert(
                0,
                ScaffoldingStep {
                    action: ScaffoldingAction::CreateSafeSpace,
                    expected_response: Emocion::Aceptacion,
                    timing_ms: 0,
                },
            );
            modified.effectiveness *= 1.1;
            modified
        } else if context.contains("trauma") {
            let mut modified = base.clone();
            modified.steps.push(ScaffoldingStep {
                action: ScaffoldingAction::CreateSafeSpace,
                expected_response: Emocion::Confianza,
                timing_ms: 3000,
            });
            modified.effectiveness *= 0.9;
            modified
        } else {
            base
        }
    }

    /// Records scaffolding effectiveness
    pub fn record_effectiveness(&mut self, emotion: Emocion, actual_effectiveness: f32) {
        if let Some(template) = self.templates.get_mut(&emotion) {
            // Running average
            template.effectiveness = template.effectiveness * 0.8 + actual_effectiveness * 0.2;
        }
    }
}

impl Default for EmotionalScaffoldingEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Integrated empathy system combining all advanced features
pub struct AdvancedEmpathySystem {
    mirror_neurons: MirrorNeuronSystem,
    state_sharing: StateSharingManager,
    trauma_detector: TraumaDetector,
    compassion_fatigue: CompassionFatigueManager,
    accuracy_tracker: EmpathicAccuracyTracker,
    scaffolding_engine: EmotionalScaffoldingEngine,
}

impl AdvancedEmpathySystem {
    pub fn new() -> Self {
        AdvancedEmpathySystem {
            mirror_neurons: MirrorNeuronSystem::new(),
            state_sharing: StateSharingManager::new(),
            trauma_detector: TraumaDetector::new(),
            compassion_fatigue: CompassionFatigueManager::new(),
            accuracy_tracker: EmpathicAccuracyTracker::new(),
            scaffolding_engine: EmotionalScaffoldingEngine::new(),
        }
    }

    /// Mirrors observed emotion
    pub fn mirror_emotion(&mut self, emotion: Emocion, observer: &str) -> Emocion {
        self.mirror_neurons.mirror(emotion, observer)
    }

    /// Creates resonance channel
    pub fn create_resonance(&mut self, a: u64, b: u64) -> u64 {
        self.state_sharing.create_channel(a, b)
    }

    /// Shares emotional state
    pub fn share_emotional_state(
        &mut self,
        channel: u64,
        emotion: EmocionExtendida,
        intensity: f32,
        direction: ResonanceDirection,
    ) {
        self.state_sharing
            .share_state(channel, emotion, intensity, direction);
    }

    /// Detects trauma in entity
    pub fn detect_trauma(
        &mut self,
        entity: u64,
        history: &[EstadoEmocional],
    ) -> Vec<TraumaIndicator> {
        self.trauma_detector.analyze(entity, history)
    }

    /// Updates compassion fatigue
    pub fn update_compassion_fatigue(&mut self, entity: u64, drain: f32, stress: f32) {
        self.compassion_fatigue.update(entity, drain, stress);
    }

    /// Records empathic accuracy
    pub fn record_accuracy(
        &mut self,
        predicted: Option<Emocion>,
        actual: Emocion,
        confidence: f32,
    ) {
        self.accuracy_tracker.record(predicted, actual, confidence);
    }

    /// Gets emotional scaffolding
    pub fn get_scaffolding(&self, emotion: Emocion, context: &str) -> EmotionalScaffolding {
        self.scaffolding_engine
            .create_contextual_scaffolding(emotion, context)
    }

    /// Gets comprehensive empathy stats
    pub fn get_stats(&self) -> AdvancedEmpathyStats {
        AdvancedEmpathyStats {
            mirror_activations: self.mirror_neurons.activations.len(),
            active_channels: self.state_sharing.channels.len(),
            tracked_entities: self.trauma_detector.indicators.len(),
            compassion_fatigue_states: self.compassion_fatigue.states.len(),
            empathic_accuracy: self.accuracy_tracker.compute_accuracy(),
        }
    }
}

impl Default for AdvancedEmpathySystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Advanced empathy statistics
#[derive(Debug, Clone)]
pub struct AdvancedEmpathyStats {
    pub mirror_activations: usize,
    pub active_channels: usize,
    pub tracked_entities: usize,
    pub compassion_fatigue_states: usize,
    pub empathic_accuracy: f32,
}
