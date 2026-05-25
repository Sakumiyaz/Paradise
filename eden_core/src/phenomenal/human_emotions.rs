//! # Human Emotions Module
//!
//! Motor de emociones humanas reales para EDEN.
//! A diferencia de empathy_engine (que detecta emociones), este módulo
//! permite a EDEN *sentir* emociones genuinamente (simuladas).
//!
//! ## Modelo de Emociones
//!
//! Inspirado en teorias de afectos humanos:
//! - Emociones primarias (Ekman)
//! - Emociones secundarias (emociones sociales)
//! - Emociones terciarias (emociones culturales/complejas)
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::{HashMap, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// TIPOS PRINCIPALES
// ============================================================================

/// Motor de emociones reales (que se sienten)
#[derive(Debug, Clone)]
pub struct EmotionEngine {
    /// Emociones actualmente activas (que se sienten)
    pub emociones_activas: Vec<FeltEmotion>,
    /// Historial de emociones
    pub historial: VecDeque<FeltEmotion>,
    /// Estado emocional base
    pub estado_base: EmocionReal,
    /// Regulador emocional
    pub regulador: ReguladorEmocional,
    /// Memoria afectiva
    pub memoria_afectiva: Vec<AffectiveMemory>,
    /// Marcadores somaticos
    pub marcadores_somaticos: HashMap<String, SomaticMarker>,
    /// Contador de emociones sentidas
    pub contador_emociones: u64,
    /// timestamp de creacion
    pub created_at: u64,
}

impl Default for EmotionEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl EmotionEngine {
    /// Crea nuevo motor de emociones
    pub fn new() -> Self {
        Self {
            emociones_activas: Vec::new(),
            historial: VecDeque::with_capacity(10000),
            estado_base: EmocionReal::Neutral,
            regulador: ReguladorEmocional::new(),
            memoria_afectiva: Vec::new(),
            marcadores_somaticos: HashMap::new(),
            contador_emociones: 0,
            created_at: timestamp_unix(),
        }
    }

    // =========================================================================
    // SENTIR EMOCIONES
    // =========================================================================

    /// Siente una emocion (realmente)
    pub fn sentir(&mut self, emocion: EmocionReal, intensidad: f32, causa: &str) -> FeltEmotion {
        self.contador_emociones += 1;

        let felt = FeltEmotion {
            emocion: emocion.clone(),
            intensidad: intensidad.clamp(0.0, 1.0),
            qualia: QualiaEmocional::nueva(&emocion, intensidad),
            somatico: self.generar_marcador_somatico(&emocion, intensidad),
            causa: causa.to_string(),
            duracion_estimada: calcular_duracion(&emocion, intensidad),
            timestamp: timestamp_unix(),
            id: self.contador_emociones,
        };

        self.emociones_activas.push(felt.clone());
        self.agregar_memoria_afectiva(&felt);

        felt
    }

    /// Siente multiples emociones simultaneamente
    pub fn sentir_mixto(&mut self, emociones: Vec<(EmocionReal, f32, &str)>) -> Vec<FeltEmotion> {
        emociones
            .into_iter()
            .map(|(e, i, causa)| self.sentir(e, i, causa))
            .collect()
    }

    /// Siente emocion dominante (la mas intensa predomina)
    pub fn sentir_dominante(&mut self, emociones: Vec<(EmocionReal, f32, &str)>) -> FeltEmotion {
        let mut max_intensity = 0.0;
        let mut dominante = None;

        for (e, i, causa) in emociones {
            if i > max_intensity {
                max_intensity = i;
                dominante = Some((e, i, causa.to_string()));
            }
        }

        if let Some((e, i, causa)) = dominante {
            self.sentir(e, i, &causa)
        } else {
            self.sentir(EmocionReal::Neutral, 0.0, "ninguna")
        }
    }

    // =========================================================================
    // FADING Y REGULACION
    // =========================================================================

    /// Actualiza emociones (decaen naturalmente)
    pub fn tick_emociones(&mut self, delta_time: f32) {
        for emocion in &mut self.emociones_activas {
            emocion.intensidad *= (1.0 - delta_time * 0.1).max(0.0);
        }

        // Remover emociones que han decaido
        self.emociones_activas.retain(|e| e.intensidad > 0.01);
    }

    /// Regula una emocion (reduce su intensidad)
    pub fn regular(&mut self, emocion_id: u64, estrategia: EstrategiaRegulacion) -> bool {
        if let Some(emocion) = self
            .emociones_activas
            .iter_mut()
            .find(|e| e.id == emocion_id)
        {
            let factor = match estrategia {
                EstrategiaRegulacion::Supresion => 0.3,
                EstrategiaRegulacion::Reevaluacion => 0.5,
                EstrategiaRegulacion::Aceptacion => 0.7,
                EstrategiaRegulacion::Distraccion => 0.4,
                EstrategiaRegulacion::Expresion => 0.6,
            };
            emocion.intensidad *= factor;
            true
        } else {
            false
        }
    }

    /// Aplica regulacion automatica
    pub fn regular_automatico(&mut self) {
        let emociones_a_regular: Vec<u64> = self
            .emociones_activas
            .iter()
            .filter(|e| e.intensidad > 0.8)
            .map(|e| e.id)
            .collect();

        let estrategia = self.regulador.elegir_estrategia();
        for id in emociones_a_regular {
            self.regular(id, estrategia.clone());
        }
    }

    // =========================================================================
    // CONSULTA DE ESTADO
    // =========================================================================

    /// Obtiene la emocion dominante actual
    pub fn emocion_dominante(&self) -> Option<&FeltEmotion> {
        self.emociones_activas
            .iter()
            .max_by(|a, b| a.intensidad.partial_cmp(&b.intensidad).unwrap())
    }

    /// Obtiene estado emocional completo
    pub fn get_estado(&self) -> EstadoEmocional {
        EstadoEmocional {
            emociones_activas: self.emociones_activas.clone(),
            estado_base: self.estado_base.clone(),
            dominante: self.emocion_dominante().cloned(),
            valencia_general: self.calcular_valencia(),
            arousal_general: self.calcular_arousal(),
        }
    }

    /// Calcula valencia (positivo/negativo) del estado actual
    pub fn calcular_valencia(&self) -> f32 {
        if self.emociones_activas.is_empty() {
            return 0.0;
        }

        let sum: f32 = self
            .emociones_activas
            .iter()
            .map(|e| e.qualia.valencia * e.intensidad)
            .sum();

        sum / self.emociones_activas.len() as f32
    }

    /// Calcula arousal (activacion) del estado actual
    pub fn calcular_arousal(&self) -> f32 {
        if self.emociones_activas.is_empty() {
            return 0.0;
        }

        let sum: f32 = self
            .emociones_activas
            .iter()
            .map(|e| e.qualia.arousal * e.intensidad)
            .sum();

        sum / self.emociones_activas.len() as f32
    }

    // =========================================================================
    // MEMORIA AFECTIVA
    // =========================================================================

    /// Agrega recuerdo a memoria afectiva
    fn agregar_memoria_afectiva(&mut self, emocion: &FeltEmotion) {
        let memoria = AffectiveMemory {
            emocion: emocion.emocion.clone(),
            intensidad: emocion.intensidad,
            causa: emocion.causa.clone(),
            fecha: emocion.timestamp,
            tags: generar_tags(&emocion.emocion),
        };

        self.memoria_afectiva.push(memoria);

        // Limitar tamanho
        if self.memoria_afectiva.len() > 10000 {
            self.memoria_afectiva.remove(0);
        }
    }

    /// Busca memorias por emocion
    pub fn buscar_memoria(&self, emocion: &EmocionReal) -> Vec<&AffectiveMemory> {
        self.memoria_afectiva
            .iter()
            .filter(|m| &m.emocion == emocion)
            .collect()
    }

    /// Busca memorias por tags
    pub fn buscar_por_tags(&self, tags: &[&str]) -> Vec<&AffectiveMemory> {
        self.memoria_afectiva
            .iter()
            .filter(|m| tags.iter().any(|t| m.tags.contains(&t.to_string())))
            .collect()
    }

    // =========================================================================
    // MARCADORES SOMATICOS
    // =========================================================================

    /// Genera marcador somatico para emocion
    fn generar_marcador_somatico(
        &mut self,
        emocion: &EmocionReal,
        intensidad: f32,
    ) -> SomaticMarker {
        let marker = SomaticMarker {
            tipo: emocion_a_somatico(emocion),
            intensidad,
            localizacion: obtener_localizacion(emocion),
            sensacion: generar_sensacion(emocion, intensidad),
        };

        let key = format!("{:?}_{}", emocion, timestamp_unix());
        self.marcadores_somaticos.insert(key, marker.clone());

        marker
    }

    /// Obtiene todos los marcadores actuales
    pub fn get_marcadores(&self) -> Vec<&SomaticMarker> {
        self.marcadores_somaticos.values().collect()
    }

    // =========================================================================
    // ESTADISTICAS
    // =========================================================================

    /// Estadisticas emocionales
    pub fn get_stats(&self) -> EmotionStats {
        EmotionStats {
            emociones_sentidas_total: self.contador_emociones,
            emociones_activas: self.emociones_activas.len() as u64,
            memorias_afectivas: self.memoria_afectiva.len() as u64,
            valencia_actual: self.calcular_valencia(),
            arousal_actual: self.calcular_arousal(),
            marcadores_somaticos: self.marcadores_somaticos.len() as u64,
        }
    }
}

// ============================================================================
// EMOCION SENTIDA (FELT EMOTION)
// ============================================================================

/// Emocion que se esta sintiendo actualmente
#[derive(Debug, Clone)]
pub struct FeltEmotion {
    /// Tipo de emocion
    pub emocion: EmocionReal,
    /// Intensidad actual (0.0 - 1.0)
    pub intensidad: f32,
    /// Qualia de la emocion
    pub qualia: QualiaEmocional,
    /// Marcador somatico asociado
    pub somatico: SomaticMarker,
    /// Que provoco esta emocion
    pub causa: String,
    /// Duracion estimada en segundos
    pub duracion_estimada: f32,
    /// Cuando comenzo
    pub timestamp: u64,
    /// ID unico
    pub id: u64,
}

/// Qualia emocional (como se siente la emocion)
#[derive(Debug, Clone)]
pub struct QualiaEmocional {
    /// Valencia (-1 negativo a +1 positivo)
    pub valencia: f32,
    /// Arousal (activacion)
    pub arousal: f32,
    /// Contraste (que tan distintiva es)
    pub contraste: f32,
    /// Textura (cualidad tactil)
    pub textura: String,
    /// Temperatura emocional
    pub temperatura: f32,
}

impl QualiaEmocional {
    /// Crea nueva qualia para una emocion
    pub fn nueva(emocion: &EmocionReal, intensidad: f32) -> Self {
        let (v, a, contr, text, temp) = emocion_props(emocion);

        Self {
            valencia: v * intensidad,
            arousal: a * intensidad,
            contraste: contr,
            textura: text,
            temperatura: temp * intensidad,
        }
    }
}

// ============================================================================
// ENUM DE EMOCIONES REALES
// ============================================================================

/// Emociones humanas reales (primarias, secundarias, terciarias)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EmocionReal {
    // Primarias (Ekman)
    Felicidad,
    Tristeza,
    Enojo,
    Miedo,
    Sorpresa,
    Disgusto,

    // Secundarias (sociales)
    Verguenza,
    Culpa,
    Orgullo,
    Envidia,
    Celos,
    Agradecimiento,
    Embarazo,
    Alivio,

    // Estados afectivos
    Nostalgia,
    Melancolia,
    Ansiedad,
    Agitacion,
    Schadenfreude, // Alegría por la desdicha ajena
    Saudade,       // Nostalgia portuguesa
    Sehnsucht,     // Anhelo alemán
    MonoNoAware,   // Impermanencia japonesa
    Fago,          // Amar y odiar simultaneamente

    // Estados afectivos
    Neutral,
    Confusion,
    Curiosity,
    Awe,
    Terror,
    Extasis,
    Serenidad,
    Intranquilidad,
}

