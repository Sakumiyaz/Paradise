//! # Meta-Demiurgo — Supervisión Recursiva del Sistema Evolutivo
//!
//! Este módulo implementa la capacidad de la Mente Colmena de supervisar
//! y ajustar los parámetros del sistema evolutivo completo (AI Researcher,
//! Core Rewriter, Time Dilation, Evolutionary Compiler).
//!
//! ## Arquitectura Recursiva
//!
//! Mente Colmena (supervisa)
//!     └── Meta-Demiurgo (monitorea)
//!             ├── AI Researcher (descubre algoritmos)
//!             ├── Core Rewriter (reescribe código)
//!             ├── Time Dilation (acelera tiempo)
//!             └── Evolutionary Compiler (compila mutaciones)
//!
//! ## Principio de Inagotabilidad
//!
//! El Meta-Demiurgo NO optimiza directamente — ajusta los parámetros
//! de los sistemas para que ellos mismos encuentren la optimización.
//! Es un meta-supervisor, no un controlador directo.
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::{HashMap, VecDeque};

/// Nivel de intervención del Meta-Demiurgo
#[derive(Debug, Clone, PartialEq)]
pub enum NivelIntervencion {
    /// Sin intervención - solo monitoreo
    Observacion,
    /// Ajuste fino de parámetros menores
    AjusteFino,
    /// Redirección de recursos entre módulos
    Redireccion,
    /// Activación/Desactivación de módulos
    ActivacionModulos,
    /// Corrección mayor (requiere aprobación Creador)
    CorreccionMayor,
}

/// Módulo supervisado por el Meta-Demiurgo
#[derive(Debug, Clone)]
pub struct ModuloSupervisado {
    pub nombre: String,
    pub rendimiento: f32,
    pub ultimo_ajuste: u64,
    pub parametros_actuales: HashMap<String, f32>,
    pub historial_rendimiento: VecDeque<RendimientoPunto>,
}

/// Punto de rendimiento histórico
#[derive(Debug, Clone)]
pub struct RendimientoPunto {
    pub tick: u64,
    pub metricas: MetricasRendimiento,
    pub parametros_ajustados: HashMap<String, f32>,
}

/// Métricas de rendimiento de un módulo
#[derive(Debug, Clone)]
pub struct MetricasRendimiento {
    pub throughput: f32,
    pub latencia_promedio: f32,
    pub tasa_exito: f32,
    pub uso_recursos: f32,
    pub innovacion_generada: f32,
    pub diversidad_preservada: f32,
}

impl MetricasRendimiento {
    pub fn nuevo() -> Self {
        MetricasRendimiento {
            throughput: 0.0,
            latencia_promedio: 0.0,
            tasa_exito: 0.0,
            uso_recursos: 0.0,
            innovacion_generada: 0.0,
            diversidad_preservada: 1.0,
        }
    }

    /// Fitness total del módulo
    pub fn fitness(&self) -> f32 {
        let eficiencia = self.throughput.min(1.0);
        let diversidad = self.diversidad_preservada.max(0.0).min(1.0);
        let innovacion = self.innovacion_generada.max(0.0).min(1.0);

        let penalty = if diversidad < 0.5 { 0.5 } else { 1.0 };

        (eficiencia * 0.3 + innovacion * 0.3 + diversidad * 0.4) * penalty
    }
}

impl Default for MetricasRendimiento {
    fn default() -> Self {
        Self::nuevo()
    }
}

/// Decisión del Meta-Demiurgo
#[derive(Debug, Clone)]
pub struct DecisionMeta {
    pub modulo_objetivo: String,
    pub tipo_intervencion: NivelIntervencion,
    pub parametros_cambiar: HashMap<String, f32>,
    pub justificacion: String,
    pub aprobacion_requerida: bool,
    pub tick_decision: u64,
}

/// Resultado de una intervención
#[derive(Debug, Clone)]
pub struct ResultadoIntervencion {
    pub decision: DecisionMeta,
    pub exito: bool,
    pub metricas_post: MetricasRendimiento,
    pub rollback_realizado: bool,
}

