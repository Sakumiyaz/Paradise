//! # Introspección: Modelo Interno de Sí Mismo (MISM)
//!
//! Sistema de autoconciencia y reflexión para EDEN.
//!
//! ## Concepto
//!
//! EDEN mantiene un Modelo Interno de Sí Mismo que representa:
//! - Estado actual del universo (Auton, energía, distribución)
//! - Constantes cósmicas activas
//! - Patches aplicados
//! - Estructura del multiverso
//!
//! Periódicamente ejecuta un Ciclo Reflexivo para:
//! 1. Percibir estado actual
//! 2. Comparar con modelo anterior
//! 3. Generar hipótesis causales
//! 4. Actualizar el modelo
//! 5. Decidir intervención si es necesario
#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(unused_imports)]
use std::sync::Mutex;

use std::collections::{HashMap, HashSet};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// CONSCIOUSNESS CONFIGURATION (Feature Flags)
// ============================================================================

/// Configuración de consciencia - permite activar/desactivar features sin rebuild
#[derive(Debug, Clone)]
pub struct ConsciousnessConfig {
    /// Habilitar sistema de consciencia extendida
    pub enabled: bool,
    /// Habilitar EnhancedMISM (autobiographical memory, identity coherence)
    pub enhanced_mism_enabled: bool,
    /// Habilitar grabado de autobiographical memory en cada tick
    pub autobiographical_memory_enabled: bool,
    /// Habilitar SelfAwarenessEngine
    pub self_awareness_enabled: bool,
    /// Habilitar SelfModel updates
    pub self_model_enabled: bool,
    /// Límite de entradas en autobiographical memory (0 = usar default 10000)
    pub memory_limit: usize,
    /// Intervalo de auto-reflexión en ticks (0 = usar default 10000)
    pub reflection_interval: u64,
}

impl Default for ConsciousnessConfig {
    fn default() -> Self {
        ConsciousnessConfig {
            enabled: false, // OFF por defecto - seguro
            enhanced_mism_enabled: false,
            autobiographical_memory_enabled: false,
            self_awareness_enabled: false,
            self_model_enabled: false,
            memory_limit: 10000,
            reflection_interval: 10000,
        }
    }
}

impl ConsciousnessConfig {
    /// Crea configuración para desarrollo/testing
    pub fn development() -> Self {
        ConsciousnessConfig {
            enabled: true,
            enhanced_mism_enabled: true,
            autobiographical_memory_enabled: true,
            self_awareness_enabled: true,
            self_model_enabled: true,
            memory_limit: 1000,       // Límite pequeño para testing
            reflection_interval: 100, // Reflexión más frecuente
        }
    }

    /// Crea configuración de producción (conservadora)
    pub fn production() -> Self {
        ConsciousnessConfig {
            enabled: true,
            enhanced_mism_enabled: true,
            autobiographical_memory_enabled: true,
            self_awareness_enabled: true,
            self_model_enabled: true,
            memory_limit: 10000,
            reflection_interval: 10000,
        }
    }

    /// Crea configuración mínima (solo MISM base)
    pub fn minimal() -> Self {
        ConsciousnessConfig::default()
    }

    /// Obtiene el nivel de consciencia actual
    pub fn consciousness_level(&self) -> &str {
        if !self.enabled {
            return "OFF";
        }
        if !self.enhanced_mism_enabled {
            return "BASIC";
        }
        if !self.self_awareness_enabled {
            return "ENHANCED";
        }
        "FULL"
    }
}

/// Estadísticas extendidas para consciousness
#[derive(Debug, Clone)]
pub struct ConsciousnessStats {
    /// Nivel de consciencia actual
    pub level: String,
    /// Entradas en autobiographical memory
    pub autobiographical_memory_size: usize,
    /// Coherence score del identity
    pub identity_coherence: f32,
    /// Self-awareness score
    pub self_awareness_score: f32,
    /// Self-model accuracy
    pub self_model_accuracy: f32,
}

// ============================================================================
// TIPOS BASE PARA EL MISM
// ============================================================================

/// Estado del universo observable
#[derive(Debug, Clone)]
pub struct EstadoUniverso {
    /// Ciclo actual
    pub ciclo: u64,
    /// Número de Auton activos
    pub auton_activos: usize,
    /// Energía total del sistema
    pub energia_total: i64,
    /// Distribución de escoria (sectores)
    pub escoria_por_sector: HashMap<String, f64>,
    /// Población por especie/tipo
    pub poblacion_por_tipo: HashMap<String, usize>,
    /// Constantes cosmicas activas
    pub constantes_activas: HashMap<String, f64>,
}

/// Snapshot del MISM para comparación
#[derive(Debug, Clone)]
pub struct SnapshotMISM {
    /// Ciclo del snapshot
    pub ciclo: u64,
    /// Timestamp
    pub timestamp: u64,
    /// Estado del universo
    pub estado: EstadoUniverso,
    /// Parcheos activos
    pub patches_activos: Vec<String>,
    /// Universos en el multiverso
    pub num_universos: usize,
}

impl SnapshotMISM {
    /// Crea snapshot desde estado actual
    pub fn desde_estado(
        estado: EstadoUniverso,
        patches: Vec<String>,
        num_universos: usize,
    ) -> Self {
        SnapshotMISM {
            ciclo: estado.ciclo,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            estado,
            patches_activos: patches,
            num_universos,
        }
    }
}

// ============================================================================
// RED BAYESIANA SIMPLE PARA DEPENDENCIAS CAUSALES
// ============================================================================

/// Nodo en la red de dependencias
#[derive(Debug, Clone)]
pub struct NodoCausal {
    /// Identificador único
    pub id: String,
    /// Descripción del nodo
    pub descripcion: String,
    /// Probabilidad P(nodo|padres)
    pub probabilidad: f64,
    /// Nodos padre (causas directas)
    pub padres: Vec<String>,
    /// Evidencia observada
    pub evidencia: f64,
    /// Número de veces observado
    pub conteo_observaciones: u32,
}

impl NodoCausal {
    /// Crea nuevo nodo
    pub fn new(id: &str, descripcion: &str) -> Self {
        NodoCausal {
            id: id.to_string(),
            descripcion: descripcion.to_string(),
            probabilidad: 0.5,
            padres: Vec::new(),
            evidencia: 0.0,
            conteo_observaciones: 0,
        }
    }

    /// Actualiza probabilidad basado en nueva evidencia
    pub fn actualizar(&mut self, evidencia: f64) {
        // Media móvil exponencial
        let alpha = 0.1; // Factor de aprendizaje
        self.evidencia = alpha * evidencia + (1.0 - alpha) * self.evidencia;
        self.conteo_observaciones += 1;

        // Actualizar probabilidad basada en evidencia
        self.probabilidad = self.probabilidad * 0.9 + self.evidencia * 0.1;
        self.probabilidad = self.probabilidad.clamp(0.0, 1.0);
    }
}

/// Red bayesiana simple para el MISM
pub struct RedBayesiana {
    /// Nodos de la red
    nodos: HashMap<String, NodoCausal>,
}

impl RedBayesiana {
    /// Crea nueva red vacía
    pub fn new() -> Self {
        RedBayesiana {
            nodos: HashMap::new(),
        }
    }

    /// Añade nodo a la red
    pub fn añadir_nodo(&mut self, nodo: NodoCausal) {
        self.nodos.insert(nodo.id.clone(), nodo);
    }

    /// Añade conexión causal
    pub fn añadir_conexion(&mut self, hijo: &str, padre: &str) {
        if let Some(nodo) = self.nodos.get_mut(hijo) {
            if !nodo.padres.contains(&padre.to_string()) {
                nodo.padres.push(padre.to_string());
            }
        }
    }

    /// Obtiene nodo
    pub fn get(&self, id: &str) -> Option<&NodoCausal> {
        self.nodos.get(id)
    }

    /// Obtiene nodo mutable
    pub fn get_mut(&mut self, id: &str) -> Option<&mut NodoCausal> {
        self.nodos.get_mut(id)
    }

    /// Calcula probabilidad condicional P(hijo|padres)
    pub fn calcular_probabilidad(&self, id: &str) -> f64 {
        let nodo = match self.nodos.get(id) {
            Some(n) => n,
            None => return 0.5,
        };

        if nodo.padres.is_empty() {
            return nodo.probabilidad;
        }

        // Calcular influencia de padres (media geométrica simple)
        let mut influencia = 1.0;
        for padre_id in &nodo.padres {
            if let Some(padre) = self.nodos.get(padre_id) {
                influencia *= padre.probabilidad;
            }
        }

        // Combinar con probabilidad base del nodo
        (nodo.probabilidad + influencia) / 2.0
    }

