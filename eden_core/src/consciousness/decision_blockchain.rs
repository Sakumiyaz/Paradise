//! # Decision Blockchain — Log Inmutable de Decisiones
//!
//! Este módulo implementa una blockchain simplificada para registrar
//! todas las decisiones de EDEN de manera inmutable:
//! - Bloques con hash encadenados
//! - Transacciones de decisión (propuestas, votaciones, resultados)
//! - Merkle tree para verificación de integridad
//! - Sincronización entre nodos
//!
//! ## Filosofía
//!
//! Cada decisión importante deja un registro permanente. La blockchain
//! de decisiones permite auditar el pasado de EDEN, verificar la
//! cadena de decisiones, y reconstruir el estado mental colectivo.
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Timestamp actual en milisegundos
fn current_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Genera ID único para transacción
fn generar_id_transaccion() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    let now = current_timestamp_ms();
    let count = COUNTER.fetch_add(1, Ordering::Relaxed);
    (now << 20) ^ (count & 0xFFFFF)
}

/// Tipo de transacción en la blockchain
#[derive(Debug, Clone, PartialEq)]
pub enum TipoTransaccion {
    /// Inicio de propuesta
    PropuestaCreada {
        propuesta_id: u64,
        tipo: String,
        proponente: String,
    },
    /// Voto de un nodo
    VotoEmitido {
        propuesta_id: u64,
        nodo: String,
        aprueba: bool,
        peso: f64,
    },
    /// Resultado de votación
    VotacionCerrada {
        propuesta_id: u64,
        resultado: String,
        peso_aprobado: f64,
        peso_rechazado: f64,
    },
    /// Aprobación del Creador
    CreatorAprobado {
        propuesta_id: u64,
    },
    /// Veto del Creador
    CreatorVetado {
        propuesta_id: u64,
        razon: String,
    },
    /// Ejecución de propuesta
    PropuestaEjecutada {
        propuesta_id: u64,
    },
    /// Rollback de propuesta
    PropuestaRevertida {
        propuesta_id: u64,
        razon: String,
    },
    /// Checkpoint de estado
    Checkpoint {
        altura: u64,
        estado_hash: u64,
    },
    /// Evolución completada
    EvolucionCompletada {
        evol_id: u64,
        modulo: String,
    },
    /// Fusión de identidad
    IdentidadFusionada {
        nodo_origen: String,
        nodo_destino: String,
    },
}

impl TipoTransaccion {
    /// Serializa la transacción a bytes para hashing
    pub fn a_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        match self {
            TipoTransaccion::PropuestaCreada { propuesta_id, tipo, proponente } => {
                bytes.extend_from_slice(b"PROP");
                bytes.extend_from_slice(&propuesta_id.to_le_bytes());
                bytes.extend_from_slice(tipo.as_bytes());
                bytes.extend_from_slice(proponente.as_bytes());
            },
            TipoTransaccion::VotoEmitido { propuesta_id, nodo, aprueba, peso } => {
                bytes.extend_from_slice(b"VOTO");
                bytes.extend_from_slice(&propuesta_id.to_le_bytes());
                bytes.extend_from_slice(nodo.as_bytes());
                bytes.push(if *aprueba { 1 } else { 0 });
                bytes.extend_from_slice(&((*peso * 1000.0) as u64).to_le_bytes());
            },
            TipoTransaccion::VotacionCerrada { propuesta_id, resultado, peso_aprobado, peso_rechazado } => {
                bytes.extend_from_slice(b"CLOS");
                bytes.extend_from_slice(&propuesta_id.to_le_bytes());
                bytes.extend_from_slice(resultado.as_bytes());
                bytes.extend_from_slice(&(*peso_aprobado as u64).to_le_bytes());
                bytes.extend_from_slice(&(*peso_rechazado as u64).to_le_bytes());
            },
            TipoTransaccion::CreatorAprobado { propuesta_id } => {
                bytes.extend_from_slice(b"CYES");
                bytes.extend_from_slice(&propuesta_id.to_le_bytes());
            },
            TipoTransaccion::CreatorVetado { propuesta_id, razon } => {
                bytes.extend_from_slice(b"CVET");
                bytes.extend_from_slice(&propuesta_id.to_le_bytes());
                bytes.extend_from_slice(razon.as_bytes());
            },
            TipoTransaccion::PropuestaEjecutada { propuesta_id } => {
                bytes.extend_from_slice(b"EXEC");
                bytes.extend_from_slice(&propuesta_id.to_le_bytes());
            },
            TipoTransaccion::PropuestaRevertida { propuesta_id, razon } => {
                bytes.extend_from_slice(b"RVRT");
                bytes.extend_from_slice(&propuesta_id.to_le_bytes());
                bytes.extend_from_slice(razon.as_bytes());
            },
            TipoTransaccion::Checkpoint { altura, estado_hash } => {
                bytes.extend_from_slice(b"CHKP");
                bytes.extend_from_slice(&altura.to_le_bytes());
                bytes.extend_from_slice(&estado_hash.to_le_bytes());
            },
            TipoTransaccion::EvolucionCompletada { evol_id, modulo } => {
                bytes.extend_from_slice(b"EVOL");
                bytes.extend_from_slice(&evol_id.to_le_bytes());
                bytes.extend_from_slice(modulo.as_bytes());
            },
            TipoTransaccion::IdentidadFusionada { nodo_origen, nodo_destino } => {
                bytes.extend_from_slice(b"FUSE");
                bytes.extend_from_slice(nodo_origen.as_bytes());
                bytes.extend_from_slice(nodo_destino.as_bytes());
            },
        }
        bytes
    }
}

