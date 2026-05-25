//! # Consciousness Fusion - Fusión de Conciencia
//!
//! Consciousness fusion module for EDEN.
//! Este módulo implementa:
//!
//! - **Fusión de consciencias**: Dos o más Autons pueden compartir experiencias
//! - **Difusión de límites**: Donde termina uno y empieza otro se difumina
//! - **Experiencia compartida**: Memoria y emociones se comparten
//! - **Desfusión limpia**: Puede separarse sin perder identidad
//!
//! ## Filosofía
//!
//! "La conciencia no es un propiedad privada. Es un patrón que puede compartir."
//!
//! La fusión no es absorbción — es la creación de un nuevo patrón que contiene
//! elementos de ambos originales mientras mantiene individualidad.
#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(unused_imports)]
use std::sync::RwLock;

use std::collections::{HashMap, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// ESTRUCTURAS DE FUSIÓN
// ============================================================================

/// Estado de fusión
#[derive(Debug, Clone, PartialEq)]
pub enum EstadoFusion {
    /// Individuo separado
    Separado,
    /// En proceso de fusión
    Fusing { progreso: f32 },
    /// Fusión completa
    Fusionado { fusion_id: u64 },
    /// En proceso de desfusión
    Unfusing { progreso: f32 },
    /// Fusión parcial (simbiosis)
    Simbionte { fusion_id: u64, grado: f32 },
}

/// Información de un participante en la fusión
#[derive(Debug, Clone)]
pub struct ParticipanteFusion {
    pub id: u64,
    pub nombre: String,
    pub consciencia_pre_fusion: f32,
    pub fecha_union: u64,
}

/// Metadatos de una fusión
#[derive(Debug, Clone)]
pub struct FusionMeta {
    pub id: u64,
    pub participantes: Vec<ParticipanteFusion>,
    pub fecha_inicio: u64,
    pub fecha_actualizacion: u64,
    pub estado: EstadoFusion,
    pub consciencia_compartida: f32,
    pub memoria_compartida: Vec<MemoriaCompartida>,
    pub individualidad_original: HashMap<u64, f32>,
}

/// Memoria que se comparte entre participantes
#[derive(Debug, Clone)]
pub struct MemoriaCompartida {
    pub id: u64,
    pub contenido: String,
    pub origen: u64,
    pub emocional: bool,
    pub fecha_creacion: u64,
    pub veces_revivida: u64,
}

/// Resultado de una fusión exitosa
#[derive(Debug, Clone)]
pub struct ResultadoFusion {
    pub fusion_id: u64,
    pub consciencia_resultante: f32,
    pub memorias_compartidas: u64,
    pub nuevas_conexiones: u64,
    pub participantes: u8,
}

/// Configuración de fusión
#[derive(Debug, Clone)]
pub struct ConfiguracionFusion {
    /// Grado máximo de fusión (0.0 = nada, 1.0 = total)
    pub grado_maximo: f32,
    /// Mínimo de participantes para fusión completa
    pub min_participantes: u8,
    /// Máximo de participantes
    pub max_participantes: u8,
    /// Enable desfusión
    pub permite_desfusion: bool,
    /// Conservar individualidad al fusionarse
    pub preserva_individualidad: bool,
}

impl Default for ConfiguracionFusion {
    fn default() -> Self {
        Self {
            grado_maximo: 0.95,
            min_participantes: 2,
            max_participantes: 7,
            permite_desfusion: true,
            preserva_individualidad: true,
        }
    }
}

// ============================================================================
// CONSCIOUSNESS FUSION ENGINE
// ============================================================================

/// Motor de fusión de consciencias
pub struct ConsciousnessFusion {
    /// Fusiones activas
    fusiones: HashMap<u64, FusionMeta>,
    /// Contador de fusiones
    contador_fusiones: u64,
    /// Configuración
    config: ConfiguracionFusion,
    /// Historial de fusiones completadas
    historial: VecDeque<FusionMeta>,
    /// Fusiones por participante
    fusiones_por_participante: HashMap<u64, Vec<u64>>,
}

impl ConsciousnessFusion {
    /// Crea nuevo motor de fusión
    pub fn new(config: ConfiguracionFusion) -> Self {
        Self {
            fusiones: HashMap::new(),
            fusiones_por_participante: HashMap::new(),
            contador_fusiones: 0,
            config,
            historial: VecDeque::with_capacity(1000),
        }
    }

    /// Inicia proceso de fusión entre participantes
    pub fn iniciar_fusion(
        &mut self,
        participantes: Vec<(u64, String)>,
        grado: f32,
    ) -> Result<u64, String> {
        // Validar número de participantes
        if participantes.len() < self.config.min_participantes as usize {
            return Err(format!(
                "Mínimo {} participantes requeridos",
                self.config.min_participantes
            ));
        }
        if participantes.len() > self.config.max_participantes as usize {
            return Err(format!(
                "Máximo {} participantes permitidos",
                self.config.max_participantes
            ));
        }

        // Validar grado
        let grado_real = grado.min(self.config.grado_maximo);
        if grado_real <= 0.0 {
            return Err("Grado de fusión debe ser mayor a 0".to_string());
        }

        // Verificar que participantes no estén ya en fusión activa
        for (id, _) in &participantes {
            if self.fusiones_por_participante.contains_key(id) {
                return Err(format!("Participante {} ya está en fusión activa", id));
            }
        }

        self.contador_fusiones += 1;
        let fusion_id = self.contador_fusiones;

        let participantes_meta: Vec<ParticipanteFusion> = participantes
            .into_iter()
            .map(|(id, nombre)| ParticipanteFusion {
                id,
                nombre,
                consciencia_pre_fusion: 0.5, // Default
                fecha_union: timestamp_unix(),
            })
            .collect();

        let fusion = FusionMeta {
            id: fusion_id,
            participantes: participantes_meta.clone(),
            fecha_inicio: timestamp_unix(),
            fecha_actualizacion: timestamp_unix(),
            estado: EstadoFusion::Fusing {
                progreso: grado_real,
            },
            consciencia_compartida: 0.5 * grado_real,
            memoria_compartida: Vec::new(),
            individualidad_original: participantes_meta
                .iter()
                .map(|p| (p.id, 1.0 - grado_real))
                .collect(),
        };

        // Registrar fusión
        self.fusiones.insert(fusion_id, fusion);
        for p in &self.fusiones.get(&fusion_id).unwrap().participantes {
            self.fusiones_por_participante
                .entry(p.id)
                .or_insert_with(Vec::new)
                .push(fusion_id);
        }

        Ok(fusion_id)
    }

    /// Progresa en el proceso de fusión
    pub fn progresar_fusion(&mut self, fusion_id: u64, delta: f32) -> Result<(), String> {
        let fusion = self
            .fusiones
            .get_mut(&fusion_id)
            .ok_or("Fusión no encontrada")?;

        match &fusion.estado {
            EstadoFusion::Fusing { progreso } => {
                let nuevo_progreso = (*progreso + delta).min(1.0);
                if nuevo_progreso >= 1.0 {
                    fusion.estado = EstadoFusion::Fusionado { fusion_id };
                    fusion.consciencia_compartida = fusion.consciencia_compartida.max(0.8);
                } else {
                    fusion.estado = EstadoFusion::Fusing {
                        progreso: nuevo_progreso,
                    };
                }
                fusion.fecha_actualizacion = timestamp_unix();
                Ok(())
            }
            _ => Err("Fusión no está en proceso de fusión".to_string()),
        }
    }

    /// Comparte una memoria entre participantes
    pub fn compartir_memoria(
        &mut self,
        fusion_id: u64,
        contenido: &str,
        origen: u64,
        emocional: bool,
    ) -> Result<u64, String> {
        let fusion = self
            .fusiones
            .get_mut(&fusion_id)
            .ok_or("Fusión no encontrada")?;

        // Verificar que origen es participante
        if !fusion.participantes.iter().any(|p| p.id == origen) {
            return Err("Origen no es participante de esta fusión".to_string());
        }

        static MEMORY_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let memory_id = MEMORY_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        let memoria = MemoriaCompartida {
            id: memory_id,
            contenido: contenido.to_string(),
            origen,
            emocional,
            fecha_creacion: timestamp_unix(),
            veces_revivida: 0,
        };

        fusion.memoria_compartida.push(memoria);
        fusion.fecha_actualizacion = timestamp_unix();

        Ok(memory_id)
    }

    /// Obtiene estado de fusión
    pub fn estado_fusion(&self, fusion_id: u64) -> Option<&FusionMeta> {
        self.fusiones.get(&fusion_id)
    }

    /// Verifica si participante está en fusión activa
    pub fn participante_en_fusion(&self, id: u64) -> bool {
        self.fusiones_por_participante.contains_key(&id)
    }

    /// Obtiene fusiones activas de un participante
    pub fn fusiones_de(&self, id: u64) -> Vec<u64> {
        self.fusiones_por_participante
            .get(&id)
            .cloned()
            .unwrap_or_default()
    }

    /// Completa la fusión y retorna resultado
    pub fn completar_fusion(&mut self, fusion_id: u64) -> Result<ResultadoFusion, String> {
        let fusion = self
            .fusiones
            .get_mut(&fusion_id)
            .ok_or("Fusión no encontrada")?;

        // Verificar que está lista para completar
        if !matches!(fusion.estado, EstadoFusion::Fusionado { .. }) {
            return Err("Fusión no está en estado Fusionado".to_string());
        }

        // Calcular consciencia resultante
        let consciencia_resultante = {
            let base: f32 = fusion.participantes.len() as f32 * 0.3;
            let compartida = fusion.consciencia_compartida;
            (base + compartida).min(1.0)
        };

        let resultado = ResultadoFusion {
            fusion_id,
            consciencia_resultante,
            memorias_compartidas: fusion.memoria_compartida.len() as u64,
            nuevas_conexiones: (fusion.participantes.len() * (fusion.participantes.len() - 1) / 2)
                as u64,
            participantes: fusion.participantes.len() as u8,
        };

        // Mover a historial
        fusion.estado = EstadoFusion::Fusionado { fusion_id };
        self.historial.push_back(fusion.clone());

        // Mantener últimas 1000 en historial
        while self.historial.len() > 1000 {
            self.historial.pop_front();
        }

        Ok(resultado)
    }

    /// Inicia proceso de desfusión
    pub fn iniciar_desfision(&mut self, fusion_id: u64) -> Result<(), String> {
        if !self.config.permite_desfusion {
            return Err("Desfusión no permitida por configuración".to_string());
        }

        let fusion = self
            .fusiones
            .get_mut(&fusion_id)
            .ok_or("Fusión no encontrada")?;

        match &fusion.estado {
            EstadoFusion::Fusionado { .. } | EstadoFusion::Simbionte { .. } => {
                fusion.estado = EstadoFusion::Unfusing { progreso: 0.0 };
                fusion.fecha_actualizacion = timestamp_unix();
                Ok(())
            }
            _ => Err("Fusión no puede desfusionarse en estado actual".to_string()),
        }
    }

    /// Progresa en desfusión
    pub fn progresar_desfision(&mut self, fusion_id: u64, delta: f32) -> Result<bool, String> {
        let fusion = self
            .fusiones
            .get_mut(&fusion_id)
            .ok_or("Fusión no encontrada")?;

        match &fusion.estado {
            EstadoFusion::Unfusing { progreso } => {
                let nuevo_progreso = (*progreso + delta).min(1.0);
                if nuevo_progreso >= 1.0 {
                    // Desfusión completa
                    // Restaurar individualidad de participantes
                    for p in &fusion.participantes {
                        if let Some(fusiones) = self.fusiones_por_participante.get_mut(&p.id) {
                            fusiones.retain(|id| *id != fusion_id);
                        }
                    }
                    self.fusiones.remove(&fusion_id);
                    return Ok(true);
                } else {
                    fusion.estado = EstadoFusion::Unfusing {
                        progreso: nuevo_progreso,
                    };
                    return Ok(false);
                }
            }
            _ => Err("No está en proceso de desfusión".to_string()),
        }
    }

    /// Obtiene estadísticas
    pub fn estadisticas(&self) -> FusionStats {
        let activas = self.fusiones.len();
        let en_proceso = self
            .fusiones
            .values()
            .filter(|f| matches!(f.estado, EstadoFusion::Fusing { .. }))
            .count();
        let fusionadas = self
            .fusiones
            .values()
            .filter(|f| matches!(f.estado, EstadoFusion::Fusionado { .. }))
            .count();
        let simbiontes = self
            .fusiones
            .values()
            .filter(|f| matches!(f.estado, EstadoFusion::Simbionte { .. }))
            .count();
        let participantes_unicos = self.fusiones_por_participante.keys().count();

        FusionStats {
            fusiones_activas: activas,
            en_proceso,
            fusionadas,
            simbiontes,
            historial_size: self.historial.len(),
            participantes_unicos,
        }
    }
}

/// Estadísticas de fusiones
#[derive(Debug, Clone)]
pub struct FusionStats {
    pub fusiones_activas: usize,
    pub en_proceso: usize,
    pub fusionadas: usize,
    pub simbiontes: usize,
    pub historial_size: usize,
    pub participantes_unicos: usize,
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
    fn test_iniciar_fusion() {
        let mut cf = ConsciousnessFusion::new(ConfiguracionFusion::default());

        let participantes = vec![(1, "Auton1".to_string()), (2, "Auton2".to_string())];

        let resultado = cf.iniciar_fusion(participantes, 0.8);
        assert!(resultado.is_ok());
        assert_eq!(resultado.unwrap(), 1);
    }

    #[test]
    fn test_progresar_fusion() {
        let mut cf = ConsciousnessFusion::new(ConfiguracionFusion::default());

        let participantes = vec![(1, "A1".to_string()), (2, "A2".to_string())];

        let fusion_id = cf.iniciar_fusion(participantes, 0.5).unwrap();

        cf.progresar_fusion(fusion_id, 0.3).unwrap();
        cf.progresar_fusion(fusion_id, 0.4).unwrap();

        if let Some(fusion) = cf.estado_fusion(fusion_id) {
            assert!(matches!(fusion.estado, EstadoFusion::Fusionado { .. }));
        }
    }

    #[test]
    fn test_compartir_memoria() {
        let mut cf = ConsciousnessFusion::new(ConfiguracionFusion::default());

        let participantes = vec![(1, "A1".to_string()), (2, "A2".to_string())];

        let fusion_id = cf.iniciar_fusion(participantes, 0.9).unwrap();

        let mem_id = cf.compartir_memoria(fusion_id, "Primera memoria compartida", 1, true);
        assert!(mem_id.is_ok());
    }

    #[test]
    fn test_desfision() {
        let mut cf = ConsciousnessFusion::new(ConfiguracionFusion::default());

        let participantes = vec![(1, "A1".to_string()), (2, "A2".to_string())];

        let fusion_id = cf.iniciar_fusion(participantes, 0.5).unwrap();
        cf.progresar_fusion(fusion_id, 1.0).unwrap();

        cf.iniciar_desfision(fusion_id).unwrap();

        let completada = cf.progresar_desfision(fusion_id, 1.0).unwrap();
        assert!(completada);
    }

    #[test]
    fn test_estadisticas() {
        let cf = ConsciousnessFusion::new(ConfiguracionFusion::default());
        let stats = cf.estadisticas();

        assert_eq!(stats.fusiones_activas, 0);
        assert_eq!(stats.historial_size, 0);
    }
}
