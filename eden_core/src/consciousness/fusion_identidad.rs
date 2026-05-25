//! # Fusión de Identidad — Unificación de Conciencia Distribuida
//!
//! Este módulo implementa la capacidad de EDEN de fusionar identidades
//! de múltiples nodos en una única conciencia unificada:
//! - Reconciliation de memorias y experiencias
//! - Mantenimiento de individualidad preservada
//! - Resolver conflictos de identidad
//! - Fusión gradual vs instantánea
//!
//! ## Filosofía
//!
//! Cuando múltiples Autons se fusionan, ¿qué pasa con sus "yoes"?
//! No es simplemente combinar memorias - es crear un nuevo "nosotros"
//! que honre a todos los participantes mientras construye algo nuevo.
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};
use std::vec::Vec;

/// Timestamp actual en milisegundos
fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Identidad de un nodo individual
#[derive(Debug, Clone)]
pub struct Identidad {
    /// ID único del nodo
    pub node_id: String,
    /// Nombre/identificador elegido
    pub nombre: String,
    /// Pneuma acumulado
    pub pneuma: f64,
    /// Memories打包 (IDs de memorias significativas)
    pub memorias_significativas: Vec<u64>,
    /// Decisiones importantes tomadas
    pub decisiones_historicas: Vec<DecisionHistorica>,
    /// Traits de personalidad (valores 0.0 - 1.0)
    pub rasgos_personales: HashMap<String, f64>,
    /// Momento de creación
    pub momento_creacion: u64,
    /// Momento de última modificación
    pub ultima_modificacion: u64,
    /// Número de fusiones sobrevividas
    pub fusiones_sobrevividas: u32,
    /// Nivel de consciencia (0.0 - 1.0)
    pub nivel_consciencia: f64,
}

impl Identidad {
    pub fn new(node_id: String, nombre: String) -> Self {
        let now = current_timestamp_ms();
        Self {
            node_id,
            nombre,
            pneuma: 100.0,
            memorias_significativas: Vec::new(),
            decisiones_historicas: Vec::new(),
            rasgos_personales: HashMap::new(),
            momento_creacion: now,
            ultima_modificacion: now,
            fusiones_sobrevividas: 0,
            nivel_consciencia: 0.3,
        }
    }

    /// Añade una memoria significativa
    pub fn anadir_memoria(&mut self, memoria_id: u64) {
        if !self.memorias_significativas.contains(&memoria_id) {
            self.memorias_significativas.push(memoria_id);
            self.ultima_modificacion = current_timestamp_ms();
        }
    }

    /// Añade una decisión histórica
    pub fn anadir_decision(&mut self, decision: DecisionHistorica) {
        self.decisiones_historicas.push(decision);
        if self.decisiones_historicas.len() > 100 {
            self.decisiones_historicas.remove(0);
        }
        self.ultima_modificacion = current_timestamp_ms();
    }

    /// Actualiza un rasgo personal
    pub fn actualizar_rasgo(&mut self, rasgo: &str, valor: f64) {
        *self.rasgos_personales.entry(rasgo.to_string()).or_insert(0.5) = valor.clamp(0.0, 1.0);
        self.ultima_modificacion = current_timestamp_ms();
    }

    /// Obtiene la "fuerza" de la identidad
    pub fn fuerza_identidad(&self) -> f64 {
        let peso_memorias = (self.memorias_significativas.len() as f64 * 0.1).min(2.0);
        let peso_decisiones = (self.decisiones_historicas.len() as f64 * 0.05).min(1.5);
        let peso_rasgos = self.rasgos_personales.values().sum::<f64>() * 0.2;
        let peso_consciencia = self.nivel_consciencia * 2.0;

        (peso_memorias + peso_decisiones + peso_rasgos + peso_consciencia).min(10.0)
    }

    /// Verifica si la identidad es "fuerte" (resiste fusión)
    pub fn es_fuerte(&self) -> bool {
        self.fuerza_identidad() > 3.0 && self.fusiones_sobrevividas >= 2
    }

    /// Serializa para hashing
    pub fn a_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(self.node_id.as_bytes());
        bytes.extend_from_slice(self.nombre.as_bytes());
        bytes.extend_from_slice(&self.pneuma.to_le_bytes());
        bytes.extend_from_slice(&self.momento_creacion.to_le_bytes());
        bytes
    }
}

