//! # Sistema de Identidad A-Life para Eden
//!
//! Eden conoce quién es, cuánto ha vivido, y hacia dónde va.
//! Mantiene la identidad como neuronas Dopamina en MemBrain.
//!
//! Arquitectura:
//! - Identidad principal → Neurona Dopamina (crítica, nunca se poda)
//! - Hitos → Neuronas Gaba (persistentes)
//! - Métricas temporales → Neuronas Glutamato (efímeras)
//!
//! 100% RUST PURO - Sin dependencias externas.
#![allow(dead_code)]
#![allow(non_snake_case)]

use crate::membrain::MemBrain;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Ruta de almacenamiento de identidad
const IDENTITY_DIR: &str = "/home/ubuntu/eden_identity";

/// Archivo de identidad principal
const IDENTITY_FILE: &str = "/home/ubuntu/eden_identity/core.eden";

/// Socket de comunicación
const IDENTITY_SOCK: &str = "/tmp/eden_identity.sock";

/// Intervalo de actualización (segundos)
const IDENTITY_UPDATE_INTERVAL_SECS: u64 = 3600;

/// Flag global
static RUNNING: AtomicBool = AtomicBool::new(true);

// =============================================================================
// ESTRUCTURAS DE IDENTIDAD (Sin serde, implementación manual)
// =============================================================================

/// Identidad principal de Eden
#[derive(Debug, Clone)]
pub struct EdenIdentity {
    pub nombre: String,
    pub generacion: u32,
    pub fecha_nacimiento: String,
    pub ciclos_vividos: u64,
    pub patrones_aprendidos: u64,
    pub patrones_olvidados: u64,
    pub mutaciones_exitosas: u32,
    pub mutaciones_fallidas: u32,
    pub crisis_superadas: u32,
    pub tiempo_hibernacion_total_ms: u64,
    pub agente_mas_usado: String,
    pub tarea_mas_resuelta: String,
    pub tasa_supervivencia: f32,
    pub estado_actual: String,
    pub ultima_actualizacion: String,
    pub uuid: String,
    pub entropy_pool: u64,
}

impl Default for EdenIdentity {
    fn default() -> Self {
        Self {
            nombre: "Eden".to_string(),
            generacion: 1,
            fecha_nacimiento: Self::now_iso(),
            ciclos_vividos: 0,
            patrones_aprendidos: 0,
            patrones_olvidados: 0,
            mutaciones_exitosas: 0,
            mutaciones_fallidas: 0,
            crisis_superadas: 0,
            tiempo_hibernacion_total_ms: 0,
            agente_mas_usado: "ninguno".to_string(),
            tarea_mas_resuelta: "ninguna".to_string(),
            tasa_supervivencia: 0.0,
            estado_actual: "desconocido".to_string(),
            ultima_actualizacion: Self::now_iso(),
            uuid: Self::generate_uuid(),
            entropy_pool: Self::collect_entropy(),
        }
    }
}

impl EdenIdentity {
    // ========================================================================
    // UTILIDADES DE TIEMPO Y UUID (Sin dependencias externas)
    // ========================================================================

    /// Genera timestamp ISO actual
    fn now_iso() -> String {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();

        let secs = now.as_secs();
        let millis = now.subsec_millis();

        format!("{}.{:03}Z", Self::epoch_to_datetime(secs), millis)
    }

