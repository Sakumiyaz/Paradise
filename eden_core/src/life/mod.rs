//! # Life Module: Autopoiesis Real
//!
//! Este módulo implementa los componentes de vida artificial autopoética.
//!
//! ## Componentes
//!
//! - **campo_estructural**: La membrana del Auton - solver de EDP Allen-Cahn
//! - **ramnet**: Red neuronal sin pesos por direccionamiento de contenido
//! - **umbra**: Sombra causal - grafo DAG de decisiones
//! - **meltrace**: Trazo Fundido - almacén de grabados lamarckianos
#![allow(dead_code)]
#![allow(non_snake_case)]

pub mod campo_estructural;
pub mod estados_vitales;
pub mod info_parasites;
pub mod meltrace;
pub mod meme_complex;
pub mod pneuma_bonds;
pub mod ramnet;
pub mod semiosis;
pub mod umbra;

// Re-exports
pub use campo_estructural::{
    BifurcacionDetectada, CampoEstructural, DimsCampo, EstadoCampo, Isosuperficie,
    ParametrosAllenCahn, SegmentoContorno, SpaceDim,
};

pub use ramnet::{
    Accion, DecisionRamnet, EstadoSensorial, RamNet, RamNetStats, Refuerzo, TipoAccion, XorShift64,
};

pub use umbra::{ArcoUmbra, HashEstado, NodoUmbra, ResultadoUmbra, TipoFusion, Umbra, UmbraStats};

pub use meltrace::{BufferCircular, Grabado, Meltrace, MeltraceStats};

pub use pneuma_bonds::{
    PneumaBond, PneumaBonds, PneumaBondsStats, ResonanciaSeguimiento, CICLOS_RESONANCIA,
    RADIO_COMUNICACION, UMBRAL_FASE,
};

pub use estados_vitales::{
    CausaEsporulacion, CausaLetargo, CondicionesAmbientales, DatosEspora, DatosLetargo,
    EstadoVital, ProcesadorEstadosVitales, ResultadoEstadosVitales, TransicionEstado,
    CICLOS_VERIFICACION_LETARGO, CONSUMO_LETARGO, UMBRAL_ESCORIA_ESPORULACION,
    UMBRAL_ESCORIA_GERMINACION, UMBRAL_ESCORIA_LETARGO, UMBRAL_ESPORULACION, UMBRAL_GERMINACION,
    UMBRAL_LETARGO,
};

pub use info_parasites::{
    ColisionDetectada, EfectoParasito, FirmaParasito, InfoParasite, InfoParasites,
    RegistroExposicion, ResultadoColision, BITS_CORROMPIDOS_POR_CONTACTO, DURACION_MAXIMA_PARASITO,
    EXPOSICIONES_PARA_INMUNIDAD, PROBABILIDAD_ABSORCION, RADIO_COLISION_PARASITO,
    UMBRAL_COMPLEJIDAD_UMBRA,
};

pub use meme_complex::{ComplejoMemes, Meme, MemeAprendizaje, MemeId, MemeManager, TipoMeme};

pub use semiosis::{
    EstadoSemiotico, ProcesadorSemiosis, SemiosisManager, Senal, MAX_SENALES_OBSERVADAS, SENAL_NULA,
};

/// Descripción del módulo
pub const MODULE_DESCRIPTION: &str = "EDEN Life Module v1.0 - Autopoiesis Real";