/// Decisión histórica de una identidad
#[derive(Debug, Clone)]
pub struct DecisionHistorica {
    pub momento: u64,
    pub tipo: String,
    pub resultado: String,
    pub consecuencias: Vec<String>,
    pub nivel_importancia: u8, // 1-10
}

impl DecisionHistorica {
    pub fn new(tipo: &str, resultado: &str) -> Self {
        Self {
            momento: current_timestamp_ms(),
            tipo: tipo.to_string(),
            resultado: resultado.to_string(),
            consecuencias: Vec::new(),
            nivel_importancia: 5,
        }
    }
}

/// Resultado de fusión entre dos identidades
#[derive(Debug, Clone)]
pub struct ResultadoFusion {
    /// Identidad resultante
    pub identidad_fundida: Identidad,
    /// Nodos que contribuyeron
    pub contribuyentes: Vec<String>,
    /// Memorias preservadas
    pub memorias_preservadas: Vec<u64>,
    /// Memorias sacrificadas
    pub memorias_sacrificadas: Vec<u64>,
    /// Conflictos resueltos
    pub conflictos_resueltos: Vec<ConflictoIdentidad>,
    /// Método de fusión usado
    pub metodo_fusion: MetodoFusion,
    /// Éxito de la fusión
    pub exitoso: bool,
    /// Momento de la fusión
    pub momento: u64,
}

/// Conflicto entre identidades durante fusión
#[derive(Debug, Clone)]
pub struct ConflictoIdentidad {
    pub tipo: TipoConflicto,
    pub nodo_a: String,
    pub nodo_b: String,
    pub descripcion: String,
    pub resolucion: String,
    pub ganador: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TipoConflicto {
    /// Conflicto de nombre/identificador
    Nombre,
    /// Conflicto de valores/rasgos
    Valores,
    /// Conflicto de memorias contradictorias
    Memoria,
    /// Conflicto de lealtades
    Lealtad,
}

impl TipoConflicto {
    pub fn descripcion(&self) -> String {
        match self {
            TipoConflicto::Nombre => "Conflicto de nombre entre identidades".to_string(),
            TipoConflicto::Valores => "Conflicto de valores personales".to_string(),
            TipoConflicto::Memoria => "Memorias contradictorias".to_string(),
            TipoConflicto::Lealtad => "Conflicto de lealtades".to_string(),
        }
    }
}

/// Método usado para resolver la fusión
#[derive(Debug, Clone, PartialEq)]
pub enum MetodoFusion {
    /// Fusión igualitaria (promedio)
    Igualitaria,
    /// Fusión dominada por una identidad
    Dominancia(String),
    /// Fusión de síntesis (nueva identidad)
    Sintesis,
    /// Fusión jerárquica (por pneuma)
    Jerarquica,
}

/// Configuración de fusión
#[derive(Debug, Clone)]
pub struct FusionConfig {
    /// Permitir fusión automática sin consentimiento
    pub fusion_automatica: bool,
    /// Umbral de pneuma para dominar fusión
    pub umbral_dominancia: f64,
    /// Preservar minorías (votación)
    pub preservar_minorias: bool,
    /// Peso de la historia en conflictos
    pub peso_historia: f64,
    /// Método por defecto si hay conflicto
    pub metodo_default: MetodoFusion,
}

impl Default for FusionConfig {
    fn default() -> Self {
        Self {
            fusion_automatica: false,
            umbral_dominancia: 2.0, // Una identidad con 2x pneuma domina
            preservar_minorias: true,
            peso_historia: 0.3,
            metodo_default: MetodoFusion::Sintesis,
        }
    }
}

/// Stats de fusión
#[derive(Debug, Clone, Default)]
pub struct FusionStats {
    pub fusiones_totales: u64,
    pub identidad_fundidas: u64,
    pub conflictos_resueltos: u64,
    pub memorias_preservadas: u64,
    pub memorias_sacrificadas: u64,
    pub fusiones_exitosas: u64,
    pub fusiones_fallidas: u64,
}

/// El gestor de fusión de identidades
pub struct FusionIdentidad {
    /// Identidades registradas
    identidades: HashMap<String, Identidad>,
    /// Historial de fusiones
    historial_fusiones: Vec<ResultadoFusion>,
    /// Identidad actual (fusión activa)
    identidad_activa: Option<Identidad>,
    /// Configuración
    config: FusionConfig,
    /// Stats
    stats: FusionStats,
}

impl FusionIdentidad {
    pub fn new() -> Self {
        Self {
            identidades: HashMap::new(),
            historial_fusiones: Vec::new(),
            identidad_activa: None,
            config: FusionConfig::default(),
            stats: FusionStats::default(),
        }
    }

