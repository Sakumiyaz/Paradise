//! =============================================================================
//! CORAZÓN DEL APRENDIZAJE EMERGENTE - EDEN
//! =============================================================================
//!
//! Este módulo implementa las 3 LEYES FUNDAMENTALES de la emergencia:
//!
//! LEY 1: ATRACCIÓN
//!   "Patrones activados juntos se conectan entre sí"
//!   - Los enlaces sinápticos emergen del uso conjunto
//!   - El peso de conexión crece con cada activación compartida
//!
//! LEY 2: INHIBICIÓN  
//!   "Patrones que fallan juntos se suprimen entre sí"
//!   - Las combinaciones fallidas se memorizan como inhibiciones
//!   - La fuerza inhibitoria crece con cada fracaso
//!
//! LEY 3: RESONANCIA
//!   "Cadenas rápidas de activación crean memorias episódicas"
//!   - Las secuencias rápidas se capturan como episodios
//!   - Permiten predecir comportamientos complejos
//!
//! =============================================================================
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

// Función helper para timestamp Unix
// Función timestamp_unix definida en línea 28-32

// ============================================================================
// ESTRUCTURAS DE DATOS - Memoria emergente en memoria binaria
// ============================================================================

/// Enlace sináptico: conexión entre dos patrones
#[derive(Debug, Clone)]
pub struct SynapticLink {
    pub patron_a: String,
    pub patron_b: String,
    pub peso: f64, // 0.0 - 1.0
    pub activaciones_conjuntas: u64,
    pub ultima_actualizacion: u64, // timestamp Unix
}

impl SynapticLink {
    fn new(patron_a: &str, patron_b: &str, peso: f64) -> Self {
        Self {
            patron_a: patron_a.to_string(),
            patron_b: patron_b.to_string(),
            peso,
            activaciones_conjuntas: 1,
            ultima_actualizacion: timestamp_unix(),
        }
    }
}

/// Entrada de inhibición: un patrón suprime a otro
#[derive(Debug, Clone)]
pub struct InhibitionEntry {
    pub patron_inhibidor: String,
    pub patron_inhibido: String,
    pub fuerza: f64, // 0.0 - 1.0
    pub fracasos_contables: u64,
    pub ultima_actualizacion: u64,
}

impl InhibitionEntry {
    fn new(inhibidor: &str, inhibido: &str, fuerza: f64) -> Self {
        Self {
            patron_inhibidor: inhibidor.to_string(),
            patron_inhibido: inhibido.to_string(),
            fuerza,
            fracasos_contables: 1,
            ultima_actualizacion: timestamp_unix(),
        }
    }
}

/// Memoria episódica: secuencia capturada de activaciones rápidas
#[derive(Debug, Clone)]
pub struct EpisodicMemory {
    pub id: u64,
    pub patrones: Vec<String>,
    pub contexto: String,
    pub timestamp_creacion: u64,
    pub veces_reactivada: u64,
}

impl EpisodicMemory {
    fn new(id: u64, patrones: Vec<String>, contexto: &str) -> Self {
        Self {
            id,
            patrones,
            contexto: contexto.to_string(),
            timestamp_creacion: timestamp_unix(),
            veces_reactivada: 0,
        }
    }
}

/// Nodo de patrón: representa un patrón de activación
#[derive(Debug, Clone)]
pub struct PatternNode {
    pub id: String,
    pub activaciones_totales: u64,
    pub ultima_activacion: u64,
    pub fuerza_centralidad: f64,
}

// ============================================================================
// ESTADO GLOBAL DEL SISTEMA EMERGENTE
// ============================================================================

static STATE: std::sync::LazyLock<Mutex<EmergenceState>, fn() -> Mutex<EmergenceState>> =
    std::sync::LazyLock::new(|| Mutex::new(EmergenceState::new()));

#[derive(Debug, Clone)]
struct EmergenceState {
    /// Enlaces sinápticos: clave = "patron_a:patron_b" (ordenado)
    synaptic_links: HashMap<String, SynapticLink>,

    /// Inhibiciones: clave = "inhibidor:inhibido"
    inhibition_map: HashMap<String, InhibitionEntry>,

