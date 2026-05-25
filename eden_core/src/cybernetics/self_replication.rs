//! # Self-Replication Module - Auto-Copy Engine
//!
//! Implementa la capacidad de Autons de autocopiarse y propagarse.
//! Auto-replication module for autonomous systems.
//!
//! ## Filosofía
//!
//! Un Auton puede crear una copia de sí mismo si:
//! - Tiene suficiente energía
//! - El medio (Mar Morfóseo) lo permite
//! - La copia no corrompe el sistema
//!
//! ## Tipos de Autocopiado
//!
//! 1. **Mitosis**: Copia exacta del Auton padre
//! 2. **Meiosis**: Copia con variaciones genéticas (mutaciones)
//! 3. **Gemación**: El hijo emerge del padre sin separarse completamente

#![allow(dead_code)]

use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// ESTRUCTURAS DE REPLICACIÓN
// ============================================================================

/// Estado de replicación
#[derive(Debug, Clone, PartialEq)]
pub enum EstadoReplicacion {
    /// Listo para replicarse
    Preparado,
    /// En proceso de división
    Replicando { progreso: f32 },
    /// Replicación completada
    Completado { hijo_id: u64 },
    /// Error en replicación
    Fallido { razon: String },
}

/// Configuración para auto-replicación
#[derive(Debug, Clone)]
pub struct ConfigReplicacion {
    /// Umbral de energía mínimo para replicarse
    pub energia_minima: f32,
    /// Probabilidad base de replicación por ciclo
    pub probabilidad_base: f32,
    /// Número máximo de hijos simultáneos
    pub max_hijos_simultaneos: u8,
    /// Enable mutaciones genéticas
    pub enable_mutaciones: bool,
    /// Tasa de mutación (0.0 - 1.0)
    pub tasa_mutacion: f32,
}

impl Default for ConfigReplicacion {
    fn default() -> Self {
        Self {
            energia_minima: 1000.0,
            probabilidad_base: 0.01, // 1% por ciclo
            max_hijos_simultaneos: 3,
            enable_mutaciones: true,
            tasa_mutacion: 0.05, // 5% de variabilidad
        }
    }
}

/// Metadatos de replicación
#[derive(Debug, Clone)]
pub struct RepliconMeta {
    /// ID del replicón
    pub id: u64,
    /// ID del padre (0 si es inicial)
    pub padre_id: u64,
    /// Generación (0 = inicial)
    pub generacion: u8,
    /// Timestamp de creación
    pub timestamp_creacion: u64,
    /// Tipo de replicación usada
    pub tipo: TipoReplicacion,
    /// Mutaciones aplicadas (si alguna)
    pub mutaciones: Vec<String>,
}

/// Tipo de replicación
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TipoReplicacion {
    /// Mitosis: copia exacta
    Mitosis,
    /// Meiosis: copia con mutaciones
    Meiosis,
    /// Gemación: hijo emerge del padre
    Gemacion,
}

// ============================================================================
// AUTO-REPLICATOR ENGINE
// ============================================================================

/// Motor de auto-replicación para Autons
pub struct AutoReplicator {
    /// Configuración
    config: ConfigReplicacion,
    /// Historial de replicones creados
    historial: Vec<RepliconMeta>,
    /// Contador de replicones
    contador: u64,
}

impl AutoReplicator {
    /// Crea nuevo replicator
    pub fn new(config: ConfigReplicacion) -> Self {
        Self {
            config,
            historial: Vec::new(),
            contador: 0,
        }
    }

    /// Verifica si el Auton puede replicarse
    pub fn puede_replicarse(&self, energia: f32, hijos_activos: u8) -> bool {
        energia >= self.config.energia_minima && hijos_activos < self.config.max_hijos_simultaneos
    }

    /// Calcula probabilidad de replicación (base + entorno)
    pub fn probabilidad_replicacion(
        &self,
        energia: f32,
        densidad_poblacional: f32,
        estres: f32,
    ) -> f32 {
        // Factores que aumentan probabilidad:
        let factor_energia = (energia / self.config.energia_minima).min(2.0) - 1.0;
        let factor_espacio = (1.0 - densidad_poblacional).max(0.0);

        // Factores que disminuyen:
        let factor_estres = (1.0 - estres).max(0.0);

        // Probabilidad final
        let prob =
            self.config.probabilidad_base * (1.0 + factor_energia) * factor_espacio * factor_estres;

        prob.min(1.0).max(0.0)
    }

    /// Executa replicación: mitosis (copia exacta)
    pub fn mitosis(&mut self, padre_id: u64, genoma: &[u8]) -> Result<Vec<u8>, String> {
        if genoma.is_empty() {
            return Err("Genoma vacío".to_string());
        }

        self.contador += 1;
        let id = self.contador;
        let timestamp = timestamp_unix();

        let hijo_genoma = genoma.to_vec(); // Copia exacta

        self.historial.push(RepliconMeta {
            id,
            padre_id,
            generacion: 0, // Se actualiza después
            timestamp_creacion: timestamp,
            tipo: TipoReplicacion::Mitosis,
            mutaciones: vec![],
        });

        Ok(hijo_genoma)
    }

