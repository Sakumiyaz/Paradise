//! Pure Rust mnemosyne - knowledge graph with tiered storage (Hot/Warm/Cold)
//! No external dependencies beyond serde/serde_json

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================
// ERROR TYPES
// ============================================================

#[derive(Debug)]
pub enum LayerError {
    Io(std::io::Error),
    Serial(serde_json::Error),
    NotFound(String),
    ColdReadOnly,
}

impl From<std::io::Error> for LayerError {
    fn from(e: std::io::Error) -> Self {
        LayerError::Io(e)
    }
}

impl From<serde_json::Error> for LayerError {
    fn from(e: serde_json::Error) -> Self {
        LayerError::Serial(e)
    }
}

impl std::fmt::Display for LayerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LayerError::Io(e) => write!(f, "IO error: {}", e),
            LayerError::Serial(e) => write!(f, "Serialization error: {}", e),
            LayerError::NotFound(id) => write!(f, "Node not found: {}", id),
            LayerError::ColdReadOnly => write!(f, "Cold storage is read-only"),
        }
    }
}

impl std::error::Error for LayerError {}

// ============================================================
// LAYER ENUM
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Layer {
    Hot = 1,
    Warm = 2,
    Cold = 3,
}

impl Layer {
    pub fn name(&self) -> &'static str {
        match self {
            Layer::Hot => "HOT",
            Layer::Warm => "WARM",
            Layer::Cold => "COLD",
        }
    }
}

// ============================================================
// CORE DATA STRUCTURES
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionRecord {
    pub generacion: i64,
    pub mutacion_id: String,
    pub patron_origen: String,
    pub resultado: String,
    pub peso_final: f64,
    pub sobrevivio: bool,
    pub timestamp: i64,
    pub layer: Layer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    pub label: String,
    pub properties: serde_json::Value,
    pub layer: Layer,
    pub last_access: i64,
    pub created_at: i64,
    pub embeddings: Option<Vec<f32>>,
}

impl Node {
    pub fn is_stale(&self, ttl_ms: i64) -> bool {
        let now = Self::now_ms();
        now - self.last_access > ttl_ms
    }

    pub fn now_ms() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64
    }
}

// ============================================================
// COLD STORAGE (file-based)
// ============================================================

pub struct ColdStorage {
    nodes: Mutex<HashMap<String, Node>>,
}

impl ColdStorage {
    fn new() -> Self {
        Self {
            nodes: Mutex::new(HashMap::new()),
        }
    }

    fn insert(&self, node: Node) {
        let mut nodes = self.nodes.lock().unwrap();
        nodes.insert(node.id.clone(), node);
    }

    fn get(&self, id: &str) -> Option<Node> {
        let nodes = self.nodes.lock().unwrap();
        nodes.get(id).cloned()
    }

    fn len(&self) -> usize {
        let nodes = self.nodes.lock().unwrap();
        nodes.len()
    }

    fn to_vec(&self) -> Vec<Node> {
        let nodes = self.nodes.lock().unwrap();
        nodes.values().cloned().collect()
    }
}

// ============================================================
// VITAL EVENT LOG (for purgatory events)
// ============================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VitalEvent {
    pub tipo: String,
    pub accion: String,
    pub ram_libre: u64,
    pub cpu_pct: f64,
    pub timestamp: String,
}

pub struct VitalLog {
    events: Mutex<Vec<VitalEvent>>,
    path: String,
}

impl VitalLog {
    fn new() -> Self {
        let path = Self::vital_log_path();
        Self {
            events: Mutex::new(Vec::new()),
            path,
        }
    }

    fn vital_log_path() -> String {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/ubuntu".to_string());
        format!("{}/.eden/mnemosyne_vital.log", home)
    }

    fn append(&self, event: VitalEvent) {
        let mut events = self.events.lock().unwrap();
        events.push(event.clone());
        // Keep last 1000 events
        let len = events.len();
        if len > 1000 {
            events.drain(0..len - 1000);
        }
        // Persist to file
        drop(events);
        if let Ok(mut file) = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
        {
            use std::io::Write;
            if let Ok(line) = serde_json::to_string(&event) {
                let _ = writeln!(file, "{}", line);
            }
        }
    }

    fn get_recent(&self, count: usize) -> Vec<VitalEvent> {
        let events = self.events.lock().unwrap();
        events.iter().rev().take(count).cloned().collect()
    }
}

// ============================================================
// LAYER MANAGER (pure Rust, no async)
// ============================================================

