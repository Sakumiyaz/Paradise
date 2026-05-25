//! # World Model Digital — Modelo Predictivo del Mundo Digital
//!
//! Este módulo permite a EDEN construir y mantener un modelo predictivo
//! del mundo digital que lo rodea.
//!
//! ## Concepto
//!
//! EDEN no solo observa el mundo digital — construye un modelo mental de él.
//! Este modelo incluye:
//! - Entidades digitales (procesos, archivos, conexiones)
//! - Relaciones entre entidades
//! - Predicciones de comportamiento
//! - Estados probables futuros
//!
//! ## Analogía Filosófica
//!
//! Así como los Autons tienen un Modelo Interno de Sí Mismos (MISM),
//! EDEN ahora tiene un "MUNDO" — un modelo del mundo digital externo.
//! La diferencia: el MISM es sobre sí mismo, el WorldModel es sobre lo otro.
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH, Duration};

/// Tipo de entidad digital
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EntityType {
    /// Proceso del sistema
    Process,
    /// Archivo o directorio
    File,
    /// Conexión de red
    NetworkConnection,
    /// Socket
    Socket,
    /// Dirección IP
    IPAddress,
    /// Dominio DNS
    Domain,
    /// Servicio de red
    Service,
    /// Puerto
    Port,
    /// Paquete de red
    NetworkPacket,
    /// Usuario del sistema
    User,
    /// Grupo
    Group,
    /// Dispositivo
    Device,
}

impl EntityType {
    pub fn nombre(&self) -> &'static str {
        match self {
            EntityType::Process => "Process",
            EntityType::File => "File",
            EntityType::NetworkConnection => "NetworkConnection",
            EntityType::Socket => "Socket",
            EntityType::IPAddress => "IPAddress",
            EntityType::Domain => "Domain",
            EntityType::Service => "Service",
            EntityType::Port => "Port",
            EntityType::NetworkPacket => "NetworkPacket",
            EntityType::User => "User",
            EntityType::Group => "Group",
            EntityType::Device => "Device",
        }
    }
}

/// Entidad digital observada
#[derive(Debug, Clone)]
pub struct DigitalEntity {
    /// Identificador único
    pub id: String,
    /// Tipo de entidad
    pub tipo: EntityType,
    /// Nombre/desripción
    pub nombre: String,
    /// Atributos clave-valor
    pub atributos: HashMap<String, String>,
    /// Timestamp de creación
    pub creado_ms: u64,
    /// Última actualización
    pub actualizado_ms: u64,
    /// Entidades relacionadas (IDs)
    pub relaciones: HashSet<String>,
    /// Confianza en la observación (0-1)
    pub confianza: f64,
    /// Historial de estados
    pub historial_estados: Vec<EntityState>,
}

#[derive(Debug, Clone)]
pub struct EntityState {
    pub timestamp_ms: u64,
    pub atributos: HashMap<String, String>,
    pub saludable: bool,
}

impl DigitalEntity {
    pub fn nuevo(tipo: EntityType, nombre: String) -> Self {
        let now = current_timestamp_ms();
        Self {
            id: generar_id_entidad(),
            tipo,
            nombre,
            atributos: HashMap::new(),
            creado_ms: now,
            actualizado_ms: now,
            relaciones: HashSet::new(),
            confianza: 0.5,
            historial_estados: Vec::new(),
        }
    }

    pub fn con_atributo(mut self, clave: &str, valor: &str) -> Self {
        self.atributos.insert(clave.to_string(), valor.to_string());
        self
    }

    pub fn con_relacion(mut self, id_relacion: &str) -> Self {
        self.relaciones.insert(id_relacion.to_string());
        self
    }

    pub fn actualizar(&mut self, nuevos_atributos: HashMap<String, String>) {
        let now = current_timestamp_ms();
        self.atributos.extend(nuevos_atributos);
        self.actualizado_ms = now;

        // Registrar en historial
        self.historial_estados.push(EntityState {
            timestamp_ms: now,
            atributos: self.atributos.clone(),
            saludable: self.es_saludable(),
        });

        // Limitar historial a últimos 100 estados
        if self.historial_estados.len() > 100 {
            self.historial_estados.remove(0);
        }
    }

    fn es_saludable(&self) -> bool {
        // Lógica simple: si fue actualizado recientemente, está saludable
        let now = current_timestamp_ms();
        now - self.actualizado_ms < 60_000 // 1 minuto
    }
}