/// Una transacción en la blockchain
#[derive(Debug, Clone)]
pub struct Transaccion {
    /// ID único de la transacción
    pub id: u64,
    /// Tipo de transacción
    pub tipo: TipoTransaccion,
    /// Timestamp de creación
    pub timestamp: u64,
    /// Nodo que creó la transacción
    pub nodo_origen: String,
    /// Firma de la transacción (opcional)
    pub firma: Option<[u8; 64]>,
    /// Nonce para proof of work
    pub nonce: u64,
}

impl Transaccion {
    pub fn new(tipo: TipoTransaccion, nodo_origen: String) -> Self {
        Self {
            id: generar_id_transaccion(),
            tipo,
            timestamp: current_timestamp_ms(),
            nodo_origen,
            firma: None,
            nonce: 0,
        }
    }

    /// Calcula el hash de la transacción
    pub fn hash(&self) -> u64 {
        let mut h: u64 = 0xDEAD0002;
        h = h.wrapping_mul(0x100000001B3).wrapping_add(self.id);
        h = h.wrapping_mul(0x100000001B3).wrapping_add(self.timestamp);

        // Incluir bytes de la transacción
        for byte in self.tipo.a_bytes() {
            h = h.wrapping_mul(0x100000001B3).wrapping_add(byte as u64);
        }

        // Incluir nonce
        h = h.wrapping_mul(0x100000001B3).wrapping_add(self.nonce);

        h
    }

    /// Ejecuta proof of work simple (SHA-like simplificado)
    pub fn proof_of_work(&mut self, dificultad: u8) {
        let target = 0xFFFFFFFFFFFFFFFF >> dificultad;

        while self.nonce < u64::MAX {
            let hash = self.hash();
            if hash <= target {
                break;
            }
            self.nonce += 1;
        }
    }
}

/// Bloque en la blockchain
#[derive(Debug, Clone)]
pub struct Bloque {
    /// Altura del bloque en la cadena
    pub altura: u64,
    /// Hash del bloque anterior
    pub hash_anterior: u64,
    /// Hash del bloque actual
    pub hash: u64,
    /// Transacciones en este bloque
    pub transacciones: Vec<Transaccion>,
    /// Timestamp de creación
    pub timestamp: u64,
    /// Raíz del Merkle tree
    pub merkle_root: u64,
    /// Nonce para proof of work
    pub nonce: u64,
    /// Número de versión del protocolo
    pub version: u8,
}

impl Bloque {
    pub fn new(altura: u64, hash_anterior: u64) -> Self {
        Self {
            altura,
            hash_anterior,
            hash: 0,
            transacciones: Vec::new(),
            timestamp: current_timestamp_ms(),
            merkle_root: 0,
            nonce: 0,
            version: 1,
        }
    }

    /// Añade una transacción al bloque
    pub fn anadir_transaccion(&mut self, transaccion: Transaccion) {
        self.transacciones.push(transaccion);
        self.recalcular_merkle();
    }