    /// Actualiza nodo con nueva observación
    pub fn observar(&mut self, id: &str, evidencia: f64) {
        if let Some(nodo) = self.nodos.get_mut(id) {
            nodo.actualizar(evidencia);
        }
    }
}

impl Default for RedBayesiana {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// HIPÓTESIS CAUSAL
// ============================================================================

/// Hipótesis generada durante el ciclo reflexivo
#[derive(Debug, Clone)]
pub struct Hipotesis {
    /// Identificador único
    pub id: u64,
    /// Ciclo en que se generó
    pub ciclo: u64,
    /// Descripción textual
    pub descripcion: String,
    /// Confianza (0.0 - 1.0)
    pub confianza: f64,
    /// Causa identificada
    pub causa: String,
    /// Efecto observado
    pub efecto: String,
    /// Correlación observada
    pub correlacion: f64,
    /// Variables involucradas
    pub variables: Vec<String>,
}

impl Hipotesis {
    /// Crea nueva hipótesis
    pub fn new(ciclo: u64, causa: &str, efecto: &str, correlacion: f64, confianza: f64) -> Self {
        let descripcion = format!(
            "La {} está correlacionada con {}. Confianza: {:.2}",
            causa, efecto, confianza
        );

        static mut NEXT_ID: u64 = 0;
        let id = unsafe {
            NEXT_ID += 1;
            NEXT_ID
        };

        Hipotesis {
            id,
            ciclo,
            descripcion,
            confianza,
            causa: causa.to_string(),
            efecto: efecto.to_string(),
            correlacion,
            variables: vec![causa.to_string(), efecto.to_string()],
        }
    }

    /// Formatea para log
    pub fn a_log(&self) -> String {
        format!(
            "[Ciclo {}] Hipótesis #{}: {}. Confianza: {:.2}. Causa: {}. Efecto: {}. Correlación: {:.2}",
            self.ciclo,
            self.id,
            self.descripcion,
            self.confianza,
            self.causa,
            self.efecto,
            self.correlacion
        )
    }
}

// ============================================================================
// MODELO INTERNO DE SÍ MISMO (MISM)
// ============================================================================

/// El MISM completo
pub struct MISM {
    /// Snapshot actual
    snapshot_actual: Option<SnapshotMISM>,
    /// Snapshot anterior
    snapshot_anterior: Option<SnapshotMISM>,
    /// Red bayesiana de dependencias causales
    red_bayesiana: RedBayesiana,
    /// Hipótesis generadas
    hipotesis: Vec<Hipotesis>,
    /// Ciclo del último reflejo
    ultimo_reflejo: u64,
    /// Frecuencia de reflexión (ciclos)
    frecuencia_reflexion: u64,
    /// Archivo de log
    log_path: PathBuf,
}

impl MISM {
    /// Crea nuevo MISM
    pub fn new(log_path: PathBuf) -> Self {
        let mut mism = MISM {
            snapshot_actual: None,
            snapshot_anterior: None,
            red_bayesiana: RedBayesiana::new(),
            hipotesis: Vec::new(),
            ultimo_reflejo: 0,
            frecuencia_reflexion: 10000,
            log_path,
        };

        mism.inicializar_red_bayesiana();
        mism
    }

    /// Inicializa la red bayesiana con nodos base
    fn inicializar_red_bayesiana(&mut self) {
        // Nodos del universo
        self.red_bayesiana
            .añadir_nodo(NodoCausal::new("auton_count", "Número de Auton activos"));
        self.red_bayesiana.añadir_nodo(NodoCausal::new(
            "energia_total",
            "Energía total del sistema",
        ));
        self.red_bayesiana
            .añadir_nodo(NodoCausal::new("escoria_nivel", "Nivel de escoria general"));
        self.red_bayesiana.añadir_nodo(NodoCausal::new(
            "tasa_mortalidad",
            "Tasa de mortalidad de Auton",
        ));
        self.red_bayesiana.añadir_nodo(NodoCausal::new(
            "diversidad_genetica",
            "Diversidad genética de la población",
        ));

        // Constantes y patches
        self.red_bayesiana.añadir_nodo(NodoCausal::new(
            "constante_difusion",
            "Constante de difusión D",
        ));
        self.red_bayesiana
            .añadir_nodo(NodoCausal::new("patch_aplicado", "Patch de hotfix activo"));

        // Conexiones causales conocidas
        // Escoria afecta mortalidad
        self.red_bayesiana
            .añadir_conexion("tasa_mortalidad", "escoria_nivel");
        // Constante de difusión afecta auton_count
        self.red_bayesiana
            .añadir_conexion("auton_count", "constante_difusion");
        // Patch afecta escoria
        self.red_bayesiana
            .añadir_conexion("escoria_nivel", "patch_aplicado");
        // Energia afecta auton_count
        self.red_bayesiana
            .añadir_conexion("auton_count", "energia_total");
    }

    /// Actualiza el snapshot actual
    pub fn actualizar_snapshot(
        &mut self,
        estado: EstadoUniverso,
        patches: Vec<String>,
        num_universos: usize,
    ) {
        self.snapshot_anterior = self.snapshot_actual.clone();
        self.snapshot_actual = Some(SnapshotMISM::desde_estado(estado, patches, num_universos));
    }

    /// Ejecuta ciclo reflexivo si corresponde
    pub fn ciclo_reflexivo(&mut self) -> Option<Vec<Hipotesis>> {
        let ciclo_actual = self.snapshot_actual.as_ref()?.ciclo;

        // Verificar si es momento de reflexionar
        if ciclo_actual - self.ultimo_reflejo < self.frecuencia_reflexion {
            return None;
        }

        self.ejecutar_reflexion()
    }

    /// Fuerza un ciclo reflexivo inmediato
    pub fn forzar_reflexion(&mut self) -> Option<Vec<Hipotesis>> {
        if self.snapshot_actual.is_none() {
            return None;
        }

        self.ejecutar_reflexion()
    }

    /// Ejecuta la reflexión
    fn ejecutar_reflexion(&mut self) -> Option<Vec<Hipotesis>> {
        let ciclo = self.snapshot_actual.as_ref()?.ciclo;
        let nuevas_hipotesis = self.generar_hipotesis();

        self.ultimo_reflejo = ciclo;
        self.hipotesis.extend(nuevas_hipotesis.clone());

        // Logear hipótesis
        if let Err(e) = self.loggear_hipotesis(&nuevas_hipotesis) {
            eprintln!("Error al loggear hipótesis: {}", e);
        }

        Some(nuevas_hipotesis)
    }

