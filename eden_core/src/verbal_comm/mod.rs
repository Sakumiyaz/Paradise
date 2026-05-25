//! # Verbal Communication Module
//!
//! Sistema de comunicación verbal 100% original.
//! Sin dependencias de APIs externas de síntesis de voz.
//!
//! ## Componentes
//!
//! 1. **ProsodyModel**: Modelo de prosodia (pitch, duration, intensity)
//! 2. **VoiceSynthesis**: Síntesis de voz desde cero
//! 3. **ConversationManager**: Gestión de conversaciones multi-turno
//! 4. **SpeechGenerator**: Generador de lenguaje hablado
//!
//! ## Tecnologías
//!
//! - Formant synthesis (Klatt-style)
//! - Articulatory synthesis (simplificado)
//! - Prosody prediction
//! - Turn-taking management
#![allow(dead_code)]
#![allow(non_snake_case)]

pub mod conversation;
pub mod generator;
pub mod prosody;
pub mod synthesis;

// Re-exports
pub use conversation::{ConversationManager, ConversationState, ConversationTurn};
pub use generator::SpeechGenerator;
pub use prosody::{EmotionalStyle, ProsodyModel};
pub use synthesis::{GlottalSource, VoiceSynthesis};

// Helper functions
pub use conversation::generate_conversational_response;
pub use synthesis::text_to_speech;