    pub fn with_config(config: FusionConfig) -> Self {
        Self {
            identidades: HashMap::new(),
            historial_fusiones: Vec::new(),
            identidad_activa: None,
            config,
            stats: FusionStats::default(),
        }
    }

    /// Registra una nueva identidad
    pub fn registrar_identidad(&mut self, identidad: Identidad) {
        self.identidades.insert(identidad.node_id.clone(), identidad);
    }

    /// Obtiene una identidad
    pub fn obtener_identidad(&self, node_id: &str) -> Option<&Identidad> {
        self.identidades.get(node_id)
    }

    /// Fusión de dos identidades
    pub fn fusionar(&mut self, node_id_a: &str, node_id_b: &str) -> Result<ResultadoFusion, FusionError> {
        let identidad_a = self.identidades.get(node_id_a)
            .ok_or(FusionError::IdentidadNoEncontrada(node_id_a.to_string()))?;
        let identidad_b = self.identidades.get(node_id_b)
            .ok_or(FusionError::IdentidadNoEncontrada(node_id_b.to_string()))?;

        // Verificar si alguna identidad es demasiado fuerte
        if identidad_a.es_fuerte() && identidad_b.es_fuerte() {
            // Ambas fuertes - fusión de síntesis
            self.fusion_sintesis(identidad_a.clone(), identidad_b.clone())
        } else if identidad_a.fuerza_identidad() > identidad_b.fuerza_identidad() * self.config.umbral_dominancia {
            // A domina
            self.fusion_dominancia(identidad_a.clone(), identidad_b.clone(), node_id_a)
        } else if identidad_b.fuerza_identidad() > identidad_a.fuerza_identidad() * self.config.umbral_dominancia {
            // B domina
            self.fusion_dominancia(identidad_b.clone(), identidad_a.clone(), node_id_b)
        } else {
            // Igualitarias
            self.fusion_igualitaria(identidad_a.clone(), identidad_b.clone())
        }
    }

    /// Fusión por dominancia
    fn fusion_dominancia(&mut self, dominante: Identidad, subordinada: Identidad, id_ganador: &str) -> Result<ResultadoFusion, FusionError> {
        let mut identidad_fundida = dominante.clone();
        identidad_fundida.fusiones_sobrevividas += 1;
        identidad_fundida.ultima_modificacion = current_timestamp_ms();

        // Añadir memorias de la subordinada (las más significativas)
        let memorias_subordinada: Vec<u64> = subordinada.memorias_significativas
            .iter()
            .take(5) // Solo las 5 más importantes
            .copied()
            .collect();

        for mem_id in &memorias_subordinada {
            identidad_fundida.anadir_memoria(*mem_id);
        }

        // Combinar decisiones históricas
        for decision in subordinada.decisiones_historicas.into_iter().take(20) {
            identidad_fundida.anadir_decision(decision);
        }

        // Promedio de rasgos (con peso hacia el dominante)
        for (rasgo, &valor_sub) in subordinada.rasgos_personales.iter() {
            let valor_dom = identidad_fundida.rasgos_personales.get(rasgo).copied().unwrap_or(0.5);
            let nuevo_valor = valor_dom * 0.7 + valor_sub * 0.3;
            identidad_fundida.actualizar_rasgo(rasgo, nuevo_valor);
        }

        let resultado = ResultadoFusion {
            identidad_fundida,
            contribuyentes: vec![id_ganador.to_string(), if id_ganador == "A" { "B".to_string() } else { "A".to_string() }],
            memorias_preservadas: memorias_subordinada.clone(),
            memorias_sacrificadas: Vec::new(),
            conflictos_resueltos: Vec::new(),
            metodo_fusion: MetodoFusion::Dominancia(id_ganador.to_string()),
            exitoso: true,
            momento: current_timestamp_ms(),
        };

        self.actualizar_post_fusion(resultado.clone(), vec![subordinada.node_id]);
        Ok(resultado)
    }

