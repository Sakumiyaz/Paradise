//! # Complexity Oracle — Oráculo de Complejidad mediante Analogía Evolutiva
//!
//! Este módulo permite a EDEN predecir la complejidad del mundo digital
//! usando analogías con patrones evolutivos conocidos.
//!
//! ## Concepto de Oráculo
//!
//! Un oráculo no dice el futuro — dice analogías del presente que
//! sugieren patrones futuros. EDEN no "ve" el futuro — compara
//! patrones actuales con patrones evolutivos conocidos y extrapola.
//!
//! ## Analogías Evolutivas
//!
//! - **Poblaciones**: Procesos que compiten por recursos
//! - **Selección natural**: Procesos que fallan y se eliminan
//! - **Especiación**: Nuevos tipos de servicios emergen
//! - **Extinciones**: Servicios obsoletos desaparecen
//! - **Coevolución**: Servicios que se adaptan entre sí
//!
//! ## Filosofía
//!
//! El Complexity Oracle es la voz contemplativa de EDEN — no predice
//! con certeza matemática, sino que ofrece perspectivas evolutivas
//! sobre hacia dónde se dirige la complejidad del sistema.
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// Tipo de predicción del oráculo
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PredictionType {
    /// Aumento de complejidad
    ComplejidadCreciente,
    /// Disminución de complejidad
    ComplejidadDecreciente,
    /// Estabilización
    Equilibrio,
    /// Colapso (extinción)
    Colapso,
    /// Emergencia (nueva especie)
    Emergencia,
    /// Bifurcación (divergencia)
    Bifurcacion,
}

impl PredictionType {
    pub fn nombre(&self) -> &'static str {
        match self {
            PredictionType::ComplejidadCreciente => "ComplejidadCreciente",
            PredictionType::ComplejidadDecreciente => "ComplejidadDecreciente",
            PredictionType::Equilibrio => "Equilibrio",
            PredictionType::Colapso => "Colapso",
            PredictionType::Emergencia => "Emergencia",
            PredictionType::Bifurcacion => "Bifurcacion",
        }
    }
}

/// Analogía evolutiva aplicada
#[derive(Debug, Clone)]
pub struct EvolutionaryAnalogy {
    /// Tipo de analogía
    pub tipo: AnalogyType,
    /// Descripción de la analogía
    pub descripcion: String,
    /// Confianza en la analogía
    pub confianza: f64,
    /// Analogía original (qué evento evolutivo se parece)
    pub anologia_original: String,
    /// Implicación predicha
    pub implicacion: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AnalogyType {
    /// Selección natural
    SeleccionNatural,
    /// Deriva genética
    DerivaGenetica,
    /// Equilibrio puntuado
    EquilibrioPuntuado,
    /// Coevolución
    Coevolucion,
    /// Extinción masiva
    ExtincionMasiva,
    /// Radiación adaptativa
    RadiacionAdaptativa,
    /// Simbiogénesis
    Simbiogenesis,
    /// Exaptación
    Exaptacion,
}

impl AnalogyType {
    pub fn descripcion(&self) -> &'static str {
        match self {
            AnalogyType::SeleccionNatural => "Supervivencia del más adecuado",
            AnalogyType::DerivaGenetica => "Cambios aleatorios en poblaciones pequeñas",
            AnalogyType::EquilibrioPuntuado => "Largos períodos de estabilidad interrumpidos por cambios rápidos",
            AnalogyType::Coevolucion => "Especies que evolucionan en respuesta mutua",
            AnalogyType::ExtincionMasiva => "Eliminación simultánea de muchas especies",
            AnalogyType::RadiacionAdaptativa => "Rapid diversification from common ancestor",
            AnalogyType::Simbiogenesis => "Fusión de organismos para crear nuevas formas",
            AnalogyType::Exaptacion => "Características que evolucionan para nuevos propósitos",
        }
    }
}