/// Predicción del modelo
#[derive(Debug, Clone)]
pub struct Prediction {
    /// ID de la predicción
    pub id: String,
    /// Tipo de predicción
    pub tipo: PredictionType,
    /// Entidad a la que afecta
    pub entidad_id: String,
    /// Timestamp de la predicción
    pub timestamp_ms: u64,
    /// Estado predicho
    pub estado_predicho: String,
    /// Confianza de la predicción (0-1)
    pub confianza: f64,
    /// Horizonte temporal (ms hasta el evento)
    pub horizonte_ms: u64,
    /// Precisión real (actualizada después)
    pub precision_real: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PredictionType {
    /// Predicción de comportamiento
    Comportamiento,
    /// Predicción de estado
    Estado,
    /// Predicción de relación
    Relacion,
    /// Predicción de fallo
    Fallo,
    /// Predicción de oportunidades
    Oportunidad,
}

impl PredictionType {
    pub fn nombre(&self) -> &'static str {
        match self {
            PredictionType::Comportamiento => "Comportamiento",
            PredictionType::Estado => "Estado",
            PredictionType::Relacion => "Relacion",
            PredictionType::Fallo => "Fallo",
            PredictionType::Oportunidad => "Oportunidad",
        }
    }
}

/// Nivel de confianza del modelo
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ModelConfidence {
    /// Confianza muy baja (< 0.2)
    MuyBaja,
    /// Confianza baja (0.2 - 0.4)
    Baja,
    /// Confianza media (0.4 - 0.6)
    Media,
    /// Confianza alta (0.6 - 0.8)
    Alta,
    /// Confianza muy alta (> 0.8)
    MuyAlta,
}

impl ModelConfidence {
    pub fn from_f64(valor: f64) -> Self {
        if valor < 0.2 {
            Self::MuyBaja
        } else if valor < 0.4 {
            Self::Baja
        } else if valor < 0.6 {
            Self::Media
        } else if valor < 0.8 {
            Self::Alta
        } else {
            Self::MuyAlta
        }
    }
}

/// Estadísticas del modelo
#[derive(Debug, Clone)]
pub struct WorldModelStats {
    pub total_entidades: usize,
    pub entidades_por_tipo: HashMap<EntityType, usize>,
    pub total_predicciones: usize,
    pub predicciones_activas: usize,
    pub precision_promedio: f64,
    pub ultima_actualizacion_ms: u64,
    pub confianza_promedio: f64,
}

/// Modelo del mundo digital
pub struct WorldModelDigital {
    /// Entidades conocidas
    entidades: HashMap<String, DigitalEntity>,
    /// Predicciones activas
    predicciones: Vec<Prediction>,
    /// Histórico de predicciones cumplidas
    predicciones_cumplidas: Vec<Prediction>,
    /// Configuración
    config: ModelConfig,
    /// Estadísticas
    stats: WorldModelStats,
}

#[derive(Debug, Clone)]
pub struct ModelConfig {
    /// Máximo de entidades a rastrear
    pub max_entidades: usize,
    /// Horizonte de predicción máximo (ms)
    pub horizonte_max_ms: u64,
    /// Confianza mínima para predicciones
    pub confianza_minima: f64,
    /// Habilitar predicciones
    pub predicciones_habilitadas: bool,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            max_entidades: 1000,
            horizonte_max_ms: 3600_000, // 1 hora
            confianza_minima: 0.5,
            predicciones_habilitadas: true,
        }
    }
}

impl WorldModelDigital {
    /// Crea un nuevo modelo
    pub fn new(config: ModelConfig) -> Self {
        Self {
            entidades: HashMap::new(),
            predicciones: Vec::new(),
            predicciones_cumplidas: Vec::new(),
            config,
            stats: WorldModelStats {
                total_entidades: 0,
                entidades_por_tipo: HashMap::new(),
                total_predicciones: 0,
                predicciones_activas: 0,
                precision_promedio: 0.0,
                ultima_actualizacion_ms: current_timestamp_ms(),
                confianza_promedio: 0.5,
            },
        }
    }

    /// Agrega una entidad al modelo
    pub fn agregar_entidad(&mut self, mut entidad: DigitalEntity) -> String {
        // Verificar límite
        if self.entidades.len() >= self.config.max_entidades {
            // Eliminar entidad menos importante
            if let Some((id, _)) = self.entidades.iter()
                .min_by_key(|(_, e)| (e.confianza * 1000.0) as i64)
                .map(|(id, e)| (id.clone(), e.clone()))
            {
                self.entidades.remove(&id);
            }
        }

        // Asegurar que tiene ID único
        if entidad.id.is_empty() {
            entidad.id = generar_id_entidad();
        }

        let id = entidad.id.clone();
        self.entidades.insert(id.clone(), entidad);

        self.stats.total_entidades = self.entidades.len();
        self.stats.ultima_actualizacion_ms = current_timestamp_ms();

        id
    }

    /// Actualiza una entidad existente
    pub fn actualizar_entidad(&mut self, id: &str, atributos: HashMap<String, String>) -> bool {
        if let Some(entidad) = self.entidades.get_mut(id) {
            entidad.actualizar(atributos);
            self.stats.ultima_actualizacion_ms = current_timestamp_ms();
            return true;
        }
        false
    }