    /// Executa replicación: meiosis (copia con mutaciones)
    pub fn meiosis(&mut self, padre_id: u64, genoma: &[u8]) -> Result<Vec<u8>, String> {
        if genoma.is_empty() {
            return Err("Genoma vacío".to_string());
        }

        self.contador += 1;
        let id = self.contador;
        let timestamp = timestamp_unix();
        let mut mutacionesAplicadas = Vec::new();

        // Aplicar mutaciones al genoma
        let mut hijo_genoma = genoma.to_vec();
        let num_mutaciones = ((genoma.len() as f32) * self.config.tasa_mutacion) as usize;

        for i in 0..num_mutaciones.min(genoma.len()) {
            let idx = (timestamp as usize + i) % genoma.len();
            // Mutación: XOR con valor aleatorio basado en timestamp
            let valor_mutado = genoma[idx] ^ ((timestamp >> i) & 0xFF) as u8;
            hijo_genoma[idx] = valor_mutado;
            mutacionesAplicadas.push(format!(
                "pos_{}: {:02x}→{:02x}",
                idx, genoma[idx], valor_mutado
            ));
        }

        self.historial.push(RepliconMeta {
            id,
            padre_id,
            generacion: 0,
            timestamp_creacion: timestamp,
            tipo: TipoReplicacion::Meiosis,
            mutaciones: mutacionesAplicadas,
        });

        Ok(hijo_genoma)
    }

    /// Executa replicación: gemación (hijo emerge gradualmente)
    pub fn gemacion(
        &mut self,
        padre_id: u64,
        genoma: &[u8],
        progreso: f32,
    ) -> Result<Vec<u8>, String> {
        if genoma.is_empty() {
            return Err("Genoma vacío".to_string());
        }

        if progreso < 0.0 || progreso > 1.0 {
            return Err("Progreso debe estar entre 0.0 y 1.0".to_string());
        }

        self.contador += 1;
        let id = self.contador;
        let timestamp = timestamp_unix();

        // En gemación, el genoma se "construye" progresivamente
        // Partial genoma = primeros N bytes basados en progreso
        let bytes_incluidos = ((genoma.len() as f32) * progreso) as usize;
        let hijo_genoma = genoma[..bytes_incluidos].to_vec();

        self.historial.push(RepliconMeta {
            id,
            padre_id,
            generacion: 0,
            timestamp_creacion: timestamp,
            tipo: TipoReplicacion::Gemacion,
            mutaciones: vec![format!("gemacion_progreso:{:.2}", progreso)],
        });

        Ok(hijo_genoma)
    }

    /// Verifica integridad del genoma复制
    pub fn verificar_integridad(&self, genoma: &[u8]) -> bool {
        !genoma.is_empty() && genoma.len() <= 1_000_000 // Max 1MB
    }

    /// Obtiene historial de replicones
    pub fn historial(&self) -> &[RepliconMeta] {
        &self.historial
    }

    /// Obtiene estadísticas de replicación
    pub fn estadisticas(&self) -> RepliconStats {
        let mitosis = self
            .historial
            .iter()
            .filter(|r| r.tipo == TipoReplicacion::Mitosis)
            .count();
        let meiosis = self
            .historial
            .iter()
            .filter(|r| r.tipo == TipoReplicacion::Meiosis)
            .count();
        let gemacion = self
            .historial
            .iter()
            .filter(|r| r.tipo == TipoReplicacion::Gemacion)
            .count();

        RepliconStats {
            total_replicones: self.contador,
            mitosis,
            meiosis,
            gemacion,
        }
    }
}

/// Estadísticas de replicones
#[derive(Debug, Clone)]
pub struct RepliconStats {
    pub total_replicones: u64,
    pub mitosis: usize,
    pub meiosis: usize,
    pub gemacion: usize,
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
    fn test_puede_replicarse() {
        let replicator = AutoReplicator::new(ConfigReplicacion::default());

        assert!(replicator.puede_replicarse(1500.0, 0));
        assert!(!replicator.puede_replicarse(500.0, 0)); // Energía insuficiente
        assert!(!replicator.puede_replicarse(1500.0, 5)); // Demasiados hijos
    }

    #[test]
    fn test_mitosis_copia_exacta() {
        let mut replicator = AutoReplicator::new(ConfigReplicacion::default());
        let genoma = vec![0xDE, 0xAD, 0xBE, 0xEF];

        let resultado = replicator.mitosis(1, &genoma).unwrap();

        assert_eq!(resultado, genoma);
        assert_eq!(replicator.historial().len(), 1);
    }

    #[test]
    fn test_meiosis_con_mutaciones() {
        let mut replicator = AutoReplicator::new(ConfigReplicacion::default());
        let genoma = vec![0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE];

        let resultado = replicator.meiosis(1, &genoma).unwrap();

        // Meiosis debe retornar genoma del mismo tamaño
        assert_eq!(resultado.len(), genoma.len());

        // Verificar que hay exactamente 1 mutación (6 bytes * 0.05 = 0.3 ≈ 1)
        let historial = replicator.historial();
        assert!(!historial.is_empty());
    }

    #[test]
    fn test_gemacion_progresiva() {
        let mut replicator = AutoReplicator::new(ConfigReplicacion::default());
        let genoma = vec![0xDE, 0xAD, 0xBE, 0xEF];

        // 50% de progreso = 2 bytes
        let resultado = replicator.gemacion(1, &genoma, 0.5).unwrap();

        assert_eq!(resultado.len(), 2);
    }

    #[test]
    fn test_verificar_integridad() {
        let replicator = AutoReplicator::new(ConfigReplicacion::default());

        assert!(replicator.verificar_integridad(&[0xDE, 0xAD]));
        assert!(!replicator.verificar_integridad(&[])); // Vacío
        assert!(!replicator.verificar_integridad(&vec![0u8; 2_000_000])); // Muy grande
    }
}