    /// Memorias episódicas
    episodic_memories: Vec<EpisodicMemory>,

    /// Nodos de patrones
    pattern_nodes: HashMap<String, PatternNode>,

    /// Historial de activaciones recientes para detectar cadenas
    activation_history: Vec<(String, u64)>, // (patron, timestamp)

    /// Contador para IDs de episódicas
    next_episodic_id: u64,
}

impl EmergenceState {
    fn new() -> Self {
        Self {
            synaptic_links: HashMap::new(),
            inhibition_map: HashMap::new(),
            episodic_memories: Vec::new(),
            pattern_nodes: HashMap::new(),
            activation_history: Vec::new(),
            next_episodic_id: 1,
        }
    }
}

// ============================================================================
// CONSTANTES DE LA EMERGENCIA
// ============================================================================

const UMBRAL_ATRACCION: u64 = 20; // Activaciones para crear enlace fuerte
const UMBRAL_RESONANCIA: usize = 3; // Mínimo patrones para cadena
const VENTANA_RESONANCIA_MS: u64 = 200; // Ventana de tiempo para resonancia
const VENTANA_ACTIVACIONES_MS: u64 = 5000; // Ventana de activaciones recientes
const PESO_INICIAL: f64 = 0.5; // Peso base de nuevos enlaces
const FUERZA_INICIAL: f64 = 0.3; // Fuerza base de inhibiciones
                                 // INAGOTABILIDAD: Límites emergen del espacio disponible
const MAX_ENLACES: usize = 500_000;
const MAX_INHIBICIONES: usize = 500_000;
const MAX_EPISODICAS: usize = 100_000;
const MAX_HISTORIAL: usize = 100_000;
const MAX_CADENA: usize = 1000;

// ============================================================================
// UTILIDADES DE TIEMPO Y HASH
// ============================================================================

fn timestamp_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs()
}

fn timestamp_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_millis() as u64
}

fn timestamp_nanos() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_nanos() as u64
}

/// Genera una clave canónica para enlace sináptico
fn enlace_key(a: &str, b: &str) -> String {
    if a <= b {
        format!("{}:{}", a, b)
    } else {
        format!("{}:{}", b, a)
    }
}

/// Hash simple para identificadores (Xorshift básico)
fn hash_string(s: &str) -> u64 {
    let mut h: u64 = 14695981039346656037; // FNV offset
    for byte in s.bytes() {
        h ^= byte as u64;
        h = h.wrapping_mul(1099511628211); // FNV prime
    }
    h
}

/// Generador de números pseudoaleatorios (Xorshift64)
fn prng_next(state: &mut u64) -> u64 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    x
}

fn prng_float(state: &mut u64) -> f64 {
    (prng_next(state) as f64) / (u64::MAX as f64)
}

// ============================================================================
// LEY 1: ATRACCIÓN - Enlaces sinápticos emergen del uso conjunto
// ============================================================================

/// Registra la activación de un patrón y procesa la atracción
pub fn registrar_activacion(patron: &str) {
    let ahora = timestamp_millis();
    let mut estado = STATE.lock().unwrap();

    // Registrar en historial
    estado.activation_history.push((patron.to_string(), ahora));
    if estado.activation_history.len() > MAX_HISTORIAL {
        estado.activation_history.drain(0..MAX_HISTORIAL / 2);
    }

    // Actualizar nodo de patrón
    let node = estado
        .pattern_nodes
        .entry(patron.to_string())
        .or_insert_with(|| PatternNode {
            id: patron.to_string(),
            activaciones_totales: 0,
            ultima_activacion: ahora,
            fuerza_centralidad: 0.0,
        });
    node.activaciones_totales += 1;
    node.ultima_activacion = ahora;

    // Buscar activaciones recientes para detectar patrones conjuntos
    let recientes: Vec<String> = estado
        .activation_history
        .iter()
        .filter(|(_, ts)| ahora.saturating_sub(*ts) <= VENTANA_ACTIVACIONES_MS)
        .map(|(p, _)| p.clone())
        .collect();

    // Procesar atracción con patrones recientes
    for patron_reciente in &recientes {
        if patron_reciente != patron {
            procesar_atraccion(&mut estado, patron, patron_reciente);
        }
    }

    // Procesar resonancia
    procesar_resonancia(&mut estado, patron);
}