pub struct LayerManager {
    hot: Arc<Mutex<HashMap<String, Node>>>,
    warm: Arc<Mutex<HashMap<String, Node>>>,
    cold: Arc<ColdStorage>,
    evolution_hot: Arc<Mutex<HashMap<String, EvolutionRecord>>>,
    vital_log: Arc<VitalLog>,
}

impl LayerManager {
    pub fn new() -> Result<Self, LayerError> {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/ubuntu".to_string());
        let cold_path = format!("{}/.eden/eden_cold.bin", home);

        // Ensure .eden directory exists
        let eden_dir = format!("{}/.eden", home);
        fs::create_dir_all(&eden_dir)?;

        let cold = Arc::new(ColdStorage::new());

        // Load cold from file if exists
        if Path::new(&cold_path).exists() {
            Self::load_cold(&cold, &cold_path)?;
        }

        Ok(Self {
            hot: Arc::new(Mutex::new(HashMap::new())),
            warm: Arc::new(Mutex::new(HashMap::new())),
            cold,
            evolution_hot: Arc::new(Mutex::new(HashMap::new())),
            vital_log: Arc::new(VitalLog::new()),
        })
    }

    fn load_cold(cold: &Arc<ColdStorage>, path: &str) -> Result<(), LayerError> {
        let mut file = File::open(path)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;
        let nodes: Vec<Node> = serde_json::from_slice(&data)?;
        for node in nodes {
            cold.insert(node);
        }
        Ok(())
    }

    fn persist_hot_to_warm(&self) -> Result<(), LayerError> {
        let hot = self.hot.lock().unwrap();
        let mut warm = self.warm.lock().unwrap();
        for (id, node) in hot.iter() {
            if node.is_stale(24 * 60 * 60 * 1000) {
                // Older than 24h -> migrate to warm
                let mut warm_node = node.clone();
                warm_node.layer = Layer::Warm;
                warm.insert(id.clone(), warm_node);
            }
        }
        Ok(())
    }

    fn persist_warm_to_cold(&self) -> Result<(), LayerError> {
        let warm = self.warm.lock().unwrap();
        for (_id, node) in warm.iter() {
            if node.is_stale(7 * 24 * 60 * 60 * 1000) {
                // Older than 7 days -> migrate to cold
                let mut cold_node = node.clone();
                cold_node.layer = Layer::Cold;
                self.cold.insert(cold_node);
            }
        }
        Ok(())
    }

    fn persist_all(&self) -> Result<(), LayerError> {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/ubuntu".to_string());
        let cold_path = format!("{}/.eden/eden_cold.bin", home);

        // Persist cold
        let cold_nodes = self.cold.to_vec();
        let data = serde_json::to_vec(&cold_nodes)?;
        fs::write(&cold_path, data)?;

        // Persist warm
        let warm = self.warm.lock().unwrap();
        let warm_path = format!("{}/.eden/eden_warm.json", home);
        let warm_data = serde_json::to_vec(&*warm)?;
        fs::write(&warm_path, warm_data)?;

        Ok(())
    }

    pub fn create_node(
        &self,
        label: String,
        properties: HashMap<String, serde_json::Value>,
        _ttl: Option<i64>,
        embeddings: Option<Vec<f32>>,
    ) -> Result<Node, LayerError> {
        let id = generate_id();
        let now = Node::now_ms();
        let properties_value = serde_json::to_value(properties)?;

        let node = Node {
            id: id.clone(),
            label,
            properties: properties_value,
            layer: Layer::Hot,
            last_access: now,
            created_at: now,
            embeddings,
        };

        let mut hot = self.hot.lock().unwrap();
        hot.insert(id, node.clone());
        Ok(node)
    }

    pub fn get_node(&self, id: &str) -> Result<Node, LayerError> {
        // Check hot first
        {
            let hot = self.hot.lock().unwrap();
            if let Some(mut node) = hot.get(id).cloned() {
                node.last_access = Node::now_ms();
                return Ok(node);
            }
        }

        // Check warm
        {
            let warm = self.warm.lock().unwrap();
            if let Some(mut node) = warm.get(id).cloned() {
                node.last_access = Node::now_ms();
                return Ok(node);
            }
        }

        // Check cold
        if let Some(node) = self.cold.get(id) {
            return Ok(node);
        }

        Err(LayerError::NotFound(id.to_string()))
    }

    pub fn delete_node(&self, id: &str) -> Result<(), LayerError> {
        // Remove from hot
        {
            let mut hot = self.hot.lock().unwrap();
            if hot.remove(id).is_some() {
                return Ok(());
            }
        }

        // Remove from warm
        {
            let mut warm = self.warm.lock().unwrap();
            if warm.remove(id).is_some() {
                return Ok(());
            }
        }

        // Cold storage is archival and read-only from this API.
        if self.cold.get(id).is_some() {
            return Err(LayerError::ColdReadOnly);
        }

        Ok(())
    }

