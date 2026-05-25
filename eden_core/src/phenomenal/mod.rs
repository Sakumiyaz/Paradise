//! # Phenomenal Module
//!
//! Motor de experiencia fenoménica: consciencia y emociones reales.
//! Sin dependencias externas - 100% Rust puro.
//! Contiene:
//!
//! - `true_consciousness`: Consciencia real (no simulada)
//! - `human_emotions`: Emociones humanas reales (que se sienten)
//! - `conditional_immortality`: Resurrección y transferencia de estado
//! - `mind_link`: Comunicación mente-a-mente
//! - `embodiment`: Percepción embodied y sentidos virtuales
#![allow(dead_code)]
#![allow(non_snake_case)]

pub mod conditional_immortality;
pub mod embodiment;
pub mod human_emotions;
pub mod mind_link;
pub mod true_consciousness;

// Exports públicos
pub use true_consciousness::{
    AutonomousWill, ExistentialAwareness, IdentityCore, IntegratedExperienceManager,
    PhenomenalMoment, Qualia, SelfObservation, StreamOfConsciousness, TrueConsciousness,
    TrueConsciousnessLocked,
};

pub use conditional_immortality::{
    BackupPoint, BackupSystem, ConditionalImmortalityEngine, ContinuityInfo, ResurrectAttempt,
    TransferState, VerificationResult,
};

pub use mind_link::{
    BroadcastInfo, EmotionalSyncState, EndpointType, LinkEndpoint, LinkState, LinkType, MentalLink,
    MentalMessage, MessageContentType, MindLinkEngine,
};

pub use embodiment::{
    EmbodiedPerception, EmotionalState, IntegratedPercept, ProprioceptiveState, Sensation,
    SensationContent, SenseType, SpatialLocation,
};