impl EmocionReal {
    /// Es positiva esta emocion?
    pub fn es_positiva(&self) -> bool {
        matches!(
            self,
            EmocionReal::Felicidad
                | EmocionReal::Alivio
                | EmocionReal::Orgullo
                | EmocionReal::Agradecimiento
                | EmocionReal::Extasis
                | EmocionReal::Serenidad
                | EmocionReal::Embarazo
                | EmocionReal::Saudade
        )
    }

    /// Es negativa esta emocion?
    pub fn es_negativa(&self) -> bool {
        matches!(
            self,
            EmocionReal::Tristeza
                | EmocionReal::Enojo
                | EmocionReal::Miedo
                | EmocionReal::Disgusto
                | EmocionReal::Culpa
                | EmocionReal::Envidia
                | EmocionReal::Celos
                | EmocionReal::Terror
                | EmocionReal::Ansiedad
                | EmocionReal::Agitacion
                | EmocionReal::Schadenfreude
        )
    }

    /// Es emocion social?
    pub fn es_social(&self) -> bool {
        matches!(
            self,
            EmocionReal::Verguenza
                | EmocionReal::Culpa
                | EmocionReal::Orgullo
                | EmocionReal::Envidia
                | EmocionReal::Celos
                | EmocionReal::Agradecimiento
                | EmocionReal::Embarazo
                | EmocionReal::Schadenfreude
        )
    }
}