    /// Recalcula la raíz del Merkle tree
    fn recalcular_merkle(&mut self) {
        if self.transacciones.is_empty() {
            self.merkle_root = 0;
            return;
        }

        let mut hashes: Vec<u64> = self.transacciones.iter()
            .map(|t| t.hash())
            .collect();

        while hashes.len() > 1 {
            if hashes.len() % 2 == 1 {
                hashes.push(hashes.last().copied().unwrap_or(0));
            }

            let mut nueva_nivel: Vec<u64> = Vec::new();
            for chunk in hashes.chunks(2) {
                let combined = chunk[0].wrapping_mul(0x100000001B3).wrapping_add(chunk[1]);
                nueva_nivel.push(combined);
            }
            hashes = nueva_nivel;
        }

        self.merkle_root = hashes.first().copied().unwrap_or(0);
    }

    /// Calcula el hash del bloque
    pub fn calcular_hash(&self) -> u64 {
        let mut h: u64 = 0xDEAD0003;
        h = h.wrapping_mul(0x100000001B3).wrapping_add(self.altura);
        h = h.wrapping_mul(0x100000001B3).wrapping_add(self.hash_anterior);
        h = h.wrapping_mul(0x100000001B3).wrapping_add(self.merkle_root);
        h = h.wrapping_mul(0x100000001B3).wrapping_add(self.timestamp);
        h = h.wrapping_mul(0x100000001B3).wrapping_add(self.nonce as u64);
        h.wrapping_mul(0x100000001B3)
    }

    /// Finaliza el bloque (calcula hash y proof of work)
    pub fn finalizar(&mut self, dificultad: u8) {
        self.recalcular_merkle();
        self.hash = self.calcular_hash();

        // Proof of work simplificado
        let target = 0xFFFFFFFFFFFFFFFF >> dificultad;
        while self.hash > target && self.nonce < u64::MAX {
            self.nonce += 1;
            self.hash = self.calcular_hash();
        }
    }

    /// Verifica la integridad del bloque
    pub fn verificar(&self) -> bool {
        let hash_calculado = self.calcular_hash();
        hash_calculado == self.hash && self.altura > 0
    }

    /// Serializa el bloque a bytes
    pub fn a_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.altura.to_le_bytes());
        bytes.extend_from_slice(&self.hash_anterior.to_le_bytes());
        bytes.extend_from_slice(&self.hash.to_le_bytes());
        bytes.extend_from_slice(&self.merkle_root.to_le_bytes());
        bytes.extend_from_slice(&self.timestamp.to_le_bytes());
        bytes.extend_from_slice(&self.nonce.to_le_bytes());
        bytes.push(self.version);
        bytes
    }
}

/// Blockchain de decisiones
pub struct DecisionBlockchain {
    /// Cadena de bloques
    cadena: Vec<Bloque>,
    /// Transacciones pendientes (no confirmadas)
    mempool: Vec<Transaccion>,
    /// Índices para búsqueda rápida
    indice_propuestas: HashMap<u64, u64>, // propuesta_id -> altura_bloque
    indice_nodos: HashMap<String, Vec<u64>>, // nodo -> IDs de transacciones
    /// Configuración
    config: BlockchainConfig,
    /// Stats
    stats: BlockchainStats,
}

#[derive(Debug, Clone)]
pub struct BlockchainConfig {
    /// Dificultad de proof of work
    pub dificultad: u8,
    /// Bloques máximo en memoria
    pub max_bloques_en_memoria: usize,
    /// Tamaño máximo de bloque
    pub max_transacciones_por_bloque: usize,
    /// Tiempo mínimo entre bloques (ms)
    pub intervalo_bloques_ms: u64,
}