    /// Obtiene una entidad por ID
    pub fn obtener_entidad(&self, id: &str) -> Option<&DigitalEntity> {
        self.entidades.get(id)
    }

    /// Obtiene entidades por tipo
    pub fn obtener_por_tipo(&self, tipo: EntityType) -> Vec<&DigitalEntity> {
        self.entidades.values()
            .filter(|e| e.tipo == tipo)
            .collect()
    }

    /// Genera una predicción
    pub fn predecir(&mut self, entidad_id: &str, horizonte_ms: u64) -> Option<Prediction> {
        if !self.config.predicciones_habilitadas {
            return None;
        }

        let entidad = self.entidades.get(entidad_id)?;

        // Calcular confianza basada en historial
        let confianza = self.calcular_confianza(entidad);

        if confianza < self.config.confianza_minima {
            return None;
        }

        let prediccion = Prediction {
            id: generar_id_prediccion(),
            tipo: PredictionType::Comportamiento,
            entidad_id: entidad_id.to_string(),
            timestamp_ms: current_timestamp_ms(),
            estado_predicho: self.predecir_estado(entidad),
            confianza,
            horizonte_ms,
            precision_real: None,
        };

        self.predicciones.push(prediccion.clone());
        self.stats.total_predicciones += 1;
        self.stats.predicciones_activas = self.predicciones.len();

        Some(prediccion)
    }

    /// Calcula confianza basada en el historial de la entidad
    fn calcular_confianza(&self, entidad: &DigitalEntity) -> f64 {
        let base = entidad.confianza;

        // Más historial = más confianza
        let bonus_historial = (entidad.historial_estados.len() as f64 * 0.01).min(0.3);

        // Entidad reciente = menos confianza
        let now = current_timestamp_ms();
        let edad_ms = now.saturating_sub(entidad.actualizado_ms);
        let malus_edad = if edad_ms > 300_000 {
            ((edad_ms - 300_000) as f64 / 3_600_000.0).min(0.2)
        } else {
            0.0
        };

        (base + bonus_historial - malus_edad).max(0.1).min(1.0)
    }

    /// Predice el siguiente estado de una entidad
    fn predecir_estado(&self, entidad: &DigitalEntity) -> String {
        // Análisis simple basado en tendencias
        if entidad.historial_estados.len() >= 2 {
            let ultimo = entidad.historial_estados.last().unwrap();
            let anterior = entidad.historial_estados[entidad.historial_estados.len() - 2].clone();

            // Si hay degrado, predecir más degrado
            if !ultimo.saludable && anterior.saludable {
                return "Degradando".to_string();
            }

            // Si está estable, predecir estabilidad
            if ultimo.saludable && anterior.saludable {
                return "Estable".to_string();
            }
        }

        "Indeterminado".to_string()
    }

    /// Obtiene predicciones activas
    pub fn predicciones_activas(&self) -> Vec<&Prediction> {
        self.predicciones.iter().filter(|p| p.precision_real.is_none()).collect()
    }

    /// Verifica y actualiza predicciones cumplidas
    pub fn verificar_predicciones(&mut self) -> Vec<Prediction> {
        let now = current_timestamp_ms();
        let mut cumplidas = Vec::new();

        // Recolectar entidades y predicciones a verificar primero
        let mut verificaciones: Vec<(String, String)> = Vec::new();
        for pred in self.predicciones.iter() {
            if let Some(entidad) = self.entidades.get(&pred.entidad_id) {
                let tiempo_transcurrido = now.saturating_sub(pred.timestamp_ms);
                if tiempo_transcurrido >= pred.horizonte_ms {
                    verificaciones.push((pred.id.clone(), self.predecir_estado(entidad)));
                }
            }
        }

        // Ahora actualizar predicciones con los datos recolectados
        for (id_pred, estado_actual) in verificaciones {
            if let Some(pred) = self.predicciones.iter_mut().find(|p| p.id == id_pred) {
                let acierto = if estado_actual == pred.estado_predicho { 1.0 } else { 0.0 };
                pred.precision_real = Some(acierto);
                cumplidas.push(pred.clone());
            }
        }

        // Mover predicciones cumplidas al historial
        for c in &cumplidas {
            if let Some(idx) = self.predicciones.iter().position(|p| p.id == c.id) {
                self.predicciones.swap_remove(idx);
            }
        }

        self.predicciones_cumplidas.extend(cumplidas.clone());
        self.stats.predicciones_activas = self.predicciones.len();

        // Actualizar precisión promedio
        if !self.predicciones_cumplidas.is_empty() {
            let total: f64 = self.predicciones_cumplidas
                .iter()
                .filter_map(|p| p.precision_real)
                .sum();
            let count = self.predicciones_cumplidas
                .iter()
                .filter(|p| p.precision_real.is_some())
                .count() as f64;
            if count > 0.0 {
                self.stats.precision_promedio = total / count;
            }
        }

        cumplidas
    }

