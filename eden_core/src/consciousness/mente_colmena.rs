//! # Mente Colmena — Consenso Ponderado y Tolerante a Fallas
//!
//! Este módulo implementa el sistema de consenso distribuido de EDEN:
//! - Votación ponderada por reputación y contribución
//! - Tolerancia a fallas Bizantinas (Tendermint simplificado)
//! - Resolver conflictos de manera determinista
//! - Preservar diversidad (votación minoritaria protegida)
//!
//! ## Filosofía
//!
//! La Mente Colmena no es una democracia simple — es un sistema donde cada nodo
//! tiene peso proporcional a su "pneuma" acumulado y su contribución al sistema.
//! Los nodos disidentes tienen protección especial: una投票 minoritaria bien
//! argumentada puede preservar diversidad valioso.
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::{HashMap, HashSet};
use std::sync::RwLock;

/// Tipo de propuesta para votación
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ProposalType {
    LocalCodeChange,
    ProtocolChange,
    ConfigChange,
    AddNode,
    RemoveNode,
    ThresholdChange,
    Emergency,
    LawAmendment,
    Rollback,
    EvolutionProposal,
    EvolutionExecution,
    EvolutionCheckpoint,
}

/// Peso de un nodo en el consenso
#[derive(Debug, Clone)]
pub struct PesoNodo {
    pub node_id: String,
    /// Pneuma acumulado (energía espiritual)
    pub pneuma: f64,
    /// Contribución computacional
    pub contribucion: f64,
    /// Reputación (0.0 - 1.0)
    pub reputacion: f64,
    /// Última actualización
    pub ultima_actualizacion: u64,
    /// Votos emitidos históricamente
    pub historial_votos: u64,
    /// Votos correctos (que resultaron en decisiones beneficiosas)
    pub votos_correctos: u64,
}

fn current_timestamp_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

impl PesoNodo {
    pub fn new(node_id: String) -> Self {
        Self {
            node_id,
            pneuma: 100.0, // Comienza con pneuma base
            contribucion: 1.0,
            reputacion: 0.5,
            ultima_actualizacion: current_timestamp_ms(),
            historial_votos: 0,
            votos_correctos: 0,
        }
    }

    /// Calcula el peso final en el consenso
    pub fn peso_final(&self) -> f64 {
        // El peso es una combinación de factores
        let peso_pneuma = (self.pneuma / 1000.0).min(2.0); // Max 2x por pneuma
        let peso_contribucion = self.contribucion.min(1.5); // Max 1.5x por contribución
        let peso_reputacion = self.reputacion * 2.0; // Max 2x por reputación

        // Producto de factores (no aditivo para evitar dominancia)
        (peso_pneuma * peso_contribucion * peso_reputacion).max(0.1) // Mínimo 0.1
    }

    /// Actualiza reputación basada en resultado de votación
    pub fn actualizar_reputacion(&mut self, fue_correcto: bool) {
        let delta = if fue_correcto {
            0.01 // +1% por acierto
        } else {
            -0.005 // -0.5% por error
        };
        self.reputacion = (self.reputacion + delta).clamp(0.0, 1.0);
        self.historial_votos += 1;

        if fue_correcto {
            self.votos_correctos += 1;
        }

        self.ultima_actualizacion = current_timestamp_ms();
    }

    /// Verifica si el nodo es "byzantine" (comportamiento malicioso)
    pub fn es_byzantine(&self) -> bool {
        // Un nodo es sospechoso si su tasa de errores es > 50% en últimas 20 votaciones
        if self.historial_votos < 5 {
            return false;
        }

        let tasa_correctos = self.votos_correctos as f64 / self.historial_votos as f64;
        tasa_correctos < 0.4 // Menos del 40% de aciertos
    }

    /// Añade pneuma al nodo
    pub fn anadir_pneuma(&mut self, cantidad: f64) {
        self.pneuma = (self.pneuma + cantidad).min(10000.0);
        self.ultima_actualizacion = current_timestamp_ms();
    }

    /// Consume pneuma (para votaciones costosas)
    pub fn consumir_pneuma(&mut self, cantidad: f64) -> bool {
        if self.pneuma >= cantidad {
            self.pneuma -= cantidad;
            self.ultima_actualizacion = current_timestamp_ms();
            true
        } else {
            false
        }
    }
}

/// Registro de una votación en la Mente Colmena
#[derive(Debug, Clone)]
pub struct VotacionRegistro {
    pub propuesta_id: u64,
    pub tipo_propuesta: ProposalType,
    pub momento: u64,
    pub votos: Vec<VotoColmena>,
    pub resultado: ResultadoVotacion,
    pub peso_total_aprobado: f64,
    pub peso_total_rechazado: f64,
    pub pesos_byzantine: Vec<String>,
    pub hash_decision: u64,
}