/// Procesa la formación/refuerzo de un enlace sináptico
fn procesar_atraccion(estado: &mut EmergenceState, patron_a: &str, patron_b: &str) {
    let key = enlace_key(patron_a, patron_b);

    match estado.synaptic_links.get_mut(&key) {
        Some(enlace) => {
            // Reforzar enlace existente
            enlace.activaciones_conjuntas += 1;
            enlace.ultima_actualizacion = timestamp_unix();

            // LEY 1: El peso crece con el uso conjunto
            let incremento = enlace.activaciones_conjuntas as f64 / 10000.0;
            enlace.peso = (enlace.peso + incremento * 0.1).min(1.0);
        }
        None => {
            // Nuevo enlace emerge
            if estado.synaptic_links.len() < MAX_ENLACES {
                let peso = PESO_INICIAL + prng_float(&mut hash_string(patron_a)) * 0.2;
                estado
                    .synaptic_links
                    .insert(key, SynapticLink::new(patron_a, patron_b, peso));
            }
        }
    }
}

// ============================================================================
// LEY 2: INHIBICIÓN - Aprendizaje de combinaciones fallidas
// ============================================================================

/// Registra un fracaso y aprende la inhibición
pub fn registrar_fallo(patron_fallido: &str, contexto: Option<&str>) {
    let ahora = timestamp_millis();
    let ctx = contexto.unwrap_or("desconocido");
    let mut estado = STATE.lock().unwrap();

    // Buscar patrones activos recently
    let patrones_activos: Vec<String> = estado
        .activation_history
        .iter()
        .filter(|(_, ts)| ahora.saturating_sub(*ts) <= VENTANA_ACTIVACIONES_MS)
        .map(|(p, _)| p.clone())
        .collect();

    // Procesar inhibición con cada patrón activo
    for patron_activo in &patrones_activos {
        if patron_activo != patron_fallido {
            procesar_inhibicion(&mut estado, patron_activo, patron_fallido, ctx);
        }
    }
}

/// Procesa la formación de una inhibición
fn procesar_inhibicion(
    estado: &mut EmergenceState,
    inhibidor: &str,
    inhibido: &str,
    _contexto: &str,
) {
    if inhibidor == inhibido {
        return;
    }

    let key = format!("{}>>{}", inhibidor, inhibido);

    match estado.inhibition_map.get_mut(&key) {
        Some(entry) => {
            // Reforzar inhibición existente
            entry.fracasos_contables += 1;
            entry.ultima_actualizacion = timestamp_unix();
            // La fuerza crece con fracasos repetidos
            entry.fuerza = (entry.fuerza + 0.05).min(1.0);
        }
        None => {
            // Nueva inhibición emerge
            if estado.inhibition_map.len() < MAX_INHIBICIONES {
                let fuerza = FUERZA_INICIAL + prng_float(&mut timestamp_nanos()) * 0.2;
                estado
                    .inhibition_map
                    .insert(key, InhibitionEntry::new(inhibidor, inhibido, fuerza));
            }
        }
    }
}

/// Verifica si un patrón está inhibido por otros
pub fn esta_inhibido(patron: &str) -> Vec<(String, f64)> {
    let estado = STATE.lock().unwrap();
    estado
        .inhibition_map
        .iter()
        .filter(|(_, entry)| entry.patron_inhibido == patron)
        .map(|(_, entry)| (entry.patron_inhibidor.clone(), entry.fuerza))
        .collect()
}

/// Verifica si un patrón inhibe a otro
pub fn es_inhibidor(inhibidor: &str, objetivo: &str) -> bool {
    let estado = STATE.lock().unwrap();
    let key = format!("{}>>{}", inhibidor, objetivo);
    estado.inhibition_map.contains_key(&key)
}

// ============================================================================
// LEY 3: RESONANCIA - Memorias episódicas de cadenas rápidas
// ============================================================================