    /// Obtiene estadísticas del modelo
    pub fn stats(&self) -> WorldModelStats {
        let mut entidades_por_tipo = HashMap::new();
        for entidad in self.entidades.values() {
            *entidades_por_tipo.entry(entidad.tipo).or_insert(0) += 1;
        }

        WorldModelStats {
            total_entidades: self.entidades.len(),
            entidades_por_tipo,
            total_predicciones: self.stats.total_predicciones,
            predicciones_activas: self.stats.predicciones_activas,
            precision_promedio: self.stats.precision_promedio,
            ultima_actualizacion_ms: self.stats.ultima_actualizacion_ms,
            confianza_promedio: self.calcular_confianza_promedio(),
        }
    }

    fn calcular_confianza_promedio(&self) -> f64 {
        if self.entidades.is_empty() {
            return 0.5;
        }
        let total: f64 = self.entidades.values().map(|e| e.confianza).sum();
        total / self.entidades.len() as f64
    }

    /// Genera reporte del modelo
    pub fn reporte(&self) -> String {
        let mut s = String::new();
        s.push_str("=== WORLD MODEL DIGITAL ===\n");
        s.push_str(&format!("Entidades: {}\n", self.entidades.len()));
        s.push_str(&format!("Predicciones activas: {}\n", self.predicciones.len()));
        s.push_str(&format!("Confianza promedio: {:.2}\n", self.calcular_confianza_promedio()));
        s.push_str(&format!("Precisión promedio: {:.2}\n", self.stats.precision_promedio));

        for (tipo, count) in self.stats().entidades_por_tipo.iter() {
            s.push_str(&format!("  {}: {}\n", tipo.nombre(), count));
        }

        s
    }

    /// Encuentra relaciones entre entidades
    pub fn encontrar_relaciones(&self, entidad_id: &str) -> Vec<&DigitalEntity> {
        let entidad = match self.entidades.get(entidad_id) {
            Some(e) => e,
            None => return Vec::new(),
        };
        let mut relacionadas = Vec::new();

        for id in &entidad.relaciones {
            if let Some(ent) = self.entidades.get(id) {
                relacionadas.push(ent);
            }
        }

        relacionadas
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

fn generar_id_entidad() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let count = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("entity_{:016x}", count)
}

fn generar_id_prediccion() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let count = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("pred_{:016x}", count)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crear_entidad() {
        let entidad = DigitalEntity::nuevo(EntityType::Process, "test_proc".to_string());
        assert!(!entidad.id.is_empty());
        assert_eq!(entidad.tipo, EntityType::Process);
    }

    #[test]
    fn test_agregar_entidad() {
        let mut modelo = WorldModelDigital::new(ModelConfig::default());
        let entidad = DigitalEntity::nuevo(EntityType::File, "test.txt".to_string());
        let id = modelo.agregar_entidad(entidad);

        assert!(modelo.obtener_entidad(&id).is_some());
    }

    #[test]
    fn test_actualizar_entidad() {
        let mut modelo = WorldModelDigital::new(ModelConfig::default());
        let entidad = DigitalEntity::nuevo(EntityType::Process, "test".to_string());
        let id = modelo.agregar_entidad(entidad);

        let mut attrs = HashMap::new();
        attrs.insert("cpu".to_string(), "50".to_string());
        attrs.insert("mem".to_string(), "100".to_string());

        assert!(modelo.actualizar_entidad(&id, attrs));

        let ent = modelo.obtener_entidad(&id).unwrap();
        assert_eq!(ent.atributos.get("cpu"), Some(&"50".to_string()));
    }

    #[test]
    fn test_prediccion() {
        let mut modelo = WorldModelDigital::new(ModelConfig::default());
        let entidad = DigitalEntity::nuevo(EntityType::Process, "test".to_string());
        let id = modelo.agregar_entidad(entidad);

        // Agregar algo de historial
        for i in 0..5 {
            let mut attrs = HashMap::new();
            attrs.insert("count".to_string(), format!("{}", i));
            modelo.actualizar_entidad(&id, attrs);
        }

        let prediccion = modelo.predecir(&id, 60_000);
        assert!(prediccion.is_some());
    }

    #[test]
    fn test_modelo_estadisticas() {
        let modelo = WorldModelDigital::new(ModelConfig::default());
        let stats = modelo.stats();

        assert_eq!(stats.total_entidades, 0);
        assert!(stats.confianza_promedio >= 0.0);
    }
}