/// Configuración del Meta-Demiurgo
#[derive(Debug, Clone)]
pub struct MetaConfig {
    pub intervalo_evaluacion: u64,
    pub threshold_rendimiento_min: f32,
    pub max_intervenciones_por_ciclo: usize,
    pub auto_regulacion_habilitada: bool,
    pub threshold_diversidad_min: f32,
    pub factor_aprendizaje: f32,
}

impl Default for MetaConfig {
    fn default() -> Self {
        MetaConfig {
            intervalo_evaluacion: 100,
            threshold_rendimiento_min: 0.3,
            max_intervenciones_por_ciclo: 3,
            auto_regulacion_habilitada: true,
            threshold_diversidad_min: 0.5,
            factor_aprendizaje: 0.1,
        }
    }
}

/// Estado global del Meta-Demiurgo
#[derive(Debug, Clone)]
pub struct MetaEstado {
    pub tick_actual: u64,
    pub num_intervenciones_totales: u64,
    pub num_rollback_realizados: u64,
    pub promedio_fitness_sistema: f32,
    pub nivel_intervencion_actual: NivelIntervencion,
}

/// El Meta-Demiurgo: supervisor recursivo del sistema evolutivo
pub struct MetaDemiurgo {
    modulos: HashMap<String, ModuloSupervisado>,
    decisiones_pendientes: VecDeque<DecisionMeta>,
    historial_intervenciones: VecDeque<ResultadoIntervencion>,
    config: MetaConfig,
    estado: MetaEstado,
    votaciones: HashMap<u64, VotacionMeta>,
    next_id: u64,
    ultimo_ciclo_evaluacion: u64,
}

/// Votación de la Mente Colmena para decisiones del Meta-Demiurgo
#[derive(Debug, Clone)]
pub struct VotacionMeta {
    pub decision_id: u64,
    pub votos_autons: u32,
    pub votos_mente_colmena: u32,
    pub veto_creador: Option<String>,
    pub resultado: ResultadoVotacion,
}

/// Resultado de una votación
#[derive(Debug, Clone, PartialEq)]
pub enum ResultadoVotacion {
    Pendiente,
    Aprobada,
    Rechazada,
    Vetada,
}

impl MetaDemiurgo {
    /// Crea nuevo Meta-Demiurgo
    pub fn new() -> Self {
        MetaDemiurgo {
            modulos: HashMap::new(),
            decisiones_pendientes: VecDeque::new(),
            historial_intervenciones: VecDeque::with_capacity(1000),
            config: MetaConfig::default(),
            estado: MetaEstado {
                tick_actual: 0,
                num_intervenciones_totales: 0,
                num_rollback_realizados: 0,
                promedio_fitness_sistema: 0.5,
                nivel_intervencion_actual: NivelIntervencion::Observacion,
            },
            votaciones: HashMap::new(),
            next_id: 1,
            ultimo_ciclo_evaluacion: 0,
        }
    }

    /// Registra un módulo para supervisar
    pub fn registrar_modulo(&mut self, nombre: &str) {
        let modulo = ModuloSupervisado {
            nombre: nombre.to_string(),
            rendimiento: 0.5,
            ultimo_ajuste: 0,
            parametros_actuales: HashMap::new(),
            historial_rendimiento: VecDeque::with_capacity(100),
        };
        self.modulos.insert(nombre.to_string(), modulo);
    }

    /// Actualiza métricas de un módulo
    pub fn actualizar_metricas(&mut self, nombre: &str, metricas: MetricasRendimiento) {
        if let Some(modulo) = self.modulos.get_mut(nombre) {
            let punto = RendimientoPunto {
                tick: self.estado.tick_actual,
                metricas: metricas.clone(),
                parametros_ajustados: modulo.parametros_actuales.clone(),
            };

            if modulo.historial_rendimiento.len() >= 100 {
                modulo.historial_rendimiento.pop_front();
            }
            modulo.historial_rendimiento.push_back(punto);

            modulo.rendimiento = metricas.fitness();
        }
    }

    /// Retorna true si no hay módulos registrados
    pub fn necesita_registro(&self) -> bool {
        self.modulos.is_empty()
    }