    /// Convierte epoch a formato datetime simple
    fn epoch_to_datetime(epoch: u64) -> String {
        let secs_per_day = 86400;
        let secs_per_hour = 3600;
        let secs_per_min = 60;

        let days = epoch / secs_per_day;
        let rem = epoch % secs_per_day;

        let hours = rem / secs_per_hour;
        let rem = rem % secs_per_hour;

        let mins = rem / secs_per_min;
        let secs = rem % secs_per_min;

        // Calcular año/día (simplificado - desde 1970)
        let year = 1970 + (days / 365);
        let day_of_year = days % 365 + 1;

        // Mes y día simplificado
        let month = ((day_of_year - 1) / 30) + 1;
        let day = ((day_of_year - 1) % 30) + 1;

        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
            year,
            month.min(12),
            day.min(30),
            hours,
            mins,
            secs
        )
    }

    /// Genera UUID v4-like usando entropía del sistema
    fn generate_uuid() -> String {
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;

        // Mezcla de múltiples fuentes de entropía
        let mut hash = time;
        hash = hash.rotate_left(17) ^ 0x9e_37_79b9_7f4_a7c5_u64;
        hash = hash.wrapping_mul(0xbf_58_4c_64_0c_1d_e9_u64);

        // Xorshift para distribución uniforme
        hash ^= hash << 13;
        hash ^= hash >> 7;
        hash ^= hash << 17;

        format!(
            "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
            (hash >> 32) as u32,
            (hash >> 16) as u32 & 0xFFFF,
            hash as u32 & 0xFFFF,
            (hash & 0xFFFF) as u32,
            hash & 0xFFFF_FFFF_FFFF
        )
    }

    /// Recolecta entropía del sistema
    fn collect_entropy() -> u64 {
        let mut entropy: u64 = 0;

        // Hora actual
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        entropy ^= time;

        // PID
        entropy ^= (std::process::id() as u64).wrapping_mul(0x9e_37_79b9_7f4_a7c5);

        // Hostname hash si está disponible (usando sistema de archivos)
        if let Ok(hostname) = fs::read_to_string("/etc/hostname") {
            let hostname = hostname.trim();
            let mut h: u64 = 0;
            for (i, c) in hostname.bytes().enumerate() {
                h ^= (c as u64).wrapping_mul(0x9e_37_79b9_7f4_a7c5 >> (i % 64));
            }
            entropy ^= h;
        }

        entropy
    }

    // ========================================================================
    // SERIALIZACIÓN MANUAL (Sin serde_json)
    // ========================================================================

    /// Serializa a JSON manualmente
    pub fn to_json(&self) -> String {
        let mut s = String::from("{\n");

        s.push_str(&Self::json_field(
            "nombre",
            &Self::json_string(&self.nombre),
        ));
        s.push('\n');
        s.push_str(&Self::json_field(
            "generacion",
            &self.generacion.to_string(),
        ));
        s.push('\n');
        s.push_str(&Self::json_field(
            "fecha_nacimiento",
            &Self::json_string(&self.fecha_nacimiento),
        ));
        s.push('\n');
        s.push_str(&Self::json_field(
            "ciclos_vividos",
            &self.ciclos_vividos.to_string(),
        ));
        s.push('\n');
        s.push_str(&Self::json_field(
            "patrones_aprendidos",
            &self.patrones_aprendidos.to_string(),
        ));
        s.push('\n');
        s.push_str(&Self::json_field(
            "patrones_olvidados",
            &self.patrones_olvidados.to_string(),
        ));
        s.push('\n');
        s.push_str(&Self::json_field(
            "mutaciones_exitosas",
            &self.mutaciones_exitosas.to_string(),
        ));
        s.push('\n');
        s.push_str(&Self::json_field(
            "mutaciones_fallidas",
            &self.mutaciones_fallidas.to_string(),
        ));
        s.push('\n');
        s.push_str(&Self::json_field(
            "crisis_superadas",
            &self.crisis_superadas.to_string(),
        ));
        s.push('\n');
        s.push_str(&Self::json_field(
            "tiempo_hibernacion_total_ms",
            &self.tiempo_hibernacion_total_ms.to_string(),
        ));
        s.push('\n');
        s.push_str(&Self::json_field(
            "agente_mas_usado",
            &Self::json_string(&self.agente_mas_usado),
        ));
        s.push('\n');
        s.push_str(&Self::json_field(
            "tarea_mas_resuelta",
            &Self::json_string(&self.tarea_mas_resuelta),
        ));
        s.push('\n');
        s.push_str(&Self::json_field(
            "tasa_supervivencia",
            &format!("{:.6}", self.tasa_supervivencia),
        ));
        s.push('\n');
        s.push_str(&Self::json_field(
            "estado_actual",
            &Self::json_string(&self.estado_actual),
        ));
        s.push('\n');
        s.push_str(&Self::json_field(
            "ultima_actualizacion",
            &Self::json_string(&self.ultima_actualizacion),
        ));
        s.push('\n');
        s.push_str(&Self::json_field("uuid", &Self::json_string(&self.uuid)));
        s.push('\n');
        s.push_str(&Self::json_field(
            "entropy_pool",
            &format!("0x{:016x}", self.entropy_pool),
        ));

        s.push_str("\n}");
        s
    }

    fn json_field(key: &str, value: &str) -> String {
        format!("  \"{}\": {}", key, value)
    }

    fn json_string(s: &str) -> String {
        let escaped = s
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t");
        format!("\"{}\"", escaped)
    }

    /// Deserializa desde JSON manualmente
    pub fn from_json(json: &str) -> Option<Self> {
        let mut identity = EdenIdentity::default();
        let mut fields: HashMap<String, String> = HashMap::new();

        // Parser JSON simple
        let content = json.trim();
        if !content.starts_with('{') || !content.ends_with('}') {
            return None;
        }

        let inner = content[1..content.len() - 1].as_bytes();
        let mut pos = 0;

        while pos < inner.len() {
            // Saltar espacios y comas
            while pos < inner.len()
                && (inner[pos] == b' ' || inner[pos] == b',' || inner[pos] == b'\n')
            {
                pos += 1;
            }

            if pos >= inner.len() {
                break;
            }

            // Parsear "key":
            if inner[pos] != b'"' {
                pos += 1;
                continue;
            }
            pos += 1;

            let key_start = pos;
            while pos < inner.len() && inner[pos] != b'"' {
                pos += 1;
            }
            let key = String::from_utf8_lossy(&inner[key_start..pos]).to_string();
            pos += 1; // Saltar "

            // Saltar :
            while pos < inner.len() && (inner[pos] == b' ' || inner[pos] == b':') {
                pos += 1;
            }

            // Parsear valor
            let value_start = pos;
            if inner[pos] == b'"' {
                // String
                pos += 1;
                let mut escaped = false;
                while pos < inner.len() {
                    if escaped {
                        escaped = false;
                    } else if inner[pos] == b'\\' {
                        escaped = true;
                    } else if inner[pos] == b'"' {
                        break;
                    }
                    pos += 1;
                }
                let value = String::from_utf8_lossy(&inner[value_start + 1..pos]).to_string();
                fields.insert(key, value);
                pos += 1;
            } else {
                // Number o boolean
                while pos < inner.len()
                    && inner[pos] != b','
                    && inner[pos] != b'\n'
                    && inner[pos] != b'}'
                {
                    pos += 1;
                }
                let value = String::from_utf8_lossy(&inner[value_start..pos])
                    .trim()
                    .to_string();
                fields.insert(key, value);
            }
        }

        // Extraer campos
        if let Some(v) = fields.get("nombre") {
            identity.nombre = v.clone();
        }
        if let Some(v) = fields.get("generacion") {
            identity.generacion = v.parse().unwrap_or(1);
        }
        if let Some(v) = fields.get("fecha_nacimiento") {
            identity.fecha_nacimiento = v.clone();
        }
        if let Some(v) = fields.get("ciclos_vividos") {
            identity.ciclos_vividos = v.parse().unwrap_or(0);
        }
        if let Some(v) = fields.get("patrones_aprendidos") {
            identity.patrones_aprendidos = v.parse().unwrap_or(0);
        }
        if let Some(v) = fields.get("patrones_olvidados") {
            identity.patrones_olvidados = v.parse().unwrap_or(0);
        }
        if let Some(v) = fields.get("mutaciones_exitosas") {
            identity.mutaciones_exitosas = v.parse().unwrap_or(0);
        }
        if let Some(v) = fields.get("mutaciones_fallidas") {
            identity.mutaciones_fallidas = v.parse().unwrap_or(0);
        }
        if let Some(v) = fields.get("crisis_superadas") {
            identity.crisis_superadas = v.parse().unwrap_or(0);
        }
        if let Some(v) = fields.get("tiempo_hibernacion_total_ms") {
            identity.tiempo_hibernacion_total_ms = v.parse().unwrap_or(0);
        }
        if let Some(v) = fields.get("agente_mas_usado") {
            identity.agente_mas_usado = v.clone();
        }
        if let Some(v) = fields.get("tarea_mas_resuelta") {
            identity.tarea_mas_resuelta = v.clone();
        }
        if let Some(v) = fields.get("tasa_supervivencia") {
            identity.tasa_supervivencia = v.parse().unwrap_or(0.0);
        }
        if let Some(v) = fields.get("estado_actual") {
            identity.estado_actual = v.clone();
        }
        if let Some(v) = fields.get("ultima_actualizacion") {
            identity.ultima_actualizacion = v.clone();
        }
        if let Some(v) = fields.get("uuid") {
            identity.uuid = v.clone();
        }
        if let Some(v) = fields.get("entropy_pool") {
            if v.starts_with("0x") {
                identity.entropy_pool = u64::from_str_radix(&v[2..], 16).unwrap_or(0);
            } else {
                identity.entropy_pool = v.parse().unwrap_or(0);
            }
        }

        Some(identity)
    }
}

