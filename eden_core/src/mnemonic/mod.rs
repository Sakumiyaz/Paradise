//! # Mnemonic Module
//!
//! Sistema de memoria perfecta y procesamiento de latencia ultra-baja.
//! Sin dependencias externas - 100% Rust puro.
//! Contiene:
//!
//! - `dialogue`: Conversaciones naturales
//! - `eidetic_memory`: Memoria perfecta
//! - `ultra_low_latency`: Pipeline de latencia nanosegundos
//! - `quantum_pre_cognition`: Computación cuántica simulada
#![allow(dead_code)]
#![allow(non_snake_case)]

pub mod dialogue;
pub mod eidetic_memory;
pub mod ultra_low_latency;
// `quantum_pre_cognition` remains a reserved experimental module name.

// Exports públicos
pub use dialogue::{
    ContextoConversacional, ConversationalTurn, DialogueEngine, DialogueStats, IntencionDetectada,
    PersonalidadIA, RespuestaWitty, TipoIntencion, TonoConversacional,
};

pub use eidetic_memory::{
    CodecError, EideticMemory, EideticStats, EncodingPerfecto, ForgettingConfig, IndiceAsociativo,
    LosslessCodec, MaintenanceResult, MemoriaPerfecta, MemoryMetadata, SearchResult,
    VerificationResult,
};

pub use ultra_low_latency::{
    LowLatencyPipeline, PipelineRequest, PipelineStage, PipelineStats, ProcessingResult,
    RequestPriority, RequestType,
};