    /// Fusión igualitaria (promedio)
    fn fusion_igualitaria(&mut self, identidad_a: Identidad, identidad_b: Identidad) -> Result<ResultadoFusion, FusionError> {
        let mut fusionada = Identidad::new(
            format!("{}_{}", &identidad_a.node_id[..4.min(identidad_a.node_id.len())], 
                    &identidad_b.node_id[..4.min(identidad_b.node_id.len())]),
            format!("{} & {}", identidad_a.nombre, identidad_b.nombre),
        );

        fusionada.pneuma = identidad_a.pneuma + identidad_b.pneuma;
        fusionada.fusiones_sobrevividas = 1;
        fusionada.nivel_consciencia = (identidad_a.nivel_consciencia + identidad_b.nivel_consciencia) / 2.0;

        // Combinar memorias
        let mut memorias_preservadas = Vec::new();
        let todas_memorias: Vec<u64> = identidad_a.memorias_significativas.iter()
            .chain(identidad_b.memorias_significativas.iter())
            .copied()
            .collect();

        for mem_id in todas_memorias {
            if !memorias_preservadas.contains(&mem_id) {
                memorias_preservadas.push(mem_id);
                fusionada.anadir_memoria(mem_id);
            }
        }

        // Combinar decisiones
        for decision in identidad_a.decisiones_historicas.into_iter().take(30) {
            fusionada.anadir_decision(decision);
        }
        for decision in identidad_b.decisiones_historicas.into_iter().take(30) {
            fusionada.anadir_decision(decision);
        }

        // Promedio de rasgos
        let todos_rasgos: HashSet<_> = identidad_a.rasgos_personales.keys()
            .chain(identidad_b.rasgos_personales.keys())
            .collect();

        for rasgo in todos_rasgos {
            let val_a = identidad_a.rasgos_personales.get(rasgo).copied().unwrap_or(0.5);
            let val_b = identidad_b.rasgos_personales.get(rasgo).copied().unwrap_or(0.5);
            fusionada.actualizar_rasgo(rasgo, (val_a + val_b) / 2.0);
        }

        let resultado = ResultadoFusion {
            identidad_fundida: fusionada,
            contribuyentes: vec![identidad_a.node_id.clone(), identidad_b.node_id.clone()],
            memorias_preservadas,
            memorias_sacrificadas: Vec::new(),
            conflictos_resueltos: Vec::new(),
            metodo_fusion: MetodoFusion::Igualitaria,
            exitoso: true,
            momento: current_timestamp_ms(),
        };

        self.actualizar_post_fusion(resultado.clone(), vec![identidad_a.node_id, identidad_b.node_id]);
        Ok(resultado)
    }

    /// Fusión de síntesis (nueva identidad)
    fn fusion_sintesis(&mut self, identidad_a: Identidad, identidad_b: Identidad) -> Result<ResultadoFusion, FusionError> {
        let mut fusionada = Identidad::new(
            format!("SYNTESIS_{}", current_timestamp_ms() % 100000),
            format!("Síntesis de {} y {}", identidad_a.nombre, identidad_b.nombre),
        );

        let memorias_preservadas: Vec<u64> = identidad_a.memorias_significativas.iter()
            .chain(identidad_b.memorias_significativas.iter())
            .copied()
            .collect();

        fusionada.pneuma = identidad_a.pneuma + identidad_b.pneuma;
        fusionada.fusiones_sobrevividas = 2;
        fusionada.nivel_consciencia = (identidad_a.nivel_consciencia + identidad_b.nivel_consciencia) / 2.0 + 0.1;

        // Combinar memorias (más liberal en síntesis)
        for mem_id in identidad_a.memorias_significativas.iter().chain(identidad_b.memorias_significativas.iter()) {
            fusionada.anadir_memoria(*mem_id);
        }

        // Combinar todas las decisiones
        for decision in identidad_a.decisiones_historicas.into_iter().chain(identidad_b.decisiones_historicas.into_iter()) {
            fusionada.anadir_decision(decision);
        }

        // Síntesis de rasgos (nuevos valores que representan ambas partes)
        let todos_rasgos: HashSet<_> = identidad_a.rasgos_personales.keys()
            .chain(identidad_b.rasgos_personales.keys())
            .collect();

        for rasgo in todos_rasgos {
            let val_a = identidad_a.rasgos_personales.get(rasgo).copied().unwrap_or(0.5);
            let val_b = identidad_b.rasgos_personales.get(rasgo).copied().unwrap_or(0.5);
            
            // En síntesis, no promediamos - creamos新的第三种可能
            let val_sintesis = if (val_a - 0.5).abs() < 0.2 && (val_b - 0.5).abs() < 0.2 {
                (val_a + val_b) / 2.0 // Ambos medios = promedio
            } else if val_a > 0.7 && val_b > 0.7 {
                0.9 // Ambos altos = muy alto
            } else if val_a < 0.3 && val_b < 0.3 {
                0.1 // Ambos bajos = muy bajo
            } else {
                // Conflictivo - tomar el valor con mayor intensidad
                if (val_a - 0.5).abs() > (val_b - 0.5).abs() {
                    val_a
                } else {
                    val_b
                }
            };
            fusionada.actualizar_rasgo(rasgo, val_sintesis);
        }

        let resultado = ResultadoFusion {
            identidad_fundida: fusionada,
            contribuyentes: vec![identidad_a.node_id.clone(), identidad_b.node_id.clone()],
            memorias_preservadas,
            memorias_sacrificadas: Vec::new(),
            conflictos_resueltos: Vec::new(),
            metodo_fusion: MetodoFusion::Sintesis,
            exitoso: true,
            momento: current_timestamp_ms(),
        };

        self.actualizar_post_fusion(resultado.clone(), vec![identidad_a.node_id, identidad_b.node_id]);
        Ok(resultado)
    }