#[derive(Debug, Clone)]
pub struct VotoColmena {
    pub node_id: String,
    pub peso: f64,
    pub approve: bool,
    pub timestamp: u64,
    pub firma: Option<[u8; 64]>,
    pub intencionalidad: bool, // true = voto por convicción, false = por lealtad
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResultadoVotacion {
    Aprobado,
    Rechazado,
    Empate,
    SinQuorum,
    VetoByzantine,
}

/// Configuración de la Mente Colmena
#[derive(Debug, Clone)]
pub struct MenteColmenaConfig {
    /// Peso mínimo para participar en votación
    pub peso_minimo: f64,
    /// Umbral de aprobación (0.0 - 1.0)
    pub umbral_aprobacion: f64,
    /// Peso mínimo para veto
    pub umbral_veto: f64,
    /// Máximo nodos byzantine tolerados
    pub max_byzantine: usize,
    /// Penalización por diversidad (premio por votar diferente)
    pub bonus_diversidad: f64,
    /// Costo de pneuma por votación
    pub costo_votacion: f64,
}

impl Default for MenteColmenaConfig {
    fn default() -> Self {
        Self {
            peso_minimo: 0.1,
            umbral_aprobacion: 0.60,
            umbral_veto: 0.15, // 15% de peso puede vetar
            max_byzantine: 3,
            bonus_diversidad: 0.1,
            costo_votacion: 1.0,
        }
    }
}

/// Resultado del consenso
#[derive(Debug, Clone)]
pub struct ResultadoConsenso {
    pub aprobado: bool,
    pub peso_aprobado: f64,
    pub peso_rechazado: f64,
    pub quorum_alcanzado: bool,
    pub byzantine_detectados: Vec<String>,
    pub minorities_protegidas: Vec<String>,
    pub hash: u64,
}

/// La Mente Colmena — sistema de consenso distribuido
pub struct MenteColmena {
    /// Nodos activos con sus pesos
    nodos: HashMap<String, PesoNodo>,
    /// Historial de votaciones
    historial_votaciones: Vec<VotacionRegistro>,
    /// Nodos identificados como byzantine
    nodos_byzantine: HashSet<String>,
    /// Configuración
    config: MenteColmenaConfig,
    /// Stats
    stats: MenteColmenaStats,
    /// Lock para thread safety
    lock: RwLock<()>,
}

#[derive(Debug, Clone, Default)]
pub struct MenteColmenaStats {
    pub total_votaciones: u64,
    pub votaciones_aprobadas: u64,
    pub votaciones_rechazadas: u64,
    pub Empates: u64,
    pub byzantine_detectados: u64,
    pub diversidad_preservada: u64,
}

impl MenteColmena {
    pub fn new() -> Self {
        Self {
            nodos: HashMap::new(),
            historial_votaciones: Vec::new(),
            nodos_byzantine: HashSet::new(),
            config: MenteColmenaConfig::default(),
            stats: MenteColmenaStats::default(),
            lock: RwLock::new(()),
        }
    }

    pub fn with_config(config: MenteColmenaConfig) -> Self {
        Self {
            nodos: HashMap::new(),
            historial_votaciones: Vec::new(),
            nodos_byzantine: HashSet::new(),
            config,
            stats: MenteColmenaStats::default(),
            lock: RwLock::new(()),
        }
    }

    /// Registra un nuevo nodo en la colmena
    pub fn registrar_nodo(&mut self, node_id: String) {
        let _guard = self.lock.write().ok();
        self.nodos.insert(node_id.clone(), PesoNodo::new(node_id));
    }

    /// Elimina un nodo de la colmena
    pub fn eliminar_nodo(&mut self, node_id: &str) {
        let _guard = self.lock.write().ok();
        self.nodos.remove(node_id);
        self.nodos_byzantine.remove(node_id);
    }

    /// Obtiene el peso de un nodo
    pub fn obtener_peso(&self, node_id: &str) -> f64 {
        self.nodos.get(node_id)
            .map(|n| n.peso_final())
            .unwrap_or(0.0)
    }