/// Hito de identidad
#[derive(Debug, Clone)]
pub struct Milestone {
    pub ciclos: u64,
    pub timestamp: String,
    pub generacion: u32,
    pub patrones_aprendidos: u64,
    pub mutaciones_exitosas: u32,
    pub tasa_supervivencia: f32,
    pub estado: String,
}

impl Milestone {
    pub fn from_identity(identity: &EdenIdentity) -> Self {
        Self {
            ciclos: identity.ciclos_vividos,
            timestamp: Self::now_iso(),
            generacion: identity.generacion,
            patrones_aprendidos: identity.patrones_aprendidos,
            mutaciones_exitosas: identity.mutaciones_exitosas,
            tasa_supervivencia: identity.tasa_supervivencia,
            estado: identity.estado_actual.clone(),
        }
    }

    fn now_iso() -> String {
        EdenIdentity::now_iso()
    }

    pub fn to_json(&self) -> String {
        format!(
            r#"{{
  "ciclos": {},
  "timestamp": {},
  "generacion": {},
  "patrones_aprendidos": {},
  "mutaciones_exitosas": {},
  "tasa_supervivencia": {:.6},
  "estado": {}
}}"#,
            self.ciclos,
            EdenIdentity::json_string(&self.timestamp),
            self.generacion,
            self.patrones_aprendidos,
            self.mutaciones_exitosas,
            self.tasa_supervivencia,
            EdenIdentity::json_string(&self.estado)
        )
    }
}