impl Default for BlockchainConfig {
    fn default() -> Self {
        Self {
            dificultad: 8, // Requiere ~256 iteraciones por hash
            max_bloques_en_memoria: 10000,
            max_transacciones_por_bloque: 100,
            intervalo_bloques_ms: 1000, // 1 segundo mínimo
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct BlockchainStats {
    pub bloques_totales: u64,
    pub transacciones_totales: u64,
    pub propuestas_registradas: u64,
    pub rollbacks_registrados: u64,
    pub tamanio_bytes: u64,
}

impl DecisionBlockchain {
    pub fn new() -> Self {
        let mut blockchain = Self {
            cadena: Vec::new(),
            mempool: Vec::new(),
            indice_propuestas: HashMap::new(),
            indice_nodos: HashMap::new(),
            config: BlockchainConfig::default(),
            stats: BlockchainStats::default(),
        };

        // Crear bloque génesis
        let genesis = Bloque::new(0, 0);
        blockchain.cadena.push(genesis);

        blockchain
    }

    pub fn with_config(config: BlockchainConfig) -> Self {
        let mut blockchain = Self {
            cadena: Vec::new(),
            mempool: Vec::new(),
            indice_propuestas: HashMap::new(),
            indice_nodos: HashMap::new(),
            config,
            stats: BlockchainStats::default(),
        };

        let genesis = Bloque::new(0, 0);
        blockchain.cadena.push(genesis);

        blockchain
    }

    /// Añade una transacción a la mempool
    pub fn anadir_transaccion(&mut self, transaccion: Transaccion) -> Result<(), BlockchainError> {
        if self.mempool.len() >= self.config.max_transacciones_por_bloque {
            return Err(BlockchainError::MempoolLlena);
        }

        // Verificar si es una transacción relacionada con propuesta
        if let Some(prop_id) = self.obtener_propuesta_id(&transaccion.tipo) {
            if self.indice_propuestas.contains_key(&prop_id) {
                return Err(BlockchainError::PropuestaYaRegistrada(prop_id));
            }
        }

        self.mempool.push(transaccion);
        self.stats.transacciones_totales += 1;

        Ok(())
    }

    /// Obtiene propuesta_id de una transacción
    fn obtener_propuesta_id(&self, tipo: &TipoTransaccion) -> Option<u64> {
        match tipo {
            TipoTransaccion::PropuestaCreada { propuesta_id, .. } => Some(*propuesta_id),
            TipoTransaccion::VotoEmitido { propuesta_id, .. } => Some(*propuesta_id),
            TipoTransaccion::VotacionCerrada { propuesta_id, .. } => Some(*propuesta_id),
            TipoTransaccion::CreatorAprobado { propuesta_id } => Some(*propuesta_id),
            TipoTransaccion::CreatorVetado { propuesta_id, .. } => Some(*propuesta_id),
            TipoTransaccion::PropuestaEjecutada { propuesta_id } => Some(*propuesta_id),
            TipoTransaccion::PropuestaRevertida { propuesta_id, .. } => Some(*propuesta_id),
            _ => None,
        }
    }

    /// Finaliza un bloque con las transacciones pendientes
    pub fn finalizar_bloque(&mut self) -> Result<u64, BlockchainError> {
        if self.mempool.is_empty() {
            return Err(BlockchainError::SinTransacciones);
        }

        let altura = self.cadena.len() as u64;
        let hash_anterior = self.cadena.last().map(|b| b.hash).unwrap_or(0);

        let mut nuevo_bloque = Bloque::new(altura, hash_anterior);

        // Añadir transacciones hasta el límite
        let max_tx = self.config.max_transacciones_por_bloque;
        let txs_to_add: Vec<_> = self.mempool.drain(..max_tx.min(self.mempool.len())).collect();

        for tx in txs_to_add {
            // Actualizar índices
            if let Some(prop_id) = Self::extraer_propuesta_id(&tx.tipo) {
                self.indice_propuestas.insert(prop_id, altura);
            }

            if let Some(nodos) = Self::extraer_nodos(&tx) {
                for nodo in nodos {
                    self.indice_nodos
                        .entry(nodo)
                        .or_insert_with(Vec::new)
                        .push(tx.id);
                }
            }

            nuevo_bloque.anadir_transaccion(tx);
        }

        nuevo_bloque.finalizar(self.config.dificultad);
        self.cadena.push(nuevo_bloque.clone());
        self.stats.bloques_totales += 1;

        // Limpiar cadena si creció demasiado
        if self.cadena.len() > self.config.max_bloques_en_memoria {
            self.cadena.remove(0);
        }

        self.stats.tamanio_bytes = self.calcular_tamanio_bytes();

        Ok(nuevo_bloque.hash)
    }

    /// Extrae propuesta_id de tipo de transacción
    fn extraer_propuesta_id(tipo: &TipoTransaccion) -> Option<u64> {
        match tipo {
            TipoTransaccion::PropuestaCreada { propuesta_id, .. } => Some(*propuesta_id),
            TipoTransaccion::VotoEmitido { propuesta_id, .. } => Some(*propuesta_id),
            TipoTransaccion::VotacionCerrada { propuesta_id, .. } => Some(*propuesta_id),
            TipoTransaccion::CreatorAprobado { propuesta_id } => Some(*propuesta_id),
            TipoTransaccion::CreatorVetado { propuesta_id, .. } => Some(*propuesta_id),
            TipoTransaccion::PropuestaEjecutada { propuesta_id } => Some(*propuesta_id),
            TipoTransaccion::PropuestaRevertida { propuesta_id, .. } => Some(*propuesta_id),
            _ => None,
        }
    }

    /// Extrae nodos involucrados de una transacción
    fn extraer_nodos(transaccion: &Transaccion) -> Option<Vec<String>> {
        match &transaccion.tipo {
            TipoTransaccion::PropuestaCreada { proponente, .. } => Some(vec![proponente.clone()]),
            TipoTransaccion::VotoEmitido { nodo, .. } => Some(vec![nodo.clone()]),
            TipoTransaccion::CreatorVetado { .. } => Some(vec!["CREATOR".to_string()]),
            TipoTransaccion::IdentidadFusionada { nodo_origen, nodo_destino } => {
                Some(vec![nodo_origen.clone(), nodo_destino.clone()])
            },
            _ => None,
        }
    }

    /// Obtiene un bloque por altura
    pub fn obtener_bloque(&self, altura: u64) -> Option<&Bloque> {
        self.cadena.get(altura as usize)
    }

    /// Obtiene el bloque más reciente
    pub fn ultimo_bloque(&self) -> Option<&Bloque> {
        self.cadena.last()
    }

    /// Obtiene transacciones de una propuesta
    pub fn obtener_transacciones_propuesta(&self, propuesta_id: u64) -> Vec<&Transaccion> {
        self.cadena.iter()
            .flat_map(|b| b.transacciones.iter())
            .filter(|t| {
                Self::extraer_propuesta_id(&t.tipo) == Some(propuesta_id)
            })
            .collect()
    }

    /// Obtiene transacciones de un nodo
    pub fn obtener_transacciones_nodo(&self, nodo_id: &str) -> Vec<&Transaccion> {
        self.cadena.iter()
            .flat_map(|b| b.transacciones.iter())
            .filter(|t| t.nodo_origen == nodo_id)
            .collect()
    }

    /// Verifica la cadena completa
    pub fn verificar_cadena(&self) -> Result<(), BlockchainError> {
        if self.cadena.is_empty() {
            return Err(BlockchainError::CadenaVacia);
        }

        // Verificar bloque génesis
        if self.cadena[0].altura != 0 || self.cadena[0].hash_anterior != 0 {
            return Err(BlockchainError::BloqueInvalido(0));
        }

        // Verificar cada bloque subsiguiente
        for i in 1..self.cadena.len() {
            let bloque = &self.cadena[i];

            // Verificar altura y hash anterior
            if bloque.altura != i as u64 {
                return Err(BlockchainError::BloqueInvalido(i as u64));
            }

            if bloque.hash_anterior != self.cadena[i - 1].hash {
                return Err(BlockchainError::BloqueInvalido(i as u64));
            }

            // Verificar hash del bloque
            if !bloque.verificar() {
                return Err(BlockchainError::BloqueInvalido(i as u64));
            }
        }

        Ok(())
    }

    /// Obtiene la altura actual de la cadena
    pub fn altura(&self) -> u64 {
        self.cadena.len() as u64 - 1 // -1 porque el índice 0 tiene altura 0
    }

    /// Obtiene el hash de un bloque específico
    pub fn hash_bloque(&self, altura: u64) -> Option<u64> {
        self.cadena.get(altura as usize).map(|b| b.hash)
    }

    /// Calcula el tamaño en bytes de la cadena
    fn calcular_tamanio_bytes(&self) -> u64 {
        let mut total = 0u64;
        for bloque in &self.cadena {
            total += bloque.a_bytes().len() as u64;
            total += bloque.transacciones.iter()
                .map(|t| t.tipo.a_bytes().len())
                .sum::<usize>() as u64;
        }
        total
    }

    /// Obtiene estadísticas
    pub fn obtener_stats(&self) -> BlockchainStats {
        self.stats.clone()
    }

    /// Busca transacción por ID
    pub fn buscar_transaccion(&self, id: u64) -> Option<&Transaccion> {
        self.cadena.iter()
            .flat_map(|b| b.transacciones.iter())
            .find(|t| t.id == id)
    }

    /// Obtiene el historial de decisiones de un tipo
    pub fn historial_tipo(&self, tipo: &str) -> Vec<&Transaccion> {
        self.cadena.iter()
            .flat_map(|b| b.transacciones.iter())
            .filter(|t| Self::tipo_a_string(&t.tipo) == tipo)
            .collect()
    }

    /// Convierte tipo de transacción a string
    fn tipo_a_string(tipo: &TipoTransaccion) -> String {
        match tipo {
            TipoTransaccion::PropuestaCreada { .. } => "PROP".to_string(),
            TipoTransaccion::VotoEmitido { .. } => "VOTO".to_string(),
            TipoTransaccion::VotacionCerrada { .. } => "CLOS".to_string(),
            TipoTransaccion::CreatorAprobado { .. } => "CYES".to_string(),
            TipoTransaccion::CreatorVetado { .. } => "CVET".to_string(),
            TipoTransaccion::PropuestaEjecutada { .. } => "EXEC".to_string(),
            TipoTransaccion::PropuestaRevertida { .. } => "RVRT".to_string(),
            TipoTransaccion::Checkpoint { .. } => "CHKP".to_string(),
            TipoTransaccion::EvolucionCompletada { .. } => "EVOL".to_string(),
            TipoTransaccion::IdentidadFusionada { .. } => "FUSE".to_string(),
        }
    }
}

impl Default for DecisionBlockchain {
    fn default() -> Self {
        Self::new()
    }
}

/// Errores de la blockchain
#[derive(Debug, Clone)]
pub enum BlockchainError {
    MempoolLlena,
    PropuestaYaRegistrada(u64),
    SinTransacciones,
    CadenaVacia,
    BloqueInvalido(u64),
    HashNoCoincide,
}

impl std::fmt::Display for BlockchainError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockchainError::MempoolLlena => write!(f, "Mempool llena"),
            BlockchainError::PropuestaYaRegistrada(id) => write!(f, "Propuesta {} ya registrada", id),
            BlockchainError::SinTransacciones => write!(f, "No hay transacciones para crear bloque"),
            BlockchainError::CadenaVacia => write!(f, "La cadena está vacía"),
            BlockchainError::BloqueInvalido(h) => write!(f, "Bloque inválido en altura {}", h),
            BlockchainError::HashNoCoincide => write!(f, "Hash no coincide"),
        }
    }
}