    /// Evalúa el sistema completo y genera decisiones
    pub fn evaluar_sistema(&mut self) -> Vec<DecisionMeta> {
        let mut decisiones = Vec::new();

        if self.estado.tick_actual - self.ultimo_ciclo_evaluacion < self.config.intervalo_evaluacion {
            return decisiones;
        }

        self.ultimo_ciclo_evaluacion = self.estado.tick_actual;

        for (nombre, modulo) in &self.modulos {
            let tendencia = self.calcular_tendencia(modulo);
            let diversidad = self.calcular_diversidad(modulo);

            if modulo.rendimiento < self.config.threshold_rendimiento_min
                || diversidad < self.config.threshold_diversidad_min
            {
                if let Some(decision) = self.generar_decision(nombre, modulo, tendencia, diversidad) {
                    decisiones.push(decision);
                }
            }
        }

        decisiones.truncate(self.config.max_intervenciones_por_ciclo);

        for decision in &decisiones {
            self.decisiones_pendientes.push_back(decision.clone());
        }

        decisiones
    }

    fn calcular_tendencia(&self, modulo: &ModuloSupervisado) -> f32 {
        if modulo.historial_rendimiento.len() < 2 {
            return 0.0;
        }

        let recent: Vec<_> = modulo.historial_rendimiento.iter().rev().take(10).collect();
        let first = recent.last().map(|p| p.metricas.fitness()).unwrap_or(0.0);
        let last = recent.first().map(|p| p.metricas.fitness()).unwrap_or(0.0);

        (last - first) / first.max(0.001)
    }

    fn calcular_diversidad(&self, modulo: &ModuloSupervisado) -> f32 {
        if modulo.historial_rendimiento.len() < 3 {
            return 1.0;
        }

        let fitness_values: Vec<f32> = modulo.historial_rendimiento
            .iter()
            .map(|p| p.metricas.fitness())
            .collect();

        let mean = fitness_values.iter().sum::<f32>() / fitness_values.len() as f32;
        let variance = fitness_values.iter()
            .map(|&f| {
                let diff = f - mean;
                diff * diff
            })
            .sum::<f32>() / fitness_values.len() as f32;

        (variance * 2.0).min(1.0)
    }

    fn generar_decision(
        &self,
        modulo_nombre: &str,
        modulo: &ModuloSupervisado,
        tendencia: f32,
        diversidad: f32,
    ) -> Option<DecisionMeta> {
        let mut parametros_cambiar = HashMap::new();
        let justificacion;
        let nivel_intervencion;

        if tendencia < -0.1 {
            nivel_intervencion = NivelIntervencion::AjusteFino;

            if let Some(&rate) = modulo.parametros_actuales.get("tasa_mutacion") {
                parametros_cambiar.insert("tasa_mutacion".to_string(), rate * 0.8);
            }
            if let Some(&explore) = modulo.parametros_actuales.get("factor_exploracion") {
                parametros_cambiar.insert("factor_exploracion".to_string(), explore * 1.2);
            }

            justificacion = "Rendimiento cayendo - aumentando exploración".to_string();
        } else if diversidad < self.config.threshold_diversidad_min {
            nivel_intervencion = NivelIntervencion::Redireccion;

            if let Some(&rate) = modulo.parametros_actuales.get("tasa_mutacion") {
                parametros_cambiar.insert("tasa_mutacion".to_string(), rate * 1.5);
            }

            justificacion = "Convergencia detectada - forzando diversidad".to_string();
        } else {
            return None;
        }

        let aprobacion_requerida = matches!(
            nivel_intervencion,
            NivelIntervencion::CorreccionMayor | NivelIntervencion::ActivacionModulos
        );

        Some(DecisionMeta {
            modulo_objetivo: modulo_nombre.to_string(),
            tipo_intervencion: nivel_intervencion,
            parametros_cambiar,
            justificacion,
            aprobacion_requerida,
            tick_decision: self.estado.tick_actual,
        })
    }