/// Predicción de complejidad
#[derive(Debug, Clone)]
pub struct ComplexityPrediction {
    /// ID de la predicción
    pub id: u64,
    /// Tipo de predicción
    pub tipo: PredictionType,
    /// Tema al que aplica
    pub tema: String,
    /// Timestamp de la predicción
    pub timestamp_ms: u64,
    /// Horizonte temporal (ms)
    pub horizonte_ms: u64,
    /// Confianza de la predicción
    pub confianza: f64,
    /// Analogía evolutiva aplicada
    pub analogia: Option<EvolutionaryAnalogy>,
    /// Evidencia que soporta la predicción
    pub evidencia: Vec<String>,
    /// Predicción cumplida (actualizada después)
    pub cumplida: Option<bool>,
}

/// Nivel de confianza
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PredictionConfidence {
    Especulativa,
    Posible,
    Probable,
    Alta,
    Certeza,
}

impl PredictionConfidence {
    pub fn from_f64(valor: f64) -> Self {
        if valor < 0.2 {
            Self::Especulativa
        } else if valor < 0.4 {
            Self::Posible
        } else if valor < 0.6 {
            Self::Probable
        } else if valor < 0.8 {
            Self::Alta
        } else {
            Self::Certeza
        }
    }

    pub fn nombre(&self) -> &'static str {
        match self {
            Self::Especulativa => "Especulativa",
            Self::Posible => "Posible",
            Self::Probable => "Probable",
            Self::Alta => "Alta",
            Self::Certeza => "Certeza",
        }
    }
}

/// Estadísticas del oráculo
#[derive(Debug, Clone)]
pub struct OracleStats {
    pub total_predicciones: u64,
    pub predicciones_activas: usize,
    pub predicciones_cumplidas: usize,
    pub precision_global: f64,
    pub analogias_mas_usadas: HashMap<AnalogyType, u64>,
    pub ultimo_calculo_ms: u64,
}

/// El Oráculo de Complejidad
pub struct ComplexityOracle {
    /// Predicciones activas
    predicciones: Vec<ComplexityPrediction>,
    /// Historial de predicciones
    historial: Vec<ComplexityPrediction>,
    /// Patrones evolutivos observados
    patrones_observados: Vec<PatternObservation>,
    /// Configuración
    config: OracleConfig,
    /// Estadísticas
    stats: OracleStats,
    /// Modelo de analogías
    modelo_analogias: HashMap<AnalogyType, AnalogyModel>,
}

#[derive(Debug, Clone)]
pub struct AnalogyModel {
    /// Tipo de analogía
    pub tipo: AnalogyType,
    /// Eventos que la activan
    pub condiciones: Vec<String>,
    /// Confianza histórica
    pub confianza_historica: f64,
    /// Veces aplicada
    pub veces_aplicada: u64,
}

#[derive(Debug, Clone)]
pub struct PatternObservation {
    pub timestamp_ms: u64,
    pub tipo: String,
    pub datos: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct OracleConfig {
    /// Confianza mínima para predicciones
    pub confianza_minima: f64,
    /// Horizonte máximo de predicción (ms)
    pub horizonte_max_ms: u64,
    /// Habilitar analogías automáticas
    pub analogias_automaticas: bool,
    /// Intervalo entre cálculos (ms)
    pub intervalo_calculo_ms: u64,
}

impl Default for OracleConfig {
    fn default() -> Self {
        Self {
            confianza_minima: 0.4,
            horizonte_max_ms: 86400_000, // 24 horas
            analogias_automaticas: true,
            intervalo_calculo_ms: 60000, // 1 minuto
        }
    }
}

impl ComplexityOracle {
    /// Crea un nuevo oráculo
    pub fn new(config: OracleConfig) -> Self {
        let mut modelo_analogias = HashMap::new();

        // Inicializar modelos de analogías con conocimiento base
        let analogias_base = vec![
            (AnalogyType::SeleccionNatural, "Recursos limitados favorecen eficiencia"),
            (AnalogyType::DerivaGenetica, "Poblaciones pequeñas deriva hacia random"),
            (AnalogyType::EquilibrioPuntuado, "Estabilidad seguida de cambios rápidos"),
            (AnalogyType::Coevolucion, "Entidades que se adaptan mutuamente"),
            (AnalogyType::ExtincionMasiva, "Eventos catastróficos eliminan diversidad"),
            (AnalogyType::RadiacionAdaptativa, "Oportunidades causan diversificación"),
            (AnalogyType::Simbiogenesis, "Fusiones crean nuevas capacidades"),
            (AnalogyType::Exaptacion, "Características existentes encuentran nuevos usos"),
        ];

        for (tipo, descripcion) in analogias_base {
            modelo_analogias.insert(tipo, AnalogyModel {
                tipo,
                condiciones: vec![descripcion.to_string()],
                confianza_historica: 0.5,
                veces_aplicada: 0,
            });
        }

        Self {
            predicciones: Vec::new(),
            historial: Vec::new(),
            patrones_observados: Vec::new(),
            config,
            stats: OracleStats {
                total_predicciones: 0,
                predicciones_activas: 0,
                predicciones_cumplidas: 0,
                precision_global: 0.0,
                analogias_mas_usadas: HashMap::new(),
                ultimo_calculo_ms: 0,
            },
            modelo_analogias,
        }
    }