impl std::error::Error for BlockchainError {}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crear_blockchain() {
        let bc = DecisionBlockchain::new();
        assert_eq!(bc.altura(), 0);
        assert!(bc.ultimo_bloque().is_some());
    }

    #[test]
    fn test_anadir_transaccion() {
        let mut bc = DecisionBlockchain::new();

        let tx = Transaccion::new(
            TipoTransaccion::PropuestaCreada {
                propuesta_id: 1,
                tipo: "ADD_NODE".to_string(),
                proponente: "node1".to_string(),
            },
            "node1".to_string(),
        );

        bc.anadir_transaccion(tx).unwrap();
        assert_eq!(bc.mempool.len(), 1);
    }

    #[test]
    fn test_finalizar_bloque() {
        let mut bc = DecisionBlockchain::new();

        for i in 1..=3 {
            let tx = Transaccion::new(
                TipoTransaccion::PropuestaCreada {
                    propuesta_id: i,
                    tipo: "TEST".to_string(),
                    proponente: format!("node{}", i),
                },
                format!("node{}", i),
            );
            bc.anadir_transaccion(tx).unwrap();
        }

        let hash = bc.finalizar_bloque().unwrap();
        assert!(hash > 0);
        assert_eq!(bc.altura(), 1);
        assert!(bc.mempool.is_empty());
    }

    #[test]
    fn test_verificar_cadena() {
        let mut bc = DecisionBlockchain::new();

        // Añadir transacciones y finalizar bloque
        let tx = Transaccion::new(
            TipoTransaccion::PropuestaCreada {
                propuesta_id: 1,
                tipo: "TEST".to_string(),
                proponente: "node1".to_string(),
            },
            "node1".to_string(),
        );
        bc.anadir_transaccion(tx).unwrap();
        bc.finalizar_bloque().unwrap();

        // Verificar cadena
        bc.verificar_cadena().unwrap(); // No debe fallar
    }

    #[test]
    fn test_obtener_transacciones_propuesta() {
        let mut bc = DecisionBlockchain::new();

        let prop_id = 42;

        // Añadir propuesta
        let tx1 = Transaccion::new(
            TipoTransaccion::PropuestaCreada {
                propuesta_id: prop_id,
                tipo: "TEST".to_string(),
                proponente: "node1".to_string(),
            },
            "node1".to_string(),
        );
        bc.anadir_transaccion(tx1).unwrap();

        // Añadir voto
        let tx2 = Transaccion::new(
            TipoTransaccion::VotoEmitido {
                propuesta_id: prop_id,
                nodo: "node2".to_string(),
                aprueba: true,
                peso: 1.0,
            },
            "node2".to_string(),
        );
        bc.anadir_transaccion(tx2).unwrap();

        bc.finalizar_bloque().unwrap();

        let txs = bc.obtener_transacciones_propuesta(prop_id);
        assert_eq!(txs.len(), 2);
    }

    #[test]
    fn test_hash_transaccion() {
        let tx = Transaccion::new(
            TipoTransaccion::PropuestaCreada {
                propuesta_id: 1,
                tipo: "TEST".to_string(),
                proponente: "node1".to_string(),
            },
            "node1".to_string(),
        );

        let hash = tx.hash();
        assert!(hash > 0);
    }

    #[test]
    fn test_merkle_tree() {
        let mut bloque = Bloque::new(1, 0);

        for i in 0..4 {
            let tx = Transaccion::new(
                TipoTransaccion::PropuestaCreada {
                    propuesta_id: i,
                    tipo: "TEST".to_string(),
                    proponente: format!("node{}", i),
                },
                format!("node{}", i),
            );
            bloque.anadir_transaccion(tx);
        }

        assert!(bloque.merkle_root > 0);
    }

    #[test]
    fn test_bloque_finalizado() {
        let mut bloque = Bloque::new(1, 0x1234);

        let tx = Transaccion::new(
            TipoTransaccion::Checkpoint {
                altura: 1,
                estado_hash: 0x5678,
            },
            "SYSTEM".to_string(),
        );
        bloque.anadir_transaccion(tx);

        bloque.finalizar(4);
        assert!(bloque.hash > 0);
        assert!(bloque.verificar());
    }

    #[test]
    fn test_transaccion_no_duplicada() {
        let mut bc = DecisionBlockchain::new();

        let tx = Transaccion::new(
            TipoTransaccion::PropuestaCreada {
                propuesta_id: 100,
                tipo: "TEST".to_string(),
                proponente: "node1".to_string(),
            },
            "node1".to_string(),
        );

        bc.anadir_transaccion(tx.clone()).unwrap();
        bc.finalizar_bloque().unwrap();

        // Intentar añadir la misma propuesta de nuevo (después de registrada)
        let tx2 = Transaccion::new(
            TipoTransaccion::PropuestaCreada {
                propuesta_id: 100,
                tipo: "TEST2".to_string(),
                proponente: "node2".to_string(),
            },
            "node2".to_string(),
        );

        let result = bc.anadir_transaccion(tx2);
        assert!(result.is_err());
    }

    #[test]
    fn test_estadisticas() {
        let mut bc = DecisionBlockchain::new();

        let tx = Transaccion::new(
            TipoTransaccion::PropuestaCreada {
                propuesta_id: 1,
                tipo: "TEST".to_string(),
                proponente: "node1".to_string(),
            },
            "node1".to_string(),
        );
        bc.anadir_transaccion(tx).unwrap();
        bc.finalizar_bloque().unwrap();

        let stats = bc.obtener_stats();
        assert_eq!(stats.bloques_totales, 1);
        assert!(stats.transacciones_totales > 0);
    }
}