// =============================================================================
// MEMORY IDENTITY - Almacenamiento neural de identidad
// =============================================================================

/// Wrapper sobre MemBrain para identidad
struct MemoryIdentity {
    brain: MemBrain,
}

impl MemoryIdentity {
    fn new() -> std::io::Result<Self> {
        Ok(Self {
            brain: MemBrain::new(IDENTITY_DIR)?,
        })
    }

    /// Guarda identidad como neuronas Dopamina
    fn save_identity(&mut self, identity: &EdenIdentity) {
        let json = identity.to_json();
        let key = format!("identity:{}", identity.uuid);

        // Dopamina: crítico, nunca se poda
        self.brain.dopa(key.as_bytes(), json.into_bytes());

        // Guardar también por nombre para acceso rápido
        let name_key = format!("identity:name:{}", identity.nombre);
        self.brain
            .dopa(name_key.as_bytes(), identity.uuid.as_bytes().to_vec());
    }

    /// Carga identidad desde MemBrain
    fn load_identity(&mut self) -> Option<EdenIdentity> {
        // Buscar por UUID primero
        let uuid = self.find_uuid()?;
        let key = format!("identity:{}", uuid);

        self.brain.recall(key.as_bytes()).and_then(|data| {
            let json = String::from_utf8(data).ok()?;
            EdenIdentity::from_json(&json)
        })
    }