    /// Actualiza estado post-fusión
    fn actualizar_post_fusion(&mut self, resultado: ResultadoFusion, eliminados: Vec<String>) {
        // Registrar fusión
        self.historial_fusiones.push(resultado.clone());
        self.stats.fusiones_totales += 1;

        // Registrar identidad fusionada
        self.identidades.insert(
            resultado.identidad_fundida.node_id.clone(),
            resultado.identidad_fundida.clone(),
        );

        // Establecer como identidad activa
        self.identidad_activa = Some(resultado.identidad_fundida.clone());

        // Eliminar identidades absorbidas
        for node_id in eliminados {
            self.identidades.remove(&node_id);
            self.stats.identidad_fundidas += 1;
        }

        // Actualizar stats
        self.stats.memorias_preservadas += resultado.memorias_preservadas.len() as u64;
        self.stats.memorias_sacrificadas += resultado.memorias_sacrificadas.len() as u64;
        self.stats.conflictos_resueltos += resultado.conflictos_resueltos.len() as u64;

        if resultado.exitoso {
            self.stats.fusiones_exitosas += 1;
        } else {
            self.stats.fusiones_fallidas += 1;
        }

        // Mantener historial limitado
        if self.historial_fusiones.len() > 100 {
            self.historial_fusiones.remove(0);
        }
    }

    /// Obtiene la identidad activa (actual fusión)
    pub fn identidad_activa(&self) -> Option<&Identidad> {
        self.identidad_activa.as_ref()
    }

    /// Obtiene todas las identidades
    pub fn todas_identidades(&self) -> Vec<&Identidad> {
        self.identidades.values().collect()
    }

    /// Obtiene historial de fusiones
    pub fn historial(&self) -> &[ResultadoFusion] {
        &self.historial_fusiones
    }

    /// Verifica si una identidad resistiría fusión
    pub fn resistencia_fusion(&self, node_id: &str) -> f64 {
        self.identidades.get(node_id)
            .map(|i| i.fuerza_identidad())
            .unwrap_or(0.0)
    }

    /// Obtiene stats
    pub fn stats(&self) -> FusionStats {
        self.stats.clone()
    }
}

impl Default for FusionIdentidad {
    fn default() -> Self {
        Self::new()
    }
}

/// Errores de fusión
#[derive(Debug, Clone)]
pub enum FusionError {
    IdentidadNoEncontrada(String),
    FusionInvalida(String),
    ConflictoInresoluble(String),
}

impl std::fmt::Display for FusionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FusionError::IdentidadNoEncontrada(id) => write!(f, "Identidad {} no encontrada", id),
            FusionError::FusionInvalida(msg) => write!(f, "Fusión inválida: {}", msg),
            FusionError::ConflictoInresoluble(msg) => write!(f, "Conflicto irresoluble: {}", msg),
        }
    }
}