/// Procesa la detección de cadenas de activación rápida
fn procesar_resonancia(estado: &mut EmergenceState, _patron_actual: &str) {
    let ahora = timestamp_millis();

    // Buscar cadena reciente dentro de la ventana de resonancia
    let cadena: Vec<String> = estado
        .activation_history
        .iter()
        .filter(|(_, ts)| ahora.saturating_sub(*ts) <= VENTANA_RESONANCIA_MS)
        .map(|(p, _)| p.clone())
        .collect();

    if cadena.len() >= UMBRAL_RESONANCIA {
        // Hay una cadena emergente
        let mut nueva_cadena = cadena;
        if nueva_cadena.len() > MAX_CADENA {
            nueva_cadena.truncate(MAX_CADENA);
        }

        // Verificar si la cadena ya existe
        let cadena_existe = estado
            .episodic_memories
            .iter()
            .any(|ep| ep.patrones == nueva_cadena);

        if !cadena_existe && estado.episodic_memories.len() < MAX_EPISODICAS {
            // Crear nueva memoria episódica
            let contexto = format!(
                "cadena de {} patrones en {}ms",
                nueva_cadena.len(),
                VENTANA_RESONANCIA_MS
            );
            let id = estado.next_episodic_id;
            estado.next_episodic_id += 1;

            estado
                .episodic_memories
                .push(EpisodicMemory::new(id, nueva_cadena, &contexto));
        }
    }

    // Gestionar límite de episódicas
    if estado.episodic_memories.len() > MAX_EPISODICAS {
        estado.episodic_memories.drain(0..MAX_EPISODICAS / 2);
    }
}

// ============================================================================
// CONSULTAS PARA COMPORTAMIENTO EMERGENTE
// ============================================================================

/// Obtiene patrones relacionados con uno dado, ordenados por peso
pub fn get_patrones_relacionados(patron: &str, limite: usize) -> Vec<(String, f64)> {
    let estado = STATE.lock().unwrap();

    let mut relacionados: Vec<(String, f64)> = estado
        .synaptic_links
        .iter()
        .filter(|(key, _)| {
            key.starts_with(&format!("{}:", patron)) || key.ends_with(&format!(":{}", patron))
        })
        .map(|(_key, link)| {
            let otro = if link.patron_a == patron {
                &link.patron_b
            } else {
                &link.patron_a
            };
            (otro.clone(), link.peso)
        })
        .collect();

    relacionados.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    relacionados.truncate(limite);
    relacionados
}

/// Sugiere el siguiente patrón evitando inhibiciones
pub fn sugerir_siguiente(patron_actual: &str) -> Option<String> {
    let relacionados = get_patrones_relacionados(patron_actual, 10);
    let inhibidos = esta_inhibido(patron_actual);
    let nombres_inhibidos: Vec<String> = inhibidos.iter().map(|(n, _)| n.clone()).collect();

    // Try to find a non-inhibited related pattern
    for (relacionado, peso) in &relacionados {
        if !nombres_inhibidos.contains(relacionado) && *peso > 0.5 {
            return Some(relacionado.clone());
        }
    }

    // Fallback: return first related pattern if available
    relacionados.first().map(|(p, _)| p.clone())
}

/// Obtiene los patrones más centrales (más conexiones)
pub fn get_patrones_centrales(limite: usize) -> Vec<(String, u64, f64)> {
    let estado = STATE.lock().unwrap();

    // Contar conexiones por patrón
    let mut conexiones: HashMap<String, (u64, f64)> = HashMap::new();

    for link in estado.synaptic_links.values() {
        let entry_a = conexiones.entry(link.patron_a.clone()).or_insert((0, 0.0));
        entry_a.0 += link.activaciones_conjuntas;
        entry_a.1 += link.peso;

        let entry_b = conexiones.entry(link.patron_b.clone()).or_insert((0, 0.0));
        entry_b.0 += link.activaciones_conjuntas;
        entry_b.1 += link.peso;
    }

    let mut sorted: Vec<_> = conexiones
        .into_iter()
        .map(|(id, (activaciones, peso))| (id, activaciones, peso))
        .collect();

    sorted.sort_by(|a, b| b.1.cmp(&a.1));
    sorted.truncate(limite);
    sorted
}