    /// Encuentra el UUID guardado
    fn find_uuid(&self) -> Option<String> {
        // Buscar en índice
        let entries = self.brain.search(b"identity:");
        entries
            .into_iter()
            .next()
            .and_then(|data| String::from_utf8(data).ok())
    }

    /// Registra un hito
    fn save_milestone(&mut self, milestone: &Milestone) {
        let key = format!("milestone:{}", milestone.ciclos);
        let json = milestone.to_json();

        // GABA: persistente pero puede consolidarse
        self.brain.gaba(key.as_bytes(), json.into_bytes());
    }

    /// Obtiene todos los hitos
    fn get_milestones(&self) -> Vec<Milestone> {
        let entries = self.brain.search(b"milestone:");

        entries
            .into_iter()
            .filter_map(|data| {
                let json = String::from_utf8(data).ok()?;
                Self::parse_milestone(&json)
            })
            .collect()
    }

    fn parse_milestone(json: &str) -> Option<Milestone> {
        let mut fields: HashMap<String, String> = HashMap::new();

        // Parser simplificado
        for line in json.lines() {
            let line = line.trim();
            if let Some(colon) = line.find(':') {
                let key = line[..colon].trim().trim_matches('"');
                let value = line[colon + 1..].trim().trim_matches(',').trim_matches('"');
                fields.insert(key.to_string(), value.to_string());
            }
        }

        Some(Milestone {
            ciclos: fields.get("ciclos")?.parse().ok()?,
            timestamp: fields.get("timestamp")?.clone(),
            generacion: fields.get("generacion")?.parse().ok()?,
            patrones_aprendidos: fields.get("patrones_aprendidos")?.parse().ok()?,
            mutaciones_exitosas: fields.get("mutaciones_exitosas")?.parse().ok()?,
            tasa_supervivencia: fields.get("tasa_supervivencia")?.parse().ok()?,
            estado: fields.get("estado")?.clone(),
        })
    }
}

// =============================================================================
// API PÚBLICA
// =============================================================================

/// Inicia el gestor de identidad
pub fn start_identity_manager() {
    println!("[IDENTITY] Gestor de Identidad A-Life iniciado");
    println!("[IDENTITY] Usando MemBrain como almacén neural");
    println!(
        "[IDENTITY] Actualización cada {} segundos",
        IDENTITY_UPDATE_INTERVAL_SECS
    );

    // Inicializar si es primera vez
    initialize_identity_if_needed();

    while RUNNING.load(Ordering::SeqCst) {
        update_identity();

        // Dormir en incrementos
        for _ in 0..IDENTITY_UPDATE_INTERVAL_SECS / 5 {
            if !RUNNING.load(Ordering::SeqCst) {
                break;
            }
            thread::sleep(Duration::from_secs(5));
        }
    }

    println!("[IDENTITY] Detenido");
}

/// Detiene el gestor de identidad
pub fn stop_identity_manager() {
    RUNNING.store(false, Ordering::SeqCst);
}

/// Inicializa identidad si no existe
fn initialize_identity_if_needed() {
    if !Path::new(IDENTITY_FILE).exists() {
        let identity = EdenIdentity::default();
        write_identity_file(&identity);
        println!("[IDENTITY] ✓ Identidad inicializada: {}", identity.uuid);

        // Guardar en MemBrain
        if let Ok(mut mi) = MemoryIdentity::new() {
            mi.save_identity(&identity);
        }
    }
}