    pub fn create_edge(
        &self,
        _source: &str,
        _target: &str,
        _relation: &str,
    ) -> Result<(), LayerError> {
        Ok(())
    }

    pub fn get_stale_nodes(&self, days: i64) -> Result<Vec<Node>, LayerError> {
        let cutoff = Node::now_ms() - (days * 24 * 60 * 60 * 1000);
        let mut result = Vec::new();

        let warm = self.warm.lock().unwrap();
        for node in warm.values() {
            if node.last_access < cutoff {
                result.push(node.clone());
            }
        }

        Ok(result)
    }

    pub fn store_evolution_record(&self, record: EvolutionRecord) -> Result<(), LayerError> {
        let mut ev_record = record.clone();
        ev_record.layer = Layer::Hot;
        let mut evolution = self.evolution_hot.lock().unwrap();
        evolution.insert(record.mutacion_id.clone(), ev_record);
        Ok(())
    }

    pub fn get_evolution_record(&self, mutacion_id: &str) -> Option<EvolutionRecord> {
        let evolution = self.evolution_hot.lock().unwrap();
        evolution.get(mutacion_id).cloned()
    }

    pub fn get_all_evolution_records(&self) -> Vec<EvolutionRecord> {
        let evolution = self.evolution_hot.lock().unwrap();
        evolution.values().cloned().collect()
    }

    pub fn log_vital_event(&self, event: VitalEvent) {
        self.vital_log.append(event);
    }

    pub fn get_recent_vital_events(&self, count: usize) -> Vec<VitalEvent> {
        self.vital_log.get_recent(count)
    }

    pub fn graceful_shutdown(&self) -> Result<(), LayerError> {
        self.persist_hot_to_warm()?;
        self.persist_warm_to_cold()?;
        self.persist_all()?;
        Ok(())
    }

    pub fn get_counts(&self) -> (usize, usize, usize) {
        let hot = self.hot.lock().unwrap();
        let warm = self.warm.lock().unwrap();
        let cold_len = self.cold.len();
        (hot.len(), warm.len(), cold_len)
    }
}

// ============================================================
// ID GENERATION (pure Rust, no uuid crate)
// ============================================================

fn generate_id() -> String {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;
    let mut buf = [0u8; 16];
    // Fill with random from /dev/urandom if available
    if let Ok(mut f) = std::fs::File::open("/dev/urandom") {
        use std::io::Read;
        let _ = f.read_exact(&mut buf);
    } else {
        // Fallback: timestamp-based
        let ts_bytes = now.to_le_bytes();
        buf[..8].copy_from_slice(&ts_bytes);
        // Mix in more entropy
        let extra = now.wrapping_mul(0x5de66u64).to_le_bytes();
        buf[8..].copy_from_slice(&extra);
    }
    hex_encode(&buf)
}

fn hex_encode(bytes: &[u8]) -> String {
    const HEX_CHARS: &[u8; 16] = b"0123456789abcdef";
    let mut hex = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        hex.push(HEX_CHARS[(b >> 4) as usize] as char);
        hex.push(HEX_CHARS[(b & 0xf) as usize] as char);
    }
    hex
}

// ============================================================
// GRAPH (knowledge graph facade)
// ============================================================

pub struct KnowledgeGraph {
    layer_manager: Arc<LayerManager>,
    vectors: Arc<Mutex<HashMap<String, Vec<f32>>>>,
}