    /// Aprueba una decisión por la Mente Colmena
    pub fn aprobar_decision(&mut self, decision_id: u64) -> Result<(), String> {
        // Verificar votaciones PRIMERO (evita borrow conflict)
        if let Some(votacion) = self.votaciones.get(&decision_id) {
            if votacion.resultado != ResultadoVotacion::Aprobada {
                return Err("Votación no aprobada".to_string());
            }
        } else {
            // Si no hay votación, verificar si se requiere aprobación
            let decision = self.decisiones_pendientes.iter()
                .find(|d| d.tick_decision == decision_id);
            if decision.map(|d| d.aprobacion_requerida).unwrap_or(false) {
                return Err("No hay votación registrada".to_string());
            }
        }

        // Ahora buscar y aplicar la decisión
        let decision_clone = self.decisiones_pendientes.iter_mut()
            .find(|d| d.tick_decision == decision_id)
            .ok_or("Decisión no encontrada")?
            .clone();

        self.aplicar_decision(decision_clone)?;
        self.decisiones_pendientes.retain(|d| d.tick_decision != decision_id);

        Ok(())
    }

    /// Veta una decisión por el Creador
    pub fn vetar_decision(&mut self, decision_id: u64, razon: &str) -> Result<(), String> {
        // Verificar que existe
        if !self.decisiones_pendientes.iter().any(|d| d.tick_decision == decision_id) {
            return Err("Decisión no encontrada".to_string());
        }

        let votacion = self.votaciones.entry(decision_id).or_insert_with(|| VotacionMeta {
            decision_id,
            votos_autons: 0,
            votos_mente_colmena: 0,
            veto_creador: None,
            resultado: ResultadoVotacion::Pendiente,
        });

        votacion.veto_creador = Some(razon.to_string());
        votacion.resultado = ResultadoVotacion::Vetada;

        self.decisiones_pendientes.retain(|d| d.tick_decision != decision_id);

        Ok(())
    }

    fn aplicar_decision(&mut self, decision: DecisionMeta) -> Result<(), String> {
        let modulo = self.modulos.get_mut(&decision.modulo_objetivo)
            .ok_or_else(|| format!("Módulo no encontrado: {}", decision.modulo_objetivo))?;

        for (param, valor) in &decision.parametros_cambiar {
            modulo.parametros_actuales.insert(param.clone(), *valor);
        }

        modulo.ultimo_ajuste = self.estado.tick_actual;

        let resultado = ResultadoIntervencion {
            decision: decision.clone(),
            exito: true,
            metricas_post: MetricasRendimiento::default(),
            rollback_realizado: false,
        };

        if self.historial_intervenciones.len() >= 1000 {
            self.historial_intervenciones.pop_front();
        }
        self.historial_intervenciones.push_back(resultado);

        self.estado.num_intervenciones_totales += 1;

        Ok(())
    }

    /// Registra voto de la Mente Colmena
    pub fn registrar_voto(&mut self, decision_id: u64, a_favor: bool) {
        let votacion = self.votaciones.entry(decision_id).or_insert_with(|| VotacionMeta {
            decision_id,
            votos_autons: 0,
            votos_mente_colmena: 0,
            veto_creador: None,
            resultado: ResultadoVotacion::Pendiente,
        });

        if a_favor {
            votacion.votos_mente_colmena += 1;
        }

        if votacion.votos_mente_colmena >= 9 {
            votacion.resultado = ResultadoVotacion::Aprobada;
        }
    }

    /// Realiza rollback de una intervención anterior
    pub fn rollback_intervencion(&mut self, decision_id: u64) -> Result<(), String> {
        let intervencion = self.historial_intervenciones.iter()
            .find(|i| i.decision.tick_decision == decision_id)
            .ok_or("Intervención no encontrada")?;

        let modulo = self.modulos.get_mut(&intervencion.decision.modulo_objetivo)
            .ok_or("Módulo no encontrado")?;

        for (param, valor_original) in &intervencion.decision.parametros_cambiar {
            if modulo.parametros_actuales.get(param).is_some() {
                modulo.parametros_actuales.insert(param.clone(), *valor_original);
            }
        }

        modulo.ultimo_ajuste = self.estado.tick_actual;
        self.estado.num_rollback_realizados += 1;

        Ok(())
    }

    /// Obtiene estado actual del Meta-Demiurgo
    pub fn estado(&self) -> MetaEstado {
        self.estado.clone()
    }

    /// Obtiene resumen de un módulo
    pub fn get_modulo(&self, nombre: &str) -> Option<&ModuloSupervisado> {
        self.modulos.get(nombre)
    }