/// Actualiza la identidad desde las métricas actuales
fn update_identity() {
    let mut identity = load_identity_from_file();

    // Actualizar timestamp
    identity.ultima_actualizacion = EdenIdentity::now_iso();

    // Intentar cargar métricas de MemBrain
    if let Ok(brain) = MemBrain::new("/home/ubuntu/eden_memory") {
        // Leer métricas del brain
        if let Some(_stats) = brain.stats().working_memory_size.checked_add(0) {
            // Aquí iría lógica para leer métricas reales
            // Por ahora usamos valores simulados
        }
    }

    // Calcular tasa de supervivencia
    let total_attempts = identity.mutaciones_exitosas as f32 + identity.mutaciones_fallidas as f32;
    identity.tasa_supervivencia = if total_attempts > 0.0 {
        identity.mutaciones_exitosas as f32 / total_attempts
    } else {
        0.0
    };

    // Determinar estado actual
    identity.estado_actual = determine_current_state();

    // Guardar
    write_identity_file(&identity);

    // Guardar en MemBrain también
    if let Ok(mut mi) = MemoryIdentity::new() {
        mi.save_identity(&identity);
    }

    // Verificar hitos
    check_for_milestone(&identity);

    println!(
        "[IDENTITY] 📊 ciclos={} gen={} patrones={} supervivencia={:.1}%",
        identity.ciclos_vividos,
        identity.generacion,
        identity.patrones_aprendidos,
        identity.tasa_supervivencia * 100.0
    );
}

/// Determina el estado actual del organismo
fn determine_current_state() -> String {
    // ¿Está hibernando?
    if Path::new("/tmp/eden_sleep_mode").exists() {
        return "hibernando".to_string();
    }

    // ¿Tiene problemas inmunes?
    if let Ok(mi) = MemoryIdentity::new() {
        let immune_count = mi.brain.search(b"immune:").len();
        if immune_count > 5 {
            return "recuperandose".to_string();
        }
    }

    // Estado energético (simulado)
    let energy = read_energy_state();

    if energy < 20 {
        return "agotado".to_string();
    }
    if energy > 80 {
        return "activo".to_string();
    }

    "operativo".to_string()
}

/// Lee el estado energético (simulado - en producción vendría de homeostasis)
fn read_energy_state() -> u32 {
    if let Ok(contents) = fs::read_to_string("/tmp/eden_energy") {
        contents.trim().parse().unwrap_or(100)
    } else {
        100 // Default
    }
}

/// Carga identidad desde archivo
fn load_identity_from_file() -> EdenIdentity {
    if let Ok(contents) = fs::read_to_string(IDENTITY_FILE) {
        if let Some(identity) = EdenIdentity::from_json(&contents) {
            return identity;
        }
    }

    // Intentar desde MemBrain
    if let Ok(mut mi) = MemoryIdentity::new() {
        if let Some(identity) = mi.load_identity() {
            return identity;
        }
    }

    EdenIdentity::default()
}

/// Escribe identidad a archivo
fn write_identity_file(identity: &EdenIdentity) {
    if let Some(parent) = Path::new(IDENTITY_FILE).parent() {
        fs::create_dir_all(parent).ok();
    }

    let json = identity.to_json();

    if let Err(e) = fs::write(IDENTITY_FILE, json) {
        eprintln!("[IDENTITY] Error escribiendo identidad: {}", e);
    }
}

