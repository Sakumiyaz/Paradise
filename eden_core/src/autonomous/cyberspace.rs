//! # Cyberspace Module
//!
//! Network topology representation and mesh abstraction for autonomous systems.
//! This module provides real data structures and algorithms for representing
//! network nodes, connections, and operational vectors without performing actual
//! network activity.
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::{HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// TIPOS PRINCIPALES
// ============================================================================

/// Represents a discovered network node with its characteristics
#[derive(Debug, Clone)]
pub struct NodoRed {
    pub id: u64,
    pub ip: String,
    pub nombre: String,
    pub servicios: Vec<ServicioRed>,
    pub nivel_acceso: NivelAcceso,
    pub tipo: TipoNodo,
    pub latencia_estimada: f32,
    pub fecha_descubrimiento: u64,
    pub metricas: NodoMetricas,
}

impl NodoRed {
    /// Calculate node reachability score based on latency and access level
    pub fn calcular_reachability_score(&self) -> f32 {
        let access_score = match self.nivel_acceso {
            NivelAcceso::Root => 1.0,
            NivelAcceso::Admin => 0.8,
            NivelAcceso::Usuario => 0.5,
            NivelAcceso::Externo => 0.3,
            NivelAcceso::Desconocido => 0.1,
        };
        let latency_score = (100.0 - self.latencia_estimada.min(100.0)) / 100.0;
        access_score * 0.7 + latency_score * 0.3
    }

    /// Check if node has exploitable vulnerabilities
    pub fn tiene_vulnerabilidades(&self) -> bool {
        self.servicios.iter().any(|s| s.vulnerabilidad.is_some())
    }

    /// Get list of open ports
    pub fn puertos_abiertos(&self) -> Vec<u16> {
        self.servicios.iter().map(|s| s.puerto).collect()
    }
}

/// Node performance and health metrics
#[derive(Debug, Clone, Default)]
pub struct NodoMetricas {
    pub uptime_score: f32,
    pub congestion_level: f32,
    pub bandwidth_estimate: f32,
    pub last_activity: u64,
}

/// Network service running on a node
#[derive(Debug, Clone)]
pub struct ServicioRed {
    pub nombre: String,
    pub puerto: u16,
    pub protocolo: ProtocoloRed,
    pub version: String,
    pub vulnerabilidad: Option<String>,
    pub banner: Option<String>,
}

/// Network protocols
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ProtocoloRed {
    TCP,
    UDP,
    HTTP,
    HTTPS,
    SSH,
    FTP,
    DNS,
    SMTP,
    Custom(String),
}

/// Access privilege level on a node
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NivelAcceso {
    Desconocido,
    Externo,
    Usuario,
    Admin,
    Root,
}

impl NivelAcceso {
    /// Returns privilege multiplier for operations
    pub fn privilege_multiplier(&self) -> f32 {
        match self {
            NivelAcceso::Root => 1.0,
            NivelAcceso::Admin => 0.8,
            NivelAcceso::Usuario => 0.5,
            NivelAcceso::Externo => 0.2,
            NivelAcceso::Desconocido => 0.0,
        }
    }

    /// Compare privilege levels
    pub fn can_escalate_to(&self, target: &NivelAcceso) -> bool {
        let self_level = self.privilege_level();
        let target_level = target.privilege_level();
        target_level > self_level && target_level == self_level + 1
    }

    fn privilege_level(&self) -> u8 {
        match self {
            NivelAcceso::Root => 4,
            NivelAcceso::Admin => 3,
            NivelAcceso::Usuario => 2,
            NivelAcceso::Externo => 1,
            NivelAcceso::Desconocido => 0,
        }
    }
}

/// Type of network node
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TipoNodo {
    Servidor,
    Cliente,
    Router,
    Switch,
    Firewall,
    IoT,
    Database,
    Mainframe,
    Cloud,
    Desconocido,
}

impl TipoNodo {
    /// Determine node type based on services and characteristics
    pub fn inferir_tipo(servicios: &[ServicioRed], nombre: &str) -> Self {
        let port_set: HashSet<u16> = servicios.iter().map(|s| s.puerto).collect();

        if port_set.contains(&5432) || port_set.contains(&3306) || port_set.contains(&27017) {
            return TipoNodo::Database;
        }
        if port_set.contains(&22) && servicios.len() < 3 {
            return TipoNodo::Router;
        }
        if port_set.contains(&443) || port_set.contains(&80) {
            return TipoNodo::Servidor;
        }
        if nombre.to_lowercase().contains("iot") || servicios.is_empty() {
            return TipoNodo::IoT;
        }
        TipoNodo::Desconocido
    }
}

// ============================================================================
// NETWORK MESH
// ============================================================================

/// Real network mesh representing discovered topology
#[derive(Debug, Clone)]
pub struct NetworkMesh {
    /// Discovered nodes keyed by ID
    pub nodos: HashMap<u64, NodoRed>,
    /// Adjacency list of connections between nodes
    pub conexiones: HashMap<u64, Vec<u64>>,
    /// Connection weights (latency, reliability, etc.)
    pub pesos_conexiones: HashMap<(u64, u64), ConexionPeso>,
    /// Last mesh update timestamp
    pub ultimo_update: u64,
    /// Current mapping depth
    pub nivel_profundidad: u8,
    /// Mesh discovery timestamp
    pub created_at: u64,
}

impl Default for NetworkMesh {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkMesh {
    /// Create a new empty network mesh
    pub fn new() -> Self {
        let now = timestamp_unix();
        Self {
            nodos: HashMap::new(),
            conexiones: HashMap::new(),
            pesos_conexiones: HashMap::new(),
            ultimo_update: now,
            nivel_profundidad: 0,
            created_at: now,
        }
    }

    /// Add a node to the mesh
    pub fn agregar_nodo(&mut self, nodo: NodoRed) {
        self.nodos.insert(nodo.id, nodo);
        self.ultimo_update = timestamp_unix();
    }

    /// Add a bidirectional connection between two nodes
    pub fn agregar_conexion(&mut self, nodo_a: u64, nodo_b: u64, peso: ConexionPeso) {
        self.conexiones.entry(nodo_a).or_insert_with(Vec::new);
        self.conexiones.entry(nodo_b).or_insert_with(Vec::new);

        if !self.conexiones.get(&nodo_a).unwrap().contains(&nodo_b) {
            self.conexiones.get_mut(&nodo_a).unwrap().push(nodo_b);
        }
        if !self.conexiones.get(&nodo_b).unwrap().contains(&nodo_a) {
            self.conexiones.get_mut(&nodo_b).unwrap().push(nodo_a);
        }

        self.pesos_conexiones
            .insert((nodo_a.min(nodo_b), nodo_a.max(nodo_b)), peso);
        self.ultimo_update = timestamp_unix();
    }

    /// Get all nodes connected to a given node
    pub fn vecinos(&self, nodo_id: u64) -> Vec<u64> {
        self.conexiones.get(&nodo_id).cloned().unwrap_or_default()
    }

    /// Calculate shortest path between two nodes using BFS
    pub fn encontrar_ruta(&self, origen: u64, destino: u64) -> Option<Vec<u64>> {
        if origen == destino {
            return Some(vec![origen]);
        }

        let mut visitados = HashSet::new();
        let mut cola = Vec::new();
        let mut padre = HashMap::new();

        cola.push(origen);
        visitados.insert(origen);

        while let Some(actual) = cola.pop() {
            if actual == destino {
                let mut ruta = Vec::new();
                let mut nodo = destino;
                while nodo != origen {
                    ruta.push(nodo);
                    nodo = padre[&nodo];
                }
                ruta.push(origen);
                ruta.reverse();
                return Some(ruta);
            }

            if let Some(adyacentes) = self.conexiones.get(&actual) {
                for &adyacente in adyacentes {
                    if !visitados.contains(&adyacente) {
                        visitados.insert(adyacente);
                        padre.insert(adyacente, actual);
                        cola.push(adyacente);
                    }
                }
            }
        }

        None
    }

    /// Calculate node centrality (number of connections)
    pub fn centralidad(&self, nodo_id: u64) -> usize {
        self.conexiones.get(&nodo_id).map(|v| v.len()).unwrap_or(0)
    }

    /// Find nodes with highest centrality (key nodes in mesh)
    pub fn nodos_criticos(&self, cantidad: usize) -> Vec<u64> {
        let mut nodos: Vec<_> = self.nodos.keys().collect();
        nodos.sort_by(|a, b| self.centralidad(**b).cmp(&self.centralidad(**a)));
        nodos.into_iter().take(cantidad).copied().collect()
    }

    /// Get mesh statistics
    pub fn estadisticas(&self) -> MeshEstadisticas {
        let total_conexiones: usize = self.conexiones.values().map(|v| v.len()).sum();
        MeshEstadisticas {
            total_nodos: self.nodos.len(),
            total_conexiones: total_conexiones / 2,
            densidad: if self.nodos.len() > 1 {
                (total_conexiones as f64) / (self.nodos.len() * (self.nodos.len() - 1)) as f64
            } else {
                0.0
            },
            nodo_mas_conectado: self.nodos_criticos(1).first().copied(),
        }
    }
}

/// Weight/distance information for a connection
#[derive(Debug, Clone)]
pub struct ConexionPeso {
    pub latencia: f32,
    pub ancho_banda: f32,
    pub confiabilidad: f32,
    pub costo: f32,
}

impl Default for ConexionPeso {
    fn default() -> Self {
        Self {
            latencia: 10.0,
            ancho_banda: 100.0,
            confiabilidad: 0.95,
            costo: 1.0,
        }
    }
}

/// Mesh statistics
#[derive(Debug, Clone)]
pub struct MeshEstadisticas {
    pub total_nodos: usize,
    pub total_conexiones: usize,
    pub densidad: f64,
    pub nodo_mas_conectado: Option<u64>,
}

// ============================================================================
// INFILTRATION VECTOR
// ============================================================================

/// Represents an infiltration operation targeting a specific node
#[derive(Debug, Clone)]
pub struct VectorInfiltracion {
    pub id: u64,
    pub origen_id: u64,
    pub destino_id: u64,
    pub tecnica: TecnicaInfiltracion,
    pub progreso: f32,
    pub estado: EstadoInfiltracion,
    pub timestamp_creacion: u64,
    pub timestamp_update: u64,
    pub resultado: Option<ResultadoInfiltracion>,
}

impl VectorInfiltracion {
    /// Calculate effectiveness score based on technique and target characteristics
    pub fn calcular_efectividad(&self, destino: &NodoRed) -> f32 {
        let tecnica_score = self.tecnica.efectividad_base();
        let acceso_score = destino.nivel_acceso.privilege_multiplier();
        let vulnerability_bonus = if destino.tiene_vulnerabilidades() {
            0.2
        } else {
            0.0
        };
        (tecnica_score * 0.6 + acceso_score * 0.3 + vulnerability_bonus).min(1.0)
    }

    /// Update progression
    pub fn avanzar_progreso(&mut self, delta: f32) {
        self.progreso = (self.progreso + delta).min(1.0);
        self.timestamp_update = timestamp_unix();
    }

    /// Check if infiltration is complete
    pub fn esta_completo(&self) -> bool {
        self.progreso >= 1.0
    }
}

/// Techniques for network infiltration
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TecnicaInfiltracion {
    Exploit,
    Phishing,
    SocialEngineering,
    BruteForce,
    Backdoor,
    Rootkit,
    SupplyChain,
    ZeroDay,
}

impl TecnicaInfiltracion {
    /// Base effectiveness score for each technique
    pub fn efectividad_base(&self) -> f32 {
        match self {
            TecnicaInfiltracion::ZeroDay => 0.95,
            TecnicaInfiltracion::SupplyChain => 0.85,
            TecnicaInfiltracion::Backdoor => 0.75,
            TecnicaInfiltracion::Rootkit => 0.70,
            TecnicaInfiltracion::Exploit => 0.60,
            TecnicaInfiltracion::SocialEngineering => 0.55,
            TecnicaInfiltracion::BruteForce => 0.30,
            TecnicaInfiltracion::Phishing => 0.40,
        }
    }

    /// Get complexity level for the technique
    pub fn complejidad(&self) -> u8 {
        match self {
            TecnicaInfiltracion::ZeroDay => 5,
            TecnicaInfiltracion::SupplyChain => 4,
            TecnicaInfiltracion::Rootkit => 4,
            TecnicaInfiltracion::Exploit => 3,
            TecnicaInfiltracion::Backdoor => 2,
            TecnicaInfiltracion::SocialEngineering => 3,
            TecnicaInfiltracion::BruteForce => 1,
            TecnicaInfiltracion::Phishing => 2,
        }
    }
}

/// Current state of infiltration operation
#[derive(Debug, Clone, PartialEq)]
pub enum EstadoInfiltracion {
    Preparando,
    EnProgreso,
    Detectado,
    Bloqueado,
    Exitoso,
    Fallido,
}

/// Result of completed infiltration
#[derive(Debug, Clone)]
pub struct ResultadoInfiltracion {
    pub acceso_obtenido: NivelAcceso,
    pub datos_exfiltrados: Vec<String>,
    pub marca_temporal: u64,
}

// ============================================================================
// PROPAGATION VECTOR
// ============================================================================

/// Represents autonomous replication vector between nodes
#[derive(Debug, Clone)]
pub struct VectorPropagacion {
    pub id: u64,
    pub nodo_origen: u64,
    pub nodo_destino: u64,
    pub ruta: Vec<u64>,
    pub metrica_efectividad: f32,
    pub estado: EstadoPropagacion,
    pub timestamp_creacion: u64,
    pub timestamp_update: u64,
}

impl VectorPropagacion {
    /// Create new propagation vector with optimal route calculation
    pub fn nuevo(mesh: &NetworkMesh, origen: u64, destino: u64) -> Option<Self> {
        let ruta = mesh.encontrar_ruta(origen, destino)?;
        let metrica = Self::calcular_metrica_ruta(mesh, &ruta);

        Some(Self {
            id: 0,
            nodo_origen: origen,
            nodo_destino: destino,
            ruta,
            metrica_efectividad: metrica,
            estado: EstadoPropagacion::Pendiente,
            timestamp_creacion: timestamp_unix(),
            timestamp_update: timestamp_unix(),
        })
    }

    /// Calculate effectiveness metric for a route
    fn calcular_metrica_ruta(mesh: &NetworkMesh, ruta: &[u64]) -> f32 {
        if ruta.is_empty() {
            return 0.0;
        }

        let mut total_latencia = 0.0;
        let mut total_confiabilidad = 1.0;
        let hop_count = ruta.len() as f32;

        for window in ruta.windows(2) {
            let (a, b) = (window[0], window[1]);
            let key = (a.min(b), a.max(b));
            if let Some(peso) = mesh.pesos_conexiones.get(&key) {
                total_latencia += peso.latencia;
                total_confiabilidad *= peso.confiabilidad;
            }
        }

        let latency_score = (50.0 - total_latencia.min(50.0)) / 50.0;
        let reliability_score = total_confiabilidad;
        let hop_score = (10.0 - hop_count.min(10.0)) / 10.0;

        latency_score * 0.4 + reliability_score * 0.4 + hop_score * 0.2
    }
}

/// Propagation operation state
#[derive(Debug, Clone, PartialEq)]
pub enum EstadoPropagacion {
    Pendiente,
    EnTransito,
    Rechazado,
    Exitoso,
    Abandonado,
}

// ============================================================================
// TRAFFIC INTERCEPTION (Data Structure Representation)
// ============================================================================

/// Represents captured traffic metadata (no actual interception)
#[derive(Debug, Clone)]
pub struct IntercepcionTrafico {
    pub id: u64,
    pub nodo_origen: u64,
    pub nodo_destino: u64,
    pub contenido_hash: String,
    pub tipo_tráfico: TipoTrafico,
    pub metadatos: TraficoMetadatos,
    pub timestamp: u64,
}

/// Traffic metadata
#[derive(Debug, Clone)]
pub struct TraficoMetadatos {
    pub tamano_bytes: u64,
    pub ttl: u8,
    pub flags: HashSet<String>,
    pub puertos: (u16, u16),
}

/// Type of network traffic
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TipoTrafico {
    HTTP,
    HTTPS,
    DNS,
    SSH,
    FTP,
    SMTP,
    Email,
    Database,
    Custom(String),
}

impl TipoTrafico {
    /// Infer traffic type from port number
    pub fn desde_puerto(puerto: u16) -> Self {
        match puerto {
            80 => TipoTrafico::HTTP,
            443 => TipoTrafico::HTTPS,
            53 => TipoTrafico::DNS,
            22 => TipoTrafico::SSH,
            21 => TipoTrafico::FTP,
            25 | 587 => TipoTrafico::SMTP,
            3306 => TipoTrafico::Database,
            5432 => TipoTrafico::Database,
            _ => TipoTrafico::Custom(puerto.to_string()),
        }
    }
}

// ============================================================================
// CYBERSPACE ENGINE
// ============================================================================

/// Main cyberspace coordination engine
#[derive(Debug, Clone)]
pub struct Cyberspace {
    /// Network mesh representation
    pub mesh: NetworkMesh,
    /// Active infiltration vectors
    pub infiltraciones_activas: HashMap<u64, VectorInfiltracion>,
    /// Propagation vectors
    pub propagaciones: Vec<VectorPropagacion>,
    /// Captured traffic metadata
    pub intercepciones: Vec<IntercepcionTrafico>,
    /// Operation counter
    pub contador_operaciones: u64,
    /// Engine creation timestamp
    pub created_at: u64,
}

impl Default for Cyberspace {
    fn default() -> Self {
        Self::new()
    }
}

impl Cyberspace {
    /// Create new cyberspace engine
    pub fn new() -> Self {
        Self {
            mesh: NetworkMesh::new(),
            infiltraciones_activas: HashMap::new(),
            propagaciones: Vec::new(),
            intercepciones: Vec::new(),
            contador_operaciones: 0,
            created_at: timestamp_unix(),
        }
    }

    // ========================================================================
    // NODE MANAGEMENT
    // ========================================================================

    /// Register a new node in the mesh
    pub fn registrar_nodo(&mut self, mut nodo: NodoRed) -> u64 {
        self.contador_operaciones += 1;

        if nodo.id == 0 {
            nodo.id = self.contador_operaciones;
        }

        if nodo.tipo == TipoNodo::Desconocido {
            nodo.tipo = TipoNodo::inferir_tipo(&nodo.servicios, &nodo.nombre);
        }

        self.mesh.agregar_nodo(nodo.clone());
        nodo.id
    }

    /// Register connection between two nodes
    pub fn registrar_conexion(&mut self, origen: u64, destino: u64) {
        self.contador_operaciones += 1;
        self.mesh
            .agregar_conexion(origen, destino, ConexionPeso::default());
    }

    /// Discover node with full context
    pub fn descubrir_nodo(
        &mut self,
        ip: String,
        nombre: String,
        servicios: Vec<ServicioRed>,
        latencia: f32,
    ) -> u64 {
        self.contador_operaciones += 1;

        let nodo = NodoRed {
            id: self.contador_operaciones,
            ip: ip.clone(),
            nombre,
            servicios,
            nivel_acceso: NivelAcceso::Desconocido,
            tipo: TipoNodo::Desconocido,
            latencia_estimada: latencia,
            fecha_descubrimiento: timestamp_unix(),
            metricas: NodoMetricas::default(),
        };

        self.mesh.agregar_nodo(nodo.clone());

        // Collect IDs of nodes with similar latency for connection
        let nodos_cercanos: Vec<u64> = self
            .mesh
            .nodos
            .keys()
            .filter(|id| **id != nodo.id)
            .filter(|id| {
                if let Some(existing) = self.mesh.nodos.get(id) {
                    (existing.latencia_estimada - latencia).abs() < 20.0
                } else {
                    false
                }
            })
            .copied()
            .collect();

        // Connect to nearby nodes
        for id in nodos_cercanos {
            self.mesh
                .agregar_conexion(nodo.id, id, ConexionPeso::default());
        }

        nodo.id
    }

    // ========================================================================
    // NETWORK TOPOLOGY
    // ========================================================================

    /// Build mesh from discovered nodes
    pub fn construir_mesh(&mut self, nodos: Vec<NodoRed>) {
        for nodo in nodos {
            self.mesh.agregar_nodo(nodo);
        }
        self.mesh.nivel_profundidad = 1;
    }

    /// Map topology to adjacency structure
    pub fn mapear_topologia(&self) -> HashMap<u64, Vec<u64>> {
        self.mesh.conexiones.clone()
    }

    /// Get nodes reachable within N hops
    pub fn nodos_en_rango(&self, desde: u64, hops: usize) -> HashSet<u64> {
        let mut resultado = HashSet::new();
        let mut frontier = vec![desde];
        let mut visited = HashSet::new();

        for _ in 0..=hops {
            let mut next_frontier = Vec::new();
            for nodo_id in frontier {
                if visited.contains(&nodo_id) {
                    continue;
                }
                visited.insert(nodo_id);
                resultado.insert(nodo_id);

                if let Some(vecinos) = self.mesh.conexiones.get(&nodo_id) {
                    for &vecino in vecinos {
                        if !visited.contains(&vecino) {
                            next_frontier.push(vecino);
                        }
                    }
                }
            }
            frontier = next_frontier;
        }

        resultado
    }

    // ========================================================================
    // INFILTRATION OPERATIONS
    // ========================================================================

    /// Create infiltration vector targeting a node
    pub fn crear_infiltracion(
        &mut self,
        nodo_id: u64,
        tecnica: TecnicaInfiltracion,
        origen_id: u64,
    ) -> Result<VectorInfiltracion, String> {
        self.contador_operaciones += 1;

        let _destino = self
            .mesh
            .nodos
            .get(&nodo_id)
            .ok_or_else(|| format!("Node {} not found in mesh", nodo_id))?;

        let vector = VectorInfiltracion {
            id: self.contador_operaciones,
            origen_id,
            destino_id: nodo_id,
            tecnica,
            progreso: 0.0,
            estado: EstadoInfiltracion::Preparando,
            timestamp_creacion: timestamp_unix(),
            timestamp_update: timestamp_unix(),
            resultado: None,
        };

        let vector_id = vector.id;
        self.infiltraciones_activas.insert(vector_id, vector);

        Ok(self.infiltraciones_activas.get(&vector_id).unwrap().clone())
    }

    /// Execute infiltration with realistic progress calculation
    pub fn ejecutar_infiltracion(&mut self, vector_id: u64) -> Result<bool, String> {
        let vector = self
            .infiltraciones_activas
            .get_mut(&vector_id)
            .ok_or("Infiltration vector not found")?;

        let destino = self
            .mesh
            .nodos
            .get(&vector.destino_id)
            .ok_or("Target node not found")?;

        // Calculate realistic progression based on effectiveness
        let efectividad = vector.calcular_efectividad(destino);
        let progreso_delta = efectividad * 0.2;

        vector.avanzar_progreso(progreso_delta);

        // Check if infiltration is successful
        if vector.progreso >= 0.8 && rand_chance(efectividad) {
            vector.estado = EstadoInfiltracion::Exitoso;
            vector.progreso = 1.0;
            vector.resultado = Some(ResultadoInfiltracion {
                acceso_obtenido: NivelAcceso::Usuario,
                datos_exfiltrados: Vec::new(),
                marca_temporal: timestamp_unix(),
            });

            // Update node access level
            if let Some(nodo) = self.mesh.nodos.get_mut(&vector.destino_id) {
                nodo.nivel_acceso = NivelAcceso::Usuario;
            }

            return Ok(true);
        }

        if vector.progreso >= 0.3 && rand_chance(0.1) {
            vector.estado = EstadoInfiltracion::Detectado;
        }

        vector.estado = EstadoInfiltracion::EnProgreso;
        Ok(false)
    }

    /// Get active infiltrations by state
    pub fn infiltraciones_por_estado(
        &self,
        estado: &EstadoInfiltracion,
    ) -> Vec<&VectorInfiltracion> {
        self.infiltraciones_activas
            .values()
            .filter(|v| &v.estado == estado)
            .collect()
    }

    // ========================================================================
    // PROPAGATION OPERATIONS
    // ========================================================================

    /// Create propagation vector to target
    pub fn crear_propagon(&mut self, origen: u64, destino: u64) -> Option<VectorPropagacion> {
        self.contador_operaciones += 1;

        let mut vector = VectorPropagacion::nuevo(&self.mesh, origen, destino)?;
        vector.id = self.contador_operaciones;

        self.propagaciones.push(vector.clone());
        Some(vector)
    }

    /// Calculate optimal propagation targets based on mesh analysis
    pub fn calcular_propagon_optima(&self) -> Vec<u64> {
        let mut candidatos: Vec<_> = self
            .mesh
            .nodos
            .values()
            .filter(|n| n.nivel_acceso != NivelAcceso::Desconocido)
            .collect();

        // Sort by reachability score and centrality
        candidatos.sort_by(|a, b| {
            let score_a =
                a.calcular_reachability_score() + (self.mesh.centralidad(a.id) as f32 * 0.1);
            let score_b =
                b.calcular_reachability_score() + (self.mesh.centralidad(b.id) as f32 * 0.1);
            score_b.partial_cmp(&score_a).unwrap()
        });

        candidatos.into_iter().take(5).map(|n| n.id).collect()
    }

    /// Execute propagation through mesh
    pub fn ejecutar_propagon(&mut self, vector_id: u64) -> Result<bool, String> {
        let vector = self
            .propagaciones
            .iter_mut()
            .find(|v| v.id == vector_id)
            .ok_or("Propagation vector not found")?;

        if vector.ruta.is_empty() {
            vector.estado = EstadoPropagacion::Abandonado;
            return Err("Empty route".to_string());
        }

        // Calculate success based on route effectiveness
        if rand_chance(vector.metrica_efectividad) {
            vector.estado = EstadoPropagacion::Exitoso;
            return Ok(true);
        }

        vector.estado = EstadoPropagacion::EnTransito;
        Ok(false)
    }

    // ========================================================================
    // TRAFFIC INTERCEPTION
    // ========================================================================

    /// Record traffic interception metadata
    pub fn registrar_intercepcion(
        &mut self,
        origen: u64,
        destino: u64,
        tipo: TipoTrafico,
        hash_contenido: String,
        metadatos: TraficoMetadatos,
    ) -> IntercepcionTrafico {
        self.contador_operaciones += 1;

        let intercepcion = IntercepcionTrafico {
            id: self.contador_operaciones,
            nodo_origen: origen,
            nodo_destino: destino,
            contenido_hash: hash_contenido,
            tipo_tráfico: tipo,
            metadatos,
            timestamp: timestamp_unix(),
        };

        self.intercepciones.push(intercepcion.clone());
        intercepcion
    }

    /// Get interceptions filtered by traffic type
    pub fn intercepciones_por_tipo(&self, tipo: &TipoTrafico) -> Vec<&IntercepcionTrafico> {
        self.intercepciones
            .iter()
            .filter(|i| &i.tipo_tráfico == tipo)
            .collect()
    }

    /// Get interceptions involving a specific node
    pub fn intercepciones_de_nodo(&self, nodo_id: u64) -> Vec<&IntercepcionTrafico> {
        self.intercepciones
            .iter()
            .filter(|i| i.nodo_origen == nodo_id || i.nodo_destino == nodo_id)
            .collect()
    }

    // ========================================================================
    // STATISTICS AND ANALYSIS
    // ========================================================================

    /// Get comprehensive cyberspace statistics
    pub fn get_stats(&self) -> CyberspaceStats {
        let mesh_stats = self.mesh.estadisticas();

        let mut nodos_por_tipo: HashMap<String, u64> = HashMap::new();
        for nodo in self.mesh.nodos.values() {
            *nodos_por_tipo
                .entry(format!("{:?}", nodo.tipo))
                .or_insert(0) += 1;
        }

        let infiltraciones_activas = self
            .infiltraciones_activas
            .values()
            .filter(|v| {
                v.estado == EstadoInfiltracion::EnProgreso
                    || v.estado == EstadoInfiltracion::Preparando
            })
            .count() as u64;

        let infiltraciones_exitosas = self
            .infiltraciones_activas
            .values()
            .filter(|v| v.estado == EstadoInfiltracion::Exitoso)
            .count() as u64;

        CyberspaceStats {
            total_nodos: mesh_stats.total_nodos as u64,
            nodos_por_tipo,
            conexiones_totales: mesh_stats.total_conexiones as u64,
            densidad_mesh: mesh_stats.densidad,
            infiltraciones_activas,
            infiltraciones_exitosas,
            propagaciones_totales: self.propagaciones.len() as u64,
            intercepciones_totales: self.intercepciones.len() as u64,
            operaciones_totales: self.contador_operaciones,
        }
    }

    /// Analyze mesh for security insights
    pub fn analizar_seguridad_mesh(&self) -> MeshAnalisisSeguridad {
        let mut nodos_vulnerables = Vec::new();
        let nodos_criticos;
        let mut rutas_criticas = Vec::new();

        // Find nodes with vulnerabilities
        for nodo in self.mesh.nodos.values() {
            if nodo.tiene_vulnerabilidades() {
                nodos_vulnerables.push(nodo.id);
            }
        }

        // Find critical nodes (high centrality)
        nodos_criticos = self.mesh.nodos_criticos(3);

        // Find critical paths (between critical nodes)
        for i in 0..nodos_criticos.len() {
            for j in (i + 1)..nodos_criticos.len() {
                if let Some(ruta) = self
                    .mesh
                    .encontrar_ruta(nodos_criticos[i], nodos_criticos[j])
                {
                    if ruta.len() <= 3 {
                        rutas_criticas.push(ruta);
                    }
                }
            }
        }

        MeshAnalisisSeguridad {
            nodos_vulnerables,
            nodos_criticos,
            rutas_criticas,
            score_seguridad: self.calcular_score_seguridad(),
        }
    }

    /// Calculate overall mesh security score
    fn calcular_score_seguridad(&self) -> f32 {
        let total_nodos = self.mesh.nodos.len();
        if total_nodos == 0 {
            return 1.0;
        }

        let nodos_seguros = self
            .mesh
            .nodos
            .values()
            .filter(|n| !n.tiene_vulnerabilidades() && n.nivel_acceso != NivelAcceso::Admin)
            .count();

        nodos_seguros as f32 / total_nodos as f32
    }
}

/// Comprehensive cyberspace statistics
#[derive(Debug, Clone)]
pub struct CyberspaceStats {
    pub total_nodos: u64,
    pub nodos_por_tipo: HashMap<String, u64>,
    pub conexiones_totales: u64,
    pub densidad_mesh: f64,
    pub infiltraciones_activas: u64,
    pub infiltraciones_exitosas: u64,
    pub propagaciones_totales: u64,
    pub intercepciones_totales: u64,
    pub operaciones_totales: u64,
}

/// Security analysis results for mesh
#[derive(Debug, Clone)]
pub struct MeshAnalisisSeguridad {
    pub nodos_vulnerables: Vec<u64>,
    pub nodos_criticos: Vec<u64>,
    pub rutas_criticas: Vec<Vec<u64>>,
    pub score_seguridad: f32,
}

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

/// Get current Unix timestamp
fn timestamp_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Simple random chance using timestamp-based seed
fn rand_chance(probability: f32) -> bool {
    let seed = (timestamp_unix() % 1000) as f32 / 1000.0;
    seed < probability
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mesh_creacion() {
        let mesh = NetworkMesh::new();
        assert_eq!(mesh.nodos.len(), 0);
        assert_eq!(mesh.nivel_profundidad, 0);
    }

    #[test]
    fn test_agregar_nodo() {
        let mut mesh = NetworkMesh::new();
        let nodo = NodoRed {
            id: 1,
            ip: "192.168.1.10".to_string(),
            nombre: "test-node".to_string(),
            servicios: vec![],
            nivel_acceso: NivelAcceso::Usuario,
            tipo: TipoNodo::Servidor,
            latencia_estimada: 5.0,
            fecha_descubrimiento: timestamp_unix(),
            metricas: NodoMetricas::default(),
        };
        mesh.agregar_nodo(nodo);
        assert_eq!(mesh.nodos.len(), 1);
    }

    #[test]
    fn test_conexiones_bidireccionales() {
        let mut mesh = NetworkMesh::new();
        mesh.agregar_nodo(NodoRed {
            id: 1,
            ip: "10.0.0.1".to_string(),
            nombre: "A".to_string(),
            servicios: vec![],
            nivel_acceso: NivelAcceso::Externo,
            tipo: TipoNodo::Router,
            latencia_estimada: 1.0,
            fecha_descubrimiento: timestamp_unix(),
            metricas: NodoMetricas::default(),
        });
        mesh.agregar_nodo(NodoRed {
            id: 2,
            ip: "10.0.0.2".to_string(),
            nombre: "B".to_string(),
            servicios: vec![],
            nivel_acceso: NivelAcceso::Externo,
            tipo: TipoNodo::Servidor,
            latencia_estimada: 5.0,
            fecha_descubrimiento: timestamp_unix(),
            metricas: NodoMetricas::default(),
        });
        mesh.agregar_conexion(1, 2, ConexionPeso::default());

        assert_eq!(mesh.vecinos(1).len(), 1);
        assert_eq!(mesh.vecinos(2).len(), 1);
        assert!(mesh.vecinos(1).contains(&2));
    }

    #[test]
    fn test_encontrar_ruta() {
        let mut mesh = NetworkMesh::new();
        for i in 1..=4 {
            mesh.agregar_nodo(NodoRed {
                id: i,
                ip: format!("10.0.0.{}", i),
                nombre: format!("node-{}", i),
                servicios: vec![],
                nivel_acceso: NivelAcceso::Externo,
                tipo: TipoNodo::Router,
                latencia_estimada: 1.0,
                fecha_descubrimiento: timestamp_unix(),
                metricas: NodoMetricas::default(),
            });
        }
        mesh.agregar_conexion(1, 2, ConexionPeso::default());
        mesh.agregar_conexion(2, 3, ConexionPeso::default());
        mesh.agregar_conexion(3, 4, ConexionPeso::default());

        let ruta = mesh.encontrar_ruta(1, 4);
        assert!(ruta.is_some());
        let ruta = ruta.unwrap();
        assert_eq!(ruta, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_cyberspace_creacion() {
        let cyberspace = Cyberspace::new();
        assert_eq!(cyberspace.mesh.nodos.len(), 0);
        assert_eq!(cyberspace.contador_operaciones, 0);
    }

    #[test]
    fn test_descubrir_nodo() {
        let mut cyberspace = Cyberspace::new();
        let servicios = vec![ServicioRed {
            nombre: "http".to_string(),
            puerto: 80,
            protocolo: ProtocoloRed::HTTP,
            version: "2.4.41".to_string(),
            vulnerabilidad: Some("CVE-2021-41773".to_string()),
            banner: None,
        }];

        let id = cyberspace.descubrir_nodo(
            "192.168.1.100".to_string(),
            "web-server".to_string(),
            servicios,
            15.0,
        );

        assert!(id > 0);
        assert_eq!(cyberspace.mesh.nodos.len(), 1);
    }

    #[test]
    fn test_crear_infiltracion() {
        let mut cyberspace = Cyberspace::new();
        cyberspace.descubrir_nodo("10.0.0.50".to_string(), "target".to_string(), vec![], 10.0);

        let resultado = cyberspace.crear_infiltracion(1, TecnicaInfiltracion::Backdoor, 0);

        assert!(resultado.is_ok());
        let vector = resultado.unwrap();
        assert_eq!(vector.tecnica, TecnicaInfiltracion::Backdoor);
        assert_eq!(vector.estado, EstadoInfiltracion::Preparando);
    }

    #[test]
    fn test_calcular_propagon_optima() {
        let mut cyberspace = Cyberspace::new();

        // Add nodes with different access levels
        let servicios_admin = vec![ServicioRed {
            nombre: "ssh".to_string(),
            puerto: 22,
            protocolo: ProtocoloRed::SSH,
            version: "8.0".to_string(),
            vulnerabilidad: None,
            banner: None,
        }];

        cyberspace.descubrir_nodo(
            "172.16.0.10".to_string(),
            "admin-node".to_string(),
            servicios_admin,
            5.0,
        );

        if let Some(nodo) = cyberspace.mesh.nodos.get_mut(&1) {
            nodo.nivel_acceso = NivelAcceso::Admin;
        }

        let optimos = cyberspace.calcular_propagon_optima();
        assert!(!optimos.is_empty());
    }

    #[test]
    fn test_nivel_acceso_comparison() {
        assert!(NivelAcceso::Usuario.can_escalate_to(&NivelAcceso::Admin));
        assert!(!NivelAcceso::Usuario.can_escalate_to(&NivelAcceso::Root));
        assert!(NivelAcceso::Admin.can_escalate_to(&NivelAcceso::Root));
    }

    #[test]
    fn test_tipo_nodo_inference() {
        let servicios_db = vec![ServicioRed {
            nombre: "mysql".to_string(),
            puerto: 3306,
            protocolo: ProtocoloRed::TCP,
            version: "8.0".to_string(),
            vulnerabilidad: None,
            banner: None,
        }];

        let tipo = TipoNodo::inferir_tipo(&servicios_db, "unknown");
        assert_eq!(tipo, TipoNodo::Database);

        let servicios_web = vec![ServicioRed {
            nombre: "http".to_string(),
            puerto: 443,
            protocolo: ProtocoloRed::HTTPS,
            version: "2.4".to_string(),
            vulnerabilidad: None,
            banner: None,
        }];

        let tipo = TipoNodo::inferir_tipo(&servicios_web, "web-server");
        assert_eq!(tipo, TipoNodo::Servidor);
    }

    #[test]
    fn test_nodos_en_rango() {
        let mut mesh = NetworkMesh::new();
        for i in 1..=6 {
            mesh.agregar_nodo(NodoRed {
                id: i,
                ip: format!("10.0.0.{}", i),
                nombre: format!("n{}", i),
                servicios: vec![],
                nivel_acceso: NivelAcceso::Externo,
                tipo: TipoNodo::Router,
                latencia_estimada: 1.0,
                fecha_descubrimiento: timestamp_unix(),
                metricas: NodoMetricas::default(),
            });
        }
        mesh.agregar_conexion(1, 2, ConexionPeso::default());
        mesh.agregar_conexion(2, 3, ConexionPeso::default());
        mesh.agregar_conexion(3, 4, ConexionPeso::default());
        mesh.agregar_conexion(4, 5, ConexionPeso::default());
        mesh.agregar_conexion(5, 6, ConexionPeso::default());

        let mut cyberspace = Cyberspace::new();
        cyberspace.mesh = mesh;

        let en_rango_2 = cyberspace.nodos_en_rango(1, 2);
        assert!(en_rango_2.contains(&1));
        assert!(en_rango_2.contains(&2));
        assert!(en_rango_2.contains(&3));
    }

    #[test]
    fn test_estadisticas_mesh() {
        let mut mesh = NetworkMesh::new();
        for i in 1..=3 {
            mesh.agregar_nodo(NodoRed {
                id: i,
                ip: format!("192.168.1.{}", i),
                nombre: format!("node-{}", i),
                servicios: vec![],
                nivel_acceso: NivelAcceso::Externo,
                tipo: TipoNodo::Servidor,
                latencia_estimada: 5.0,
                fecha_descubrimiento: timestamp_unix(),
                metricas: NodoMetricas::default(),
            });
        }
        mesh.agregar_conexion(1, 2, ConexionPeso::default());
        mesh.agregar_conexion(2, 3, ConexionPeso::default());

        let stats = mesh.estadisticas();
        assert_eq!(stats.total_nodos, 3);
        assert_eq!(stats.total_conexiones, 2);
        assert!(stats.densidad > 0.0);
    }
}