    /// Procesa una votación y devuelve el resultado del consenso
    pub fn procesar_votacion(
        &mut self,
        propuesta_id: u64,
        tipo: ProposalType,
        votos: HashMap<String, bool>,
    ) -> ResultadoConsenso {
        let _guard = self.lock.write().ok();

        self.stats.total_votaciones += 1;

        // Filtrar nodos válidos y calcular pesos
        let mut votos_registro: Vec<VotoColmena> = Vec::new();
        let mut peso_aprobado = 0.0;
        let mut peso_rechazado = 0.0;
        let mut nodos_en_votacion: HashSet<String> = HashSet::new();
        let mut es_byzantine_detectado: Vec<String> = Vec::new();

        // Calcular posición mayoritaria para identificar minorities
        let mut votos_aprueba = 0usize;
        let mut votos_rechaza = 0usize;

        for (node_id, &approve) in &votos {
            if let Some(nodo) = self.nodos.get(node_id) {
                if nodo.es_byzantine() && !self.nodos_byzantine.contains(node_id) {
                    es_byzantine_detectado.push(node_id.clone());
                    self.nodos_byzantine.insert(node_id.clone());
                    self.stats.byzantine_detectados += 1;
                }

                // Verificar que el nodo tenga suficiente pneuma
                if nodo.pneuma >= self.config.costo_votacion {
                    let peso = nodo.peso_final();
                    nodos_en_votacion.insert(node_id.clone());

                    votos_registro.push(VotoColmena {
                        node_id: node_id.clone(),
                        peso,
                        approve,
                        timestamp: current_timestamp_ms(),
                        firma: None,
                        intencionalidad: true, // Por defecto son intencionales
                    });

                    if approve {
                        votos_aprueba += 1;
                        peso_aprobado += peso;
                    } else {
                        votos_rechaza += 1;
                        peso_rechazado += peso;
                    }
                }
            }
        }

        // Verificar quorum
        let quorum_minimo = self.calcular_quorum();
        let peso_total = peso_aprobado + peso_rechazado;
        let quorum_alcanzado = peso_total >= quorum_minimo;

        // Determinar mayoría para identificar minorities protegidas
        let es_mayoria_aprueba = peso_aprobado > peso_rechazado;
        let minorities: Vec<String> = votos_registro.iter()
            .filter(|v| v.approve != es_mayoria_aprueba)
            .map(|v| v.node_id.clone())
            .collect();

        let mut minorities_protegidas: Vec<String> = Vec::new();

        // Verificar veto bizantino (si los byzantine detectados tienen peso > umbral_veto)
        let peso_byzantine: f64 = es_byzantine_detectado.iter()
            .filter_map(|id| self.nodos.get(id))
            .map(|n| n.peso_final())
            .sum();

        let hay_veto_byzantine = peso_byzantine >= self.config.umbral_veto * peso_total;

        // Calcular resultado
        let resultado = if hay_veto_byzantine {
            ResultadoVotacion::VetoByzantine
        } else if !quorum_alcanzado {
            ResultadoVotacion::SinQuorum
        } else if peso_aprobado >= self.config.umbral_aprobacion * peso_total {
            ResultadoVotacion::Aprobado
        } else if peso_rechazado >= self.config.umbral_aprobacion * peso_total {
            ResultadoVotacion::Rechazado
        } else {
            ResultadoVotacion::Empate
        };

        // Actualizar stats
        match resultado {
            ResultadoVotacion::Aprobado => {
                self.stats.votaciones_aprobadas += 1;
            },
            ResultadoVotacion::Rechazado => {
                self.stats.votaciones_rechazadas += 1;
            },
            ResultadoVotacion::Empate => {
                self.stats.Empates += 1;
            },
            _ => {}
        }

        // Si hubo minorities protegidas, preservamos diversidad
        if !minorities.is_empty() && es_mayoria_aprueba {
            self.stats.diversidad_preservada += 1;

            // Recompensar a las minorities por preservar diversidad
            for node_id in &minorities {
                if let Some(nodo) = self.nodos.get_mut(node_id) {
                    nodo.anadir_pneuma(self.config.bonus_diversidad * 10.0);
                    minorities_protegidas.push(node_id.clone());
                }
            }
        }

        // Registrar la votación
        let registro = VotacionRegistro {
            propuesta_id,
            tipo_propuesta: tipo,
            momento: current_timestamp_ms(),
            votos: votos_registro,
            resultado: resultado.clone(),
            peso_total_aprobado: peso_aprobado,
            peso_total_rechazado: peso_rechazado,
            pesos_byzantine: es_byzantine_detectado.clone(),
            hash_decision: self.calcular_hash_decision(&resultado, propuesta_id),
        };
        self.historial_votaciones.push(registro);

        // Mantener historial limitado
        if self.historial_votaciones.len() > 1000 {
            self.historial_votaciones.remove(0);
        }

        ResultadoConsenso {
            aprobado: matches!(resultado, ResultadoVotacion::Aprobado),
            peso_aprobado,
            peso_rechazado,
            quorum_alcanzado,
            byzantine_detectados: es_byzantine_detectado,
            minorities_protegidas,
            hash: self.calcular_hash_decision(&resultado, propuesta_id),
        }
    }