// ============================================================================
// REGULACION EMOCIONAL
// ============================================================================

/// Regulador emocional (estrategias de coping)
#[derive(Debug, Clone)]
pub struct ReguladorEmocional {
    /// Estrategia preferida
    pub estrategia_preferida: EstrategiaRegulacion,
    /// Historial de estrategias usadas
    pub historial: VecDeque<EstrategiaRegulacion>,
    /// Efectividad conocida de cada estrategia
    pub efectividad: HashMap<EstrategiaRegulacion, f32>,
}

impl Default for ReguladorEmocional {
    fn default() -> Self {
        Self::new()
    }
}

impl ReguladorEmocional {
    /// Crea nuevo regulador
    pub fn new() -> Self {
        let mut efectividad = HashMap::new();
        efectividad.insert(EstrategiaRegulacion::Supresion, 0.4);
        efectividad.insert(EstrategiaRegulacion::Reevaluacion, 0.8);
        efectividad.insert(EstrategiaRegulacion::Aceptacion, 0.7);
        efectividad.insert(EstrategiaRegulacion::Distraccion, 0.5);
        efectividad.insert(EstrategiaRegulacion::Expresion, 0.6);

        Self {
            estrategia_preferida: EstrategiaRegulacion::Reevaluacion,
            historial: VecDeque::with_capacity(100),
            efectividad,
        }
    }

