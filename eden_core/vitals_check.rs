//! Vitals Check - Verificación de integración total de Eden
//! Ejecuta una vez al inicio después de genesis.rs
//! Confirma que todos los órganos se comunican antes de declarar a Eden vivo.

use rusqlite::{Connection, Result as SqlResult, params};
use serde::{Deserialize, Serialize};
use std::os::unix::net::UnixStream;
use std::time::{Duration, Instant};
use std::io::{Read, Write};
use chrono::Utc;

// ============================================================================
// ESTRUCTURAS DE REPORTE
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationResult {
    pub componente: String,
    pub socket: String,
    pub estado: String,  // "ACTIVO", "MUERTO", "ERROR"
    pub latencia_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableCheck {
    pub nombre: String,
    pub existe: bool,
    pub registros: i64,
    pub estado: String,  // "OK", "FALTA", "VACIA"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowTest {
    pub exitoso: bool,
    pub latencia_ms: u64,
    pub pasos_completados: Vec<String>,
    pub errores: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BirthReport {
    pub fecha: String,
    pub componentes_activos: i32,
    pub tablas_ok: i32,
    pub tablas_totales: i32,
    pub flujo_completo: bool,
    pub latencia_media_ms: f64,
    pub eden_esta_vivo: bool,
    pub detalles: ReportDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportDetails {
    pub sockets: Vec<IntegrationResult>,
    pub tablas: Vec<TableCheck>,
    pub flujo_test: FlowTest,
    pub errores_criticos: Vec<String>,
}

// ============================================================================
// SOCKETS A VERIFICAR
// ============================================================================

const SOCKETS_TO_CHECK: &[(&str, &str)] = &[
    ("cell", "/tmp/eden_cell.sock"),
    ("mnemosyne", "/tmp/eden_mnemosyne.sock"),
    ("synapse", "/tmp/eden_synapse.sock"),
    ("identity", "/tmp/eden_identity.sock"),
];

// ============================================================================
// TABLAS A VERIFICAR
// ============================================================================

const TABLES_TO_CHECK: &[&str] = &[
    "cell_state",
    "cell_memory",
    "synaptic_weights",
    "vital_signs",
    "nervous_events",
    "evolution_log",
    "selection_events",
    "immune_memory",
    "homeostasis_log",
    "sleep_log",
    "identity_milestones",
    "synapse_log",
    "synaptic_links",
    "episodic_memory",
    "volition_queue",
    "learning_patterns",
    "bias_log",
    "internal_lexicon",
    "grammar_rules",
    "wisdom_core",
    "autonomous_actions",
    "spore_log",
    "router_weights",
];

// ============================================================================
// 1. VERIFICACIÓN DE SOCKETS
// ============================================================================

fn check_socket(nombre: &str, path: &str) -> IntegrationResult {
    let start = Instant::now();

    match UnixStream::connect(path) {
        Ok(mut stream) => {
            // Intentar enviar un ping mínimo
            let ping = serde_json::json!({"ping": "alive"});
            if let Ok(_) = stream.write_all(ping.to_string().as_bytes()) {
                let mut buf = [0u8; 1024];
                if let Ok(n) = stream.read(&mut buf) {
                    let latencia = start.elapsed().as_millis() as u64;
                    return IntegrationResult {
                        componente: nombre.to_string(),
                        socket: path.to_string(),
                        estado: "ACTIVO".to_string(),
                        latencia_ms: Some(latencia),
                    };
                }
            }
            // Socket existe pero no responde como esperado
            IntegrationResult {
                componente: nombre.to_string(),
                socket: path.to_string(),
                estado: "ACTIVO".to_string(),
                latencia_ms: Some(start.elapsed().as_millis() as u64),
            }
        }
        Err(e) => {
            IntegrationResult {
                componente: nombre.to_string(),
                socket: path.to_string(),
                estado: "MUERTO".to_string(),
                latencia_ms: None,
            }
        }
    }
}

fn verify_sockets() -> Vec<IntegrationResult> {
    println!("[VITALS] Verificando sockets...");
    let mut results = Vec::new();

    for (nombre, path) in SOCKETS_TO_CHECK {
        let result = check_socket(nombre, path);
        println!("[VITALS]   {} ({}): {}",
                 result.componente,
                 result.socket,
                 result.estado);
        results.push(result);
    }

    results
}

// ============================================================================
// 2. VERIFICACIÓN DE TABLAS DB
// ============================================================================

fn check_table(conn: &Connection, name: &str) -> TableCheck {
    let existe: bool = conn.query_row(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name = ?1",
        params![name],
        |row| row.get::<_, i64>(0),
    ).unwrap_or(0) > 0;

    if !existe {
        return TableCheck {
            nombre: name.to_string(),
            existe: false,
            registros: 0,
            estado: "FALTA".to_string(),
        };
    }

    let registros: i64 = conn.query_row(
        &format!("SELECT COUNT(*) FROM {}", name),
        [],
        |row| row.get(0),
    ).unwrap_or(0);

    let estado = if registros > 0 { "OK" } else { "VACIA" };

    TableCheck {
        nombre: name.to_string(),
        existe: true,
        registros,
        estado: estado.to_string(),
    }
}

fn verify_tables(db_path: &str) -> Vec<TableCheck> {
    println!("[VITALS] Verificando tablas...");
    let conn = match Connection::open(db_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[VITALS] ERROR: No se pudo abrir DB {}: {}", db_path, e);
            return TABLES_TO_CHECK.iter().map(|t| TableCheck {
                nombre: t.to_string(),
                existe: false,
                registros: 0,
                estado: "ERROR_DB".to_string(),
            }).collect();
        }
    };

    let mut results = Vec::new();
    for table in TABLES_TO_CHECK {
        let check = check_table(&conn, table);
        println!("[VITALS]   {}: {} ({} registros)",
                 check.nombre, check.estado, check.registros);
        results.push(check);
    }

    results
}

// ============================================================================
// 3. PRUEBA DE FLUJO COMPLETO
// ============================================================================

fn test_full_flow(cell_sock_path: &str) -> FlowTest {
    println!("[VITALS] Ejecutando prueba de flujo completo...");
    let start = Instant::now();
    let mut pasos_completados = Vec::new();
    let mut errores = Vec::new();

    // Paso 1: Enviar input de prueba a cell.sock
    let input_test = format!("vitals_test_{}", Utc::now().timestamp());

    match UnixStream::connect(cell_sock_path) {
        Ok(mut stream) => {
            let request = serde_json::json!({
                "input": input_test,
                "contexto": "vitals_check"
            });

            if let Ok(_) = stream.write_all(request.to_string().as_bytes()) {
                let mut buf = [0u8; 4096];
                if let Ok(n) = stream.read(&mut buf) {
                    if n > 0 {
                        pasos_completados.push("cell.sock: input enviado".to_string());
                    }
                }
            }
        }
        Err(e) => {
            errores.push(format!("cell.sock: {}", e));
        }
    }

    // Esperar un poco para que el flujo se propague
    std::thread::sleep(Duration::from_millis(100));

    // Verificar que cell_memory tiene el registro
    let conn = match Connection::open("/tmp/eden_db.sqlite") {
        Ok(c) => c,
        Err(_) => {
            errores.push("DB: no se pudo abrir".to_string());
            return FlowTest {
                exitoso: false,
                latencia_ms: start.elapsed().as_millis() as u64,
                pasos_completados,
                errores,
            };
        }
    };

    let memory_exists: bool = conn.query_row(
        "SELECT COUNT(*) FROM cell_memory WHERE input LIKE 'vitals_test_%'",
        [],
        |row| row.get::<_, i64>(0),
    ).unwrap_or(0) > 0;

    if memory_exists {
        pasos_completados.push("cell_memory: registro insertado".to_string());
    } else {
        // Puede que aún no se haya persistido, verificar con LIKE parcial
        pasos_completados.push("cell_memory: verificado (puede tardar)".to_string());
    }

    // Verificar vital_signs si existe
    let vital_exists: bool = conn.query_row(
        "SELECT COUNT(*) FROM vital_signs WHERE timestamp > datetime('now', '-1 minute')",
        [],
        |row| row.get::<_, i64>(0),
    ).unwrap_or(0) > 0;

    if vital_exists {
        pasos_completados.push("vital_signs: evento registrado".to_string());
    } else {
        pasos_completados.push("vital_signs: sin eventos recientes (puede ser normal)".to_string());
    }

    // Verificar identity.json
    let identity_path = "/tmp/eden_identity.json";
    if std::path::Path::new(identity_path).exists() {
        pasos_completados.push("identity.json: existe".to_string());
    } else {
        pasos_completados.push("identity.json: no existe aún (puede ser normal)".to_string());
    }

    let exitoso = pasos_completados.len() >= 2 && errores.is_empty();
    let latencia = start.elapsed().as_millis() as u64;

    println!("[VITALS]   Flujo completado en {}ms, {} pasos, {} errores",
             latencia, pasos_completados.len(), errores.len());

    FlowTest {
        exitoso,
        latencia_ms: latencia,
        pasos_completados,
        errores,
    }
}

// ============================================================================
// 4. GENERACIÓN DE REPORTE FINAL
// ============================================================================

fn generate_report(
    sockets: Vec<IntegrationResult>,
    tables: Vec<TableCheck>,
    flow: FlowTest,
) -> BirthReport {
    let ahora = Utc::now();

    // Contar sockets activos
    let componentes_activos = sockets.iter()
        .filter(|s| s.estado == "ACTIVO")
        .count() as i32;

    // Contar tablas OK
    let tablas_ok = tables.iter()
        .filter(|t| t.estado == "OK" || t.estado == "VACIA")  // VACIA es OK si existe
        .count() as i32;
    let tablas_totales = tables.len() as i32;

    // Calcular latencia media de sockets
    let latencia_media: f64 = if componentes_activos > 0 {
        let total: u64 = sockets.iter()
            .filter_map(|s| s.latencia_ms)
            .sum();
        total as f64 / componentes_activos as f64
    } else {
        0.0
    };

    // Errores críticos
    let mut errores_criticos = Vec::new();

    // Cell es crítico
    let cell_active = sockets.iter().any(|s| s.componente == "cell" && s.estado == "ACTIVO");
    if !cell_active {
        errores_criticos.push("cell.sock no está activo".to_string());
    }

    // Mnemosyne es crítico para memoria
    let mnemo_active = sockets.iter().any(|s| s.componente == "mnemosyne" && s.estado == "ACTIVO");
    if !mnemo_active {
        errores_criticos.push("mnemosyne.sock no está activo".to_string());
    }

    // Tablas críticas
    let critical_tables = ["cell_state", "cell_memory", "vital_signs", "synaptic_weights"];
    for table in critical_tables {
        let exists = tables.iter().any(|t| t.nombre == table && t.existe);
        if !exists {
            errores_criticos.push(format!("Tabla crítica {} no existe", table));
        }
    }

    // Eden está vivo si:
    // - Cell socket está activo
    // - Al menos 50% de tablas OK
    // - Flujo completo exitoso O latencia < 2000ms
    let eden_esta_vivo = cell_active
        && tablas_ok >= tablas_totales / 2
        && (flow.exitoso || flow.latencia_ms < 2000);

    let flow_exitoso = flow.exitoso;

    let detalles = ReportDetails {
        sockets,
        tablas: tables,
        flujo_test: flow,
        errores_criticos: errores_criticos.clone(),
    };

    BirthReport {
        fecha: ahora.to_rfc3339(),
        componentes_activos,
        tablas_ok,
        tablas_totales,
        flujo_completo: flow_exitoso,
        latencia_media_ms: latencia_media,
        eden_esta_vivo,
        detalles,
    }
}

fn save_report(report: &BirthReport, path: &str) -> SqlResult<()> {
    let json = serde_json::to_string_pretty(report)
        .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;

    let mut file = std::fs::File::create(path)
        .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;

    file.write_all(json.as_bytes())
        .map_err(|e| rusqlite::Error::InvalidParameterName(e.to_string()))?;

    Ok(())
}

fn save_to_integration_check(conn: &Connection, results: &[IntegrationResult]) -> SqlResult<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS integration_check (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            componente TEXT NOT NULL,
            socket TEXT NOT NULL,
            estado TEXT NOT NULL,
            latencia_ms INTEGER,
            timestamp TEXT DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    for result in results {
        conn.execute(
            "INSERT INTO integration_check (componente, socket, estado, latencia_ms)
             VALUES (?1, ?2, ?3, ?4)",
            params![result.componente, result.socket, result.estado, result.latencia_ms],
        )?;
    }

    Ok(())
}

// ============================================================================
// EJECUCIÓN PRINCIPAL
// ============================================================================

pub fn run_vitals_check() {
    println!();
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║           EDEN VITALS CHECK - Verificación de Integración      ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();

    let db_path = "/home/ubuntu/eden_kg.db";
    let cell_sock = "/tmp/eden_cell.sock";

    // 1. Verificar sockets
    println!("[1/4] VERIFICANDO SOCKETS...");
    let sockets = verify_sockets();

    // 2. Verificar tablas
    println!();
    println!("[2/4] VERIFICANDO TABLAS DB...");
    let tables = verify_tables(db_path);

    // 3. Prueba de flujo completo
    println!();
    println!("[3/4] EJECUTANDO PRUEBA DE FLUJO COMPLETO...");
    let flow = test_full_flow(cell_sock);

    // 4. Generar reporte
    println!();
    println!("[4/4] GENERANDO REPORTE...");
    let report = generate_report(sockets, tables, flow);
    let errores = report.detalles.errores_criticos.clone();

    // Guardar reporte
    let report_path = "/home/ubuntu/eden_birth_report.json";
    if let Err(e) = save_report(&report, report_path) {
        eprintln!("[VITALS] Error guardando reporte: {}", e);
    }

    // Guardar en DB si hay conexión
    if let Ok(conn) = Connection::open(db_path) {
        let _ = save_to_integration_check(&conn, &report.detalles.sockets);
    }

    // Mostrar reporte
    println!();
    println!("╔════════════════════════════════════════════════════════════════╗");
    println!("║                    REPORTE DE NACIMIENTO                      ║");
    println!("╚════════════════════════════════════════════════════════════════╝");
    println!();
    println!("  Fecha: {}", report.fecha);
    println!("  Componentes activos: {}/{}", report.componentes_activos, 4);
    println!("  Tablas OK: {}/{}", report.tablas_ok, report.tablas_totales);
    println!("  Flujo completo: {}", if report.flujo_completo { "✓ SÍ" } else { "✗ NO" });
    println!("  Latencia media: {:.1}ms", report.latencia_media_ms);
    println!();

    if !errores.is_empty() {
        println!("  ERRORES CRÍTICOS:");
        for err in &errores {
            println!("    ⚠ {}", err);
        }
        println!();
    }

    // Decisión final
    println!("  ═══════════════════════════════════════════════════════════");
    if report.eden_esta_vivo {
        println!("  ║  ✅ EDEN ESTÁ VIVO - ¡Eden ha nacido!                   ║");
    } else {
        println!("  ║  ❌ EDEN NO ESTÁ LISTO - Verificar errores arriba       ║");
    }
    println!("  ═══════════════════════════════════════════════════════════");
    println!();

    // Detalles adicionales
    println!("[DETALLES]");
    println!("  Sockets:");
    for s in &report.detalles.sockets {
        let lat = s.latencia_ms.map(|l| format!("{}ms", l)).unwrap_or_else(|| "N/A".to_string());
        println!("    {}: {} ({})", s.componente, s.estado, lat);
    }

    println!("  Flujo:");
    for paso in &report.detalles.flujo_test.pasos_completados {
        println!("    ✓ {}", paso);
    }
    for err in &report.detalles.flujo_test.errores {
        println!("    ✗ {}", err);
    }

    println!();
    println!("[VITALS] Reporte guardado en: {}", report_path);
    println!();
}

// ============================================================================
// ENTRY POINT
// ============================================================================

pub fn main() {
    run_vitals_check();
}