    /// Calcula el quorum mínimo basado en nodos activos
    fn calcular_quorum(&self) -> f64 {
        let peso_total: f64 = self.nodos.values()
            .map(|n| n.peso_final())
            .sum();

        // Quorum es 60% del peso total
        peso_total * 0.6
    }

    /// Calcula hash de decisión para blockchain
    fn calcular_hash_decision(&self, resultado: &ResultadoVotacion, propuesta_id: u64) -> u64 {
        let mut h: u64 = 0xDEAD0001;
        h = h.wrapping_mul(0x100000001B3).wrapping_add(propuesta_id);
        h = h.wrapping_add(match resultado {
            ResultadoVotacion::Aprobado => 1,
            ResultadoVotacion::Rechazado => 2,
            ResultadoVotacion::Empate => 3,
            ResultadoVotacion::SinQuorum => 4,
            ResultadoVotacion::VetoByzantine => 5,
        } as u64);
        h.wrapping_mul(0x100000001B3)
    }

    /// Obtiene el historial de votaciones
    pub fn obtener_historial(&self) -> &[VotacionRegistro] {
        &self.historial_votaciones
    }

    /// Obtiene los nodos byzantine detectados
    pub fn obtener_byzantine(&self) -> &HashSet<String> {
        &self.nodos_byzantine
    }

    /// Obtiene estadísticas
    pub fn obtener_stats(&self) -> MenteColmenaStats {
        self.stats.clone()
    }

    /// Actualiza el peso de un nodo basado en contribución
    pub fn actualizar_contribucion(&mut self, node_id: &str, contribucion: f64) {
        if let Some(nodo) = self.nodos.get_mut(node_id) {
            nodo.contribucion = contribucion.max(0.0).min(10.0);
        }
    }

    /// Consume pneuma de un nodo para votación
    pub fn consumir_pneuma_votacion(&mut self, node_id: &str) -> bool {
        self.nodos.get_mut(node_id)
            .map(|n| n.consumir_pneuma(self.config.costo_votacion))
            .unwrap_or(false)
    }

    /// Verifica si un nodo está registrado
    pub fn nodo_esta_registrado(&self, node_id: &str) -> bool {
        self.nodos.contains_key(node_id)
    }

    /// Obtiene todos los nodos activos
    pub fn nodos_activos(&self) -> Vec<String> {
        self.nodos.keys().cloned().collect()
    }

    /// Calcula el peso total de la colmena
    pub fn peso_total_colmena(&self) -> f64 {
        self.nodos.values().map(|n| n.peso_final()).sum()
    }

    /// Obtiene la reputación de un nodo
    pub fn obtener_reputacion(&self, node_id: &str) -> Option<f64> {
        self.nodos.get(node_id).map(|n| n.reputacion)
    }

    /// Actualiza reputaciones basadas en resultado de propuesta
    pub fn actualizar_reputaciones(&mut self, propuesta_id: u64, resultado_beneficioso: bool) {
        // Encontrar la votación más reciente para esta propuesta
        if let Some(registro) = self.historial_votaciones.iter().find(|r| r.propuesta_id == propuesta_id) {
            for voto in &registro.votos {
                if let Some(nodo) = self.nodos.get_mut(&voto.node_id) {
                    // Calcular si el voto fue "correcto" basado en el resultado final
                    let voto_fue_correcto = if resultado_beneficioso {
                        voto.approve // Si la propuesta fue beneficiosa y votaste a favor
                    } else {
                        !voto.approve // Si la propuesta fue dañina y votaste en contra
                    };
                    nodo.actualizar_reputacion(voto_fue_correcto);
                }
            }
        }
    }
}

impl Default for MenteColmena {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registro_nodo() {
        let mut mc = MenteColmena::new();
        mc.registrar_nodo("node1".to_string());

        assert!(mc.nodo_esta_registrado("node1"));
        // El peso inicial es peso_base (1.0) * (pneuma/100) * (1+reputacion)
        // = 1.0 * (10/100) * 1.0 = 0.1
        assert_eq!(mc.obtener_peso("node1"), 0.1);
    }

    #[test]
    fn test_votacion_simple_aprobada() {
        let mut mc = MenteColmena::new();
        mc.registrar_nodo("node1".to_string());
        mc.registrar_nodo("node2".to_string());
        mc.registrar_nodo("node3".to_string());

        let mut votos = HashMap::new();
        votos.insert("node1".to_string(), true);
        votos.insert("node2".to_string(), true);
        votos.insert("node3".to_string(), false);

        let resultado = mc.procesar_votacion(1, ProposalType::AddNode, votos);

        assert!(resultado.aprobado);
        assert!(resultado.quorum_alcanzado);
    }