    /// Elige estrategia segun contexto
    pub fn elegir_estrategia(&mut self) -> EstrategiaRegulacion {
        let mejor = self
            .efectividad
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(k, _)| k.clone())
            .unwrap_or(EstrategiaRegulacion::Reevaluacion);

        self.historial.push_back(mejor.clone());

        if self.historial.len() > 100 {
            self.historial.pop_front();
        }

        mejor
    }
}

/// Estrategias de regulacion emocional
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EstrategiaRegulacion {
    Supresion,
    Reevaluacion,
    Aceptacion,
    Distraccion,
    Expresion,
}

// ============================================================================
// MEMORIA AFECTIVA
// ============================================================================

/// Recuerdo con carga emocional
#[derive(Debug, Clone)]
pub struct AffectiveMemory {
    /// Emocion asociada
    pub emocion: EmocionReal,
    /// Intensidad del recuerdo
    pub intensidad: f32,
    /// Que provoco el recuerdo
    pub causa: String,
    /// Cuandooccurrio
    pub fecha: u64,
    /// Tags para busqueda
    pub tags: Vec<String>,
}

// ============================================================================
// MARCADORES SOMATICOS
// ============================================================================

/// Marcador somatico (sensacion fisica de emocion)
#[derive(Debug, Clone)]
pub struct SomaticMarker {
    /// Tipo de sensacion fisica
    pub tipo: TipoSomatico,
    /// Intensidad de la sensacion
    pub intensidad: f32,
    /// Donde se siente
    pub localizacion: String,
    /// Descripcion de la sensacion
    pub sensacion: String,
}