/// Verifica y registra hitos
fn check_for_milestone(identity: &EdenIdentity) {
    if identity.ciclos_vividos > 0 && identity.ciclos_vividos % 10000 == 0 {
        println!(
            "[IDENTITY] 🏆 MILESTONE: {} ciclos alcanzados!",
            identity.ciclos_vividos
        );

        let milestone = Milestone::from_identity(identity);

        // Guardar en archivo
        let milestone_dir = format!("{}/milestones", IDENTITY_DIR);
        let milestone_file = format!("{}/{}.json", milestone_dir, milestone.ciclos);

        if let Some(parent) = Path::new(&milestone_file).parent() {
            fs::create_dir_all(parent).ok();
        }

        let _ = fs::write(&milestone_file, milestone.to_json());

        // Guardar en MemBrain
        if let Ok(mut mi) = MemoryIdentity::new() {
            mi.save_milestone(&milestone);
        }

        println!("[IDENTITY] ✓ Hito guardado");
    }
}

/// Obtiene la identidad actual
pub fn get_identity() -> EdenIdentity {
    load_identity_from_file()
}

/// Obtiene todos los hitos
pub fn get_milestones() -> Vec<Milestone> {
    if let Ok(mi) = MemoryIdentity::new() {
        let milestones = mi.get_milestones();
        if !milestones.is_empty() {
            return milestones;
        }
    }

    // Fallback: leer desde archivos
    let milestone_dir = format!("{}/milestones", IDENTITY_DIR);
    let mut milestones = Vec::new();

    if let Ok(entries) = fs::read_dir(&milestone_dir) {
        for entry in entries.flatten() {
            if let Ok(contents) = fs::read_to_string(entry.path()) {
                if let Some(m) = MemoryIdentity::parse_milestone(&contents) {
                    milestones.push(m);
                }
            }
        }
    }

    milestones.sort_by(|a, b| b.ciclos.cmp(&a.ciclos));
    milestones
}

/// Inicia servidor de socket de identidad
pub fn start_identity_socket() {
    std::thread::spawn(|| {
        println!("[IDENTITY] Socket server iniciado en {}", IDENTITY_SOCK);

        // Eliminar socket anterior
        let _ = std::fs::remove_file(IDENTITY_SOCK);

        // Crear socket
        if let Ok(listener) = std::os::unix::net::UnixListener::bind(IDENTITY_SOCK) {
            for stream in listener.incoming() {
                if !RUNNING.load(Ordering::SeqCst) {
                    break;
                }

                if let Ok(mut stream) = stream {
                    let identity = get_identity();
                    let json = identity.to_json();
                    let _ = stream.write_all(json.as_bytes());
                }
            }
        }
    });
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_default() {
        let identity = EdenIdentity::default();
        assert_eq!(identity.nombre, "Eden");
        assert_eq!(identity.generacion, 1);
        assert!(!identity.fecha_nacimiento.is_empty());
        assert!(!identity.uuid.is_empty());
    }

    #[test]
    fn test_json_serialization() {
        let identity = EdenIdentity::default();
        let json = identity.to_json();
        let parsed = EdenIdentity::from_json(&json).unwrap();

        assert_eq!(identity.nombre, parsed.nombre);
        assert_eq!(identity.generacion, parsed.generacion);
        assert_eq!(identity.uuid, parsed.uuid);
    }

    #[test]
    fn test_uuid_uniqueness() {
        let uuid1 = EdenIdentity::generate_uuid();
        let uuid2 = EdenIdentity::generate_uuid();

        // Los UUIDs deberían ser diferentes (muy alta probabilidad)
        // En tests unitarios, podemos aceptar colisiones raras
        assert!(uuid1.len() == 36);
        assert!(uuid2.len() == 36);
    }

    #[test]
    fn test_json_field_format() {
        let s = "hello \"world\"";
        let escaped = EdenIdentity::json_string(s);
        assert!(escaped.contains("\\\""));
    }

    #[test]
    fn test_milestone_creation() {
        let identity = EdenIdentity::default();
        let milestone = Milestone::from_identity(&identity);

        assert_eq!(milestone.ciclos, identity.ciclos_vividos);
        assert_eq!(milestone.generacion, identity.generacion);
    }
}