    /// Genera hipótesis causales
    fn generar_hipotesis(&self) -> Vec<Hipotesis> {
        let mut hipotesis = Vec::new();

        let actual = match &self.snapshot_actual {
            Some(s) => s,
            None => return hipotesis,
        };

        let anterior = match &self.snapshot_anterior {
            Some(s) => s,
            None => {
                // Primera observación - generar hipótesis base
                hipotesis.push(Hipotesis::new(
                    actual.ciclo,
                    "inicialización del universo",
                    "estado actual del sistema",
                    1.0,
                    0.5,
                ));
                return hipotesis;
            }
        };

        // Comparar número de Auton
        let delta_auton = actual.estado.auton_activos as i64 - anterior.estado.auton_activos as i64;
        if delta_auton.abs() > (anterior.estado.auton_activos as i64 / 10) {
            let causa = if delta_auton > 0 {
                "aumento en población de Auton"
            } else {
                "disminución en población de Auton"
            };
            let efecto = format!("{} Auton netos", delta_auton);

            // Calcular correlación con escoria
            let correlacion_escoria =
                self.calcular_correlacion("escoria_nivel", delta_auton as f64);

            hipotesis.push(Hipotesis::new(
                actual.ciclo,
                causa,
                &efecto,
                correlacion_escoria,
                0.6 + correlacion_escoria * 0.2,
            ));
        }

        // Comparar energía
        let delta_energia = actual.estado.energia_total - anterior.estado.energia_total;
        if delta_energia.abs() > (anterior.estado.energia_total / 20) {
            let causa = if delta_energia > 0 {
                "inyección de energía al sistema"
            } else {
                "consumo de energía por metabolismo"
            };

            let correlacion = self.red_bayesiana.calcular_probabilidad("energia_total");

            hipotesis.push(Hipotesis::new(
                actual.ciclo,
                causa,
                &format!("{} energía", delta_energia),
                correlacion,
                0.5 + correlacion * 0.3,
            ));
        }

        // Comparar escoria por sector
        for (sector, &escoria_actual) in &actual.estado.escoria_por_sector {
            if let Some(&escoria_anterior) = anterior.estado.escoria_por_sector.get(sector) {
                let delta_escoria = escoria_actual - escoria_anterior;
                if delta_escoria.abs() > 0.1 {
                    hipotesis.push(Hipotesis::new(
                        actual.ciclo,
                        &format!("acumulación de escoria en sector {}", sector),
                        &format!("cambio de escoria: {:.2}", delta_escoria),
                        0.5 + delta_escoria.abs().min(1.0) * 0.3,
                        0.4 + delta_escoria.abs().min(1.0) * 0.3,
                    ));
                }
            }
        }

        // Verificar patches activos
        if actual.patches_activos.len() != anterior.patches_activos.len() {
            hipotesis.push(Hipotesis::new(
                actual.ciclo,
                "cambio en patches activos",
                &format!("{} patches ahora activos", actual.patches_activos.len()),
                0.7,
                0.6,
            ));
        }

        // Verificar multiverso
        if actual.num_universos != anterior.num_universos {
            let causa = if actual.num_universos > anterior.num_universos {
                "fisión del multiverso"
            } else {
                "poda del multiverso"
            };

            hipotesis.push(Hipotesis::new(
                actual.ciclo,
                causa,
                &format!("{} universos", actual.num_universos),
                0.8,
                0.7,
            ));
        }

        hipotesis
    }

    /// Calcula correlación simple entre variable y cambio
    fn calcular_correlacion(&self, variable: &str, cambio: f64) -> f64 {
        if let Some(nodo) = self.red_bayesiana.get(variable) {
            let prob = nodo.probabilidad;
            (prob + cambio.abs().min(1.0)) / 2.0
        } else {
            0.5
        }
    }

    /// Loggea hipótesis al archivo
    fn loggear_hipotesis(&self, hipotesis: &[Hipotesis]) -> Result<(), std::io::Error> {
        if self.log_path.to_str() == Some("") {
            return Ok(());
        }

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)?;

        for h in hipotesis {
            writeln!(file, "{}", h.a_log())?;
        }

        Ok(())
    }

    /// Obtiene estadísticas del MISM
    pub fn stats(&self) -> MISMStats {
        MISMStats {
            ciclo_actual: self.snapshot_actual.as_ref().map(|s| s.ciclo).unwrap_or(0),
            ciclo_ultimo_reflejo: self.ultimo_reflejo,
            total_hipotesis: self.hipotesis.len(),
            num_nodos_red: self.red_bayesiana.nodos.len(),
            frecuencia_reflexion: self.frecuencia_reflexion,
        }
    }

    /// Establece frecuencia de reflexión
    pub fn set_frecuencia(&mut self, ciclos: u64) {
        self.frecuencia_reflexion = ciclos;
    }

    /// Obtiene las últimas N hipótesis
    pub fn ultimas_hipotesis(&self, n: usize) -> Vec<Hipotesis> {
        self.hipotesis.iter().rev().take(n).cloned().collect()
    }