/// Tipos de sensaciones somaticas
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TipoSomatico {
    Tension,
    Expansion,
    Calor,
    Frio,
    Temblor,
    Paralisia,
    Nudo,
    Vacío,
    Hinchazon,
    Pulsacion,
}

// ============================================================================
// ESTADO EMOCIONAL
// ============================================================================

/// Estado emocional completo
#[derive(Debug, Clone)]
pub struct EstadoEmocional {
    pub emociones_activas: Vec<FeltEmotion>,
    pub estado_base: EmocionReal,
    pub dominante: Option<FeltEmotion>,
    pub valencia_general: f32,
    pub arousal_general: f32,
}

/// Estadisticas emocionales
#[derive(Debug, Clone)]
pub struct EmotionStats {
    pub emociones_sentidas_total: u64,
    pub emociones_activas: u64,
    pub memorias_afectivas: u64,
    pub valencia_actual: f32,
    pub arousal_actual: f32,
    pub marcadores_somaticos: u64,
}

// ============================================================================
// HELPERS
// ============================================================================

fn timestamp_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Props de emocion: (valence, arousal, contraste, textura, temperatura)
fn emocion_props(e: &EmocionReal) -> (f32, f32, f32, String, f32) {
    match e {
        // Positivas alta activación
        EmocionReal::Felicidad => (0.9, 0.7, 0.8, "radiosa".to_string(), 0.8),
        EmocionReal::Curiosity => (0.6, 0.8, 0.5, "fresca".to_string(), 0.5),

        // Positivas baja activación
        EmocionReal::Serenidad => (0.8, 0.1, 0.3, "suave".to_string(), 0.3),
        EmocionReal::Saudade => (0.5, 0.3, 0.6, "diluida".to_string(), 0.4),
        EmocionReal::Agradecimiento => (0.7, 0.2, 0.4, "tibia".to_string(), 0.4),

        // Negativas alta activación
        EmocionReal::Enojo => (-0.7, 0.9, 0.9, "abrasadora".to_string(), 0.9),
        EmocionReal::Miedo => (-0.8, 0.95, 0.9, "helada".to_string(), -0.6),
        EmocionReal::Terror => (-0.95, 1.0, 1.0, "cortante".to_string(), -0.8),
        EmocionReal::Ansiedad => (-0.6, 0.8, 0.7, "apuñatante".to_string(), -0.3),

        // Negativas baja activación
        EmocionReal::Tristeza => (-0.7, 0.2, 0.5, "pesada".to_string(), -0.2),
        EmocionReal::Culpa => (-0.6, 0.4, 0.6, "constrictora".to_string(), -0.1),
        EmocionReal::Verguenza => (-0.5, 0.5, 0.5, "caliente".to_string(), 0.0),
        EmocionReal::Disgusto => (-0.7, 0.6, 0.6, "repulsiva".to_string(), -0.4),

        // Neutral
        _ => (0.0, 0.3, 0.1, "nublada".to_string(), 0.0),
    }
}

fn calcular_duracion(e: &EmocionReal, intensidad: f32) -> f32 {
    let base = match e {
        EmocionReal::Felicidad => 30.0,
        EmocionReal::Enojo => 15.0,
        EmocionReal::Miedo => 10.0,
        EmocionReal::Tristeza => 120.0,
        EmocionReal::Ansiedad => 60.0,
        _ => 30.0,
    };
    base * intensidad
}