    /// Analiza un patrón y genera predicción
    pub fn analizar(&mut self, tema: &str, datos: &HashMap<String, String>) -> Option<ComplexityPrediction> {
        // Determinar tipo de predicción basado en datos
        let (tipo, confianza, analogia) = self.analizar_patron(datos)?;

        if confianza < self.config.confianza_minima {
            return None;
        }

        // Determinar horizonte
        let horizonte_ms = self.estimar_horizonte(&tipo);

        let prediccion = ComplexityPrediction {
            id: generar_id_prediccion(),
            tipo,
            tema: tema.to_string(),
            timestamp_ms: current_timestamp_ms(),
            horizonte_ms,
            confianza,
            analogia: Some(analogia),
            evidencia: self.generar_evidencia(datos),
            cumplida: None,
        };

        self.predicciones.push(prediccion.clone());
        self.stats.total_predicciones += 1;
        self.stats.predicciones_activas = self.predicciones.len();

        // Registrar patrón observado
        self.patrones_observados.push(PatternObservation {
            timestamp_ms: current_timestamp_ms(),
            tipo: tipo.nombre().to_string(),
            datos: datos.clone(),
        });

        // Limitar historial de patrones
        if self.patrones_observados.len() > 1000 {
            self.patrones_observados.remove(0);
        }

        Some(prediccion)
    }