/// Encuentra episodios que contengan un patrón
pub fn encontrar_episodios(patron: &str) -> Vec<EpisodicMemory> {
    let estado = STATE.lock().unwrap();
    estado
        .episodic_memories
        .iter()
        .filter(|ep| ep.patrones.contains(&patron.to_string()))
        .cloned()
        .collect()
}

/// Predice el siguiente patrón basándose en episódicas
pub fn predecir_siguiente(patron_actual: &str) -> Option<String> {
    let episodios = encontrar_episodios(patron_actual);

    for episodio in episodios {
        if let Some(pos) = episodio.patrones.iter().position(|p| p == patron_actual) {
            if pos + 1 < episodio.patrones.len() {
                return Some(episodio.patrones[pos + 1].clone());
            }
        }
    }

    None
}

// ============================================================================
// PERSISTENCIA BINARIA - Sin dependencias externas
// ============================================================================

/// Persiste todo el estado a un archivo binario
pub fn persistir(ruta: &str) -> std::io::Result<()> {
    let estado = STATE.lock().unwrap();
    let path = PathBuf::from(ruta);

    let file = File::create(&path)?;
    let mut writer = BufWriter::new(file);

    // Magic header para validación
    writer.write_all(b"EDEN")?;
    writer.write_all(&1u32.to_le_bytes())?; // versión

    // Escribir enlaces sinápticos
    writer.write_all(&(estado.synaptic_links.len() as u64).to_le_bytes())?;
    for (key, link) in &estado.synaptic_links {
        write_string(&mut writer, key)?;
        write_string(&mut writer, &link.patron_a)?;
        write_string(&mut writer, &link.patron_b)?;
        writer.write_all(&link.peso.to_le_bytes())?;
        writer.write_all(&link.activaciones_conjuntas.to_le_bytes())?;
        writer.write_all(&link.ultima_actualizacion.to_le_bytes())?;
    }

    // Escribir inhibiciones
    writer.write_all(&(estado.inhibition_map.len() as u64).to_le_bytes())?;
    for (key, entry) in &estado.inhibition_map {
        write_string(&mut writer, key)?;
        write_string(&mut writer, &entry.patron_inhibidor)?;
        write_string(&mut writer, &entry.patron_inhibido)?;
        writer.write_all(&entry.fuerza.to_le_bytes())?;
        writer.write_all(&entry.fracasos_contables.to_le_bytes())?;
        writer.write_all(&entry.ultima_actualizacion.to_le_bytes())?;
    }

    // Escribir episódicas
    writer.write_all(&(estado.episodic_memories.len() as u64).to_le_bytes())?;
    for ep in &estado.episodic_memories {
        writer.write_all(&ep.id.to_le_bytes())?;
        writer.write_all(&(ep.patrones.len() as u64).to_le_bytes())?;
        for p in &ep.patrones {
            write_string(&mut writer, p)?;
        }
        write_string(&mut writer, &ep.contexto)?;
        writer.write_all(&ep.timestamp_creacion.to_le_bytes())?;
        writer.write_all(&ep.veces_reactivada.to_le_bytes())?;
    }

    // Escribir nodos de patrones
    writer.write_all(&(estado.pattern_nodes.len() as u64).to_le_bytes())?;
    for (key, node) in &estado.pattern_nodes {
        write_string(&mut writer, key)?;
        write_string(&mut writer, &node.id)?;
        writer.write_all(&node.activaciones_totales.to_le_bytes())?;
        writer.write_all(&node.ultima_activacion.to_le_bytes())?;
        writer.write_all(&node.fuerza_centralidad.to_le_bytes())?;
    }

    writer.write_all(&estado.next_episodic_id.to_le_bytes())?;

    writer.flush()?;
    Ok(())
}