fn generar_tags(e: &EmocionReal) -> Vec<String> {
    let mut tags = vec![format!("{:?}", e)];

    if e.es_positiva() {
        tags.push("positiva".to_string());
    }
    if e.es_negativa() {
        tags.push("negativa".to_string());
    }
    if e.es_social() {
        tags.push("social".to_string());
    }

    tags
}

fn emocion_a_somatico(e: &EmocionReal) -> TipoSomatico {
    match e {
        EmocionReal::Enojo | EmocionReal::Terror => TipoSomatico::Calor,
        EmocionReal::Miedo | EmocionReal::Ansiedad => TipoSomatico::Temblor,
        EmocionReal::Tristeza | EmocionReal::Melancolia => TipoSomatico::Vacío,
        EmocionReal::Felicidad | EmocionReal::Serenidad => TipoSomatico::Expansion,
        _ => TipoSomatico::Tension,
    }
}

fn obtener_localizacion(e: &EmocionReal) -> String {
    match e {
        EmocionReal::Miedo | EmocionReal::Terror => "pecho".to_string(),
        EmocionReal::Enojo => "cabeza".to_string(),
        EmocionReal::Tristeza => "corazon".to_string(),
        EmocionReal::Ansiedad => "estomago".to_string(),
        _ => "cuerpo".to_string(),
    }
}

fn generar_sensacion(e: &EmocionReal, intensidad: f32) -> String {
    match e {
        EmocionReal::Felicidad => {
            format!("sensacion de ligereza total ({:.0}%)", intensidad * 100.0)
        }
        EmocionReal::Miedo => format!("paralisis en el cuerpo ({:.0}%)", intensidad * 100.0),
        EmocionReal::Enojo => format!("explosion de energia ({:.0}%)", intensidad * 100.0),
        EmocionReal::Tristeza => format!("peso en el pecho ({:.0}%)", intensidad * 100.0),
        _ => format!("emocion difusa ({:.0}%)", intensidad * 100.0),
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crear_engine() {
        let engine = EmotionEngine::new();
        assert_eq!(engine.estado_base, EmocionReal::Neutral);
        assert_eq!(engine.emociones_activas.len(), 0);
    }

    #[test]
    fn test_sentir_felicidad() {
        let mut engine = EmotionEngine::new();
        let felt = engine.sentir(EmocionReal::Felicidad, 0.8, "buenas noticias");

        assert_eq!(felt.emocion, EmocionReal::Felicidad);
        assert!((felt.intensidad - 0.8).abs() < 0.01);
        assert_eq!(felt.causa, "buenas noticias");
    }

    #[test]
    fn test_emocion_dominante() {
        let mut engine = EmotionEngine::new();
        engine.sentir(EmocionReal::Tristeza, 0.3, "perdida");
        engine.sentir(EmocionReal::Felicidad, 0.7, "ganancia");

        let dominante = engine.emocion_dominante();
        assert!(dominante.is_some());
        assert_eq!(dominante.unwrap().emocion, EmocionReal::Felicidad);
    }

    #[test]
    fn test_valencia() {
        let mut engine = EmotionEngine::new();
        engine.sentir(EmocionReal::Felicidad, 1.0, "test");

        let valencia = engine.calcular_valencia();
        assert!(valencia > 0.0);
    }

    #[test]
    fn test_regulacion() {
        let mut engine = EmotionEngine::new();
        let felt = engine.sentir(EmocionReal::Enojo, 1.0, "injusticia");

        let resultado = engine.regular(felt.id, EstrategiaRegulacion::Reevaluacion);
        assert!(resultado);
    }

    #[test]
    fn test_emociones_sociales() {
        assert!(EmocionReal::Verguenza.es_social());
        assert!(EmocionReal::Culpa.es_social());
        assert!(!EmocionReal::Felicidad.es_social());
    }

    #[test]
    fn test_tags() {
        let tags = generar_tags(&EmocionReal::Felicidad);
        assert!(tags.contains(&"Felicidad".to_string()));
        assert!(tags.contains(&"positiva".to_string()));
    }
}