impl KnowledgeGraph {
    pub fn new() -> Result<Self, LayerError> {
        let layer_manager = Arc::new(LayerManager::new()?);
        Ok(Self {
            layer_manager,
            vectors: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub fn create_node(
        &self,
        label: String,
        properties: HashMap<String, serde_json::Value>,
        ttl: Option<i64>,
        embeddings: Option<Vec<f32>>,
    ) -> Result<Node, LayerError> {
        let node = self
            .layer_manager
            .create_node(label, properties, ttl, embeddings.clone())?;
        if let Some(emb) = embeddings {
            let mut vectors = self.vectors.lock().unwrap();
            vectors.insert(node.id.clone(), emb);
        }
        Ok(node)
    }

    pub fn delete_node(&self, id: &str) -> Result<(), LayerError> {
        self.layer_manager.delete_node(id)?;
        let mut vectors = self.vectors.lock().unwrap();
        vectors.remove(id);
        Ok(())
    }

    pub fn create_edge(
        &self,
        source: &str,
        target: &str,
        relation: &str,
    ) -> Result<(), LayerError> {
        self.layer_manager.create_edge(source, target, relation)
    }

    pub fn get_node(&self, id: &str) -> Result<Node, LayerError> {
        self.layer_manager.get_node(id)
    }

    pub fn get_stale_nodes(&self, days: i64) -> Result<Vec<Node>, LayerError> {
        self.layer_manager.get_stale_nodes(days)
    }

    pub fn search_similar(
        &self,
        embedding: &[f32],
        top_k: usize,
    ) -> Result<Vec<(String, f32)>, LayerError> {
        let vectors = self.vectors.lock().unwrap();
        let mut scores = Vec::new();

        for (id, vec) in vectors.iter() {
            if vec.len() != embedding.len() {
                continue;
            }
            let similarity = cosine_similarity(embedding, vec);
            scores.push((id.clone(), similarity));
        }

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scores.truncate(top_k);
        Ok(scores)
    }

    pub fn graceful_shutdown(&self) -> Result<(), LayerError> {
        self.layer_manager.graceful_shutdown()
    }

    pub fn store_evolution_record(&self, record: EvolutionRecord) -> Result<(), LayerError> {
        self.layer_manager.store_evolution_record(record)
    }

    pub fn get_evolution_record(&self, mutacion_id: &str) -> Option<EvolutionRecord> {
        self.layer_manager.get_evolution_record(mutacion_id)
    }

    pub fn get_all_evolution_records(&self) -> Vec<EvolutionRecord> {
        self.layer_manager.get_all_evolution_records()
    }

    pub fn log_vital_event(&self, event: VitalEvent) {
        self.layer_manager.log_vital_event(event);
    }

    pub fn get_recent_vital_events(&self, count: usize) -> Vec<VitalEvent> {
        self.layer_manager.get_recent_vital_events(count)
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_log_path(name: &str) -> String {
        let path = std::env::temp_dir().join(format!(
            "mnemosyne-layers-{}-{}.log",
            std::process::id(),
            name
        ));
        let _ = std::fs::remove_file(&path);
        path.to_string_lossy().to_string()
    }

    fn test_manager(name: &str) -> LayerManager {
        LayerManager {
            hot: Arc::new(Mutex::new(HashMap::new())),
            warm: Arc::new(Mutex::new(HashMap::new())),
            cold: Arc::new(ColdStorage::new()),
            evolution_hot: Arc::new(Mutex::new(HashMap::new())),
            vital_log: Arc::new(VitalLog {
                events: Mutex::new(Vec::new()),
                path: test_log_path(name),
            }),
        }
    }

    fn test_node(id: &str, layer: Layer) -> Node {
        Node {
            id: id.to_string(),
            label: "test".to_string(),
            properties: serde_json::json!({ "kind": "unit" }),
            layer,
            last_access: Node::now_ms(),
            created_at: Node::now_ms(),
            embeddings: None,
        }
    }

    fn test_vital_event(action: &str) -> VitalEvent {
        VitalEvent {
            tipo: "health".to_string(),
            accion: action.to_string(),
            ram_libre: 1024,
            cpu_pct: 12.5,
            timestamp: "test-time".to_string(),
        }
    }

    #[test]
    fn cold_storage_delete_reports_read_only() {
        let manager = test_manager("cold-read-only");
        manager.cold.insert(test_node("cold-node", Layer::Cold));

        let err = manager.delete_node("cold-node").unwrap_err();
        assert!(matches!(err, LayerError::ColdReadOnly));
        assert_eq!(err.to_string(), "Cold storage is read-only");
    }

    #[test]
    fn manager_counts_and_recent_vital_events_are_observable() {
        let manager = test_manager("manager-observability");
        manager
            .create_node("hot-node".to_string(), HashMap::new(), None, None)
            .unwrap();
        manager.log_vital_event(test_vital_event("compact"));

        assert_eq!(manager.get_counts(), (1, 0, 0));
        let recent = manager.get_recent_vital_events(1);
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].accion, "compact");
    }

    #[test]
    fn knowledge_graph_exposes_recent_vital_events() {
        let kg = KnowledgeGraph {
            layer_manager: Arc::new(test_manager("kg-observability")),
            vectors: Arc::new(Mutex::new(HashMap::new())),
        };

        kg.log_vital_event(test_vital_event("rebalance"));
        let recent = kg.get_recent_vital_events(1);

        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].accion, "rebalance");
    }
}