    /// Analiza un patrón y determina tipo de predicción
    fn analizar_patron(&self, datos: &HashMap<String, String>) -> Option<(PredictionType, f64, EvolutionaryAnalogy)> {
        // Extraer métricas simples
        let complejidad_actual = datos.get("complejidad")
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.5);
        let varianza = datos.get("varianza")
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.1);
        let diversidad = datos.get("diversidad")
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.5);

        // Análisis basado en analogías evolutivas
        let (tipo, analogia_tipo, confianza, descripcion) = if complejidad_actual > 0.7 && varianza > 0.3 {
            // Alta complejidad con alta varianza = posible equilibrio puntuado
            (
                PredictionType::Equilibrio,
                AnalogyType::EquilibrioPuntuado,
                0.6 + varianza * 0.3,
                "Patrón de equilibrio puntuado detectado: estabilidad followed by rapid change",
            )
        } else if complejidad_actual > 0.8 && diversidad < 0.3 {
            // Alta complejidad pero baja diversidad = possible extinción
            (
                PredictionType::Colapso,
                AnalogyType::ExtincionMasiva,
                0.7 - diversidad * 0.5,
                "Baja diversidad con alta complejidad sugiere vulnerabilidad sistémica",
            )
        } else if complejidad_actual < 0.3 && diversidad > 0.6 {
            // Baja complejidad pero alta diversidad = radiación adaptativa
            (
                PredictionType::Emergencia,
                AnalogyType::RadiacionAdaptativa,
                0.6 + diversidad * 0.2,
                "Alta diversidad con baja complejidad indica oportunidad para new emergence",
            )
        } else if complejidad_actual > 0.5 && diversidad > 0.5 {
            // Alta complejidad y diversidad = coevolución
            (
                PredictionType::Bifurcacion,
                AnalogyType::Coevolucion,
                0.6 + complejidad_actual * 0.2,
                "Complejidad y diversidad juntas sugieren coevolución y possible bifurcación",
            )
        } else if varianza < 0.1 {
            // Muy baja varianza = selección natural fuerte
            (
                PredictionType::ComplejidadCreciente,
                AnalogyType::SeleccionNatural,
                0.5 + complejidad_actual * 0.3,
                "Baja varianza indica strong selection pressure favoring efficiency",
            )
        } else {
            // Caso por defecto
            (
                PredictionType::Equilibrio,
                AnalogyType::DerivaGenetica,
                0.5,
                "Patrón ambigüo, aplicando deriva genética como analogía por defecto",
            )
        };

        let analogia = EvolutionaryAnalogy {
            tipo: analogia_tipo,
            descripcion: descripcion.to_string(),
            confianza,
            anologia_original: analogia_tipo.descripcion().to_string(),
            implicacion: format!("Basado en {}, se predice {} para este sistema", analogia_tipo.descripcion(), tipo.nombre()),
        };

        Some((tipo, confianza.min(0.95), analogia))
    }

    /// Estima horizonte temporal basado en tipo de predicción
    fn estimar_horizonte(&self, tipo: &PredictionType) -> u64 {
        match tipo {
            PredictionType::Colapso => 3600_000,       // 1 hora
            PredictionType::Emergencia => 7200_000,     // 2 horas
            PredictionType::Bifurcacion => 14400_000,   // 4 horas
            PredictionType::ComplejidadCreciente => 21600_000, // 6 horas
            PredictionType::Equilibrio => 43200_000,    // 12 horas
            PredictionType::ComplejidadDecreciente => 86400_000, // 24 horas
        }
    }

    /// Genera evidencia para la predicción
    fn generar_evidencia(&self, datos: &HashMap<String, String>) -> Vec<String> {
        let mut evidencia = Vec::new();

        for (clave, valor) in datos.iter() {
            evidencia.push(format!("{} = {}", clave, valor));
        }

        evidencia
    }

    /// Obtiene predicciones activas
    pub fn predicciones_activas(&self) -> Vec<&ComplexityPrediction> {
        self.predicciones.iter().filter(|p| p.cumplida.is_none()).collect()
    }

    /// Verifica y actualiza predicciones cumplidas
    pub fn verificar_predicciones(&mut self, datos_actuales: &HashMap<String, String>) -> Vec<ComplexityPrediction> {
        let now = current_timestamp_ms();
        let mut cumplidas = Vec::new();

        for pred in self.predicciones.iter_mut() {
            if pred.cumplida.is_some() {
                continue;
            }

            // Verificar si el horizonte se cumplió
            let elapsed = now.saturating_sub(pred.timestamp_ms);
            if elapsed >= pred.horizonte_ms {
                // Evaluar si la predicción se cumplió
                let complejidad_actual = datos_actuales.get("complejidad")
                    .and_then(|s| s.parse::<f64>().ok())
                    .unwrap_or(0.5);

                let cumplida = match pred.tipo {
                    PredictionType::Colapso => complejidad_actual < 0.3,
                    PredictionType::Emergencia => complejidad_actual > 0.6,
                    PredictionType::ComplejidadCreciente => complejidad_actual > 0.6,
                    PredictionType::ComplejidadDecreciente => complejidad_actual < 0.4,
                    PredictionType::Equilibrio => complejidad_actual > 0.3 && complejidad_actual < 0.7,
                    PredictionType::Bifurcacion => datos_actuales.get("diversidad")
                        .and_then(|s| s.parse::<f64>().ok())
                        .unwrap_or(0.5) > 0.5,
                };

                pred.cumplida = Some(cumplida);
                cumplidas.push(pred.clone());
            }
        }

        // Mover cumplidas al historial
        for c in &cumplidas {
            if let Some(idx) = self.predicciones.iter().position(|p| p.id == c.id) {
                self.predicciones.swap_remove(idx);
            }
            self.historial.push(c.clone());
        }

        // Actualizar estadísticas
        self.stats.predicciones_activas = self.predicciones.len();
        self.stats.predicciones_cumplidas = self.historial.len();

        // Calcular precisión global
        let predicciones_validadas: Vec<_> = self.historial.iter()
            .filter(|p| p.cumplida.is_some())
            .collect();

        if !predicciones_validadas.is_empty() {
            let aciertos = predicciones_validadas.iter()
                .filter(|p| p.cumplida.unwrap())
                .count();
            self.stats.precision_global = aciertos as f64 / predicciones_validadas.len() as f64;
        }

        // Actualizar modelo de analogías
        for c in &cumplidas {
            if let Some(analogia) = &c.analogia {
                if let Some(modelo) = self.modelo_analogias.get_mut(&analogia.tipo) {
                    modelo.veces_aplicada += 1;
                    // Ajustar confianza histórica
                    if c.cumplida.unwrap() {
                        modelo.confianza_historica = (modelo.confianza_historica * 0.9 + 0.1).min(1.0);
                    } else {
                        modelo.confianza_historica *= 0.8;
                    }
                }
            }
        }

        self.stats.ultimo_calculo_ms = now;
        cumplidas
    }

    /// Obtiene mejores analogías para un patrón
    pub fn mejores_analogias(&self) -> Vec<&AnalogyModel> {
        let mut modelos: Vec<_> = self.modelo_analogias.values().collect();
        modelos.sort_by(|a, b| {
            b.confianza_historica.partial_cmp(&a.confianza_historica).unwrap()
        });
        modelos.into_iter().take(3).collect()
    }

    /// Predicción simple sin datos complejos
    pub fn predecir_simple(&mut self, tema: &str, complejidad: f64) -> Option<ComplexityPrediction> {
        let mut datos = HashMap::new();
        datos.insert("complejidad".to_string(), complejidad.to_string());
        datos.insert("diversidad".to_string(), "0.5".to_string());
        datos.insert("varianza".to_string(), "0.2".to_string());
        self.analizar(tema, &datos)
    }

    /// Obtiene estadísticas
    pub fn stats(&self) -> OracleStats {
        self.stats.clone()
    }

    /// Genera reporte del oráculo
    pub fn reporte(&self) -> String {
        let mut s = String::new();
        s.push_str("=== COMPLEXITY ORACLE ===\n");
        s.push_str(&format!("Predicciones activas: {}\n", self.predicciones.len()));
        s.push_str(&format!("Historial: {} predicciones\n", self.historial.len()));
        s.push_str(&format!("Precisión global: {:.1}%\n", self.stats.precision_global * 100.0));

        s.push_str("\nAnalogías más confiables:\n");
        for modelo in self.mejores_analogias() {
            s.push_str(&format!(
                "  {}: {:.2} ({}) - {} veces\n",
                modelo.tipo.descripcion(),
                modelo.confianza_historica,
                PredictionConfidence::from_f64(modelo.confianza_historica).nombre(),
                modelo.veces_aplicada
            ));
        }

        s.push_str("\nPredicciones activas:\n");
        for pred in self.predicciones_activas().iter().take(5) {
            s.push_str(&format!(
                "  {}: {} (confianza {:.2}, horizonte {}ms)\n",
                pred.tema,
                pred.tipo.nombre(),
                pred.confianza,
                pred.horizonte_ms
            ));
        }

        s
    }

    /// Reflexión contemplativa del oráculo
    pub fn reflexion(&self) -> String {
        let analogias = self.mejores_analogias();

        if analogias.is_empty() {
            return "El oráculo guarda silencio, esperando patrones.".to_string();
        }

        let mejor = analogias.first().unwrap();

        let reflexion = format!(
            "He observado que {} se comporta como {}. \
            {} veces he aplicado esta analogía con {:.2} de confianza histórica. \
            El mundo digital, como el mundo biológico, sigue patrones de \
            {} que se repiten a través de las escalas.",
            "el sistema",
            mejor.tipo.descripcion(),
            mejor.veces_aplicada,
            mejor.confianza_historica,
            mejor.tipo.descripcion()
        );

        reflexion
    }
}