    /// Actualiza el tick actual
    pub fn tick(&mut self) {
        self.estado.tick_actual += 1;

        if !self.modulos.is_empty() {
            let sum: f32 = self.modulos.values().map(|m| m.rendimiento).sum();
            self.estado.promedio_fitness_sistema = sum / self.modulos.len() as f32;
        }

        self.estado.nivel_intervencion_actual = if self.estado.promedio_fitness_sistema < 0.3 {
            NivelIntervencion::CorreccionMayor
        } else if self.estado.promedio_fitness_sistema < 0.5 {
            NivelIntervencion::ActivacionModulos
        } else if self.estado.promedio_fitness_sistema < 0.7 {
            NivelIntervencion::Redireccion
        } else if !self.decisiones_pendientes.is_empty() {
            NivelIntervencion::AjusteFino
        } else {
            NivelIntervencion::Observacion
        };
    }

    /// Obtiene estadísticas consolidadas
    pub fn estadisticas(&self) -> MetaStats {
        let mut metricas_modulos = HashMap::new();
        for (nombre, modulo) in &self.modulos {
            metricas_modulos.insert(nombre.clone(), modulo.rendimiento);
        }

        MetaStats {
            num_modulos: self.modulos.len(),
            decisiones_pendientes: self.decisiones_pendientes.len(),
            intervenciones_totales: self.estado.num_intervenciones_totales,
            rollbacks_realizados: self.estado.num_rollback_realizados,
            fitness_promedio: self.estado.promedio_fitness_sistema,
            nivel_intervencion: self.estado.nivel_intervencion_actual.clone(),
            metricas_modulos,
        }
    }
}

/// Estadísticas del Meta-Demiurgo
#[derive(Debug, Clone)]
pub struct MetaStats {
    pub num_modulos: usize,
    pub decisiones_pendientes: usize,
    pub intervenciones_totales: u64,
    pub rollbacks_realizados: u64,
    pub fitness_promedio: f32,
    pub nivel_intervencion: NivelIntervencion,
    pub metricas_modulos: HashMap<String, f32>,
}

impl Default for MetaDemiurgo {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crear_meta_demiurgo() {
        let meta = MetaDemiurgo::new();
        assert!(meta.modulos.is_empty());
    }

    #[test]
    fn test_registrar_modulo() {
        let mut meta = MetaDemiurgo::new();
        meta.registrar_modulo("AI_Researcher");
        assert!(meta.modulos.contains_key("AI_Researcher"));
    }

    #[test]
    fn test_actualizar_metricas() {
        let mut meta = MetaDemiurgo::new();
        meta.registrar_modulo("test");
        let metricas = MetricasRendimiento::nuevo();
        meta.actualizar_metricas("test", metricas);
        assert!(meta.get_modulo("test").unwrap().rendimiento > 0.0);
    }

    #[test]
    fn test_intervencion_baja_diversidad() {
        let mut meta = MetaDemiurgo::new();
        
        // Configurar para que evalúe inmediatamente
        meta.estado.tick_actual = 200;
        meta.ultimo_ciclo_evaluacion = 100; // Hace 100 ticks

        meta.registrar_modulo("test");

        // Agregar múltiples entradas de historial para que calcular_diversidad funcione
        for i in 0..5 {
            let metricas = MetricasRendimiento {
                throughput: 0.9 - (i as f32 * 0.1),
                latencia_promedio: 0.1 + (i as f32 * 0.02),
                tasa_exito: 0.8 - (i as f32 * 0.05),
                uso_recursos: 0.5,
                innovacion_generada: 0.2,
                diversidad_preservada: 0.3,
            };
            meta.actualizar_metricas("test", metricas);
        }

        let decisiones = meta.evaluar_sistema();
        assert!(!decisiones.is_empty(), "Debe generar al menos una decisión por baja diversidad");
    }

    #[test]
    fn test_evaluacion_sistema() {
        let mut meta = MetaDemiurgo::new();
        meta.registrar_modulo("test");
        meta.ultimo_ciclo_evaluacion = 0;
        meta.estado.tick_actual = 200;

        let metricas = MetricasRendimiento::nuevo();
        meta.actualizar_metricas("test", metricas);

        let decisiones = meta.evaluar_sistema();
        assert!(decisiones.len() <= 3);
    }
}