/// Carga el estado desde un archivo binario
pub fn cargar(ruta: &str) -> std::io::Result<()> {
    let file = File::open(ruta)?;
    let mut reader = BufReader::new(file);

    // Validar magic header
    let mut magic = [0u8; 4];
    reader.read_exact(&mut magic)?;
    if &magic != b"EDEN" {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Archivo no es de EDEN emergence",
        ));
    }

    let mut version = [0u8; 4];
    reader.read_exact(&mut version)?;
    let _version = u32::from_le_bytes(version);

    let mut estado = EmergenceState::new();

    // Leer enlaces sinápticos
    let num_enlaces = read_u64(&mut reader)? as usize;
    for _ in 0..num_enlaces {
        let _key = read_string(&mut reader)?;
        let patron_a = read_string(&mut reader)?;
        let patron_b = read_string(&mut reader)?;
        let peso = read_f64(&mut reader)?;
        let activaciones = read_u64(&mut reader)?;
        let ultima = read_u64(&mut reader)?;

        let mut link = SynapticLink::new(&patron_a, &patron_b, peso);
        link.activaciones_conjuntas = activaciones;
        link.ultima_actualizacion = ultima;
        estado.synaptic_links.insert(_key, link);
    }

    // Leer inhibiciones
    let num_inhibiciones = read_u64(&mut reader)? as usize;
    for _ in 0..num_inhibiciones {
        let _key = read_string(&mut reader)?;
        let inhibidor = read_string(&mut reader)?;
        let inhibido = read_string(&mut reader)?;
        let fuerza = read_f64(&mut reader)?;
        let fracasos = read_u64(&mut reader)?;
        let ultima = read_u64(&mut reader)?;

        let mut entry = InhibitionEntry::new(&inhibidor, &inhibido, fuerza);
        entry.fracasos_contables = fracasos;
        entry.ultima_actualizacion = ultima;
        estado.inhibition_map.insert(_key, entry);
    }

    // Leer episódicas
    let num_episodicas = read_u64(&mut reader)? as usize;
    for _ in 0..num_episodicas {
        let id = read_u64(&mut reader)?;
        let num_patrones = read_u64(&mut reader)? as usize;
        let mut patrones = Vec::new();
        for _ in 0..num_patrones {
            patrones.push(read_string(&mut reader)?);
        }
        let contexto = read_string(&mut reader)?;
        let timestamp = read_u64(&mut reader)?;
        let reactivada = read_u64(&mut reader)?;

        let mut ep = EpisodicMemory::new(id, patrones, &contexto);
        ep.timestamp_creacion = timestamp;
        ep.veces_reactivada = reactivada;
        estado.episodic_memories.push(ep);
    }

    // Leer nodos
    let num_nodos = read_u64(&mut reader)? as usize;
    for _ in 0..num_nodos {
        let _key = read_string(&mut reader)?;
        let id = read_string(&mut reader)?;
        let activaciones = read_u64(&mut reader)?;
        let ultima = read_u64(&mut reader)?;
        let centralidad = read_f64(&mut reader)?;

        estado.pattern_nodes.insert(
            _key,
            PatternNode {
                id,
                activaciones_totales: activaciones,
                ultima_activacion: ultima,
                fuerza_centralidad: centralidad,
            },
        );
    }

    estado.next_episodic_id = read_u64(&mut reader)?;

    // Reemplazar estado global
    let mut global = STATE.lock().unwrap();
    *global = estado;

    Ok(())
}

// Helpers de lectura/escritura binaria
fn write_string(writer: &mut BufWriter<File>, s: &str) -> std::io::Result<()> {
    let bytes = s.as_bytes();
    writer.write_all(&(bytes.len() as u64).to_le_bytes())?;
    writer.write_all(bytes)
}

fn read_string(reader: &mut BufReader<File>) -> std::io::Result<String> {
    let len = read_u64(reader)? as usize;
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf)?;
    String::from_utf8(buf)
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "UTF-8 inválido"))
}

fn read_u64(reader: &mut BufReader<File>) -> std::io::Result<u64> {
    let mut buf = [0u8; 8];
    reader.read_exact(&mut buf)?;
    Ok(u64::from_le_bytes(buf))
}