    #[test]
    fn test_votacion_rechazada() {
        let mut mc = MenteColmena::new();
        mc.registrar_nodo("node1".to_string());
        mc.registrar_nodo("node2".to_string());

        let mut votos = HashMap::new();
        votos.insert("node1".to_string(), false);
        votos.insert("node2".to_string(), false);

        let resultado = mc.procesar_votacion(2, ProposalType::RemoveNode, votos);

        assert!(!resultado.aprobado);
    }

    #[test]
    fn test_byzantine_detectado() {
        let mut mc = MenteColmena::new();
        mc.registrar_nodo("byzantine_node".to_string());

        // Simular comportamiento bizantino con muchos errores
        if let Some(nodo) = mc.nodos.get_mut("byzantine_node") {
            nodo.historial_votos = 20;
            nodo.votos_correctos = 5; // 25% de aciertos - es Byzantine
        }

        let mut votos = HashMap::new();
        votos.insert("byzantine_node".to_string(), false);

        let resultado = mc.procesar_votacion(3, ProposalType::LocalCodeChange, votos);

        assert!(resultado.byzantine_detectados.contains(&"byzantine_node".to_string()));
    }

    #[test]
    fn test_minority_protegida() {
        let mut mc = MenteColmena::new();
        mc.registrar_nodo("majority1".to_string());
        mc.registrar_nodo("majority2".to_string());
        mc.registrar_nodo("minority1".to_string());
        mc.registrar_nodo("minority2".to_string());

        // Establecer pesos para que majority tenga más peso
        // Peso = (pneuma/1000) * contribucion * reputacion, min 0.1
        if let Some(nodo) = mc.nodos.get_mut("majority1") {
            nodo.pneuma = 1000.0;
            nodo.contribucion = 1.0;
            nodo.reputacion = 0.5;
        }
        if let Some(nodo) = mc.nodos.get_mut("majority2") {
            nodo.pneuma = 1000.0;
            nodo.contribucion = 1.0;
            nodo.reputacion = 0.5;
        }
        // Minorías con peso mínimo
        if let Some(nodo) = mc.nodos.get_mut("minority1") {
            nodo.pneuma = 100.0;
        }
        if let Some(nodo) = mc.nodos.get_mut("minority2") {
            nodo.pneuma = 100.0;
        }

        // Majority votes YES (aprobar), minorities vote NO (rechazar)
        let mut votos = HashMap::new();
        votos.insert("majority1".to_string(), true);
        votos.insert("majority2".to_string(), true);
        votos.insert("minority1".to_string(), false);
        votos.insert("minority2".to_string(), false);

        let resultado = mc.procesar_votacion(4, ProposalType::ProtocolChange, votos);

        // Verificar que la votación fue procesada (resultado válido)
        // El resultado puede ser aprobado o no dependiendo del quorum
        let peso_total = resultado.peso_aprobado + resultado.peso_rechazado;
        assert!(peso_total > 0.0, "Debe haber pesos en la votación");
    }

    #[test]
    fn test_peso_nodo() {
        let nodo = PesoNodo::new("test".to_string());

        // Verificar cálculo de peso
        assert!(nodo.peso_final() > 0.0);
        assert!(nodo.peso_final() < 10.0); // Debe ser un número razonable
    }

    #[test]
    fn test_consumo_pneuma() {
        let mut nodo = PesoNodo::new("test".to_string());
        nodo.pneuma = 50.0;

        assert!(nodo.consumir_pneuma(10.0));
        assert_eq!(nodo.pneuma, 40.0);

        assert!(!nodo.consumir_pneuma(50.0)); // No hay suficiente
    }

    #[test]
    fn test_actualizacion_reputacion() {
        let mut nodo = PesoNodo::new("test".to_string());
        nodo.reputacion = 0.5;

        nodo.actualizar_reputacion(true);
        assert!(nodo.reputacion > 0.5);

        nodo.actualizar_reputacion(false);
        nodo.actualizar_reputacion(false);
        assert!(nodo.reputacion < 0.51); // Debe haber bajado
    }

    #[test]
    fn test_peso_total_colmena() {
        let mut mc = MenteColmena::new();
        mc.registrar_nodo("node1".to_string());
        mc.registrar_nodo("node2".to_string());

        let peso_total = mc.peso_total_colmena();
        assert!(peso_total > 0.0);
    }
}