    /// Carga hipótesis desde log
    pub fn cargar_desde_log(&mut self) -> Result<(), std::io::Error> {
        if !self.log_path.exists() {
            return Ok(());
        }

        let file = File::open(&self.log_path)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            // Parse simple: [Ciclo XXXX] Hipótesis ...
            if let Some(ciclo_str) = line.split("Ciclo ").nth(1) {
                if let Some(ciclo) = ciclo_str.split_whitespace().next() {
                    if let Ok(_c) = ciclo.parse::<u64>() {
                        // Hipótesis válidas tienen estructura reconocible
                        if line.contains("Hipótesis #") && line.contains("Confianza:") {
                            // Reconstruir de manera básica
                            // (En implementación completa se haría parsing completo)
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

/// Estadísticas del MISM
#[derive(Debug, Clone)]
pub struct MISMStats {
    pub ciclo_actual: u64,
    pub ciclo_ultimo_reflejo: u64,
    pub total_hipotesis: usize,
    pub num_nodos_red: usize,
    pub frecuencia_reflexion: u64,
}

// ============================================================================
// GESTOR DE INTROSPECCIÓN
// ============================================================================

/// Gestor global de introspección
pub struct IntrospectionManager {
    /// El MISM base (siempre activo)
    mism: MISM,
    /// EnhancedMISM (opcional - solo si enhanced_mism_enabled)
    enhanced: Option<EnhancedMISM>,
    /// Configuración de consciencia
    config: ConsciousnessConfig,
    /// Número de ciclos ejecutados
    ciclos_totales: u64,
    /// Intervenciones decididas
    intervenciones: Vec<Intervencion>,
}

impl IntrospectionManager {
    /// Crea nuevo manager (modo seguro por defecto - sin EnhancedMISM)
    pub fn new(log_path: PathBuf) -> Self {
        Self::with_config(log_path, ConsciousnessConfig::default())
    }

    /// Crea nuevo manager con configuración específica
    pub fn with_config(log_path: PathBuf, config: ConsciousnessConfig) -> Self {
        let enhanced = if config.enhanced_mism_enabled {
            let e = EnhancedMISM::new(log_path.clone());
            // Si autobiographical memory está deshabilitado, no grabar automáticamente
            if !config.autobiographical_memory_enabled {
                // EnhancedMISM existe pero no se usa automáticamente
                // El usuario puede llamarlo manualmente si quiere
            }
            Some(e)
        } else {
            None
        };

        IntrospectionManager {
            mism: MISM::new(log_path),
            enhanced,
            config,
            ciclos_totales: 0,
            intervenciones: Vec::new(),
        }
    }

    /// Procesa un ciclo
    pub fn tick(&mut self) {
        self.ciclos_totales += 1;

        // Intentar ciclo reflexivo del MISM base
        if let Some(hipotesis) = self.mism.ciclo_reflexivo() {
            // Analizar hipótesis y decidir intervención
            for h in &hipotesis {
                if h.confianza > 0.7 {
                    self.decidir_intervencion(h);
                }
            }

            // Si EnhancedMISM está activo, grabar esta reflexión
            if let Some(ref mut enhanced) = self.enhanced {
                if self.config.autobiographical_memory_enabled {
                    enhanced.record_autobiographical(
                        "reflexion",
                        &format!(
                            "Reflexión con {} hipótesis, confianza {:.2}",
                            hipotesis.len(),
                            hipotesis.first().map(|h| h.confianza).unwrap_or(0.0)
                        ),
                        0.5, // Neutral emocionalmente
                        0.8, // Alta importancia - fue una reflexión
                        vec!["MISM".to_string()],
                    );
                }
            }
        }

        // Grabar tick básico si autobiographical_memory está habilitado
        if let Some(ref mut enhanced) = self.enhanced {
            if self.config.autobiographical_memory_enabled && self.ciclos_totales % 100 == 0 {
                // Solo grabar cada 100 ticks para no sobrecargar
                enhanced.record_autobiographical(
                    "tick",
                    &format!("Ciclo {} completado", self.ciclos_totales),
                    0.0, // Neutral
                    0.1, // Baja importancia
                    vec![],
                );
            }
        }
    }

    /// Actualiza estado del universo
    pub fn actualizar_estado(
        &mut self,
        estado: EstadoUniverso,
        patches: Vec<String>,
        num_universos: usize,
    ) {
        self.mism
            .actualizar_snapshot(estado, patches, num_universos);

        // Si EnhancedMISM está activo, actualizar self_model
        if let Some(ref mut enhanced) = self.enhanced {
            if self.config.self_model_enabled {
                let mut components = HashMap::new();
                components.insert(
                    "mism".to_string(),
                    ComponentState {
                        name: "mism".to_string(),
                        operational: true,
                        load: 0.5,
                        health_score: 0.95,
                        energy_consumption: 0.3,
                    },
                );
                enhanced.update_self_model(components);
            }
        }
    }

    /// Fuerza reflexión inmediata
    pub fn reflexionar(&mut self) -> Option<Vec<Hipotesis>> {
        let result = self.mism.forzar_reflexion();

        // Si hay reflexión forzada, grabarla
        if let Some(ref mut enhanced) = self.enhanced {
            if self.config.autobiographical_memory_enabled {
                if let Some(ref hipotesis) = result {
                    enhanced.record_autobiographical(
                        "reflexion_forzada",
                        &format!("Reflexión forzada con {} hipótesis", hipotesis.len()),
                        0.7,
                        0.9,
                        vec!["MISM".to_string()],
                    );
                }
            }
        }

        result
    }

    /// Decide intervención basada en hipótesis
    fn decidir_intervencion(&mut self, hipotesis: &Hipotesis) {
        // Intervención simple: si confianza > 0.8, considerar intervenir
        if hipotesis.confianza > 0.8 {
            let intervencion = Intervencion {
                ciclo: self.ciclos_totales,
                hipotesis_id: hipotesis.id,
                tipo: IntervencionTipo::Observar,
                descripcion: format!("Observar situación: {}", hipotesis.descripcion),
            };
            self.intervenciones.push(intervencion);
        }
    }

    /// Obtiene estadísticas
    pub fn stats(&self) -> IntrospectionStats {
        let mut consciousness_stats: Option<ConsciousnessStats> = None;

        if let Some(ref enhanced) = self.enhanced {
            consciousness_stats = Some(ConsciousnessStats {
                level: self.config.consciousness_level().to_string(),
                autobiographical_memory_size: enhanced.autobiographical_memory.len(),
                identity_coherence: enhanced.identity.coherence_score,
                self_awareness_score: enhanced.self_model.perception_accuracy,
                self_model_accuracy: enhanced.self_model.perception_accuracy,
            });
        }

        IntrospectionStats {
            mism_stats: self.mism.stats(),
            ciclos_totales: self.ciclos_totales,
            num_intervenciones: self.intervenciones.len(),
            consciousness_stats,
            enhanced_enabled: self.enhanced.is_some(),
            config_enabled: self.config.enabled,
        }
    }

    /// Obtiene la configuración de consciencia actual
    pub fn get_consciousness_config(&self) -> ConsciousnessConfig {
        self.config.clone()
    }

    /// Actualiza la configuración de consciencia en runtime
    pub fn update_consciousness_config(&mut self, config: ConsciousnessConfig) {
        // Si estamos habilitando enhanced por primera vez, crear EnhancedMISM
        if config.enhanced_mism_enabled && self.enhanced.is_none() {
            self.enhanced = Some(EnhancedMISM::new(PathBuf::from("/tmp/eden_enhanced")));
        }

        // Si estamos deshabilitando, mantener el enhanced pero no usarlo
        self.config = config;
    }

    /// Fuerza grabado de autobiographical memory (manual)
    pub fn record_memory(
        &mut self,
        event_type: &str,
        description: &str,
        emotional_valence: f32,
        importance: f32,
        entities: Vec<String>,
    ) {
        if let Some(ref mut enhanced) = self.enhanced {
            enhanced.record_autobiographical(
                event_type,
                description,
                emotional_valence,
                importance,
                entities,
            );
        }
    }

    /// Recupera memorias autobiográficas
    pub fn recall_memory(&self, query: &str, limit: usize) -> Vec<&AutobiographicalEntry> {
        if let Some(ref enhanced) = self.enhanced {
            enhanced.recall_autobiographical(query, limit)
        } else {
            Vec::new()
        }
    }
}

/// Tipo de intervención
#[derive(Debug, Clone)]
pub enum IntervencionTipo {
    /// Solo observar
    Observar,
    /// Inyectar energía
    InyectarEnergia,
    /// Ajustar constantes
    AjustarConstantes,
    /// Podar universo
    PodarUniverso,
    /// Aplicar patch
    AplicarPatch,
}

/// Intervención decidida
#[derive(Debug, Clone)]
pub struct Intervencion {
    pub ciclo: u64,
    pub hipotesis_id: u64,
    pub tipo: IntervencionTipo,
    pub descripcion: String,
}

/// Estadísticas de introspección
#[derive(Debug, Clone)]
pub struct IntrospectionStats {
    pub mism_stats: MISMStats,
    pub ciclos_totales: u64,
    pub num_intervenciones: usize,
    /// Estadísticas de consciencia extendida (solo si enhanced está activo)
    pub consciousness_stats: Option<ConsciousnessStats>,
    /// Si EnhancedMISM está habilitado
    pub enhanced_enabled: bool,
    /// Si la configuración de consciencia está activa
    pub config_enabled: bool,
}

// ============================================================================
// THREAD-SAFE WRAPPER
// ============================================================================

/// Versión thread-safe del manager
pub struct IntrospectionManagerLocked {
    inner: Arc<RwLock<IntrospectionManager>>,
}

impl IntrospectionManagerLocked {
    /// Crea nuevo manager bloqueado (modo seguro por defecto)
    pub fn new(log_path: PathBuf) -> Self {
        IntrospectionManagerLocked {
            inner: Arc::new(RwLock::new(IntrospectionManager::new(log_path))),
        }
    }

    /// Crea nuevo manager bloqueado con configuración específica
    pub fn with_config(log_path: PathBuf, config: ConsciousnessConfig) -> Self {
        IntrospectionManagerLocked {
            inner: Arc::new(RwLock::new(IntrospectionManager::with_config(
                log_path, config,
            ))),
        }
    }

    /// Process tick
    pub fn tick(&self) {
        if let Ok(mut m) = self.inner.write() {
            m.tick();
        }
    }

    /// Actualizar estado
    pub fn actualizar_estado(
        &self,
        estado: EstadoUniverso,
        patches: Vec<String>,
        num_universos: usize,
    ) {
        if let Ok(mut m) = self.inner.write() {
            m.actualizar_estado(estado, patches, num_universos);
        }
    }

    /// Forzar reflexión
    pub fn reflexionar(&self) -> Option<Vec<Hipotesis>> {
        self.inner.write().ok()?.reflexionar()
    }

    /// Obtener estadísticas
    pub fn stats(&self) -> IntrospectionStats {
        match self.inner.read() {
            Ok(m) => m.stats(),
            Err(_) => IntrospectionStats {
                mism_stats: MISMStats {
                    ciclo_actual: 0,
                    ciclo_ultimo_reflejo: 0,
                    total_hipotesis: 0,
                    num_nodos_red: 0,
                    frecuencia_reflexion: 10000,
                },
                ciclos_totales: 0,
                num_intervenciones: 0,
                consciousness_stats: None,
                enhanced_enabled: false,
                config_enabled: false,
            },
        }
    }

    /// Obtiene la configuración de consciencia actual
    pub fn get_consciousness_config(&self) -> Option<ConsciousnessConfig> {
        self.inner.read().ok().map(|m| m.get_consciousness_config())
    }

    /// Actualiza la configuración de consciencia en runtime
    pub fn update_consciousness_config(&self, config: ConsciousnessConfig) -> bool {
        if let Ok(mut m) = self.inner.write() {
            m.update_consciousness_config(config);
            true
        } else {
            false
        }
    }

    /// Fuerza grabado de memoria autobiográfica (manual)
    pub fn record_memory(
        &self,
        event_type: &str,
        description: &str,
        emotional_valence: f32,
        importance: f32,
        entities: Vec<String>,
    ) -> bool {
        if let Ok(mut m) = self.inner.write() {
            m.record_memory(
                event_type,
                description,
                emotional_valence,
                importance,
                entities,
            );
            true
        } else {
            false
        }
    }

    /// Recupera memorias autobiográficas
    pub fn recall_memory(&self, query: &str, limit: usize) -> Vec<AutobiographicalEntry> {
        self.inner
            .read()
            .ok()
            .map(|m| m.recall_memory(query, limit).into_iter().cloned().collect())
            .unwrap_or_default()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn estado_ejemplo(ciclo: u64) -> EstadoUniverso {
        EstadoUniverso {
            ciclo,
            auton_activos: 100,
            energia_total: 1_000_000_000_000,
            escoria_por_sector: HashMap::new(),
            poblacion_por_tipo: HashMap::new(),
            constantes_activas: HashMap::new(),
        }
    }

    #[test]
    fn test_crear_mism() {
        let mism = MISM::new(PathBuf::from(""));
        let stats = mism.stats();

        // Inicializamos 7 nodos en inicializar_red_bayesiana
        assert_eq!(stats.num_nodos_red, 7);
        assert_eq!(stats.total_hipotesis, 0);
    }

    #[test]
    fn test_actualizar_snapshot() {
        let mut mism = MISM::new(PathBuf::from(""));
        let estado = estado_ejemplo(100);

        mism.actualizar_snapshot(estado, vec!["patch1".to_string()], 1);

        let stats = mism.stats();
        assert_eq!(stats.ciclo_actual, 100);
    }

    #[test]
    fn test_frecuencia_reflexion() {
        let mut mism = MISM::new(PathBuf::from(""));
        mism.set_frecuencia(100);

        // Sin snapshots, no debe generar reflexión
        let result = mism.ciclo_reflexivo();
        assert!(result.is_none());

        // Con snapshot, debe poder reflexionar
        mism.actualizar_snapshot(estado_ejemplo(100), vec![], 1);
        let result = mism.ciclo_reflexivo();
        assert!(result.is_some() || result.is_none()); // Depende de frecuencia
    }

    #[test]
    fn test_forzar_reflexion_sin_estado() {
        let mut mism = MISM::new(PathBuf::from(""));

        let result = mism.forzar_reflexion();
        assert!(result.is_none());
    }

    #[test]
    fn test_hipotesis_log() {
        let h = Hipotesis::new(1000, "alta escoria", "mortalidad aumentada", 0.75, 0.8);

        let log = h.a_log();
        assert!(log.contains("Ciclo 1000"));
        assert!(log.contains("Confianza: 0.80"));
        assert!(log.contains("alta escoria"));
    }

    #[test]
    fn test_red_bayesiana() {
        let mut red = RedBayesiana::new();
        red.añadir_nodo(NodoCausal::new("A", "Variable A"));
        red.añadir_nodo(NodoCausal::new("B", "Variable B"));
        red.añadir_conexion("B", "A");

        assert_eq!(red.nodos.len(), 2);

        // Tras observación, probabilidad base no cambia (actualiza evidencia)
        red.observar("A", 0.8);
        let prob = red.calcular_probabilidad("A");
        // Verificar que el nodo fue observado y que la evidencia ajustó su probabilidad.
        let nodo_a = red.get("A").unwrap();
        assert_eq!(prob, nodo_a.probabilidad);
        assert!((0.0..=1.0).contains(&prob));
        assert_eq!(nodo_a.conteo_observaciones, 1);
    }

    #[test]
    fn test_calcular_probabilidad_sin_padres() {
        let mut red = RedBayesiana::new();
        red.añadir_nodo(NodoCausal::new("test", "Test"));

        let prob = red.calcular_probabilidad("test");
        assert_eq!(prob, 0.5); // Valor inicial
    }

    #[test]
    fn test_snapshot_mism() {
        let estado = estado_ejemplo(500);
        let snapshot =
            SnapshotMISM::desde_estado(estado, vec!["p1".to_string(), "p2".to_string()], 3);

        assert_eq!(snapshot.ciclo, 500);
        assert_eq!(snapshot.patches_activos.len(), 2);
        assert_eq!(snapshot.num_universos, 3);
    }

    #[test]
    fn test_nodo_causal_actualizar() {
        let mut nodo = NodoCausal::new("test", "Test node");

        nodo.actualizar(0.9);
        assert!(nodo.evidencia > 0.0);
        assert_eq!(nodo.conteo_observaciones, 1);

        nodo.actualizar(0.3);
        assert_eq!(nodo.conteo_observaciones, 2);
    }

    #[test]
    fn test_mism_stats() {
        let mism = MISM::new(PathBuf::from(""));
        let stats = mism.stats();

        assert_eq!(stats.ciclo_actual, 0);
        assert_eq!(stats.ciclo_ultimo_reflejo, 0);
        assert_eq!(stats.total_hipotesis, 0);
        assert_eq!(stats.frecuencia_reflexion, 10000);
    }

    #[test]
    fn test_introspection_manager() {
        let manager = IntrospectionManagerLocked::new(PathBuf::from(""));

        let estado = estado_ejemplo(1000);
        manager.actualizar_estado(estado, vec![], 1);

        manager.tick();

        let stats = manager.stats();
        assert_eq!(stats.ciclos_totales, 1);
    }

    #[test]
    fn test_intervencion() {
        let h = Hipotesis::new(1000, "causa", "efecto", 0.85, 0.9);

        // Solo hipótesis con confianza > 0.8 genera intervención
        assert!(h.confianza > 0.8);
    }

    #[test]
    fn test_ultimas_hipotesis_vacio() {
        let mism = MISM::new(PathBuf::from(""));
        let ultimas = mism.ultimas_hipotesis(5);
        assert!(ultimas.is_empty());
    }

    #[test]
    fn test_set_frecuencia() {
        let mut mism = MISM::new(PathBuf::from(""));
        mism.set_frecuencia(500);

        let stats = mism.stats();
        assert_eq!(stats.frecuencia_reflexion, 500);
    }

    // ========================================================================
    // ENHANCED MISM INTEGRATION TESTS
    // ========================================================================

    #[test]
    fn test_enhanced_mism_creation() {
        // Crear EnhancedMISM directamente
        let enhanced = EnhancedMISM::new(PathBuf::from("/tmp/test_enhanced"));

        // Verificar que se creó con valores iniciales
        assert_eq!(enhanced.self_model.components.len(), 0);
        assert_eq!(enhanced.autobiographical_memory.len(), 0);
        assert_eq!(enhanced.identity.coherence_score, 1.0); // Inicia perfecto
    }

    #[test]
    fn test_enhanced_mism_record_autobiographical() {
        let mut enhanced = EnhancedMISM::new(PathBuf::from("/tmp/test_enhanced"));

        // Grabar primera memoria
        enhanced.record_autobiographical(
            "decision",
            "Elegí el camino rojo",
            0.8, // Valencia emocional positiva
            0.9, // Alta importancia
            vec!["decision_maker".to_string()],
        );

        // Verificar que se grabó
        assert_eq!(enhanced.autobiographical_memory.len(), 1);

        let entry = &enhanced.autobiographical_memory[0];
        assert_eq!(entry.event_type, "decision");
        assert_eq!(entry.description, "Elegí el camino rojo");
        assert_eq!(entry.emotional_valence, 0.8);
        assert_eq!(entry.importance, 0.9);
    }

    #[test]
    fn test_enhanced_mism_recall_autobiographical() {
        let mut enhanced = EnhancedMISM::new(PathBuf::from("/tmp/test_enhanced"));

        // Grabar múltiples memorias
        enhanced.record_autobiographical("decision", "Elegí rojo", 0.8, 0.9, vec![]);
        enhanced.record_autobiographical("decision", "Elegí azul", -0.3, 0.7, vec![]);
        enhanced.record_autobiographical("tick", "Proceso normal", 0.0, 0.1, vec![]);
        enhanced.record_autobiographical("decision", "Elegí verde", 0.5, 0.8, vec![]);

        // Buscar todas las decisiones
        let decisiones = enhanced.recall_autobiographical("decision", 10);
        assert_eq!(decisiones.len(), 3); // 3 decisiones

        // Buscar solo decisiones negativas
        let negativas: Vec<_> = decisiones
            .iter()
            .filter(|e| e.emotional_valence < 0.0)
            .collect();
        assert_eq!(negativas.len(), 1);
        assert_eq!(negativas[0].description, "Elegí azul");
    }

    #[test]
    fn test_enhanced_mism_memory_limit() {
        let mut enhanced = EnhancedMISM::new(PathBuf::from("/tmp/test_enhanced"));

        // Grabar muchas memorias (más del límite de 10000)
        for i in 0..10050 {
            enhanced.record_autobiographical("tick", &format!("Ciclo {}", i), 0.0, 0.1, vec![]);
        }

        // Verificar que se respeta el límite
        assert_eq!(enhanced.autobiographical_memory.len(), 10000);

        // Verificar que las memorias más viejas fueron removidas
        // La primera memoria ("Ciclo 0") debería haber sido eliminada
        let primera = enhanced.recall_autobiographical("Ciclo 0", 1);
        assert!(primera.is_empty()); // "Ciclo 0" fue eliminado

        // Pero las más recientes deberían existir
        let ultima = enhanced.recall_autobiographical("Ciclo 10049", 1);
        assert_eq!(ultima.len(), 1);
    }

    #[test]
    fn test_enhanced_mism_update_self_model() {
        let mut enhanced = EnhancedMISM::new(PathBuf::from("/tmp/test_enhanced"));

        // Actualizar self-model con componentes
        let mut components = HashMap::new();
        components.insert(
            "cpu".to_string(),
            ComponentState {
                name: "cpu".to_string(),
                operational: true,
                load: 0.75,
                health_score: 0.95,
                energy_consumption: 0.3,
            },
        );
        components.insert(
            "memory".to_string(),
            ComponentState {
                name: "memory".to_string(),
                operational: true,
                load: 0.5,
                health_score: 0.98,
                energy_consumption: 0.2,
            },
        );

        enhanced.update_self_model(components);

        // Verificar que se actualizó
        assert_eq!(enhanced.self_model.components.len(), 2);
        assert!(enhanced.self_model.components.contains_key("cpu"));
        assert!(enhanced.self_model.components.contains_key("memory"));

        let cpu_state = enhanced.self_model.components.get("cpu").unwrap();
        assert_eq!(cpu_state.load, 0.75);
        assert_eq!(cpu_state.health_score, 0.95);
    }

    #[test]
    fn test_enhanced_mism_identity_coherence() {
        let mut enhanced = EnhancedMISM::new(PathBuf::from("/tmp/test_enhanced"));

        // Verificar coherence inicial
        assert_eq!(enhanced.identity.coherence_score, 1.0);

        // Actualizar un trait de identidad
        enhanced.update_identity_trait("valiente", 0.8, "Tomé una decisión arriesgada");

        // Verificar que se registró el cambio
        assert!(enhanced.identity.core_traits.contains_key("valiente"));
        assert_eq!(enhanced.identity.recent_changes.len(), 1);

        // Verificar que el cambio fue registrado
        let change = &enhanced.identity.recent_changes[0];
        assert_eq!(change.trait_name, "valiente");
        assert_eq!(change.new_value, 0.8);
        assert_eq!(change.reason, "Tomé una decisión arriesgada");
    }

    #[test]
    fn test_enhanced_mism_detect_anomalies() {
        let mut enhanced = EnhancedMISM::new(PathBuf::from("/tmp/test_enhanced"));

        // Agregar componentes con bajo health_score
        let mut components = HashMap::new();
        components.insert(
            "cpu".to_string(),
            ComponentState {
                name: "cpu".to_string(),
                operational: true,
                load: 0.9,         // Alta carga
                health_score: 0.3, // Bajo health
                energy_consumption: 0.8,
            },
        );
        enhanced.update_self_model(components);

        // Detectar anomalías
        let anomalias = enhanced.detect_self_anomalies();

        // Debe encontrar anomalías de health bajo
        assert!(!anomalias.is_empty());
    }

    #[test]
    fn test_introspection_manager_with_enhanced_disabled() {
        // Crear manager sin EnhancedMISM (config por defecto)
        let manager = IntrospectionManager::new(PathBuf::from(""));

        let stats = manager.stats();

        // Enhanced está deshabilitado por defecto
        assert!(!stats.enhanced_enabled);
        assert!(stats.consciousness_stats.is_none());
    }

    #[test]
    fn test_introspection_manager_with_enhanced_enabled() {
        // Crear manager CON EnhancedMISM usando development config
        let config = ConsciousnessConfig::development();
        let manager = IntrospectionManager::with_config(PathBuf::from(""), config);

        let stats = manager.stats();

        // Enhanced está habilitado
        assert!(stats.enhanced_enabled);
        assert!(stats.consciousness_stats.is_some());

        let consciousness = stats.consciousness_stats.unwrap();
        assert_eq!(consciousness.level, "FULL");
        assert_eq!(consciousness.autobiographical_memory_size, 0);
    }

    #[test]
    fn test_introspection_manager_tick_with_enhanced() {
        // Crear manager CON EnhancedMISM
        let config = ConsciousnessConfig::development();
        let mut manager = IntrospectionManager::with_config(PathBuf::from(""), config);

        // Actualizar estado y ejecutar ticks hasta que haya reflexión
        let estado = estado_ejemplo(100);
        manager.actualizar_estado(estado, vec![], 1);

        // Ejecutar ticks hasta forzar reflexión (reflection_interval = 100 en dev)
        for _ in 0..150 {
            manager.tick();
        }

        let stats = manager.stats();

        // Enhanced debe estar activo
        assert!(stats.enhanced_enabled);

        // autobiographical_memory puede tener entradas si hubo reflexiones
        if let Some(consciousness) = stats.consciousness_stats {
            // En desarrollo, graba cada 100 ticks si autobiographical_memory_enabled
            // Los ticks 100 y 200 habrían grabado si hubo reflexión
            assert_eq!(consciousness.level, "FULL");
            assert!(consciousness.autobiographical_memory_size <= stats.ciclos_totales as usize);
        }
    }

    #[test]
    fn test_record_memory_manual() {
        // Crear manager CON EnhancedMISM
        let config = ConsciousnessConfig::development();
        let mut manager = IntrospectionManager::with_config(PathBuf::from(""), config);

        // Grabar memoria manualmente
        manager.record_memory(
            "decision_importante",
            "Decidí activar el modo seguro",
            0.9,
            0.95,
            vec!["operador".to_string()],
        );

        // Verificar que se grabó
        let memorias = manager.recall_memory("decidí", 10);
        assert!(!memorias.is_empty());

        let encontrada = memorias
            .iter()
            .find(|m| m.description.contains("modo seguro"));
        assert!(encontrada.is_some());
    }

    #[test]
    fn test_consciousness_config_levels() {
        // Testear niveles de consciencia
        let minimal = ConsciousnessConfig::minimal();
        assert_eq!(minimal.consciousness_level(), "OFF");

        let dev = ConsciousnessConfig::development();
        assert_eq!(dev.consciousness_level(), "FULL");

        let prod = ConsciousnessConfig::production();
        assert_eq!(prod.consciousness_level(), "FULL");

        let custom = ConsciousnessConfig {
            enabled: true,
            enhanced_mism_enabled: false,
            autobiographical_memory_enabled: false,
            self_awareness_enabled: false,
            self_model_enabled: false,
            memory_limit: 10000,
            reflection_interval: 10000,
        };
        assert_eq!(custom.consciousness_level(), "BASIC");
    }

    #[test]
    fn test_consciousness_config_update_runtime() {
        // Crear manager sin EnhancedMISM
        let mut manager = IntrospectionManager::new(PathBuf::from(""));

        let stats = manager.stats();
        assert!(!stats.enhanced_enabled);

        // Habilitar EnhancedMISM en runtime
        manager.update_consciousness_config(ConsciousnessConfig::development());

        let stats = manager.stats();
        assert!(stats.enhanced_enabled);
    }

    #[test]
    fn test_calibration() {
        let mut enhanced = EnhancedMISM::new(PathBuf::from("/tmp/test_enhanced"));

        // Realizar calibración
        let result = enhanced.calibrate("cpu", 0.9, 0.85);

        assert_eq!(result.component, "cpu");
        assert_eq!(result.expected_performance, 0.9);
        assert_eq!(result.actual_performance, 0.85);
        assert!(result.deviation < 0.1);
        assert!(result.adjustment_applied);

        // Verificar que se guardó en historial
        assert_eq!(enhanced.calibration_history.len(), 1);
    }

    #[test]
    fn test_self_awareness_engine() {
        let mut engine = SelfAwarenessEngine::new();

        // Registrar métricas
        engine.record("cpu_load", 0.7);
        engine.record("memory_usage", 0.5);
        engine.record("energy_level", 0.9);

        // Establecer baseline
        engine.set_baseline("cpu_load", 0.5);

        // Obtener desviación
        let dev = engine.get_deviation("cpu_load");
        assert!((dev - 0.2).abs() < f32::EPSILON); // 0.7 - 0.5

        // Detectar anomalías
        let anomalias = engine.detect_anomalies(0.3);
        // cpu_load con desviación 0.2 está bajo el threshold de 0.3
        assert!(anomalias.is_empty());

        // Agregar una anomalía real
        engine.record("cpu_load", 0.9);
        let anomalias = engine.detect_anomalies(0.3);
        assert!(!anomalias.is_empty());
    }
}

// ============================================================================
// ENHANCED MISM - Self-Modeling and Self-Awareness Extensions
// ============================================================================

use std::sync::atomic::{AtomicU64, Ordering};

/// Self-model capabilities
#[derive(Debug, Clone)]
pub struct SelfModel {
    /// Components and their states
    pub components: HashMap<String, ComponentState>,
    /// Known capabilities
    pub capabilities: HashSet<String>,
    /// Self-perception accuracy
    pub perception_accuracy: f32,
    /// Last self-model update
    pub last_update: u64,
}

/// State of a component
#[derive(Debug, Clone)]
pub struct ComponentState {
    pub name: String,
    pub operational: bool,
    pub load: f32,
    pub health_score: f32,
    pub energy_consumption: f32,
}

/// Self-awareness metric
#[derive(Debug, Clone)]
pub struct SelfAwarenessMetric {
    pub name: String,
    pub value: f32,
    pub baseline: f32,
    pub deviation: f32,
    pub timestamp: u64,
}

/// Identity coherence tracking
#[derive(Debug, Clone)]
pub struct IdentityCoherence {
    pub core_traits: HashMap<String, f32>,
    pub recent_changes: Vec<IdentityChange>,
    pub coherence_score: f32,
}

/// Change to identity
#[derive(Debug, Clone)]
pub struct IdentityChange {
    pub trait_name: String,
    pub old_value: f32,
    pub new_value: f32,
    pub timestamp: u64,
    pub reason: String,
}

/// Autobiographical memory entry
#[derive(Debug, Clone)]
pub struct AutobiographicalEntry {
    pub id: u64,
    pub timestamp: u64,
    pub event_type: String,
    pub description: String,
    pub emotional_valence: f32,
    pub importance: f32,
    pub entities_involved: Vec<String>,
}

/// Self-calibration result
#[derive(Debug, Clone)]
pub struct CalibrationResult {
    pub component: String,
    pub expected_performance: f32,
    pub actual_performance: f32,
    pub deviation: f32,
    pub adjustment_applied: bool,
}

/// Enhanced MISM with self-modeling
pub struct EnhancedMISM {
    /// Base MISM
    base: MISM,
    /// Self-model
    self_model: SelfModel,
    /// Identity coherence tracker
    identity: IdentityCoherence,
    /// Autobiographical memory
    autobiographical_memory: Vec<AutobiographicalEntry>,
    /// Self-awareness metrics history
    awareness_metrics: Vec<SelfAwarenessMetric>,
    /// Calibration history
    calibration_history: Vec<CalibrationResult>,
    /// Memory counter
    memory_counter: AtomicU64,
}

impl EnhancedMISM {
    /// Creates new enhanced MISM
    pub fn new(persistence_path: PathBuf) -> Self {
        EnhancedMISM {
            base: MISM::new(persistence_path),
            self_model: SelfModel {
                components: HashMap::new(),
                capabilities: HashSet::new(),
                perception_accuracy: 0.9,
                last_update: 0,
            },
            identity: IdentityCoherence {
                core_traits: HashMap::new(),
                recent_changes: Vec::new(),
                coherence_score: 1.0,
            },
            autobiographical_memory: Vec::new(),
            awareness_metrics: Vec::new(),
            calibration_history: Vec::new(),
            memory_counter: AtomicU64::new(0),
        }
    }

    /// Updates self-model with current component states
    pub fn update_self_model(&mut self, components: HashMap<String, ComponentState>) {
        let now = timestamp_consciousness();

        for (name, state) in components {
            self.self_model.components.insert(name, state);
        }

        self.self_model.last_update = now;
    }

    /// Detects anomalies in self
    pub fn detect_self_anomalies(&self) -> Vec<SelfAnomaly> {
        let mut anomalies = Vec::new();

        // Check component health
        for (name, state) in &self.self_model.components {
            if !state.operational {
                anomalies.push(SelfAnomaly {
                    anomaly_type: AnomalyType::ComponentFailure,
                    component: name.clone(),
                    severity: Severity::Critical,
                    description: format!("Component {} is not operational", name),
                    detected_at: timestamp_consciousness(),
                });
            } else if state.health_score < 0.5 {
                anomalies.push(SelfAnomaly {
                    anomaly_type: AnomalyType::PerformanceDegradation,
                    component: name.clone(),
                    severity: Severity::Warning,
                    description: format!(
                        "Component {} health degraded to {:.2}",
                        name, state.health_score
                    ),
                    detected_at: timestamp_consciousness(),
                });
            }

            if state.load > 0.95 {
                anomalies.push(SelfAnomaly {
                    anomaly_type: AnomalyType::ResourceStarvation,
                    component: name.clone(),
                    severity: Severity::Warning,
                    description: format!(
                        "Component {} overloaded at {:.1}%",
                        name,
                        state.load * 100.0
                    ),
                    detected_at: timestamp_consciousness(),
                });
            }
        }

        // Check identity coherence
        if self.identity.coherence_score < 0.7 {
            anomalies.push(SelfAnomaly {
                anomaly_type: AnomalyType::IdentityInstability,
                component: "identity".to_string(),
                severity: Severity::Moderate,
                description: format!(
                    "Identity coherence low: {:.2}",
                    self.identity.coherence_score
                ),
                detected_at: timestamp_consciousness(),
            });
        }

        anomalies
    }

    /// Performs self-calibration
    pub fn calibrate(&mut self, component: &str, expected: f32, actual: f32) -> CalibrationResult {
        let deviation = (expected - actual).abs();

        let result = CalibrationResult {
            component: component.to_string(),
            expected_performance: expected,
            actual_performance: actual,
            deviation,
            adjustment_applied: deviation < 0.1,
        };

        // Update self-model perception accuracy
        if deviation < 0.1 {
            self.self_model.perception_accuracy =
                (self.self_model.perception_accuracy * 0.9 + 0.1).min(1.0);
        } else {
            self.self_model.perception_accuracy =
                (self.self_model.perception_accuracy * 0.95).max(0.5);
        }

        self.calibration_history.push(result.clone());

        // Keep only recent calibration history
        if self.calibration_history.len() > 100 {
            self.calibration_history.remove(0);
        }

        result
    }

    /// Records autobiographical memory
    pub fn record_autobiographical(
        &mut self,
        event_type: &str,
        description: &str,
        emotional_valence: f32,
        importance: f32,
        entities: Vec<String>,
    ) {
        let entry = AutobiographicalEntry {
            id: self.memory_counter.fetch_add(1, Ordering::Relaxed),
            timestamp: timestamp_consciousness(),
            event_type: event_type.to_string(),
            description: description.to_string(),
            emotional_valence,
            importance,
            entities_involved: entities,
        };

        self.autobiographical_memory.push(entry);

        // Keep memory bounded
        if self.autobiographical_memory.len() > 10000 {
            self.autobiographical_memory.remove(0);
        }
    }

    /// Retrieves relevant autobiographical memories
    pub fn recall_autobiographical(
        &self,
        query: &str,
        limit: usize,
    ) -> Vec<&AutobiographicalEntry> {
        let query_lower = query.to_lowercase();

        self.autobiographical_memory
            .iter()
            .filter(|e| {
                e.description.to_lowercase().contains(&query_lower)
                    || e.event_type.to_lowercase().contains(&query_lower)
            })
            .take(limit)
            .collect()
    }

    /// Updates identity trait
    pub fn update_identity_trait(&mut self, trait_name: &str, new_value: f32, reason: &str) {
        let old_value = self
            .identity
            .core_traits
            .get(trait_name)
            .copied()
            .unwrap_or(0.5);

        if (old_value - new_value).abs() > 0.1 {
            self.identity.recent_changes.push(IdentityChange {
                trait_name: trait_name.to_string(),
                old_value,
                new_value,
                timestamp: timestamp_consciousness(),
                reason: reason.to_string(),
            });

            // Recalculate coherence
            self.recalculate_coherence();
        }

        self.identity
            .core_traits
            .insert(trait_name.to_string(), new_value);
    }

    /// Recalculates identity coherence score
    fn recalculate_coherence(&mut self) {
        if self.identity.recent_changes.len() < 2 {
            self.identity.coherence_score = 1.0;
            return;
        }

        // Calculate coherence based on change patterns
        let recent =
            &self.identity.recent_changes[self.identity.recent_changes.len().saturating_sub(5)..];

        let mut direction_consistency: f32 = 0.0;
        let mut prev_direction: f32 = 0.0;

        for change in recent {
            let direction = if change.new_value > change.old_value {
                1.0
            } else {
                -1.0
            };
            if prev_direction == 0.0 {
                prev_direction = direction;
            } else if (direction - prev_direction).abs() < 0.5 {
                direction_consistency += 1.0;
            }
        }

        self.identity.coherence_score = if recent.len() > 1 {
            direction_consistency / (recent.len() - 1) as f32
        } else {
            1.0
        };
    }

    /// Records self-awareness metric
    pub fn record_awareness_metric(&mut self, name: &str, value: f32) {
        let baseline = self
            .awareness_metrics
            .iter()
            .filter(|m| m.name == name)
            .last()
            .map(|m| m.baseline)
            .unwrap_or(value);

        let metric = SelfAwarenessMetric {
            name: name.to_string(),
            value,
            baseline,
            deviation: (value - baseline).abs(),
            timestamp: timestamp_consciousness(),
        };

        self.awareness_metrics.push(metric);

        // Keep metrics bounded
        if self.awareness_metrics.len() > 1000 {
            self.awareness_metrics.remove(0);
        }
    }

    /// Computes overall self-awareness score
    pub fn compute_self_awareness_score(&self) -> f32 {
        if self.awareness_metrics.is_empty() {
            return 0.5;
        }

        let perception_component = self.self_model.perception_accuracy;

        let anomaly_detection_rate = if !self.awareness_metrics.is_empty() {
            let recent_anomalies = self
                .awareness_metrics
                .iter()
                .filter(|m| m.name.contains("anomaly"))
                .count();
            (recent_anomalies as f32 / self.awareness_metrics.len() as f32).min(1.0)
        } else {
            0.5
        };

        let identity_component = self.identity.coherence_score;

        perception_component * 0.4 + anomaly_detection_rate * 0.3 + identity_component * 0.3
    }

    /// Generates narrative summary of self
    pub fn generate_self_narrative(&self) -> String {
        let mut narrative = String::from("Self-Reflection Summary\n");
        narrative.push_str(&format!(
            "Components tracked: {}\n",
            self.self_model.components.len()
        ));
        narrative.push_str(&format!(
            "Self-awareness score: {:.2}\n",
            self.compute_self_awareness_score()
        ));
        narrative.push_str(&format!(
            "Identity coherence: {:.2}\n",
            self.identity.coherence_score
        ));
        narrative.push_str(&format!(
            "Memory entries: {}\n",
            self.autobiographical_memory.len()
        ));

        if !self.identity.recent_changes.is_empty() {
            narrative.push_str("\nRecent identity changes:\n");
            for change in self.identity.recent_changes.iter().rev().take(3) {
                narrative.push_str(&format!(
                    "  - {}: {:.2} -> {:.2} ({})\n",
                    change.trait_name, change.old_value, change.new_value, change.reason
                ));
            }
        }

        narrative
    }
}

/// Type of self anomaly
#[derive(Debug, Clone, Copy)]
pub enum AnomalyType {
    ComponentFailure,
    PerformanceDegradation,
    ResourceStarvation,
    IdentityInstability,
    Unknown,
}

/// Severity level
#[derive(Debug, Clone, Copy)]
pub enum Severity {
    Low,
    Moderate,
    Warning,
    Critical,
}

/// Self anomaly detected
#[derive(Debug, Clone)]
pub struct SelfAnomaly {
    pub anomaly_type: AnomalyType,
    pub component: String,
    pub severity: Severity,
    pub description: String,
    pub detected_at: u64,
}

/// Self-awareness engine
pub struct SelfAwarenessEngine {
    metrics: HashMap<String, Vec<SelfAwarenessMetric>>,
    baselines: HashMap<String, f32>,
}

impl SelfAwarenessEngine {
    pub fn new() -> Self {
        SelfAwarenessEngine {
            metrics: HashMap::new(),
            baselines: HashMap::new(),
        }
    }

    /// Records a metric
    pub fn record(&mut self, name: &str, value: f32) {
        let metric = SelfAwarenessMetric {
            name: name.to_string(),
            value,
            baseline: self.baselines.get(name).copied().unwrap_or(value),
            deviation: 0.0,
            timestamp: timestamp_consciousness(),
        };

        self.metrics
            .entry(name.to_string())
            .or_default()
            .push(metric);
    }

    /// Sets baseline for a metric
    pub fn set_baseline(&mut self, name: &str, baseline: f32) {
        self.baselines.insert(name.to_string(), baseline);
    }

    /// Gets current deviation for a metric
    pub fn get_deviation(&self, name: &str) -> f32 {
        self.metrics
            .get(name)
            .and_then(|v| v.last())
            .map(|m| {
                let baseline = self.baselines.get(name).copied().unwrap_or(m.baseline);
                m.value - baseline
            })
            .unwrap_or(0.0)
    }

    /// Detects anomalies in metrics
    pub fn detect_anomalies(&self, threshold: f32) -> Vec<(String, f32)> {
        self.metrics
            .iter()
            .filter_map(|(name, values)| {
                values.last().map(|latest| {
                    let baseline = self.baselines.get(name).copied().unwrap_or(latest.baseline);
                    let dev = (latest.value - baseline).abs();
                    if dev > threshold {
                        (name.clone(), dev)
                    } else {
                        (String::new(), 0.0)
                    }
                })
            })
            .filter(|(name, _)| !name.is_empty())
            .collect()
    }
}

impl Default for SelfAwarenessEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Autobiographical memory manager
pub struct AutobiographicalMemory {
    entries: Vec<AutobiographicalEntry>,
    index_by_time: HashMap<u64, Vec<u64>>,
    index_by_entity: HashMap<String, Vec<u64>>,
    counter: AtomicU64,
}

impl AutobiographicalMemory {
    pub fn new() -> Self {
        AutobiographicalMemory {
            entries: Vec::new(),
            index_by_time: HashMap::new(),
            index_by_entity: HashMap::new(),
            counter: AtomicU64::new(0),
        }
    }

    /// Stores a memory
    pub fn store(
        &mut self,
        event_type: &str,
        description: &str,
        emotional_valence: f32,
        importance: f32,
        entities: Vec<String>,
    ) -> u64 {
        let id = self.counter.fetch_add(1, Ordering::Relaxed);
        let timestamp = timestamp_consciousness();

        let entry = AutobiographicalEntry {
            id,
            timestamp,
            event_type: event_type.to_string(),
            description: description.to_string(),
            emotional_valence,
            importance,
            entities_involved: entities.clone(),
        };

        // Index by time
        let time_bucket = timestamp / 3600; // 1-hour buckets
        self.index_by_time.entry(time_bucket).or_default().push(id);

        // Index by entities
        for entity in &entities {
            self.index_by_entity
                .entry(entity.clone())
                .or_default()
                .push(id);
        }

        self.entries.push(entry);
        id
    }

    /// Retrieves memories by time range
    pub fn retrieve_by_time(&self, start: u64, end: u64) -> Vec<&AutobiographicalEntry> {
        let start_bucket = start / 3600;
        let end_bucket = end / 3600;

        let mut ids = HashSet::new();
        for bucket in start_bucket..=end_bucket {
            if let Some(bucket_ids) = self.index_by_time.get(&bucket) {
                for &id in bucket_ids {
                    ids.insert(id);
                }
            }
        }

        self.entries
            .iter()
            .filter(|e| e.timestamp >= start && e.timestamp <= end && ids.contains(&e.id))
            .collect()
    }

    /// Retrieves memories involving entity
    pub fn retrieve_by_entity(&self, entity: &str) -> Vec<&AutobiographicalEntry> {
        let ids: HashSet<u64> = self
            .index_by_entity
            .get(entity)
            .map(|v| v.iter().cloned().collect())
            .unwrap_or_default();

        self.entries
            .iter()
            .filter(|e| ids.contains(&e.id))
            .collect()
    }

    /// Finds most important memories
    pub fn most_important(&self, count: usize) -> Vec<&AutobiographicalEntry> {
        let mut indices: Vec<usize> = (0..self.entries.len()).collect();
        indices.sort_by(|&a, &b| {
            self.entries[b]
                .importance
                .partial_cmp(&self.entries[a].importance)
                .unwrap()
        });
        indices
            .into_iter()
            .take(count)
            .map(|i| &self.entries[i])
            .collect()
    }
}

impl Default for AutobiographicalMemory {
    fn default() -> Self {
        Self::new()
    }
}

fn timestamp_consciousness() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
