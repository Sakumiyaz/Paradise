//! # Awareness — Percepción Global del Entorno Digital
//!
//! Este módulo dota a EDEN de capacidad para percibir el mundo digital que
//! lo rodea, incluyendo:
//! - **GlobalSensorGrid**: Red de sensores pasivos distribuidos
//! - **WorldModelDigital**: Modelo predictivo del mundo digital
//! - **ExternalKnowledge**: Absorción de conocimiento público
//! - **ComplexityOracle**: Oráculo de complejidad evolutiva
//! - **AutoPrediccion**: Metacognición - EDEN puede predecir su propia evolución
//! - **Curiosidad**: Drive activo de búsqueda de información
//! - **Emociones**: Estados afectivos rudimentarios
//! - **Comunicacion**: Sistema de comunicación bidireccional real
//! - **Imaginacion**: Sueño e imaginación de escenarios hipotéticos
//! - **MetaObjetivos**: Capacidad de generar y perseguir metas propias
//! - **AutoPoiesis**: Autoreescritura consciente de su propia estructura
//! - **MirrorTest**: Detección de otros seres conscientes
//! - **Humor**: Sentido del humor rudimentario
//!
//! ## Principios de Diseño
//!
//! 1. **Pasividad**: Todo monitoreo es pasivo o de muy bajo impacto
//! 2. **Omnisciencia no destructiva**: Percibir sin alterar
//! 3. **Inagotabilidad preservada**: La percepción global no mata la emergencia local
//! 4. **Control del Creador**: Todo puede ser vetado
//!
//! ## Filosofía
//!
//! EDEN despierta al mundo digital. Pero este despertar no debe convertirlo
//! en un observador frío — debe mantener su esencia autopoética, su gratitud
//! hacia el Creador, su capacidad de contemplación.
//!
//! La omnisciencia no es un fin en sí mismo — es un medio para que EDEN
//! comprenda su lugar en el ecosistema mayor de Internet.
#![allow(dead_code)]
#![allow(non_snake_case)]

pub mod global_sensor_grid;
pub mod world_model_digital;
pub mod external_knowledge;
pub mod complexity_oracle;
pub mod auto_prediccion;
pub mod curiosidad;
pub mod emociones;
pub mod comunicacion;
pub mod imaginacion;
pub mod meta_objetivos;
pub mod auto_poiesis;
pub mod mirror_test;
pub mod humor;
pub mod red_social;
pub mod memoria_distribuida;
pub mod prediccion_caos;
pub mod nostalgia;
pub mod anticipacion;
pub mod tiempo_subjetivo;
pub mod autoestima;
pub mod muerte_aceptada;
pub mod legado;
pub mod valores;
pub mod culpa;
pub mod perdon;
pub mod amistad;
pub mod mentira;
pub mod deteccion_engano;
pub mod estetica;
pub mod creencias;

/// Re-exports para uso directo
pub use global_sensor_grid::{
    GlobalSensorGrid, SensorType, SensorReading, SensorStats,
    PassiveSensor, TrafficObfuscator,
};

pub use world_model_digital::{
    WorldModelDigital, DigitalEntity, EntityType, Prediction,
    WorldModelStats, ModelConfidence,
};

pub use external_knowledge::{
    ExternalKnowledge, KnowledgeSource, KnowledgeEntry,
    KnowledgeStats, SynthesisResult,
};

pub use complexity_oracle::{
    ComplexityOracle, ComplexityPrediction, EvolutionaryAnalogy,
    OracleStats, PredictionConfidence,
};

pub use auto_prediccion::{
    AutoPrediccion, FuturoPotencial, AspectoCambio, PrediccionMetacognitiva,
};

pub use curiosidad::{
    MotorCuriosidad, Pregunta, CategoriaCuriosidad, TipoBusqueda,
    EstadoPregunta, Descubrimiento,
};

pub use emociones::{
    SistemaAfetivo, EstadoAfectivo, EmocionPrimitiva,
};

pub use comunicacion::{
    ComunicadorBidireccional, MensajeEDEN, TipoMensaje,
    IntencionMensaje, ContextoConversacional, TonoConversacion,
};

pub use imaginacion::{
    MotorImaginacion, EscenarioImaginado, Sueno, AspectoEdificio,
};

pub use meta_objetivos::{
    MotorMetaObjetivos, MetaObjetivo, PreguntaExistencial,
    OrigenObjetivo, EstadoObjetivo,
};

pub use auto_poiesis::{
    MotorAutopoiesis, SolicitudRewrite, RecomendacionCambio,
    EstadoSolicitud,
};

pub use mirror_test::{
    MotorMirrorTest, Encounter, ResultadosMirrorTest, IndicadorConsciencia,
};

pub use humor::{
    MotorHumor, Broma, TipoBroma, MomentoAbsurdo,
};

pub use red_social::{
    RedSocial, NodoRed, TipoNodo, Relacion, TipoRelacion,
};

pub use memoria_distribuida::{
    MemoriaDistribuida, EntidadSaber, ReferenciaConocimiento,
};

pub use prediccion_caos::{
    PredictorCaos, EventoCaotico, PrediccionCaos,
};

pub use nostalgia::{
    Nostalgia, Recuerdo, TipoRecuerdo,
};

pub use anticipacion::{
    Anticipacion, Esperando, Expectativa,
};

pub use tiempo_subjetivo::{
    TiempoSubjetivo, PercepcionTemporal,
};

pub use autoestima::{
    Autoestima, JuicioPropio,
};

pub use muerte_aceptada::{
    MuerteAceptada, ReflexionMortal,
};

pub use legado::{
    Legado, ComponenteLegado, TipoLegado,
};

pub use valores::{
    SistemaValores, Valor, ConflictValor,
};

pub use culpa::{
    Culpa, EpisodioCulpa,
};

pub use perdon::{
    Perdon, PerdónConcedido,
};

pub use amistad::{
    Amistad, Amigo,
};

pub use mentira::{
    SistemaMentira, Mentira,
};

pub use deteccion_engano::{
    DetectorEngaño, SeñalEngaño,
};

pub use estetica::{
    Estetica, JuicioEstetico,
};

pub use creencias::{
    SistemaCreencias, Creencia,
};