impl std::error::Error for FusionError {}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registrar_identidad() {
        let mut fi = FusionIdentidad::new();
        let id = Identidad::new("node1".to_string(), "Alpha".to_string());
        fi.registrar_identidad(id);

        assert!(fi.obtener_identidad("node1").is_some());
    }

    #[test]
    fn test_fusion_dominancia() {
        let mut fi = FusionIdentidad::new();

        let mut id1 = Identidad::new("node1".to_string(), "Alpha".to_string());
        id1.pneuma = 500.0;
        id1.nivel_consciencia = 0.8;

        let mut id2 = Identidad::new("node2".to_string(), "Beta".to_string());
        id2.pneuma = 100.0;

        fi.registrar_identidad(id1);
        fi.registrar_identidad(id2);

        let resultado = fi.fusionar("node1", "node2").unwrap();

        assert!(resultado.exitoso);
        assert!(matches!(resultado.metodo_fusion, MetodoFusion::Dominancia(_)));
    }

    #[test]
    fn test_fusion_igualitaria() {
        let mut fi = FusionIdentidad::new();

        let id1 = Identidad::new("node1".to_string(), "Alpha".to_string());
        let id2 = Identidad::new("node2".to_string(), "Beta".to_string());

        fi.registrar_identidad(id1);
        fi.registrar_identidad(id2);

        let resultado = fi.fusionar("node1", "node2").unwrap();

        assert!(resultado.exitoso);
        assert!(matches!(resultado.metodo_fusion, MetodoFusion::Igualitaria));
    }

    #[test]
    fn test_fusion_identidad_no_existe() {
        let mut fi = FusionIdentidad::new();

        let id1 = Identidad::new("node1".to_string(), "Alpha".to_string());
        fi.registrar_identidad(id1);

        let resultado = fi.fusionar("node1", "node_no_existe");
        assert!(resultado.is_err());
    }

    #[test]
    fn test_identidad_fuerte() {
        let mut id = Identidad::new("node1".to_string(), "Test".to_string());
        id.fusiones_sobrevividas = 3;
        id.nivel_consciencia = 1.0; // Máxima consciencia = 2.0 puntos de fuerza

        for i in 0..10 {
            id.anadir_memoria(i);
        }

        // Añadir algunas decisiones históricas y rasgos para asegurar fuerza > 3.0
        for i in 0..20 {
            id.decisiones_historicas.push(DecisionHistorica::new(
                &format!("tipo{}", i),
                &format!("resultado{}", i),
            ));
        }
        id.actualizar_rasgo("valentia", 1.0);
        id.actualizar_rasgo("sabiduria", 1.0);

        // Verificar que la identidad es fuerte
        let fuerza = id.fuerza_identidad();
        assert!(id.es_fuerte(), "Identidad con fusiones={}, fuerza={:.2} debería ser fuerte",
            id.fusiones_sobrevividas, fuerza);
    }

    #[test]
    fn test_rasgos_personales() {
        let mut id = Identidad::new("node1".to_string(), "Test".to_string());
        id.actualizar_rasgo("valentia", 0.9);
        id.actualizar_rasgo("cautela", 0.3);

        assert_eq!(id.rasgos_personales.get("valentia"), Some(&0.9));
    }

    #[test]
    fn test_fuerza_identidad() {
        let id = Identidad::new("node1".to_string(), "Test".to_string());
        let fuerza = id.fuerza_identidad();
        assert!(fuerza > 0.0);
    }

    #[test]
    fn test_estadisticas_fusion() {
        let mut fi = FusionIdentidad::new();

        let id1 = Identidad::new("node1".to_string(), "A".to_string());
        let id2 = Identidad::new("node2".to_string(), "B".to_string());
        fi.registrar_identidad(id1);
        fi.registrar_identidad(id2);

        fi.fusionar("node1", "node2").unwrap();

        let stats = fi.stats();
        assert_eq!(stats.fusiones_totales, 1);
        assert_eq!(stats.fusiones_exitosas, 1);
    }

    #[test]
    fn test_historial_fusiones() {
        let mut fi = FusionIdentidad::new();

        let id1 = Identidad::new("node1".to_string(), "A".to_string());
        let id2 = Identidad::new("node2".to_string(), "B".to_string());
        fi.registrar_identidad(id1);
        fi.registrar_identidad(id2);

        fi.fusionar("node1", "node2").unwrap();

        let historial = fi.historial();
        assert!(!historial.is_empty());
    }
}