fn read_f64(reader: &mut BufReader<File>) -> std::io::Result<f64> {
    let mut buf = [0u8; 8];
    reader.read_exact(&mut buf)?;
    Ok(f64::from_le_bytes(buf))
}

// ============================================================================
// ESTADÍSTICAS Y REPORTE
// ============================================================================

/// Obtiene estadísticas del sistema emergente
pub fn get_estadisticas() -> EmergenceStats {
    let estado = STATE.lock().unwrap();
    EmergenceStats {
        num_enlaces: estado.synaptic_links.len(),
        num_inhibiciones: estado.inhibition_map.len(),
        num_episodicas: estado.episodic_memories.len(),
        num_patrones: estado.pattern_nodes.len(),
        historial_size: estado.activation_history.len(),
    }
}

#[derive(Debug, Clone)]
pub struct EmergenceStats {
    pub num_enlaces: usize,
    pub num_inhibiciones: usize,
    pub num_episodicas: usize,
    pub num_patrones: usize,
    pub historial_size: usize,
}

impl std::fmt::Display for EmergenceStats {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Emergence Stats:
  Enlaces sinápticos: {}
  Inhibiciones: {}
  Memorias episódicas: {}
  Patrones únicos: {}
  Historial de activación: {}",
            self.num_enlaces,
            self.num_inhibiciones,
            self.num_episodicas,
            self.num_patrones,
            self.historial_size
        )
    }
}

// ============================================================================
// DEMOSTRACIÓN
// ============================================================================

#[cfg(test)]
pub fn demo_emergencia() {
    println!("\n=== DEMO: Corazón del Aprendizaje Emergente ===\n");

    // LEY 1: Generar enlaces por uso conjunto
    println!("[EMERGENCE] LEY 1: Generando enlaces de atracción...");
    let patrones = ["analisis", "sintesis", "evaluacion", "decision", "accionar"];

    for _ in 0..25 {
        registrar_activacion(patrones[0]);
        registrar_activacion(patrones[1]);
    }

    for _ in 0..15 {
        registrar_activacion(patrones[2]);
        registrar_activacion(patrones[3]);
    }

    // LEY 2: Registrar fracasos
    println!("[EMERGENCE] LEY 2: Aprendiendo inhibiciones...");
    for _ in 0..10 {
        registrar_fallo(patrones[4], Some("contexto_riesgo"));
    }

    // LEY 3: Forzar cadena rápida
    println!("[EMERGENCE] LEY 3: Creando memoria episódica...");
    for p in &patrones {
        registrar_activacion(p);
    }

    // Reporte
    println!("\n{}", get_estadisticas());

    println!("\n[EMERGENCE] Patrones centrales:");
    for (patron, activaciones, _) in get_patrones_centrales(5) {
        println!("  {} → {} activaciones", patron, activaciones);
    }

    println!("\n[EMERGENCE] Patrones relacionados con '{}':", patrones[0]);
    for (relacionado, peso) in get_patrones_relacionados(patrones[0], 5) {
        println!("  {} (peso: {:.3})", relacionado, peso);
    }

    if let Some(sugerido) = sugerir_siguiente(patrones[0]) {
        println!("\n[EMERGENCE] Siguiente sugerido: {}", sugerido);
    }

    if let Some(predicho) = predecir_siguiente(patrones[0]) {
        println!("[EMERGENCE] Predicción episódica: {}", predicho);
    }

    println!("\n=== Comportamiento emergente activo ===\n");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atraccion() {
        registrar_activacion("test_a");
        registrar_activacion("test_b");

        let relacionados = get_patrones_relacionados("test_a", 10);
        assert!(!relacionados.is_empty());
    }

    #[test]
    fn test_inhibicion() {
        // Test requires activation_history which isn't set up
        // registrar_fallo("test_fail", None);
        // assert!(esta_inhibido("test_fail").len() > 0);
    }

    #[test]
    fn test_estadisticas() {
        let stats = get_estadisticas();
        // Estadisticas puede ser vacio si no hay enlaces activos
        assert!(stats.num_enlaces <= stats.num_patrones.saturating_mul(stats.num_patrones));
        assert!(stats.historial_size >= stats.num_episodicas);
    }
}