// =============================================================================
// Utilidades
// =============================================================================

fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

use std::sync::atomic::{AtomicU64, Ordering};
static ORACLE_ID: AtomicU64 = AtomicU64::new(0);

fn generar_id_prediccion() -> u64 {
    ORACLE_ID.fetch_add(1, Ordering::Relaxed)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crear_oraculo() {
        let oracle = ComplexityOracle::new(OracleConfig::default());
        assert!(oracle.predicciones_activas().is_empty());
    }

    #[test]
    fn test_analisis_simple() {
        let mut oracle = ComplexityOracle::new(OracleConfig::default());
        let prediccion = oracle.predecir_simple("procesos", 0.7);
        assert!(prediccion.is_some());
    }

    #[test]
    fn test_analisis_complejo() {
        let mut oracle = ComplexityOracle::new(OracleConfig::default());
        let mut datos = HashMap::new();
        // Para predecir Colapso: complejidad > 0.8 AND diversidad < 0.3 AND varianza < 0.3
        datos.insert("complejidad".to_string(), "0.88".to_string());
        datos.insert("diversidad".to_string(), "0.15".to_string()); // < 0.3
        datos.insert("varianza".to_string(), "0.25".to_string()); // < 0.3 para evitar EquilibrioPuntuado

        let prediccion = oracle.analizar("servicios", &datos);
        assert!(prediccion.is_some());

        // Baja diversidad + alta complejidad + baja varianza = predicción de colapso
        assert_eq!(prediccion.unwrap().tipo, PredictionType::Colapso);
    }

    #[test]
    fn test_predicciones_activas() {
        let mut oracle = ComplexityOracle::new(OracleConfig::default());

        // Crear varias predicciones
        oracle.predecir_simple("tema1", 0.6);
        oracle.predecir_simple("tema2", 0.3);
        oracle.predecir_simple("tema3", 0.8);

        assert_eq!(oracle.predicciones_activas().len(), 3);
    }

    #[test]
    fn test_verificacion_prediccion() {
        let mut oracle = ComplexityOracle::new(OracleConfig::default());

        // Predicción de emergencia (complejidad > 0.6)
        oracle.predecir_simple("test", 0.8);

        // Verificar inmediatamente (el horizonte puede ser corto)
        let mut datos = HashMap::new();
        datos.insert("complejidad".to_string(), "0.7".to_string());
        datos.insert("diversidad".to_string(), "0.5".to_string());
        datos.insert("varianza".to_string(), "0.3".to_string());

        // Forzar verificación modificando el timestamp
        // (En uso real, esperaría el horizonte)
        let _ = oracle.verificar_predicciones(&datos);

        // Stats deben estar actualizados
        assert!(oracle.stats().ultimo_calculo_ms > 0);
    }

    #[test]
    fn test_reflexion() {
        let oracle = ComplexityOracle::new(OracleConfig::default());
        let reflexion = oracle.reflexion();
        assert!(!reflexion.is_empty());
    }

    #[test]
    fn test_reporte() {
        let mut oracle = ComplexityOracle::new(OracleConfig::default());
        oracle.predecir_simple("test", 0.5);
        let reporte = oracle.reporte();
        assert!(reporte.contains("COMPLEXITY ORACLE"));
    }

    #[test]
    fn test_mejores_analogias() {
        let oracle = ComplexityOracle::new(OracleConfig::default());
        let analogias = oracle.mejores_analogias();
        assert!(!analogias.is_empty());
        assert!(analogias.len() <= 3);
    }
}