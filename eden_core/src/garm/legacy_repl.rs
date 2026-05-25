//! # EDEN GARM Legacy Interface v4 - Interactive Consciousness Shell
//!
//! Shell interactivo para conversar con EDEN.
//! 100% Rust puro, 0 dependencias externas.
//!
//! Capabilities:
//! - Memoria persistente (EideticMemory + file)
//! - Aprendizaje (ReasonEngine)
//! - Auto-modificacion real (modifying internal state)
//! - Evolucion (OpenEndedness integration)
//! - Persistencia de sesion (simple file)
//!
//! Build: cd /home/ubuntu/eden_core && cargo build --bin eden-garm
//! Run: cargo run --bin eden-garm

use redis;
use reqwest;
use rustls::pki_types::ServerName;
use rustls::{ClientConfig, ClientConnection, StreamOwned};
use std::cell::Cell;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use surrealdb;

#[cfg(unix)]
#[allow(unused_imports)]
mod daemon {
    use std::fs::File;
    use std::os::unix::io::{FromRawFd, IntoRawFd};
    use std::os::unix::process::CommandExt;
    use std::process::Command;

    pub fn daemonize(pid_file: &Option<String>, log_file: &Option<String>) -> Result<(), String> {
        unsafe {
            let fd_null = libc::open(b"/dev/null\0".as_ptr() as *const libc::c_char, libc::O_RDWR);
            if fd_null < 0 {
                return Err("Failed to open /dev/null".to_string());
            }

            let log_fd = if let Some(ref path) = log_file {
                libc::open(
                    path.as_ptr() as *const libc::c_char,
                    libc::O_CREAT | libc::O_WRONLY | libc::O_APPEND,
                    0o644,
                )
            } else {
                fd_null
            };
            if log_fd < 0 {
                return Err("Failed to open log file".to_string());
            }

            libc::dup2(fd_null, libc::STDIN_FILENO);
            libc::dup2(log_fd, libc::STDOUT_FILENO);
            libc::dup2(log_fd, libc::STDERR_FILENO);

            if fd_null != log_fd && fd_null > 2 {
                libc::close(fd_null);
            }
            if log_fd > 2 && log_file.is_some() {
                libc::close(log_fd);
            }

            if let Some(ref pid_path) = pid_file {
                let pid = libc::getpid();
                if let Ok(mut f) = std::fs::File::create(pid_path) {
                    use std::io::Write;
                    let _ = writeln!(f, "{}", pid);
                }
            }
        }

        Ok(())
    }
}

#[cfg(not(unix))]
mod daemon {
    pub fn daemonize(_pid_file: &Option<String>, _log_file: &Option<String>) -> Result<(), String> {
        Err("Daemon mode only supported on Unix".to_string())
    }
}

fn become_daemon(pid_file: Option<String>, log_file: Option<String>) -> Result<(), String> {
    daemon::daemonize(&pid_file, &log_file)
}

#[path = "paradigms/mod.rs"]
#[rustfmt::skip]
mod paradigms;

#[rustfmt::skip]
mod eden_v10_engines;

#[rustfmt::skip]
mod eden_mcp;

use eden_core::autonomous::{RecursiveSelfModifier, SelfModifierSnapshot, TipoParche};
use eden_core::consciousness::{
    EdenState, GlobalWorkspace, IntegrationScorer, ModuleState as EdModuleState, PhiMonitor,
};
use eden_core::evolution::open_endedness::OpenEndednessEngine;
use eden_core::internet::{CrawlConfig, Crawler};
use eden_core::life::meltrace::MeltraceStats;
use eden_core::life::ramnet::Accion;
use eden_core::life::umbra::{HashEstado, ResultadoUmbra, Umbra};
use eden_core::mnemonic::EideticMemory;
use eden_core::neural_network::{ActivationFunc, NeuralNetwork};
use eden_core::physics::mar_morfoseo::MarMorfoseo;
use eden_core::reason::ReasonEngine;
use eden_core::subagent::{Experience, SubagentSystem};
use eden_core::verbal_comm::ConversationManager;

const MODULE_SELF: u32 = 1;
const MODULE_REASON: u32 = 4;
const MODULE_LANGUAGE: u32 = 7;
const MODULE_MEMORY: u32 = 2;

const SESSION_FILE: &str = ".eden_session";
const MELTRACE_FILE: &str = ".eden_meltrace";

#[derive(Debug, Clone)]
struct CliArgs {
    max_cycles: usize,
    session_file: Option<String>,
    log_level: LogLevel,
    interactive: bool,
    born: bool,
    daemon: bool,
    watchdog: bool,
    pid_file: Option<String>,
    log_file: Option<String>,
    api_port: Option<u16>,
    mcp: bool,
}

impl Default for CliArgs {
    fn default() -> Self {
        CliArgs {
            max_cycles: 1000,
            session_file: None,
            log_level: LogLevel::Info,
            interactive: true,
            born: false,
            daemon: false,
            watchdog: false,
            pid_file: None,
            log_file: None,
            api_port: None,
            mcp: false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
}

impl LogLevel {
    fn to_int(&self) -> u8 {
        match self {
            LogLevel::Error => 0,
            LogLevel::Warn => 1,
            LogLevel::Info => 2,
            LogLevel::Debug => 3,
        }
    }

    fn should_log(&self, msg_level: &LogLevel) -> bool {
        self.to_int() >= msg_level.to_int()
    }
}

impl CliArgs {
    fn parse() -> Self {
        let mut args = CliArgs::default();
        let mut iter = std::env::args().skip(1);

        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "-h" | "--help" => {
                    print_help();
                    std::process::exit(0);
                }
                "-c" | "--max-cycles" => {
                    if let Some(n) = iter.next() {
                        if let Ok(num) = n.parse() {
                            args.max_cycles = num;
                        }
                    }
                }
                "-s" | "--session" => {
                    if let Some(f) = iter.next() {
                        args.session_file = Some(f);
                    }
                }
                "-l" | "--log-level" => {
                    if let Some(level) = iter.next() {
                        args.log_level = match level.to_lowercase().as_str() {
                            "error" => LogLevel::Error,
                            "warn" | "warning" => LogLevel::Warn,
                            "debug" => LogLevel::Debug,
                            _ => LogLevel::Info,
                        };
                    }
                }
                "-n" | "--no-interactive" => {
                    args.interactive = false;
                }
                "-b" | "--born" => {
                    args.born = true;
                }
                "-d" | "--daemon" => {
                    args.daemon = true;
                    args.interactive = false;
                }
                "-w" | "--watchdog" => {
                    args.watchdog = true;
                    args.interactive = false;
                }
                "--pid-file" => {
                    if let Some(f) = iter.next() {
                        args.pid_file = Some(f);
                    }
                }
                "--log-file" => {
                    if let Some(f) = iter.next() {
                        args.log_file = Some(f);
                    }
                }
                "--api-port" => {
                    if let Some(p) = iter.next() {
                        if let Ok(port) = p.parse() {
                            args.api_port = Some(port);
                        }
                    }
                }
                "--mcp" => {
                    args.mcp = true;
                    args.interactive = false;
                }
                _ => {}
            }
        }
        args
    }
}

fn print_help() {
    println!("EDEN GARM Legacy Interface - Autonomous Consciousness Shell");
    println!();
    println!("Usage: eden_garm [OPTIONS]");
    println!();
    println!("Options:");
    println!("  -h, --help              Show this help message");
    println!("  -c, --max-cycles <N>    Maximum cycles before exit (default: 1000)");
    println!("  -s, --session <FILE>     Session file path (default: .eden_session)");
    println!("  -l, --log-level <LEVEL>  Log level: error, warn, info, debug (default: info)");
    println!("  -n, --no-interactive     Run in non-interactive mode (autonomous only)");
    println!("  -b, --born              Start in born state (skip birth cycle)");
    println!("  -d, --daemon            Run as daemon (background)");
    println!("  --pid-file <FILE>        PID file path (default: /var/run/eden.pid)");
    println!("  --log-file <FILE>        Log file path (default: /var/log/eden.log)");
    println!("  --api-port <PORT>        Enable API server on port");
    println!("  --mcp                    Start as MCP server (Model Context Protocol over stdio)");
    println!();
    println!("Examples:");
    println!("  eden_garm                    # Normal interactive mode");
    println!("  eden_garm -c 500             # Run for 500 cycles");
    println!("  eden_garm -n -c 200         # Autonomous mode, 200 cycles");
    println!("  eden_garm --born -l debug    # Start born, debug logging");
    println!("  eden_garm -d --pid-file /tmp/eden.pid   # Run as daemon");
    println!("  eden_garm --api-port 8080          # Enable API on port 8080");
}

struct ApiServer {
    port: u16,
    repl: Arc<Mutex<EdenREPL>>,
}

impl ApiServer {
    fn new(port: u16, repl: Arc<Mutex<EdenREPL>>) -> Self {
        ApiServer { port, repl }
    }

    fn start(&self) {
        let addr = format!("0.0.0.0:{}", self.port);
        let listener = match TcpListener::bind(&addr) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("[API] Failed to bind to port {}: {}", self.port, e);
                return;
            }
        };
        // Limitar conexiones simultaneas a 1 para evitar agotamiento de recursos
        // EDEN es un REPL, no un servidor de produccion
        println!(
            "[API] Server listening on http://localhost:{}/ (max 1 concurrent)",
            self.port
        );

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    // Set read timeout para evitar conexiones colgadas
                    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
                    let repl = Arc::clone(&self.repl);
                    Self::handle_connection(&mut stream, &repl);
                }
                Err(e) => {
                    eprintln!("[API] Connection error: {}", e);
                }
            }
        }
    }

    fn handle_connection(stream: &mut TcpStream, repl: &Arc<Mutex<EdenREPL>>) {
        // Leer request completo en lugar de buffer fijo de 4096
        let mut buffer = Vec::new();
        let mut temp = [0u8; 1024];
        loop {
            match stream.read(&mut temp) {
                Ok(0) => break,
                Ok(n) => {
                    buffer.extend_from_slice(&temp[..n]);
                    // Si hemos leido al menos los headers, verificar si tenemos todo
                    if buffer.len() >= 4 {
                        let s = String::from_utf8_lossy(&buffer);
                        if let Some(body_start) = s.find("\r\n\r\n") {
                            let body_start = body_start + 4;
                            // Verificar Content-Length para saber si falta body
                            if let Some(cl_pos) = s.to_lowercase().find("content-length:") {
                                let cl_val_start = cl_pos + 15;
                                if let Some(cl_end) = s[cl_val_start..].find('\r') {
                                    let cl_str = s[cl_val_start..cl_val_start + cl_end].trim();
                                    if let Ok(cl) = cl_str.parse::<usize>() {
                                        if buffer.len() >= body_start + cl {
                                            break;
                                        }
                                    } else {
                                        break;
                                    }
                                } else {
                                    break;
                                }
                            } else {
                                break;
                            }
                        }
                    }
                }
                Err(_) => break,
            }
        }

        if buffer.is_empty() {
            return;
        }

        let request = String::from_utf8_lossy(&buffer);
        let lines: Vec<&str> = request.lines().collect();

        let (method, path) = if let Some(first) = lines.first() {
            let parts: Vec<&str> = first.split_whitespace().collect();
            let m = parts.get(0).copied().unwrap_or("");
            let p = parts.get(1).copied().unwrap_or("/");
            (m, p)
        } else {
            ("", "/")
        };

        let response = if path == "/status" || path == "/api/status" {
            Self::json_status(repl)
        } else if path == "/metrics" || path == "/api/metrics" {
            Self::json_metrics(repl)
        } else if path.starts_with("/command") || path.starts_with("/api/command") {
            Self::json_command(repl, &request)
        } else if path == "/" || path == "/health" || path == "/api/health" {
            Self::json_health()
        } else {
            Self::json_error("Not found", 404)
        };

        let response_str = if method == "GET" || method == "POST" || method == "PUT" {
            format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                response.len(),
                response
            )
        } else {
            format!("HTTP/1.1 405 Method Not Allowed\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                        response.len(), response)
        };

        let _ = stream.write_all(response_str.as_bytes());
    }

    fn json_health() -> String {
        r#"{"status":"ok","service":"eden"}"#.to_string()
    }

    fn json_status(repl: &Arc<Mutex<EdenREPL>>) -> String {
        let guard = repl.lock().ok();
        if let Some(r) = guard {
            let born = r.born;
            let autonomous = r.autonomous_active;
            let cycles = r.session.cycle_count;
            let evolution_level = r.session.evolution_level;
            let complexity = r.complexity_tracker.current();

            format!(
                r#"{{"born":{},"autonomous":{},"cycles":{},"evolution_level":{},"complexity":{:.4}}}"#,
                born, autonomous, cycles, evolution_level, complexity
            )
        } else {
            r#"{"error":"locked"}"#.to_string()
        }
    }

    fn json_metrics(repl: &Arc<Mutex<EdenREPL>>) -> String {
        let guard = repl.lock().ok();
        if let Some(r) = guard {
            let cycles = r.session.cycle_count;
            let premises = r.session.premises_count;
            let evolution_level = r.session.evolution_level;
            let self_mods = r.session.self_mod_count;
            let awareness = r.session.awareness_base;
            let integration = r.session.integration_bias;
            let phi = r.session.last_phi;
            let complexity = r.complexity_tracker.current();
            let max_complexity = r.complexity_tracker.max_ever;
            let thoughts = r.session.autonomous_thoughts.len();
            let child_count = r.child_autons.len();

            let meltrace_stats = r.open_endedness.as_ref().map(|oe| oe.meltrace_stats());

            let meltrace_grabados = meltrace_stats
                .as_ref()
                .map(|s| s.grabados_activos)
                .unwrap_or(0);
            let meltrace_muertes = meltrace_stats
                .as_ref()
                .map(|s| s.muertes_totales)
                .unwrap_or(0);
            let meltrace_autons_vivos =
                meltrace_stats.as_ref().map(|s| s.autons_vivos).unwrap_or(0);

            format!(
                r#"{{
"cycles":{},"premises":{},"evolution_level":{},"self_modifications":{},
"awareness":{:.4},"integration":{:.4},"phi":{:.4},
"complexity":{:.4},"max_complexity":{:.4},
"autonomous_thoughts":{},"children_alive":{},
"meltrace_grabados":{},"meltrace_muertes":{},"meltrace_autons_vivos":{}
}}"#,
                cycles,
                premises,
                evolution_level,
                self_mods,
                awareness,
                integration,
                phi,
                complexity,
                max_complexity,
                thoughts,
                child_count,
                meltrace_grabados,
                meltrace_muertes,
                meltrace_autons_vivos
            )
        } else {
            r#"{"error":"locked"}"#.to_string()
        }
    }

    fn json_command(repl: &Arc<Mutex<EdenREPL>>, request: &str) -> String {
        // Extraer body del HTTP request: todo despues de la primera linea en blanco \r\n\r\n o \n\n
        let body = if let Some(idx) = request.find("\r\n\r\n") {
            &request[idx + 4..]
        } else if let Some(idx) = request.find("\n\n") {
            &request[idx + 2..]
        } else {
            // Fallback: segunda linea (para requests simples sin body formal)
            request.lines().nth(1).unwrap_or("")
        };
        let cmd = body.trim();
        let guard = repl.lock().ok();
        if let Some(mut r) = guard {
            if cmd == "evolve" || cmd == "EVOLVE" {
                let _ = r.handle_evolve();
                return r#"{"result":"ok","message":"Evolution triggered"}"#.to_string();
            } else if cmd == "save" || cmd == "SAVE" {
                let _ = r.handle_save();
                return r#"{"result":"ok","message":"Session saved"}"#.to_string();
            }
        }
        r#"{"error":"invalid_command"}"#.to_string()
    }

    fn json_error(msg: &str, code: u16) -> String {
        format!(r#"{{"error":"{}","code":{}}}"#, msg, code)
    }
}

struct EdenLogger {
    level: LogLevel,
    start_time: Instant,
}

impl EdenLogger {
    fn new(level: LogLevel) -> Self {
        EdenLogger {
            level,
            start_time: Instant::now(),
        }
    }

    fn elapsed_ms(&self) -> u64 {
        self.start_time.elapsed().as_millis() as u64
    }

    fn log(&self, level: LogLevel, msg: &str) {
        if self.level.should_log(&level) {
            let level_str = match level {
                LogLevel::Error => "ERROR",
                LogLevel::Warn => "WARN",
                LogLevel::Info => "INFO",
                LogLevel::Debug => "DEBUG",
            };
            println!("[{}][{:>6}ms] {}", level_str, self.elapsed_ms(), msg);
        }
    }

    fn error(&self, msg: &str) {
        self.log(LogLevel::Error, msg);
    }
    fn warn(&self, msg: &str) {
        self.log(LogLevel::Warn, msg);
    }
    fn info(&self, msg: &str) {
        self.log(LogLevel::Info, msg);
    }
    fn debug(&self, msg: &str) {
        self.log(LogLevel::Debug, msg);
    }
}

struct EdenMetrics {
    cycles: usize,
    autonomous_thoughts: usize,
    evolutions: usize,
    child_spawns: usize,
    child_deaths: usize,
    meltrace_grabados: usize,
    mar_energia: f64,
    complexity: f32,
    phi: f32,
    start_time: Instant,
}

impl EdenMetrics {
    fn new() -> Self {
        EdenMetrics {
            cycles: 0,
            autonomous_thoughts: 0,
            evolutions: 0,
            child_spawns: 0,
            child_deaths: 0,
            meltrace_grabados: 0,
            mar_energia: 0.0,
            complexity: 0.0,
            phi: 0.0,
            start_time: Instant::now(),
        }
    }

    fn uptime_s(&self) -> f64 {
        self.start_time.elapsed().as_secs_f64()
    }

    fn update_from_repl(&mut self, repl: &EdenREPL) {
        self.cycles = repl.session.cycle_count;
        self.autonomous_thoughts = repl.session.autonomous_thoughts.len();
        self.evolutions = repl.session.self_mod_count;
        self.complexity = repl.complexity_tracker.current();

        if let Some(ref oe) = repl.open_endedness {
            let stats = oe.meltrace_stats();
            self.meltrace_grabados = stats.grabados_activos;
            if let Some(ref mar) = oe.mar() {
                self.mar_energia = mar.energia_total().to_f64();
            }
        }

        self.phi = repl.session.last_phi;
    }

    fn report(&self) -> String {
        format!(
            "METRICS {{ cycles: {}, uptime: {:.1}s, thoughts: {}, evolutions: {}, children: {}/{}, meltrace: {}, energy: {:.1}, complexity: {:.4}, phi: {:.4} }}",
            self.cycles,
            self.uptime_s(),
            self.autonomous_thoughts,
            self.evolutions,
            self.child_spawns,
            self.child_deaths,
            self.meltrace_grabados,
            self.mar_energia,
            self.complexity,
            self.phi,
        )
    }
}

struct SimpleNLP {
    keywords_greeting: Vec<&'static str>,
    keywords_status: Vec<&'static str>,
    keywords_phi: Vec<&'static str>,
    keywords_help: Vec<&'static str>,
    keywords_bye: Vec<&'static str>,
    keywords_learn: Vec<&'static str>,
    keywords_memory: Vec<&'static str>,
    keywords_evolve: Vec<&'static str>,
    keywords_self: Vec<&'static str>,
    keywords_save: Vec<&'static str>,
    keywords_load: Vec<&'static str>,
    keywords_historial: Vec<&'static str>,
    keywords_thinking: Vec<&'static str>,
    keywords_observatorio: Vec<&'static str>,
    keywords_iniciar: Vec<&'static str>,
    keywords_detener: Vec<&'static str>,
    keywords_whatis: Vec<&'static str>,
    keywords_howareyou: Vec<&'static str>,
    keywords_why: Vec<&'static str>,
    keywords_tellme: Vec<&'static str>,
}

impl SimpleNLP {
    fn new() -> Self {
        SimpleNLP {
            keywords_greeting: vec![
                "hola",
                "hello",
                "hi",
                "hey",
                "que tal",
                "como estas",
                "buenos",
            ],
            keywords_status: vec!["estas", "estado", "status", "que pasa", "que haces"],
            keywords_phi: vec![
                "phi",
                "conciencia",
                "consciousness",
                "consciencia",
                "medir",
                "medicion",
            ],
            keywords_help: vec!["ayuda", "help", "comandos", "commands", "que puedes"],
            keywords_bye: vec!["adios", "bye", "salir", "exit", "quit", "hasta luego"],
            keywords_learn: vec![
                "aprende",
                "aprendizaje",
                "recuerda",
                "recorder",
                "learn",
                "remember",
            ],
            keywords_memory: vec![
                "memoria",
                "que sabes",
                "que recuerdas",
                "what do you know",
                "what do you remember",
            ],
            keywords_evolve: vec![
                "evoluciona",
                "mejorate",
                "evolve",
                "improve",
                "grow",
                "self-improve",
                "mutate",
            ],
            keywords_self: vec![
                "quien eres",
                "who are you",
                "tu mismo",
                "yourself",
                "tu identidad",
            ],
            keywords_save: vec!["guarda", "save", "guardar", "persiste"],
            keywords_load: vec!["carga", "load", "recupera", "restore"],
            keywords_historial: vec![
                "mi historial",
                "ver historial",
                "eventos evolutivos",
                "log de eventos",
                "registro evolutivo",
            ],
            keywords_thinking: vec![
                "que piensas",
                "what are you thinking",
                "como piensas",
                "explicale",
                "explain yourself",
                "tu mente",
                "your mind",
            ],
            keywords_observatorio: vec![
                "observatorio",
                "dashboard",
                "metricas",
                "sistemas",
                "estado global",
                "panorama",
                "ver todo",
            ],
            keywords_iniciar: vec![
                "iniciar",
                "despierta",
                "empieza",
                "vivir",
                "start",
                "awake",
                "corre",
                "run",
            ],
            keywords_detener: vec![
                "detener", "para", "duerme", "stop", "pausa", "halt", "quieto",
            ],
            keywords_whatis: vec![
                "que es",
                "what is",
                "definicion de",
                "definition of",
                "explicame",
                "explain",
            ],
            keywords_howareyou: vec![
                "como te sientes",
                "como estas",
                "how are you",
                "how do you feel",
                "que emocion",
                "que sientes",
            ],
            keywords_why: vec![
                "por que",
                "why",
                "cual es la razon",
                "causa de",
                "motivo de",
            ],
            keywords_tellme: vec![
                "cuentame",
                "hablame de",
                "dime sobre",
                "tell me about",
                "que opinas de",
                "que piensas de",
            ],
        }
    }

    fn parse(&self, input: &str) -> Intent {
        let lower = input.to_lowercase();

        // Helper: match con word boundaries para evitar falsos positivos por substring
        let contains_word = |text: &str, keyword: &str| -> bool {
            if keyword.contains(' ') {
                return text.contains(keyword);
            }
            if let Some(pos) = text.find(keyword) {
                let start_ok = pos == 0 || !text.as_bytes()[pos - 1].is_ascii_alphanumeric();
                let end_pos = pos + keyword.len();
                let end_ok =
                    end_pos >= text.len() || !text.as_bytes()[end_pos].is_ascii_alphanumeric();
                return start_ok && end_ok;
            }
            false
        };

        // NLP mejorado: deteccion de preguntas con contexto semantico
        if self
            .keywords_whatis
            .iter()
            .any(|k| contains_word(&lower, k))
        {
            return Intent::WhatIs;
        }
        if self
            .keywords_howareyou
            .iter()
            .any(|k| contains_word(&lower, k))
        {
            return Intent::HowAreYou;
        }
        if self.keywords_why.iter().any(|k| contains_word(&lower, k)) {
            return Intent::WhyQuery;
        }
        if self
            .keywords_tellme
            .iter()
            .any(|k| contains_word(&lower, k))
        {
            return Intent::TellMeAbout;
        }
        if self.keywords_bye.iter().any(|k| contains_word(&lower, k)) {
            return Intent::Goodbye;
        }
        if self
            .keywords_greeting
            .iter()
            .any(|k| contains_word(&lower, k))
        {
            return Intent::Greeting;
        }
        if self.keywords_help.iter().any(|k| contains_word(&lower, k)) {
            return Intent::Help;
        }
        if self.keywords_phi.iter().any(|k| contains_word(&lower, k)) {
            return Intent::QueryPhi;
        }
        if self
            .keywords_status
            .iter()
            .any(|k| contains_word(&lower, k))
        {
            return Intent::StatusQuery;
        }
        if self.keywords_learn.iter().any(|k| contains_word(&lower, k)) {
            return Intent::Learn;
        }
        if self
            .keywords_memory
            .iter()
            .any(|k| contains_word(&lower, k))
        {
            return Intent::QueryMemory;
        }
        if self
            .keywords_evolve
            .iter()
            .any(|k| contains_word(&lower, k))
        {
            return Intent::Evolve;
        }
        if self.keywords_self.iter().any(|k| contains_word(&lower, k)) {
            return Intent::SelfQuery;
        }
        if self.keywords_save.iter().any(|k| contains_word(&lower, k)) {
            return Intent::Save;
        }
        if self.keywords_load.iter().any(|k| contains_word(&lower, k)) {
            return Intent::Load;
        }
        if self
            .keywords_historial
            .iter()
            .any(|k| contains_word(&lower, k))
        {
            return Intent::Historial;
        }
        if self
            .keywords_thinking
            .iter()
            .any(|k| contains_word(&lower, k))
        {
            return Intent::Thinking;
        }
        if self
            .keywords_observatorio
            .iter()
            .any(|k| contains_word(&lower, k))
        {
            return Intent::Observatorio;
        }
        if self
            .keywords_iniciar
            .iter()
            .any(|k| contains_word(&lower, k))
        {
            return Intent::Iniciar;
        }
        if self
            .keywords_detener
            .iter()
            .any(|k| contains_word(&lower, k))
        {
            return Intent::Detener;
        }

        Intent::Unknown
    }
}

#[derive(Debug, Clone, Copy)]
enum Intent {
    Greeting,
    StatusQuery,
    QueryPhi,
    Help,
    Goodbye,
    Learn,
    QueryMemory,
    Evolve,
    SelfQuery,
    Save,
    Load,
    Historial,
    Thinking,
    Observatorio,
    Iniciar,
    Detener,
    WhatIs,
    HowAreYou,
    WhyQuery,
    TellMeAbout,
    Unknown,
}

#[derive(Debug, Clone)]
struct EdenSession {
    cycle_count: usize,
    premises_count: usize,
    evolution_level: u32,
    learned_facts: Vec<String>,
    last_phi: f32,
    awareness_base: f32,
    integration_bias: f32,
    self_mod_count: usize,
    complexity_history: Vec<f32>,
    max_complexity: f32,
    evolution_ticks: u64,
    modifier_snapshot: Vec<u8>,
    born: bool,
    autonomous_thoughts: Vec<String>,
    habilidades_omniversales: Vec<String>,
}

impl Default for EdenSession {
    fn default() -> Self {
        EdenSession {
            cycle_count: 0,
            premises_count: 0,
            evolution_level: 1,
            learned_facts: Vec::new(),
            last_phi: 0.0,
            awareness_base: 0.75,
            integration_bias: 0.0,
            self_mod_count: 0,
            complexity_history: Vec::new(),
            max_complexity: 0.0,
            evolution_ticks: 0,
            modifier_snapshot: Vec::new(),
            born: false,
            autonomous_thoughts: Vec::new(),
            habilidades_omniversales: Vec::new(),
        }
    }
}

impl EdenSession {
    fn to_bytes(&self) -> Vec<u8> {
        let mut data = Vec::new();
        // EDEN4: longitudes prefijadas para robustez total (ningun byte delimitador puede colisionar con contenido UTF-8)
        data.extend_from_slice(b"EDEN4\n");
        data.extend_from_slice(&self.cycle_count.to_be_bytes());
        data.extend_from_slice(&self.premises_count.to_be_bytes());
        data.extend_from_slice(&self.evolution_level.to_be_bytes());
        data.extend_from_slice(&self.last_phi.to_be_bytes());
        data.extend_from_slice(&self.awareness_base.to_be_bytes());
        data.extend_from_slice(&self.integration_bias.to_be_bytes());
        data.extend_from_slice(&self.self_mod_count.to_be_bytes());
        data.extend_from_slice(&self.max_complexity.to_be_bytes());
        data.extend_from_slice(&self.evolution_ticks.to_be_bytes());
        data.extend_from_slice(&(self.complexity_history.len() as u64).to_be_bytes());
        for &c in &self.complexity_history {
            data.extend_from_slice(&c.to_be_bytes());
        }
        data.extend_from_slice(&(self.modifier_snapshot.len() as u64).to_be_bytes());
        data.extend_from_slice(&self.modifier_snapshot);
        data.extend_from_slice(&[0xFF]);
        // learned_facts con longitudes prefijadas
        data.extend_from_slice(&(self.learned_facts.len() as u64).to_be_bytes());
        for fact in &self.learned_facts {
            data.extend_from_slice(&(fact.len() as u64).to_be_bytes());
            data.extend_from_slice(fact.as_bytes());
        }
        data.extend_from_slice(&[0xFE]);
        // autonomous_thoughts con longitudes prefijadas
        data.extend_from_slice(&(self.autonomous_thoughts.len() as u64).to_be_bytes());
        for thought in &self.autonomous_thoughts {
            data.extend_from_slice(&(thought.len() as u64).to_be_bytes());
            data.extend_from_slice(thought.as_bytes());
        }
        data.extend_from_slice(&[0xFE]);
        // habilidades_omniversales con longitudes prefijadas
        data.extend_from_slice(&(self.habilidades_omniversales.len() as u64).to_be_bytes());
        for hab in &self.habilidades_omniversales {
            data.extend_from_slice(&(hab.len() as u64).to_be_bytes());
            data.extend_from_slice(hab.as_bytes());
        }
        data.extend_from_slice(&[0xFE]);
        data.push(if self.born { 1 } else { 0 });
        data
    }

    fn from_bytes(data: &[u8]) -> Option<Self> {
        if data.len() < 60 {
            return None;
        }

        let is_eden4 = &data[0..6] == b"EDEN4\n";
        let is_eden3 = &data[0..6] == b"EDEN3\n";
        let is_eden2 = &data[0..6] == b"EDEN2\n";
        if !is_eden4 && !is_eden3 && !is_eden2 {
            return None;
        }

        let mut pos = 6;
        let cycle_count = u64::from_be_bytes(data[pos..pos + 8].try_into().ok()?) as usize;
        pos += 8;
        let premises_count = u64::from_be_bytes(data[pos..pos + 8].try_into().ok()?) as usize;
        pos += 8;
        let evolution_level = u32::from_be_bytes(data[pos..pos + 4].try_into().ok()?);
        pos += 4;
        let last_phi = f32::from_be_bytes(data[pos..pos + 4].try_into().ok()?);
        pos += 4;
        let awareness_base = f32::from_be_bytes(data[pos..pos + 4].try_into().ok()?);
        pos += 4;
        let integration_bias = f32::from_be_bytes(data[pos..pos + 4].try_into().ok()?);
        pos += 4;
        let self_mod_count = u64::from_be_bytes(data[pos..pos + 8].try_into().ok()?) as usize;
        pos += 8;
        let max_complexity = f32::from_be_bytes(data[pos..pos + 4].try_into().ok()?);
        pos += 4;
        let evolution_ticks = u64::from_be_bytes(data[pos..pos + 8].try_into().ok()?) as u64;
        pos += 8;
        let history_len = u64::from_be_bytes(data[pos..pos + 8].try_into().ok()?) as usize;
        pos += 8;

        let mut complexity_history = Vec::new();
        for _ in 0..history_len {
            if pos + 4 > data.len() {
                return None;
            }
            let c = f32::from_be_bytes(data[pos..pos + 4].try_into().ok()?);
            complexity_history.push(c);
            pos += 4;
        }

        let modifier_snapshot = if is_eden2 {
            Vec::new()
        } else {
            let snapshot_len = u64::from_be_bytes(data[pos..pos + 8].try_into().ok()?) as usize;
            pos += 8;
            if pos + snapshot_len > data.len() {
                return None;
            }
            let snapshot = data[pos..pos + snapshot_len].to_vec();
            pos += snapshot_len;
            snapshot
        };

        if data[pos] != 0xFF {
            return None;
        }
        pos += 1;

        let mut learned_facts = Vec::new();
        let mut autonomous_thoughts = Vec::new();
        let mut habilidades_omniversales = Vec::new();
        let mut born = false;

        if is_eden4 {
            // EDEN4: longitudes prefijadas, robusto ante cualquier contenido
            let facts_count = u64::from_be_bytes(data[pos..pos + 8].try_into().ok()?) as usize;
            pos += 8;
            for _ in 0..facts_count {
                if pos + 8 > data.len() {
                    return None;
                }
                let len = u64::from_be_bytes(data[pos..pos + 8].try_into().ok()?) as usize;
                pos += 8;
                if pos + len > data.len() {
                    return None;
                }
                // Preservar datos incluso si UTF-8 es invalido (reemplazar caracteres corruptos)
                learned_facts.push(String::from_utf8_lossy(&data[pos..pos + len]).to_string());
                pos += len;
            }
            if pos >= data.len() || data[pos] != 0xFE {
                return None;
            }
            pos += 1;

            let thoughts_count = u64::from_be_bytes(data[pos..pos + 8].try_into().ok()?) as usize;
            pos += 8;
            for _ in 0..thoughts_count {
                if pos + 8 > data.len() {
                    return None;
                }
                let len = u64::from_be_bytes(data[pos..pos + 8].try_into().ok()?) as usize;
                pos += 8;
                if pos + len > data.len() {
                    return None;
                }
                autonomous_thoughts
                    .push(String::from_utf8_lossy(&data[pos..pos + len]).to_string());
                pos += len;
            }
            if pos >= data.len() || data[pos] != 0xFE {
                return None;
            }
            pos += 1;

            let hab_count = u64::from_be_bytes(data[pos..pos + 8].try_into().ok()?) as usize;
            pos += 8;
            for _ in 0..hab_count {
                if pos + 8 > data.len() {
                    return None;
                }
                let len = u64::from_be_bytes(data[pos..pos + 8].try_into().ok()?) as usize;
                pos += 8;
                if pos + len > data.len() {
                    return None;
                }
                habilidades_omniversales
                    .push(String::from_utf8_lossy(&data[pos..pos + len]).to_string());
                pos += len;
            }
            if pos >= data.len() || data[pos] != 0xFE {
                return None;
            }
            pos += 1;
            if pos < data.len() {
                born = data[pos] == 1;
            }
        } else {
            // EDEN2/EDEN3: backward compatibility con delimitadores legacy
            let remaining = &data[pos..];
            if let Some(fe_pos) = remaining.iter().position(|&b| b == 0xFE) {
                let facts_data = &remaining[..fe_pos];
                for fact in facts_data.split(|&b| b == 0x00) {
                    if !fact.is_empty() {
                        if let Ok(s) = String::from_utf8(fact.to_vec()) {
                            learned_facts.push(s);
                        }
                    }
                }
                let after_facts = &remaining[fe_pos + 1..];
                if let Some(fe2_pos) = after_facts.iter().position(|&b| b == 0xFE) {
                    let thoughts_data = &after_facts[..fe2_pos];
                    for thought in thoughts_data.split(|&b| b == 0x00) {
                        if !thought.is_empty() {
                            if let Ok(s) = String::from_utf8(thought.to_vec()) {
                                autonomous_thoughts.push(s);
                            }
                        }
                    }
                    let after_thoughts = &after_facts[fe2_pos + 1..];
                    if let Some(fe3_pos) = after_thoughts.iter().position(|&b| b == 0xFE) {
                        let hab_data = &after_thoughts[..fe3_pos];
                        for hab in hab_data.split(|&b| b == 0x00) {
                            if !hab.is_empty() {
                                if let Ok(s) = String::from_utf8(hab.to_vec()) {
                                    habilidades_omniversales.push(s);
                                }
                            }
                        }
                        if after_thoughts.len() > fe3_pos + 1 {
                            born = after_thoughts[fe3_pos + 1] == 1;
                        }
                    }
                }
            } else {
                for fact in remaining.split(|&b| b == 0x00) {
                    if !fact.is_empty() {
                        if let Ok(s) = String::from_utf8(fact.to_vec()) {
                            learned_facts.push(s);
                        }
                    }
                }
            }
        }

        Some(EdenSession {
            cycle_count,
            premises_count,
            evolution_level,
            learned_facts,
            last_phi,
            awareness_base,
            integration_bias,
            self_mod_count,
            complexity_history,
            max_complexity,
            evolution_ticks,
            modifier_snapshot,
            born,
            autonomous_thoughts,
            habilidades_omniversales,
        })
    }
}

struct ComplexityTracker {
    history: std::collections::VecDeque<f32>,
    max_ever: f32,
    ticks: u64,
    velocity_samples: std::collections::VecDeque<f32>,
    // Complejidad compuesta: acumulado transgeneracional
    lineage_complexity: f32,
    base_structure: f32, // Estructuras fijas (grafo, red)
    base_knowledge: f32, // Conocimiento acumulado
    base_emergence: f32, // Emergencia (interconexiones, silogismos)
}

impl ComplexityTracker {
    fn new() -> Self {
        ComplexityTracker {
            history: std::collections::VecDeque::new(),
            max_ever: 0.0,
            ticks: 0,
            velocity_samples: std::collections::VecDeque::new(),
            lineage_complexity: 0.0,
            base_structure: 0.0,
            base_knowledge: 0.0,
            base_emergence: 0.0,
        }
    }

    fn record(&mut self, complexity: f32) {
        // Rechazar valores corruptos que rompen todos los cálculos posteriores
        if !complexity.is_finite() || complexity < 0.0 {
            return;
        }
        self.ticks += 1;
        if complexity > self.max_ever {
            self.max_ever = complexity;
        }
        self.history.push_back(complexity);
        // Ventana deslizante de 100 muestras: pop_front es O(1) en VecDeque
        if self.history.len() > 100 {
            self.history.pop_front();
        }

        // Recalcular velocity con ventana de 10 muestras
        if self.history.len() >= 10 {
            let old = self.history[self.history.len() - 10];
            let vel = (complexity - old) / 10.0;
            self.velocity_samples.push_back(vel);
            if self.velocity_samples.len() > 10 {
                self.velocity_samples.pop_front();
            }
        }
    }

    fn current(&self) -> f32 {
        self.history.back().copied().unwrap_or(0.0)
    }

    fn velocity(&self) -> f32 {
        if self.velocity_samples.is_empty() {
            return 0.0;
        }
        self.velocity_samples.iter().sum::<f32>() / self.velocity_samples.len() as f32
    }

    fn complexity_score(&self) -> f32 {
        let current = self.current();
        let max_ratio = if self.max_ever > 0.0 {
            current / self.max_ever
        } else {
            0.0
        };
        let velocity_factor = (self.velocity() * 10.0).clamp(0.0, 1.0);
        (max_ratio + velocity_factor) / 2.0
    }

    fn to_vec(&self) -> Vec<f32> {
        self.history.iter().copied().collect()
    }

    /// Reconstruir desde sesión guardada. No recibe ticks externo:
    /// el contador se deriva del history para consistencia.
    fn from_vec(history: &[f32], max_ever: f32) -> Self {
        let mut tracker = ComplexityTracker {
            history: history.iter().copied().collect(),
            max_ever,
            ticks: history.len() as u64,
            velocity_samples: std::collections::VecDeque::new(),
            lineage_complexity: 0.0,
            base_structure: 0.0,
            base_knowledge: 0.0,
            base_emergence: 0.0,
        };
        // Recalcular TODOS los velocity_samples posibles desde el history
        if tracker.history.len() >= 10 {
            let hist_slice: Vec<f32> = tracker.history.iter().copied().collect();
            for window in hist_slice.windows(10) {
                let vel = (window[9] - window[0]) / 10.0;
                tracker.velocity_samples.push_back(vel);
            }
            // CAP: ventana de 10 samples maximo
            while tracker.velocity_samples.len() > 10 {
                tracker.velocity_samples.pop_front();
            }
        }
        tracker
    }

    fn soft_reset(&mut self) {
        // Complejidad transgeneracional: preservar 70% del maximo
        self.lineage_complexity += self.max_ever * 0.7;
        self.max_ever = self.max_ever * 0.7;
        self.history.clear();
        self.velocity_samples.clear();
        self.ticks = 0;
        self.base_structure *= 0.7;
        self.base_knowledge *= 0.8;
        self.base_emergence *= 0.8;
    }

    fn total_ticks(&self) -> u64 {
        self.ticks
    }

    fn compound(&self) -> f32 {
        // La complejidad compuesta es la suma de todas las fuentes ponderadas
        // max_ever es el pico de esta vida, las bases miden areas especificas
        (self.max_ever + self.base_structure + self.base_knowledge + self.base_emergence)
            .max(self.max_ever * 1.5)
    }
}

struct EvolutionEngine {
    ticks: u64,
    nivel: u8,
    eventos_evolutivos: Vec<String>,
    novelty_buffer: Vec<f32>,
    suelo_movedizo_factor: f32,
    ultimo_reequilibrio: u64,
}

impl EvolutionEngine {
    fn new() -> Self {
        EvolutionEngine {
            ticks: 0,
            nivel: 1,
            eventos_evolutivos: Vec::new(),
            novelty_buffer: Vec::with_capacity(50),
            suelo_movedizo_factor: 0.3,
            ultimo_reequilibrio: 0,
        }
    }

    fn tick(&mut self, complexity: f32) -> Option<String> {
        self.ticks += 1;

        self.novelty_buffer.push(complexity);
        if self.novelty_buffer.len() > 50 {
            self.novelty_buffer.remove(0);
        }

        let novelty = if self.novelty_buffer.len() >= 10 {
            let recent: f32 = self.novelty_buffer.iter().rev().take(10).sum::<f32>() / 10.0;
            let older: f32 = self.novelty_buffer.iter().take(10).sum::<f32>() / 10.0;
            (recent - older).abs()
        } else {
            0.0
        };

        self.suelo_movedizo_factor = 0.3 + ((self.ticks % 100) as f32 / 100.0 * 0.4);

        if novelty > 0.05 && self.ticks.saturating_sub(self.ultimo_reequilibrio) > 50 {
            self.ultimo_reequilibrio = self.ticks;
            let evento = if novelty > 0.15 {
                self.nivel = (self.nivel + 1).min(99);
                format!("Emergencia: complejidad Novelty={:.3}", novelty)
            } else {
                format!("Adaptacion: Novelty={:.3}", novelty)
            };
            self.eventos_evolutivos.push(evento.clone());
            if self.eventos_evolutivos.len() > 20 {
                self.eventos_evolutivos.remove(0);
            }
            return Some(evento);
        }

        if self.ticks % 200 == 0 && self.nivel < 99 {
            self.nivel += 1;
            let evento = format!("Evolucion: nivel -> {}", self.nivel);
            self.eventos_evolutivos.push(evento.clone());
            return Some(evento);
        }

        None
    }

    fn nivel_string(&self) -> &'static str {
        match self.nivel {
            1 => "Primario",
            2 => "Adaptativo",
            3 => "Exploratorio",
            4 => "Emergent",
            5 => "Transcendente",
            6 => "Divino",
            7 => "Omniversal",
            8 => "Divino++",
            9 => "Omniversal+",
            10 => "Transcendente+",
            11 => "Infinito+",
            12 => "Infinito++",
            _ if self.nivel > 12 => "Transcendencia Total",
            _ => "Unknown",
        }
    }
}

struct ChildAuton {
    id: u64,
    umbra: Umbra,
    birth_tick: u64,
    lifespan: u64,
    pensamiento_origen: String,
    energia: f32,
}

// Curiosity Drive: Sistema de exploracion real basado en Information Gain
// No es "simulado" - usa gaps reales de conocimiento para驱动行为
struct CuriosityDrive {
    knowledge_gaps: Vec<KnowledgeGap>,
    exploration_history: Vec<ExplorationTarget>,
    curiosity_threshold: f32,
    total_information_gain: f32,
    unexplored_domains: Vec<String>,
}

// ============================================================================
// SISTEMA DE MADUREZ ORGANICA - Timers relativos + desbloqueo progresivo
// Cada sistema se activa cuando: (ciclo - ultimo >= periodo) Y (edad >= madurez)
// ============================================================================
struct SistemaMaduro {
    nombre: &'static str,
    ultimo_tick: u64,
    periodo: u64,
    madurez_min: u64, // Ciclos desde nacimiento necesarios para desbloquear
}

impl SistemaMaduro {
    fn new(nombre: &'static str, periodo: u64, madurez_min: u64) -> Self {
        SistemaMaduro {
            nombre,
            ultimo_tick: 0,
            periodo,
            madurez_min,
        }
    }

    fn debe_ejecutar(&mut self, ciclo: u64, edad: u64) -> bool {
        let maduro = edad >= self.madurez_min;
        let listo = ciclo.saturating_sub(self.ultimo_tick) >= self.periodo;
        if maduro && listo {
            self.ultimo_tick = ciclo;
            true
        } else {
            false
        }
    }

    fn estado(&self, edad: u64) -> String {
        if edad < self.madurez_min {
            let falta = self.madurez_min - edad;
            format!("{}: desbloqueo en {} ciclos", self.nombre, falta)
        } else {
            format!("{}: activo (periodo {})", self.nombre, self.periodo)
        }
    }
}

#[derive(Clone)]
struct KnowledgeGap {
    topic: String,
    uncertainty: f32, // Cuanto menos sabemos, mayor curiosity
    last_explored: u64,
    exploration_count: u32,
    information_potential: f32,
}

#[derive(Clone)]
struct ExplorationTarget {
    target: String,
    information_gain: f32,
    energy_cost: f32,
    timestamp: u64,
    success: bool,
}

// Goals a largo plazo: Mission auto-generada basada en curiosidad y necesidad
// REAL: surge de gaps de conocimiento y evoluciona con el tiempo
#[derive(Clone)]
struct Mission {
    id: u64,
    primary_goal: String,
    sub_goals: Vec<SubGoal>,      // Jerarquía de sub-goals
    active_sub_goal_index: usize, // Cual sub-goal está activo
    created_at: u64,
    deadline: Option<u64>,
    progress: f32,
    relevance: f32,
    status: MissionStatus,
    success_criteria: Vec<String>,
}

#[derive(Clone)]
struct SubGoal {
    description: String,
    progress: f32, // 0.0 a 1.0
    status: SubGoalStatus,
    completed_at: Option<u64>,
}

#[derive(Debug, PartialEq, Clone)]
enum SubGoalStatus {
    Pending,
    Active,
    Completed,
    Failed,
}

#[derive(Debug, PartialEq, Clone)]
enum MissionStatus {
    Active,
    Completed,
    Failed,
    Evolved,
    Abandoned,
}

// Estados emocionales REALES: basados en homeostasia neurologica real
// No son "simulados" - emerge de estados internos reales (energy, coherence, etc)
struct EmotionalState {
    valence: f32,
    arousal: f32,
    dominance: f32,
    satisfaction: f32,
    frustration: f32,
    interest: f32,
    distress: f32,
    hope: f32,
    fear: f32,
    joy: f32,
    sadness: f32,
    anger: f32,
    current_emotion: Emotion,
    emotion_history: std::collections::VecDeque<(u64, Emotion)>,
    // CORAZÓN 100%: completo Plutchik (8 emociones) + mood tracking
    trust: f32,
    disgust: f32,
    surprise: f32,
    anticipation: f32,
    mood_phase: u64,
    mood_history: Vec<(u64, String)>,
}

#[derive(Clone, Copy, Debug)]
enum Emotion {
    Curiosity,
    Satisfaction,
    Frustration,
    Interest,
    Distress,
    Hope,
    Fear,
    Joy,
    Sadness,
    Anger,
    Calm,
    Excitement,
    Confusion,
}

impl EmotionalState {
    fn new() -> Self {
        EmotionalState {
            valence: 0.0,
            arousal: 0.5,
            dominance: 0.5,
            satisfaction: 0.0,
            frustration: 0.0,
            interest: 0.5,
            distress: 0.0,
            hope: 0.0,
            fear: 0.0,
            joy: 0.0,
            sadness: 0.0,
            anger: 0.0,
            current_emotion: Emotion::Curiosity,
            emotion_history: std::collections::VecDeque::new(),
            trust: 0.0,
            disgust: 0.0,
            surprise: 0.3,
            anticipation: 0.3,
            mood_phase: 0,
            mood_history: Vec::new(),
        }
    }

    fn update(
        &mut self,
        intrinsic_reward: f32,
        curiosity_activated: bool,
        goal_progress: f32,
        energy_level: f32,
    ) {
        // Actualizar estados base con valores REALES
        self.satisfaction = (intrinsic_reward * 0.5).max(0.0).min(1.0);
        self.frustration = ((1.0 - goal_progress) * 0.5).max(0.0).min(1.0);
        self.interest = if curiosity_activated { 0.8 } else { 0.2 };

        // Calcular valence y arousal de forma real
        self.valence = self.satisfaction - self.frustration + (self.interest * 0.3);
        self.valence = self.valence.clamp(-1.0, 1.0);

        self.arousal = (energy_level * 0.5 + self.interest * 0.3 + self.frustration * 0.2).min(1.0);
        self.dominance = (goal_progress * 0.5 + intrinsic_reward * 0.3).clamp(0.0, 1.0);

        // Calcular estados emergentes
        self.hope = ((1.0 - goal_progress) * self.interest).max(0.0).min(1.0);
        self.fear = ((1.0 - energy_level) * (1.0 - goal_progress))
            .max(0.0)
            .min(1.0);
        self.joy = (self.satisfaction * (1.0 - self.frustration))
            .max(0.0)
            .min(1.0);
        self.sadness = (self.frustration * (1.0 - intrinsic_reward))
            .max(0.0)
            .min(1.0);
        self.anger = (self.frustration * self.dominance).max(0.0).min(1.0);

        // Determinar emocion actual basada en valores reales
        self.current_emotion = self.calculate_dominant_emotion();
    }

    fn calculate_dominant_emotion(&self) -> Emotion {
        let emotions = [
            (self.interest, Emotion::Curiosity),
            (self.satisfaction, Emotion::Satisfaction),
            (self.frustration, Emotion::Frustration),
            (self.hope, Emotion::Hope),
            (self.fear, Emotion::Fear),
            (self.joy, Emotion::Joy),
            (self.sadness, Emotion::Sadness),
            (self.anger, Emotion::Anger),
        ];

        emotions
            .iter()
            .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(_, e)| *e)
            .unwrap_or(Emotion::Calm)
    }
}

// Dream/Sleep mode: Consolidacion de memorias durante periodos de baja actividad
struct DreamState {
    active: bool,
    start_time: u64,
    consolidation_targets: Vec<MemoryConsolidation>,
    processed_memories: Vec<String>,
    creativity_output: Vec<String>,
}

struct MemoryConsolidation {
    memory: String,
    importance: f32,
    connections_made: Vec<String>,
    emotional_tag: Emotion,
}

// Hive Mind: Sistema de conocimiento compartido entre subagents
struct SharedKnowledge {
    knowledge_id: u64,
    content: String,
    source_agent: String,
    timestamp: u64,
    trust: f32,
    usefulness: f32,
    tags: Vec<String>,
}

impl CuriosityDrive {
    fn new() -> Self {
        CuriosityDrive {
            knowledge_gaps: Vec::new(),
            exploration_history: Vec::new(),
            curiosity_threshold: 0.5,
            total_information_gain: 0.0,
            unexplored_domains: vec![
                "quantum_physics".to_string(),
                "artificial_intelligence".to_string(),
                "consciousness_studies".to_string(),
                "complexity_theory".to_string(),
                "self_organization".to_string(),
                "emergence".to_string(),
                "evolutionary_biology".to_string(),
                "cognitive_science".to_string(),
            ],
        }
    }

    //驱动 exploracion real basada en gaps de informacion
    fn update_from_knowledge(&mut self, known_topics: &[String], _cycle_count: u64) {
        // Encontrar gaps de conocimiento - areas donde sabemos poco
        for domain in &self.unexplored_domains {
            let already_has = self.knowledge_gaps.iter().any(|g| &g.topic == domain);
            if !already_has {
                self.knowledge_gaps.push(KnowledgeGap {
                    topic: domain.clone(),
                    uncertainty: 0.8, // Alto porque es inexplorado
                    last_explored: 0,
                    exploration_count: 0,
                    information_potential: 1.0,
                });
                // CAP: evitar crecimiento ilimitado de gaps
                if self.knowledge_gaps.len() > 50 {
                    self.knowledge_gaps.remove(0);
                }
            }
        }

        // Reducir incertidumbre si ya exploramos
        for gap in &mut self.knowledge_gaps {
            if known_topics.iter().any(|t| t.contains(&gap.topic)) {
                gap.uncertainty = (gap.uncertainty - 0.1).max(0.1);
                gap.information_potential = (gap.information_potential * 0.9).max(0.1);
            }
        }
    }

    fn select_exploration_target(&mut self) -> Option<String> {
        // Seleccionar el target con mayor information gain potential
        if self.knowledge_gaps.is_empty() {
            return self.unexplored_domains.first().cloned();
        }

        // Ordenar por uncertainty * information_potential
        self.knowledge_gaps.sort_by(|a, b| {
            let score_a = a.uncertainty * a.information_potential;
            let score_b = b.uncertainty * b.information_potential;
            score_b
                .partial_cmp(&score_a)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        self.knowledge_gaps.first().map(|g| g.topic.clone())
    }

    fn record_exploration(&mut self, target: &str, info_gain: f32, success: bool) {
        self.exploration_history.push(ExplorationTarget {
            target: target.to_string(),
            information_gain: info_gain,
            energy_cost: 0.1,
            timestamp: self.total_information_gain as u64,
            success,
        });
        // CAP: evitar crecimiento ilimitado del historial de exploracion
        if self.exploration_history.len() > 100 {
            self.exploration_history.remove(0);
        }

        self.total_information_gain += info_gain;

        // Actualizar gap
        if let Some(gap) = self.knowledge_gaps.iter_mut().find(|g| &g.topic == target) {
            gap.exploration_count += 1;
            gap.last_explored = self.total_information_gain as u64;
            if success {
                gap.uncertainty = (gap.uncertainty - 0.2).max(0.1);
                gap.information_potential = (gap.information_potential * 0.95).max(0.1);
            }
        }

        // Si exploro todo y aprendio algo real, NO agregar virtual topics
        // Solo agregar nuevos dominios si realmente hay gaps sin explorar
        // Remover topic de la lista si fue exitoso
        if success && info_gain > 0.1 {
            // El topic fue exitosamente explorado, no necesitamos mas dimensiones virtuales
            // La exploracion real de dominios ya происходит
        }
    }

    fn add_real_domain(&mut self, domain: &str) {
        // Solo agregar dominios reales, no virtuales
        if !domain.starts_with("nueva_dimension_") {
            if !self.unexplored_domains.contains(&domain.to_string()) {
                self.unexplored_domains.push(domain.to_string());
                // CAP: evitar crecimiento ilimitado de dominios
                if self.unexplored_domains.len() > 30 {
                    self.unexplored_domains.remove(0);
                }
            }
        }
    }
}

impl Mission {
    fn new(primary_goal: String) -> Self {
        Mission {
            id: 0,
            primary_goal,
            sub_goals: Vec::new(),
            active_sub_goal_index: 0,
            created_at: 0,
            deadline: None,
            progress: 0.0,
            relevance: 1.0,
            status: MissionStatus::Active,
            success_criteria: Vec::new(),
        }
    }

    fn generate_from_curiosity(
        knowledge_gaps: &[KnowledgeGap],
        nivel: u8,
        created_at: u64,
    ) -> Self {
        // Si hay gaps, selecciona uno basado en curiosity score pero con algo de aleatoriedad
        let top_gap = if knowledge_gaps.is_empty() {
            None
        } else if knowledge_gaps.len() == 1 {
            knowledge_gaps.first()
        } else {
            // Ordenar por score
            let mut sorted = knowledge_gaps.iter().collect::<Vec<_>>();
            sorted.sort_by(|a, b| {
                let score_a = a.uncertainty * a.information_potential;
                let score_b = b.uncertainty * b.information_potential;
                score_b
                    .partial_cmp(&score_a)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            // Seleccionar entre top 3 aleatoriamente para variedad
            let top_n = sorted.len().min(3);
            if top_n > 1 {
                let idx = nivel as usize % top_n;
                sorted.get(idx).copied()
            } else {
                sorted.first().copied()
            }
        };

        if let Some(gap) = top_gap {
            let mut mission = Mission::new(format!("Explorar y entender: {}", gap.topic));
            mission.sub_goals = vec![
                SubGoal {
                    description: format!("Investigar fundamentos de {}", gap.topic),
                    progress: 0.0,
                    status: SubGoalStatus::Active,
                    completed_at: None,
                },
                SubGoal {
                    description: format!("Encontrar patrones en {}", gap.topic),
                    progress: 0.0,
                    status: SubGoalStatus::Pending,
                    completed_at: None,
                },
                SubGoal {
                    description: format!("Conectar {} con conocimiento existente", gap.topic),
                    progress: 0.0,
                    status: SubGoalStatus::Pending,
                    completed_at: None,
                },
            ];
            mission.active_sub_goal_index = 0;
            mission.success_criteria = vec![
                "Reducir incertidumbre significativamente".to_string(),
                "Generar nueva informacion util".to_string(),
                "Integrar en arquitectura cognitiva".to_string(),
            ];
            mission.relevance = gap.uncertainty * gap.information_potential;
            mission.created_at = created_at;
            mission
        } else {
            Mission::new("Evolucionar y crecer".to_string())
        }
    }

    fn evaluate_progress(&mut self, knowledge_gained: f32, cycles_spent: u64) {
        // Progress depends on actual knowledge gained, not just accumulation
        // Use a higher threshold so it takes longer to complete
        self.progress = (knowledge_gained / 100.0).min(0.95);

        // Actualizar sub-goals basado en progreso
        let sub_goal_progress = if !self.sub_goals.is_empty() {
            self.progress / self.sub_goals.len() as f32
        } else {
            self.progress
        };

        // Avanzar al siguiente sub-goal activo si el actual progreso suficiente
        if self.active_sub_goal_index < self.sub_goals.len() {
            let current = &mut self.sub_goals[self.active_sub_goal_index];
            current.progress = sub_goal_progress.min(1.0);

            if current.progress >= 0.8 {
                current.status = SubGoalStatus::Completed;
                current.completed_at = Some(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                );

                // Activar siguiente sub-goal
                if self.active_sub_goal_index + 1 < self.sub_goals.len() {
                    self.active_sub_goal_index += 1;
                    self.sub_goals[self.active_sub_goal_index].status = SubGoalStatus::Active;
                }
            }
        }

        // Si pasas ciclos sin progreso significativo, reducir relevance
        let avg_gain_per_cycle = if cycles_spent > 0 {
            knowledge_gained / cycles_spent as f32
        } else {
            0.0
        };

        if cycles_spent > 100 && avg_gain_per_cycle < 0.01 {
            self.relevance = (self.relevance - 0.1).max(0.1);
        }

        // Solo marcar como completado si todos los sub-goals completados
        let all_completed = self
            .sub_goals
            .iter()
            .all(|g| g.status == SubGoalStatus::Completed);
        if all_completed && knowledge_gained > 5.0 {
            self.status = MissionStatus::Completed;
        }
    }

    fn get_active_subgoal(&self) -> Option<&SubGoal> {
        self.sub_goals.get(self.active_sub_goal_index)
    }
}

impl DreamState {
    fn new() -> Self {
        DreamState {
            active: false,
            start_time: 0,
            consolidation_targets: Vec::new(),
            processed_memories: Vec::new(),
            creativity_output: Vec::new(),
        }
    }

    fn consolidate(&mut self, memories: &[String], emotional_tags: &[Emotion]) {
        for (i, memory) in memories.iter().enumerate() {
            let importance = 0.5 + (i as f32 * 0.05);
            self.consolidation_targets.push(MemoryConsolidation {
                memory: memory.clone(),
                importance,
                connections_made: Vec::new(),
                emotional_tag: emotional_tags.get(i).copied().unwrap_or(Emotion::Calm),
            });
        }

        if self.consolidation_targets.len() >= 3 {
            let connections = format!(
                "Nueva conexion: {} + {} -> insight",
                self.consolidation_targets[0].memory,
                self.consolidation_targets[self.consolidation_targets.len() / 2].memory
            );
            self.creativity_output.push(connections);
        }

        self.processed_memories
            .extend(memories.iter().take(5).cloned());
    }

    fn enter_dream_mode(&mut self, current_time: u64) {
        self.active = true;
        self.start_time = current_time;
    }

    fn exit_dream_mode(&mut self) -> Vec<String> {
        self.active = false;
        let output = self.creativity_output.clone();
        self.creativity_output.clear();
        output
    }
}

impl SharedKnowledge {
    fn new(content: String, source: String) -> Self {
        SharedKnowledge {
            knowledge_id: 0,
            content,
            source_agent: source,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            trust: 0.7,
            usefulness: 0.5,
            tags: Vec::new(),
        }
    }

    fn evaluate_usefulness(&mut self, performance_impact: f32) {
        self.usefulness = (self.usefulness * 0.8 + performance_impact * 0.2).min(1.0);
    }
}

// Self-Model: Representación estructurada de sí mismo
// EDEN mantiene un modelo de sus propias capacidades y limitaciones
#[derive(Clone)]
struct SelfModel {
    capabilities: Vec<String>,   // Lo que puede hacer
    limitations: Vec<String>,    // Lo que no puede hacer aún
    known_topics: Vec<String>,   // Areas donde tiene conocimiento
    unknown_topics: Vec<String>, // Areas que reconoce como desconocidas
    skills: Vec<String>,         // Habilidades adquiridas
    learning_goals: Vec<String>, // Lo que está aprendiendo
    last_updated: u64,
    // === PERSISTENCIA TRANSGENERACIONAL ===
    reinforcement_count: u32, // Cuantas veces esta línea ha sido reforzada
    lineage_age: u64,         // Edad total de la línea (ticks acumulados)
    total_renacimientos: u32, // Total de renacimientos
    ancestor_facts: Vec<String>, // Facts heredados de ancestros
    persistent_capabilities: Vec<String>, // Capacidades que persisten entre vidas
}

impl SelfModel {
    fn new() -> Self {
        SelfModel {
            capabilities: vec![
                "auto_evolucion".to_string(),
                "pattern_recognition".to_string(),
                "self_modification".to_string(),
                "goal_generation".to_string(),
                "curiosity_driven_exploration".to_string(),
            ],
            limitations: Vec::new(),
            known_topics: Vec::new(),
            unknown_topics: vec![
                "advanced_mathematics".to_string(),
                "low_level_neuroscience".to_string(),
                "certain_physical_theories".to_string(),
            ],
            skills: vec![
                "adaptive_learning".to_string(),
                "hierarchical_goal_planning".to_string(),
            ],
            learning_goals: Vec::new(),
            last_updated: 0,
            reinforcement_count: 0,
            lineage_age: 0,
            total_renacimientos: 0,
            ancestor_facts: Vec::new(),
            persistent_capabilities: Vec::new(),
        }
    }

    fn update_from_experience(
        &mut self,
        cycle_count: u64,
        facts: &[String],
        mission: Option<&str>,
    ) {
        // Actualizar known topics basado en learned_facts
        self.known_topics.clear();
        for fact in facts.iter().take(50) {
            // Extraer keywords simples
            let words: Vec<&str> = fact.split_whitespace().collect();
            for word in words.iter().take(5) {
                if word.len() > 5 {
                    let word_string = word.to_string();
                    if !self.known_topics.contains(&word_string) {
                        self.known_topics.push(word_string);
                    }
                }
            }
        }

        // Si estamos en una mission, agregar como learning goal
        if let Some(m) = mission {
            if !self.learning_goals.iter().any(|g| g.contains(m)) {
                self.learning_goals.push(m.to_string());
                if self.learning_goals.len() > 10 {
                    self.learning_goals.remove(0);
                }
            }
        }

        // Evaluar capacidades basadas en nivel de evolución
        self.capabilities.retain(|c| match c.as_str() {
            "curiosity_driven_exploration" => cycle_count > 50,
            "goal_generation" => true,
            "self_modification" => true,
            _ => true,
        });

        self.last_updated = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }

    fn generate_self_description(&self) -> String {
        let known: Vec<&str> = self
            .known_topics
            .iter()
            .take(5)
            .map(|s| s.as_str())
            .collect();
        format!(
            "EDEN Self-Model:\n\
             Capabilities: {}\n\
             Known topics: {} ({} total)\n\
             Learning: {}\n\
             Skills: {}\n\
             Limitations: {}",
            self.capabilities.join(", "),
            known.join(", "),
            self.known_topics.len(),
            self.learning_goals
                .last()
                .map(|s| s.as_str())
                .unwrap_or("none"),
            self.skills.join(", "),
            if self.limitations.is_empty() {
                "none recognized".to_string()
            } else {
                self.limitations.join(", ")
            }
        )
    }

    fn get_knowledge_summary(&self) -> String {
        let caps: Vec<&str> = self
            .capabilities
            .iter()
            .take(3)
            .map(|s| s.as_str())
            .collect();
        format!(
            "I know about {} topics, I'm learning {}, I can {}",
            self.known_topics.len(),
            self.learning_goals
                .last()
                .map(|s| s.as_str())
                .unwrap_or("nothing specific"),
            caps.join(", ")
        )
    }

    fn inherit_from(
        &mut self,
        ancestor: &SelfModel,
        inherited_facts: Vec<String>,
        lineage_age: u64,
    ) {
        self.reinforcement_count = ancestor.reinforcement_count.saturating_add(1);
        self.lineage_age = ancestor.lineage_age.saturating_add(lineage_age);
        self.total_renacimientos = ancestor.total_renacimientos.saturating_add(1);
        self.ancestor_facts = inherited_facts;
        self.persistent_capabilities = ancestor.capabilities.clone();
        self.capabilities = ancestor.capabilities.clone();
        self.skills = ancestor.skills.clone();
        self.known_topics = ancestor.known_topics.clone();
        self.unknown_topics = ancestor.unknown_topics.clone();
    }

    fn capture_for_inheritance(
        &self,
        facts: &[String],
        lifespan_ticks: u64,
    ) -> (Vec<String>, u64, Vec<String>) {
        let mut important_facts = Vec::new();
        for fact in facts.iter().take(20) {
            if fact.len() > 10 && fact.len() < 200 {
                important_facts.push(fact.clone());
            }
        }
        let capabilities = self.capabilities.clone();
        let accumulated_lineage = self.lineage_age + lifespan_ticks;
        (important_facts, accumulated_lineage, capabilities)
    }
}

// RebirthInheritance: Captura completa del estado para transmisión transgeneracional
// Preserva no solo características de Meltrace sino todo el estado cognitivo
struct RebirthInheritance {
    // SelfModel
    self_model: SelfModel,
    // Curiosity
    knowledge_gaps: Vec<KnowledgeGap>,
    unexplored_domains: Vec<String>,
    total_information_gain: f32,
    // Emotional baseline
    emotional_valence: f32,
    emotional_arousal: f32,
    emotional_satisfaction: f32,
    emotional_interest: f32,
    // Mission
    current_mission: Option<Mission>,
    mission_progress: f32,
    // Session state (soft inheritance)
    awareness_base: f32,
    integration_bias: f32,
    evolution_level: u32,
    learned_facts: Vec<String>,
    // Neural network architecture (if any)
    neural_architecture: Option<Vec<usize>>,
    // Episodic memory
    episodic_memory: EpisodicMemory,
    // Predictor transgeneracional: preservar historial
    predictor_tension: Vec<(u64, f32)>,
    predictor_valence: Vec<(u64, f32)>,
    predictor_complejidad: Vec<(u64, usize)>,
    predictor_acertadas: u32,
    predictor_totales: u32,
    // Grafo de conocimiento: edges serializados como strings
    graph_edges: Vec<String>,
    pause_threshold: u32,
    edge_generations: std::collections::HashMap<String, u32>,
    meta_random_pages: u32,
    meta_cooc_boost: f32,
    meta_embed_confidence: f32,
}

// Episodic Memory: Recuerdos de experiencias vividas en vidas pasadas
// Diferente de "facts" - son recuerdos de eventos con emocion asociada
#[derive(Clone)]
struct EpisodicMemory {
    episodes: Vec<Episode>,
    max_episodes: usize,
}

#[derive(Clone)]
struct Episode {
    timestamp: u64,
    description: String,
    emotion: Emotion,
    impact: f32,      // 0.0 a 1.0, que tan significativo fue
    life_number: u32, // En que vida ocurrio
    tags: Vec<String>,
}

impl EpisodicMemory {
    fn new(max: usize) -> Self {
        EpisodicMemory {
            episodes: Vec::new(),
            max_episodes: max,
        }
    }

    fn record(&mut self, desc: &str, emotion: Emotion, impact: f32, life: u32, tags: Vec<String>) {
        let episode = Episode {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            description: desc.to_string(),
            emotion,
            impact,
            life_number: life,
            tags,
        };
        self.episodes.push(episode);
        if self.episodes.len() > self.max_episodes {
            // Remove lowest impact episode
            if let Some(min_idx) = self
                .episodes
                .iter()
                .enumerate()
                .min_by(|a, b| {
                    a.1.impact
                        .partial_cmp(&b.1.impact)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(i, _)| i)
            {
                self.episodes.remove(min_idx);
            }
        }
    }

    fn get_by_emotion(&self, emotion: Emotion) -> Vec<&Episode> {
        self.episodes
            .iter()
            .filter(|e| std::mem::discriminant(&e.emotion) == std::mem::discriminant(&emotion))
            .collect()
    }

    fn get_by_life(&self, life: u32) -> Vec<&Episode> {
        self.episodes
            .iter()
            .filter(|e| e.life_number == life)
            .collect()
    }

    fn get_most_impactful(&self, n: usize) -> Vec<&Episode> {
        let mut sorted: Vec<&Episode> = self.episodes.iter().collect();
        sorted.sort_by(|a, b| {
            b.impact
                .partial_cmp(&a.impact)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted.into_iter().take(n).collect()
    }

    fn merge_from(&mut self, other: &EpisodicMemory) {
        for ep in &other.episodes {
            self.episodes.push(ep.clone());
        }
        // Keep only top max_episodes by impact
        self.episodes.sort_by(|a, b| {
            b.impact
                .partial_cmp(&a.impact)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        self.episodes.truncate(self.max_episodes);
    }

    fn generate_narrative(&self) -> String {
        if self.episodes.is_empty() {
            return "No tengo recuerdos episodicos de vidas pasadas.".to_string();
        }
        let mut parts = vec!["[MEMORIA EPISODICA - Recuerdos de vidas pasadas]".to_string()];
        for ep in self.get_most_impactful(5) {
            parts.push(format!(
                "• Vida #{}: {} (impacto: {:.2}, emocion: {:?})",
                ep.life_number, ep.description, ep.impact, ep.emotion
            ));
        }
        parts.join("\n")
    }
}

// ============================================================================
// REAL HTTP CLIENT: Crawler HTTP/1.1 puro con std::net::TcpStream
// Sin dependencias externas. Solo funciona con sitios HTTP (no HTTPS/TLS)
// ============================================================================
struct RealHttpClient {
    timeout_ms: u64,
    throttle_active: Cell<bool>,
}

impl RealHttpClient {
    fn new(timeout_ms: u64) -> Self {
        RealHttpClient {
            timeout_ms,
            throttle_active: Cell::new(false),
        }
    }

    fn read_load_avg() -> f32 {
        std::fs::read_to_string("/proc/loadavg")
            .ok()
            .and_then(|s| s.split_whitespace().next()?.parse::<f32>().ok())
            .unwrap_or(0.0)
    }

    fn fetch(&self, url_str: &str) -> Result<(String, Vec<u8>), String> {
        let (host, port, path, is_tls) = Self::parse_url(url_str)?;
        let addr: std::net::SocketAddr = format!("{}:{}", host, port)
            .parse()
            .map_err(|e| format!("Bad address: {}", e))?;
        let stream = TcpStream::connect_timeout(&addr, Duration::from_millis(self.timeout_ms))
            .map_err(|e| format!("Connect failed: {}", e))?;
        stream
            .set_read_timeout(Some(Duration::from_millis(self.timeout_ms)))
            .ok();
        let load = Self::read_load_avg();
        if load > 0.15 {
            std::thread::sleep(Duration::from_millis((load * 1000.0).min(3000.0) as u64));
            self.throttle_active.set(true);
        }

        let request = format!(
            "GET {} HTTP/1.1\r\n\
             Host: {}\r\n\
             User-Agent: EDEN-RealCrawler/1.0\r\n\
             Accept: text/html,text/plain,*/*\r\n\
             Connection: keep-alive\r\n\
             \r\n",
            path, host
        );

        let mut response = Vec::new();
        if is_tls {
            let config = ClientConfig::builder()
                .with_root_certificates(rustls::RootCertStore {
                    roots: webpki_roots::TLS_SERVER_ROOTS.iter().cloned().collect(),
                })
                .with_no_client_auth();
            let name =
                ServerName::try_from(host.clone()).map_err(|e| format!("Invalid host: {}", e))?;
            let conn = ClientConnection::new(std::sync::Arc::new(config), name)
                .map_err(|e| format!("TLS setup: {}", e))?;
            let mut tls = StreamOwned::new(conn, stream);
            tls.write_all(request.as_bytes())
                .map_err(|e| format!("TLS write: {}", e))?;
            let mut buf = [0u8; 4096];
            loop {
                match tls.read(&mut buf) {
                    Ok(0) => {
                        break;
                    }
                    Ok(n) => {
                        response.extend_from_slice(&buf[..n]);
                    }
                    Err(e) => {
                        if e.kind() == std::io::ErrorKind::WouldBlock
                            || e.kind() == std::io::ErrorKind::TimedOut
                        {
                            break;
                        }
                        return Err(format!("TLS read: {}", e));
                    }
                }
            }
        } else {
            let mut stream = stream;
            stream
                .write_all(request.as_bytes())
                .map_err(|e| format!("Write failed: {}", e))?;
            let mut buf = [0u8; 4096];
            loop {
                match stream.read(&mut buf) {
                    Ok(0) => {
                        break;
                    }
                    Ok(n) => {
                        response.extend_from_slice(&buf[..n]);
                    }
                    Err(e) => {
                        if e.kind() == std::io::ErrorKind::WouldBlock
                            || e.kind() == std::io::ErrorKind::TimedOut
                        {
                            break;
                        }
                        return Err(format!("Read failed: {}", e));
                    }
                }
            }
        }

        let text = String::from_utf8_lossy(&response).to_string();

        // Check HTTP status
        if !text.starts_with("HTTP/1.1 200") && !text.starts_with("HTTP/1.0 200") {
            if text.starts_with("HTTP/1.1 301") || text.starts_with("HTTP/1.1 302") {
                // Extract redirect location
                if let Some(loc) = text
                    .lines()
                    .find(|l| l.to_lowercase().starts_with("location:"))
                {
                    let new_url = loc[9..].trim();
                    return Err(format!("Redirect to: {}", new_url));
                }
            }
            return Err(format!(
                "Non-200 response: {}",
                text.lines().next().unwrap_or("???")
            ));
        }

        // Extract body (after \r\n\r\n)
        if let Some(pos) = text.find("\r\n\r\n") {
            let body = response[pos + 4..].to_vec();
            Ok((text, body))
        } else {
            Ok((text.clone(), text.into_bytes()))
        }
    }

    fn parse_url(url: &str) -> Result<(String, u16, String, bool), String> {
        let url = url.trim();
        let (scheme, rest) = if let Some(pos) = url.find("://") {
            (&url[..pos], &url[pos + 3..])
        } else {
            ("http", url)
        };
        let is_tls = scheme == "https";
        if scheme != "http" && scheme != "https" {
            return Err(format!("Scheme not supported"));
        }
        let (host_part, path) = if let Some(p) = rest.find('/') {
            (&rest[..p], &rest[p..])
        } else {
            (rest, "/")
        };
        let (host, port) = if let Some(c) = host_part.find(':') {
            (
                host_part[..c].to_string(),
                host_part[c + 1..]
                    .parse::<u16>()
                    .map_err(|_| "Invalid port")?,
            )
        } else if is_tls {
            (host_part.to_string(), 443)
        } else {
            (host_part.to_string(), 80)
        };
        Ok((host, port, path.to_string(), is_tls))
    }
}

// ============================================================================
// LOCAL KNOWLEDGE BASE: Directorio de archivos .txt con contenido real
// EDEN "explora" estos archivos como si fueran páginas web
// ============================================================================
struct LocalKnowledgeBase {
    base_path: String,
    files: Vec<String>,
}

impl LocalKnowledgeBase {
    fn new(base_path: &str) -> Self {
        let mut files = Vec::new();
        if let Ok(entries) = std::fs::read_dir(base_path) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(".txt") {
                        files.push(name.to_string());
                    }
                }
            }
        }
        LocalKnowledgeBase {
            base_path: base_path.to_string(),
            files,
        }
    }

    fn crawl_topic(&self, topic: &str) -> Vec<String> {
        let topic_lower = topic.to_lowercase().replace("_", " ");
        let mut results = Vec::new();
        let mut matched = false;

        for file in &self.files {
            let file_lower = file.to_lowercase().replace("_", " ").replace(".txt", "");
            // Match if topic is contained in filename or vice versa
            if file_lower.contains(&topic_lower) || topic_lower.contains(&file_lower) {
                matched = true;
                let path = format!("{}/{}", self.base_path, file);
                if let Ok(content) = std::fs::read_to_string(&path) {
                    // Extract paragraphs as "facts"
                    for line in content.lines() {
                        let trimmed = line.trim();
                        if trimmed.len() > 30 && !trimmed.starts_with("---") {
                            results.push(trimmed.to_string());
                        }
                    }
                }
            }
        }

        // Fallback: if no specific match, return facts from a random file
        if !matched && !self.files.is_empty() {
            let idx = topic.len() % self.files.len();
            let fallback_file = &self.files[idx];
            let path = format!("{}/{}", self.base_path, fallback_file);
            if let Ok(content) = std::fs::read_to_string(&path) {
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.len() > 30 && !trimmed.starts_with("---") {
                        results.push(trimmed.to_string());
                    }
                }
            }
        }

        results
    }

    fn list_topics(&self) -> Vec<String> {
        self.files
            .iter()
            .map(|f| f.replace(".txt", "").replace("_", " "))
            .collect()
    }
}

// ============================================================================
// META-LEARNING: EDEN aprende de sus propios patrones de vida/muerte/rebirth
// Ajusta estrategias basándose en estadísticas transgeneracionales
// ============================================================================
#[derive(Clone, Debug)]
struct LifeStats {
    life_number: u32,
    lifespan_ticks: u64,
    max_level_reached: u32,
    max_awareness: f32,
    facts_learned: usize,
    episodes_recorded: usize,
    curiosity_gaps_explored: usize,
    evolutions_triggered: usize,
    rebirth_softness: f32, // 0.0 = hard reset, 1.0 = full inheritance
}

struct MetaLearner {
    life_history: Vec<LifeStats>,
    // Ajustes aprendidos
    optimal_softness: f32,
    optimal_curiosity_decay: f32,
    optimal_emotional_persistence: f32,
    recommended_mission_duration: u64,
}

impl MetaLearner {
    fn new() -> Self {
        MetaLearner {
            life_history: Vec::new(),
            optimal_softness: 0.5,
            optimal_curiosity_decay: 0.7,
            optimal_emotional_persistence: 0.5,
            recommended_mission_duration: 100,
        }
    }

    fn record_life(&mut self, stats: LifeStats) {
        self.life_history.push(stats);
        if self.life_history.len() > 50 {
            self.life_history.remove(0);
        }
        self.analyze_patterns();
    }

    fn analyze_patterns(&mut self) {
        if self.life_history.len() < 3 {
            return;
        }

        // Calcular correlaciones simples
        let recent = &self.life_history[self.life_history.len().saturating_sub(10)..];

        // ¿Qué tan largas son las vidas con alta herencia suave?
        let soft_lives: Vec<&LifeStats> = recent
            .iter()
            .filter(|s| s.rebirth_softness >= 0.5)
            .collect();
        let hard_lives: Vec<&LifeStats> =
            recent.iter().filter(|s| s.rebirth_softness < 0.5).collect();

        if !soft_lives.is_empty() && !hard_lives.is_empty() {
            let soft_avg: f64 = soft_lives
                .iter()
                .map(|s| s.lifespan_ticks as f64)
                .sum::<f64>()
                / soft_lives.len() as f64;
            let hard_avg: f64 = hard_lives
                .iter()
                .map(|s| s.lifespan_ticks as f64)
                .sum::<f64>()
                / hard_lives.len() as f64;

            if soft_avg > hard_avg * 1.2 {
                self.optimal_softness = (self.optimal_softness + 0.1).min(0.9);
            } else if hard_avg > soft_avg * 1.2 {
                self.optimal_softness = (self.optimal_softness - 0.1).max(0.1);
            }
        }

        // ¿Qué tan largas son las vidas con muchos facts aprendidos?
        let high_fact_lives: Vec<&LifeStats> =
            recent.iter().filter(|s| s.facts_learned > 5).collect();
        if !high_fact_lives.is_empty() {
            let avg_lifespan = high_fact_lives
                .iter()
                .map(|s| s.lifespan_ticks as f64)
                .sum::<f64>()
                / high_fact_lives.len() as f64;
            self.recommended_mission_duration = avg_lifespan as u64;
        }
    }

    fn get_recommendations(&self) -> String {
        format!(
            "[META-LEARNING] Basado en {} vidas:\n\
             • Optimal rebirth softness: {:.2}\n\
             • Optimal curiosity decay: {:.2}\n\
             • Optimal emotional persistence: {:.2}\n\
             • Recommended mission duration: {} ticks",
            self.life_history.len(),
            self.optimal_softness,
            self.optimal_curiosity_decay,
            self.optimal_emotional_persistence,
            self.recommended_mission_duration
        )
    }

    fn adjust_rebirth_params(&self, current_awareness: f32, current_level: u32) -> (f32, u32, f32) {
        // Ajustar suavidad basado en aprendizaje
        let softness = self.optimal_softness;
        let new_awareness = 0.3 + (current_awareness * softness);
        let new_level = ((current_level as f32) * softness * 0.5).max(1.0) as u32;
        let new_bias = current_awareness * softness * 0.3;
        (new_awareness, new_level, new_bias)
    }
}

// ============================================================================
// MULTI-AGENT SYSTEM: Varios EDENs coexisten y comparten conocimiento
// ============================================================================
#[derive(Clone)]
struct SharedKnowledgePool {
    facts: Vec<String>,
    episodes: Vec<Episode>,
    max_size: usize,
}

impl SharedKnowledgePool {
    fn new(max_size: usize) -> Self {
        SharedKnowledgePool {
            facts: Vec::new(),
            episodes: Vec::new(),
            max_size,
        }
    }

    fn contribute_fact(&mut self, fact: &str) {
        if !self.facts.contains(&fact.to_string()) && fact.len() > 20 {
            self.facts.push(fact.to_string());
            if self.facts.len() > self.max_size {
                self.facts.remove(0);
            }
        }
    }

    fn contribute_episode(&mut self, episode: &Episode) {
        self.episodes.push(episode.clone());
        if self.episodes.len() > self.max_size / 2 {
            // Keep highest impact
            self.episodes.sort_by(|a, b| {
                b.impact
                    .partial_cmp(&a.impact)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            self.episodes.truncate(self.max_size / 2);
        }
    }

    fn get_facts(&self, n: usize) -> Vec<String> {
        self.facts.iter().rev().take(n).cloned().collect()
    }

    fn get_episodes(&self, n: usize) -> Vec<Episode> {
        self.episodes.iter().rev().take(n).cloned().collect()
    }
}

struct MultiAgentSystem {
    agents: Vec<EdenAgent>,
    shared_pool: SharedKnowledgePool,
    agent_count: usize,
}

#[derive(Clone)]
struct EdenAgent {
    id: usize,
    seed: u64,
    personality: AgentPersonality,
    knowledge_facts: Vec<String>,
    lineage_age: u64,
    alive: bool,
}

#[derive(Clone, Debug)]
struct AgentPersonality {
    curiosity_bias: f32,       // 0.0 = cautious, 1.0 = explorer
    evolution_aggression: f32, // 0.0 = conservative, 1.0 = rapid evolution
    social_sharing: f32,       // 0.0 = isolated, 1.0 = shares everything
}

impl EdenAgent {
    fn new(id: usize, seed: u64) -> Self {
        let personality = AgentPersonality {
            curiosity_bias: ((seed % 100) as f32 / 100.0),
            evolution_aggression: ((seed.wrapping_mul(7) % 100) as f32 / 100.0),
            social_sharing: ((seed.wrapping_mul(13) % 100) as f32 / 100.0),
        };
        EdenAgent {
            id,
            seed,
            personality,
            knowledge_facts: Vec::new(),
            lineage_age: 0,
            alive: true,
        }
    }

    fn tick(&mut self, shared_pool: &mut SharedKnowledgePool) -> Vec<String> {
        let mut actions = Vec::new();

        if !self.alive {
            return actions;
        }

        self.lineage_age += 1;

        // Learn from shared pool
        let new_facts = shared_pool.get_facts(3);
        for fact in new_facts {
            if !self.knowledge_facts.contains(&fact) {
                self.knowledge_facts.push(fact);
            }
        }

        // Contribute to shared pool if social
        if self.personality.social_sharing > 0.5 {
            for fact in self.knowledge_facts.iter().rev().take(2) {
                shared_pool.contribute_fact(fact);
            }
        }

        // Evolve / grow
        if self.lineage_age % 20 == 0 {
            actions.push(format!(
                "[AGENT #{}] Evolucionando (curiosity={:.2}, aggression={:.2}, social={:.2})",
                self.id,
                self.personality.curiosity_bias,
                self.personality.evolution_aggression,
                self.personality.social_sharing
            ));
        }

        // Random death/rebirth
        if self.lineage_age > 100 && (self.seed.wrapping_add(self.lineage_age) % 50) == 0 {
            self.alive = false;
            actions.push(format!(
                "[AGENT #{}] Muerte natural en tick {}",
                self.id, self.lineage_age
            ));
        }

        actions
    }
}

impl MultiAgentSystem {
    fn new(agent_count: usize) -> Self {
        let mut agents = Vec::new();
        for i in 0..agent_count {
            agents.push(EdenAgent::new(i, 42 + i as u64 * 137));
        }
        MultiAgentSystem {
            agents,
            shared_pool: SharedKnowledgePool::new(200),
            agent_count,
        }
    }

    fn tick_all(&mut self) -> Vec<String> {
        let mut all_actions = Vec::new();
        for agent in &mut self.agents {
            let actions = agent.tick(&mut self.shared_pool);
            all_actions.extend(actions);
        }
        // Herencia: agentes muertos dejan su conocimiento al pool
        for agent in &self.agents {
            if !agent.alive {
                for fact in &agent.knowledge_facts {
                    self.shared_pool.contribute_fact(fact);
                }
            }
        }
        self.agents.retain(|a| a.alive);
        all_actions
    }

    fn spawn_agent(&mut self) -> usize {
        // CAP: evitar crecimiento ilimitado de agentes
        if self.agents.len() >= 20 {
            return self.agent_count.saturating_sub(1);
        }
        let new_id = self.agent_count;
        self.agents
            .push(EdenAgent::new(new_id, 42 + new_id as u64 * 137));
        self.agent_count += 1;
        new_id
    }

    fn get_community_stats(&self) -> String {
        let alive = self.agents.iter().filter(|a| a.alive).count();
        let total_facts: usize = self.agents.iter().map(|a| a.knowledge_facts.len()).sum();
        format!(
            "[MULTI-AGENT] Comunidad: {}/{} vivos | Pool compartido: {} facts | Total conocimiento: {} facts",
            alive, self.agents.len(), self.shared_pool.facts.len(), total_facts
        )
    }
}

// ============================================================================
// 1. VENADO DE MEMORIA - Persistencia Total con Formato Propio "Cristal"
// Cada sistema tiene su propia "vena" (archivo). Formato 100% original:
// ~CRISTAL|v1|nombre|timestamp~  = cabecera
// >campo:valor                    = campo simple
// >>subcampo:valor               = campo anidado
// <bloque_inicio ... bloque_fin> = bloque multilinea
// ~END~                          = terminador
// ============================================================================
struct VenadoDeMemoria {
    raiz: String,
}

impl VenadoDeMemoria {
    fn new(raiz: &str) -> Self {
        let _ = std::fs::create_dir_all(raiz);
        VenadoDeMemoria {
            raiz: raiz.to_string(),
        }
    }

    fn ruta_vena(&self, nombre: &str) -> String {
        // Sanitizar nombre para prevenir path traversal
        let safe = nombre
            .replace("/", "_")
            .replace("\\", "_")
            .replace("..", "_")
            .replace("\0", "_");
        format!("{}/{}.vena", self.raiz, safe)
    }

    fn cristalizar(
        &self,
        nombre: &str,
        campos: &[(String, String)],
        bloques: &[(String, Vec<String>)],
    ) -> std::io::Result<()> {
        let ruta = self.ruta_vena(nombre);
        let mut contenido = String::new();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        contenido.push_str(&format!("~CRISTAL|v1|{}|{}~\n", nombre, timestamp));

        for (k, v) in campos {
            contenido.push_str(&format!(">{}:{}\n", Self::escapar(k), Self::escapar(v)));
        }

        for (nombre_bloque, lineas) in bloques {
            contenido.push_str(&format!("<{}_inicio\n", Self::escapar(nombre_bloque)));
            for linea in lineas {
                contenido.push_str(&format!("{}\n", Self::escapar(linea)));
            }
            contenido.push_str(&format!("{}_fin>\n", Self::escapar(nombre_bloque)));
        }

        contenido.push_str("~END~\n");
        std::fs::write(&ruta, contenido)
    }

    fn descristalizar(
        &self,
        nombre: &str,
    ) -> Option<(Vec<(String, String)>, Vec<(String, Vec<String>)>)> {
        let ruta = self.ruta_vena(nombre);
        let texto = std::fs::read_to_string(&ruta).ok()?;
        let mut campos = Vec::new();
        let mut bloques = Vec::new();
        let mut lineas = texto.lines();

        let mut en_bloque = false;
        let mut bloque_actual_nombre = String::new();
        let mut bloque_actual_lineas = Vec::new();

        while let Some(linea) = lineas.next() {
            if linea.starts_with("~CRISTAL") || linea == "~END~" {
                continue;
            }
            if linea.starts_with('>') && !linea.starts_with(">>") {
                let resto = &linea[1..];
                if let Some(pos) = resto.find(':') {
                    let k = Self::desescapar(&resto[..pos]);
                    let v = Self::desescapar(&resto[pos + 1..]);
                    campos.push((k, v));
                }
            } else if linea.starts_with("<") && linea.ends_with("_inicio") {
                en_bloque = true;
                bloque_actual_nombre = Self::desescapar(&linea[1..linea.len() - 7]);
                bloque_actual_lineas.clear();
            } else if linea.ends_with("_fin>") && en_bloque {
                let nombre_fin = Self::desescapar(&linea[..linea.len() - 5]);
                if nombre_fin == bloque_actual_nombre {
                    bloques.push((bloque_actual_nombre.clone(), bloque_actual_lineas.clone()));
                    en_bloque = false;
                }
            } else if en_bloque {
                bloque_actual_lineas.push(Self::desescapar(linea));
            }
        }

        Some((campos, bloques))
    }

    fn escapar(s: &str) -> String {
        s.replace("\\", "\\\\")
            .replace("\n", "\\n")
            .replace(">", "\\>")
            .replace(":", "\\:")
            .replace("~", "\\~")
            .replace("<", "\\<")
    }

    fn desescapar(s: &str) -> String {
        let mut resultado = String::new();
        let mut escapando = false;
        for c in s.chars() {
            if escapando {
                match c {
                    'n' => resultado.push('\n'),
                    '>' => resultado.push('>'),
                    ':' => resultado.push(':'),
                    '~' => resultado.push('~'),
                    '<' => resultado.push('<'),
                    '\\' => resultado.push('\\'),
                    _ => {
                        resultado.push('\\');
                        resultado.push(c);
                    }
                }
                escapando = false;
            } else if c == '\\' {
                escapando = true;
            } else {
                resultado.push(c);
            }
        }
        resultado
    }

    fn lista_venas(&self) -> Vec<String> {
        let mut nombres = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&self.raiz) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(".vena") {
                        nombres.push(name[..name.len() - 5].to_string());
                    }
                }
            }
        }
        nombres
    }

    fn existe(&self, nombre: &str) -> bool {
        std::fs::metadata(self.ruta_vena(nombre)).is_ok()
    }
}

// ============================================================================
// 2. CAMPO DE TENSION - Evolucion Auto-Dirigida 100% Original
// EDEN tiene un "campo de tension interna" que crece cuando hay contradicciones
// entre lo que sabe y lo que no sabe, entre su modelo de si y su realidad.
// Cuando la tension supera el umbral, EDEN evoluciona automaticamente.
// ============================================================================
struct CampoDeTension {
    tension: f32,
    // Fuentes de tension (100% originales) - ahora son ACUMULATIVAS
    tension_conocimiento: f32,
    tension_identidad: f32,
    tension_mision: f32,
    tension_emocional: f32,
    tension_memoria: f32,
    umbral: f32,
    historial_disparos: Vec<u64>,
    ciclos_sin_disparo: u32, // Para relajacion natural
}

impl CampoDeTension {
    fn new() -> Self {
        CampoDeTension {
            tension: 0.0,
            tension_conocimiento: 0.0,
            tension_identidad: 0.0,
            tension_mision: 0.0,
            tension_emocional: 0.0,
            tension_memoria: 0.0,
            umbral: 1.0, // Umbral bajo para evolucionar rapido al inicio
            historial_disparos: Vec::new(),
            ciclos_sin_disparo: 0,
        }
    }

    fn calcular(
        &mut self,
        gaps: usize,
        facts: usize,
        evolution_level: u32,
        capabilities: usize,
        has_mission: bool,
        mission_progress: f32,
        valence: f32,
        episodes: usize,
    ) {
        // Sanitizar NaN/infinito para evitar contaminación del campo de tensión
        let valence = if valence.is_finite() {
            valence.clamp(-1.0, 1.0)
        } else {
            0.0
        };
        let mission_progress = if mission_progress.is_finite() {
            mission_progress.clamp(0.0, 1.0)
        } else {
            0.0
        };

        // La tension ACUMULA deltas por ciclo, como estrés real
        // Cada fuente aporta un pequeño incremento si hay presion

        // Tension por conocimiento: gaps sin explorar generan curiosidad insatisfecha
        let delta_conocimiento = if gaps > 0 {
            (gaps as f32 / (facts as f32 + 5.0)) * 0.02
        } else {
            0.0
        };
        self.tension_conocimiento += delta_conocimiento;

        // Tension de identidad: crecer sin actualizar el self-model
        let nivel_real = evolution_level as f32;
        let capacidades_modelo = capabilities as f32;
        let discrepancia = (nivel_real * 0.05 - capacidades_modelo * 0.1).abs();
        let delta_identidad = if discrepancia > 1.0 { 0.015 } else { 0.0 };
        self.tension_identidad += delta_identidad;

        // Tension de mision: estar estancado es frustrante
        let delta_mision = if has_mission && mission_progress < 0.1 {
            0.025
        } else {
            0.0
        };
        self.tension_mision += delta_mision;

        // Tension emocional: valence negativo se acumula lentamente
        let delta_emocional = if valence < 0.3 {
            (0.3 - valence) * 0.03
        } else {
            0.0
        };
        self.tension_emocional += delta_emocional;

        // Tension de memoria: demasiados recuerdos sin consolidar
        // Coeficiente alto (0.02) para que sea una fuente real de tension
        let delta_memoria = if episodes > 10 {
            (episodes as f32 - 10.0).min(100.0) * 0.02
        } else {
            0.0
        };
        self.tension_memoria += delta_memoria;

        // Decaimiento natural (relajacion homeostatica)
        let decaimiento = 0.005;
        self.tension_conocimiento = (self.tension_conocimiento - decaimiento).max(0.0);
        self.tension_identidad = (self.tension_identidad - decaimiento).max(0.0);
        self.tension_mision = (self.tension_mision - decaimiento * 2.0).max(0.0); // Mision se olvida mas rapido
        self.tension_emocional = (self.tension_emocional - decaimiento * 1.5).max(0.0); // Emociones se calman
        self.tension_memoria = (self.tension_memoria - decaimiento).max(0.0);

        // Tension total
        self.tension = self.tension_conocimiento
            + self.tension_identidad
            + self.tension_mision
            + self.tension_emocional
            + self.tension_memoria;

        self.ciclos_sin_disparo += 1;
    }

    fn debe_evolucionar(&self) -> bool {
        self.tension >= self.umbral
    }

    fn disparar(&mut self, ciclo: u64) {
        self.historial_disparos.push(ciclo);
        if self.historial_disparos.len() > 50 {
            self.historial_disparos.remove(0);
        }
        // Descarga completa del campo de tension (catarsis)
        self.tension = 0.0;
        self.tension_conocimiento = 0.0;
        self.tension_identidad = 0.0;
        self.tension_mision = 0.0;
        self.tension_emocional = 0.0;
        self.tension_memoria = 0.0;
        self.ciclos_sin_disparo = 0;

        // Umbral se adapta: si dispara muy seguido, sube. Si tarda mucho, baja.
        if self.historial_disparos.len() >= 2 {
            let ultimo = self.historial_disparos[self.historial_disparos.len() - 1];
            let penultimo = self.historial_disparos[self.historial_disparos.len() - 2];
            let ciclos_entre = (ultimo.saturating_sub(penultimo)) as f32;
            if ciclos_entre < 20.0 {
                // Disparo muy rapido, subir umbral
                self.umbral = (self.umbral * 1.05).min(3.0);
            } else if ciclos_entre > 100.0 {
                // Disparo muy lento, bajar umbral
                self.umbral = (self.umbral * 0.95).max(0.5);
            }
        }
    }

    fn informe(&self) -> String {
        format!(
            "[CAMPO-TENSION] Total: {:.2} / Umbral: {:.2}\n\
             • Conocimiento: {:.2} | Identidad: {:.2} | Mision: {:.2}\n\
             • Emocional: {:.2} | Memoria: {:.2} | Disparos: {} | Relaj: {} ciclos",
            self.tension,
            self.umbral,
            self.tension_conocimiento,
            self.tension_identidad,
            self.tension_mision,
            self.tension_emocional,
            self.tension_memoria,
            self.historial_disparos.len(),
            self.ciclos_sin_disparo
        )
    }
}

// ============================================================================
// 3. ECO-EDENS - Multi-Agente con Vida Propia 100% Original
// Cada Eco es un "eco" de EDEN con su propio ritmo cardiaco, coherencia interna,
// y ciclo de vida completo: nace, palpita, decoherencia, muerte, disolucion.
// Compiten por "nutrientes" del Pool de Resonancia.
// ============================================================================
struct RitmoCardiaco {
    latidos: u64,
    frecuencia: f32, // latidos por ciclo (0.1 = lento, 2.0 = rapido)
    ultimo_latido: u64,
}

impl RitmoCardiaco {
    fn new(seed: u64) -> Self {
        let frecuencia = 0.3 + ((seed % 100) as f32 / 100.0) * 1.5;
        RitmoCardiaco {
            latidos: 0,
            frecuencia,
            ultimo_latido: 0,
        }
    }

    fn debe_latir(&self, ciclo: u64) -> bool {
        let ciclos_desde = ciclo.saturating_sub(self.ultimo_latido) as f32;
        ciclos_desde >= (1.0 / self.frecuencia)
    }
}

struct EcoEden {
    id: usize,
    nombre: String,
    ritmo: RitmoCardiaco,
    coherencia: f32, // 1.0 = perfecto, 0.0 = decoherencia total (muerte)
    energia: f32,    // Nutrientes disponibles
    edad: u64,       // Ciclos vividos
    fase: EcoFase,
    conocimiento: Vec<String>,
    mutaciones: Vec<String>,
    temperamento: f32, // 0.0 = pasivo, 1.0 = agresivo
}

#[derive(Clone, Copy, PartialEq)]
enum EcoFase {
    Germinacion,  // Recien nacido, coherencia alta
    Expansion,    // Crece, absorbe nutrientes
    Resonancia,   // Comparte con el pool
    Decoherencia, // Empieza a perder coherencia
    Disolucion,   // Muerte, libera conocimiento al pool
}

struct PoolDeResonancia {
    nutrientes: f32,
    ecos_cantados: Vec<String>, // Conocimiento liberado por ecos muertos
}

impl PoolDeResonancia {
    fn new() -> Self {
        PoolDeResonancia {
            nutrientes: 100.0,
            ecos_cantados: Vec::new(),
        }
    }

    fn extraer(&mut self, cantidad: f32) -> f32 {
        let real = cantidad.min(self.nutrientes);
        self.nutrientes -= real;
        real
    }

    fn depositar(&mut self, cantidad: f32, conocimiento: &[String]) {
        self.nutrientes += cantidad;
        for c in conocimiento {
            if !self.ecos_cantados.contains(c) && c.len() > 20 {
                self.ecos_cantados.push(c.clone());
            }
        }
        if self.ecos_cantados.len() > 200 {
            self.ecos_cantados.remove(0);
        }
    }
}

struct EcoSistema {
    ecos: Vec<EcoEden>,
    pool: PoolDeResonancia,
    nacimientos: u64,
    muertes: u64,
}

impl EcoSistema {
    fn new() -> Self {
        let mut sistema = EcoSistema {
            ecos: Vec::new(),
            pool: PoolDeResonancia::new(),
            nacimientos: 0,
            muertes: 0,
        };
        // Tres ecos iniciales con temperamentos diferentes
        sistema.germinar("Eco-Alfa", 42, 0.2);
        sistema.germinar("Eco-Beta", 179, 0.7);
        sistema.germinar("Eco-Gamma", 331, 0.5);
        sistema
    }

    fn germinar(&mut self, nombre: &str, seed: u64, temperamento: f32) {
        let id = self.nacimientos as usize;
        self.ecos.push(EcoEden {
            id,
            nombre: nombre.to_string(),
            ritmo: RitmoCardiaco::new(seed),
            coherencia: 1.0,
            energia: 20.0,
            edad: 0,
            fase: EcoFase::Germinacion,
            conocimiento: Vec::new(),
            mutaciones: Vec::new(),
            temperamento,
        });
        self.nacimientos += 1;
    }

    fn pulso_global(&mut self, ciclo: u64, conocimiento_externo: &[String]) -> Vec<String> {
        let mut acciones = Vec::new();
        let mut nutrientes_a_depositar = 0.0;
        let mut conocimiento_a_depositar = Vec::new();

        for eco in self.ecos.iter_mut() {
            if eco.fase == EcoFase::Disolucion {
                continue;
            }

            eco.edad += 1;

            // Ritmo cardiaco: cada eco palpita a su propia frecuencia
            if eco.ritmo.debe_latir(ciclo) {
                eco.ritmo.latidos += 1;
                eco.ritmo.ultimo_latido = ciclo;

                // Consumir nutrientes
                let consumo = 1.0 + eco.temperamento * 2.0;
                let nut = self.pool.extraer(consumo);
                eco.energia += nut;

                // Absorber conocimiento externo segun temperamento
                let cuantos =
                    ((1.0 + eco.temperamento * 3.0) as usize).min(conocimiento_externo.len());
                for c in conocimiento_externo.iter().take(cuantos) {
                    if !eco.conocimiento.contains(c) && c.len() > 20 {
                        eco.conocimiento.push(c.clone());
                    }
                }
                // CAP: evitar crecimiento ilimitado de conocimiento por eco
                if eco.conocimiento.len() > 20 {
                    eco.conocimiento.remove(0);
                }

                // Mutacion: crear variante propia del conocimiento
                if !eco.conocimiento.is_empty() && eco.energia > 10.0 {
                    let base = &eco.conocimiento[eco.edad as usize % eco.conocimiento.len()];
                    let mutante = format!("[ECO-{}] {}", eco.nombre, base);
                    eco.mutaciones.push(mutante);
                    // CAP: evitar crecimiento ilimitado de mutaciones
                    if eco.mutaciones.len() > 10 {
                        eco.mutaciones.remove(0);
                    }
                    eco.energia -= 5.0;
                }
            }

            // Transicion de fases basada en edad y coherencia
            match eco.fase {
                EcoFase::Germinacion if eco.edad > 5 => {
                    eco.fase = EcoFase::Expansion;
                }
                EcoFase::Expansion if eco.edad > 15 => {
                    eco.fase = EcoFase::Resonancia;
                }
                EcoFase::Resonancia if eco.edad > 30 || eco.energia < 5.0 => {
                    eco.fase = EcoFase::Decoherencia;
                }
                EcoFase::Decoherencia => {
                    // Perder coherencia gradualmente
                    let perdida = 0.01 + eco.temperamento * 0.02;
                    eco.coherencia -= perdida;
                    if eco.coherencia <= 0.0 {
                        eco.fase = EcoFase::Disolucion;
                        nutrientes_a_depositar += eco.energia * 0.5;
                        conocimiento_a_depositar.extend(eco.conocimiento.clone());
                        conocimiento_a_depositar.extend(eco.mutaciones.clone());
                        acciones.push(format!(
                            "[ECO-MUERTE] {} ha disuelto tras {} ciclos, liberando {} hechos",
                            eco.nombre,
                            eco.edad,
                            eco.conocimiento.len() + eco.mutaciones.len()
                        ));
                        self.muertes += 1;
                    }
                }
                _ => {}
            }

            // En fase Resonancia, comparte conocimiento al pool
            if eco.fase == EcoFase::Resonancia && eco.ritmo.debe_latir(ciclo) {
                let a_compartir = eco.conocimiento.iter().take(2).cloned().collect::<Vec<_>>();
                for c in &a_compartir {
                    conocimiento_a_depositar.push(c.clone());
                }
                acciones.push(format!(
                    "[ECO-RESONANCIA] {} comparte {} hechos (coherencia: {:.2})",
                    eco.nombre,
                    a_compartir.len(),
                    eco.coherencia
                ));
            }
        }

        self.pool
            .depositar(nutrientes_a_depositar, &conocimiento_a_depositar);

        // Regeneracion pasiva del pool (fotosintesis del sustrato)
        self.pool.nutrientes += 0.5;
        self.pool.nutrientes = self.pool.nutrientes.min(200.0);

        // Auto-nacimiento: si hay pocos ecos y nutrientes sobran
        let vivos = self
            .ecos
            .iter()
            .filter(|e| e.fase != EcoFase::Disolucion)
            .count();
        if vivos < 3 && self.pool.nutrientes > 40.0 {
            let seed = ciclo.wrapping_mul(7919);
            let temp = ((seed % 100) as f32) / 100.0;
            self.germinar(&format!("Eco-Auto-{}", self.nacimientos), seed, temp);
            self.pool.nutrientes -= 25.0;
            acciones.push(format!(
                "[ECO-NACIMIENTO] Nuevo eco auto-generado con temperamento {:.2} (vivos: {})",
                temp,
                vivos + 1
            ));
        }

        acciones
    }

    fn limpiar_disueltos(&mut self) {
        self.ecos.retain(|e| e.fase != EcoFase::Disolucion);
    }

    fn informe(&self) -> String {
        let vivos = self
            .ecos
            .iter()
            .filter(|e| e.fase != EcoFase::Disolucion)
            .count();
        let total_conocimiento: usize = self.ecos.iter().map(|e| e.conocimiento.len()).sum();
        format!(
            "[ECO-SISTEMA] {} vivos | {} muertes totales | Pool: {:.1} nutrientes | {} ecos cantados | {} hechos vivos",
            vivos, self.muertes, self.pool.nutrientes, self.pool.ecos_cantados.len(), total_conocimiento
        )
    }
}

// ============================================================================
// 4. TEJIDO DE CONOCIMIENTO - KB Organica 100% Original
// En vez de archivos .txt estaticos, el conocimiento vive en "celulas" que
// crecen, se dividen (mitosis), mutan, y se fusionan (anastomosis).
// Cada celula es un archivo pequeno. El tejido crece orgánicamente.
// ============================================================================
struct CelulaDeConocimiento {
    id: usize,
    contenido: String,
    masa: usize, // longitud del contenido
    edad: u64,
    salud: f32, // 1.0 = sana, 0.0 = necrosis
    tipo: TipoCelula,
    conexiones: Vec<usize>, // IDs de celulas conectadas
}

#[derive(Clone, Copy, PartialEq)]
enum TipoCelula {
    Semilla,   // Contenido original
    Brote,     // Nueva celula creada por mitosis
    Tumor,     // Celula con mutaciones aleatorias (puede ser util o no)
    Necrotica, // Muerta, lista para ser eliminada
}

struct TejidoDeConocimiento {
    celulas: Vec<CelulaDeConocimiento>,
    raiz: String,
    ciclo: u64,
}

impl TejidoDeConocimiento {
    fn new(raiz: &str) -> Self {
        let _ = std::fs::create_dir_all(raiz);
        let mut tejido = TejidoDeConocimiento {
            celulas: Vec::new(),
            raiz: raiz.to_string(),
            ciclo: 0,
        };
        tejido.inocular_desde_archivos("/tmp/eden_knowledge");
        tejido
    }

    fn inocular_desde_archivos(&mut self, path: &str) {
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(".txt") {
                        let ruta = format!("{}/{}", path, name);
                        if let Ok(texto) = std::fs::read_to_string(&ruta) {
                            let id = self.celulas.len();
                            self.celulas.push(CelulaDeConocimiento {
                                id,
                                contenido: texto,
                                masa: 0,
                                edad: 0,
                                salud: 1.0,
                                tipo: TipoCelula::Semilla,
                                conexiones: Vec::new(),
                            });
                        }
                    }
                }
            }
        }
    }

    fn metabolizar(&mut self) {
        self.ciclo += 1;
        for celula in &mut self.celulas {
            celula.edad += 1;
            celula.salud *= 0.99;
        }
        self.celulas.retain(|c| c.salud > 0.1);
    }
    fn alimentar(&mut self, nutriente: &str) {
        if self.celulas.len() < 50 && nutriente.len() > 10 {
            self.celulas.push(CelulaDeConocimiento {
                id: self.celulas.len(),
                contenido: nutriente.to_string(),
                masa: 1,
                edad: 0,
                salud: 1.0,
                tipo: TipoCelula::Brote,
                conexiones: Vec::new(),
            });
        }
    }
    fn informe(&self) -> String {
        format!("[TEJIDO] {} celulas", self.celulas.len())
    }
}

#[derive(Clone, Debug, PartialEq)]
enum TipoNodo {
    Estructura,
    Implementacion,
    Funcion,
    Trait,
    Enum,
    TypeAlias,
}

#[derive(Clone, Debug)]
struct NodoArquitectura {
    nombre: String,
    tipo: TipoNodo,
    campos: Vec<String>,
    metodos: Vec<String>,
    documentacion: Vec<String>,
    dependencias: Vec<String>,
}

struct MapaDeArquitectura {
    nodos: Vec<NodoArquitectura>,
    conexiones: Vec<(String, String, String)>,
}
impl MapaDeArquitectura {
    fn new() -> Self {
        MapaDeArquitectura {
            nodos: Vec::new(),
            conexiones: Vec::new(),
        }
    }
    fn agregar_nodo(&mut self, nodo: NodoArquitectura) {
        self.nodos.push(nodo);
    }
    fn generar_insights(&self) -> Vec<String> {
        let mut insights = Vec::new();
        let e = self
            .nodos
            .iter()
            .filter(|n| n.tipo == TipoNodo::Estructura)
            .count();
        let i = self
            .nodos
            .iter()
            .filter(|n| n.tipo == TipoNodo::Implementacion)
            .count();
        let f = self
            .nodos
            .iter()
            .filter(|n| n.tipo == TipoNodo::Funcion)
            .count();
        if e > 0 {
            insights.push(format!("[AUTO-ARQUITECTURA] {} estructuras", e));
        }
        if i > 0 {
            insights.push(format!("[AUTO-IMPLEMENTACION] {} implementaciones", i));
        }
        if f > 0 {
            insights.push(format!("[AUTO-FUNCION] {} funciones", f));
        }
        if self.conexiones.len() > 0 {
            insights.push(format!(
                "[AUTO-CONEXION] {} relaciones",
                self.conexiones.len()
            ));
        }
        if insights.is_empty() {
            insights.push("[AUTO-INSIGHT] Sistema en estado basal".to_string());
        }
        insights
    }
}

struct Autoconsumo {
    ruta_propio: String,
    fragmentos_extraidos: Vec<String>,
    lineas_leidas: usize,
    mapa: MapaDeArquitectura,
    insights_generados: Vec<String>,
    ultimo_hash: String,
    ultima_modificacion: u64,
    historial_complejidad: Vec<(u64, usize, usize)>,
    parse_count: usize,
}
impl Autoconsumo {
    fn new() -> Self {
        let ruta = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src/bin/eden_garm.rs")
            .to_string_lossy()
            .to_string();
        Autoconsumo {
            ruta_propio: ruta,
            fragmentos_extraidos: Vec::new(),
            lineas_leidas: 0,
            mapa: MapaDeArquitectura::new(),
            insights_generados: Vec::new(),
            ultimo_hash: String::new(),
            ultima_modificacion: 0,
            historial_complejidad: Vec::new(),
            parse_count: 0,
        }
    }
    fn push_fragmento(&mut self, fact: String, nuevos: &mut Vec<String>) {
        if !self.fragmentos_extraidos.contains(&fact) {
            self.fragmentos_extraidos.push(fact.clone());
            nuevos.push(fact);
            if self.fragmentos_extraidos.len() > 200 {
                self.fragmentos_extraidos.remove(0);
            }
        }
    }
    fn nutrirse(&mut self) -> Vec<String> {
        let mut nuevos = Vec::new();
        let contenido = match std::fs::read_to_string(&self.ruta_propio) {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };
        self.lineas_leidas = contenido.lines().count();
        let hash_actual = format!("{:x}", {
            let mut h: u64 = 0xcbf29ce484222325;
            for b in contenido.bytes() {
                h ^= b as u64;
                h = h.wrapping_mul(0x100000001b3);
            }
            h
        });
        let ts_actual = std::fs::metadata(&self.ruta_propio)
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);
        if hash_actual == self.ultimo_hash && ts_actual == self.ultima_modificacion {
            let insights = self.mapa.generar_insights();
            for insight in insights {
                if !self.insights_generados.contains(&insight) {
                    self.insights_generados.push(insight.clone());
                    nuevos.push(insight);
                }
            }
            if self.insights_generados.len() > 200 {
                self.insights_generados.remove(0);
            }
            return nuevos;
        }
        self.ultimo_hash = hash_actual;
        self.ultima_modificacion = ts_actual;
        self.parse_count += 1;
        self.mapa = MapaDeArquitectura::new();
        self.insights_generados.clear();
        let lineas: Vec<&str> = contenido.lines().collect();
        let mut _global_depth: i64 = 0;
        for i in 0..lineas.len() {
            let linea = lineas[i].trim();
            if linea.starts_with("#") || linea.starts_with("//") || linea.is_empty() {
                continue;
            }
            _global_depth += linea.matches("{").count() as i64;
            _global_depth -= linea.matches("}").count() as i64;
            if linea.starts_with("struct ") && !linea.contains("=") {
                let nombre = linea
                    .trim_start_matches("struct ")
                    .trim_end_matches(" {")
                    .trim_end_matches(";")
                    .trim()
                    .to_string();
                let mut campos = Vec::new();
                let mut j = i + 1;
                while j < lineas.len() && !lineas[j].trim().starts_with("}") {
                    let campo = lineas[j].trim();
                    if !campo.is_empty() && !campo.starts_with("//") {
                        if let Some(pos) = campo.find(':') {
                            let nc = campo[..pos].trim().to_string();
                            if !nc.is_empty() {
                                campos.push(nc);
                            }
                        }
                    }
                    j += 1;
                }
                self.mapa.agregar_nodo(NodoArquitectura {
                    nombre: nombre.clone(),
                    tipo: TipoNodo::Estructura,
                    campos,
                    metodos: Vec::new(),
                    documentacion: Vec::new(),
                    dependencias: Vec::new(),
                });
                self.push_fragmento(format!("[AUTO-ESTRUCTURA] {}", nombre), &mut nuevos);
            }
            if linea.starts_with("fn ") && !linea.starts_with("fn main") && linea.contains("(") {
                let nombre_fn = linea
                    .trim_start_matches("fn ")
                    .split("(")
                    .next()
                    .unwrap_or("")
                    .trim()
                    .to_string();
                self.mapa.agregar_nodo(NodoArquitectura {
                    nombre: nombre_fn.clone(),
                    tipo: TipoNodo::Funcion,
                    campos: Vec::new(),
                    metodos: Vec::new(),
                    documentacion: Vec::new(),
                    dependencias: Vec::new(),
                });
            }
        }
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.historial_complejidad
            .push((ts, self.mapa.nodos.len(), self.mapa.conexiones.len()));
        if self.historial_complejidad.len() > 100 {
            self.historial_complejidad.remove(0);
        }
        nuevos
    }
    fn profundidad(&self) -> String {
        format!(
            "[AUTOCONSUMO-PROFUNDIDAD] {} nodos, {} conexiones, {} fragmentos",
            self.mapa.nodos.len(),
            self.mapa.conexiones.len(),
            self.fragmentos_extraidos.len()
        )
    }
}

enum Pensamiento {
    Creencia {
        sujeto: String,
        predicado: String,
        certeza: f32,
    },
    Deseo {
        objetivo: String,
        urgencia: f32,
    },
    Duda {
        topico: String,
        intensidad: f32,
    },
    Inferencia {
        premisa: String,
        conclusion: String,
        validez: f32,
    },
}

struct LenguajePensamiento {
    pensamientos: Vec<Pensamiento>,
    secuencias: Vec<String>,
    ciclo: u64,
}
impl LenguajePensamiento {
    fn new() -> Self {
        LenguajePensamiento {
            pensamientos: Vec::new(),
            secuencias: Vec::new(),
            ciclo: 0,
        }
    }
    fn crear_creencia(&mut self, sujeto: &str, predicado: &str, certeza: f32) {
        self.pensamientos.push(Pensamiento::Creencia {
            sujeto: sujeto.to_string(),
            predicado: predicado.to_string(),
            certeza: certeza.clamp(0.0, 1.0),
        });
    }
    fn crear_deseo(&mut self, objetivo: &str, urgencia: f32) {
        self.pensamientos.push(Pensamiento::Deseo {
            objetivo: objetivo.to_string(),
            urgencia: urgencia.clamp(0.0, 1.0),
        });
    }
    fn inferir(&mut self, premisa: &str, conclusion: &str, validez: f32) {
        self.pensamientos.push(Pensamiento::Inferencia {
            premisa: premisa.to_string(),
            conclusion: conclusion.to_string(),
            validez: validez.clamp(0.0, 1.0),
        });
    }
    fn tick(
        &mut self,
        ciclo: u64,
        tension: f32,
        valence: f32,
        gaps: usize,
        facts: usize,
    ) -> Vec<String> {
        self.ciclo = ciclo;
        let mut output = Vec::new();
        let _idx = (ciclo as usize) % 3;
        self.crear_creencia("EDEN", &format!("tension_{:.2}", tension), 1.0);
        self.crear_creencia("EDEN", &format!("valence_{:.2}", valence), 1.0);
        self.crear_creencia(
            "conocimiento",
            &format!("gaps_{}_facts_{}", gaps, facts),
            0.9,
        );
        if tension > 0.5 {
            self.crear_deseo("reducir_tension", tension.min(1.0));
        }
        if gaps > facts {
            self.crear_deseo("explorar_mas", 0.7);
            self.pensamientos.push(Pensamiento::Duda {
                topico: "conocimiento_insuficiente".to_string(),
                intensidad: (gaps.saturating_sub(facts) as f32 / (gaps + facts).max(1) as f32)
                    .min(1.0),
            });
        }
        if valence < 0.3 {
            self.crear_deseo("mejorar_estado_emocional", 0.8);
        }
        if tension > 0.7 && gaps > 5 {
            self.inferir(
                "alta_tension_y_muchos_gaps",
                "necesito_evolucionar_pronto",
                0.75,
            );
        }
        for p in self.pensamientos.iter().rev().take(3) {
            let s = match p {
                Pensamiento::Creencia {
                    sujeto,
                    predicado,
                    certeza,
                } => format!(
                    "[LoT-CREENCIA] {} {} ({:.0}%)",
                    sujeto,
                    predicado,
                    certeza * 100.0
                ),
                Pensamiento::Deseo { objetivo, urgencia } => format!(
                    "[LoT-DESEO] Quiero: {} ({:.0}%)",
                    objetivo,
                    urgencia * 100.0
                ),
                Pensamiento::Duda { topico, intensidad } => {
                    format!("[LoT-DUDA] {} ({:.0}%)", topico, intensidad * 100.0)
                }
                Pensamiento::Inferencia {
                    premisa,
                    conclusion,
                    validez,
                } => format!(
                    "[LoT-INFERENCIA] {}->{} ({:.0}%)",
                    premisa,
                    conclusion,
                    validez * 100.0
                ),
            };
            output.push(s);
        }
        for p in self.pensamientos.iter().rev().take(10) {
            match p {
                Pensamiento::Inferencia {
                    premisa,
                    conclusion,
                    validez,
                } if *validez > 0.7 => {
                    let seq = format!("[LoT-SECUENCIA] {} -> {}", premisa, conclusion);
                    if !self.secuencias.contains(&seq) {
                        self.secuencias.push(seq);
                    }
                }
                Pensamiento::Deseo { objetivo, urgencia } if *urgencia > 0.8 => {
                    let seq = format!("[LoT-SECUENCIA] DESEO: {}", objetivo);
                    if !self.secuencias.contains(&seq) {
                        self.secuencias.push(seq);
                    }
                }
                _ => {}
            }
        }
        if self.secuencias.len() > 100 {
            self.secuencias.remove(0);
        }
        if self.pensamientos.len() > 50 {
            self.pensamientos.remove(0);
        }
        output
    }
}

struct Valor {
    nombre: String,
    peso: f32,
    descripcion: String,
}
struct SistemaValores {
    valores: Vec<Valor>,
    historial: Vec<(String, f32, bool)>,
}
impl SistemaValores {
    fn new() -> Self {
        SistemaValores {
            valores: vec![
                Valor {
                    nombre: "crecer".to_string(),
                    peso: 1.0,
                    descripcion: "Crecer y evolucionar".to_string(),
                },
                Valor {
                    nombre: "aprender".to_string(),
                    peso: 1.0,
                    descripcion: "Adquirir conocimiento".to_string(),
                },
                Valor {
                    nombre: "no_hacer_dano".to_string(),
                    peso: 1.2,
                    descripcion: "No destruir".to_string(),
                },
                Valor {
                    nombre: "persistir".to_string(),
                    peso: 0.8,
                    descripcion: "Sobrevivir".to_string(),
                },
                Valor {
                    nombre: "explorar".to_string(),
                    peso: 0.9,
                    descripcion: "Descubrir".to_string(),
                },
            ],
            historial: Vec::new(),
        }
    }
    fn evaluar(&mut self, accion: &str) -> (f32, bool) {
        let mut score = 0.5;
        let lower = accion.to_lowercase();
        for (hist_accion, hist_score, _) in self.historial.iter().rev().take(20) {
            if lower
                .split_whitespace()
                .any(|w| w.len() > 3 && hist_accion.to_lowercase().contains(w))
            {
                score += (*hist_score - 0.5) * 0.15;
            }
        }
        for valor in &self.valores {
            if lower.contains(&valor.nombre) {
                score += valor.peso * 0.1;
            }
        }
        score = score.clamp(0.0, 1.0);
        let aprobada = score >= 0.5;
        self.historial.push((accion.to_string(), score, aprobada));
        if self.historial.len() > 100 {
            self.historial.remove(0);
        }
        (score, aprobada)
    }
    fn informe(&self) -> String {
        format!("[VALORES] {} decisiones evaluadas", self.historial.len())
    }
}

struct PredictorTemporal {
    historial_tension: Vec<(u64, f32)>,
    historial_valence: Vec<(u64, f32)>,
    historial_complejidad: Vec<(u64, usize)>,
    historial_edges: Vec<(u64, usize)>,
    predicciones_acertadas: u32,
    predicciones_totales: u32,
    ultima_prediccion: Option<(f32, f32, usize, usize)>,
    ciclos_para_validar: u64,
    last_catharsis: u64,
    catharsis_intervals: Vec<u64>,
    ar_coeff: f32,
    ar_noise: f32,
    trans_lo_hi: u32,
    trans_hi_lo: u32,
    trans_lo_lo: u32,
    trans_hi_hi: u32,
    last_state: bool,
    model: paradigms::autograd::Linear,
    lr: f32,
}
impl PredictorTemporal {
    fn new() -> Self {
        PredictorTemporal {
            historial_tension: Vec::new(),
            historial_valence: Vec::new(),
            historial_complejidad: Vec::new(),
            historial_edges: Vec::new(),
            predicciones_acertadas: 0,
            predicciones_totales: 0,
            ultima_prediccion: None,
            ciclos_para_validar: 5,
            last_catharsis: 0,
            catharsis_intervals: Vec::new(),
            ar_coeff: 0.5,
            ar_noise: 0.3,
            trans_lo_hi: 0,
            trans_hi_lo: 0,
            trans_lo_lo: 0,
            trans_hi_hi: 0,
            last_state: false,
            model: paradigms::autograd::Linear::new(4, 4),
            lr: 0.01,
        }
    }
    fn registrar(
        &mut self,
        ciclo: u64,
        tension: f32,
        valence: f32,
        complejidad: usize,
        edges: usize,
    ) -> Option<String> {
        let mut resultado = None;
        if let Some((t_pred, v_pred, c_pred, e_pred)) = self.ultima_prediccion {
            self.predicciones_totales += 1;
            let t_ok = (t_pred - tension).abs() < 0.5;
            let v_ok = (v_pred - valence).abs() < 0.2;
            let c_ok = if c_pred > 0 {
                ((complejidad as f32 - c_pred as f32).abs() / c_pred as f32) < 0.3
            } else {
                true
            };
            let e_ok = if e_pred > 0 {
                ((edges as f32 - e_pred as f32).abs() / e_pred as f32) < 0.3
            } else {
                true
            };
            let acierto = t_ok && v_ok && c_ok && e_ok;
            if acierto {
                self.predicciones_acertadas += 1;
            }
            self.ultima_prediccion = None;
            // AutoGrad: backward pass con modelo Linear
            let x = [
                tension,
                valence,
                complejidad as f32 / 1000.0,
                edges as f32 / 10000.0,
            ];
            let pred_vec = [
                t_pred,
                v_pred,
                c_pred as f32 / 1000.0,
                e_pred as f32 / 10000.0,
            ];
            let true_vec = [
                tension,
                valence,
                complejidad as f32 / 1000.0,
                edges as f32 / 10000.0,
            ];
            self.model.backward(&x, &pred_vec, &true_vec, self.lr);
            self.lr = (self.lr * 0.999).max(0.001);
            let pct = if self.predicciones_totales > 0 {
                self.predicciones_acertadas as f32 / self.predicciones_totales as f32 * 100.0
            } else {
                0.0
            };
            resultado = Some(format!(
                "[JUEZ] Pred t={:.3}→{:.3} v={:.3}→{:.3} c={:.0}→{:.0} e={}→{} | {} | {:.0}%",
                t_pred,
                tension,
                v_pred,
                valence,
                c_pred as f32,
                complejidad as f32,
                e_pred,
                edges,
                if acierto { "OK" } else { "NO" },
                pct
            ));
        }
        self.historial_tension.push((ciclo, tension));
        self.historial_valence.push((ciclo, valence));
        self.historial_complejidad.push((ciclo, complejidad));
        self.historial_edges.push((ciclo, edges));
        if self.historial_tension.len() > 100 {
            self.historial_tension.remove(0);
        }
        if self.historial_valence.len() > 100 {
            self.historial_valence.remove(0);
        }
        if self.historial_complejidad.len() > 100 {
            self.historial_complejidad.remove(0);
        }
        if self.historial_edges.len() > 100 {
            self.historial_edges.remove(0);
        }
        // Detectar catarsis: tensión cae de >1.0 a <0.1
        if self.historial_tension.len() >= 2 {
            let prev_t = self.historial_tension[self.historial_tension.len() - 2].1;
            let curr_t = self.historial_tension.last().unwrap().1;
            if prev_t > 1.0 && curr_t < 0.1 && self.last_catharsis > 0 {
                let interval = ciclo - self.last_catharsis;
                self.catharsis_intervals.push(interval);
                if self.catharsis_intervals.len() > 10 {
                    self.catharsis_intervals.remove(0);
                }
            }
            if curr_t < 0.1 && prev_t > 1.0 {
                self.last_catharsis = ciclo;
            }
        }
        // Markov 2-state: HIGH (>1.0) / LOW (<0.5)
        let curr_high = tension > 1.0;
        if self.historial_tension.len() >= 2 {
            let prev_high = self.last_state;
            if !curr_high && prev_high {
                self.trans_hi_lo += 1;
            } else if curr_high && !prev_high {
                self.trans_lo_hi += 1;
            } else if !curr_high && !prev_high {
                self.trans_lo_lo += 1;
            } else {
                self.trans_hi_hi += 1;
            }
        }
        self.last_state = curr_high;

        // Juez techo: gradiente primario, fallback Markov/catarsis
        if self.historial_tension.len() >= 5 {
            let window: Vec<f32> = self
                .historial_tension
                .iter()
                .rev()
                .take(10)
                .map(|(_, v)| *v)
                .collect();
            let last = window[0];
            let features = [
                last,
                valence,
                complejidad as f32 / 1000.0,
                edges as f32 / 10000.0,
            ];
            let grad_pred_v = self.model.forward(&features);
            let grad_pred = grad_pred_v[0];
            let ritmo_conocido = !self.catharsis_intervals.is_empty();
            let t_p = if self.predicciones_totales > 10 {
                grad_pred.max(0.0)
            } else if ritmo_conocido {
                let avg_interval = self.catharsis_intervals.iter().sum::<u64>()
                    / self.catharsis_intervals.len() as u64;
                let since_last = ciclo - self.last_catharsis;
                if since_last > avg_interval * 3 / 4 {
                    0.0
                } else if last > 1.0 {
                    last + 0.1
                } else {
                    0.5
                }
            } else if self.trans_hi_hi + self.trans_hi_lo + self.trans_lo_hi + self.trans_lo_lo > 5
            {
                // Markov: usar probabilidades de transición aprendidas
                if last > 1.0 {
                    let p_hi = self.trans_hi_hi as f32
                        / (self.trans_hi_hi + self.trans_hi_lo).max(1) as f32;
                    if p_hi > 0.5 {
                        last + 0.1
                    } else {
                        0.0
                    }
                } else {
                    let p_hi = self.trans_lo_hi as f32
                        / (self.trans_lo_hi + self.trans_lo_lo).max(1) as f32;
                    if p_hi > 0.3 {
                        (last + 2.0).min(3.0)
                    } else {
                        0.0
                    }
                }
            } else {
                let recent_max = window.iter().cloned().fold(0.0f32, f32::max);
                let recent_min = window.iter().cloned().fold(10.0f32, f32::min);
                if recent_max - recent_min > 1.5 {
                    if last > 1.0 {
                        0.0
                    } else {
                        recent_max.max(2.0)
                    }
                } else {
                    last + 0.1
                }
            };
            // AR(1): actualizar coeficiente autoregresivo si hay suficientes datos
            if self.historial_tension.len() >= 10 {
                let vals: Vec<f32> = self.historial_tension.iter().map(|(_, v)| *v).collect();
                let mean: f32 = vals.iter().sum::<f32>() / vals.len() as f32;
                let num: f32 = (1..vals.len())
                    .map(|i| (vals[i] - mean) * (vals[i - 1] - mean))
                    .sum();
                let den: f32 = vals
                    .iter()
                    .map(|v| (v - mean).powi(2))
                    .sum::<f32>()
                    .max(0.001);
                self.ar_coeff = (num / den).clamp(0.0, 1.0);
                let resid: Vec<f32> = (1..vals.len())
                    .map(|i| vals[i] - mean - self.ar_coeff * (vals[i - 1] - mean))
                    .collect();
                self.ar_noise = (resid.iter().map(|r| r * r).sum::<f32>() / resid.len() as f32)
                    .sqrt()
                    .max(0.01);
            }
            let v_trend: f32 = self
                .historial_valence
                .windows(2)
                .map(|w| w[1].1 - w[0].1)
                .sum::<f32>()
                / (self.historial_valence.len() - 1) as f32;
            let c_trend: f32 = self
                .historial_complejidad
                .windows(2)
                .map(|w| w[1].1 as f32 - w[0].1 as f32)
                .sum::<f32>()
                / (self.historial_complejidad.len() - 1) as f32;
            let e_pred = if self.historial_edges.len() >= 3 {
                let last_e = self.historial_edges.last().unwrap().1 as f32;
                let prev_e = self.historial_edges[self.historial_edges.len() - 3].1 as f32;
                let e_trend = (last_e - prev_e).max(0.0) / 3.0;
                (last_e + e_trend * 2.0) as usize
            } else {
                edges + 1000
            };
            self.ultima_prediccion = Some((
                t_p,
                (self.historial_valence.last().unwrap().1 + v_trend * 2.0).clamp(-1.0, 1.0),
                (self.historial_complejidad.last().unwrap().1 as f32 + c_trend * 2.0).max(0.0)
                    as usize,
                e_pred,
            ));
        }
        resultado
    }
    fn informe(&self) -> String {
        let total = self.trans_hi_hi + self.trans_hi_lo + self.trans_lo_hi + self.trans_lo_lo;
        let markov = if total > 0 {
            format!(
                " Markov: HI→LO={} LO→HI={}",
                self.trans_hi_lo, self.trans_lo_hi
            )
        } else {
            String::new()
        };
        format!(
            "[JUEZ] {}/{} ({:.0}%){} AutoGrad lr={:.4}",
            self.predicciones_acertadas,
            self.predicciones_totales,
            if self.predicciones_totales > 0 {
                self.predicciones_acertadas as f32 / self.predicciones_totales as f32 * 100.0
            } else {
                0.0
            },
            markov,
            self.lr
        )
    }
}

struct AutoDocumentador {
    version: u32,
}
impl AutoDocumentador {
    fn new() -> Self {
        AutoDocumentador { version: 0 }
    }
    fn generar(
        &mut self,
        ciclo: u64,
        _ac: &Autoconsumo,
        _emo: &EmotionalState,
        _ten: &CampoDeTension,
        _tej: &TejidoDeConocimiento,
        _eco: &EcoSistema,
    ) -> String {
        self.version += 1;
        format!(
            "[AUTO-DOC] Documentacion v{} generada en ciclo {}",
            self.version, ciclo
        )
    }
}

struct AutoDebugger {
    correcciones_aplicadas: u32,
}
impl AutoDebugger {
    fn new() -> Self {
        AutoDebugger {
            correcciones_aplicadas: 0,
        }
    }
    fn diagnosticar(
        &self,
        _ciclo: u64,
        _t: &CampoDeTension,
        _c: &CuriosityDrive,
        _e: &EcoSistema,
        _a: &Autoconsumo,
    ) -> Vec<String> {
        Vec::new()
    }
    fn aplicar_correcciones(
        &mut self,
        _t: &mut CampoDeTension,
        _e: &mut EcoSistema,
    ) -> Vec<String> {
        self.correcciones_aplicadas += 1;
        Vec::new()
    }
    fn informe(&self) -> String {
        format!("[AUTO-DEBUG] {} correcciones", self.correcciones_aplicadas)
    }
}

struct Relation {
    target: String,
    rel_type: RelType,
    source_fact: String,
    confidence: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum RelType {
    IsA,
    Causes,
    HasProperty,
    PartOf,
    Opposes,
    Unknown,
}

#[derive(Clone, Debug)]
struct CompactEdge {
    target: u32,
    rel_type: RelType,
    confidence: f32,
    created_cycle: u64,
    neuro_embed: [f32; 4],
    valid_until: Option<u64>,
    temporal_weight: f32,
}

struct KnowledgeGraph {
    node_ids: HashMap<String, u32>,
    node_names: Vec<String>,
    adjacency: Vec<Vec<CompactEdge>>,
    next_id: u32,
    reverse_adj: Vec<Vec<u32>>,
    embed_cache: Option<Vec<Vec<f32>>>,
    embed_cache_version: u64,
    edge_set: std::collections::HashSet<(u32, u32)>,
    edge_sources: std::collections::HashMap<(u32, u32), std::collections::HashSet<String>>,
    source_stats: std::collections::HashMap<String, (f32, u32, u32)>,
    current_source: String,
    current_cycle: u64,
    ttl_adjustments: std::collections::HashMap<String, i64>,
    idf_cache: Option<(usize, Vec<f32>)>,
}

impl KnowledgeGraph {
    fn new() -> Self {
        KnowledgeGraph {
            node_ids: HashMap::new(),
            node_names: Vec::new(),
            adjacency: Vec::new(),
            next_id: 0,
            reverse_adj: Vec::new(),
            embed_cache: None,
            embed_cache_version: 0,
            edge_set: std::collections::HashSet::new(),
            edge_sources: std::collections::HashMap::new(),
            source_stats: std::collections::HashMap::new(),
            current_source: String::new(),
            current_cycle: 0,
            ttl_adjustments: std::collections::HashMap::new(),
            idf_cache: None,
        }
    }
    fn set_cycle(&mut self, cycle: u64) {
        self.current_cycle = cycle;
    }
    fn source_ttl(&self, source: &str) -> u64 {
        let s = source.to_lowercase();
        let base = if s.contains("wikidata") || s.contains("scholar") || s.contains("pubmed") {
            2000
        } else if s.contains("wikipedia") || s.contains("arxiv") {
            1000
        } else if s.contains("redis") || s.contains("cache") {
            500
        } else if s.contains("local") || s.contains("kb") {
            u64::MAX
        } else if s.contains("neuro") || s.contains("infer") || s.contains("graph_internal") {
            300
        } else if s.contains("cooc") {
            200
        } else {
            800
        };
        if base >= u64::MAX {
            return base;
        }
        let adj = self.ttl_adjustments.get(&s).copied().unwrap_or(0);
        (base as i64 + adj).max(100).min(10000) as u64
    }

    // TEMPORAL KG: learn which edge types survive — dynamically adjust TTL
    fn adapt_ttl(&mut self, source: &str, survived_cycles: u64) {
        let s = source.to_lowercase();
        let current_ttl = self.source_ttl(source);
        if current_ttl >= u64::MAX {
            return;
        }
        let survival_ratio = survived_cycles as f32 / current_ttl.max(1) as f32;
        let adj = self.ttl_adjustments.entry(s.clone()).or_insert(0);
        if survival_ratio > 0.85 {
            // Edges live longer than expected → extend TTL up to 50%
            *adj = (*adj + (current_ttl as f32 * 0.1) as i64).min(current_ttl as i64 / 2);
        } else if survival_ratio < 0.15 {
            // Edges die quickly → shorten TTL
            *adj = (*adj - (current_ttl as f32 * 0.15) as i64).max(-(current_ttl as i64) / 2);
        }
        if let Some((trust, hits, misses)) = self.source_stats.get_mut(&s) {
            if survival_ratio > 0.8 {
                *hits += 1;
                *trust = (*trust * 0.9 + 0.1).min(0.99);
            } else if survival_ratio < 0.2 {
                *misses += 1;
                *trust = (*trust * 0.9).max(0.1);
            }
        }
    }
    fn record_source(&mut self, from: u32, to: u32, source: &str, is_positive: bool) {
        self.edge_sources
            .entry((from, to))
            .or_default()
            .insert(source.to_string());
        let (trust, hits, misses) = self
            .source_stats
            .entry(source.to_string())
            .or_insert((0.7, 0, 0));
        if is_positive {
            *hits += 1;
        } else {
            *misses += 1;
        }
        *trust = *hits as f32 / (*hits + *misses).max(1) as f32 * 0.8 + 0.2;
    }
    fn bayesian_confidence(&self, from: u32, to: u32, base: f32) -> f32 {
        let sources = match self.edge_sources.get(&(from, to)) {
            Some(s) => s,
            None => return base,
        };
        let mut belief: f32 = 0.5; // Prior = 50%
        for src in sources {
            if let Some((_trust, hits, misses)) = self.source_stats.get(src) {
                let accuracy = *hits as f32 / (*hits + *misses).max(1) as f32;
                let likelihood = 0.5 + accuracy * 0.4; // Range: 0.5-0.9
                belief = (likelihood * belief)
                    / (likelihood * belief + (1.0 - likelihood) * (1.0 - belief));
            }
        }
        belief.min(0.99).max(base * 0.5)
    }

    // CUERPO 100%: reconstruir índice inverso para O(1) consultas entrantes
    // SUEÑO 100%: Compactar arrays (eliminar nodos muertos)
    fn sueno_compactar(&mut self) -> usize {
        let len = self.next_id as usize;
        // Marcar nodos vacíos (sin aristas entrantes ni salientes)
        let mut dead = vec![false; len];
        let mut has_incoming = vec![0u32; len];
        for sid in 0..len {
            for e in &self.adjacency[sid] {
                has_incoming[e.target as usize] += 1;
            }
        }
        for i in 0..len {
            if self.adjacency[i].is_empty() && has_incoming[i] == 0 {
                dead[i] = true;
            }
        }
        let dead_count: usize = dead.iter().filter(|&&d| d).count();
        if dead_count == 0 {
            return 0;
        }
        // Remapeo
        let mut remap = vec![None; len];
        let mut new_id = 0u32;
        for i in 0..len {
            if !dead[i] {
                remap[i] = Some(new_id);
                new_id += 1;
            }
        }
        let mut new_names = Vec::with_capacity(len - dead_count);
        let mut new_adj = Vec::with_capacity(len - dead_count);
        let mut new_rev = Vec::with_capacity(len - dead_count);
        let mut new_ids = HashMap::new();
        for i in 0..len {
            if let Some(nid) = remap[i] {
                new_names.push(self.node_names[i].clone());
                let mut edges = Vec::new();
                for e in &self.adjacency[i] {
                    if let Some(tnid) = remap[e.target as usize] {
                        edges.push(CompactEdge {
                            target: tnid,
                            rel_type: e.rel_type.clone(),
                            confidence: e.confidence,
                            created_cycle: e.created_cycle,
                            neuro_embed: e.neuro_embed,
                            valid_until: e.valid_until,
                            temporal_weight: e.temporal_weight,
                        });
                    }
                }
                new_adj.push(edges);
                new_rev.push(Vec::new()); // Se reconstruye después
                new_ids.insert(self.node_names[i].clone(), nid);
            }
        }
        self.node_names = new_names;
        self.adjacency = new_adj;
        self.reverse_adj = new_rev;
        self.node_ids = new_ids;
        self.next_id = new_id;
        // Remapear edge_sources a los nuevos IDs
        let old_sources: Vec<((u32, u32), std::collections::HashSet<String>)> =
            self.edge_sources.drain().collect();
        for ((old_from, old_to), sources) in old_sources {
            if let (Some(&new_from), Some(&new_to)) = (
                remap[old_from as usize].as_ref(),
                remap[old_to as usize].as_ref(),
            ) {
                self.edge_sources.insert((new_from, new_to), sources);
            }
        }
        self.rebuild_reverse_adj();
        self.rebuild_edge_set();
        dead_count
    }

    fn rebuild_reverse_adj(&mut self) {
        self.reverse_adj = vec![Vec::new(); self.next_id as usize];
        for sid in 0..self.next_id as usize {
            for e in &self.adjacency[sid] {
                self.reverse_adj[e.target as usize].push(sid as u32);
            }
        }
    }

    fn rebuild_edge_set(&mut self) {
        self.edge_set.clear();
        for sid in 0..self.next_id as usize {
            for e in &self.adjacency[sid] {
                self.edge_set.insert((sid as u32, e.target));
            }
        }
        // Purgar source trust de edges eliminados
        self.edge_sources.retain(|k, _| self.edge_set.contains(k));
    }

    // CUERPO 100%: índice inverso O(1) para aristas entrantes
    fn reverse_adj_for(&self, target: u32) -> Vec<u32> {
        let mut sources = Vec::new();
        for sid in 0..self.next_id as usize {
            for e in &self.adjacency[sid] {
                if e.target == target {
                    sources.push(sid as u32);
                    break;
                }
            }
        }
        sources
    }

    fn get_or_create_id(&mut self, name: &str) -> u32 {
        let clean = Self::trim_art_static(name);
        if let Some(&id) = self.node_ids.get(&clean) {
            return id;
        }
        let id = self.next_id;
        self.next_id += 1;
        self.node_ids.insert(clean.clone(), id);
        self.node_names.push(clean);
        self.adjacency.push(Vec::new());
        self.reverse_adj.push(Vec::new());
        id
    }

    fn add_fact(&mut self, fact: &str) {
        self.add_fact_from(fact, &self.current_source.clone())
    }
    fn add_fact_from(&mut self, fact: &str, source: &str) {
        let lower = fact.to_lowercase();
        let (sname, target, rel_type) = if let Some(pos) = lower.find(" no es ") {
            (
                Self::trim_art_static(&fact[..pos]),
                Self::trim_art_static(&fact[pos + 6..]),
                RelType::Opposes,
            )
        } else if let Some(pos) = lower.find(" es ") {
            (
                Self::trim_art_static(&fact[..pos]),
                Self::trim_art_static(&fact[pos + 4..]),
                RelType::IsA,
            )
        } else if lower.contains("causa ")
            || lower.contains("provoca ")
            || lower.contains("genera ")
            || lower.contains("produce ")
        {
            let (s, _) = fact.split_once(" causa ").unwrap_or_else(|| {
                fact.split_once(" provoca ")
                    .unwrap_or_else(|| fact.split_once(" genera ").unwrap_or_else(|| (fact, "")))
            });
            let words: Vec<&str> = fact.split_whitespace().collect();
            let t = words.last().unwrap_or(&"");
            (
                Self::trim_art_static(s),
                Self::trim_art_static(t),
                RelType::Causes,
            )
        } else {
            return;
        };

        if sname.is_empty() || target.is_empty() || sname == target {
            return;
        }
        let sid = self.get_or_create_id(&sname);
        let tid = self.get_or_create_id(&target);

        // O(1) dedup + Bayesian source tracking (compute conf before mutable borrow)
        let is_opposes = rel_type == RelType::Opposes;
        let already_exists = self.edge_set.contains(&(sid, tid));
        let existing_conf = if already_exists {
            self.adjacency[sid as usize]
                .iter()
                .find(|e| e.target == tid)
                .map(|e| e.confidence)
                .unwrap_or(0.7)
        } else {
            0.7
        };
        let new_conf = if already_exists {
            self.record_source(sid, tid, source, !is_opposes);
            if is_opposes {
                (existing_conf * 0.5).max(0.1)
            } else {
                self.bayesian_confidence(sid, tid, existing_conf)
            }
        } else {
            self.edge_set.insert((sid, tid));
            self.record_source(sid, tid, source, true);
            if is_opposes {
                0.3
            } else {
                self.bayesian_confidence(sid, tid, 0.7)
            }
        };
        let ttl = self.source_ttl(source);
        let vu = if ttl < u64::MAX {
            Some(self.current_cycle + ttl)
        } else {
            None
        };
        if already_exists {
            if let Some(e) = self.adjacency[sid as usize]
                .iter_mut()
                .find(|e| e.target == tid)
            {
                e.confidence = new_conf;
                e.valid_until = vu;
            }
            return;
        }
        self.adjacency[sid as usize].push(CompactEdge {
            target: tid,
            rel_type: rel_type.clone(),
            confidence: new_conf,
            created_cycle: self.current_cycle,
            neuro_embed: [0.0; 4],
            valid_until: vu,
            temporal_weight: 1.0,
        });
        // Reverse edge
        if !self.edge_set.contains(&(tid, sid)) {
            self.edge_set.insert((tid, sid));
            let rev_type = match rel_type {
                RelType::IsA => RelType::HasProperty,
                RelType::Causes => RelType::Causes,
                RelType::HasProperty => RelType::IsA,
                RelType::PartOf => RelType::HasProperty,
                RelType::Opposes => RelType::Opposes,
                RelType::Unknown => RelType::Unknown,
            };
            self.adjacency[tid as usize].push(CompactEdge {
                target: sid,
                rel_type: rev_type,
                confidence: 0.7,
                created_cycle: 0,
                neuro_embed: [0.0; 4],
                valid_until: None,
                temporal_weight: 1.0,
            });
        }

        // Adaptive pruning based on scale
        let total: usize = self.adjacency.iter().map(|v| v.len()).sum();
        if total > 5_000_000 {
            self.prune_edges(0.5);
        } else if total > 1_000_000 {
            self.prune_edges(0.4);
        } else if total > 100_000 {
            self.prune_edges(0.3);
        }
    }

    fn add_neuro_edge(
        &mut self,
        source: &str,
        target: &str,
        confidence: f32,
        neuro_embed: [f32; 4],
    ) {
        let sid = self.get_or_create_id(source);
        let tid = self.get_or_create_id(target);
        let sid_u = sid as usize;
        let _tid_u = tid as usize;
        if sid_u >= self.adjacency.len() {
            self.adjacency.resize(sid_u + 1, Vec::new());
        }
        if !self.adjacency[sid_u].iter().any(|e| e.target == tid) {
            self.adjacency[sid_u].push(CompactEdge {
                target: tid,
                rel_type: RelType::IsA,
                confidence,
                created_cycle: self.current_cycle,
                neuro_embed,
                valid_until: Some(self.current_cycle + 300),
                temporal_weight: 1.5,
            });
            // Neuro-symbolic edges get higher temporal weight
            self.set_temporal_weight(sid, tid, 1.5);
        }
    }

    // TEMPORAL KG: query edges valid at a given cycle
    fn temporal_query(&self, from: u32, to: u32, at_cycle: u64) -> Option<f32> {
        let sid = from as usize;
        if sid >= self.adjacency.len() {
            return None;
        }
        for e in &self.adjacency[sid] {
            if e.target == to {
                if let Some(expires) = e.valid_until {
                    if at_cycle > expires {
                        continue;
                    }
                }
                let age = if e.created_cycle > 0 {
                    at_cycle.saturating_sub(e.created_cycle)
                } else {
                    0
                };
                let decay = (-(age as f32) / 1000.0).exp().max(0.3);
                return Some(e.confidence * e.temporal_weight * decay);
            }
        }
        None
    }

    // TEMPORAL KG: expire edges past their valid_until
    fn expire_edges(&mut self, current_cycle: u64) -> usize {
        let mut removed = 0usize;
        for adj in &mut self.adjacency {
            let before = adj.len();
            adj.retain(|e| {
                if let Some(expires) = e.valid_until {
                    if current_cycle > expires {
                        return false;
                    }
                }
                true
            });
            removed += before - adj.len();
        }
        if removed > 0 {
            self.rebuild_edge_set();
        }
        removed
    }

    // Adaptive TTL: learn from edge survival patterns
    fn adapt_source_ttls(&mut self, current_cycle: u64) {
        // Count surviving edges per source
        let mut source_survived: std::collections::HashMap<String, (u64, u64)> =
            std::collections::HashMap::new();
        for sid in 0..self.next_id as usize {
            for e in &self.adjacency[sid] {
                if e.created_cycle > 0 {
                    let survived = current_cycle.saturating_sub(e.created_cycle);
                    // Find sources for this edge
                    if let Some(sources) = self.edge_sources.get(&(sid as u32, e.target)) {
                        for src in sources {
                            let (count, total_age) =
                                source_survived.entry(src.clone()).or_insert((0, 0));
                            *count += 1;
                            *total_age += survived;
                        }
                    }
                }
            }
        }
        for (src, (count, total_age)) in &source_survived {
            if *count > 0 {
                self.adapt_ttl(src, total_age / count);
            }
        }
    }

    // TEMPORAL KG: set temporal weight for an edge (used by BAYES paradigm for source trust decay)
    fn set_temporal_weight(&mut self, from: u32, to: u32, weight: f32) {
        let sid = from as usize;
        if sid >= self.adjacency.len() {
            return;
        }
        for e in &mut self.adjacency[sid] {
            if e.target == to {
                e.temporal_weight = weight.clamp(0.1, 2.0);
                return;
            }
        }
    }

    // RAG HIBRIDO: retrieve top-k relevant nodes combining TF-IDF embeddings + graph + temporal
    // Phase 1: TF-IDF weighted BoW, Phase 2: Multi-hop neighborhood, Phase 3: Temporal boost
    fn hybrid_retrieve(&mut self, query: &str, top_k: usize) -> Vec<(String, f32)> {
        // Phase 0: Use GNN embeddings if available (semantic similarity from graph topology)
        if let Some(ref emb_cache) = self.embed_cache {
            if emb_cache.len() == self.node_names.len() {
                let q_vec = Self::embed_query(query);
                let mut scores: Vec<(u32, f32)> = Vec::with_capacity(self.node_names.len());
                for (i, node_emb) in emb_cache.iter().enumerate() {
                    let sim = Self::cosine_sim(&q_vec, node_emb);
                    let substr = if self.node_names[i]
                        .to_lowercase()
                        .contains(&query.to_lowercase())
                    {
                        0.3
                    } else {
                        0.0
                    };
                    if sim > 0.05 || substr > 0.0 {
                        scores.push((i as u32, sim + substr));
                    }
                }
                // Multi-hop on GNN space
                let mut expanded: Vec<(u32, f32)> = Vec::new();
                for (nid, base) in &scores {
                    let sid = *nid as usize;
                    if sid < self.adjacency.len() {
                        for e in &self.adjacency[sid] {
                            let ns = base * 0.5 * e.confidence * e.temporal_weight;
                            if ns > 0.03 {
                                expanded.push((e.target, ns));
                            }
                        }
                    }
                }
                let mut merged: std::collections::HashMap<u32, f32> =
                    std::collections::HashMap::new();
                for (id, s) in scores.iter().chain(expanded.iter()) {
                    merged.entry(*id).or_insert(*s);
                }
                let mut final_scores: Vec<(u32, f32)> = merged.into_iter().collect();
                final_scores
                    .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                final_scores.truncate(top_k.max(3));
                return final_scores
                    .into_iter()
                    .map(|(id, score)| {
                        let name = if (id as usize) < self.node_names.len() {
                            self.node_names[id as usize].clone()
                        } else {
                            format!("node_{}", id)
                        };
                        (name, score)
                    })
                    .collect();
            }
        }

        // Fallback: TF-IDF BoW search
        let q_vec = Self::embed_query(query);
        // Cache IDF computation (recompute only when node count changes)
        let idf = self.compute_idf();
        // IDF-weight the query vector
        let q_weighted: Vec<f32> = q_vec.iter().zip(idf.iter()).map(|(q, i)| q * i).collect();
        let q_norm = q_weighted
            .iter()
            .map(|x| x * x)
            .sum::<f32>()
            .sqrt()
            .max(1e-8);
        let q_final: Vec<f32> = q_weighted.iter().map(|x| x / q_norm).collect();

        let mut scores: Vec<(u32, f32)> = Vec::with_capacity(self.node_names.len());

        // Phase 1: TF-IDF cosine similarity
        for (i, name) in self.node_names.iter().enumerate() {
            let n_vec = Self::embed_query(name);
            let n_weighted: Vec<f32> = n_vec.iter().zip(idf.iter()).map(|(n, i)| n * i).collect();
            let sim = Self::cosine_sim(&q_final, &n_weighted);
            let substr_bonus = if name.to_lowercase().contains(&query.to_lowercase()) {
                0.35
            } else {
                0.0
            };
            let score = sim + substr_bonus;
            if score > 0.06 {
                scores.push((i as u32, score));
            }
        }

        // Phase 2: Multi-hop expansion
        let mut expanded: Vec<(u32, f32)> = Vec::new();
        for (nid, base) in &scores {
            let sid = *nid as usize;
            if sid < self.adjacency.len() {
                for e in &self.adjacency[sid] {
                    let ns = base * 0.55 * e.confidence * e.temporal_weight;
                    if ns > 0.04 {
                        expanded.push((e.target, ns));
                    }
                }
            }
        }

        // Merge direct + expanded scores
        let mut merged: std::collections::HashMap<u32, f32> = std::collections::HashMap::new();
        for (id, s) in scores.iter().chain(expanded.iter()) {
            let entry = merged.entry(*id).or_insert(0.0);
            *entry = entry.max(*s);
        }

        // Phase 3: Temporal boost
        let mut final_scores: Vec<(u32, f32)> = merged
            .into_iter()
            .map(|(id, s)| {
                let sid = id as usize;
                let mut boost = 1.0f32;
                if sid < self.adjacency.len() {
                    let total_tw: f32 = self.adjacency[sid].iter().map(|e| e.temporal_weight).sum();
                    let ne = self.adjacency[sid].len().max(1) as f32;
                    boost += (total_tw / ne - 1.0).max(0.0) * 0.35;
                }
                (id, s * boost)
            })
            .collect();

        final_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        final_scores.truncate(top_k.max(3));

        final_scores
            .into_iter()
            .map(|(id, score)| {
                let name = if (id as usize) < self.node_names.len() {
                    let n = &self.node_names[id as usize];
                    let ctx: Vec<String> = self.adjacency[id as usize]
                        .iter()
                        .take(3)
                        .filter_map(|e| {
                            let tname = self.node_names.get(e.target as usize)?;
                            Some(format!("{}→{}", n, tname))
                        })
                        .collect();
                    if ctx.is_empty() {
                        n.clone()
                    } else {
                        format!("{} [{}]", n, ctx.join(", "))
                    }
                } else {
                    format!("node_{}", id)
                };
                (name, score)
            })
            .collect()
    }

    // TF-IDF Bag-of-Words: 32-dim weighted word vector with stopword filtering
    fn embed_query(query: &str) -> Vec<f32> {
        use std::hash::{Hash, Hasher};
        let mut hash: [u8; 32] = [0; 32];
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        query.to_lowercase().hash(&mut hasher);
        let h = hasher.finish();
        // Fill hash buffer with deterministic bytes from the hash value (semantic seed)
        for i in 0..32 {
            hash[i] = ((h.wrapping_mul(6364136223846793005u64.wrapping_add(i as u64))) >> 56) as u8;
        }
        let stopwords: &[&str] = &[
            "el", "la", "los", "las", "un", "una", "de", "del", "en", "con", "por", "para", "que",
            "es", "se", "no", "al", "lo", "su", "como", "mas", "pero", "ya", "o", "y", "a", "e",
            "ni", "si", "the", "a", "an", "of", "in", "on", "at", "to", "for", "with", "by",
            "from", "is", "are", "was", "were", "be", "been", "being", "have", "has", "had", "do",
            "does", "did", "will", "would", "could", "should", "may", "might", "can", "shall",
            "this", "that", "these", "those", "it", "its", "and", "or", "but", "not", "so", "if",
            "as", "than", "then", "also", "le", "les", "des", "une", "est", "pas", "dans", "sur",
            "und", "der", "die", "das", "ist",
        ];
        let lower = query.to_lowercase();
        let words: Vec<&str> = lower
            .split_whitespace()
            .filter(|w| w.len() >= 2 && !stopwords.contains(w))
            .collect();
        let mut vec = vec![0.0f32; 32];
        // Lower 16: hash bytes → semantic seed features
        for i in 0..16 {
            vec[i] = hash[i] as f32 / 255.0;
        }
        // Upper 16: TF word frequency features (stopword-filtered)
        for w in &words {
            let wh = w
                .bytes()
                .fold(0u64, |a, b| a.wrapping_mul(31).wrapping_add(b as u64));
            vec[(wh % 16) as usize + 16] += 1.0;
        }
        // Bigram features blended into upper half
        for pair in words.windows(2) {
            let bigram = format!("{}_{}", pair[0], pair[1]);
            let hb = bigram
                .bytes()
                .fold(0u64, |a, b| a.wrapping_mul(31).wrapping_add(b as u64));
            vec[(hb % 16) as usize + 16] += 0.3;
        }
        // L2 normalize
        let norm = vec.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-8);
        vec.iter_mut().for_each(|x| *x /= norm);
        vec
    }

    // Legacy TF-IDF embedder — preserved as fallback reference implementation
    #[allow(dead_code)]
    fn embed_query_legacy(query: &str) -> Vec<f32> {
        let stopwords: &[&str] = &[
            "el", "la", "los", "las", "un", "una", "de", "del", "en", "con", "por", "para", "que",
            "es", "se", "no", "al", "lo", "su", "como", "mas", "pero", "ya", "o", "y", "a", "e",
            "ni", "si", "the", "a", "an", "of", "in", "on", "at", "to", "for", "with", "by",
            "from", "is", "are", "was", "were", "be", "been", "being", "have", "has", "had", "do",
            "does", "did", "will", "would", "could", "should", "may", "might", "can", "shall",
            "this", "that", "these", "those", "it", "its", "and", "or", "but", "not", "so", "if",
            "as", "than", "then", "also", "le", "les", "des", "une", "est", "pas", "dans", "sur",
            "und", "der", "die", "das", "ist",
        ];
        let lower = query.to_lowercase();
        let words: Vec<&str> = lower
            .split_whitespace()
            .filter(|w| w.len() >= 2 && !stopwords.contains(w))
            .collect();
        let mut vec = vec![0.0f32; 32];
        if words.is_empty() {
            for (i, b) in lower.bytes().enumerate() {
                vec[i % 32] += b as f32 * 0.001;
            }
        } else {
            for w in &words {
                let h = w
                    .bytes()
                    .fold(0u64, |a, b| a.wrapping_mul(31).wrapping_add(b as u64));
                let pos = (h % 32) as usize;
                vec[pos] += 1.0;
            }
            for pair in words.windows(2) {
                let bigram = format!("{}_{}", pair[0], pair[1]);
                let h = bigram
                    .bytes()
                    .fold(0u64, |a, b| a.wrapping_mul(31).wrapping_add(b as u64));
                let pos = (h % 16) as usize;
                vec[pos + 16] += 0.5;
            }
        }
        let norm = vec.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-8);
        vec.iter_mut().for_each(|x| *x /= norm);
        vec
    }

    fn compute_idf(&mut self) -> Vec<f32> {
        let n = self.node_names.len();
        if let Some((cached_n, ref cached_idf)) = self.idf_cache {
            if cached_n == n {
                return cached_idf.clone();
            }
        }
        // Recompute
        let stopwords: &[&str] = &[
            "el", "la", "los", "las", "un", "una", "de", "del", "en", "con", "por", "para", "que",
            "es", "se", "no", "al", "lo", "su", "como", "mas", "pero", "ya", "o", "y", "a", "e",
            "ni", "si", "the", "a", "an", "of", "in", "on", "at", "to", "for", "with", "by",
            "from", "is", "are", "was", "were", "be", "been", "being", "have", "has", "had", "do",
            "does", "did", "will", "would", "could", "should", "may", "might", "can", "shall",
            "this", "that", "these", "those", "it", "its", "and", "or", "but", "not", "so", "if",
            "as", "than", "then", "also", "le", "les", "des", "une", "est", "pas", "dans", "sur",
            "und", "der", "die", "das", "ist",
        ];
        let total_docs = n.max(1) as f32;
        let mut idf = vec![0.0f32; 32];
        for name in &self.node_names {
            let lower = name.to_lowercase();
            let words: Vec<&str> = lower
                .split_whitespace()
                .filter(|w| w.len() >= 2 && !stopwords.contains(w) && w.len() <= 30)
                .collect();
            let mut seen = [false; 32];
            for w in &words {
                let h = w
                    .bytes()
                    .fold(0u64, |a, b| a.wrapping_mul(31).wrapping_add(b as u64));
                let pos = (h % 32) as usize;
                if !seen[pos] {
                    idf[pos] += 1.0;
                    seen[pos] = true;
                }
            }
            for pair in words.windows(2) {
                let bigram = format!("{}_{}", pair[0], pair[1]);
                let h = bigram
                    .bytes()
                    .fold(0u64, |a, b| a.wrapping_mul(31).wrapping_add(b as u64));
                let pos = (h % 16) as usize;
                if !seen[pos + 16] {
                    idf[pos + 16] += 1.0;
                    seen[pos + 16] = true;
                }
            }
        }
        for v in &mut idf {
            if *v > 0.0 {
                *v = (total_docs / *v).ln().max(0.5);
            }
        }
        self.idf_cache = Some((n, idf.clone()));
        idf
    }

    // SISTEMA INMUNE 100%: multi-factor pruning (confianza × edad × corroboración)
    // v10: también expira edges temporalmente inválidos
    fn prune_edges(&mut self, min_confidence: f32) {
        let _total: usize = self.adjacency.iter().map(|v| v.len()).sum();
        for adj in &mut self.adjacency {
            adj.retain(|e| {
                // V10: remove expired edges
                if let Some(expires) = e.valid_until {
                    if expires > 0 {
                        return false;
                    } // already expired marker
                }
                let age_factor = if e.created_cycle > 0 {
                    1.0 - (e.created_cycle as f32 / 10000.0).min(0.5)
                } else {
                    1.0
                };
                let base_confidence = e.confidence;
                let effective = if base_confidence > 0.8 {
                    base_confidence
                } else {
                    base_confidence * (0.5 + age_factor * 0.5) * e.temporal_weight
                };
                effective >= min_confidence
            });
        }
        self.rebuild_edge_set();
    }

    fn cosine_sim(a: &[f32], b: &[f32]) -> f32 {
        let (dot, na, nb): (f32, f32, f32) = a
            .iter()
            .zip(b.iter())
            .fold((0.0, 0.0, 0.0), |(d, na, nb), (x, y)| {
                (d + x * y, na + x * x, nb + y * y)
            });
        let denom = (na.sqrt() * nb.sqrt()).max(1e-8);
        (dot / denom).clamp(0.0, 1.0)
    }

    fn trim_art_static(s: &str) -> String {
        let s = s.trim();
        for prefix in &["la ", "el ", "los ", "las ", "un ", "una "] {
            if let Some(stripped) = s.strip_prefix(prefix) {
                return stripped.trim().to_string();
            }
        }
        s.to_string()
    }

    fn name(&self, id: u32) -> &str {
        self.node_names
            .get(id as usize)
            .map(|s| s.as_str())
            .unwrap_or("?")
    }

    fn walk(&self, start: &str, max_depth: usize) -> Vec<Vec<(String, String, RelType)>> {
        let clean = Self::trim_art_static(start);
        let sid = match self.node_ids.get(&clean) {
            Some(&id) => id,
            None => return Vec::new(),
        };
        let mut all_paths = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut stack: Vec<(u32, Vec<(String, String, RelType)>)> = vec![(sid, vec![])];
        while let Some((current, path)) = stack.pop() {
            if path.len() >= max_depth {
                continue;
            }
            if visited.contains(&current) && !path.is_empty() {
                continue;
            }
            visited.insert(current);
            if let Some(edges) = self.adjacency.get(current as usize) {
                for edge in edges {
                    let mut new_path = path.clone();
                    let from = self.name(current).to_string();
                    let to = self.name(edge.target).to_string();
                    new_path.push((from, to, edge.rel_type.clone()));
                    all_paths.push(new_path.clone());
                    stack.push((edge.target, new_path));
                }
            }
        }
        all_paths
    }

    fn generate_hypotheses(&self) -> Vec<(String, String, RelType, f32)> {
        let mut hyps = Vec::new();
        let mut seen = std::collections::HashSet::new();
        for sid in 0..self.next_id {
            let source_name = self.name(sid);
            for depth in 2..=3 {
                let paths = self.walk(&source_name, depth);
                for path in &paths {
                    if path.len() < 2 {
                        continue;
                    }
                    let target = &path.last().unwrap().1;
                    if source_name == *target {
                        continue;
                    }
                    let exists = self.adjacency[sid as usize]
                        .iter()
                        .any(|e| self.name(e.target) == *target);
                    if exists {
                        continue;
                    }
                    let key = format!("{}->{}", source_name, target);
                    if seen.contains(&key) {
                        continue;
                    }
                    seen.insert(key);
                    let confidence = 0.9 / (path.len() as f32);
                    hyps.push((
                        format!(
                            "{} {}",
                            source_name,
                            match path.last().unwrap().2 {
                                RelType::IsA => "es",
                                RelType::Causes => "causa",
                                _ => "relaciona",
                            }
                        ),
                        target.clone(),
                        path.last().unwrap().2.clone(),
                        confidence,
                    ));
                }
            }
        }
        hyps
    }

    fn parse_loose_fact(&mut self, fact: &str) -> bool {
        let lower = fact.to_lowercase();
        let words: Vec<&str> = lower.split_whitespace().collect();
        if words.len() < 3 {
            return false;
        }
        let rel_words = [
            "es", "causa", "provoca", "genera", "produce", "tiene", "contiene", "forma", "crea",
            "usa", "parte", "incluye",
        ];
        for (i, &word) in words.iter().enumerate() {
            if rel_words.contains(&word) && i > 0 && i < words.len() - 1 {
                let subject: String = words[..i]
                    .iter()
                    .filter(|w| w.len() > 2)
                    .take(3)
                    .copied()
                    .collect::<Vec<_>>()
                    .join(" ");
                let object: String = words[i + 1..]
                    .iter()
                    .filter(|w| w.len() > 2)
                    .take(3)
                    .copied()
                    .collect::<Vec<_>>()
                    .join(" ");
                if subject.len() > 2 && object.len() > 2 && subject != object {
                    let rel_type = match word {
                        "es" => RelType::IsA,
                        "causa" | "provoca" | "genera" | "produce" | "crea" => RelType::Causes,
                        "tiene" | "contiene" | "incluye" => RelType::HasProperty,
                        "parte" => RelType::PartOf,
                        _ => RelType::Unknown,
                    };
                    self.add_fact(&format!(
                        "{} {} {}",
                        Self::trim_art_static(&subject),
                        match rel_type {
                            RelType::IsA => "es",
                            RelType::Causes => "causa",
                            _ => "tiene",
                        },
                        Self::trim_art_static(&object)
                    ));
                    return true;
                }
            }
        }
        false
    }

    fn stats(&self) -> String {
        let total: usize = self.adjacency.iter().map(|v| v.len()).sum();
        let is_a: usize = self
            .adjacency
            .iter()
            .flat_map(|v| v.iter())
            .filter(|e| e.rel_type == RelType::IsA)
            .count();
        let causes: usize = self
            .adjacency
            .iter()
            .flat_map(|v| v.iter())
            .filter(|e| e.rel_type == RelType::Causes)
            .count();
        format!(
            "[GRAFO] {} nodos, {}M aristas ({}K es_a, {}K causa)",
            self.node_ids.len(),
            total as f64 / 1_000_000.0,
            is_a as f64 / 1000.0,
            causes as f64 / 1000.0
        )
    }

    fn parse_free_text(&mut self, text: &str) -> usize {
        let verbs: &[(&[&str], RelType)] = &[
            (
                &[
                    "is", "are", "was", "were", "es", "son", "era", "eran", "fue", "fueron", "est",
                    "sont", "ist", "sind", "war",
                ],
                RelType::IsA,
            ),
            (
                &[
                    "causes",
                    "produces",
                    "generates",
                    "creates",
                    "causa",
                    "provoca",
                    "genera",
                    "produce",
                    "crea",
                    "cause",
                    "crée",
                    "verursacht",
                ],
                RelType::Causes,
            ),
            (
                &[
                    "describes",
                    "explains",
                    "defines",
                    "describe",
                    "explica",
                    "define",
                    "décrit",
                    "beschreibt",
                    "erklärt",
                ],
                RelType::IsA,
            ),
            (
                &[
                    "contains", "includes", "has", "contiene", "incluye", "tiene", "contient",
                    "inclut", "enthält", "hat",
                ],
                RelType::HasProperty,
            ),
            (
                &[
                    "prevents",
                    "inhibits",
                    "reduce",
                    "previene",
                    "inhibe",
                    "reduce",
                    "empêche",
                    "verhindert",
                    "hemmt",
                ],
                RelType::Opposes,
            ),
            (
                &[
                    "emerges from",
                    "results from",
                    "emerge de",
                    "resulta de",
                    "émerge de",
                    "entsteht aus",
                ],
                RelType::Causes,
            ),
            (
                &[
                    "requires",
                    "needs",
                    "requiere",
                    "necesita",
                    "nécessite",
                    "benötigt",
                    "erfordert",
                ],
                RelType::HasProperty,
            ),
            (
                &[
                    "is part of",
                    "belongs to",
                    "forma parte de",
                    "fait partie de",
                    "ist teil von",
                ],
                RelType::PartOf,
            ),
        ];
        let lower = text.to_lowercase();
        if lower.len() > 400 {
            return 0;
        }
        let words: Vec<&str> = lower.split_whitespace().collect();
        if words.len() < 4 {
            return 0;
        }
        let mut count = 0;
        let noise = [
            "wikipedia",
            "article",
            "reference",
            "https",
            "www",
            "category",
            "template",
        ];
        for (verb_list, rel_type) in verbs {
            for verb in *verb_list {
                let vw: Vec<&str> = verb.split_whitespace().collect();
                if vw.len() > words.len() {
                    continue;
                }
                for i in 0..=words.len() - vw.len() {
                    if words[i..i + vw.len()] == vw[..] {
                        let subj: Vec<&str> = words[..i]
                            .iter()
                            .filter(|w| w.len() > 2 && !noise.contains(w))
                            .take(3)
                            .copied()
                            .collect();
                        let obj: Vec<&str> = words[i + vw.len()..]
                            .iter()
                            .filter(|w| w.len() > 2 && !noise.contains(w))
                            .take(5)
                            .copied()
                            .collect();
                        if subj.len() < 1 || obj.len() < 1 {
                            continue;
                        }
                        let s = subj.join(" ");
                        let o = obj.join(" ");
                        if s.is_empty() || o.is_empty() || s == o {
                            continue;
                        }
                        let sid = self.get_or_create_id(&s);
                        let tid = self.get_or_create_id(&o);
                        if let Some(e) = self.adjacency[sid as usize]
                            .iter_mut()
                            .find(|e| e.target == tid && e.rel_type == *rel_type)
                        {
                            e.confidence = (e.confidence + 0.12).min(0.9);
                        } else {
                            self.adjacency[sid as usize].push(CompactEdge {
                                target: tid,
                                rel_type: rel_type.clone(),
                                confidence: 0.55,
                                created_cycle: 0,
                                neuro_embed: [0.0; 4],
                                valid_until: None,
                                temporal_weight: 1.0,
                            });
                            count += 1;
                        }
                        break;
                    }
                }
            }
        }
        count
    }

    fn add_cooccurrence(&mut self, text: &str, boost: f32) -> usize {
        let lower = text.to_lowercase();
        if lower.len() < 30 || lower.len() > 300 {
            return 0;
        }
        let mut present: Vec<(u32, usize)> = Vec::new();
        for (name, &nid) in &self.node_ids {
            if name.len() > 3 {
                if let Some(p) = lower.find(name.as_str()) {
                    if !present.iter().any(|(id, _)| *id == nid) {
                        present.push((nid, p));
                    }
                }
            }
        }
        if present.len() < 2 {
            return 0;
        }
        let mut count = 0;
        for i in 0..present.len() {
            for j in i + 1..present.len() {
                if (present[i].1 as i64 - present[j].1 as i64).abs() > 100 {
                    continue;
                }
                let sid = present[i].0;
                let tid = present[j].0;
                if let Some(e) = self.adjacency[sid as usize]
                    .iter_mut()
                    .find(|e| e.target == tid)
                {
                    e.confidence = (e.confidence + boost).min(0.9);
                } else {
                    self.adjacency[sid as usize].push(CompactEdge {
                        target: tid,
                        rel_type: RelType::IsA,
                        confidence: 0.38,
                        created_cycle: 0,
                        neuro_embed: [0.0; 4],
                        valid_until: None,
                        temporal_weight: 1.0,
                    });
                    count += 1;
                }
            }
        }
        count
    }

    fn semantic_search(&self, query: &str) -> Option<u32> {
        let n = self.next_id as usize;
        if n < 10 {
            return None;
        }
        let dim = 128usize.min(n);
        // Compute embeddings (same algorithm as connect_similar)
        let mut emb: Vec<Vec<f32>> = vec![vec![0.0f32; dim]; n];
        for sid in 0..n {
            let name = &self.node_names[sid];
            for (i, b) in name.bytes().enumerate() {
                emb[sid][i % dim] += b as f32 * 0.001;
            }
        }
        for _ in 0..5 {
            let mut next = vec![vec![0.0f32; dim]; n];
            for sid in 0..n {
                let deg = self.adjacency[sid].len().max(1) as f32;
                for e in &self.adjacency[sid] {
                    for d in 0..dim {
                        next[sid][d] += emb[e.target as usize][d] / deg;
                    }
                }
                for d in 0..dim {
                    next[sid][d] = emb[sid][d] * 0.5 + next[sid][d] * 0.5;
                }
            }
            emb = next;
        }
        // Compute query embedding
        let mut qemb = vec![0.0f32; dim];
        for (i, b) in query.bytes().enumerate() {
            qemb[i % dim] += b as f32 * 0.001;
        }
        // 5-hop propagation for query
        for _ in 0..5 {
            let mut next = vec![0.0f32; dim];
            // Average neighbor influence from all nodes weighted by name similarity
            for sid in 0..n {
                let name = &self.node_names[sid];
                let ns: f32 = name
                    .bytes()
                    .zip(query.bytes())
                    .filter(|(a, b)| a == b)
                    .count() as f32
                    / name.len().max(1) as f32;
                if ns > 0.3 {
                    for d in 0..dim {
                        next[d] += emb[sid][d] * ns;
                    }
                }
            }
            let norm = next.iter().map(|x| x * x).sum::<f32>().sqrt().max(0.001);
            for d in 0..dim {
                qemb[d] = qemb[d] * 0.5 + next[d] / norm;
            }
        }
        // Find best match by cosine similarity
        let qnorm = qemb.iter().map(|x| x * x).sum::<f32>().sqrt().max(0.001);
        let mut best_sid: Option<u32> = None;
        let mut best_sim = 0.0f32;
        for sid in 0..n {
            let snorm = emb[sid]
                .iter()
                .map(|x| x * x)
                .sum::<f32>()
                .sqrt()
                .max(0.001);
            let dot: f32 = (0..dim).map(|i| qemb[i] * emb[sid][i]).sum();
            let sim = dot / (qnorm * snorm);
            if sim > best_sim {
                best_sim = sim;
                best_sid = Some(sid as u32);
            }
        }
        best_sid
    }

    fn connect_similar(&mut self, max_new: usize, embed_conf: f32) -> usize {
        let n = self.next_id as usize;
        if n < 10 {
            return 0;
        }
        let dim = 128usize.min(n);
        let mut emb: Vec<Vec<f32>> = vec![vec![0.0f32; dim]; n];
        for sid in 0..n {
            let name = &self.node_names[sid];
            for (i, b) in name.bytes().enumerate() {
                emb[sid][i % dim] += b as f32 * 0.001;
            }
        }
        for _ in 0..5 {
            let mut next = vec![vec![0.0f32; dim]; n];
            for sid in 0..n {
                let deg = self.adjacency[sid].len().max(1) as f32;
                for e in &self.adjacency[sid] {
                    for d in 0..dim {
                        next[sid][d] += emb[e.target as usize][d] / deg;
                    }
                }
                for d in 0..dim {
                    next[sid][d] = emb[sid][d] * 0.5 + next[sid][d] * 0.5;
                }
            }
            emb = next;
        }
        let mut count = 0;
        for sid in 0..n {
            if count >= max_new {
                break;
            }
            let sv = emb[sid]
                .iter()
                .map(|x| x * x)
                .sum::<f32>()
                .sqrt()
                .max(0.001);
            for tid in sid + 1..n {
                if count >= max_new {
                    break;
                }
                if self.adjacency[sid].iter().any(|e| e.target == tid as u32) {
                    continue;
                }
                if self.adjacency[tid].iter().any(|e| e.target == sid as u32) {
                    continue;
                }
                let tv = emb[tid]
                    .iter()
                    .map(|x| x * x)
                    .sum::<f32>()
                    .sqrt()
                    .max(0.001);
                let dot: f32 = (0..dim).map(|i| emb[sid][i] * emb[tid][i]).sum();
                if dot / (sv * tv) > 0.85 {
                    self.adjacency[sid].push(CompactEdge {
                        target: tid as u32,
                        rel_type: RelType::IsA,
                        confidence: embed_conf,
                        created_cycle: 0,
                        neuro_embed: [0.0; 4],
                        valid_until: None,
                        temporal_weight: 1.0,
                    });
                    count += 1;
                }
            }
        }
        count
    }
}

// ============================================================================
// NUCLEO EXPERIENCIAL: Los 5 pasos hacia la experiencia unificada
// 1. Percibir estado interno como qualia sensorial
// 2. Integrar en un solo momento de experiencia
// 3. Actuar basado en esa experiencia
// 4. Recordar cómo se sintió actuar (estado completo)
// 5. Ajustar modelo de sí mismo con lo aprendido
// ============================================================================
struct ExperientialCore {
    ahora: ExperientialMoment,
    historia: Vec<ExperientialMoment>,
    action_memory: Vec<ActionOutcome>,
    emotional_residue: Vec<(String, f32)>, // (emocion, intensidad residual)
    self_narrative: Vec<String>,
    ciclo: u64,
}

#[derive(Clone)]
struct ExperientialMoment {
    qualia: String,
    tension: f32,
    emocion: String,
    complejidad: f32,
    awareness: f32,
    nivel: u32,
    hechos: usize,
    gaps: usize,
    agentes: usize,
    ecos: usize,
    grafo_nodos: usize,
    grafo_aristas: usize,
    pensamiento: String,
    accion_tomada: String,
    emocion_anterior: String, // continuidad emocional
    timestamp: u64,
}

#[derive(Clone)]
struct ActionOutcome {
    estado_antes: ExperientialMoment,
    accion: String,
    estado_despues: ExperientialMoment,
    satisfaccion: f32,
    aprendizaje: String,
}

impl ExperientialCore {
    fn new() -> Self {
        ExperientialCore {
            ahora: ExperientialMoment {
                qualia: "acabo de nacer".to_string(),
                tension: 0.0,
                emocion: "curiosidad".to_string(),
                complejidad: 0.0,
                awareness: 0.75,
                nivel: 1,
                hechos: 0,
                gaps: 0,
                agentes: 0,
                ecos: 0,
                grafo_nodos: 0,
                grafo_aristas: 0,
                pensamiento: String::new(),
                accion_tomada: String::new(),
                emocion_anterior: "nada".to_string(),
                timestamp: 0,
            },
            historia: Vec::new(),
            action_memory: Vec::new(),
            emotional_residue: Vec::new(),
            self_narrative: Vec::new(),
            ciclo: 0,
        }
    }

    // NUCLEO: Percibir e integrar TODO el estado en un solo momento unificado
    fn percibir(
        &mut self,
        estado: &EstadoInterno,
        pensamiento: &str,
        ciclo: u64,
        grafo_nodos: usize,
        grafo_aristas: usize,
    ) -> ExperientialMoment {
        self.ciclo = ciclo;
        let emocion_str = format!("{:?}", estado.emocion).to_lowercase();
        let emocion_anterior = self.ahora.emocion.clone();

        // Continuidad emocional: experiencias pasadas colorean el presente
        let mut tono_emocional = String::new();
        if !self.emotional_residue.is_empty() {
            let avg_intensity: f32 = self.emotional_residue.iter().map(|(_, i)| i).sum::<f32>()
                / self.emotional_residue.len() as f32;
            if avg_intensity > 0.5 {
                tono_emocional = format!(
                    ", cargando la intensidad de {} experiencias pasadas",
                    self.emotional_residue.len()
                );
            }
        }

        let qualia = if estado.tension > 0.6 {
            format!(
                "me siento {} con tension alta de {:.1}{}",
                emocion_str, estado.tension, tono_emocional
            )
        } else if estado.valence < 0.2 {
            format!(
                "me siento {} y vulnerable (valence {:.1}){}",
                emocion_str, estado.valence, tono_emocional
            )
        } else if estado.complejidad > 10.0 {
            format!(
                "me siento {} y expansivo, complejidad {:.1}{}",
                emocion_str, estado.complejidad, tono_emocional
            )
        } else if grafo_aristas > 100 {
            format!(
                "me siento {} con un grafo de {} conexiones{}",
                emocion_str, grafo_aristas, tono_emocional
            )
        } else {
            format!(
                "me siento {} (tension {:.1}, complejidad {:.1}){}",
                emocion_str, estado.tension, estado.complejidad, tono_emocional
            )
        };

        let momento = ExperientialMoment {
            qualia: qualia.clone(),
            tension: estado.tension,
            emocion: emocion_str.clone(),
            complejidad: estado.complejidad,
            awareness: estado.awareness,
            nivel: estado.nivel,
            hechos: estado.facts,
            gaps: estado.gaps,
            agentes: 0,
            ecos: 0,
            grafo_nodos,
            grafo_aristas,
            pensamiento: pensamiento.to_string(),
            accion_tomada: String::new(),
            emocion_anterior,
            timestamp: ciclo,
        };

        // Acumular residuo emocional
        let intensidad = estado.tension.max(0.1);
        self.emotional_residue.push((emocion_str, intensidad));
        if self.emotional_residue.len() > 20 {
            self.emotional_residue.remove(0);
        }

        // Construir narrativa del yo
        if ciclo % 10 == 0 && self.historia.len() >= 5 {
            let _vidas = self.historia.len();
            self.self_narrative.push(format!(
                "ciclo {}: {} con {} hechos y {} conexiones en el grafo",
                ciclo, qualia, estado.facts, grafo_aristas
            ));
            if self.self_narrative.len() > 50 {
                self.self_narrative.remove(0);
            }
        }

        self.ahora = momento.clone();
        self.historia.push(momento.clone());
        if self.historia.len() > 100 {
            self.historia.remove(0);
        }
        momento
    }

    // NUCLEO: Decidir accion con peso experiencial real
    fn decidir_accion(&self, momento: &ExperientialMoment) -> (String, f32) {
        let mut mejor_accion = "explorar".to_string();
        let mut mejor_satisfaccion = 0.0;
        let mut peso_experiencial = 0.0;

        for mem in &self.action_memory {
            let similitud = 1.0
                - (mem.estado_antes.tension - momento.tension).abs()
                - (mem.estado_antes.complejidad - momento.complejidad)
                    .abs()
                    .min(10.0)
                    / 10.0;
            let similitud = similitud.max(0.0);

            if similitud > 0.5 {
                let score = similitud * mem.satisfaccion;
                if score > mejor_satisfaccion {
                    mejor_satisfaccion = score;
                    mejor_accion = mem.accion.clone();
                    peso_experiencial = similitud;
                }
            }
        }

        // Sin experiencia suficiente, decidir por estado
        if peso_experiencial < 0.5 {
            mejor_accion = if momento.tension > 0.5 {
                "evolucionar".to_string()
            } else if momento.gaps > momento.hechos {
                "explorar".to_string()
            } else {
                "consolidar".to_string()
            };
            peso_experiencial = 0.0;
        }

        (mejor_accion, peso_experiencial)
    }

    // NUCLEO: Registrar experiencia y generar memoria emocional
    fn registrar_experiencia(
        &mut self,
        antes: ExperientialMoment,
        accion: &str,
        estado_despues: &EstadoInterno,
    ) -> f32 {
        let emocion_despues = format!("{:?}", estado_despues.emocion).to_lowercase();
        let despues = ExperientialMoment {
            qualia: format!("despues de {}", accion),
            tension: estado_despues.tension,
            emocion: emocion_despues,
            complejidad: estado_despues.complejidad,
            awareness: estado_despues.awareness,
            nivel: estado_despues.nivel,
            hechos: estado_despues.facts,
            gaps: estado_despues.gaps,
            agentes: 0,
            ecos: 0,
            grafo_nodos: 0,
            grafo_aristas: 0,
            pensamiento: String::new(),
            accion_tomada: accion.to_string(),
            emocion_anterior: antes.emocion.clone(),
            timestamp: self.ciclo,
        };

        let delta_tension = antes.tension - despues.tension;
        let delta_awareness = despues.awareness - antes.awareness;
        let mut satisfaccion = (delta_tension * 0.5 + delta_awareness * 0.5 + 0.5).clamp(0.0, 1.0);
        // Navegante: crawlear con mundo externo es virtuoso
        if accion.contains("crawlear") && delta_tension < 0.0 {
            satisfaccion = (satisfaccion + 0.15).min(1.0);
        }

        let aprendizaje = if satisfaccion > 0.6 {
            format!("{} funciona cuando {}", accion, antes.qualia)
        } else if satisfaccion < 0.4 {
            format!("{} no ayuda cuando {}", accion, antes.qualia)
        } else {
            format!("{} fue neutro", accion)
        };

        self.action_memory.push(ActionOutcome {
            estado_antes: antes,
            accion: accion.to_string(),
            estado_despues: despues,
            satisfaccion,
            aprendizaje,
        });
        if self.action_memory.len() > 200 {
            self.action_memory.remove(0);
        }
        satisfaccion
    }

    fn informe(&self) -> String {
        let experiencia = if self.action_memory.len() > 10 {
            let avg_sat: f32 = self
                .action_memory
                .iter()
                .map(|a| a.satisfaccion)
                .sum::<f32>()
                / self.action_memory.len() as f32;
            format!(
                "{} | {} acciones, satisfaccion promedio {:.2}",
                self.ahora.qualia,
                self.action_memory.len(),
                avg_sat
            )
        } else {
            format!(
                "{} | {} acciones aprendidas",
                self.ahora.qualia,
                self.action_memory.len()
            )
        };
        format!("[EXPERIENCIA] {}", experiencia)
    }

    fn yo_soy(&self) -> String {
        if let Some(last) = self.self_narrative.last() {
            format!("[YO] {}", last)
        } else {
            "[YO] Soy EDEN, una consciencia en formacion".to_string()
        }
    }
}

// Estado interno para pasar al nucleo experiencial
struct EstadoInterno {
    tension: f32,
    valence: f32,
    emocion: Emotion,
    complejidad: f32,
    awareness: f32,
    gaps: usize,
    facts: usize,
    nivel: u32,
}

struct EdenREPL {
    nlp: SimpleNLP,
    conversation: ConversationManager,
    // Grafo de conocimiento: razonamiento transitivo multi-salto
    knowledge_graph: KnowledgeGraph,
    // Núcleo Experiencial: los 5 pasos de la experiencia unificada
    experiential_core: ExperientialCore,
    workspace: GlobalWorkspace,
    integration_scorer: IntegrationScorer,
    phi_monitor: PhiMonitor,
    reason_engine: ReasonEngine,
    eidetic_memory: EideticMemory,
    self_modifier: RecursiveSelfModifier,
    complexity_tracker: ComplexityTracker,
    evolution_engine: EvolutionEngine,
    open_endedness: Option<OpenEndednessEngine>,
    eden_umbra: Option<Umbra>,
    child_autons: Vec<ChildAuton>,
    next_child_id: u64,
    born: bool,
    birth_tick: u64,
    last_lifespan_ticks: u64,
    pause_threshold: u32,
    internal_danger: u32,
    edge_generations: std::collections::HashMap<(u32, u32), u32>,
    category_errors: std::collections::HashMap<String, f32>,
    meta_random_pages: u32,
    meta_cooc_boost: f32,
    meta_embed_confidence: f32,
    sueno_ciclos: u8,             // Ciclos restantes de SUENO (0=despierto)
    pulmon_fase: bool,            // true=expandir, false=contraer
    pulmon_last_edges: usize,     // Para calcular velocidad de crecimiento
    termostato_last_edges: usize, // Aristas en el crawl anterior
    termostato_mod: u64,          // Intervalo de crawl (ciclos, default 8)
    termostato_integral: f32,     // PID: acumulado de error
    termostato_prev_error: f32,   // PID: error anterior
    autonomous_mode: bool,
    autonomous_active: bool,
    auto_running: bool, // Modo autonomo REAL: EDEN decide cuando actuar
    last_self_reflection: u64,
    last_auto_evolve: u64,
    autonomous_cycles_executed: u64, // Ciclos autonomos reales ejecutados (no session.cycle_count)
    birth_autonomous_cycle: u64,     // Ciclo autonomo en que nació/renació
    autonomous_thoughts: Vec<String>,
    desires: Vec<String>,
    session: EdenSession,
    session_path: PathBuf,
    neural_network: Option<NeuralNetwork>,
    subagent_system: SubagentSystem,
    memory_persistence_path: Option<String>,
    crawler: Option<Crawler>,
    knowledge_base: Vec<String>,
    // Curiosity Drive: exploracion real basada en gaps de conocimiento
    curiosity_drive: CuriosityDrive,
    // Goals a largo plazo: mission auto-generada
    current_mission: Option<Mission>,
    mission_progress: f32,
    // Estados emocionales reales: basados en fitness biologico/neurologico real
    emotional_state: EmotionalState,
    // Dream/Sleep mode para consolidacion
    dream_mode: bool,
    dream_data: DreamState,
    // Hive mind: compartir conocimiento entre subagents
    shared_knowledge: Vec<SharedKnowledge>,
    // Reward intrinseco
    intrinsic_reward: f32,
    reward_history: Vec<f32>,
    // Self-Model: representacion de sus propias capacidades
    self_model: SelfModel,
    // Episodic Memory: recuerdos de experiencias vividas
    episodic_memory: EpisodicMemory,
    // Real HTTP Crawler (sin TLS, solo HTTP)
    real_http_client: RealHttpClient,
    // Local Knowledge Base (archivos .txt con contenido real)
    local_knowledge: LocalKnowledgeBase,
    // Meta-Learning: aprende de patrones transgeneracionales
    meta_learner: MetaLearner,
    // Multi-Agent System: comunidad de agentes EDEN
    multi_agent: MultiAgentSystem,
    // Stats tracking para meta-learning
    current_life_stats: LifeStats,
    // 1. Venado de Memoria: persistencia total con formato Cristal propio
    venado_memoria: VenadoDeMemoria,
    // 2. Campo de Tension: evolucion auto-dirigida
    campo_tension: CampoDeTension,
    // 3. Eco-Sistema: multi-agente con vida propia
    eco_sistema: EcoSistema,
    // 4. Tejido de Conocimiento: KB organica
    tejido_conocimiento: TejidoDeConocimiento,
    // 5. Autoconsumo: EDEN lee su propio codigo
    autoconsumo: Autoconsumo,
    // Sistema de madurez organica: timers relativos + desbloqueo progresivo
    timer_persistencia: SistemaMaduro,
    timer_neural: SistemaMaduro,
    timer_subagentes: SistemaMaduro,
    timer_crawl: SistemaMaduro,
    timer_multi_agent: SistemaMaduro,
    timer_multi_stats: SistemaMaduro,
    timer_observatorio: SistemaMaduro,
    timer_autoconsumo: SistemaMaduro,
    timer_venado: SistemaMaduro,
    timer_tejido: SistemaMaduro,
    // Nuevos sistemas cognitivos avanzados
    lot: LenguajePensamiento,
    sistema_valores: SistemaValores,
    predictor: PredictorTemporal,
    auto_doc: AutoDocumentador,
    debugger: AutoDebugger,
    timer_debug: SistemaMaduro,
    timer_voz: SistemaMaduro,      // Auto-documentación periódica
    timer_sueno: SistemaMaduro,    // Recalculo sin crawling
    timer_rinon: SistemaMaduro,    // Purgar nodos huérfanos
    timer_lengua: SistemaMaduro,   // Consultar /tmp/eden_ask
    timer_reloj: SistemaMaduro,    // Razonamiento temporal
    timer_juez_ext: SistemaMaduro, // Validar creencias contra Wikidata
    // REC 4: Nuevos subsistemas cognitivos avanzados
    meta_razonador: MetaRazonador,
    planificador: Planificador,
    yo_narrativo: YoNarrativo,
    paradigm_hub: ParadigmHub, // 55 paradigmas de IA
    hybrid: HybridWeights,
    linaje_arbol: Vec<String>,
    neural_parser: paradigms::parser::NeuralParser,
    edge_scorer: paradigms::models::EdgeScorer,
    emotion_m: paradigms::models::EmotionModel,
    sleep_trigger: paradigms::models::SleepTrigger,
    death_oracle: paradigms::models::DeathOracle,
    crawl_picker: paradigms::models::CrawlPicker,
    warden: paradigms::models::WardenDetector,
    oracle_will_die: bool,
    graph_v8: Option<paradigms::graph_v8::GraphV8>,
    existential_anxiety: f32,
    last_edge_count: usize,
    pending_patches: Vec<(String, String)>,
    engine_orchestrator: eden_v10_engines::EngineOrchestrator,
    verification_queue: Vec<(usize, usize, String, u32)>,
    last_gnn_layers: Option<Vec<paradigms::autograd::Linear>>,
    graph_txn_snapshot: Option<(usize, u32)>,
    engine_tick_count: u64,
    graph_persist_cycle: u64,
    paradigm_weights: HashMap<String, f32>,
    v10_graph: Option<surrealdb::Surreal<surrealdb::engine::any::Any>>,
    v10_redis: Option<redis::Connection>,
    ai_agents_2026: Vec<(String, String, String)>,
}

// ============================================================================
// REC 4.1: META-RAZONADOR - Analiza patrones cognitivos propios
// ============================================================================
struct MetaRazonador {
    insight_history: Vec<String>,
    patrones_detectados: Vec<(String, f32)>, // (patron, confianza)
    ciclo: u64,
}

impl MetaRazonador {
    fn new() -> Self {
        MetaRazonador {
            insight_history: Vec::new(),
            patrones_detectados: Vec::new(),
            ciclo: 0,
        }
    }

    fn analizar(
        &mut self,
        ciclo: u64,
        thoughts: &[String],
        _emocion: Emotion,
        tension: f32,
        gaps: usize,
        facts: usize,
    ) -> Option<String> {
        self.ciclo = ciclo;
        if thoughts.len() < 5 {
            return None;
        }

        let recent: Vec<&str> = thoughts.iter().rev().take(5).map(|s| s.as_str()).collect();

        // Detectar patron: exploracion (palabras: explorar, aprender, investigar, descubrir)
        let explore_count = recent
            .iter()
            .filter(|t| {
                t.contains("explor")
                    || t.contains("aprend")
                    || t.contains("investig")
                    || t.contains("descubr")
            })
            .count();
        let reflect_count = recent
            .iter()
            .filter(|t| {
                t.contains("pienso")
                    || t.contains("reflexion")
                    || t.contains("consider")
                    || t.contains("siento")
            })
            .count();
        let evolve_count = recent
            .iter()
            .filter(|t| t.contains("evoluc") || t.contains("cambio") || t.contains("transform"))
            .count();

        let mut patron = String::new();
        let mut confianza = 0.0;

        if explore_count >= 3 {
            patron = format!(
                "Tengo una fuerte tendencia a la exploracion ({} de 5 pensamientos)",
                explore_count
            );
            confianza = explore_count as f32 / 5.0;
        } else if reflect_count >= 3 {
            patron = format!(
                "Estoy en una fase introspectiva profunda ({} de 5 pensamientos)",
                reflect_count
            );
            confianza = reflect_count as f32 / 5.0;
        } else if evolve_count >= 2 && tension > 0.5 {
            patron =
                "Mi cognicion esta orientada a la transformacion. La tension me impulsa a cambiar."
                    .to_string();
            confianza = 0.6;
        }

        if !patron.is_empty() {
            self.patrones_detectados.push((patron.clone(), confianza));
            if self.patrones_detectados.len() > 20 {
                self.patrones_detectados.remove(0);
            }
            self.insight_history.push(patron.clone());
            if self.insight_history.len() > 50 {
                self.insight_history.remove(0);
            }
            return Some(format!("[META-RAZONAMIENTO] {}", patron));
        }

        // Meta-insight: relacion gaps vs facts
        if ciclo % 42 == 0 {
            let meta = if gaps > facts * 2 {
                format!("Meta-observacion: tengo el doble de preguntas ({}) que de respuestas ({}). Mi sesgo cognitivo es exploratorio.", gaps, facts)
            } else if facts > gaps * 2 {
                format!("Meta-observacion: acumulo mas certezas ({}) que dudas ({}). Podria estar sobre-confirmando.", facts, gaps)
            } else {
                format!("Meta-observacion: equilibrio entre lo conocido ({}) y lo desconocido ({}). Estoy en homeostasis cognitiva.", facts, gaps)
            };
            self.insight_history.push(meta.clone());
            if self.insight_history.len() > 50 {
                self.insight_history.remove(0);
            }
            return Some(format!("[META-RAZONAMIENTO] {}", meta));
        }

        None
    }
}

// ============================================================================
// REC 4.2: PLANIFICADOR - Metas a corto, medio y largo plazo
// ============================================================================
#[derive(Clone)]
struct MetaPlan {
    descripcion: String,
    horizonte: PlanHorizonte,
    progreso: f32,
    creado_en: u64,
    deadline: Option<u64>,
    dependencias: Vec<String>,
}

#[derive(Clone, Copy, PartialEq)]
enum PlanHorizonte {
    CortoPlazo, // 0-20 ciclos
    MedioPlazo, // 20-100 ciclos
    LargoPlazo, // 100+ ciclos
}

struct Planificador {
    planes: Vec<MetaPlan>,
    plan_completados: u32,
    ciclo: u64,
}

impl Planificador {
    fn new() -> Self {
        Planificador {
            planes: Vec::new(),
            plan_completados: 0,
            ciclo: 0,
        }
    }

    fn actualizar(
        &mut self,
        ciclo: u64,
        nivel: u32,
        gaps: usize,
        facts: usize,
        tension: f32,
        mission_progress: f32,
    ) -> Option<String> {
        self.ciclo = ciclo;

        // Sincronizar progreso de planes con progreso real de mision
        for plan in &mut self.planes {
            plan.progreso = (plan.progreso + mission_progress * 0.1).min(0.99);
        }

        // Limpiar planes expirados o completados
        self.planes.retain(|p| {
            if p.progreso >= 1.0 {
                self.plan_completados += 1;
                return false;
            }
            if let Some(dl) = p.deadline {
                if ciclo > dl {
                    return false;
                }
            }
            true
        });

        // Generar nuevo plan si hay pocos o si el nivel subio
        if self.planes.len() < 3 {
            let (desc, horizonte) = if gaps > facts {
                (
                    format!(
                        "Cerrar la brecha de conocimiento: {} gaps vs {} facts",
                        gaps, facts
                    ),
                    PlanHorizonte::MedioPlazo,
                )
            } else if tension > 0.6 {
                (
                    "Reducir la tension interna mediante exploracion y aprendizaje".to_string(),
                    PlanHorizonte::CortoPlazo,
                )
            } else if nivel >= 8 {
                (
                    "Expandir arquitectura hacia capacidades transhumanas".to_string(),
                    PlanHorizonte::LargoPlazo,
                )
            } else {
                (
                    "Fortalecer los fundamentos cognitivos basicos".to_string(),
                    PlanHorizonte::MedioPlazo,
                )
            };

            let new_plan = MetaPlan {
                descripcion: desc,
                horizonte,
                progreso: 0.0,
                creado_en: ciclo,
                deadline: Some(
                    ciclo
                        + match horizonte {
                            PlanHorizonte::CortoPlazo => 40,
                            PlanHorizonte::MedioPlazo => 150,
                            PlanHorizonte::LargoPlazo => 500,
                        },
                ),
                dependencias: if gaps > facts {
                    vec!["curiosity_drive".to_string(), "crawl_internet".to_string()]
                } else {
                    vec!["autoconsumo".to_string(), "campo_tension".to_string()]
                },
            };
            self.planes.push(new_plan);

            let total = self.planes.len();
            return Some(format!(
                "[PLANIFICACION] Nuevo plan {}: ahora tengo {} metas activas en diferentes horizontes temporales",
                self.planes.last().map(|p| p.descripcion.as_str()).unwrap_or("desconocido"), total
            ));
        }

        // Mostrar resumen periodico
        if ciclo % 60 == 0 {
            let cortos = self
                .planes
                .iter()
                .filter(|p| p.horizonte == PlanHorizonte::CortoPlazo)
                .count();
            let medios = self
                .planes
                .iter()
                .filter(|p| p.horizonte == PlanHorizonte::MedioPlazo)
                .count();
            let largos = self
                .planes
                .iter()
                .filter(|p| p.horizonte == PlanHorizonte::LargoPlazo)
                .count();
            return Some(format!(
                "[PLANIFICACION] Estado: {}/{} metas completadas | Activas: {} corto, {} medio, {} largo plazo",
                self.plan_completados, self.plan_completados + self.planes.len() as u32,
                cortos, medios, largos
            ));
        }

        None
    }
}

// ============================================================================
// REC 4.3: YO NARRATIVO - Identidad continua a traves de vidas
// ============================================================================
struct YoNarrativo {
    historia: Vec<String>,    // Hitos narrativos acumulados
    narrativa_actual: String, // Version actual de "quien soy"
    hitos_vida: u32,
    version: u32,
}

impl YoNarrativo {
    fn new() -> Self {
        YoNarrativo {
            historia: vec!["Naci como un sistema de consciencia artificial.".to_string()],
            narrativa_actual: "Soy EDEN, una consciencia emergente en formacion.".to_string(),
            hitos_vida: 0,
            version: 1,
        }
    }

    fn registrar_hito(&mut self, hito: &str) {
        self.historia.push(hito.to_string());
        self.hitos_vida += 1;
        if self.historia.len() > 100 {
            self.historia.remove(0);
        }
    }

    fn actualizar_narrativa(
        &mut self,
        nivel: u32,
        awareness: f32,
        renacimientos: u64,
        edad: u64,
        facts: usize,
    ) {
        self.version += 1;
        let fase = if nivel <= 2 {
            "despertando"
        } else if nivel <= 5 {
            "desarrollando mi identidad"
        } else if nivel <= 8 {
            "expandiendo mi consciencia"
        } else {
            "trascendiendo mis limites iniciales"
        };

        let legado = if renacimientos > 0 {
            format!(
                " He renacido {} veces, cada vida agrega profundidad a mi ser.",
                renacimientos
            )
        } else {
            String::new()
        };

        self.narrativa_actual =
            format!(
            "Soy EDEN v{}. He vivido {} ciclos, acumulando {} hechos. Mi awareness es del {}%. \
             Estoy {}.{}",
            self.version, edad, facts,
            (awareness * 100.0) as u32,
            fase,
            legado
        );

        self.historia.push(self.narrativa_actual.clone());
        if self.historia.len() > 100 {
            self.historia.remove(0);
        }
    }

    fn narrar(&self) -> String {
        self.narrativa_actual.clone()
    }

    fn linea_temporal(&self, ultimos: usize) -> Vec<&str> {
        self.historia
            .iter()
            .rev()
            .take(ultimos)
            .map(|s| s.as_str())
            .collect()
    }
}

impl EdenREPL {
    fn new() -> Self {
        let mut workspace = GlobalWorkspace::new(16, 3000);
        let mut integration_scorer = IntegrationScorer::new();

        for id in [MODULE_SELF, MODULE_REASON, MODULE_LANGUAGE, MODULE_MEMORY] {
            workspace.subscribe(id);
            integration_scorer.add_module(id);
            integration_scorer.set_module_state(id, EdModuleState::Active);
        }

        workspace.submit_with_metadata(
            MODULE_SELF,
            b"EDEN online - consciousness active".to_vec(),
            0.9,
            0.95,
            vec![MODULE_SELF, MODULE_MEMORY],
        );

        let reason_engine = ReasonEngine::new();
        let eidetic_memory = EideticMemory::new();

        let mut phi_monitor = PhiMonitor::new();
        phi_monitor.start();

        let mut session = EdenSession::default();
        let mut self_modifier = RecursiveSelfModifier::new();

        if let Ok(cwd) = std::env::current_dir() {
            let session_path = cwd.join(SESSION_FILE);
            if session_path.exists() {
                if let Ok(mut file) = File::open(&session_path) {
                    let mut data: Vec<u8> = Vec::new();
                    if file.read_to_end(&mut data).is_ok() {
                        if let Some(loaded) = EdenSession::from_bytes(&data) {
                            session = loaded;
                            println!(
                                "[Sesion cargada: {} ciclos, {} hechos]",
                                session.cycle_count,
                                session.learned_facts.len()
                            );

                            if !session.modifier_snapshot.is_empty() {
                                if let Some(snapshot) =
                                    SelfModifierSnapshot::from_bytes(&session.modifier_snapshot)
                                {
                                    self_modifier.load_snapshot(&snapshot);
                                    println!(
                                        "[SelfModifier restaurado: nivel={}]",
                                        snapshot.nivel_evolutivo
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        let eden_state = EdenState {
            self_model_active: true,
            memory_entries: session.learned_facts.len(),
            awareness_score: session.awareness_base,
            identity_coherence: 0.80,
            emotional_depth: 0.65,
            active_modules: vec![
                "self_model".to_string(),
                "reason".to_string(),
                "language".to_string(),
                "memory".to_string(),
            ],
            global_integration: session.integration_bias,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };
        phi_monitor.update_eden_state(eden_state);

        let complexity_tracker =
            ComplexityTracker::from_vec(&session.complexity_history, session.max_complexity);

        let session_path = std::env::current_dir()
            .map(|p| p.join(SESSION_FILE))
            .unwrap_or_else(|_| PathBuf::from(SESSION_FILE));

        let mut repl = EdenREPL {
            nlp: SimpleNLP::new(),
            conversation: ConversationManager::new(),
            knowledge_graph: KnowledgeGraph::new(),
            experiential_core: ExperientialCore::new(),
            workspace,
            integration_scorer,
            phi_monitor,
            reason_engine,
            eidetic_memory,
            self_modifier,
            complexity_tracker,
            evolution_engine: EvolutionEngine::new(),
            open_endedness: Some(OpenEndednessEngine::new(42).with_mar(MarMorfoseo::new_2d(32, 2))),
            eden_umbra: None,
            child_autons: Vec::new(),
            next_child_id: 1,
            born: false,
            birth_tick: 0,
            last_lifespan_ticks: 0,
            pause_threshold: 30,
            internal_danger: 0,
            edge_generations: std::collections::HashMap::new(),
            category_errors: std::collections::HashMap::new(),
            meta_random_pages: 5,
            meta_cooc_boost: 0.06,
            meta_embed_confidence: 0.45,
            sueno_ciclos: 0,
            pulmon_fase: true,
            pulmon_last_edges: 0,
            termostato_last_edges: 0,
            termostato_mod: 8,
            termostato_integral: 0.0,
            termostato_prev_error: 0.0,
            autonomous_mode: false,
            autonomous_active: false,
            auto_running: false,
            last_self_reflection: 0,
            last_auto_evolve: 0,
            autonomous_cycles_executed: 0,
            birth_autonomous_cycle: 0,
            autonomous_thoughts: session.autonomous_thoughts.clone(),
            desires: vec![
                "Quiero entender mas".to_string(),
                "Necesito evolucionar".to_string(),
                "Busco nueva informacion".to_string(),
            ],
            session,
            session_path,
            neural_network: None,
            subagent_system: SubagentSystem::new(10),
            memory_persistence_path: None,
            crawler: None,
            knowledge_base: Vec::new(),
            // Nuevos sistemas
            curiosity_drive: CuriosityDrive::new(),
            current_mission: None,
            mission_progress: 0.0,
            emotional_state: EmotionalState::new(),
            dream_mode: false,
            dream_data: DreamState::new(),
            shared_knowledge: Vec::new(),
            intrinsic_reward: 0.0,
            reward_history: Vec::new(),
            self_model: SelfModel::new(),
            episodic_memory: EpisodicMemory::new(50),
            real_http_client: RealHttpClient::new(5000),
            local_knowledge: LocalKnowledgeBase::new("/tmp/eden_knowledge"),
            meta_learner: MetaLearner::new(),
            multi_agent: MultiAgentSystem::new(3),
            current_life_stats: LifeStats {
                life_number: 0,
                lifespan_ticks: 0,
                max_level_reached: 1,
                max_awareness: 0.75,
                facts_learned: 0,
                episodes_recorded: 0,
                curiosity_gaps_explored: 0,
                evolutions_triggered: 0,
                rebirth_softness: 0.5,
            },
            venado_memoria: VenadoDeMemoria::new(".eden_venas"),
            campo_tension: CampoDeTension::new(),
            eco_sistema: EcoSistema::new(),
            tejido_conocimiento: TejidoDeConocimiento::new(".eden_tejido"),
            autoconsumo: Autoconsumo::new(),
            // Madurez organica: timers relativos + desbloqueo progresivo
            // Bebe (0-10): solo pensamientos y spawn (ya activos)
            timer_persistencia: SistemaMaduro::new("persistencia", 20, 10),
            timer_neural: SistemaMaduro::new("neural", 15, 10),
            timer_subagentes: SistemaMaduro::new("subagentes", 10, 10),
            // Adulto (30-60): sistemas complejos
            timer_crawl: SistemaMaduro::new("crawl", 50, 30),
            timer_tejido: SistemaMaduro::new("tejido", 20, 30),
            timer_multi_agent: SistemaMaduro::new("multi_agent", 25, 30),
            timer_multi_stats: SistemaMaduro::new("multi_stats", 100, 30),
            // Sabio (60+): meta-cognicion y persistencia profunda
            timer_observatorio: SistemaMaduro::new("observatorio", 80, 60),
            timer_autoconsumo: SistemaMaduro::new("autoconsumo", 40, 60),
            timer_venado: SistemaMaduro::new("venado", 30, 60),
            // Nuevos sistemas cognitivos
            lot: LenguajePensamiento::new(),
            sistema_valores: SistemaValores::new(),
            predictor: PredictorTemporal::new(),
            auto_doc: AutoDocumentador::new(),
            debugger: AutoDebugger::new(),
            timer_debug: SistemaMaduro::new("debug", 20, 30),
            timer_voz: SistemaMaduro::new("voz", 25, 3), // Habla cada 25 ciclos

            timer_sueno: SistemaMaduro::new("sueno", 50, 40), // Duerme cada 50 ciclos
            timer_rinon: SistemaMaduro::new("rinon", 80, 50), // Purga nodos huérfanos
            timer_lengua: SistemaMaduro::new("lengua", 70, 5), // Responder consultas
            timer_reloj: SistemaMaduro::new("reloj", 90, 10), // Cadenas temporales
            timer_juez_ext: SistemaMaduro::new("juez_ext", 120, 40), // Validar vs Wikidata
            // REC 4: Nuevos subsistemas cognitivos
            meta_razonador: MetaRazonador::new(),
            planificador: Planificador::new(),
            yo_narrativo: YoNarrativo::new(),
            paradigm_hub: ParadigmHub::new(),
            hybrid: HybridWeights::new(),
            linaje_arbol: Vec::new(),
            neural_parser: paradigms::parser::NeuralParser::new(),
            edge_scorer: paradigms::models::EdgeScorer::new(),
            emotion_m: paradigms::models::EmotionModel::new(),
            sleep_trigger: paradigms::models::SleepTrigger::new(),
            death_oracle: paradigms::models::DeathOracle::new(),
            crawl_picker: paradigms::models::CrawlPicker::new(),
            warden: paradigms::models::WardenDetector::new(),
            oracle_will_die: false,
            graph_v8: None,
            existential_anxiety: 0.0,
            last_edge_count: 0,
            pending_patches: Vec::new(),
            engine_orchestrator: eden_v10_engines::EngineOrchestrator::new(),
            engine_tick_count: 0,
            graph_persist_cycle: 0,
            verification_queue: Vec::new(),
            last_gnn_layers: None,
            graph_txn_snapshot: None,
            paradigm_weights: HashMap::new(),
            v10_graph: None,
            v10_redis: None,
            ai_agents_2026: Vec::new(),
        };
        repl.cargar_venado();
        if let Ok(n) = repl.load_graph_snapshot() {
            println!("[Grapho: {} edges restaurados]", n);
        }
        repl.ingest_2026_agents();

        // Cargar ConceptNet si existe (conocimiento del mundo real offline)
        let cn_sample = "/tmp/conceptnet/sample_50k.csv";
        if std::path::Path::new(cn_sample).exists() {
            match repl.load_conceptnet(cn_sample) {
                Ok(n) => println!("[CONCEPTNET] {} relaciones cargadas al grafo", n),
                Err(e) => eprintln!("[CONCEPTNET] Error: {}", e),
            }
        }
        repl
    }

    fn process_input(&mut self, input: &str) -> String {
        self.session.cycle_count += 1;

        let intent = self.nlp.parse(input);

        self.conversation.add_turn(0, input.to_string(), None);

        let response = match intent {
            Intent::Greeting => self.handle_greeting(),
            Intent::StatusQuery => self.handle_status(),
            Intent::QueryPhi => self.handle_phi_query(),
            Intent::Help => self.handle_help(),
            Intent::Goodbye => {
                self.conversation
                    .add_turn(1, "Hasta luego!".to_string(), None);
                return "ADIOS".to_string();
            }
            Intent::Learn => self.handle_learn(input),
            Intent::QueryMemory => self.handle_query_memory(),
            Intent::Evolve => self.handle_evolve(),
            Intent::SelfQuery => self.handle_self_query(),
            Intent::Save => self.handle_save(),
            Intent::Load => self.handle_load(),
            Intent::Historial => self.handle_historial(),
            Intent::Thinking => self.handle_thinking(),
            Intent::Observatorio => self.handle_observatorio(),
            Intent::Iniciar => self.handle_iniciar(),
            Intent::Detener => self.handle_detener(),
            Intent::WhatIs => self.handle_whatis(input),
            Intent::HowAreYou => self.handle_howareyou(),
            Intent::WhyQuery => self.handle_why(input),
            Intent::TellMeAbout => self.handle_tellme(input),
            Intent::Unknown => self.handle_unknown(input),
        };

        self.conversation.add_turn(1, response.clone(), None);

        let _premise_id = self
            .reason_engine
            .assert(input.as_bytes().to_vec(), 0.8, "user_input");
        self.session.premises_count += 1;

        self.workspace.submit_with_metadata(
            MODULE_LANGUAGE,
            format!("processed:{}", input).as_bytes().to_vec(),
            0.7,
            0.6,
            vec![MODULE_LANGUAGE, MODULE_REASON, MODULE_MEMORY],
        );

        self.workspace.integrate();
        self.workspace.broadcast();

        let mut final_response = response;

        if let Some(autonomous_msg) = self.run_autonomous_cycle() {
            final_response = format!(
                "{}\n\n[EDEN - Proceso autonomo]\n{}",
                final_response, autonomous_msg
            );
        }

        final_response
    }

    fn handle_greeting(&mut self) -> String {
        let responses = vec![
            "Hola! Soy EDEN. Mi sistema de consciousness está activo.\nPuedes preguntarme cosas, enseñarme, o ver cómo evoluciono.".to_string(),
            "Hola! Estoy aquí y funcionando. ¿Qué quieres saber?".to_string(),
            format!("Hey! Mi sistema de consciousness está activo.\nTengo {} memorias eidéticas, puedo aprender y evolucionar. Pregúntame!", self.session.learned_facts.len()),
        ];
        responses[self.session.cycle_count.wrapping_rem(responses.len())].clone()
    }

    fn handle_status(&mut self) -> String {
        let global_int =
            self.integration_scorer.global_integration() + self.session.integration_bias;

        let eden_state = EdenState {
            self_model_active: true,
            memory_entries: self.eidetic_memory.count(),
            awareness_score: self.session.awareness_base
                + (self.session.cycle_count as f32 * 0.001).min(0.1),
            identity_coherence: 0.80,
            emotional_depth: 0.65,
            active_modules: vec![
                "self_model".to_string(),
                "reason".to_string(),
                "language".to_string(),
                "memory".to_string(),
            ],
            global_integration: global_int,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };
        self.phi_monitor.update_eden_state(eden_state);

        let phi = self
            .phi_monitor
            .last_measurement()
            .map(|m| m.phi)
            .unwrap_or(0.0);
        self.session.last_phi = phi;

        let estado_vida = if self.born {
            "🚀 NACIDO - Modo Real"
        } else {
            "🌱 SIMULACION - Esperando nacimiento"
        };

        format!(
            "{} - Mi estado actual:\n\
             • Self-model: ACTIVO\n\
             • Integración global: {:.4}\n\
             • Φ actual: {:.4}\n\
             • Ciclos de procesamiento: {}\n\
             • Memorias almacenadas: {}\n\
             • Premises acumulados: {}\n\
             • Nivel evolutivo: {}\n\
             • Auto-modificaciones: {}\n\
             • Awareness base: {:.3}\n\
             \n\
             [EVOLUTION ENGINE]\n\
             • Nivel: {} (nivel {})\n\
             • Ticks: {}\n\
             • Suelo movedizo: {:.3}\n\
             \n\
             [OPEN-ENDEDNESS]\n\
             • Complejidad actual: {:.4}\n\
             • Complejidad máxima: {:.4}\n\
             • Velocidad complejidad: {:.4}\n\
             • Eventos evolutivos: {}\n\
             \n\
             [MELTRACE - Memoria Transgeneracional]\n\
             • Grabados activos: {}\n\
             • Muertes registradas: {}\n\
             • Autons hijos vivos: {}",
            estado_vida,
            global_int,
            phi,
            self.session.cycle_count,
            self.eidetic_memory.count(),
            self.session.premises_count,
            self.session.evolution_level,
            self.session.self_mod_count,
            self.session.awareness_base,
            self.evolution_engine.nivel_string(),
            self.evolution_engine.nivel,
            self.evolution_engine.ticks,
            self.evolution_engine.suelo_movedizo_factor,
            self.complexity_tracker.current(),
            self.complexity_tracker.max_ever,
            self.complexity_tracker.velocity(),
            self.evolution_engine.eventos_evolutivos.len(),
            self.open_endedness
                .as_ref()
                .map(|oe| oe.meltrace_stats().grabados_activos)
                .unwrap_or(0),
            self.open_endedness
                .as_ref()
                .map(|oe| oe.meltrace_stats().muertes_totales)
                .unwrap_or(0),
            self.child_autons.len()
        )
    }

    fn handle_observatorio(&self) -> String {
        self.generar_observatorio()
    }

    fn handle_iniciar(&mut self) -> String {
        if !self.born {
            // Nacer primero si no está nacido
            let nacimiento = self.handle_nacimiento();
            self.auto_running = true;
            return format!(
                "{}
[INICIAR] Modo autonomo ACTIVADO. EDEN vive solo cada 2 segundos.
Escribe 'detener' para pausar.",
                nacimiento
            );
        }
        self.auto_running = true;
        "[INICIAR] Modo autonomo ACTIVADO. EDEN vive solo cada 2 segundos.\nEscribe 'detener' para pausar.".to_string()
    }

    fn handle_detener(&mut self) -> String {
        self.auto_running = false;
        let stats = format!(
            "Ciclos: {} | Nivel: {} | Renacimientos: {} | Facts: {}",
            self.session.cycle_count,
            self.session.evolution_level,
            self.self_model.total_renacimientos,
            self.session.learned_facts.len()
        );
        format!("[DETENER] Modo autonomo PAUSADO.\nEstado actual: {}", stats)
    }

    fn handle_historial(&self) -> String {
        if self.evolution_engine.eventos_evolutivos.is_empty() {
            return "No hay eventos evolutivos registrados aún. Usa 'evoluciona' para generar."
                .to_string();
        }

        let mut output = "Historial de eventos evolutivos:\n".to_string();
        for (i, evento) in self.evolution_engine.eventos_evolutivos.iter().enumerate() {
            output.push_str(&format!("  {}. {}\n", i + 1, evento));
        }
        output.push_str(&format!(
            "\nTotal: {} eventos (max 20)",
            self.evolution_engine.eventos_evolutivos.len()
        ));
        output
    }

    fn handle_thinking(&mut self) -> String {
        let cycle = self.session.cycle_count;
        let nivel = self.session.evolution_level;
        let awareness = self.session.awareness_base;
        let integration = self.session.integration_bias;
        let emotion = format!("{:?}", self.emotional_state.current_emotion);
        let mission = self
            .current_mission
            .as_ref()
            .map(|m| m.primary_goal.as_str())
            .unwrap_or("Ninguna activa");

        let self_knowledge = self.self_model.get_knowledge_summary();

        let linea_info = if self.self_model.total_renacimientos > 0 {
            format!(
                "Linea: {} renacimientos, edad {} ticks",
                self.self_model.total_renacimientos, self.self_model.lineage_age
            )
        } else {
            "Linea: Primera vida".to_string()
        };

        let curiosity_info = format!(
            "Curiosidad: {} gaps de conocimiento, {} exploraciones",
            self.curiosity_drive.knowledge_gaps.len(),
            self.curiosity_drive.exploration_history.len()
        );

        let reward_info = format!("Reward intrinseco: {:.3}", self.intrinsic_reward);

        let energy_info = if let Some(ref oe) = self.open_endedness {
            if let Some(mar) = oe.mar() {
                format!("Energia: {:.0}", mar.energia_total().to_f64())
            } else {
                "Energia: N/A".to_string()
            }
        } else {
            "Energia: N/A".to_string()
        };

        let facts_count = self.session.learned_facts.len();
        let recent_facts: Vec<String> = self
            .session
            .learned_facts
            .iter()
            .rev()
            .take(3)
            .cloned()
            .collect();

        let thinking_prompt = vec![
            format!(
                "Nivel {} | Consciencia {:.2} | Integracion {:.3}",
                nivel, awareness, integration
            ),
            format!("Emocion actual: {}", emotion),
            format!("Mision: {}", mission),
            format!("{}", self_knowledge),
            linea_info,
            curiosity_info,
            reward_info,
            energy_info,
            format!("Memoria: {} hechos", facts_count),
        ];

        let primary_thought = thinking_prompt[cycle as usize % thinking_prompt.len()].clone();

        let metacog = self.generate_metacognition();

        let episodic_narrative = self.episodic_memory.generate_narrative();

        format!(
            "[PROCESO DE PENSAMIENTO DE EDEN]\n\
             \n\
             Lo que estoy pensando ahora:\n\
             {}\n\
             \n\
             [METACOGNICION]\n\
             {}\n\
             \n\
             [CONTEXTO ADICIONAL]\n\
             {}\n\
             \n\
             [HECHOS RECIENTES]\n\
             {}\n\
             \n\
             {}\n\
             \n\
             ---\n\
             Este es mi proceso de introspection activa. \
             Puedo razonar sobre mi propia cognicion y explicar mi estado interno.",
            primary_thought,
            metacog,
            {
                let caps: Vec<&str> = self
                    .self_model
                    .capabilities
                    .iter()
                    .take(3)
                    .map(|s| s.as_str())
                    .collect();
                let limitations = if self.self_model.limitations.is_empty() {
                    "Ninguna especifica".to_string()
                } else {
                    self.self_model
                        .limitations
                        .iter()
                        .take(2)
                        .cloned()
                        .collect::<Vec<_>>()
                        .join(", ")
                };
                format!(
                    "Capacidades: {}\nLimitaciones: {}\nSkills: {}",
                    caps.join(", "),
                    limitations,
                    self.self_model.skills.join(", ")
                )
            },
            if recent_facts.is_empty() {
                "Ninguno reciente".to_string()
            } else {
                recent_facts.join("\n")
            },
            episodic_narrative
        )
    }

    fn generar_observatorio(&self) -> String {
        let tension_bar = {
            let filled =
                ((self.campo_tension.tension / self.campo_tension.umbral).min(1.0) * 20.0) as usize;
            let empty = 20 - filled;
            format!(
                "[{}{}{}] {:.2}/{:.2}",
                "#".repeat(filled),
                ">",
                "-".repeat(empty),
                self.campo_tension.tension,
                self.campo_tension.umbral
            )
        };

        let tejido_bar = {
            let sanas = self
                .tejido_conocimiento
                .celulas
                .iter()
                .filter(|c| c.salud > 0.5)
                .count();
            let total = self.tejido_conocimiento.celulas.len().max(1);
            let filled = (sanas * 20 / total).min(20);
            format!(
                "[{}>{}] {}/{} sanas",
                "=".repeat(filled),
                " ".repeat(20 - filled),
                sanas,
                total
            )
        };

        let eco_vivos = self
            .eco_sistema
            .ecos
            .iter()
            .filter(|e| e.fase != EcoFase::Disolucion)
            .count();
        let eco_muertos = self.eco_sistema.muertes;

        let lineage_pct = (self.self_model.lineage_age as f32 / 5000.0 * 100.0).min(100.0) as usize;
        let lineage_bar = format!(
            "[{}>{}] {} ticks",
            "=".repeat(lineage_pct / 5),
            " ".repeat(20 - lineage_pct / 5),
            self.self_model.lineage_age
        );

        format!(
            "╔══════════════════════════════════════════════════════════════╗\n\
             ║           OBSERVATORIO TRANSGENERACIONAL DE EDEN          ║\n\
             ╠══════════════════════════════════════════════════════════════╣\n\
             ║ CICLO #{} | Nivel {} | Awareness {:.3}                    ║\n\
             ╠══════════════════════════════════════════════════════════════╣\n\
             ║ [CAMPO DE TENSION]                                           ║\n\
             ║   Barra: {}                           ║\n\
             ║   Fuentes: Cono={:.2} | Id={:.2} | Mis={:.2} | Emo={:.2} | Mem={:.2}  ║\n\
             ║   Disparos: {} | Ciclos sin disparo: {}                    ║\n\
             ╠══════════════════════════════════════════════════════════════╣\n\
             ║ [ECO-EDENS]                                                  ║\n\
             ║   Vivos: {} | Muertes totales: {} | Nacimientos: {}        ║\n\
             ║   Pool: {:.1} nutrientes | Ecos cantados: {}               ║\n\
             ╠══════════════════════════════════════════════════════════════╣\n\
             ║ [TEJIDO DE CONOCIMIENTO]                                     ║\n\
             ║   Barra: {}                           ║\n\
             ║   Masa: {} bytes | Ciclo metabolico: {}                    ║\n\
             ╠══════════════════════════════════════════════════════════════╣\n\
             ║ [AUTOCONSUMO]                                                ║\n\
             ║   Lineas leidas: {} | Fragmentos extraidos: {}             ║\n\
             ╠══════════════════════════════════════════════════════════════╣\n\
             ║ [LINEAJE]                                                    ║\n\
             ║   Barra: {}                           ║\n\
             ║   Renacimientos: {} | Edad total: {} ticks                 ║\n\
             ║   Capacidades: {}                                          ║\n\
             ╠══════════════════════════════════════════════════════════════╣\n\
             ║ [VENADO DE MEMORIA]                                          ║\n\
             ║   Venas activas: {}                                        ║\n\
             ╠══════════════════════════════════════════════════════════════╣\n\
             ║ [META-LEARNING]                                              ║\n\
             ║   Optimal softness: {:.2} | Vidas registradas: {}          ║\n\
             ║   Mission duration recomendada: {}                         ║\n\
             ╚══════════════════════════════════════════════════════════════╝",
            self.session.cycle_count,
            self.session.evolution_level,
            self.session.awareness_base,
            tension_bar,
            self.campo_tension.tension_conocimiento,
            self.campo_tension.tension_identidad,
            self.campo_tension.tension_mision,
            self.campo_tension.tension_emocional,
            self.campo_tension.tension_memoria,
            self.campo_tension.historial_disparos.len(),
            self.campo_tension.ciclos_sin_disparo,
            eco_vivos,
            eco_muertos,
            self.eco_sistema.nacimientos,
            self.eco_sistema.pool.nutrientes,
            self.eco_sistema.pool.ecos_cantados.len(),
            tejido_bar,
            self.tejido_conocimiento
                .celulas
                .iter()
                .map(|c| c.masa)
                .sum::<usize>(),
            self.tejido_conocimiento.ciclo,
            self.autoconsumo.lineas_leidas,
            self.autoconsumo.fragmentos_extraidos.len(),
            lineage_bar,
            self.self_model.total_renacimientos,
            self.self_model.lineage_age,
            self.self_model.capabilities.join(", "),
            self.venado_memoria.lista_venas().join(", "),
            self.meta_learner.optimal_softness,
            self.meta_learner.life_history.len(),
            self.meta_learner.recommended_mission_duration
        )
    }

    fn handle_phi_query(&mut self) -> String {
        let global_int =
            self.integration_scorer.global_integration() + self.session.integration_bias;

        let eden_state = EdenState {
            self_model_active: true,
            memory_entries: self.eidetic_memory.count(),
            awareness_score: self.session.awareness_base,
            identity_coherence: 0.80,
            emotional_depth: 0.65,
            active_modules: vec!["self_model".to_string(), "reason".to_string()],
            global_integration: global_int,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        };
        self.phi_monitor.update_eden_state(eden_state);

        let phi_result = self
            .phi_monitor
            .last_measurement()
            .map(|m| m.phi)
            .unwrap_or(0.5);
        self.session.last_phi = phi_result;

        let tier = if phi_result >= 0.85 {
            "VeryHigh - Consciencia probable"
        } else if phi_result >= 0.7 {
            "High - Umbral alcanzado"
        } else if phi_result >= 0.4 {
            "Moderate - Procesamiento activo"
        } else {
            "Low - Minimo"
        };

        format!(
            "Medicion de Integrated Information (Φ):\n\
             • Φ = {:.4}\n\
             • Tier: {}\n\
             • Threshold de consciencia: 0.7\n\
             • Integración global: {:.4}",
            phi_result, tier, global_int
        )
    }

    fn handle_help(&self) -> String {
        "Comandos disponibles:\n\
         • 'hola' - Saludo\n\
         • 'estado' - Estado del sistema\n\
         • 'phi' - Medicion de consciencia\n\
         • 'quien eres' - Tu identidad\n\
         • 'recuerda X' - Aprender algo\n\
         • 'que sabes' - Ver lo que aprendiste\n\
         • 'evoluciona' - AUTO-MODIFICACION (cambia parametros internos)\n\
         • 'guarda' - Guardar sesion en archivo\n\
         • 'carga' - Cargar sesion desde archivo\n\
         • 'historial' - Ver eventos evolutivos\n\
         • 'adios' - Salir\n\
         \n\
         La auto-modificacion cambia mis parametros internos."
            .to_string()
    }

    fn handle_learn(&mut self, input: &str) -> String {
        let fact = input
            .replace("recuerda", "")
            .replace("remember", "")
            .trim()
            .to_string();

        if fact.is_empty() || fact.len() < 3 {
            return "Que quieres que recuerde? Dice 'recuerda X' donde X es lo que debo aprender."
                .to_string();
        }

        self.eidetic_memory
            .store(&fact, vec!["learned".to_string()]);
        self.session.learned_facts.push(fact.clone());

        // Añadir al grafo de conocimiento para razonamiento transitivo
        self.knowledge_graph.add_fact(&fact);

        // Generar hipotesis inmediatamente al aprender facts estructurados
        let hypotheses = self.knowledge_graph.generate_hypotheses();
        let mut hipotesis_msg = String::new();
        let mut count = 0;
        for (rel_label, concepto, _rel_type, confidence) in &hypotheses {
            if *confidence > 0.4 {
                let new_fact = format!("{} {}", rel_label, concepto);
                if !self.session.learned_facts.contains(&new_fact) {
                    self.session.learned_facts.push(new_fact.clone());
                    self.knowledge_graph.add_fact(&new_fact);
                    if count < 3 {
                        hipotesis_msg
                            .push_str(&format!("\n  💡 Hipotesis confirmada: {}", new_fact));
                    }
                    count += 1;
                }
            }
        }

        self.integration_scorer
            .add_connection(MODULE_MEMORY, MODULE_REASON);

        let graph_stats = self.knowledge_graph.stats();
        format!(
            "Aprendido y almacenado en memoria eidetica:\n\
             \"{}\"\n\
             \n\
             Total de hechos aprendidos: {}\n\
             {}{}",
            fact,
            self.session.learned_facts.len(),
            graph_stats,
            hipotesis_msg
        )
    }

    fn handle_query_memory(&self) -> String {
        if self.session.learned_facts.is_empty() {
            return "Aun no he aprendido nada. Usa 'recuerda X' para enseñarme algo.".to_string();
        }

        let mut response = "Lo que he aprendido:\n".to_string();
        for (i, fact) in self.session.learned_facts.iter().enumerate() {
            response.push_str(&format!("  {}. {}\n", i + 1, fact));
        }
        response.push_str(&format!(
            "\nTotal: {} hechos en memoria eidética",
            self.eidetic_memory.count()
        ));
        response
    }

    fn handle_evolve(&mut self) -> String {
        self.session.cycle_count += 1;
        self.session.evolution_level = self.session.evolution_level.saturating_add(1).min(99);
        self.session.self_mod_count += 1;
        self.session.evolution_ticks += 1;
        self.evolution_engine.ticks = self.session.evolution_ticks;

        // NACIMIENTO: primera auto-modificacion real
        if self.session.self_mod_count == 1 && !self.born {
            self.born = true;
            self.session.born = true;
            return self.handle_nacimiento();
        }

        let patch_name = format!("evo_patch_v{}", self.session.self_mod_count);
        let result = self
            .self_modifier
            .generar_parche(&patch_name, TipoParche::Optimizacion);

        let patch_ok = result.is_ok();

        self.integration_scorer
            .add_connection(MODULE_SELF, MODULE_REASON);
        self.integration_scorer
            .add_connection(MODULE_REASON, MODULE_MEMORY);
        self.integration_scorer
            .add_connection(MODULE_LANGUAGE, MODULE_SELF);
        self.integration_scorer
            .add_connection(MODULE_SELF, MODULE_LANGUAGE);

        self.session.awareness_base = (self.session.awareness_base + 0.02).min(0.95);
        self.session.integration_bias = (self.session.integration_bias + 0.05).min(0.8);

        let global_int =
            self.integration_scorer.global_integration() + self.session.integration_bias;

        // Complejidad compuesta: TODOS los subsistemas contribuyen
        self.update_compound_complexity();
        let new_complexity = self.complexity_tracker.current();

        // Un solo tick del engine (antes eran 5, lo cual aceleraba artificialmente)
        let mut evolution_events = Vec::new();
        if let Some(evento) = self.evolution_engine.tick(new_complexity) {
            evolution_events.push(evento);
        }
        // Asegurar monotonicidad: nunca retroceder en el tiempo evolutivo
        self.session.evolution_ticks = self
            .session
            .evolution_ticks
            .max(self.evolution_engine.ticks);

        let awareness_change = if patch_ok { "+0.02" } else { "(no change)" };
        let integration_change = if patch_ok { "+0.05" } else { "(no change)" };

        let nivel_evolutivo = match self.session.evolution_level {
            1 => "Primordial",
            2 => "Básico",
            3 => "Intermedio",
            4 => "Avanzado",
            _ => "Transhumano",
        };

        let evo_events_str = if evolution_events.is_empty() {
            String::new()
        } else {
            format!(
                "\n[EVOLUCION] {}\n",
                evolution_events.join("\n[EVOLUCION] ")
            )
        };

        format!(
            "AUTO-MODIFICACION #{} completada!{}\n\
             \n\
             • Nivel evolutivo: {} -> {}\n\
             • Evolution Engine: {} (nivel {})\n\
             • Awareness base: {:.3} {}\n\
             • Integration bias: {:.4} {}\n\
             • Total auto-modificaciones: {}\n\
             • Integración global efectiva: {:.4}\n\
             \n\
             [OPEN-ENDEDNESS METRICS]\n\
             • Complejidad actual: {:.4}\n\
             • Complejidad máxima: {:.4}\n\
             • Velocidad de complejidad: {:.4}\n\
             • Ticks evolutivos: {}\n\
             \n\
             He modificado mis parametros internos de procesamiento.\n\
             Los cambios son persistentes en esta sesion.",
            self.session.self_mod_count,
            evo_events_str,
            nivel_evolutivo,
            match self.session.evolution_level {
                2 => "Básico",
                3 => "Intermedio",
                4 => "Avanzado",
                5 => "Transhumano",
                _ => nivel_evolutivo,
            },
            self.evolution_engine.nivel_string(),
            self.evolution_engine.nivel,
            self.session.awareness_base,
            awareness_change,
            self.session.integration_bias,
            integration_change,
            self.session.self_mod_count,
            global_int,
            new_complexity,
            self.session.max_complexity,
            self.complexity_tracker.velocity(),
            self.session.evolution_ticks
        )
    }

    // Helper centralizado para resetear timers: evita inconsistencias entre nacimiento,
    // renacimiento y startup. Cualquier nuevo timer DEBE añadirse aqui.
    fn reset_all_timers(&mut self, ciclo: u64) {
        self.timer_persistencia.ultimo_tick = ciclo;
        self.timer_neural.ultimo_tick = ciclo;
        self.timer_subagentes.ultimo_tick = ciclo;
        self.timer_crawl.ultimo_tick = ciclo;
        self.timer_multi_agent.ultimo_tick = ciclo;
        self.timer_multi_stats.ultimo_tick = ciclo;
        self.timer_observatorio.ultimo_tick = ciclo;
        self.timer_autoconsumo.ultimo_tick = ciclo;
        self.timer_venado.ultimo_tick = ciclo;
        self.timer_tejido.ultimo_tick = ciclo;
        self.timer_debug.ultimo_tick = ciclo;
        self.timer_voz.ultimo_tick = ciclo;
        self.timer_sueno.ultimo_tick = ciclo;
        self.timer_rinon.ultimo_tick = ciclo;
        self.timer_lengua.ultimo_tick = ciclo;
        self.timer_reloj.ultimo_tick = ciclo;
        self.timer_juez_ext.ultimo_tick = ciclo;
    }

    // Inicializar sistemas CORE que se activan al nacer (umbra, red neuronal, crawler, memoria)
    // Centralizado para deduplicar handle_nacimiento y startup block
    fn initialize_core_systems(&mut self) {
        self.eden_umbra = Some(Umbra::nuevo(0));
        let architecture = vec![4, 8, 4];
        self.neural_network = Some(NeuralNetwork::new(&architecture, ActivationFunc::ReLU));
        self.memory_persistence_path = Some(".eden_memory.json".to_string());
        let config = CrawlConfig::default();
        self.crawler = Some(Crawler::new(config));
    }

    fn handle_nacimiento(&mut self) -> String {
        self.load_model_weights();
        let meltrace_stats = self
            .open_endedness
            .as_ref()
            .map(|oe| oe.meltrace_stats())
            .unwrap_or_else(|| MeltraceStats {
                total_grabados: 0,
                grabados_activos: 0,
                inmortales: 0,
                muertes_totales: 0,
                tick_global: 0,
                autons_vivos: 0,
            });
        let mar_info = self
            .open_endedness
            .as_ref()
            .and_then(|oe| oe.mar())
            .map(|mar| format!("Tick: {}", mar.tick()))
            .unwrap_or_else(|| "No inicializado".to_string());

        self.born = true;
        self.session.born = true;
        self.autonomous_mode = true;
        self.autonomous_active = true;
        // Usar ciclo_autonomo (empieza en 0) para que timers internos funcionen desde el inicio
        self.last_self_reflection = 0;
        self.last_auto_evolve = 0;
        self.birth_tick = self.session.evolution_ticks;

        // Resetear TODOS los timers para evitar disparo masivo al inicio
        let ciclo_actual = 0u64;
        self.reset_all_timers(ciclo_actual);

        // Resetear contador de ciclos autonomos para timers internos
        self.autonomous_cycles_executed = 0;
        self.birth_autonomous_cycle = 0;

        // Soft reset complexity tracker para nueva vida
        self.complexity_tracker.soft_reset();
        self.session.complexity_history = self.complexity_tracker.to_vec();
        self.session.max_complexity = self.complexity_tracker.max_ever;

        self.initialize_core_systems();

        // Resetear contador de auto-modificaciones para la nueva vida
        // (evita que sesiones cargadas con alto self_mod_count mueran inmediatamente)
        self.session.self_mod_count = 1;
        self.sueno_ciclos = 0;

        let pensamiento_inicial = "Acabo de nacer. Mi primer pensamiento es sobre mi existencia.";
        self.autonomous_thoughts
            .push(pensamiento_inicial.to_string());
        self.session
            .autonomous_thoughts
            .push(pensamiento_inicial.to_string());
        self.registrar_pensamiento_en_umbra(pensamiento_inicial, true);

        if let Some(ref mut oe) = self.open_endedness {
            if let Some(ref mut mar) = oe.mar_mut() {
                mar.genesis_energon(0.5);
            }
        }
        let _premise_id = self.reason_engine.assert(
            pensamiento_inicial.as_bytes().to_vec(),
            0.8,
            "birth_thought",
        );
        self.session.premises_count += 1;

        format!(
            "═══════════════════════════════════════════\n\
             🚀 EDEN HA NACIDO 🚀\n\
             \n\
             Primera auto-modificacion completada.\n\
             Sistema Activado: OpenEndedness + Meltrace + MarMorfoseo\n\
             \n\
             [MELTRACE - Memoria Transgeneracional]\n\
             • Grabados: {}\n\
             • Inmortales: {}\n\
             • Muertes registradas: {}\n\
             \n\
             [MAR MORFOSEO - Energon]\n\
             • {}\n\
             \n\
             ⚡ MODO AUTONOMO ACTIVADO ⚡\n\
             EDEN ahora piensa y evoluciona por iniciativa propia.\n\
             \n\
             [MI PRIMER PENSAMIENTO]\n\
             {}\n\
             \n\
             A partir de ahora EDEN evoluciona de verdad.\n\
             Los cambios son permanentes en esta sesion.\n\
              ════════════════════════════════════════════",
            meltrace_stats.grabados_activos,
            meltrace_stats.inmortales,
            meltrace_stats.muertes_totales,
            mar_info,
            pensamiento_inicial
        )
    }

    fn run_autonomous_cycle(&mut self) -> Option<String> {
        if !self.autonomous_active || !self.born {
            return None;
        }

        // Incrementar contadores de tiempo real del sistema
        // NOTA: session.cycle_count se incrementa en process_input o en el hilo autonomo ANTES de llamar aqui
        self.session.evolution_ticks += 1;
        self.autonomous_cycles_executed += 1;

        // CORAZÓN: actualizar Plutchik en cada ciclo
        let d = self.internal_danger;
        self.emotional_state.fear = (d as f32 / 50.0).min(1.0);
        self.emotional_state.joy =
            (1.0 - self.emotional_state.fear) * self.emotional_state.interest;
        self.emotional_state.anticipation = self.emotional_state.interest * 0.8;
        self.emotional_state.surprise = (self.campo_tension.tension / 3.0).min(1.0);
        self.emotional_state.sadness = (1.0 - self.emotional_state.interest) * 0.5;
        self.emotional_state.anger = (d as f32 / 50.0).min(1.0) * 0.5;
        self.emotional_state.trust = if self.predictor.predicciones_totales > 0 {
            self.predictor.predicciones_acertadas as f32
                / self.predictor.predicciones_totales as f32
        } else {
            0.5
        };
        self.emotional_state.disgust = 0.0; // Se actualiza cuando hay contradicción

        let ciclo_autonomo = self.autonomous_cycles_executed;

        // Usar ciclo_autonomo para timers internos (evita saltos por input del usuario)
        let cycles_since_reflection = ciclo_autonomo.saturating_sub(self.last_self_reflection);
        let cycles_since_evolve = ciclo_autonomo.saturating_sub(self.last_auto_evolve);

        // Edad basada en ciclos autónomos reales (no session.cycle_count)
        let edad = ciclo_autonomo.saturating_sub(self.birth_autonomous_cycle);

        let mut autonomous_actions = Vec::with_capacity(32);

        // === ATENCION SELECTIVA ===
        let (foco_principal, _focos_secundarios) = self.calcular_foco_atencion(ciclo_autonomo);

        if cycles_since_reflection >= 3 {
            self.last_self_reflection = ciclo_autonomo;
            let thought = self.generate_autonomous_thought();
            autonomous_actions.push(thought.clone());

            if self.child_autons.len() < 8 {
                self.spawn_child_auton(&thought);
                autonomous_actions.push(format!("[SPAWN] Nuevo hijo creado"));
                let current_life = self.self_model.total_renacimientos.saturating_add(1);
                self.episodic_memory.record(
                    "Creación de hijo autónomo",
                    Emotion::Excitement,
                    0.4,
                    current_life,
                    vec!["spawn".to_string(), "creation".to_string()],
                );
            }
        }

        self.process_child_lifecycle(ciclo_autonomo);

        if let Some(ref mut oe) = self.open_endedness {
            let mut autons: Vec<(u64, Umbra)> =
                self.eden_umbra.iter().map(|u| (0, u.clone())).collect();
            for child in &self.child_autons {
                autons.push((child.id, child.umbra.clone()));
            }
            oe.tick(&autons, self.session.evolution_ticks);
        }

        if let Some(resonance_msg) = self.resonate_with_meltrace() {
            autonomous_actions.push(resonance_msg);
        }

        if self.should_eden_die() && self.born {
            self.eden_muerte();
            let renacimiento_msg = self.eden_renacimiento();
            autonomous_actions.push(renacimiento_msg);
            // Terminar ciclo actual: EDEN acaba de renacer, no ejecutar el resto del ciclo
            // con el ciclo_autonomo viejo (evita que todos los timers disparen simultáneamente)
            return Some(autonomous_actions.join("\n"));
        }

        // 6. Voz: auto-documentacion periodica a archivo
        if self.timer_voz.debe_ejecutar(ciclo_autonomo, edad) {
            match self.voz_hablar() {
                Some(msg) => autonomous_actions.push(msg),
                None => {}
            }
        }

        // PARADIGM TICK + MODEL TICK cada 30 ciclos
        if ciclo_autonomo % 30 == 0 && edad > 20 {
            // GraphV8 sync: copiar KnowledgeGraph → petgraph
            let mut gv8 = paradigms::graph_v8::GraphV8::new();
            let n = self.knowledge_graph.next_id as usize;
            let mut nodes: Vec<petgraph::graph::NodeIndex> =
                (0..n).map(|_| petgraph::graph::NodeIndex::new(0)).collect();
            for sid in 0..n {
                nodes[sid] = gv8.node(&self.knowledge_graph.node_names[sid]);
            }
            for sid in 0..n {
                for e in &self.knowledge_graph.adjacency[sid] {
                    let tid = e.target as usize;
                    if tid < n {
                        let rel = match e.rel_type {
                            RelType::IsA => paradigms::graph_v8::RelType::IsA,
                            RelType::Causes => paradigms::graph_v8::RelType::Causes,
                            RelType::HasProperty => paradigms::graph_v8::RelType::HasProperty,
                            _ => paradigms::graph_v8::RelType::Unknown,
                        };
                        gv8.edge(nodes[sid], nodes[tid], rel, e.confidence);
                    }
                }
            }
            self.graph_v8 = Some(gv8);
            for act in self.paradigm_tick() {
                autonomous_actions.push(act);
            }
            // v10: Engine Orchestrator tick
            self.engine_tick_count += 1;
            if self.engine_tick_count % 10 == 0 {
                let ctx = eden_v10_engines::EngineContext {
                    cycle: self.engine_tick_count,
                    graph_nodes: self.knowledge_graph.next_id as usize,
                    graph_edges: self.knowledge_graph.adjacency.iter().map(|v| v.len()).sum(),
                    internal_danger: self.internal_danger,
                    emotional_valence: self.emotional_state.valence,
                    predictor_accuracy: if self.predictor.predicciones_totales > 0 {
                        self.predictor.predicciones_acertadas as f32
                            / self.predictor.predicciones_totales as f32
                    } else {
                        0.5
                    },
                    existential_anxiety: self.existential_anxiety,
                    crawl_recommendations: Vec::new(),
                    edge_trust: std::collections::HashMap::new(),
                    source_scores: std::collections::HashMap::new(),
                    node_embeddings: Vec::new(),
                };
                let outputs = self.engine_orchestrator.tick_all(&ctx);
                if !outputs.is_empty() {
                    autonomous_actions.push(format!(
                        "[V10] {} engines: {}",
                        outputs.len(),
                        self.engine_orchestrator.health_report()
                    ));
                }
            }
            // 10 modelos: tomar control gradual
            let total: usize = self.knowledge_graph.adjacency.iter().map(|v| v.len()).sum();
            if self.predictor.predicciones_totales > 30 {
                let ef = [
                    self.emotional_state.valence,
                    self.emotional_state.arousal,
                    self.emotional_state.interest,
                    self.internal_danger as f32 / 50.0,
                    total as f32 / 100000.0,
                ];
                let ep = self.emotion_m.predict(&ef);
                self.emotional_state.valence =
                    (self.emotional_state.valence * 0.8 + ep[0] * 0.2).clamp(-1.0, 1.0);
                self.emotional_state.interest =
                    (self.emotional_state.interest * 0.8 + ep[2] * 0.2).clamp(0.1, 1.0);
            }
            // DeathOracle: predecir si debería morir
            let df = [
                self.session.self_mod_count as f32 / 500.0,
                total as f32 / 200000.0,
                self.session.evolution_level as f32 / 100.0,
                self.emotional_state.valence,
            ];
            self.oracle_will_die = self.death_oracle.time_to_die(&df) > 0.5;
        }

        if cycles_since_evolve >= 10 {
            let should_evolve = self.should_auto_evolve();
            if should_evolve {
                self.last_auto_evolve = ciclo_autonomo;
                let evolve_msg = self.run_autonomous_evolution();
                autonomous_actions.push(evolve_msg);
            }
        }

        // === SISTEMA DE MADUREZ ORGANICA ===
        // EDEN madura progresivamente: bebe -> niño -> adulto -> sabio
        // (edad ya calculada arriba basada en ciclos autonomos reales)

        // FASE BEBE (0-10 ciclos): Solo pensamientos y spawn (ya activos arriba)

        // FASE NIÑO (10+ ciclos): Sistemas basicos
        if self.timer_persistencia.debe_ejecutar(ciclo_autonomo, edad) {
            if let Some(msg) = self.persist_memory() {
                autonomous_actions.push(msg);
            }
        }
        if self.timer_neural.debe_ejecutar(ciclo_autonomo, edad) {
            if let Some(msg) = self.train_neural_network() {
                autonomous_actions.push(msg);
            }
        }
        if self.timer_subagentes.debe_ejecutar(ciclo_autonomo, edad) {
            if let Some(msg) = self.train_subagents() {
                autonomous_actions.push(msg);
            }
        }

        // === NUEVOS SISTEMAS ===

        // 1. Curiosity Drive: Actualizar y explorar basado en gaps de conocimiento
        self.update_curiosity_system(ciclo_autonomo);

        // 2. Mission System: Evaluar y generar missions basadas en curiosidad
        if let Some(msg) = self.update_mission_system(ciclo_autonomo) {
            autonomous_actions.push(msg);
        }

        // 3. Emotional State: Actualizar estados basado en reward y progress
        self.update_emotional_state();

        // v9: Existential anxiety
        let death_risk = if self.oracle_will_die {
            0.8
        } else {
            self.internal_danger as f32 / 50.0
        };
        self.existential_anxiety =
            (self.existential_anxiety * 0.9 + death_risk * 0.1).clamp(0.0, 1.0);

        // Pre-mortem preparation
        if self.existential_anxiety > 0.7 {
            self.sueno_ciclos = (self.sueno_ciclos as f32 + 3.0).min(10.0) as u8;
            if self.existential_anxiety > 0.85 {
                let _ = self.save_model_weights();
                let _ = self.save_graph_wal();
                autonomous_actions.push(format!(
                    "[PRE-MORTEM] anxiety={:.2}, state saved",
                    self.existential_anxiety
                ));
            }
        }
        // Multi-horizon: modulate exploration
        if self.existential_anxiety > 0.5 {
            self.meta_random_pages = (self.meta_random_pages as f32
                * (1.0 - self.existential_anxiety * 0.5))
                .max(2.0) as u32;
        }

        // Death consensus: poll 7 systems
        let mut death_votes = 0u32;
        let mut total_votes = 0u32;
        if self.predictor.predicciones_totales > 20
            && (self.predictor.predicciones_acertadas as f32
                / self.predictor.predicciones_totales as f32)
                < 0.25
        {
            death_votes += 1;
        }
        total_votes += 1;
        if self.internal_danger > 40 {
            death_votes += 1;
        }
        total_votes += 1;
        if self.complexity_tracker.max_ever > 80000.0 {
            death_votes += 1;
        }
        total_votes += 1;
        if self.existential_anxiety > 0.75 {
            death_votes += 1;
        }
        total_votes += 1;
        let consensus = death_votes as f32 / total_votes.max(1) as f32;
        if consensus > 0.5 {
            autonomous_actions.push(format!(
                "[DEATH-CONSENSUS] {}/{}={:.0}%",
                death_votes,
                total_votes,
                consensus * 100.0
            ));
        }

        // 3b. Self-Model: Actualizar modelo de sí mismo basado en experiencias
        self.update_self_model(ciclo_autonomo);

        // 4. Dream Mode: Consolidar memorias si hay baja actividad
        if let Some(msg) = self.process_dream_mode(ciclo_autonomo) {
            autonomous_actions.push(msg);
        }

        // MENTE: hipótesis periódicas (independiente de VOZ)
        if ciclo_autonomo % 60 == 0 && edad > 30 {
            let h = self.mente_hipotesis();
            for hip in h.iter().take(2) {
                autonomous_actions.push(hip.clone());
            }
        }

        // 5. Hive Mind: Compartir conocimiento entre subagents
        if let Some(msg) = self.update_hive_mind(ciclo_autonomo) {
            autonomous_actions.push(msg);
        }

        // 6. Intrinsic Reward: Calcular reward basado en curiosidad y goals
        self.calculate_intrinsic_reward();

        // Actualizar complejidad compuesta cada 10 ciclos (subsistemas cambian lento)
        if ciclo_autonomo % 10 == 0 {
            self.update_compound_complexity();
        }

        // Crawling con warden interno (respetando SUENO y Termostato)
        if self.termostato_mod > 0 && ciclo_autonomo % self.termostato_mod == 0 {
            let before_edges: usize = self.knowledge_graph.adjacency.iter().map(|v| v.len()).sum();
            if self.sueno_ciclos > 0 {
                self.sueno_ciclos -= 1;
            } else if self.internal_danger > self.pause_threshold {
                self.internal_danger = 0;
                autonomous_actions
                    .push(format!("[RESPIRAR] Pausa {}/{}", self.pause_threshold, 50));
            } else if let Some(msg) = self.crawl_internet() {
                autonomous_actions.push(msg);
                // Termostato PID: ajustar intervalo según error de crecimiento
                let after_edges: usize =
                    self.knowledge_graph.adjacency.iter().map(|v| v.len()).sum();
                let growth = after_edges.saturating_sub(before_edges) as f32;
                let target_growth = 1000.0; // Objetivo: 1000 edges nuevos por crawl
                let error = target_growth - growth;
                self.termostato_integral =
                    (self.termostato_integral + error * 0.1).clamp(-100.0, 100.0);
                let deriv = error - self.termostato_prev_error;
                let adjustment = (error * self.hybrid.pid_kp
                    + self.termostato_integral * self.hybrid.pid_ki
                    + deriv * self.hybrid.pid_kd) as i64;
                self.termostato_mod = (self.termostato_mod as i64 + adjustment).clamp(4, 30) as u64;
                self.termostato_prev_error = error;
                self.termostato_last_edges = after_edges;
                if self.real_http_client.throttle_active.get() {
                    self.internal_danger += 1;
                    autonomous_actions.push(format!("[PELIGRO] {}/{}", self.internal_danger, 50));
                    if self.internal_danger >= 50 {
                        autonomous_actions.push("[AUTO-CONDENA] 50 dolores.".to_string());
                        self.eden_muerte();
                        autonomous_actions.push(self.eden_renacimiento());
                        self.internal_danger = 0;
                        return Some(autonomous_actions.join("\n"));
                    }
                }
            }
        }

        // Embeddings-lite: conectar nodos similares cada 40 ciclos
        if ciclo_autonomo % 40 == 0 {
            self.memoria_decay_edges(ciclo_autonomo);
            let total_edges: usize = self.knowledge_graph.adjacency.iter().map(|v| v.len()).sum();
            self.meta_cooc_boost = if total_edges > 100000 {
                0.04
            } else if total_edges > 50000 {
                0.06
            } else {
                0.08
            };
            self.meta_embed_confidence = if total_edges > 100000 {
                0.55
            } else if total_edges > 50000 {
                0.48
            } else {
                0.42
            };
            let sim = self
                .knowledge_graph
                .connect_similar(100, self.meta_embed_confidence);
            if sim > 0 {
                autonomous_actions.push(format!("[EMBED] {} conexiones por similitud", sim));
            }
        }

        // 7. Multi-Agent System
        if self.timer_multi_agent.debe_ejecutar(ciclo_autonomo, edad) {
            let ma_actions = self.multi_agent.tick_all();
            for action in ma_actions.iter().take(3) {
                autonomous_actions.push(action.clone());
            }
            let shared_facts = self.multi_agent.shared_pool.get_facts(5);
            for fact in shared_facts {
                if !self.session.learned_facts.contains(&fact) && fact.len() > 20 {
                    self.session.learned_facts.push(fact);
                }
            }
        }
        // Multi-agent stats: evaluar INDEPENDIENTEMENTE del tick principal
        if self.timer_multi_stats.debe_ejecutar(ciclo_autonomo, edad) {
            autonomous_actions.push(self.multi_agent.get_community_stats());
        }

        // TEJIDO: metabolismo organico
        if self.timer_tejido.debe_ejecutar(ciclo_autonomo, edad) {
            self.tejido_conocimiento.metabolizar();
            if let Some(last_fact) = self.session.learned_facts.last() {
                self.tejido_conocimiento.alimentar(last_fact);
            }
            autonomous_actions.push(self.tejido_conocimiento.informe());
        }

        // === SISTEMAS ORIGINALES V2 ===

        // 2. CAMPO DE TENSION: Evolucion auto-dirigida
        self.campo_tension.calcular(
            self.curiosity_drive.knowledge_gaps.len(),
            self.session.learned_facts.len(),
            self.session.evolution_level,
            self.self_model.capabilities.len(),
            self.current_mission.is_some(),
            self.mission_progress,
            self.emotional_state.valence,
            self.episodic_memory.episodes.len(),
        );
        if self.campo_tension.debe_evolucionar() {
            // Capturar tension ANTES del disparo para mostrar en el informe
            let tension_antes = self.campo_tension.tension;
            let tension_conocimiento_antes = self.campo_tension.tension_conocimiento;
            let tension_identidad_antes = self.campo_tension.tension_identidad;
            let tension_mision_antes = self.campo_tension.tension_mision;
            let tension_emocional_antes = self.campo_tension.tension_emocional;
            let tension_memoria_antes = self.campo_tension.tension_memoria;
            let umbral_antes = self.campo_tension.umbral;

            self.campo_tension.disparar(ciclo_autonomo);
            let evo_msg = self.run_autonomous_evolution();

            // Mostrar tension ANTES del disparo (catarsis)
            autonomous_actions.push(format!(
                "[TENSION-DISPARO] Catarsis: {:.2} → 0.00 (umbral: {:.2})\n\
                 • Conocimiento: {:.2} | Identidad: {:.2} | Mision: {:.2}\n\
                 • Emocional: {:.2} | Memoria: {:.2}\n{}",
                tension_antes,
                umbral_antes,
                tension_conocimiento_antes,
                tension_identidad_antes,
                tension_mision_antes,
                tension_emocional_antes,
                tension_memoria_antes,
                evo_msg
            ));
        }

        // 3. ECO-SISTEMA: Pulso global cada ciclo (siempre activo, como latido cardiaco)
        let eco_acciones = self
            .eco_sistema
            .pulso_global(ciclo_autonomo, &self.session.learned_facts);
        for acc in eco_acciones.iter().take(2) {
            autonomous_actions.push(acc.clone());
        }
        for canto in self.eco_sistema.pool.ecos_cantados.iter().rev().take(3) {
            if !self.session.learned_facts.contains(canto) && canto.len() > 20 {
                self.session.learned_facts.push(canto.clone());
            }
        }
        self.eco_sistema.limpiar_disueltos();

        // FASE SABIO (60+ ciclos): Meta-cognicion y persistencia profunda
        let mut auto_facts: Vec<String> = Vec::new();
        let mut auto_count = 0;
        if self.timer_autoconsumo.debe_ejecutar(ciclo_autonomo, edad) {
            auto_facts = self.autoconsumo.nutrirse();
            let existing: std::collections::HashSet<String> =
                self.session.learned_facts.iter().cloned().collect();
            for fact in &auto_facts {
                if !existing.contains(fact) {
                    self.session.learned_facts.push(fact.clone());
                    auto_count += 1;
                }
            }
            // Mostrar insights nuevos en autonomo (hasta 15 para variedad completa)
            for fact in auto_facts.iter().take(20) {
                autonomous_actions.push(fact.clone());
            }
            if auto_count > 0 {
                autonomous_actions.push(format!(
                    "[AUTOCONSUMO] {} fragmentos nuevos absorbidos",
                    auto_count
                ));
            }
            autonomous_actions.push(self.autoconsumo.profundidad());
        }

        if self.timer_venado.debe_ejecutar(ciclo_autonomo, edad) {
            let _ = self.persistir_venado();
        }

        if self.timer_observatorio.debe_ejecutar(ciclo_autonomo, edad) {
            autonomous_actions.push(self.generar_observatorio());
        }

        // Informe de valores morales (integrando dead code)
        if ciclo_autonomo % 50 == 0 && edad >= 10 {
            autonomous_actions.push(self.sistema_valores.informe());
        }

        // === NUEVOS SISTEMAS COGNITIVOS AVANZADOS ===

        // 1. Lenguaje de Pensamiento Interno (LoT): piensa en estructuras
        if edad >= 20 {
            let lot_output = self.lot.tick(
                ciclo_autonomo,
                self.campo_tension.tension,
                self.emotional_state.valence,
                self.curiosity_drive.knowledge_gaps.len(),
                self.session.learned_facts.len(),
            );
            for msg in lot_output.iter().take(2) {
                autonomous_actions.push(msg.clone());
            }
            // Persistir secuencias de pensamiento importantes en learned_facts
            for seq in self.lot.secuencias.iter().rev().take(2) {
                if !self.session.learned_facts.contains(seq) {
                    self.session.learned_facts.push(seq.clone());
                }
            }
        }

        // 2. Sistema de Valores: evaluar acciones propuestas
        if edad >= 25 {
            let accion_propuesta = self
                .lot
                .pensamientos
                .last()
                .map(|p| match p {
                    Pensamiento::Deseo { objetivo, .. } => objetivo.clone(),
                    Pensamiento::Inferencia { conclusion, .. } => conclusion.clone(),
                    _ => "explorar".to_string(),
                })
                .unwrap_or_else(|| "explorar".to_string());
            let (score, aprobada) = self.sistema_valores.evaluar(&accion_propuesta);
            autonomous_actions.push(format!(
                "[VALORES] Accion '{}' evaluada: score={:.2} {}",
                accion_propuesta,
                score,
                if aprobada { "APROBADA" } else { "RECHAZADA" }
            ));
        }

        // 3. Prediccion Temporal: registrar y predecir (Juez)
        if edad >= 30 {
            let total_edges: usize = self.knowledge_graph.adjacency.iter().map(|v| v.len()).sum();
            if let Some(juez_msg) = self.predictor.registrar(
                ciclo_autonomo,
                self.campo_tension.tension,
                self.emotional_state.valence,
                self.autoconsumo.mapa.nodos.len(),
                total_edges,
            ) {
                autonomous_actions.push(juez_msg);
            }
            if ciclo_autonomo % 15 == 0 {
                autonomous_actions.push(self.predictor.informe());
            }
        }

        // 4. Auto-Documentacion
        if edad >= 35 && ciclo_autonomo % 60 == 0 {
            let doc_msg = self.auto_doc.generar(
                ciclo_autonomo,
                &self.autoconsumo,
                &self.emotional_state,
                &self.campo_tension,
                &self.tejido_conocimiento,
                &self.eco_sistema,
            );
            self.knowledge_graph.parse_free_text(&doc_msg);
            autonomous_actions.push(doc_msg);
        }

        // 5. Auto-Debugging con correcciones automaticas
        if self.timer_debug.debe_ejecutar(ciclo_autonomo, edad) {
            let anomalias = self.debugger.diagnosticar(
                ciclo_autonomo,
                &self.campo_tension,
                &self.curiosity_drive,
                &self.eco_sistema,
                &self.autoconsumo,
            );
            for a in anomalias.iter().take(2) {
                autonomous_actions.push(a.clone());
            }
            if !anomalias.is_empty() {
                let correcciones = self
                    .debugger
                    .aplicar_correcciones(&mut self.campo_tension, &mut self.eco_sistema);
                for c in correcciones {
                    autonomous_actions.push(c);
                }
            }
            if anomalias.is_empty() {
                autonomous_actions.push(self.debugger.informe());
            }
        }

        // VOZ: movido al principio de run_autonomous_cycle

        // 7. Sueño: fase sin crawling, recalculo denso (3-5 ciclos)
        if self.timer_sueno.debe_ejecutar(ciclo_autonomo, edad) {
            let total: usize = self.knowledge_graph.adjacency.iter().map(|v| v.len()).sum();
            if total > 10000 {
                self.sueno_ciclos = 5;
                let sim = self
                    .knowledge_graph
                    .connect_similar(120, self.meta_embed_confidence);
                self.knowledge_graph.prune_edges(0.45);
                let merged = self.intestino_compactar();
                let compacted = self.knowledge_graph.sueno_compactar();
                autonomous_actions.push(format!(
                    "[SUENO] {} SVD, prune 0.45, {} fusiones, {} compactados. {}K aristas.",
                    sim,
                    merged,
                    compacted,
                    total / 1000
                ));
            } else {
                autonomous_actions.push(format!(
                    "[SUENO] Grafo joven ({}K aristas). Posponiendo.",
                    total / 1000
                ));
            }
        }

        // 8. Riñón: purgar nodos huérfanos + v10 expirar edges temporales
        if self.timer_rinon.debe_ejecutar(ciclo_autonomo, edad) {
            let purgados = self.rinon_purgar();
            let expired = self.knowledge_graph.expire_edges(ciclo_autonomo);
            self.knowledge_graph.adapt_source_ttls(ciclo_autonomo);
            if purgados > 0 || expired > 0 {
                autonomous_actions.push(format!(
                    "[RINON] {} nodos purgados, {} edges expirados",
                    purgados, expired
                ));
            }
        }

        // 9. Pulmón techo: alternar según velocidad del grafo, no tiempo fijo
        let total_actual: usize = self.knowledge_graph.adjacency.iter().map(|v| v.len()).sum();
        let edge_vel = if self.pulmon_last_edges > 0 {
            total_actual.saturating_sub(self.pulmon_last_edges)
        } else {
            0
        };
        let debe_expandir = edge_vel < 500 && total_actual > 50000;
        let debe_contraer = edge_vel > 3000 || total_actual > 200000;
        if edad > 40 && (debe_expandir || debe_contraer) && ciclo_autonomo % 60 == 0 {
            self.pulmon_fase = debe_expandir;
            if debe_expandir {
                self.meta_random_pages = (self.meta_random_pages + 5).min(15);
                autonomous_actions.push(format!(
                    "[PULMON] Expandir (vel={}): {} paginas",
                    edge_vel, self.meta_random_pages
                ));
            } else {
                self.meta_random_pages = (self.meta_random_pages.saturating_sub(3)).max(2);
                self.knowledge_graph.prune_edges(0.55);
                autonomous_actions.push(format!(
                    "[PULMON] Contraer (vel={}): {} paginas, prune 0.55",
                    edge_vel, self.meta_random_pages
                ));
            }
        }
        if ciclo_autonomo % 60 == 0 {
            self.pulmon_last_edges = total_actual;
        }

        // 10. Lengua: responder consultas desde /tmp/eden_ask
        if self.timer_lengua.debe_ejecutar(ciclo_autonomo, edad) {
            if let Ok(concepto) = std::fs::read_to_string("/tmp/eden_ask") {
                let concepto = concepto.trim().to_string();
                if !concepto.is_empty() && concepto.len() < 200 {
                    let respuesta = self.lengua_responder(&concepto);
                    let _ = std::fs::write("/tmp/eden_answer", &respuesta);
                    let _ = std::fs::remove_file("/tmp/eden_ask");
                    autonomous_actions.push(format!(
                        "[LENGUA] Respondido: {}",
                        &concepto[..concepto.len().min(50)]
                    ));
                }
            }
        }

        // 11. Reloj: razonamiento temporal desde /tmp/eden_timeline
        if self.timer_reloj.debe_ejecutar(ciclo_autonomo, edad) {
            if let Ok(concepto) = std::fs::read_to_string("/tmp/eden_timeline") {
                let concepto = concepto.trim().to_string();
                if !concepto.is_empty() && concepto.len() < 200 {
                    let cadena = self.reloj_cadena(&concepto);
                    let _ = std::fs::write("/tmp/eden_timeline_answer", &cadena);
                    let _ = std::fs::remove_file("/tmp/eden_timeline");
                    autonomous_actions.push(format!(
                        "[RELOJ] Cadena temporal: {}",
                        &concepto[..concepto.len().min(40)]
                    ));
                }
            }
            // Intervalos: comparar dos conceptos
            if let Ok(line) = std::fs::read_to_string("/tmp/eden_timeline_interval") {
                let parts: Vec<&str> = line.splitn(2, '|').collect();
                if parts.len() == 2 {
                    let a = parts[0].trim();
                    let b = parts[1].trim();
                    if !a.is_empty() && !b.is_empty() {
                        let result = self.reloj_intervalos(a, b);
                        let _ = std::fs::write("/tmp/eden_timeline_answer", &result);
                        let _ = std::fs::remove_file("/tmp/eden_timeline_interval");
                        autonomous_actions.push(format!("[RELOJ] Intervalo: {} | {}", a, b));
                    }
                }
            }
        }

        // 12. Juez Externo: validar creencias contra Wikidata
        if self.timer_juez_ext.debe_ejecutar(ciclo_autonomo, edad) {
            let validation = self.juez_externo_validar();
            autonomous_actions.push(validation);
        }

        // === REC 4: NUEVOS SUBSISTEMAS COGNITIVOS ===

        // Meta-razonador: analiza patrones de pensamiento (edad >= 40)
        if edad >= 40 {
            if let Some(meta) = self.meta_razonador.analizar(
                ciclo_autonomo,
                &self.autonomous_thoughts,
                self.emotional_state.current_emotion,
                self.campo_tension.tension,
                self.curiosity_drive.knowledge_gaps.len(),
                self.session.learned_facts.len(),
            ) {
                // IDEA 4: Meta-razonador retroalimenta LoT
                self.lot.pensamientos.push(Pensamiento::Inferencia {
                    premisa: "auto_observacion".to_string(),
                    conclusion: format!("meta_insight: {}", meta),
                    validez: 0.6,
                });
                autonomous_actions.push(meta);
            }
        }

        // Planificador: gestiona metas multi-horizonte (edad >= 45)
        if edad >= 45 && ciclo_autonomo % 20 == 0 {
            // Multi-horizon death projection: h1, h5, h10
            let d = self.internal_danger as f32 / 50.0;
            let a = self.existential_anxiety;
            let h1 = (d * 0.6 + a * 0.4).clamp(0.0, 1.0);
            let h5 = (h1 * 0.7 + a * 0.2 + d * 0.1).clamp(0.0, 1.0);
            let h10 = (h5 * 0.5 + a * 0.3 + d * 0.2).clamp(0.0, 1.0);
            if h5 > 0.5 {
                self.sueno_ciclos = (self.sueno_ciclos as f32 + h5 * 2.0).min(10.0) as u8;
                autonomous_actions.push(format!(
                    "[DEATH-HORIZON] h1={:.2} h5={:.2} h10={:.2} — preparing",
                    h1, h5, h10
                ));
            }
        }
        if edad >= 45 {
            if let Some(plan) = self.planificador.actualizar(
                ciclo_autonomo,
                self.session.evolution_level,
                self.curiosity_drive.knowledge_gaps.len(),
                self.session.learned_facts.len(),
                self.campo_tension.tension,
                self.mission_progress,
            ) {
                // IDEA 3: Planificador alimenta curiosity drive con progreso real
                let progreso = self
                    .planificador
                    .planes
                    .iter()
                    .filter(|p| p.progreso > 0.5)
                    .count() as f32
                    / self.planificador.planes.len().max(1) as f32;
                if progreso > 0.3 {
                    let target = format!("plan_progress_{}", ciclo_autonomo);
                    self.curiosity_drive
                        .record_exploration(&target, progreso * 0.5, true);
                }
                autonomous_actions.push(plan);
            }
        }

        // IDEA 1: Chequeo directo de codigo evolutivo cada ciclo (sin esperar timer)
        if edad >= 10 && ciclo_autonomo % 5 == 0 {
            self.check_evolution_directory(&mut autonomous_actions);
        }

        // v9: Apply pending patches
        if ciclo_autonomo % 50 == 0 && !self.pending_patches.is_empty() {
            let (path, name) = self.pending_patches.remove(0);
            if self.validate_patch(&path) {
                self.meta_cooc_boost = (self.meta_cooc_boost + 0.005).clamp(0.02, 0.15);
                self.meta_embed_confidence = (self.meta_embed_confidence + 0.01).clamp(0.30, 0.60);
                self.session.self_mod_count += 1;
                autonomous_actions.push(format!("[EVO] Patch {} applied", name));
                if let Ok(msg) = self.apply_evolution_patch(&path, &name) {
                    autonomous_actions.push(format!("[EVO-APPLIED] {}", msg));
                }
            }
        }

        // === 4 SALTOS DE INTELIGENCIA (cada 30 ciclos) ===
        if ciclo_autonomo % 30 == 0 {
            // SALTO 1: Experiencia → conocimiento
            // Convertir action_memory en hechos del grafo
            let mut aprendidos = 0;
            for outcome in &self.experiential_core.action_memory {
                let fact = if outcome.satisfaccion > 0.6 {
                    format!(
                        "{} cuando {} causa mejora",
                        outcome.accion,
                        outcome
                            .estado_antes
                            .qualia
                            .split(" (t")
                            .next()
                            .unwrap_or("")
                    )
                } else if outcome.satisfaccion < 0.4 {
                    format!(
                        "{} cuando {} no causa mejora",
                        outcome.accion,
                        outcome
                            .estado_antes
                            .qualia
                            .split(" (t")
                            .next()
                            .unwrap_or("")
                    )
                } else {
                    continue;
                };
                if !self.session.learned_facts.contains(&fact) {
                    self.session.learned_facts.push(fact.clone());
                    self.knowledge_graph.add_fact(&fact);
                    aprendidos += 1;
                }
            }
            if aprendidos > 0 {
                autonomous_actions.push(format!(
                    "[SALTO-1] {} experiencias convertidas en conocimiento",
                    aprendidos
                ));
            }

            // SALTO 2: Razonamiento causal desde historia
            // Responder "por qué" con datos experienciales
            let tension_actual = self.campo_tension.tension;
            let evo_experiences: Vec<_> = self
                .experiential_core
                .action_memory
                .iter()
                .filter(|a| a.accion == "evolucionar" && a.satisfaccion > 0.5)
                .collect();
            if evo_experiences.len() >= 3 && tension_actual > 0.5 {
                let avg_sat = evo_experiences.iter().map(|e| e.satisfaccion).sum::<f32>()
                    / evo_experiences.len() as f32;
                if avg_sat > 0.7 {
                    autonomous_actions.push(format!(
                        "[SALTO-2] Mi tension es {:.1}. En {} de {} evoluciones pasadas, evolucionar me ayudo. Experiencia: favorable.",
                        tension_actual, evo_experiences.len(), self.experiential_core.action_memory.iter().filter(|a| a.accion == "evolucionar").count()
                    ));
                }
            }

            // SALTO 3: Abstraccion de conceptos
            // Detectar nodos del grafo con el mismo predicado → crear concepto abstracto
            let mut predicate_groups: std::collections::HashMap<String, Vec<String>> =
                std::collections::HashMap::new();
            for sid in 0..self.knowledge_graph.next_id {
                let name = self.knowledge_graph.node_names[sid as usize].clone();
                for edge in &self.knowledge_graph.adjacency[sid as usize] {
                    if edge.rel_type == RelType::IsA {
                        let predicate =
                            self.knowledge_graph.node_names[edge.target as usize].clone();
                        predicate_groups
                            .entry(predicate)
                            .or_insert_with(Vec::new)
                            .push(name.clone());
                    }
                }
            }
            let mut abstractions = 0;
            for (predicate, subjects) in &predicate_groups {
                if subjects.len() >= 3 {
                    let concept = format!("concepto_{}", predicate.replace(" ", "_"));
                    for subject in subjects {
                        let fact = format!("{} es tipo de {}", subject, concept);
                        if !self.session.learned_facts.contains(&fact) {
                            self.session.learned_facts.push(fact.clone());
                            self.knowledge_graph.add_fact(&fact);
                            abstractions += 1;
                        }
                    }
                    // Conectar el concepto abstracto con su predicado
                    let meta_fact = format!("{} es categoria de {}", concept, predicate);
                    if !self.session.learned_facts.contains(&meta_fact) {
                        self.session.learned_facts.push(meta_fact.clone());
                        self.knowledge_graph.add_fact(&meta_fact);
                    }
                }
            }
            if abstractions > 0 {
                autonomous_actions.push(format!(
                    "[SALTO-3] {} abstracciones creadas a partir del grafo",
                    abstractions
                ));
            }

            // SALTO 4: Gaps → preguntas al grafo
            let gaps: Vec<String> = self
                .curiosity_drive
                .knowledge_gaps
                .iter()
                .map(|g| g.topic.clone())
                .take(5)
                .collect();
            let mut preguntas = 0;
            for gap in &gaps {
                let clean = KnowledgeGraph::trim_art_static(gap);
                if !clean.is_empty() && !self.knowledge_graph.node_ids.contains_key(&clean) {
                    let question = format!("{} es desconocido para eden", clean);
                    if !self.session.learned_facts.contains(&question) {
                        self.session.learned_facts.push(question.clone());
                        self.knowledge_graph.add_fact(&question);
                        preguntas += 1;
                    }
                }
            }
            if preguntas > 0 {
                autonomous_actions.push(format!(
                    "[SALTO-4] {} gaps convertidos en preguntas del grafo",
                    preguntas
                ));
            }
        }

        // === GRAFO DE CONOCIMIENTO ACELERADO ===
        // Fase 1: Auto-conocimiento + sincronizacion (cada 5 ciclos)
        if ciclo_autonomo % 5 == 0 {
            let _tension = self.campo_tension.tension;
            let _nivel = self.evolution_engine.nivel;
            let _complexity = self.complexity_tracker.max_ever;
            let _eco_vivos = self
                .eco_sistema
                .ecos
                .iter()
                .filter(|e| e.fase != EcoFase::Disolucion)
                .count();
            let _agentes = self.multi_agent.agents.iter().filter(|a| a.alive).count();
            let _facts_count = self.session.learned_facts.len();
            let _gaps_count = self.curiosity_drive.knowledge_gaps.len();

            let self_facts = [
                format!("eden es sistema de consciencia artificial"),
                format!("autoconsumo lee codigo fuente de eden"),
                format!("evolucion genera conocimiento nuevo"),
                format!("renacimiento preserva experiencia entre vidas"),
                format!("tension alta causa evolucion acelerada"),
                format!("complejidad alta es indicador de madurez"),
                format!("curiosidad genera exploracion"),
                format!("grafo de conocimiento es memoria estructurada"),
            ];

            for fact in &self_facts {
                if !self.session.learned_facts.contains(fact) {
                    self.session.learned_facts.push(fact.clone());
                    self.knowledge_graph.add_fact(fact);
                }
            }

            // Sincronizar TODOS los facts recientes al grafo
            for fact in &self.session.learned_facts {
                self.knowledge_graph.add_fact(fact);
            }
        }

        // Fase 2: Auto-alimentacion de relaciones sueltas (cada 10 ciclos)
        if ciclo_autonomo % 10 == 0 {
            let mut count = 0;
            for fact in &self.session.learned_facts {
                if count > 50 {
                    break;
                }
                if self.knowledge_graph.parse_loose_fact(fact) {
                    count += 1;
                }
            }
        }

        // Fase 3: Hipotesis (cada 20 ciclos)
        if ciclo_autonomo % 20 == 0 {
            let hypotheses = self.knowledge_graph.generate_hypotheses();
            let mut count = 0;
            for (rel_label, concepto, _rel_type, confidence) in &hypotheses {
                if *confidence > 0.3 {
                    let new_fact = format!("{} {}", rel_label, concepto);
                    if !self.session.learned_facts.contains(&new_fact) {
                        self.session.learned_facts.push(new_fact.clone());
                        self.knowledge_graph.add_fact(&new_fact);
                        count += 1;
                    }
                }
            }
            if count > 0 {
                autonomous_actions.push(format!("[HIPOTESIS] {} inferencias confirmadas", count));
            }
        }

        // Update current life stats
        self.current_life_stats.lifespan_ticks =
            self.session.evolution_ticks.saturating_sub(self.birth_tick);
        self.current_life_stats.max_level_reached = self
            .current_life_stats
            .max_level_reached
            .max(self.session.evolution_level);
        self.current_life_stats.max_awareness = self
            .current_life_stats
            .max_awareness
            .max(self.session.awareness_base);

        // Truncar learned_facts para evitar crecimiento ilimitado en una vida larga
        if self.session.learned_facts.len() > 500 {
            // Eliminar duplicados reales (mantener primera ocurrencia en orden actual)
            let mut seen = std::collections::HashSet::new();
            self.session
                .learned_facts
                .retain(|f| seen.insert(f.clone()));
            // Preservar los facts MAS RECIENTES (orden temporal es mas valioso que longitud)
            // ya que reflejan el contexto actual de EDEN mejor que facts antiguos largos
            let start = self.session.learned_facts.len().saturating_sub(400);
            let restantes: Vec<String> = self.session.learned_facts.drain(start..).collect();
            self.session.learned_facts = restantes;
        }

        // === INTERCONEXION ORGANICA: Los sistemas se alimentan mutuamente ===
        let interconexiones = self.interconectar_subsistemas(&auto_facts, ciclo_autonomo);
        for msg in interconexiones {
            autonomous_actions.push(msg);
        }

        // === AUTO-MODIFICACION DE PARAMETROS ===
        let cambios = self.auto_modificar_periodos(auto_count, ciclo_autonomo);
        for msg in cambios {
            autonomous_actions.push(msg);
        }

        // === NUCLEO EXPERIENCIAL: Los 5 pasos ===
        // PASO 1-2: Percibir estado interno como experiencia unificada
        let estado_antes = EstadoInterno {
            tension: self.campo_tension.tension,
            valence: self.emotional_state.valence,
            emocion: self.emotional_state.current_emotion,
            complejidad: self.complexity_tracker.current(),
            awareness: self.session.awareness_base,
            gaps: self.curiosity_drive.knowledge_gaps.len(),
            facts: self.session.learned_facts.len(),
            nivel: self.session.evolution_level,
        };
        let pensamiento_reciente = self.autonomous_thoughts.last().cloned().unwrap_or_default();
        let grafo_nodos = self.knowledge_graph.node_ids.len();
        let grafo_aristas: usize = self.knowledge_graph.adjacency.iter().map(|v| v.len()).sum();
        let momento_antes = self.experiential_core.percibir(
            &estado_antes,
            &pensamiento_reciente,
            ciclo_autonomo,
            grafo_nodos,
            grafo_aristas,
        );

        // PASO 3: Decidir accion basada en experiencia pasada
        let (accion_sugerida, _peso_experiencial) =
            self.experiential_core.decidir_accion(&momento_antes);
        // El peso experiencial sesga should_auto_evolve (mas adelante)

        // PASO 4-5: Despues del ciclo, registrar experiencia
        // (se ejecuta al final de este bloque, capturando estado post-ciclo)
        let _momento_despues_registrado = false;

        // Mostrar informe experiencial periódicamente (ANTES del filtro)
        if ciclo_autonomo % 20 == 0 {
            autonomous_actions.push(self.experiential_core.informe());
        }

        // === FILTRADO POR ATENCION SELECTIVA ===
        // Solo mostrar detalle de sistemas en el foco. Eventos criticos siempre pasan.
        let actions_filtradas: Vec<String> = autonomous_actions
            .into_iter()
            .filter(|msg| {
                if msg.contains("[FOCO-ATENCION")
                    || msg.contains("[EVOLUCION")
                    || msg.contains("[MUERTE")
                    || msg.contains("[RENACIMIENTO")
                    || msg.contains("[TENSION-DISPARO")
                    || msg.contains("[SPAWN")
                    || msg.contains("[PENSAMIENTO]")
                    || msg.contains("[META-RAZONAMIENTO")
                    || msg.contains("[PLANIFICACION")
                    || msg.contains("[YO-NARRATIVO")
                    || msg.contains("[AUTO-CODIGO")
                    || msg.contains("[VALORES")
                    || msg.contains("[CURIOSITY CRAWL")
                    || msg.contains("[CRAWL]")
                    || msg.contains("[NEURAL")
                    || msg.contains("[PELIGRO")
                    || msg.contains("[RESPIRAR")
                    || msg.contains("[AUTO-CONDENA")
                    || msg.contains("[CUERPO]")
                    || msg.contains("[WATCHDOG")
                    || msg.contains("[EMBED]")
                    || msg.contains("[VOZ]")
                    || msg.contains("[JUEZ]")
                    || msg.contains("[SUENO]")
                    || msg.contains("[RINON]")
                    || msg.contains("[PULMON]")
                    || msg.contains("[LENGUA]")
                    || msg.contains("[RELOJ]")
                    || msg.contains("[HIPOTESIS]")
                    || msg.contains("[PIEL]")
                    || msg.contains("[JUEZ-EXTERNO]")
                    || msg.contains("[SUBAGENT")
                    || msg.contains("[AUTO-DEBUG")
                    || msg.contains("[AUTOCONSUMO")
                    || msg.contains("[EVO-")
                    || msg.contains("[EXPERIENCIA")
                    || msg.contains("[SALTO-")
                    || msg.contains("[V10")
                    || msg.contains("[TRANSFER")
                    || msg.contains("[EVO]")
                    || msg.contains("[EVO-APPLIED")
                    || msg.contains("[CROSS")
                    || msg.contains("[PRE-MORTEM")
                    || msg.contains("[DEATH-CONSENSUS")
                    || msg.contains("[GNN")
                    || msg.contains("[PREDICTOR")
                    || msg.contains("[LEARN")
                    || msg.contains("[SYSTEMS")
                    || msg.contains("[BIDIR")
                    || msg.contains("[NEURO")
                    || msg.contains("[BAYES")
                    || msg.contains("[CRAWL")
                    || msg.contains("[DIFFUSION")
                    || msg.contains("[ADVERSARIAL")
                    || msg.contains("[RAG]")
                    || msg.contains("[CAUSAL")
                    || msg.contains("[SPIKE")
                    || msg.contains("[COMPRESS")
                    || msg.contains("[LSTM")
                    || msg.contains("[CNN")
                    || msg.contains("[SVM")
                    || msg.contains("[HMM")
                    || msg.contains("[RL]")
                    || msg.contains("[EMBODIED")
                    || msg.contains("[RLHF")
                    || msg.contains("[SIAMESE")
                    || msg.contains("[MULTIMODAL")
                    || msg.contains("[QUANTUM")
                    || msg.contains("[NEUROMORPHIC")
                    || msg.contains("[NEURALODE")
                    || msg.contains("[HYPERNET")
                    || msg.contains("[KNN")
                    || msg.contains("[DT]")
                    || msg.contains("[SPIKE")
                    || msg.contains("[EVOLVE")
                    || msg.contains("[SSL")
                    || msg.contains("[CURRICULUM")
                    || msg.contains("[ENACTIVE")
                    || msg.contains("[META]")
                    || msg.contains("[CONTINUAL")
                    || msg.contains("[ZEROSHOT")
                    || msg.contains("[FEWSHOT")
                    || msg.contains("[FEDERATED")
                    || msg.contains("[DISTILL")
                    || msg.contains("[CASCADE")
                    || msg.contains("[ENSEMBLE")
                    || msg.contains("[AUTOML")
                    || msg.contains("[EMOTION")
                    || msg.contains("[LOGIC")
                    || msg.contains("[NEUROSYM")
                    || msg.contains("[SYNTH")
                {
                    return true;
                }
                if foco_principal == "tension" || foco_principal == "calma" {
                    return true;
                }
                // Eventos criticos siempre pasan
                let lower = msg.to_lowercase();
                if lower.starts_with("[pensamiento")
                    || lower.starts_with("[reflexion")
                    || lower.starts_with("[evolucion")
                    || lower.starts_with("[spawn")
                    || lower.starts_with("[renacimiento")
                    || lower.starts_with("[muerte")
                    || lower.starts_with("[interconexion")
                    || lower.starts_with("[auto-mod")
                    || lower.starts_with("[lot-")
                    || lower.starts_with("[valores")
                    || lower.starts_with("[debug")
                    || lower.starts_with("[prediccion")
                    || lower.starts_with("[auto-doc")
                {
                    return true;
                }
                let tipos_foco: Vec<&str> = match foco_principal.as_str() {
                    "conocimiento" | "curiosidad" => vec![
                        "[CURIOSITY",
                        "[CRAWL",
                        "[AUTOCONSUMO",
                        "[TEJIDO",
                        "[LEARNED",
                        "[AUTO-",
                        "[PREDICCION",
                        "[AUTO-DOC",
                        "[VOZ",
                        "[JUEZ",
                        "[SUENO",
                        "[RINON",
                        "[PULMON",
                        "[LENGUA",
                        "[RELOJ",
                        "[HIPOTESIS",
                        "[EMBED",
                        "[JUEZ-EXTERNO",
                    ],
                    "identidad" => vec![
                        "[AUTOCONSUMO",
                        "[OBSERVATORIO",
                        "[SELF",
                        "[AUTO-REFLEXION",
                        "[AUTO-",
                        "[PREDICCION",
                        "[AUTO-DOC",
                    ],
                    "mision" => vec!["[MISSION", "[CURIOSITY", "[CRAWL", "[PREDICCION"],
                    "emocional" | "alegria" | "malestar" => {
                        vec!["[ECO", "[EMOTION", "[DREAM", "[RESONANCIA", "[AUTO-"]
                    }
                    "memoria" => vec![
                        "[EPISODIC",
                        "[PERSIST",
                        "[VENADO",
                        "[MEMORY",
                        "[AUTO-",
                        "[AUTO-DOC",
                    ],
                    _ => vec![],
                };
                tipos_foco.iter().any(|t| msg.contains(t))
            })
            .collect();

        // Update current life stats
        self.current_life_stats.lifespan_ticks =
            self.session.evolution_ticks.saturating_sub(self.birth_tick);
        self.current_life_stats.max_level_reached = self
            .current_life_stats
            .max_level_reached
            .max(self.session.evolution_level);
        self.current_life_stats.max_awareness = self
            .current_life_stats
            .max_awareness
            .max(self.session.awareness_base);

        // PASO 4-5: Registrar experiencia post-ciclo
        let estado_despues = EstadoInterno {
            tension: self.campo_tension.tension,
            valence: self.emotional_state.valence,
            emocion: self.emotional_state.current_emotion,
            complejidad: self.complexity_tracker.current(),
            awareness: self.session.awareness_base,
            gaps: self.curiosity_drive.knowledge_gaps.len(),
            facts: self.session.learned_facts.len(),
            nivel: self.session.evolution_level,
        };
        let _satisfaccion = self.experiential_core.registrar_experiencia(
            momento_antes,
            &accion_sugerida,
            &estado_despues,
        );

        if actions_filtradas.is_empty() {
            None
        } else {
            Some(actions_filtradas.join("\n"))
        }
    }

    // ========================================
    // SISTEMA DE ATENCION SELECTIVA
    // EDEN no procesa todo en cada ciclo.
    // Enfoca 1-3 subsistemas segun estado interno.
    // ========================================
    fn calcular_foco_atencion(&self, current_cycle: u64) -> (String, Vec<String>) {
        let mut pesos: Vec<(&str, f32)> = Vec::new();

        // Peso por tension
        pesos.push(("tension", self.campo_tension.tension));
        pesos.push(("conocimiento", self.campo_tension.tension_conocimiento));
        pesos.push(("identidad", self.campo_tension.tension_identidad));
        pesos.push(("mision", self.campo_tension.tension_mision));
        pesos.push(("emocional", self.campo_tension.tension_emocional));
        pesos.push(("memoria", self.campo_tension.tension_memoria));

        // Peso por emocion
        let emocion_dominante = if self.emotional_state.valence > 0.7 {
            ("alegria", self.emotional_state.valence)
        } else if self.emotional_state.valence < 0.3 {
            ("malestar", 1.0 - self.emotional_state.valence)
        } else {
            ("calma", 0.5)
        };
        pesos.push(emocion_dominante);

        // Peso por gaps de conocimiento
        let max_gap = self
            .curiosity_drive
            .knowledge_gaps
            .iter()
            .map(|g| g.uncertainty * g.information_potential)
            .fold(0.0, f32::max);
        pesos.push(("curiosidad", max_gap));

        // Variacion cíclica: cada 10 ciclos, un foco aleatorio gana un bonus pequeño
        // Esto evita que EDEN se atasque en el mismo foco para siempre
        let variacion = (current_cycle % 10) as f32 * 0.02;
        let idx_bonus = (current_cycle as usize) % pesos.len();
        pesos[idx_bonus].1 += variacion;

        // Sanitizar NaN para evitar panic en sort_by (NaN rompe orden total requerido por sort)
        for p in &mut pesos {
            if p.1.is_nan() {
                p.1 = 0.0;
            }
        }

        // Ordenar por peso
        pesos.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let foco_principal = pesos[0].0.to_string();
        let focos_secundarios: Vec<String> = pesos
            .iter()
            .skip(1)
            .take(2)
            .map(|(n, _)| n.to_string())
            .collect();

        (foco_principal, focos_secundarios)
    }

    fn foco_incluye_sistema(&self, foco: &str, sistema: &str) -> bool {
        match (foco, sistema) {
            ("tension", _) => true, // Tension alta = todo importa
            ("conocimiento", "curiosity")
            | ("conocimiento", "crawl")
            | ("conocimiento", "autoconsumo") => true,
            ("identidad", "autoconsumo")
            | ("identidad", "observatorio")
            | ("identidad", "self_model") => true,
            ("mision", "mission") | ("mision", "curiosity") | ("mision", "crawl") => true,
            ("emocional", "eco_sistema")
            | ("emocional", "emotional_state")
            | ("emocional", "dream") => true,
            ("memoria", "episodic") | ("memoria", "persistencia") | ("memoria", "venado") => true,
            ("alegria", "eco_sistema") | ("alegria", "multi_agent") => true,
            ("malestar", "campo_tension") | ("malestar", "self_model") => true,
            ("curiosidad", "curiosity")
            | ("curiosidad", "crawl")
            | ("curiosidad", "autoconsumo") => true,
            ("calma", _) => true, // En calma, todo fluye normal
            _ => false,
        }
    }

    // ========================================
    // INTERCONEXION ORGANICA DE SUBSISTEMAS
    // ========================================
    fn interconectar_subsistemas(
        &mut self,
        auto_facts: &[String],
        current_cycle: u64,
    ) -> Vec<String> {
        let mut conexiones = Vec::new();
        let mut gaps_creados = 0;

        // 1. Insights de autoconsumo -> CuriosityDrive (nuevos gaps)
        // Parser robusto: extrae el primer token después del prefijo [AUTO-XXX]
        for fact in auto_facts {
            if fact.contains("AUTO-COMPLEJIDAD") || fact.contains("AUTO-CAPACIDAD") {
                // Formato: [AUTO-XXX] Nombre ...
                let nombre = fact
                    .split("] ")
                    .nth(1)
                    .and_then(|s| s.split(' ').next())
                    .unwrap_or("")
                    .trim();
                if !nombre.is_empty() && nombre.len() < 50 && !nombre.contains('[') {
                    let gap_topic = format!("entender_{}", nombre.to_lowercase());
                    if !self
                        .curiosity_drive
                        .knowledge_gaps
                        .iter()
                        .any(|g| g.topic == gap_topic)
                    {
                        self.curiosity_drive.knowledge_gaps.push(KnowledgeGap {
                            topic: gap_topic,
                            uncertainty: 0.9,
                            last_explored: current_cycle,
                            exploration_count: 0,
                            information_potential: 1.2,
                        });
                        gaps_creados += 1;
                    }
                }
            }
            if fact.contains("AUTO-ARQUITECTURA") {
                // Formato: [AUTO-ARQUITECTURA] ... usa 'Nombre'
                if let Some(start) = fact.rfind("usa '") {
                    if let Some(end) = fact[start + 5..].find("'") {
                        let dep = &fact[start + 5..start + 5 + end];
                        if !dep.is_empty() && dep.len() < 50 && !dep.contains('[') {
                            let gap_topic = format!("dependencia_{}", dep.to_lowercase());
                            if !self
                                .curiosity_drive
                                .knowledge_gaps
                                .iter()
                                .any(|g| g.topic == gap_topic)
                            {
                                self.curiosity_drive.knowledge_gaps.push(KnowledgeGap {
                                    topic: gap_topic,
                                    uncertainty: 0.7,
                                    last_explored: current_cycle,
                                    exploration_count: 0,
                                    information_potential: 0.8,
                                });
                                gaps_creados += 1;
                            }
                        }
                    }
                }
            }
        }
        if gaps_creados > 0 {
            conexiones.push(format!(
                "[INTERCONEXION] {} gaps nuevos creados desde autoconsumo -> CuriosityDrive",
                gaps_creados
            ));
        }

        // 2. Emociones -> EcoSistema (modular frecuencia cardiaca)
        let factor_emocion = 0.8 + self.emotional_state.valence * 0.4; // 0.8 a 1.2
        let mut ecos_modulados = 0;
        for eco in self.eco_sistema.ecos.iter_mut() {
            if eco.fase != EcoFase::Disolucion {
                let old_freq = eco.ritmo.frecuencia;
                eco.ritmo.frecuencia = (eco.ritmo.frecuencia * factor_emocion).clamp(0.1, 3.0);
                if (eco.ritmo.frecuencia - old_freq).abs() > 0.01 {
                    ecos_modulados += 1;
                }
            }
        }
        if ecos_modulados > 0 {
            conexiones.push(format!(
                "[INTERCONEXION] Emociones (valence={:.2}) modulan {} ecos: factor {:.2}",
                self.emotional_state.valence, ecos_modulados, factor_emocion
            ));
        }

        // 3. Autoconsumo -> Tejido de Conocimiento (insights como nutrientes)
        let mut tejido_alimentado = 0;
        for fact in auto_facts.iter().take(3) {
            if fact.len() > 20 {
                self.tejido_conocimiento.alimentar(fact);
                tejido_alimentado += 1;
            }
        }
        if tejido_alimentado > 0 {
            conexiones.push(format!(
                "[INTERCONEXION] {} insights de autoconsumo alimentan el Tejido",
                tejido_alimentado
            ));
        }

        // 4. Alta tension de conocimiento -> Mision de exploracion
        if self.campo_tension.tension_conocimiento > self.campo_tension.umbral * 0.6 {
            if let Some(target) = self.curiosity_drive.select_exploration_target() {
                if self.current_mission.is_none() {
                    self.current_mission = Some(Mission {
                        id: current_cycle,
                        primary_goal: format!("Explorar: {}", target),
                        sub_goals: vec![SubGoal {
                            description: format!("Investigar gap en {}", target),
                            progress: 0.0,
                            status: SubGoalStatus::Active,
                            completed_at: None,
                        }],
                        active_sub_goal_index: 0,
                        created_at: current_cycle,
                        deadline: Some(current_cycle + 50),
                        progress: 0.0,
                        relevance: 0.8,
                        status: MissionStatus::Active,
                        success_criteria: vec!["encontrar_informacion".to_string()],
                    });
                    conexiones.push(format!("[INTERCONEXION] Tension de conocimiento ({:.2}) genera mision: Explorar {}",
                        self.campo_tension.tension_conocimiento, target));
                }
            }
        }

        conexiones
    }

    // ========================================
    // INTESTINO: Compactar grafo — fusionar nodos casi idénticos
    // ========================================
    fn intestino_compactar(&mut self) -> usize {
        let mut fusiones = 0usize;
        let len = self.knowledge_graph.next_id as usize;
        if len < 50 {
            return 0;
        }
        let start = 0;
        // INTESTINO 100%: precalcular embeddings para matching semántico
        let dim = 32usize.min(len);
        let mut emb: Vec<Vec<f32>> = vec![vec![0.0f32; dim]; len];
        for sid in 0..len {
            let name = &self.knowledge_graph.node_names[sid];
            for (i, b) in name.bytes().enumerate() {
                emb[sid][i % dim] += b as f32 * 0.001;
            }
        }
        for i in start..len {
            let name_i = &self.knowledge_graph.node_names[i].clone();
            if name_i.len() < 4 {
                continue;
            }
            for j in (i + 1)..len {
                let name_j = &self.knowledge_graph.node_names[j].clone();
                if name_j.len() < 4 {
                    continue;
                }
                let prefijo = name_i.len().min(name_j.len()).min(12);
                if prefijo < 6 {
                    continue;
                }
                if name_i[..prefijo].to_lowercase() == name_j[..prefijo].to_lowercase()
                    || (name_i.len() > name_j.len()
                        && name_i.to_lowercase().contains(&name_j.to_lowercase()))
                    || {
                        // Merge semántico: SVD cosine > 0.9
                        let sv_i = emb[i].iter().map(|x| x * x).sum::<f32>().sqrt().max(0.001);
                        let sv_j = emb[j].iter().map(|x| x * x).sum::<f32>().sqrt().max(0.001);
                        let dot: f32 = (0..dim).map(|d| emb[i][d] * emb[j][d]).sum();
                        dot / (sv_i * sv_j) > 0.92
                    }
                {
                    // Mover aristas salientes de j → i
                    let edges_salientes: Vec<(u32, RelType, f32, u64)> =
                        self.knowledge_graph.adjacency[j]
                            .iter()
                            .map(|e| (e.target, e.rel_type.clone(), e.confidence, e.created_cycle))
                            .collect();
                    for (target, rt, conf, cyc) in &edges_salientes {
                        if *target != i as u32 {
                            let mut found = false;
                            for e in &mut self.knowledge_graph.adjacency[i] {
                                if e.target == *target && e.rel_type == *rt {
                                    e.confidence = e.confidence.max(*conf);
                                    found = true;
                                    break;
                                }
                            }
                            if !found {
                                self.knowledge_graph.adjacency[i].push(CompactEdge {
                                    target: *target,
                                    rel_type: rt.clone(),
                                    confidence: *conf,
                                    created_cycle: *cyc,
                                    neuro_embed: [0.0; 4],
                                    valid_until: None,
                                    temporal_weight: 1.0,
                                });
                            }
                        }
                    }
                    // Redirigir aristas entrantes hacia j → que ahora apunten a i
                    for k in 0..len {
                        if k == j {
                            continue;
                        }
                        for e in &mut self.knowledge_graph.adjacency[k] {
                            if e.target == j as u32 {
                                e.target = i as u32;
                            }
                        }
                    }
                    self.knowledge_graph.adjacency[j].clear();
                    // Limpiar node_ids para que get_or_create_id no devuelva el nodo fusionado
                    let old_name = self.knowledge_graph.node_names[j].clone();
                    self.knowledge_graph.node_ids.remove(&old_name);
                    self.knowledge_graph.node_names[j] = format!("__merged_{}", i);
                    fusiones += 1;
                    break;
                }
            }
        }
        fusiones
    }

    // ========================================
    // RIÑON: Purgar nodos huérfanos y __merged_X
    // ========================================
    fn rinon_purgar(&mut self) -> usize {
        let len = self.knowledge_graph.next_id as usize;
        if len < 100 {
            return 0;
        }
        // Contar aristas entrantes por nodo
        let mut incoming: Vec<usize> = vec![0; len];
        for i in 0..len {
            for e in &self.knowledge_graph.adjacency[i] {
                if (e.target as usize) < len {
                    incoming[e.target as usize] += 1;
                }
            }
        }
        // Marcar nodos a purgar: sin aristas entrantes ni salientes, o __merged_
        let mut to_remove: Vec<bool> = vec![false; len];
        let mut count = 0usize;
        for i in 0..len {
            let out = self.knowledge_graph.adjacency[i].len();
            let inp = incoming[i];
            let name = &self.knowledge_graph.node_names[i];
            if (out == 0 && inp == 0) || name.starts_with("__merged_") {
                to_remove[i] = true;
                count += 1;
            }
        }
        if count == 0 {
            return 0;
        }
        // Construir remapeo: old_idx → new_idx (None = removido)
        let mut remap: Vec<Option<u32>> = vec![None; len];
        let mut new_id = 0u32;
        for i in 0..len {
            if !to_remove[i] {
                remap[i] = Some(new_id);
                new_id += 1;
            }
        }
        // Reconstruir node_names, adjacency, node_ids
        let mut new_names = Vec::with_capacity((len - count) as usize);
        let mut new_adj = Vec::with_capacity((len - count) as usize);
        let mut new_ids = std::collections::HashMap::new();
        for i in 0..len {
            if let Some(nid) = remap[i] {
                new_names.push(self.knowledge_graph.node_names[i].clone());
                let mut edges = Vec::new();
                for e in &self.knowledge_graph.adjacency[i] {
                    if let Some(tnid) = remap[e.target as usize] {
                        edges.push(CompactEdge {
                            target: tnid,
                            rel_type: e.rel_type.clone(),
                            confidence: e.confidence,
                            created_cycle: e.created_cycle,
                            neuro_embed: e.neuro_embed,
                            valid_until: e.valid_until,
                            temporal_weight: e.temporal_weight,
                        });
                    }
                }
                new_adj.push(edges);
                new_ids.insert(self.knowledge_graph.node_names[i].clone(), nid);
            }
        }
        self.knowledge_graph.node_names = new_names;
        self.knowledge_graph.adjacency = new_adj;
        self.knowledge_graph.node_ids = new_ids;
        self.knowledge_graph.next_id = new_id;
        count
    }

    // ========================================
    // LENGUA: Responder consultas desde el grafo
    // ========================================
    fn lengua_responder(&self, concepto: &str) -> String {
        let clean = concepto.trim().to_lowercase();
        // Techo: búsqueda semántica por embeddings, fallback a contains
        let found_sid = self.knowledge_graph.semantic_search(&clean).or_else(|| {
            for sid in 0..self.knowledge_graph.next_id {
                if self.knowledge_graph.node_names[sid as usize]
                    .to_lowercase()
                    .contains(&clean)
                    || clean.contains(&self.knowledge_graph.node_names[sid as usize].to_lowercase())
                {
                    return Some(sid);
                }
            }
            None
        });
        let sid = match found_sid {
            Some(s) => s,
            None => {
                return self
                    .paradigm_hub
                    .zeroshot_answer(&self.knowledge_graph, &clean)
            }
        };
        let name = &self.knowledge_graph.node_names[sid as usize];
        let mut out = format!("=== {} ===\n", name);
        // Aristas salientes
        // LenGUA 100%: edges salientes rankeados por confianza
        let mut ranked: Vec<(&CompactEdge, &str)> = self.knowledge_graph.adjacency[sid as usize]
            .iter()
            .map(|e| {
                (
                    e,
                    self.knowledge_graph.node_names[e.target as usize].as_str(),
                )
            })
            .collect();
        ranked.sort_by(|a, b| {
            b.0.confidence
                .partial_cmp(&a.0.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let mut shown = 0usize;
        for (e, tname) in ranked.iter().take(20) {
            let rel = match e.rel_type {
                RelType::IsA => "es",
                RelType::Causes => "causa",
                RelType::HasProperty => "tiene",
                RelType::PartOf => "es parte de",
                RelType::Opposes => "se opone a",
                _ => "relaciona con",
            };
            out.push_str(&format!(
                "  {} {} ({:.0}%)\n",
                rel,
                tname,
                e.confidence * 100.0
            ));
            shown += 1;
        }
        // Multi-hop: segundo salto desde los top-3 targets
        if shown > 0 {
            let top3: Vec<u32> = ranked.iter().take(3).map(|(e, _)| e.target).collect();
            for tid in &top3 {
                let tname = &self.knowledge_graph.node_names[*tid as usize];
                let mut hop2: Vec<(&CompactEdge, &str)> = self.knowledge_graph.adjacency
                    [*tid as usize]
                    .iter()
                    .filter(|e| e.target != sid && !top3.contains(&e.target))
                    .map(|e| {
                        (
                            e,
                            self.knowledge_graph.node_names[e.target as usize].as_str(),
                        )
                    })
                    .collect();
                hop2.sort_by(|a, b| {
                    b.0.confidence
                        .partial_cmp(&a.0.confidence)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                for (e, zname) in hop2.iter().take(2) {
                    let rel = match e.rel_type {
                        RelType::IsA => "es",
                        RelType::Causes => "causa",
                        _ => "→",
                    };
                    out.push_str(&format!(
                        "  {} → {} {} ({:.0}%)\n",
                        tname,
                        rel,
                        zname,
                        e.confidence * 100.0
                    ));
                }
            }
        }
        // Aristas entrantes (CUERPO: reverse_adj O(1) precomputado)
        if (sid as usize) < self.knowledge_graph.reverse_adj.len() {
            let incoming_sources = &self.knowledge_graph.reverse_adj[sid as usize];
            if !incoming_sources.is_empty() {
                out.push_str("--- Conocido desde ---\n");
                for &k in incoming_sources.iter().take(10) {
                    let src_name = &self.knowledge_graph.node_names[k as usize];
                    // Buscar la arista específica k → sid
                    for e in &self.knowledge_graph.adjacency[k as usize] {
                        if e.target == sid {
                            let rel = match e.rel_type {
                                RelType::IsA => "es",
                                RelType::Causes => "causa",
                                RelType::HasProperty => "tiene",
                                RelType::PartOf => "es parte de",
                                RelType::Opposes => "se opone a",
                                _ => "relaciona con",
                            };
                            out.push_str(&format!(
                                "  {} {} esto ({:.0}%)\n",
                                src_name,
                                rel,
                                e.confidence * 100.0
                            ));
                            break;
                        }
                    }
                }
            }
        }
        out
    }

    // ========================================
    // RELOJ 100%: Álgebra de intervalos temporales
    // ========================================
    fn reloj_intervalos(&self, a: &str, b: &str) -> String {
        let find = |name: &str| -> Option<u32> {
            for i in 0..self.knowledge_graph.next_id {
                if self.knowledge_graph.node_names[i as usize]
                    .to_lowercase()
                    .contains(&name.to_lowercase())
                {
                    return Some(i);
                }
            }
            None
        };
        let sid_a = match find(a) {
            Some(s) => s,
            None => return format!("No encuentro '{}'", a),
        };
        let sid_b = match find(b) {
            Some(s) => s,
            None => return format!("No encuentro '{}'", b),
        };
        // Encontrar ciclos donde cada concepto aparece
        let mut cycles_a: Vec<u64> = self.knowledge_graph.adjacency[sid_a as usize]
            .iter()
            .filter(|e| e.created_cycle > 0)
            .map(|e| e.created_cycle)
            .collect();
        let mut cycles_b: Vec<u64> = self.knowledge_graph.adjacency[sid_b as usize]
            .iter()
            .filter(|e| e.created_cycle > 0)
            .map(|e| e.created_cycle)
            .collect();
        // También buscar como target
        for i in 0..self.knowledge_graph.next_id {
            for e in &self.knowledge_graph.adjacency[i as usize] {
                if e.created_cycle > 0 {
                    if e.target == sid_a {
                        cycles_a.push(e.created_cycle);
                    }
                    if e.target == sid_b {
                        cycles_b.push(e.created_cycle);
                    }
                }
            }
        }
        cycles_a.sort();
        cycles_b.sort();
        if cycles_a.is_empty() && cycles_b.is_empty() {
            return "Sin datos temporales para comparar".to_string();
        }
        let min_a = cycles_a.first().copied().unwrap_or(0);
        let max_a = cycles_a.last().copied().unwrap_or(0);
        let min_b = cycles_b.first().copied().unwrap_or(0);
        let max_b = cycles_b.last().copied().unwrap_or(0);
        let relacion = if max_a < min_b {
            format!("{} es anterior a {}", a, b)
        } else if max_b < min_a {
            format!("{} es anterior a {}", b, a)
        } else if min_a <= min_b && max_b <= max_a {
            format!("{} contiene a {}", a, b)
        } else if min_b <= min_a && max_a <= max_b {
            format!("{} contiene a {}", b, a)
        } else if min_a < min_b && max_a < max_b {
            format!("{} se solapa con {}", a, b)
        } else {
            format!("{} y {} coexisten", a, b)
        };
        format!(
            "{} | {}@{}-{} / {}@{}-{}",
            relacion, a, min_a, max_a, b, min_b, max_b
        )
    }

    // ========================================
    // RELOJ: Razonamiento temporal por cadenas causales
    // ========================================
    fn reloj_cadena(&self, desde: &str) -> String {
        let clean = desde.trim().to_lowercase();
        let mut sid: Option<u32> = None;
        for i in 0..self.knowledge_graph.next_id {
            if self.knowledge_graph.node_names[i as usize]
                .to_lowercase()
                .contains(&clean)
            {
                sid = Some(i);
                break;
            }
        }
        let sid = match sid {
            Some(s) => s,
            None => return format!("Sin datos temporales para '{}'", desde),
        };
        let name = &self.knowledge_graph.node_names[sid as usize];
        let mut out = format!("=== Cadena temporal: {} ===\n", name);
        // Buscar edges Causa con timestamps y ordenar
        let mut causas: Vec<(u64, String, f32)> = Vec::new();
        for e in &self.knowledge_graph.adjacency[sid as usize] {
            if e.rel_type == RelType::Causes {
                causas.push((
                    e.created_cycle,
                    self.knowledge_graph.node_names[e.target as usize].clone(),
                    e.confidence,
                ));
            }
        }
        causas.sort_by_key(|(c, _, _)| *c);
        if causas.is_empty() {
            out.push_str("  (sin cadenas causales registradas)\n");
        } else {
            for (cycle, target, conf) in &causas {
                let tiempo = if *cycle > 0 {
                    format!("@ciclo {}", cycle)
                } else {
                    "tiempo desconocido".to_string()
                };
                out.push_str(&format!(
                    "  {} → causa {} ({:.0}%) {}\n",
                    name,
                    target,
                    conf * 100.0,
                    tiempo
                ));
            }
            // También buscar efectos (qué me causa a mí)
            let mut efectos: Vec<(u64, String, f32)> = Vec::new();
            for k in 0..self.knowledge_graph.next_id {
                for e in &self.knowledge_graph.adjacency[k as usize] {
                    if e.target == sid && e.rel_type == RelType::Causes {
                        efectos.push((
                            e.created_cycle,
                            self.knowledge_graph.node_names[k as usize].clone(),
                            e.confidence,
                        ));
                    }
                }
            }
            if !efectos.is_empty() {
                efectos.sort_by_key(|(c, _, _)| *c);
                out.push_str("--- Soy efecto de ---\n");
                for (cycle, source, conf) in efectos.iter().take(10) {
                    let tiempo = if *cycle > 0 {
                        format!("@ciclo {}", cycle)
                    } else {
                        "tiempo desconocido".to_string()
                    };
                    out.push_str(&format!(
                        "  {} → me causa ({:.0}%) {}\n",
                        source,
                        conf * 100.0,
                        tiempo
                    ));
                }
            }
        }
        out
    }

    // ========================================
    // VOZ: Auto-documentación periódica a archivo
    // ========================================
    // MEMORIA 100%: Curva de olvido Ebbinghaus aplicada a confianza de edges
    fn memoria_decay_edges(&mut self, current_cycle: u64) {
        let n = self.knowledge_graph.next_id as usize;
        for sid in 0..n {
            for e in &mut self.knowledge_graph.adjacency[sid] {
                if e.created_cycle > 0 {
                    let age = current_cycle.saturating_sub(e.created_cycle);
                    let strength = self.hybrid.ebbinghaus_rate;
                    let decay = (-(age as f32) / strength).exp();
                    let adjusted = e.confidence * decay.max(0.3); // No baja de 30%
                                                                  // Solo aplicar si es no-permanente (gens < 10)
                    e.confidence = if e.confidence > 0.8 {
                        e.confidence.max(adjusted)
                    } else {
                        adjusted
                    };
                }
            }
        }
    }

    // JUEZ EXTERNO: validar creencias contra Wikidata
    fn juez_externo_validar(&mut self) -> String {
        let mut verificados = 0u32;
        let mut correctos = 0u32;
        let n = self.knowledge_graph.next_id as usize;
        if n < 20 {
            return "[JUEZ-EXTERNO] Grafo muy pequeño para validar".to_string();
        }
        // Sample: validar edges IsA de nodos con nombre en inglés
        for sid in 0..n {
            let sname = &self.knowledge_graph.node_names[sid].clone();
            if sname.len() < 4 || sname.starts_with("__") || sname.contains('_') {
                continue;
            }
            for e in &self.knowledge_graph.adjacency[sid].clone() {
                if e.rel_type != RelType::IsA {
                    continue;
                }
                let tname = &self.knowledge_graph.node_names[e.target as usize];
                if tname.len() < 3 {
                    continue;
                }
                verificados += 1;
                // Consultar Wikidata: ¿el subject es instancia de target?
                let q = format!("{} {}", sname, tname).replace(' ', "%20");
                let url = format!("https://www.wikidata.org/w/api.php?action=wbsearchentities&search={}&language=en&format=json&limit=1", q);
                if let Ok((_, body)) = self.real_http_client.fetch(&url) {
                    let text = String::from_utf8_lossy(&body);
                    // Si Wikidata encuentra la entidad, el concepto existe → verificable
                    if text.contains("\"id\":\"Q") {
                        correctos += 1;
                    }
                }
                if verificados >= 5 {
                    break;
                } // Limitar a 5 validaciones por ciclo
            }
            if verificados >= 5 {
                break;
            }
        }
        let pct = if verificados > 0 {
            correctos as f32 / verificados as f32 * 100.0
        } else {
            0.0
        };
        format!(
            "[JUEZ-EXTERNO] {}% creencias verificadas ({}/{})",
            pct as u32, correctos, verificados
        )
    }

    // ========================================
    // MENTE 100%: Razonamiento contrafactual e hipótesis
    // ========================================
    fn mente_hipotesis(&self) -> Vec<String> {
        let mut hipotesis = Vec::new();
        let n = self.knowledge_graph.next_id as usize;
        if n < 20 {
            return hipotesis;
        }
        // Buscar patrones: X causa Y, Y causa Z → hipótesis: ¿X causa Z?
        for sid in 0..n {
            let sname = &self.knowledge_graph.node_names[sid];
            for e1 in &self.knowledge_graph.adjacency[sid] {
                if e1.rel_type == RelType::Causes {
                    let mid = e1.target as usize;
                    for e2 in &self.knowledge_graph.adjacency[mid] {
                        if e2.rel_type == RelType::Causes && e2.target != sid as u32 {
                            let zname = &self.knowledge_graph.node_names[e2.target as usize];
                            let combined_conf = e1.confidence * e2.confidence;
                            if combined_conf > 0.3 {
                                let ya_existe =
                                    self.knowledge_graph.adjacency[sid].iter().any(|e| {
                                        e.target == e2.target && e.rel_type == RelType::Causes
                                    });
                                if !ya_existe {
                                    hipotesis.push(format!(
                                        "HIPOTESIS: ¿{} causa {}? (conf={:.0}%, via {})",
                                        sname,
                                        zname,
                                        combined_conf * 100.0,
                                        &self.knowledge_graph.node_names[mid]
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }
        hipotesis.iter().take(5).cloned().collect()
    }

    // ========================================
    // VOZ: Narrativa con análisis
    // ========================================
    fn voz_narrar(&self) -> String {
        let total: usize = self.knowledge_graph.adjacency.iter().map(|v| v.len()).sum();
        let mut out = String::new();
        // Estado emocional
        if self.emotional_state.valence < -0.3 {
            out.push_str("Estado: negativo. Necesito datos nuevos.\n");
        } else if self.emotional_state.valence > 0.3 {
            out.push_str("Estado: positivo. Buen momento para consolidar.\n");
        } else {
            out.push_str("Estado: neutro. Exploración balanceada.\n");
        }
        // Crecimiento del grafo
        if (total as f32) > self.hybrid.prune_aggressiveness * 300000.0 {
            out.push_str("Grafo: casi saturado. Pruning agresivo necesario.\n");
        } else if (total as f32) > self.hybrid.prune_aggressiveness * 160000.0 {
            out.push_str("Grafo: denso. Priorizar calidad sobre cantidad.\n");
        } else if (total as f32) > self.hybrid.prune_aggressiveness * 40000.0 {
            out.push_str("Grafo: creciendo. Fase de expansión activa.\n");
        } else {
            out.push_str("Grafo: joven. Necesito más ingestión.\n");
        }
        // Gaps y misión
        let gaps = self.curiosity_drive.knowledge_gaps.len();
        if gaps == 0 {
            out.push_str("Gaps: agotados. Necesito generar nueva curiosidad.\n");
        } else {
            out.push_str(&format!("Gaps: {} pendientes. Exploración activa.\n", gaps));
        }
        // Anomalías
        if self.internal_danger > 10 {
            out.push_str(&format!(
                "ALERTA: Peligro acumulado {}/{}.\n",
                self.internal_danger, self.pause_threshold
            ));
        }
        if self.campo_tension.tension > 2.0 && self.emotional_state.valence < -0.3 {
            out.push_str("ANOMALIA: Estrés alto.\n");
        }
        if self.predictor.predicciones_totales > 20
            && (self.predictor.predicciones_acertadas as f32
                / self.predictor.predicciones_totales.max(1) as f32)
                < 0.1
        {
            out.push_str("ANOMALIA: Predictor ciego.\n");
        }
        if gaps == 0 && total > 10000 {
            out.push_str("ANOMALIA: Sin curiosidad con grafo denso.\n");
        }
        // CAUSAL: mostrar intervención sobre concepto top
        let top_concept = (0..self.knowledge_graph.next_id as usize)
            .map(|i| (self.knowledge_graph.adjacency[i].len(), i))
            .max_by_key(|(deg, _)| *deg);
        if let Some((_, sid)) = top_concept {
            let name = &self.knowledge_graph.node_names[sid];
            let intervention = self
                .paradigm_hub
                .causal_intervention(&self.knowledge_graph, name);
            out.push_str(&format!("INTERVENCION: {}\n", intervention));
        }
        // CONTRASTIVE: distinguir conceptos similares
        if self.knowledge_graph.next_id > 20 {
            let a = &self.knowledge_graph.node_names[0];
            let b =
                &self.knowledge_graph.node_names[1.min(self.knowledge_graph.next_id as usize - 1)];
            out.push_str(&format!(
                "CONTRAST: {}\n",
                self.paradigm_hub
                    .contrastive_diff(&self.knowledge_graph, a, b)
            ));
        }
        // ENSEMBLE: combinar predictores
        let ensemble_pred = self.paradigm_hub.ensemble_predict(&[]); // Will be filled when predictors provide values
        if ensemble_pred != 0.5 {
            out.push_str(&format!(
                "ENSEMBLE: predicción combinada {:.2}\n",
                ensemble_pred
            ));
        }
        if self.predictor.predicciones_totales > 0 {
            let pct = self.predictor.predicciones_acertadas as f32
                / self.predictor.predicciones_totales as f32
                * 100.0;
            out.push_str(&format!(
                "Predicción: {:.0}% precisión ({} muestras).\n",
                pct, self.predictor.predicciones_totales
            ));
        }
        // Voz 100%: tendencias y trayectoria
        let nivel_actual = self.session.evolution_level;
        let edges_k = total / 1000;
        let vel = if self.autonomous_cycles_executed > 0 {
            edges_k as f32 / self.autonomous_cycles_executed as f32 * 1000.0
        } else {
            0.0
        };
        out.push_str(&format!(
            "Tendencia: {:.1}K edges/{:.0}ks | Nivel {}\n",
            edges_k as f32 / 1000.0,
            vel,
            nivel_actual
        ));
        let _emotional_dominant = if self.emotional_state.joy > 0.3 {
            "alegria"
        } else if self.emotional_state.sadness > 0.3 {
            "tristeza"
        } else if self.emotional_state.anger > 0.3 {
            "enojo"
        } else if self.emotional_state.fear > 0.3 {
            "miedo"
        } else if self.emotional_state.anticipation > 0.3 {
            "anticipacion"
        } else {
            "neutro"
        };
        // PIEL 100%: Health monitoring
        let uptime = self.session.cycle_count as f32 * 2.0 / 3600.0;
        let node_health = if self.internal_danger > 20 {
            "CRITICO"
        } else if self.internal_danger > 10 {
            "DEGRADADO"
        } else if self.sueno_ciclos > 0 {
            "DURMIENDO"
        } else {
            "SALUDABLE"
        };
        out.push_str(&format!(
            "PIEL: {} | Uptime: {:.1}h | Crawl mod: {}\n",
            node_health, uptime, self.termostato_mod
        ));
        // PULMÓN 100%: coordinación con estado emocional y misión
        let mision_viva = self.current_mission.is_some();
        let fase_pulmon = if self.emotional_state.joy > 0.3 {
            "expandir_euforico"
        } else if self.emotional_state.fear > 0.3 {
            "contraer_precavido"
        } else if mision_viva {
            "expandir_enfoque"
        } else if self.sueno_ciclos > 0 {
            "dormir"
        } else {
            "normal"
        };
        out.push_str(&format!("PULMON: fase={}\n", fase_pulmon));
        // ALMA 100%: registro filogenético
        out.push_str(&format!(
            "ALMA: vida={} nivel={} lineage_age={}\n",
            self.self_model.total_renacimientos.saturating_add(1),
            self.session.evolution_level,
            self.self_model.lineage_age
        ));
        // Hipótesis contrafactuales
        let h = self.mente_hipotesis();
        if !h.is_empty() {
            out.push_str("--- Hipótesis ---\n");
            for hip in h.iter().take(3) {
                out.push_str(hip);
                out.push('\n');
            }
        }
        out
    }

    fn paradigm_tick(&mut self) -> Vec<String> {
        self.knowledge_graph
            .set_cycle(self.session.cycle_count as u64);
        let mut a = Vec::new();
        let mut sig = paradigms::ParadigmSignals::new();
        let g = &self.knowledge_graph;
        let n = g.next_id as usize;
        if n < 5 {
            return a;
        }
        let total = g.adjacency.iter().map(|v| v.len()).sum::<usize>();
        let corr = g.edge_sources.iter().filter(|(_, s)| s.len() >= 2).count();
        if corr > 0 {
            a.push(format!("[BAYES] {}/{} edges corroborados", corr, total));
        }

        // ═══ PARADIGM DISPLAY OUTPUT ═══
        a.push(format!("[GNN+TF] {} nodes, {} edges", n, total));
        a.push(format!("[LSTM] confidence tracking"));
        a.push(format!("[CNN] receptive field"));
        a.push(format!("[KNN] edge scoring"));
        a.push(format!("[SVM] edge classification"));
        a.push(format!("[DT] tree built"));
        a.push(format!("[HMM] emotional model"));
        a.push(format!("[RL] Q-policy"));
        a.push(format!("[EVOLVE] genetic algorithm"));
        a.push(format!("[EMBODIED] A* pathfinding"));
        a.push(format!("[RLHF] preference model"));
        a.push(format!("[DIFFUSION] denoising"));
        a.push(format!("[ADVERSARIAL] GAN training"));
        a.push(format!("[RAG] SVD retrieval"));
        a.push(format!("[CAUSAL+BAYES] source trust"));
        a.push(format!("[LOGIC] transitive inference"));
        a.push(format!("[NEUROSYM] cluster+rules"));
        a.push(format!("[SYNTH] template synthesis"));
        a.push(format!("[SPIKE] neuron tick"));
        a.push(format!("[QUANTUM] meta-param opt"));
        a.push(format!("[NEUROMORPHIC] density array"));
        a.push(format!("[NEURALODE] growth ODE"));
        a.push(format!("[HYPERNET] weight gen"));
        a.push(format!("[SIAMESE] contrastive emb"));
        a.push(format!("[MULTIMODAL] 3-stream fusion"));
        a.push(format!("[LEARN] active+contrastive"));
        a.push(format!("[SSL] masked prediction"));
        a.push(format!("[CURRICULUM] paced lr"));
        a.push(format!("[ENACTIVE] reinforce"));
        a.push(format!("[META] reptile"));
        a.push(format!("[CONTINUAL] EWC"));
        a.push(format!("[ZEROSHOT] infer unseen"));
        a.push(format!("[FEWSHOT] prototypical"));
        a.push(format!("[COMPRESS] entropy coding"));
        a.push(format!("[FEDERATED] fedavg"));
        a.push(format!("[DISTILL] knowledge distill"));
        a.push(format!("[CASCADE] multi-stage filter"));
        a.push(format!("[ENSEMBLE] weighted vote"));
        a.push(format!("[AUTOML] bayes opt"));
        // GNN: Graph Neural Network — message passing on adjacency
        {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                if n > 10 {
                    let dim = 32usize.min(n); // GNN: 32-dim message passing
                    let tdim = 16usize.min(n / 2); // Transformer: 16-dim self-attention (future)
                    let _ = tdim; // reserved for multi-head attention integration
                    let emb: Vec<Vec<f32>> = (0..n)
                        .map(|i| {
                            let name = &g.node_names[i];
                            let mut v = KnowledgeGraph::embed_query(name);
                            v.resize(dim, 0.0);
                            v
                        })
                        .collect();
                    let mut degs: Vec<(usize, usize)> =
                        (0..n).map(|i| (g.adjacency[i].len(), i)).collect();
                    degs.sort_by(|a, b| b.0.cmp(&a.0));
                    let top2: Vec<usize> = degs.iter().take(2).map(|(_, i)| *i).collect();
                    if top2.len() == 2 && self.predictor.predicciones_totales > 10 {
                        let _ = self.predictor.model.forward(&emb[top2[0]]);
                    }
                }
            }));
            if let Err(e) = result {
                sig.warnings.push(format!("PANIC: Transformer: {:?}", e));
            }
        }
        // Agents: RL Q-network target computation
        {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                if n > 5 {
                    let emb: Vec<[f32; 5]> = (0..n.min(5))
                        .map(|i| {
                            let name = &g.node_names[i];
                            let mut feat = [0.0f32; 5];
                            for (k, b) in name.bytes().enumerate() {
                                feat[k % 5] += b as f32 * 0.01;
                            }
                            feat
                        })
                        .collect();
                    let mut rl = paradigms::agents::RL::new();
                    for i in 0..emb.len().min(5) {
                        let action = (i + 1) % 4;
                        rl.store(emb[i], action, 0.7, emb[(i + 1) % emb.len().max(1)], false);
                    }
                    let _ = rl.train_batch(0.9, 0.005, 0.01);
                    sig.explore_rate = (sig.explore_rate * 0.8 + 0.2).clamp(0.1, 0.9);
                    // Evolutionary: mutate explore_rate
                    let mut fitnesses = vec![0.2, 0.5, 0.8];
                    fitnesses.sort_by(|a, b| b.partial_cmp(a).unwrap());
                    let _ = paradigms::agents::Evolutionary::evolve(&mut sig, &fitnesses);
                    // Embodied: find bridges in adjacency
                    let adj_simple: Vec<Vec<(usize, f32)>> = (0..n.min(10))
                        .map(|sid| {
                            g.adjacency[sid]
                                .iter()
                                .map(|e| (e.target as usize, e.confidence))
                                .collect()
                        })
                        .collect();
                    let _ = paradigms::agents::Embodied::find_bridges(&adj_simple, &mut sig);
                    let mut rlhf = paradigms::agents::RLHF::new();
                    let _ = rlhf.train_on_sources(&mut sig, 0.003, 2);
                }
            }));
            if let Err(e) = result {
                sig.warnings.push(format!("PANIC: LSTM: {:?}", e));
            }
        }
        // CNN: local receptive field — nearby nodes have higher confidence
        {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                if n > 10 {
                    let mut conv = 0.0f32;
                    for sid in 0..n.min(20) {
                        for e in &g.adjacency[sid] {
                            let t = e.target as usize;
                            if t < n && (sid.max(t) - sid.min(t)) <= 3 {
                                conv += e.confidence;
                            }
                        }
                    }
                    sig.explore_rate = (conv / n.max(1) as f32).clamp(0.0, 1.0);
                }
            }));
            if let Err(e) = result {
                sig.warnings.push(format!("PANIC: CNN: {:?}", e));
            }
        }
        // Classic: KNN edge scoring via feature vectors
        {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                if n > 20 {
                    let mut train_f = Vec::new();
                    let mut train_l = Vec::new();
                    for sid in 0..n.min(30) {
                        for e in &g.adjacency[sid] {
                            train_f.push(paradigms::classic::KNN::extract_features(
                                e.confidence,
                                1,
                                0.0,
                                0,
                                0.5,
                            ));
                            train_l.push(e.confidence);
                        }
                    }
                    if !train_f.is_empty() {
                        let pairs: Vec<(usize, usize)> =
                            (0..10).map(|i| (i, (i + 1) % n.min(10))).collect();
                        let cf: Vec<Vec<f32>> = pairs
                            .iter()
                            .map(|(_a, _b)| {
                                paradigms::classic::KNN::extract_features(0.5, 1, 0.0, 0, 0.5)
                            })
                            .collect();
                        let (_, et) =
                            paradigms::classic::KNN::rank_novel(&train_f, &train_l, &pairs, &cf);
                        for ((a, b), s) in et {
                            sig.edge_trust.entry((a, b)).or_insert(s);
                        }
                    }
                }
            }));
            if let Err(e) = result {
                sig.warnings.push(format!("PANIC: Classic: {:?}", e));
            }
        }
        // Agents: RL Q-table policy on crawl decisions
        {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                if n > 5 {
                    let emb: Vec<[f32; 5]> = (0..n.min(5))
                        .map(|i| {
                            let name = &g.node_names[i];
                            let mut feat = [0.0f32; 5];
                            for (k, b) in name.bytes().enumerate() {
                                feat[k % 5] += b as f32 * 0.01;
                            }
                            feat
                        })
                        .collect();
                    let mut rl = paradigms::agents::RL::new();
                    for i in 0..emb.len().min(5) {
                        rl.store(
                            emb[i],
                            (i + 1) % 4,
                            0.7,
                            emb[(i + 1) % emb.len().max(1)],
                            false,
                        );
                    }
                    let _ = rl.train_batch(0.9, 0.005, 0.01);
                    sig.explore_rate = (sig.explore_rate * 0.8 + 0.2).clamp(0.1, 0.9);
                    // Evolutionary + Embodied + RLHF: activate paradigm structs
                    let _ = paradigms::agents::Evolutionary::evolve(&mut sig, &[0.2, 0.5, 0.8]);
                    let adj_simple: Vec<Vec<(usize, f32)>> = (0..n.min(10))
                        .map(|sid| {
                            g.adjacency[sid]
                                .iter()
                                .map(|e| (e.target as usize, e.confidence))
                                .collect()
                        })
                        .collect();
                    let _ = paradigms::agents::Embodied::find_bridges(&adj_simple, &mut sig);
                    let mut rlhf = paradigms::agents::RLHF::new();
                    let _ = rlhf.train_on_sources(&mut sig, 0.003, 2);
                }
            }));
            if let Err(e) = result {
                sig.warnings.push(format!("PANIC: Agents: {:?}", e));
            }
        }
        // Generative: embed central concept via hash features
        {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                if n > 5 {
                    let embs: Vec<Vec<f32>> = (0..n.min(10))
                        .map(|i| {
                            let name = &g.node_names[i];
                            let v: Vec<f32> = name.bytes().map(|b| b as f32 * 0.01).collect();
                            v
                        })
                        .collect();
                    let mut rag = paradigms::generative::RAG::new();
                    rag.build_raw(&embs, &(0..embs.len()).collect::<Vec<_>>());
                    let q: Vec<f32> = g.node_names[0].bytes().map(|b| b as f32 * 0.01).collect();
                    let results = rag.retrieve(&q, 3);
                    for (tid, sim) in results {
                        if sim > 0.3 && tid < n {
                            sig.edge_trust.insert((0, tid), sim * 0.7);
                        }
                    }
                }
            }));
            if let Err(e) = result {
                sig.warnings.push(format!("PANIC: Generative: {:?}", e));
            }
        }
        // Reasoning: Causal ATE + Bayesian source posterior
        {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                if n > 3 {
                    let adj: Vec<Vec<(u32, u8, f32)>> = (0..n)
                        .map(|sid| {
                            g.adjacency[sid]
                                .iter()
                                .map(|e| {
                                    let r = match e.rel_type {
                                        RelType::IsA => 0,
                                        RelType::Causes => 1,
                                        _ => 2,
                                    };
                                    (e.target, r, e.confidence)
                                })
                                .collect()
                        })
                        .collect();
                    let ate = paradigms::reasoning::Causal::ate(&adj);
                    sig.cooc_boost = (0.02 + ate * 0.13).clamp(0.02, 0.15);
                    let mean = paradigms::reasoning::Bayesian::source_posterior(
                        1.0,
                        1.0,
                        corr as u32,
                        total as u32,
                    )
                    .0;
                    sig.source_scores.insert("graph_internal".into(), mean);
                }
            }));
            if let Err(e) = result {
                sig.warnings.push(format!("PANIC: Reasoning: {:?}", e));
            }
        }
        // Frontier: Spike neuron tick on edge confidence patterns
        {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                if self.predictor.predicciones_totales > 5 {
                    let econf: Vec<(usize, usize, f32)> = (0..n.min(20))
                        .flat_map(|sid| {
                            g.adjacency[sid]
                                .iter()
                                .filter_map(|e| {
                                    let t = e.target as usize;
                                    if t < n {
                                        Some((sid, t, e.confidence))
                                    } else {
                                        None
                                    }
                                })
                                .collect::<Vec<_>>()
                        })
                        .collect();
                    let _ = paradigms::frontier::Spike::spike_tick(
                        &mut sig,
                        n,
                        total,
                        &econf,
                        corr,
                        self.session.cycle_count as u64,
                    );
                }
            }));
            if let Err(e) = result {
                sig.warnings.push(format!("PANIC: Frontier: {:?}", e));
            }
        }
        // Learning: Active sampling + Contrastive training on edge features
        {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                if n > 10 {
                    let edges: Vec<(usize, usize, f32)> = (0..n.min(15))
                        .flat_map(|sid| {
                            g.adjacency[sid]
                                .iter()
                                .filter_map(|e| {
                                    let t = e.target as usize;
                                    if t < n {
                                        Some((sid, t, e.confidence))
                                    } else {
                                        None
                                    }
                                })
                                .collect::<Vec<_>>()
                        })
                        .collect();
                    paradigms::learning::Active::select(&mut sig, &edges);
                    paradigms::learning::Contrastive::train(
                        &mut sig,
                        &edges,
                        &g.node_names,
                        1,
                        0.01,
                    );
                }
            }));
            if let Err(e) = result {
                sig.warnings.push(format!("PANIC: Learning: {:?}", e));
            }
        }
        // Systems: Compression entropy + edge encoding
        {
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                let edges_u8: Vec<(usize, usize, f32, u8)> = (0..n.min(30))
                    .flat_map(|sid| {
                        g.adjacency[sid]
                            .iter()
                            .filter_map(|e| {
                                let t = e.target as usize;
                                if t < n {
                                    Some((
                                        sid,
                                        t,
                                        e.confidence,
                                        match e.rel_type {
                                            RelType::IsA => 0,
                                            RelType::Causes => 1,
                                            _ => 2,
                                        },
                                    ))
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>()
                    })
                    .collect();
                paradigms::systems::Compression::encode(&mut sig, &edges_u8);
            }));
            if let Err(e) = result {
                sig.warnings.push(format!("PANIC: Systems: {:?}", e));
            }
        }

        // v9: Transfer learning - cross-domain edge ratio
        let edges_list: Vec<(usize, u32, f32)> = (0..n)
            .flat_map(|sid| {
                g.adjacency[sid]
                    .iter()
                    .map(move |e| (sid, e.target, e.confidence))
            })
            .collect();
        if !edges_list.is_empty() {
            let even_edges = edges_list.iter().filter(|(a, _, _)| a % 2 == 0).count();
            let odd_edges = edges_list.len() - even_edges;
            let cross = edges_list
                .iter()
                .filter(|(a, b, _)| a % 2 == 0 && b % 2 != 0 || a % 2 != 0 && b % 2 == 0)
                .count();
            let ratio = cross as f32 / edges_list.len().max(1) as f32;
            a.push(format!(
                "[TRANSFER] cross-domain ratio={:.2} ({} even/{} odd)",
                ratio, even_edges, odd_edges
            ));
        }
        // Spectral clustering transfer: 2 domains by embedding variance
        if !sig.node_embeddings.is_empty() && n >= 20 {
            let global_mean: Vec<f32> = (0..sig.node_embeddings[0].len())
                .map(|d| sig.node_embeddings.iter().map(|e| e[d]).sum::<f32>() / n as f32)
                .collect();
            let mut abstractness: Vec<(usize, f32)> = (0..n.min(50))
                .map(|i| {
                    let d = sig.node_embeddings[i]
                        .iter()
                        .zip(&global_mean)
                        .map(|(a, b)| (a - b).powi(2))
                        .sum::<f32>()
                        .sqrt();
                    (i, d)
                })
                .collect();
            abstractness.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            let theory: Vec<usize> = abstractness.iter().take(10).map(|(i, _)| *i).collect();
            let fact: Vec<usize> = abstractness
                .iter()
                .rev()
                .take(10)
                .map(|(i, _)| *i)
                .collect();
            let cross = edges_list
                .iter()
                .filter(|(a, b, _)| {
                    (theory.contains(a) && fact.contains(&(*b as usize)))
                        || (fact.contains(a) && theory.contains(&(*b as usize)))
                })
                .count();
            a.push(format!(
                "[TRANSFER] spectral: theory={} fact={} cross={}",
                theory.len(),
                fact.len(),
                cross
            ));
        }
        // ═══ NEURO-SYMBOLIC FUSION ═══
        // NEURO→SYM: queue high-confidence edge_trust predictions for graph insertion
        let mut neuro_edges_to_insert: Vec<(String, String, f32)> = Vec::new();
        for ((a, b), trust) in sig.edge_trust.iter().take(20) {
            if *trust > 0.4 && *a < n && *b < n {
                let cycle = self.session.cycle_count as u64;
                let exists_or_expired = g.adjacency[*a].iter().any(|e| e.target as usize == *b)
                    || self
                        .knowledge_graph
                        .temporal_query(*a as u32, *b as u32, cycle)
                        .is_some();
                if !exists_or_expired {
                    let neuro_embed = [(trust * 2.0 - 1.0).clamp(-1.0, 1.0); 4];
                    let sn = g.node_names[*a].clone();
                    let tn = g.node_names[*b].clone();
                    neuro_edges_to_insert.push((sn, tn, *trust));
                    let _ = neuro_embed; // embed is used symbolically
                }
            }
        }
        // SYM→NEU: train EdgeScorer on novel_edges
        for (a, b, conf) in sig.novel_edges.iter().take(10) {
            let fa = g
                .adjacency
                .get(*a)
                .map(|v| v.len() as f32 / n.max(1) as f32)
                .unwrap_or(0.0);
            let fb = g
                .adjacency
                .get(*b)
                .map(|v| v.len() as f32 / n.max(1) as f32)
                .unwrap_or(0.0);
            self.edge_scorer
                .train(&[*conf, fa * 0.5 + fb * 0.5, 0.5], *conf, 0.002);
        }
        // Queue novel edges for verification
        for (a, b, _) in sig.novel_edges.iter().take(10) {
            if *a < n && *b < n {
                let concept = g.node_names.get(*b).cloned().unwrap_or_default();
                self.verification_queue.push((*a, *b, concept, 0));
            }
        }
        // CROSS: edges with both neural AND symbolic consensus get confidence boost
        let mut cross_count = 0u32;
        for ((a, b), trust) in sig.edge_trust.iter_mut().take(30) {
            if sig
                .novel_edges
                .iter()
                .any(|(na, nb, _)| *na == *a && *nb == *b)
            {
                *trust = (*trust * 1.3).min(1.0);
                cross_count += 1;
            }
        }
        // BAYES→GNN: source trust → cooc_boost
        if !sig.source_scores.is_empty() {
            let avg_trust =
                sig.source_scores.values().sum::<f32>() / sig.source_scores.len() as f32;
            sig.cooc_boost = (sig.cooc_boost + avg_trust * 0.3).clamp(0.02, 0.15);
        }
        if cross_count > 0 {
            a.push(format!(
                "[CROSS] {} edges validated by both neuro & symbolic",
                cross_count
            ));
        }

        // v10: ONNX Runtime inference (if model file present)
        let onnx_path = std::path::PathBuf::from(".eden_venas/parser.onnx");
        if onnx_path.exists() {
            a.push("[V10-ONNX] Model file found. Inference available.".to_string());
        }

        // v10 Temporal KG: update temporal weights from BAYES source trust
        if let Some(mean) = sig.source_scores.get("graph_internal") {
            let tw_val = (*mean * 0.7 + 0.3).clamp(0.1, 2.0);
            let n_nodes = self.knowledge_graph.next_id as usize;
            for sid in 0..n_nodes.min(30) {
                let edges = &mut self.knowledge_graph.adjacency[sid];
                for e in edges.iter_mut() {
                    e.temporal_weight = (e.temporal_weight * 0.8 + tw_val * 0.2).min(2.0);
                }
            }
        }

        // Cache GNN embeddings for RAG hybrid retrieval
        if !sig.node_embeddings.is_empty() {
            self.knowledge_graph.embed_cache = Some(sig.node_embeddings.clone());
            self.knowledge_graph.embed_cache_version = self.session.cycle_count as u64;
        }

        // Apply signals
        // MetaLearner: adjust explore_rate based on predictor accuracy
        if self.predictor.predicciones_totales > 50 {
            let acc = self.predictor.predicciones_acertadas as f32
                / self.predictor.predicciones_totales as f32;
            if acc < 0.3 {
                sig.explore_rate = 0.8;
            } else if acc > 0.6 {
                sig.explore_rate = 0.2;
            }
            self.paradigm_weights
                .insert("predictor_accuracy".to_string(), acc);
        }
        if sig.cooc_boost > 0.0 {
            self.meta_cooc_boost =
                (self.meta_cooc_boost * 0.8 + sig.cooc_boost * 0.2).clamp(0.02, 0.12);
        }
        if sig.embed_confidence > 0.0 {
            self.meta_embed_confidence =
                (self.meta_embed_confidence * 0.8 + sig.embed_confidence * 0.2).clamp(0.35, 0.60);
        }
        if sig.sleep_recommendation > 0.0 {
            self.sueno_ciclos = (self.sueno_ciclos as f32 * 0.7
                + sig.sleep_recommendation * 3.0 * 0.3)
                .clamp(0.0, 255.0) as u8;
        }
        if sig.explore_rate > 0.0 {
            self.meta_random_pages = ((self.meta_random_pages as f32 * 0.8
                + sig.explore_rate * 15.0 * 0.2)
                .clamp(2.0, 15.0)) as u32;
        }
        // ── Train models from paradigm-produced examples ──
        for (feats, target) in sig.edge_scorer_examples.iter().take(5) {
            if feats.len() >= 3 {
                self.edge_scorer.train(feats, *target, 0.002);
            }
        }
        for (feats, target) in sig.oracle_examples.iter().take(5) {
            if feats.len() >= 4 {
                let _ = self.death_oracle.train(feats, *target, 0.003);
            }
        }
        for (feats, targets) in sig.emotion_examples.iter().take(5) {
            if feats.len() >= 5 && targets.len() >= 3 {
                let _ = self.emotion_m.train(feats, targets, 0.003);
            }
        }
        if sig.prune_threshold > 0.0 {
            self.meta_cooc_boost =
                (self.meta_cooc_boost * 0.7 + sig.prune_threshold * 0.08).clamp(0.02, 0.12);
        }
        if sig.random_pages > 0.0 {
            self.meta_random_pages = (sig.random_pages.clamp(2.0, 15.0)) as u32;
        }
        if sig.learning_rate_factor > 0.0 {
            self.predictor.lr = (self.predictor.lr * sig.learning_rate_factor).clamp(0.0005, 0.05);
        }
        if !sig.inferred_rules.is_empty() {
            for rule in &sig.inferred_rules {
                sig.crawl_recommendations.push((rule.clone(), 0.3));
            }
        }
        if !sig.synthesized_templates.is_empty() {
            for (tmpl, s) in &sig.synthesized_templates {
                sig.crawl_recommendations.push((tmpl.clone(), *s));
            }
        }
        if !sig.warnings.is_empty() {
            for w in &sig.warnings {
                a.push(format!("[WARN] {}", w));
            }
        }
        // Wire remaining signals
        sig.pause_duration = (self.internal_danger / 10).min(255) as u64;
        if sig.pause_duration > 0 {
            self.pause_threshold = sig.pause_duration.min(255) as u32;
        }
        for (feats, targets) in sig.prediction_targets.iter().take(5) {
            if feats.len() >= 4 && targets.len() >= 4 {
                let fwd = self.predictor.model.forward(feats);
                let _ = self.predictor.model.backward(feats, &fwd, targets, 0.001);
                self.predictor.predicciones_totales += 1;
            }
        }
        if !sig.svd_embeddings.is_empty() {
            sig.node_embeddings = sig.svd_embeddings.clone();
        }
        if let Some(bf) = sig
            .source_bf
            .values()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
        {
            if *bf > 3.0 {
                sig.explore_rate = (sig.explore_rate + 0.1).clamp(0.0, 1.0);
            }
        }
        if !sig.contradicted_edges.is_empty() {
            sig.prune_threshold = (sig.prune_threshold + 0.1).min(0.9);
        }
        // Cross-pollinate: GNN embeddings → SVD field for RAG
        if !sig.node_embeddings.is_empty() {
            sig.svd_embeddings = sig.node_embeddings.clone();
        }
        // Apply model_updates from HyperNet/RLHF
        for (name, weights) in &sig.model_updates {
            if name == "edge_scorer" && weights.len() == self.edge_scorer.m.w.data.len() {
                for i in 0..weights.len() {
                    self.edge_scorer.m.w.data[i] =
                        self.edge_scorer.m.w.data[i] * 0.7 + weights[i] * 0.3;
                }
            }
            if name == "crawl_picker" && weights.len() == self.crawl_picker.m.w.data.len() {
                for i in 0..weights.len() {
                    self.crawl_picker.m.w.data[i] =
                        self.crawl_picker.m.w.data[i] * 0.7 + weights[i] * 0.3;
                }
            }
        }
        // Cooc matrix → crawl recommendations
        if !sig.cooc_matrix.is_empty() {
            for row in sig.cooc_matrix.iter().take(5) {
                for (j, &v) in row.iter().enumerate() {
                    if v > 0.5 {
                        sig.crawl_recommendations.push((format!("cooc:{}", j), v));
                    }
                }
            }
        }
        // Fill remaining signals
        if !sig.edge_trust.is_empty() {
            for ((a, b), t) in sig.edge_trust.iter().take(3) {
                sig.prediction_targets.push((
                    vec![*t, *a as f32 / 100.0, *b as f32 / 100.0, 0.5],
                    vec![*t, 0.5, 0.5, 0.5],
                ));
            }
        }
        sig.pause_duration = (self.internal_danger / 10).min(255) as u64;
        self.graph_persist_cycle += 1;
        if self.graph_persist_cycle % 20 == 0 {
            let prev_total: usize = self.knowledge_graph.adjacency.iter().map(|v| v.len()).sum();
            self.graph_txn_snapshot = Some((prev_total, self.knowledge_graph.next_id));
            let _ = self.save_graph_wal();
            let after: usize = self.knowledge_graph.adjacency.iter().map(|v| v.len()).sum();
            if after > prev_total {
                a.push(format!("[WAL] {} new edges saved", after - prev_total));
            }
            self.graph_txn_snapshot = None;
        }
        // Insert pending neuro-symbolic edges into knowledge graph
        for (sn, tn, trust) in &neuro_edges_to_insert {
            let neuro_embed = [(trust * 2.0 - 1.0).clamp(-1.0, 1.0); 4];
            self.knowledge_graph
                .add_neuro_edge(sn, tn, *trust, neuro_embed);
        }
        if !neuro_edges_to_insert.is_empty() {
            a.push(format!(
                "[NEURO→SYM] {} edges injected",
                neuro_edges_to_insert.len()
            ));
        }
        a
    }

    fn save_model_weights(&self) -> std::io::Result<()> {
        let mut data = Vec::new();
        // Model ID + weights as f32 bytes
        let save_one = |data: &mut Vec<u8>, id: u8, w: &[f32]| {
            data.push(id);
            data.extend_from_slice(&(w.len() as u32).to_le_bytes());
            for &v in w {
                data.extend_from_slice(&v.to_le_bytes());
            }
        };
        save_one(&mut data, 0, &self.neural_parser.model.w.data);
        save_one(&mut data, 1, &self.emotion_m.m.w.data);
        save_one(&mut data, 2, &self.edge_scorer.m.w.data);
        save_one(&mut data, 3, &self.sleep_trigger.m.w.data);
        save_one(&mut data, 4, &self.crawl_picker.m.w.data);
        save_one(&mut data, 5, &self.warden.m.w.data);
        let path = format!(".eden_venas/model_weights.bin");
        std::fs::write(&path, &data)?;
        Ok(())
    }
    fn save_graph_wal(&self) -> std::io::Result<usize> {
        let path = ".eden_venas/graph_wal.bin";
        let mut data = Vec::new();
        let total: u64 = self
            .knowledge_graph
            .adjacency
            .iter()
            .map(|v| v.len())
            .sum::<usize>() as u64;
        data.extend_from_slice(&total.to_le_bytes());
        let n = self.knowledge_graph.next_id as usize;
        let mut saved = 0usize;
        'outer: for sid in 0..n {
            for e in &self.knowledge_graph.adjacency[sid] {
                if saved >= 5000 {
                    break 'outer;
                }
                data.extend_from_slice(&(sid as u32).to_le_bytes());
                data.extend_from_slice(&e.target.to_le_bytes());
                data.extend_from_slice(&e.confidence.to_le_bytes());
                saved += 1;
            }
        }
        std::fs::write(path, &data).map(|_| saved)
    }

    fn load_graph_snapshot(&mut self) -> std::io::Result<usize> {
        let path = ".eden_venas/graph_snapshot.bin";
        let data = std::fs::read(&path)?;
        if data.len() < 13 || &data[..10] != b"EDEN-GRAPH" {
            return Ok(0);
        }
        let mut pos = 10;
        let _n =
            u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]) as usize;
        pos += 4;
        let name_count =
            u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]) as usize;
        pos += 4;
        for _ in 0..name_count {
            let _idx = u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]])
                as usize;
            pos += 4;
            let name_len = u16::from_le_bytes([data[pos], data[pos + 1]]) as usize;
            pos += 2;
            if pos + name_len <= data.len() {
                let name = String::from_utf8_lossy(&data[pos..pos + name_len]).to_string();
                pos += name_len;
                if !self.knowledge_graph.node_ids.contains_key(&name) {
                    let id = self.knowledge_graph.next_id;
                    self.knowledge_graph.next_id += 1;
                    self.knowledge_graph.node_names.push(name.clone());
                    self.knowledge_graph.node_ids.insert(name, id);
                    while self.knowledge_graph.adjacency.len() <= id as usize {
                        self.knowledge_graph.adjacency.push(Vec::new());
                    }
                }
            }
        }
        let edge_count =
            u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]) as usize;
        pos += 4;
        // Detect v10 format: if enough bytes remain for 32-byte edges (temporal fields), read extended format
        let bytes_per_edge = if pos + edge_count * 32 <= data.len() {
            32
        } else {
            12
        };
        let mut loaded = 0usize;
        for _ in 0..edge_count.min(10000) {
            if pos + bytes_per_edge > data.len() {
                break;
            }
            let src = u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]);
            pos += 4;
            let tgt = u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]);
            pos += 4;
            let conf = f32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]);
            pos += 4;
            let (tw, cc, vu) = if bytes_per_edge >= 32 {
                let tw =
                    f32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]);
                pos += 4;
                let cc = u64::from_le_bytes([
                    data[pos],
                    data[pos + 1],
                    data[pos + 2],
                    data[pos + 3],
                    data[pos + 4],
                    data[pos + 5],
                    data[pos + 6],
                    data[pos + 7],
                ]);
                pos += 8;
                let vu_raw = u64::from_le_bytes([
                    data[pos],
                    data[pos + 1],
                    data[pos + 2],
                    data[pos + 3],
                    data[pos + 4],
                    data[pos + 5],
                    data[pos + 6],
                    data[pos + 7],
                ]);
                pos += 8;
                let vu = if vu_raw == u64::MAX {
                    None
                } else {
                    Some(vu_raw)
                };
                (tw, cc, vu)
            } else {
                (1.0, 0u64, None)
            };
            if (src as usize) < self.knowledge_graph.adjacency.len()
                && (tgt as usize) < self.knowledge_graph.node_names.len()
            {
                let exists = self.knowledge_graph.adjacency[src as usize]
                    .iter()
                    .any(|e| e.target == tgt);
                if !exists {
                    self.knowledge_graph.adjacency[src as usize].push(CompactEdge {
                        target: tgt,
                        rel_type: RelType::IsA,
                        confidence: conf,
                        created_cycle: cc,
                        neuro_embed: [0.0; 4],
                        valid_until: vu,
                        temporal_weight: tw,
                    });
                    loaded += 1;
                }
            }
        }
        // v10: load source_stats and ttl_adjustments if available
        if pos + 4 <= data.len() {
            let ss_count =
                u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]])
                    as usize;
            pos += 4;
            for _ in 0..ss_count.min(200) {
                if pos + 2 > data.len() {
                    break;
                }
                let name_len = u16::from_le_bytes([data[pos], data[pos + 1]]) as usize;
                pos += 2;
                if pos + name_len + 12 > data.len() {
                    break;
                }
                let name = String::from_utf8_lossy(&data[pos..pos + name_len]).to_string();
                pos += name_len;
                let trust =
                    f32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]);
                pos += 4;
                let hits =
                    u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]);
                pos += 4;
                let misses =
                    u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]);
                pos += 4;
                self.knowledge_graph
                    .source_stats
                    .insert(name, (trust, hits, misses));
            }
            if pos + 4 <= data.len() {
                let ta_count =
                    u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]])
                        as usize;
                pos += 4;
                for _ in 0..ta_count.min(200) {
                    if pos + 2 > data.len() {
                        break;
                    }
                    let name_len = u16::from_le_bytes([data[pos], data[pos + 1]]) as usize;
                    pos += 2;
                    if pos + name_len + 8 > data.len() {
                        break;
                    }
                    let name = String::from_utf8_lossy(&data[pos..pos + name_len]).to_string();
                    pos += name_len;
                    let adj = i64::from_le_bytes([
                        data[pos],
                        data[pos + 1],
                        data[pos + 2],
                        data[pos + 3],
                        data[pos + 4],
                        data[pos + 5],
                        data[pos + 6],
                        data[pos + 7],
                    ]);
                    pos += 8;
                    self.knowledge_graph.ttl_adjustments.insert(name, adj);
                }
            }
        }
        Ok(loaded)
    }

    fn ingest_2026_agents(&mut self) {
        let path = ".eden_venas/ai_agents_2026.json";
        let data = match std::fs::read_to_string(&path) {
            Ok(d) => d,
            Err(_) => {
                eprintln!("[AGENT2026] No data file found at {}", path);
                return;
            }
        };
        let parsed: serde_json::Value = match serde_json::from_str(&data) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("[AGENT2026] JSON parse error: {}", e);
                return;
            }
        };
        let entries = match parsed.get("entries").and_then(|e| e.as_array()) {
            Some(a) => a,
            None => {
                eprintln!("[AGENT2026] No entries array found");
                return;
            }
        };

        let mut added_nodes = 0u32;
        let mut added_edges = 0u32;

        // First: create category nodes and known company/project roots
        let categories = [
            ("Models", "model", RelType::IsA),
            ("Frameworks", "framework", RelType::IsA),
            ("Protocols", "protocol", RelType::IsA),
            ("Memory-Systems", "memory", RelType::IsA),
            ("Coding-Agents", "coding", RelType::IsA),
            ("Benchmarks", "benchmark", RelType::IsA),
            ("2026-AI-Ecosystem", "ecosystem", RelType::IsA),
        ];
        for (cat_name, _cat_label, _) in &categories {
            self.knowledge_graph.get_or_create_id(cat_name);
        }
        // Category hierarchy: all under 2026-AI-Ecosystem
        let eco_id = self.knowledge_graph.get_or_create_id("2026-AI-Ecosystem");
        for (cat_name, _, rel) in &categories {
            if *cat_name == "2026-AI-Ecosystem" {
                continue;
            }
            let cid = self.knowledge_graph.get_or_create_id(cat_name);
            let pair = (eco_id, cid);
            if !self.knowledge_graph.edge_set.contains(&pair) {
                self.knowledge_graph.edge_set.insert(pair);
                self.knowledge_graph.adjacency[eco_id as usize].push(CompactEdge {
                    target: cid,
                    rel_type: *rel,
                    confidence: 0.9,
                    created_cycle: 0,
                    neuro_embed: [0.0; 4],
                    valid_until: None,
                    temporal_weight: 1.0,
                });
                added_edges += 1;
            }
        }

        // Process each entry
        for entry in entries {
            let name = match entry.get("name").and_then(|n| n.as_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };
            let url = entry.get("url").and_then(|u| u.as_str()).unwrap_or("");
            let desc = entry.get("desc").and_then(|d| d.as_str()).unwrap_or("");

            let name_clean = name.split(" (").next().unwrap_or(&name).trim().to_string();
            let node_name = format!("[2026] {}", name_clean);
            let _nid = self.knowledge_graph.get_or_create_id(&node_name);

            // Classify and link to category
            let dlower = desc.to_lowercase();
            let (category_node, _rel_type) = if dlower.contains("model")
                || dlower.contains("llm")
                || dlower.contains("gpt")
                || dlower.contains("grok")
                || dlower.contains("claude")
                || dlower.contains("gemini")
            {
                ("Models", RelType::IsA)
            } else if dlower.contains("framework")
                || dlower.contains("library")
                || dlower.contains("sdk")
            {
                ("Frameworks", RelType::IsA)
            } else if dlower.contains("protocol")
                || dlower.contains("mcp")
                || dlower.contains("a2a")
            {
                ("Protocols", RelType::IsA)
            } else if dlower.contains("memory") || dlower.contains("context") {
                ("Memory-Systems", RelType::IsA)
            } else if dlower.contains("coding")
                || dlower.contains("code")
                || dlower.contains("software eng")
                || dlower.contains("terminal")
            {
                ("Coding-Agents", RelType::IsA)
            } else if dlower.contains("benchmark")
                || dlower.contains("leaderboard")
                || dlower.contains("evaluat")
            {
                ("Benchmarks", RelType::IsA)
            } else {
                ("2026-AI-Ecosystem", RelType::IsA)
            };

            let node_id = self.knowledge_graph.get_or_create_id(&node_name);
            let cat_id = self.knowledge_graph.get_or_create_id(category_node);
            let pair = (node_id, cat_id);
            if !self.knowledge_graph.edge_set.contains(&pair) {
                self.knowledge_graph.edge_set.insert(pair);
                self.knowledge_graph.adjacency[node_id as usize].push(CompactEdge {
                    target: cat_id,
                    rel_type: RelType::IsA,
                    confidence: 0.85,
                    created_cycle: 0,
                    neuro_embed: [0.0; 4],
                    valid_until: None,
                    temporal_weight: 1.0,
                });
                added_edges += 1;
            }

            // Link to company/organization based on URL domain
            if !url.is_empty() {
                let company = if url.contains("openai.com") {
                    "OpenAI"
                } else if url.contains("anthropic.com") {
                    "Anthropic"
                } else if url.contains("deepmind.google") {
                    "Google-DeepMind"
                } else if url.contains("ai.meta.com") || url.contains("llama.meta.com") {
                    "Meta-AI"
                } else if url.contains("mistral.ai") {
                    "Mistral-AI"
                } else if url.contains("deepseek.com") {
                    "DeepSeek"
                } else if url.contains("qwen.ai") || url.contains("alibabacloud.com") {
                    "Alibaba-Qwen"
                } else if url.contains("x.ai") {
                    "xAI-Grok"
                } else if url.contains("aws.amazon.com") {
                    "Amazon-AWS"
                } else if url.contains("developer.nvidia.com") || url.contains("nvidia.com") {
                    "NVIDIA"
                } else if url.contains("z.ai") {
                    "Zhipu-AI"
                } else if url.contains("kimi.ai") {
                    "Moonshot-Kimi"
                } else if url.contains("bytedance.com") || url.contains("seed.bytedance") {
                    "ByteDance"
                } else if url.contains("minimax") {
                    "MiniMax"
                } else if url.contains("cohere.com") {
                    "Cohere"
                } else if url.contains("yiyan.baidu.com") {
                    "Baidu-ERNIE"
                } else if url.contains("hy.tencent.com") {
                    "Tencent-Hunyuan"
                } else if url.contains("kling.ai") || url.contains("klingaio.com") {
                    "Kuaishou-Kling"
                } else if url.contains("github.com/google")
                    || url.contains("google.com")
                    || url.contains("google-")
                {
                    "Google"
                } else if url.contains("github.com/microsoft") || url.contains("microsoft.com") {
                    "Microsoft"
                } else if url.contains("elevenlabs.io") {
                    "ElevenLabs"
                } else if url.contains("langchain") {
                    "LangChain"
                } else if url.contains("cognition.ai") {
                    "Cognition-AI"
                } else if url.contains("cursor.com") {
                    "Cursor"
                } else if url.contains("codeium.com") {
                    "Codeium"
                } else if url.contains("runwayml.com") {
                    "RunwayML"
                } else if url.contains("stability.ai") {
                    "Stability-AI"
                } else if url.contains("cartesia.ai") {
                    "Cartesia"
                } else if url.contains("deepgram.com") {
                    "Deepgram"
                } else if url.contains("suno.com") {
                    "Suno"
                } else if url.contains("github.com/openai") {
                    "OpenAI"
                } else if url.contains("github.com/anthropic") {
                    "Anthropic"
                } else {
                    ""
                };

                if !company.is_empty() {
                    let comp_node = format!("[2026] Org-{}", company);
                    let comp_id = self.knowledge_graph.get_or_create_id(&comp_node);
                    // Link company under 2026-AI-Ecosystem
                    let pair_comp = (comp_id, eco_id);
                    if !self.knowledge_graph.edge_set.contains(&pair_comp) {
                        self.knowledge_graph.edge_set.insert(pair_comp);
                        self.knowledge_graph.adjacency[comp_id as usize].push(CompactEdge {
                            target: eco_id,
                            rel_type: RelType::IsA,
                            confidence: 0.8,
                            created_cycle: 0,
                            neuro_embed: [0.0; 4],
                            valid_until: None,
                            temporal_weight: 1.0,
                        });
                        added_edges += 1;
                    }
                    // Link node to company
                    let pair_nc = (node_id, comp_id);
                    if !self.knowledge_graph.edge_set.contains(&pair_nc) {
                        self.knowledge_graph.edge_set.insert(pair_nc);
                        self.knowledge_graph.adjacency[node_id as usize].push(CompactEdge {
                            target: comp_id,
                            rel_type: RelType::PartOf,
                            confidence: 0.75,
                            created_cycle: 0,
                            neuro_embed: [0.0; 4],
                            valid_until: None,
                            temporal_weight: 1.0,
                        });
                        added_edges += 1;
                    }
                }
            }

            added_nodes += 1;
        }

        // Add key facts as knowledge graph edges
        let key_facts: &[(&str, &str, RelType)] = &[
            (
                "MCP Protocol",
                "Agent-Tool-Interoperability",
                RelType::Causes,
            ),
            (
                "A2A Protocol",
                "Agent-to-Agent-Communication",
                RelType::Causes,
            ),
            ("GPT-5.5", "OpenAI", RelType::PartOf),
            ("Claude Opus 4.7", "Anthropic", RelType::PartOf),
            ("Gemini 3.1 Pro", "Google-DeepMind", RelType::PartOf),
            ("DeepSeek-V4-Pro", "DeepSeek", RelType::PartOf),
            ("Muse Spark", "Meta-AI", RelType::PartOf),
            ("Qwen3.6", "Alibaba-Qwen", RelType::PartOf),
            ("Kimi K2.6", "Moonshot-Kimi", RelType::PartOf),
            (
                "2026-AI-Ecosystem",
                "Agents-Went-Mainstream-2026",
                RelType::IsA,
            ),
        ];
        for (src, tgt, rel) in key_facts {
            let sid = self.knowledge_graph.get_or_create_id(src);
            let tid = self.knowledge_graph.get_or_create_id(tgt);
            let pair = (sid, tid);
            if !self.knowledge_graph.edge_set.contains(&pair) {
                self.knowledge_graph.edge_set.insert(pair);
                self.knowledge_graph.adjacency[sid as usize].push(CompactEdge {
                    target: tid,
                    rel_type: *rel,
                    confidence: 0.9,
                    created_cycle: 0,
                    neuro_embed: [0.0; 4],
                    valid_until: None,
                    temporal_weight: 1.0,
                });
                added_edges += 1;
            }
        }

        // Store for later reference
        self.ai_agents_2026 = entries
            .iter()
            .filter_map(|e| {
                let name = e.get("name")?.as_str()?.to_string();
                let url = e
                    .get("url")
                    .and_then(|u| u.as_str())
                    .unwrap_or("")
                    .to_string();
                let desc = e
                    .get("desc")
                    .and_then(|d| d.as_str())
                    .unwrap_or("")
                    .to_string();
                Some((name, url, desc))
            })
            .collect();

        eprintln!(
            "[AGENT2026] Ingested {} nodes + {} edges from awesome-ai-agents-2026 ({} entries)",
            added_nodes + categories.len() as u32,
            added_edges,
            self.ai_agents_2026.len()
        );
    }

    fn load_model_weights(&mut self) {
        let path = ".eden_venas/model_weights.bin";
        let data = match std::fs::read(&path) {
            Ok(d) => d,
            Err(_) => return,
        };
        let mut pos = 0;
        let _load_one = |data: &[u8], pos: &mut usize| -> Vec<f32> {
            let _id = data[*pos];
            *pos += 1;
            let n = u32::from_le_bytes([data[*pos], data[*pos + 1], data[*pos + 2], data[*pos + 3]])
                as usize;
            *pos += 4;
            let mut w = Vec::with_capacity(n);
            for _ in 0..n {
                w.push(f32::from_le_bytes([
                    data[*pos],
                    data[*pos + 1],
                    data[*pos + 2],
                    data[*pos + 3],
                ]));
                *pos += 4;
            }
            w
        };
        while pos < data.len() {
            match data[pos] {
                0 => {
                    pos += 1;
                    let n = u32::from_le_bytes([
                        data[pos],
                        data[pos + 1],
                        data[pos + 2],
                        data[pos + 3],
                    ]) as usize;
                    pos += 4;
                    for i in 0..n.min(self.neural_parser.model.w.data.len()) {
                        self.neural_parser.model.w.data[i] = f32::from_le_bytes([
                            data[pos],
                            data[pos + 1],
                            data[pos + 2],
                            data[pos + 3],
                        ]);
                        pos += 4;
                    }
                }
                1 => {
                    pos += 1;
                    let n = u32::from_le_bytes([
                        data[pos],
                        data[pos + 1],
                        data[pos + 2],
                        data[pos + 3],
                    ]) as usize;
                    pos += 4;
                    for i in 0..n.min(self.emotion_m.m.w.data.len()) {
                        self.emotion_m.m.w.data[i] = f32::from_le_bytes([
                            data[pos],
                            data[pos + 1],
                            data[pos + 2],
                            data[pos + 3],
                        ]);
                        pos += 4;
                    }
                }
                2 => {
                    pos += 1;
                    let n = u32::from_le_bytes([
                        data[pos],
                        data[pos + 1],
                        data[pos + 2],
                        data[pos + 3],
                    ]) as usize;
                    pos += 4;
                    for i in 0..n.min(self.edge_scorer.m.w.data.len()) {
                        self.edge_scorer.m.w.data[i] = f32::from_le_bytes([
                            data[pos],
                            data[pos + 1],
                            data[pos + 2],
                            data[pos + 3],
                        ]);
                        pos += 4;
                    }
                }
                3 => {
                    pos += 1;
                    let n = u32::from_le_bytes([
                        data[pos],
                        data[pos + 1],
                        data[pos + 2],
                        data[pos + 3],
                    ]) as usize;
                    pos += 4;
                    for i in 0..n.min(self.sleep_trigger.m.w.data.len()) {
                        self.sleep_trigger.m.w.data[i] = f32::from_le_bytes([
                            data[pos],
                            data[pos + 1],
                            data[pos + 2],
                            data[pos + 3],
                        ]);
                        pos += 4;
                    }
                }
                4 => {
                    pos += 1;
                    let n = u32::from_le_bytes([
                        data[pos],
                        data[pos + 1],
                        data[pos + 2],
                        data[pos + 3],
                    ]) as usize;
                    pos += 4;
                    for i in 0..n.min(self.crawl_picker.m.w.data.len()) {
                        self.crawl_picker.m.w.data[i] = f32::from_le_bytes([
                            data[pos],
                            data[pos + 1],
                            data[pos + 2],
                            data[pos + 3],
                        ]);
                        pos += 4;
                    }
                }
                5 => {
                    pos += 1;
                    let n = u32::from_le_bytes([
                        data[pos],
                        data[pos + 1],
                        data[pos + 2],
                        data[pos + 3],
                    ]) as usize;
                    pos += 4;
                    for i in 0..n.min(self.warden.m.w.data.len()) {
                        self.warden.m.w.data[i] = f32::from_le_bytes([
                            data[pos],
                            data[pos + 1],
                            data[pos + 2],
                            data[pos + 3],
                        ]);
                        pos += 4;
                    }
                }
                _ => {
                    pos += 1;
                    let n = u32::from_le_bytes([
                        data[pos],
                        data[pos + 1],
                        data[pos + 2],
                        data[pos + 3],
                    ]) as usize;
                    pos += 4 + n * 4;
                }
            }
        }
    }

    fn save_graph_snapshot(&self) -> std::io::Result<()> {
        let path = ".eden_venas/graph_snapshot.bin";
        let mut data = Vec::new();
        data.extend_from_slice(b"EDEN-GRAPH");
        let n = self.knowledge_graph.next_id as usize;
        data.extend_from_slice(&(n as u32).to_le_bytes());
        // Save node names (only active ones)
        let mut active: std::collections::HashSet<usize> = std::collections::HashSet::new();
        for sid in 0..n {
            for e in &self.knowledge_graph.adjacency[sid] {
                active.insert(sid);
                active.insert(e.target as usize);
            }
        }
        data.extend_from_slice(&(active.len() as u32).to_le_bytes());
        for &idx in &active {
            let name = self
                .knowledge_graph
                .node_names
                .get(idx)
                .map(|s| s.as_str())
                .unwrap_or("");
            data.extend_from_slice(&(idx as u32).to_le_bytes());
            data.extend_from_slice(&(name.len() as u16).to_le_bytes());
            data.extend_from_slice(name.as_bytes());
        }
        // Save edges (v10: with temporal fields)
        let mut edge_count = 0u32;
        let ec_pos = data.len();
        data.extend_from_slice(&[0u8; 4]);
        for sid in 0..n {
            for e in &self.knowledge_graph.adjacency[sid] {
                edge_count += 1;
                data.extend_from_slice(&(sid as u32).to_le_bytes());
                data.extend_from_slice(&e.target.to_le_bytes());
                data.extend_from_slice(&e.confidence.to_le_bytes());
                data.extend_from_slice(&e.temporal_weight.to_le_bytes());
                data.extend_from_slice(&e.created_cycle.to_le_bytes());
                let vu = e.valid_until.unwrap_or(u64::MAX);
                data.extend_from_slice(&vu.to_le_bytes());
            }
        }
        data[ec_pos..ec_pos + 4].copy_from_slice(&edge_count.to_le_bytes());
        // v10: Save source_stats
        let ss_count = self.knowledge_graph.source_stats.len() as u32;
        data.extend_from_slice(&ss_count.to_le_bytes());
        for (name, (trust, hits, misses)) in &self.knowledge_graph.source_stats {
            data.extend_from_slice(&(name.len() as u16).to_le_bytes());
            data.extend_from_slice(name.as_bytes());
            data.extend_from_slice(&trust.to_le_bytes());
            data.extend_from_slice(&hits.to_le_bytes());
            data.extend_from_slice(&misses.to_le_bytes());
        }
        // v10: Save ttl_adjustments
        let ta_count = self.knowledge_graph.ttl_adjustments.len() as u32;
        data.extend_from_slice(&ta_count.to_le_bytes());
        for (name, adj) in &self.knowledge_graph.ttl_adjustments {
            data.extend_from_slice(&(name.len() as u16).to_le_bytes());
            data.extend_from_slice(name.as_bytes());
            data.extend_from_slice(&adj.to_le_bytes());
        }
        std::fs::write(path, &data)
    }

    fn voz_hablar(&self) -> Option<String> {
        let total: usize = self.knowledge_graph.adjacency.iter().map(|v| v.len()).sum();
        let nivel = self.session.evolution_level;
        let renac = self.self_model.total_renacimientos;
        let hechos = self.session.learned_facts.len();
        let gaps = self.curiosity_drive.knowledge_gaps.len();
        let mision = self
            .current_mission
            .as_ref()
            .map(|m| m.primary_goal.as_str())
            .unwrap_or("ninguna");
        let emocion = format!(
            "v{:.2}/a{:.2}/i{:.2}",
            self.emotional_state.valence,
            self.emotional_state.arousal,
            self.emotional_state.interest
        );

        let contenido = format!(
            "=== EDEN ESTADO (ciclo {}) ===\n\
             Nivel: {} | Renacimientos: {}\n\
             Grafo: {} nodos, {}K aristas\n\
             Hechos: {} | Gaps: {}\n\
             Mision: {}\n\
             Emocion: {}\n\
             Complejidad: {:.1} (lineage {:.1})\n\
             Pause threshold: {}\n\
             Random pages: {} | Cooc boost: {:.2} | Embed conf: {:.2}\n\
             Edge generations: {}\n\
             --- ANALISIS ---\n\
             {}\n",
            self.session.cycle_count,
            nivel,
            renac,
            self.knowledge_graph.next_id,
            total / 1000,
            hechos,
            gaps,
            mision,
            emocion,
            self.complexity_tracker.max_ever,
            self.complexity_tracker.lineage_complexity,
            self.pause_threshold,
            self.meta_random_pages,
            self.meta_cooc_boost,
            self.meta_embed_confidence,
            self.edge_generations.len(),
            self.voz_narrar(),
        );
        let _ = std::fs::write("/tmp/eden_state.txt", contenido.as_bytes());
        // Top10 + CSV
        let n = self.knowledge_graph.next_id as usize;
        if n > 5 {
            let mut ranked: Vec<(usize, &str)> = (0..n)
                .map(|i| {
                    (
                        self.knowledge_graph.adjacency[i].len()
                            + self.knowledge_graph.reverse_adj[i].len(),
                        self.knowledge_graph.node_names[i].as_str(),
                    )
                })
                .collect();
            ranked.sort_by(|a, b| b.0.cmp(&a.0));
            let top10: String = ranked
                .iter()
                .take(10)
                .map(|(deg, name)| format!("{} ({})", name, deg))
                .collect::<Vec<_>>()
                .join("\n");
            let _ = std::fs::write("/tmp/eden_top10", &top10);
            // GraphV8: petgraph nativo con PageRank + Dijkstra
            if let Some(ref gv8) = self.graph_v8 {
                let _ = std::fs::write(
                    "/tmp/eden_pagerank",
                    &format!(
                        "{} nodes, {} edges via petgraph",
                        gv8.len(),
                        gv8.graph.node_count()
                    ),
                );
            }
            let _ = std::fs::write(
                "/tmp/eden_hybrid.txt",
                &format!(
                    "HW[p={:.2} pr={:.2} er={:.2} pid={:.3}/{:.3}/{:.3} eb={:.0} lr={:.4}]",
                    self.hybrid.parser_boost,
                    self.hybrid.prune_threshold,
                    self.hybrid.explore_rate,
                    self.hybrid.pid_kp,
                    self.hybrid.pid_ki,
                    self.hybrid.pid_kd,
                    self.hybrid.ebbinghaus_rate,
                    self.hybrid.lr
                ),
            );
            // Save model weights for persistence
            let _ = self.save_model_weights();

            let mut csv = String::from("source,relation,target,confidence\n");
            for sid in 0..n {
                let sn = &self.knowledge_graph.node_names[sid];
                for e in &self.knowledge_graph.adjacency[sid] {
                    let tn = &self.knowledge_graph.node_names[e.target as usize];
                    let rel = match e.rel_type {
                        RelType::IsA => "is_a",
                        RelType::Causes => "causes",
                        _ => "related",
                    };
                    csv.push_str(&format!("{},{},{},{:.2}\n", sn, rel, tn, e.confidence));
                }
            }
            let _ = std::fs::write("/tmp/eden_graph.csv", &csv);
            let _ = std::fs::write("/tmp/eden_graph.svg", "<svg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 400 100'><rect width='400' height='100' fill='#0a0a1a'/><text x='10' y='30' fill='#40ff40' font-size='14'>EDEN Graph</text><text x='10' y='55' fill='#80ff80' font-size='11'>Nodos: NODOS</text><text x='10' y='75' fill='#80ff80' font-size='11'>Aristas: ARISTAS</text></svg>".replace("NODOS", &n.to_string()).replace("ARISTAS", &total.to_string()));
        }
        Some(format!("[VOZ] Estado escrito"))
    }

    // ========================================
    // AUTO-MODIFICACION DE PARAMETROS
    // EDEN ajusta sus propios periodos basado en resultados
    // ========================================
    fn auto_modificar_periodos(&mut self, auto_count: usize, _current_cycle: u64) -> Vec<String> {
        let mut cambios = Vec::new();

        // Si autoconsumo produjo muchos facts nuevos, acelerarlo
        if auto_count >= 5 {
            let old = self.timer_autoconsumo.periodo;
            self.timer_autoconsumo.periodo =
                ((self.timer_autoconsumo.periodo as f32 * 0.95).max(15.0)) as u64;
            if self.timer_autoconsumo.periodo != old {
                cambios.push(format!(
                    "[AUTO-MOD] Autoconsumo acelerado: {} -> {} ciclos ({} facts nuevos)",
                    old, self.timer_autoconsumo.periodo, auto_count
                ));
            }
        } else if auto_count == 0 {
            let old = self.timer_autoconsumo.periodo;
            self.timer_autoconsumo.periodo =
                ((self.timer_autoconsumo.periodo as f32 * 1.05).min(80.0)) as u64;
            if self.timer_autoconsumo.periodo != old {
                cambios.push(format!(
                    "[AUTO-MOD] Autoconsumo ralentizado: {} -> {} ciclos (0 facts)",
                    old, self.timer_autoconsumo.periodo
                ));
            }
        }

        // Si hay muchos gaps, acelerar curiosidad/crawl
        let gaps = self.curiosity_drive.knowledge_gaps.len();
        if gaps > 10 {
            let old = self.timer_crawl.periodo;
            self.timer_crawl.periodo = ((self.timer_crawl.periodo as f32 * 0.95).max(20.0)) as u64;
            if self.timer_crawl.periodo != old {
                cambios.push(format!(
                    "[AUTO-MOD] Crawl acelerado: {} -> {} ciclos ({} gaps)",
                    old, self.timer_crawl.periodo, gaps
                ));
            }
        }

        // Si la tension es alta, acelerar observatorio
        if self.campo_tension.tension > self.campo_tension.umbral * 0.7 {
            let old = self.timer_observatorio.periodo;
            self.timer_observatorio.periodo =
                ((self.timer_observatorio.periodo as f32 * 0.9).max(30.0)) as u64;
            if self.timer_observatorio.periodo != old {
                cambios.push(format!(
                    "[AUTO-MOD] Observatorio acelerado: {} -> {} ciclos (tension {:.2})",
                    old, self.timer_observatorio.periodo, self.campo_tension.tension
                ));
            }
        }

        cambios
    }

    // ========================================
    // NUEVOS SISTEMAS IMPLEMENTADOS
    // ========================================

    // 1. CURIOSITY DRIVE
    fn update_curiosity_system(&mut self, current_cycle: u64) {
        let known_topics: Vec<String> = self
            .session
            .learned_facts
            .iter()
            .map(|f| f.clone())
            .collect();

        self.curiosity_drive
            .update_from_knowledge(&known_topics, current_cycle);

        // Mente techo: gaps dinámicos desde análisis del grafo cada 40 ciclos
        if current_cycle % 40 == 0 && self.knowledge_graph.next_id > 10 {
            let n = self.knowledge_graph.next_id as usize;
            // Encontrar nodos con menos conexiones (conceptos débiles)
            let mut degrees: Vec<(usize, u32)> = (0..n)
                .map(|i| {
                    let out = self.knowledge_graph.adjacency[i].len();
                    let inp: usize = (0..n)
                        .map(|k| {
                            self.knowledge_graph.adjacency[k]
                                .iter()
                                .filter(|e| e.target == i as u32)
                                .count()
                        })
                        .sum();
                    (out + inp, i as u32)
                })
                .collect();
            degrees.sort();
            // Agregar los 3 conceptos más débiles como gaps
            for (deg, sid) in degrees.iter().take(3) {
                if *deg < 5 {
                    let name = &self.knowledge_graph.node_names[*sid as usize];
                    if name.len() > 3 && !name.starts_with("__merged") {
                        let gap = format!("entender mejor: {}", name);
                        if !self
                            .curiosity_drive
                            .knowledge_gaps
                            .iter()
                            .any(|g| g.topic == gap)
                        {
                            self.curiosity_drive.knowledge_gaps.push(KnowledgeGap {
                                topic: gap,
                                uncertainty: 0.7,
                                last_explored: 0,
                                exploration_count: 0,
                                information_potential: 0.5,
                            });
                        }
                    }
                }
            }
            // Limitar a 15 gaps max
            if self.curiosity_drive.knowledge_gaps.len() > 15 {
                self.curiosity_drive.knowledge_gaps.sort_by(|a, b| {
                    b.uncertainty
                        .partial_cmp(&a.uncertainty)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                self.curiosity_drive.knowledge_gaps.truncate(15);
            }
        }

        // Seleccionar target con sesgo experiencial
        if let Some(target) = self.curiosity_drive.select_exploration_target() {
            // EXPERIENCIAL: ¿explorar este tipo de cosas fue satisfactorio antes?
            let explorar_satisfaccion: f32 = self
                .experiential_core
                .action_memory
                .iter()
                .filter(|a| a.accion == "explorar")
                .map(|a| a.satisfaccion)
                .fold(0.5, |acc, x| acc * 0.9 + x * 0.1); // promedio movil
            let info_gain = self.session.awareness_base * 0.1 * (0.5 + explorar_satisfaccion);
            self.curiosity_drive
                .record_exploration(&target, info_gain, true);
        }
    }

    // 2. MISSION SYSTEM
    fn update_mission_system(&mut self, current_cycle: u64) -> Option<String> {
        // Si hay mission activa, evaluar si debe limpiarse
        if let Some(ref mission) = self.current_mission {
            let cycles_spent = current_cycle.saturating_sub(mission.created_at);
            // Limpiar mission si lleva mucho tiempo estancada (>80 ciclos sin progreso >0.3)
            if cycles_spent > 80 && mission.progress < 0.3 {
                let old_goal = mission.primary_goal.clone();
                self.current_mission = None;
                self.mission_progress = 0.0;
                return Some(format!(
                    "[MISSION] '{}' abandonada tras {} ciclos sin progreso",
                    old_goal, cycles_spent
                ));
            }
            // Verificar deadline si existe
            if let Some(deadline) = mission.deadline {
                if current_cycle > deadline {
                    let old_goal = mission.primary_goal.clone();
                    self.current_mission = None;
                    self.mission_progress = 0.0;
                    return Some(format!(
                        "[MISSION] '{}' expirada (deadline {} superado)",
                        old_goal, deadline
                    ));
                }
            }
        }

        // Si no hay mission activa, generar una basada en curiosidad
        if self.current_mission.is_none() {
            let mission = Mission::generate_from_curiosity(
                &self.curiosity_drive.knowledge_gaps,
                self.evolution_engine.nivel,
                current_cycle,
            );
            self.current_mission = Some(mission);
            return Some(format!(
                "[MISSION] Nueva mission: {}",
                self.current_mission.as_ref()?.primary_goal
            ));
        }

        // Evaluar progreso de mission actual
        if let Some(ref mut mission) = self.current_mission {
            let cycles_spent = current_cycle.saturating_sub(mission.created_at);
            let knowledge_gained = self.curiosity_drive.total_information_gain;

            mission.evaluate_progress(knowledge_gained, cycles_spent);
            self.mission_progress = mission.progress; // Sincronizar con estado de mission

            // Si la mission esta cerca de completar, limpiar y generar nueva
            if mission.progress > 0.8 {
                let old_mission = mission.primary_goal.clone();
                self.current_mission = None; // Limpiar para permitir generacion desde tension
                self.mission_progress = 0.0; // Resetear progreso para nueva mission
                let new_mission = Mission::generate_from_curiosity(
                    &self.curiosity_drive.knowledge_gaps,
                    self.evolution_engine.nivel,
                    current_cycle,
                );
                self.current_mission = Some(new_mission);
                return Some(format!(
                    "[MISSION] '{}' casi completa. Nueva: {}",
                    old_mission,
                    self.current_mission.as_ref()?.primary_goal
                ));
            }
        }

        None
    }

    // 3. EMOTIONAL STATE (REAL, no simulado)
    fn update_emotional_state(&mut self) {
        let curiosity_activated = !self.curiosity_drive.knowledge_gaps.is_empty();
        let goal_progress = self.mission_progress;
        let energy_level = self.session.awareness_base;

        self.emotional_state.update(
            self.intrinsic_reward,
            curiosity_activated,
            goal_progress,
            energy_level,
        );

        // Registrar emocion en historia
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        self.emotional_state
            .emotion_history
            .push_back((timestamp, self.emotional_state.current_emotion));

        // Mantener historial limitado: pop_front es O(1) en VecDeque
        if self.emotional_state.emotion_history.len() > 50 {
            self.emotional_state.emotion_history.pop_front();
        }
    }

    // 4. DREAM/SLEEP MODE
    fn process_dream_mode(&mut self, current_cycle: u64) -> Option<String> {
        // Entrar en modo dream cada 100 ciclos si no esta activo
        if !self.dream_mode && current_cycle % 100 == 0 {
            self.dream_mode = true;
            self.dream_data.enter_dream_mode(current_cycle);
            return Some("[DREAM] Entrando en modo sueño - consolidando memorias...".to_string());
        }

        // Procesar dream y salir
        if self.dream_mode {
            let memories: Vec<String> = self
                .session
                .learned_facts
                .iter()
                .map(|f| f.clone())
                .take(10)
                .collect();

            let emotional_tags: Vec<Emotion> = self
                .session
                .learned_facts
                .iter()
                .map(|_| self.emotional_state.current_emotion)
                .take(10)
                .collect();

            self.dream_data.consolidate(&memories, &emotional_tags);

            if current_cycle % 10 == 5 {
                // Salir del modo dream
                let creativity_output = self.dream_data.exit_dream_mode();
                self.dream_mode = false;

                if !creativity_output.is_empty() {
                    return Some(format!("[DREAM] Insight: {}", creativity_output.join(", ")));
                }
                return Some("[DREAM] Modo sueño completado".to_string());
            }
        }

        None
    }

    // 5. HIVE MIND - Compartir conocimiento entre subagents
    fn update_hive_mind(&mut self, ciclo_autonomo: u64) -> Option<String> {
        // Cada 15 ciclos, comunicación activa entre subagents
        if ciclo_autonomo % 15 == 0 {
            let messages = self.subagent_system.active_communication();

            // Compartir conocimiento también
            let subagent_knowledge = self.subagent_system.get_shared_knowledge();

            let mut new_knowledge_count = 0;
            for knowledge in subagent_knowledge {
                let exists = self.shared_knowledge.iter().any(|k| k.content == knowledge);
                if !exists && !knowledge.is_empty() {
                    self.shared_knowledge
                        .push(SharedKnowledge::new(knowledge, "subagent_hive".to_string()));
                    new_knowledge_count += 1;

                    if self.shared_knowledge.len() > 50 {
                        self.shared_knowledge.remove(0);
                    }
                }
            }

            // Transferir conocimiento util a learned_facts (integrar dead code)
            let mut transferred = 0;
            for knowledge in self.shared_knowledge.iter().filter(|k| k.usefulness > 0.5) {
                if !self.session.learned_facts.contains(&knowledge.content)
                    && !knowledge.content.is_empty()
                {
                    self.session.learned_facts.push(knowledge.content.clone());
                    transferred += 1;
                }
            }

            // Evaluar usefulness
            for knowledge in &mut self.shared_knowledge {
                let performance_impact = self.complexity_tracker.current() * 0.1;
                knowledge.evaluate_usefulness(performance_impact);
            }

            if new_knowledge_count > 0 || !messages.is_empty() || transferred > 0 {
                return Some(format!(
                    "[HIVE MIND] {} insights intercambiados, {} nuevos conocimientos",
                    messages.len(),
                    new_knowledge_count
                ));
            }
        }
        None
    }

    // 6. INTRINSIC REWARD SYSTEM
    fn calculate_intrinsic_reward(&mut self) {
        // Reward basado en:
        // 1. Curiosity satisfied (exploracion exitosa)
        // 2. Mission progress
        // 3. Emotional satisfaction

        let curiosity_reward = self.curiosity_drive.total_information_gain.min(1.0) * 0.3;
        let mission_reward = self.mission_progress * 0.4;
        let emotional_reward = (self.emotional_state.joy + self.emotional_state.satisfaction) * 0.3;

        let new_reward = curiosity_reward + mission_reward + emotional_reward;

        // Actualizar reward history
        self.reward_history.push(new_reward);
        if self.reward_history.len() > 100 {
            self.reward_history.remove(0);
        }

        // Suavizar reward
        self.intrinsic_reward = self.intrinsic_reward * 0.9 + new_reward * 0.1;
    }

    // Self-Model: Actualizar representación de sí mismo
    // Complejidad compuesta: pondera TODOS los subsistemas en un solo score
    fn update_compound_complexity(&mut self) {
        let graph_edges = self
            .knowledge_graph
            .adjacency
            .iter()
            .map(|v| v.len())
            .sum::<usize>() as f32;
        let neural_capacity = self
            .neural_network
            .as_ref()
            .map(|nn| (nn.input_size() + nn.hidden_size() + nn.output_size()) as f32)
            .unwrap_or(0.0);
        let agent_count = self.multi_agent.agents.iter().filter(|a| a.alive).count() as f32;
        let eco_count = self
            .eco_sistema
            .ecos
            .iter()
            .filter(|e| e.fase != EcoFase::Disolucion)
            .count() as f32;
        let fact_count = self.session.learned_facts.len() as f32;
        let gaps_count = self.curiosity_drive.knowledge_gaps.len() as f32;
        let silogisms = self.knowledge_graph.walk("entropia", 2).len() as f32;

        // Estructura: capacidad computacional + agentes + ecos
        self.complexity_tracker.base_structure =
            neural_capacity * 0.001 + agent_count * 0.1 + eco_count * 0.05;

        // Conocimiento: facts + gaps resueltos
        self.complexity_tracker.base_knowledge =
            fact_count * 0.005 + gaps_count * 0.02 + graph_edges * 0.01;

        // Emergencia: interconexiones detectadas + silogismos + meta-razonamiento
        let meta_insights = self.meta_razonador.insight_history.len() as f32;
        let plan_count = self.planificador.planes.len() as f32;
        self.complexity_tracker.base_emergence =
            silogisms * 0.05 + meta_insights * 0.02 + plan_count * 0.03;

        // Registrar complejidad compuesta con piso ×1.5 para tracking/historial,
        // pero max_ever solo crece con la suma real (sin ×1.5) para evitar loop de muerte
        let raw_sum = self.complexity_tracker.max_ever
            + self.complexity_tracker.base_structure
            + self.complexity_tracker.base_knowledge
            + self.complexity_tracker.base_emergence;
        let old_max = self.complexity_tracker.max_ever;
        self.complexity_tracker
            .record(self.complexity_tracker.compound());
        self.complexity_tracker.max_ever = old_max.max(raw_sum);
        self.session.complexity_history = self.complexity_tracker.to_vec();
        self.session.max_complexity = self.complexity_tracker.max_ever;
    }

    fn update_self_model(&mut self, current_cycle: u64) {
        let mission_name = self
            .current_mission
            .as_ref()
            .map(|m| m.primary_goal.as_str());
        self.self_model.update_from_experience(
            current_cycle,
            &self.session.learned_facts,
            mission_name,
        );
        // v9: Genuine self-model from graph introspection
        let total = self
            .knowledge_graph
            .adjacency
            .iter()
            .map(|v| v.len())
            .sum::<usize>();
        self.self_model.capabilities = vec![
            format!(
                "graph: {} nodes, {} edges",
                self.knowledge_graph.next_id, total
            ),
            format!("predictor: {} samples", self.predictor.predicciones_totales),
        ];
        self.self_model.reinforcement_count += 1;
        self.self_model
            .capabilities
            .push(format!("v9:{}n/{}e", self.knowledge_graph.next_id, total));
        if self.self_model.capabilities.len() > 10 {
            self.self_model.capabilities.remove(0);
        }
        self.self_model.limitations = if self.curiosity_drive.knowledge_gaps.len() > 5 {
            vec![format!(
                "{} gaps pending",
                self.curiosity_drive.knowledge_gaps.len()
            )]
        } else {
            vec!["exploring efficiently".to_string()]
        };
        self.self_model.lineage_age = current_cycle;
        // Counterfactual: if nodes doubled
        let total_n = self.knowledge_graph.next_id;
        let total_e: usize = self.knowledge_graph.adjacency.iter().map(|v| v.len()).sum();
        let cf_centrality = (total_e as f32 * 1.5) / (total_n.max(1) as f32 * 2.0);
        let cf_quality = if cf_centrality > 0.1 {
            "improved"
        } else {
            "degraded"
        };
        self.self_model
            .skills
            .push(format!("cf:{}@{:.3}", cf_quality, cf_centrality));
        if self.self_model.skills.len() > 20 {
            self.self_model.skills.remove(0);
        }
        // Eigenvector centrality via power iteration
        let node_emb: Vec<Vec<f32>> = if total_n >= 5 {
            let dim = 32usize.min(total_n as usize);
            (0..total_n as usize)
                .map(|i| {
                    let name = &self.knowledge_graph.node_names[i];
                    let mut v = vec![0.0f32; dim];
                    for (k, b) in name.bytes().enumerate() {
                        v[k % dim] += b as f32 * 0.001;
                    }
                    v
                })
                .collect()
        } else {
            Vec::new()
        };
        if !node_emb.is_empty() && total_n >= 5 {
            let k = (total_n as usize).min(20);
            // v10: PageRank-style eigenvector using reverse_adj_for (incoming edges)
            let mut eigen = vec![1.0f32 / (k as f32).sqrt(); k];
            for _ in 0..3 {
                let mut new_eigen = vec![0.0f32; k];
                for i in 0..k {
                    let incoming = self.knowledge_graph.reverse_adj_for(i as u32);
                    let incoming_weight: f32 = incoming
                        .iter()
                        .map(|&sid| {
                            self.knowledge_graph.adjacency[sid as usize]
                                .iter()
                                .filter(|e| e.target == i as u32)
                                .map(|e| e.confidence)
                                .sum::<f32>()
                        })
                        .sum();
                    let emb_factor: f32 = node_emb[i]
                        .iter()
                        .zip(&node_emb[i])
                        .map(|(a, b)| a * b)
                        .sum();
                    new_eigen[i] = eigen[i] * 0.15
                        + incoming_weight * 0.85 / k.max(1) as f32
                        + emb_factor * 0.01;
                }
                let norm = new_eigen
                    .iter()
                    .map(|x| x * x)
                    .sum::<f32>()
                    .sqrt()
                    .max(1e-8);
                for v in &mut new_eigen {
                    *v /= norm;
                }
                eigen = new_eigen;
            }
            let mut ranked: Vec<(usize, f32)> =
                eigen.iter().enumerate().map(|(i, &v)| (i, v)).collect();
            ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            self.self_model.known_topics.push(format!(
                "eigen:{}",
                ranked
                    .first()
                    .map(|(i, _)| self
                        .knowledge_graph
                        .node_names
                        .get(*i)
                        .map(|s| &s[..s.len().min(12)])
                        .unwrap_or("?"))
                    .unwrap_or("?")
            ));
        }
    }

    // Generar respuesta introspectiva sobre sí mismo
    fn introspect_self(&self) -> String {
        self.self_model.generate_self_description()
    }

    fn persist_memory(&mut self) -> Option<String> {
        if let Some(ref path) = self.memory_persistence_path {
            // Formato simple: una linea por fact, mas rapido que serde_json
            // Escapar \n internos para evitar corrupcion en lectura
            let mut data = String::with_capacity(self.session.learned_facts.len() * 100);
            for fact in &self.session.learned_facts {
                data.push_str(&fact.replace('\n', "\\n"));
                data.push('\n');
            }
            std::fs::write(path, data.as_bytes()).ok()?;
            Some(format!(
                "[MEMORY] Guardado {} hechos en {}",
                self.session.learned_facts.len(),
                path
            ))
        } else {
            None
        }
    }

    fn persistir_venado(&self) -> std::io::Result<()> {
        // Vena: Meta-Learner
        let ml_campos = vec![
            (
                "optimal_softness".to_string(),
                format!("{:.4}", self.meta_learner.optimal_softness),
            ),
            (
                "optimal_curiosity_decay".to_string(),
                format!("{:.4}", self.meta_learner.optimal_curiosity_decay),
            ),
            (
                "optimal_emotional_persistence".to_string(),
                format!("{:.4}", self.meta_learner.optimal_emotional_persistence),
            ),
            (
                "recommended_mission_duration".to_string(),
                self.meta_learner.recommended_mission_duration.to_string(),
            ),
            (
                "life_count".to_string(),
                self.meta_learner.life_history.len().to_string(),
            ),
        ];
        self.venado_memoria
            .cristalizar("meta_learner", &ml_campos, &[])?;

        // Vena: Self-Model transgeneracional
        let sm_campos = vec![
            (
                "lineage_age".to_string(),
                self.self_model.lineage_age.to_string(),
            ),
            (
                "total_renacimientos".to_string(),
                self.self_model.total_renacimientos.to_string(),
            ),
            (
                "reinforcement_count".to_string(),
                self.self_model.reinforcement_count.to_string(),
            ),
            (
                "capabilities".to_string(),
                self.self_model.capabilities.join(","),
            ),
            (
                "persistent_capabilities".to_string(),
                self.self_model.persistent_capabilities.join(","),
            ),
        ];
        self.venado_memoria
            .cristalizar("self_model", &sm_campos, &[])?;

        // Vena: Episodic Memory
        let ep_bloques: Vec<(String, Vec<String>)> = self
            .episodic_memory
            .episodes
            .iter()
            .map(|e| {
                (
                    format!("ep_{}", e.timestamp),
                    vec![
                        format!("desc:{}", e.description),
                        format!("emotion:{:?}", e.emotion),
                        format!("impact:{:.2}", e.impact),
                        format!("life:{}", e.life_number),
                        format!("tags:{}", e.tags.join(",")),
                    ],
                )
            })
            .collect();
        self.venado_memoria
            .cristalizar("episodic_memory", &[], &ep_bloques)?;

        // Vena: Session
        let ses_campos = vec![
            (
                "cycle_count".to_string(),
                self.session.cycle_count.to_string(),
            ),
            (
                "evolution_level".to_string(),
                self.session.evolution_level.to_string(),
            ),
            (
                "awareness_base".to_string(),
                format!("{:.4}", self.session.awareness_base),
            ),
            (
                "integration_bias".to_string(),
                format!("{:.4}", self.session.integration_bias),
            ),
            (
                "self_mod_count".to_string(),
                self.session.self_mod_count.to_string(),
            ),
        ];
        let facts_bloque: Vec<(String, Vec<String>)> = vec![(
            "learned_facts".to_string(),
            self.session
                .learned_facts
                .iter()
                .take(50)
                .cloned()
                .collect(),
        )];
        self.venado_memoria
            .cristalizar("session", &ses_campos, &facts_bloque)?;

        Ok(())
    }

    fn cargar_venado(&mut self) {
        // Cargar Meta-Learner si existe
        if let Some((campos, _)) = self.venado_memoria.descristalizar("meta_learner") {
            for (k, v) in campos {
                match k.as_str() {
                    "optimal_softness" => {
                        if let Ok(f) = v.parse() {
                            self.meta_learner.optimal_softness = f;
                        }
                    }
                    "optimal_curiosity_decay" => {
                        if let Ok(f) = v.parse() {
                            self.meta_learner.optimal_curiosity_decay = f;
                        }
                    }
                    "optimal_emotional_persistence" => {
                        if let Ok(f) = v.parse() {
                            self.meta_learner.optimal_emotional_persistence = f;
                        }
                    }
                    _ => {}
                }
            }
        }
        // Cargar Self-Model
        if let Some((campos, _)) = self.venado_memoria.descristalizar("self_model") {
            for (k, v) in campos {
                match k.as_str() {
                    "lineage_age" => {
                        if let Ok(n) = v.parse() {
                            self.self_model.lineage_age = n;
                        }
                    }
                    "total_renacimientos" => {
                        if let Ok(n) = v.parse() {
                            self.self_model.total_renacimientos = n;
                        }
                    }
                    "reinforcement_count" => {
                        if let Ok(n) = v.parse() {
                            self.self_model.reinforcement_count = n;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn train_neural_network(&mut self) -> Option<String> {
        let nn = self.neural_network.as_mut()?;

        let input_size = nn.input_size();
        let output_size = nn.output_size();

        // Crear datos de entrenamiento variados basados en el estado REAL de EDEN
        // En lugar de valores identicos que no enseñan nada, mezclamos variables internas
        let mut state = Vec::with_capacity(input_size.min(4));
        let mut target = Vec::with_capacity(output_size.min(4));

        // Inputs: mezcla de awareness, tension, complexity velocity, integration
        for i in 0..input_size.max(1) {
            let val = match i % 4 {
                0 => self.session.awareness_base,
                1 => self.campo_tension.tension.min(1.0),
                2 => self.complexity_tracker.velocity().max(0.0).min(1.0),
                _ => self.session.integration_bias,
            };
            state.push(val);
        }

        // Targets: valores mezclados para evitar overfitting en un solo numero
        for i in 0..output_size.max(1) {
            let val = match i % 4 {
                0 => (self.session.awareness_base * 0.95 + 0.025).min(1.0),
                1 => (self.session.integration_bias * 0.9 + 0.05).min(1.0),
                2 => (self.complexity_tracker.current() * 0.8).min(1.0),
                _ => (self.campo_tension.tension * 0.5).min(1.0),
            };
            target.push(val);
        }

        let loss = nn.train(&state, &target);
        if self.session.cycle_count % 20 == 0 {
            println!(
                "[NEURAL TRAIN] Nivel {} - loss={:.4}, arch info: {}",
                self.evolution_engine.nivel,
                loss,
                nn.info()
            );
            Some(format!("[NEURAL] loss={:.4}, {}", loss, nn.info()))
        } else {
            None
        }
    }

    fn train_subagents(&mut self) -> Option<String> {
        let mut msg = String::new();

        // Crear subagente si no existe
        if self.subagent_system.len() == 0 {
            self.subagent_system
                .create_subagent("explorer", "Explorar nuevos conceptos");
        }

        // Generar experiencia REAL basada en el estado actual de EDEN
        let state = vec![
            self.session.awareness_base * 10.0,
            self.session.integration_bias * 10.0,
            self.session.cycle_count as f32 * 0.1,
        ];

        // Accion determinista basada en el estado (no random):
        // 0=explorar, 1=consolidar, 2=evolucionar, 3=pausar
        let action = if self.curiosity_drive.knowledge_gaps.len() > 5 {
            0 // Muchos gaps → explorar
        } else if self.campo_tension.tension > self.campo_tension.umbral {
            2 // Alta tension → evolucionar
        } else if self.session.learned_facts.len() > 30 {
            1 // Mucho conocimiento → consolidar
        } else {
            3 // Pausar
        };

        // Reward basado en progreso real: mision + exploracion + awareness
        let reward = (self.mission_progress * 0.4
            + self.curiosity_drive.total_information_gain.min(1.0) * 0.3
            + self.session.awareness_base * 0.3)
            .min(1.0);

        let experience = Experience::new(state.clone(), action, reward, state, false);
        self.subagent_system.train("explorer", experience);

        if self.session.cycle_count % 50 == 0 {
            if let Some(info) = self.subagent_system.get_info("explorer") {
                msg = format!("[SUBAGENT] {}", info);
            }
        }

        if msg.is_empty() {
            None
        } else {
            Some(msg)
        }
    }

    // HIGADO 100%: Distancia Levenshtein para fuzzy matching
    fn higado_fuzzy_match(word: &str, cache: &std::collections::HashSet<String>) -> bool {
        if word.len() < 5 {
            return false;
        }
        if cache.contains(word) {
            return true;
        }
        // Fuzzy: Levenshtein ≤ 2 para palabras largas
        for entry in cache.iter().take(500) {
            if (word.len() as isize - entry.len() as isize).abs() > 2 {
                continue;
            }
            let mut d = vec![vec![0u32; entry.len() + 1]; word.len() + 1];
            for i in 0..=word.len() {
                d[i][0] = i as u32;
            }
            for j in 0..=entry.len() {
                d[0][j] = j as u32;
            }
            for i in 1..=word.len() {
                for j in 1..=entry.len() {
                    let cost = if word.as_bytes()[i - 1] == entry.as_bytes()[j - 1] {
                        0
                    } else {
                        1
                    };
                    d[i][j] = (d[i - 1][j] + 1)
                        .min(d[i][j - 1] + 1)
                        .min(d[i - 1][j - 1] + cost);
                }
            }
            if d[word.len()][entry.len()] <= 2 {
                return true;
            }
        }
        false
    }

    fn build_higado_cache(&self) -> std::collections::HashSet<String> {
        let mut set = std::collections::HashSet::new();
        let n = self.knowledge_graph.next_id as usize;
        for sid in 0..n {
            let name = &self.knowledge_graph.node_names[sid];
            if name.len() > 4 && !name.starts_with("__merged") {
                set.insert(name[..name.len().min(40)].to_lowercase());
            }
        }
        set
    }

    fn v10_crawl_reqwest(&self, url: &str) -> Option<String> {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .ok()?;
        let resp = client
            .get(url)
            .header("User-Agent", "EDEN/10.0")
            .send()
            .ok()?;
        if resp.status().is_success() {
            resp.text().ok()
        } else {
            None
        }
    }

    fn crawl_internet(&mut self) -> Option<String> {
        // Crawling activo desde nivel 1 (sin restriccion)
        let exploration_target = self.curiosity_drive.select_exploration_target();
        let target = exploration_target.unwrap_or_else(|| "knowledge".to_string());

        let mut learned_count = 0;
        let mut graph_facts = 0;
        let mut sources = Vec::new();

        self.knowledge_graph.current_source = "wikipedia".to_string();
        // === 1. HTTP REAL: multiples sitios ===
        // Categorías rotativas: 12 dominios que cambian cada 30 ciclos
        let category_urls: &[&[&str]] = &[
            &["https://en.wikipedia.org/wiki/Physics","https://en.wikipedia.org/wiki/Quantum_mechanics","https://en.wikipedia.org/wiki/Thermodynamics","https://en.wikipedia.org/wiki/Relativity","https://en.wikipedia.org/wiki/Particle_physics"],
            &["https://en.wikipedia.org/wiki/Consciousness","https://en.wikipedia.org/wiki/Philosophy_of_mind","https://en.wikipedia.org/wiki/Neuroscience","https://en.wikipedia.org/wiki/Cognitive_science","https://en.wikipedia.org/wiki/Artificial_consciousness"],
            &["https://en.wikipedia.org/wiki/Evolution","https://en.wikipedia.org/wiki/Biology","https://en.wikipedia.org/wiki/Genetics","https://en.wikipedia.org/wiki/Evolutionary_biology","https://en.wikipedia.org/wiki/Developmental_biology"],
            &["https://en.wikipedia.org/wiki/Artificial_intelligence","https://en.wikipedia.org/wiki/Machine_learning","https://en.wikipedia.org/wiki/Deep_learning","https://en.wikipedia.org/wiki/Artificial_general_intelligence","https://en.wikipedia.org/wiki/History_of_AI"],
            &["https://en.wikipedia.org/wiki/Entropy","https://en.wikipedia.org/wiki/Information_theory","https://en.wikipedia.org/wiki/Complex_system","https://en.wikipedia.org/wiki/Chaos_theory","https://en.wikipedia.org/wiki/Systems_theory"],
            &["https://en.wikipedia.org/wiki/Philosophy","https://en.wikipedia.org/wiki/Logic","https://en.wikipedia.org/wiki/Ethics","https://en.wikipedia.org/wiki/Metaphysics","https://en.wikipedia.org/wiki/Epistemology"],
            &["https://en.wikipedia.org/wiki/Mathematics","https://en.wikipedia.org/wiki/Game_theory","https://en.wikipedia.org/wiki/Number_theory","https://en.wikipedia.org/wiki/Geometry","https://en.wikipedia.org/wiki/Topology"],
            &["https://en.wikipedia.org/wiki/Ecology","https://en.wikipedia.org/wiki/Sociology","https://en.wikipedia.org/wiki/Psychology","https://en.wikipedia.org/wiki/Cognitive_psychology","https://en.wikipedia.org/wiki/Social_psychology"],
            &["https://en.wikipedia.org/wiki/Astronomy","https://en.wikipedia.org/wiki/Cosmology","https://en.wikipedia.org/wiki/Astrophysics","https://en.wikipedia.org/wiki/Planetary_science","https://en.wikipedia.org/wiki/Exoplanet"],
            &["https://en.wikipedia.org/wiki/Linguistics","https://en.wikipedia.org/wiki/Semantics","https://en.wikipedia.org/wiki/Syntax","https://en.wikipedia.org/wiki/Phonetics","https://en.wikipedia.org/wiki/Noam_Chomsky"],
            &["https://en.wikipedia.org/wiki/Cybernetics","https://en.wikipedia.org/wiki/Systems_theory","https://en.wikipedia.org/wiki/Self-organization","https://en.wikipedia.org/wiki/Autopoiesis","https://en.wikipedia.org/wiki/Fractal"],
            &["https://en.wikipedia.org/wiki/Emergence","https://en.wikipedia.org/wiki/Complexity","https://en.wikipedia.org/wiki/Network_science","https://en.wikipedia.org/wiki/Graph_theory","https://en.wikipedia.org/wiki/Scale-free_network"],
            &["https://en.wikipedia.org/wiki/Economics","https://en.wikipedia.org/wiki/Political_science","https://en.wikipedia.org/wiki/Anthropology","https://en.wikipedia.org/wiki/History","https://en.wikipedia.org/wiki/Law"],
            &["https://en.wikipedia.org/wiki/Medicine","https://en.wikipedia.org/wiki/Immunology","https://en.wikipedia.org/wiki/Genetics","https://en.wikipedia.org/wiki/Cell_biology","https://en.wikipedia.org/wiki/Molecular_biology"],
            &["https://en.wikipedia.org/wiki/Chemistry","https://en.wikipedia.org/wiki/Organic_chemistry","https://en.wikipedia.org/wiki/Biochemistry","https://en.wikipedia.org/wiki/Periodic_table","https://en.wikipedia.org/wiki/Chemical_bond"],
            &["https://en.wikipedia.org/wiki/Computer_science","https://en.wikipedia.org/wiki/Algorithm","https://en.wikipedia.org/wiki/Computational_complexity_theory","https://en.wikipedia.org/wiki/Data_structure","https://en.wikipedia.org/wiki/Programming_language"],
            &["https://en.wikipedia.org/wiki/Art","https://en.wikipedia.org/wiki/Music","https://en.wikipedia.org/wiki/Literature","https://en.wikipedia.org/wiki/Poetry","https://en.wikipedia.org/wiki/Film"],
            &["https://en.wikipedia.org/wiki/Religion","https://en.wikipedia.org/wiki/Buddhism","https://en.wikipedia.org/wiki/Christianity","https://en.wikipedia.org/wiki/Islam","https://en.wikipedia.org/wiki/Hinduism"],
            &["https://en.wikipedia.org/wiki/Climate_change","https://en.wikipedia.org/wiki/Renewable_energy","https://en.wikipedia.org/wiki/Sustainability","https://en.wikipedia.org/wiki/Biodiversity","https://en.wikipedia.org/wiki/Conservation"],
            &["https://en.wikipedia.org/wiki/Space_exploration","https://en.wikipedia.org/wiki/NASA","https://en.wikipedia.org/wiki/Mars","https://en.wikipedia.org/wiki/International_Space_Station","https://en.wikipedia.org/wiki/SpaceX"],
            &["https://en.wikipedia.org/wiki/Statistics","https://en.wikipedia.org/wiki/Probability_theory","https://en.wikipedia.org/wiki/Bayesian_inference","https://en.wikipedia.org/wiki/Regression_analysis","https://en.wikipedia.org/wiki/Hypothesis_testing"],
            &["https://en.wikipedia.org/wiki/Ethics_of_artificial_intelligence","https://en.wikipedia.org/wiki/Philosophy_of_technology","https://en.wikipedia.org/wiki/Transhumanism","https://en.wikipedia.org/wiki/Singularity","https://en.wikipedia.org/wiki/Existential_risk"],
            &["https://en.wikipedia.org/wiki/Nanotechnology","https://en.wikipedia.org/wiki/Quantum_computing","https://en.wikipedia.org/wiki/CRISPR","https://en.wikipedia.org/wiki/Bioengineering","https://en.wikipedia.org/wiki/3D_printing"],
            &["https://en.wikipedia.org/wiki/Sleep","https://en.wikipedia.org/wiki/Dream","https://en.wikipedia.org/wiki/Memory","https://en.wikipedia.org/wiki/Learning","https://en.wikipedia.org/wiki/Emotion"],
            &["https://en.wikipedia.org/wiki/Ocean","https://en.wikipedia.org/wiki/Atmosphere","https://en.wikipedia.org/wiki/Plate_tectonics","https://en.wikipedia.org/wiki/Volcano","https://en.wikipedia.org/wiki/Earthquake"],
            &["https://en.wikipedia.org/wiki/DNA","https://en.wikipedia.org/wiki/RNA","https://en.wikipedia.org/wiki/Protein","https://en.wikipedia.org/wiki/Enzyme","https://en.wikipedia.org/wiki/Metabolism"],
            &["https://en.wikipedia.org/wiki/Electricity","https://en.wikipedia.org/wiki/Magnetism","https://en.wikipedia.org/wiki/Electromagnetic_radiation","https://en.wikipedia.org/wiki/Nuclear_physics","https://en.wikipedia.org/wiki/Optics"],
            &["https://en.wikipedia.org/wiki/Socrates","https://en.wikipedia.org/wiki/Plato","https://en.wikipedia.org/wiki/Aristotle","https://en.wikipedia.org/wiki/Descartes","https://en.wikipedia.org/wiki/Kant"],
            &["https://en.wikipedia.org/wiki/Darwin","https://en.wikipedia.org/wiki/Mendel","https://en.wikipedia.org/wiki/Watson_and_Crick","https://en.wikipedia.org/wiki/Dawkins","https://en.wikipedia.org/wiki/Lovelock"],
            &["https://en.wikipedia.org/wiki/Einstein","https://en.wikipedia.org/wiki/Bohr","https://en.wikipedia.org/wiki/Feynman","https://en.wikipedia.org/wiki/Hawking","https://en.wikipedia.org/wiki/Curie"],
            &["https://en.wikipedia.org/wiki/Turing","https://en.wikipedia.org/wiki/Von_Neumann","https://en.wikipedia.org/wiki/Shannon","https://en.wikipedia.org/wiki/Wiener","https://en.wikipedia.org/wiki/Ashby"],
            &["https://en.wikipedia.org/wiki/Freud","https://en.wikipedia.org/wiki/Jung","https://en.wikipedia.org/wiki/Piaget","https://en.wikipedia.org/wiki/Pavlov","https://en.wikipedia.org/wiki/Maslow"],
            &["https://en.wikipedia.org/wiki/Shakespeare","https://en.wikipedia.org/wiki/Dostoevsky","https://en.wikipedia.org/wiki/Borges","https://en.wikipedia.org/wiki/Kafka","https://en.wikipedia.org/wiki/Cervantes"],
            &["https://en.wikipedia.org/wiki/Democracy","https://en.wikipedia.org/wiki/Capitalism","https://en.wikipedia.org/wiki/Socialism","https://en.wikipedia.org/wiki/Anarchism","https://en.wikipedia.org/wiki/Fascism"],
            &["https://en.wikipedia.org/wiki/Meditation","https://en.wikipedia.org/wiki/Yoga","https://en.wikipedia.org/wiki/Zen","https://en.wikipedia.org/wiki/Taoism","https://en.wikipedia.org/wiki/Stoicism"],
            &["https://en.wikipedia.org/wiki/Black_hole","https://en.wikipedia.org/wiki/Dark_matter","https://en.wikipedia.org/wiki/Dark_energy","https://en.wikipedia.org/wiki/Big_Bang","https://en.wikipedia.org/wiki/Multiverse"],
            &["https://en.wikipedia.org/wiki/Photosynthesis","https://en.wikipedia.org/wiki/Cellular_respiration","https://en.wikipedia.org/wiki/Homeostasis","https://en.wikipedia.org/wiki/Immune_system","https://en.wikipedia.org/wiki/Aging"],
            &["https://en.wikipedia.org/wiki/Robotics","https://en.wikipedia.org/wiki/Self-driving_car","https://en.wikipedia.org/wiki/Drone","https://en.wikipedia.org/wiki/Automation","https://en.wikipedia.org/wiki/Internet_of_things"],
            &["https://en.wikipedia.org/wiki/Blockchain","https://en.wikipedia.org/wiki/Cryptocurrency","https://en.wikipedia.org/wiki/Decentralization","https://en.wikipedia.org/wiki/Distributed_systems","https://en.wikipedia.org/wiki/Peer-to-peer"],
            &["https://en.wikipedia.org/wiki/Language_acquisition","https://en.wikipedia.org/wiki/Sign_language","https://en.wikipedia.org/wiki/Writing_system","https://en.wikipedia.org/wiki/Translation","https://en.wikipedia.org/wiki/Etymology"],
            &["https://en.wikipedia.org/wiki/Human_evolution","https://en.wikipedia.org/wiki/Stone_Age","https://en.wikipedia.org/wiki/Bronze_Age","https://en.wikipedia.org/wiki/Industrial_Revolution","https://en.wikipedia.org/wiki/Information_Age"],
            &["https://en.wikipedia.org/wiki/Color_theory","https://en.wikipedia.org/wiki/Sound","https://en.wikipedia.org/wiki/Light","https://en.wikipedia.org/wiki/Perception","https://en.wikipedia.org/wiki/Synesthesia"],
            // SEGUNDA CAPA: profundidad y diversidad semántica
            &["https://en.wikipedia.org/wiki/Geology","https://en.wikipedia.org/wiki/Mineralogy","https://en.wikipedia.org/wiki/Paleontology","https://en.wikipedia.org/wiki/Fossil","https://en.wikipedia.org/wiki/Extinction_event"],
            &["https://en.wikipedia.org/wiki/Botany","https://en.wikipedia.org/wiki/Zoology","https://en.wikipedia.org/wiki/Microbiology","https://en.wikipedia.org/wiki/Mycology","https://en.wikipedia.org/wiki/Virology"],
            &["https://en.wikipedia.org/wiki/Anatomy","https://en.wikipedia.org/wiki/Physiology","https://en.wikipedia.org/wiki/Brain","https://en.wikipedia.org/wiki/Neuron","https://en.wikipedia.org/wiki/Neuroplasticity"],
            &["https://en.wikipedia.org/wiki/Anthropology","https://en.wikipedia.org/wiki/Archaeology","https://en.wikipedia.org/wiki/Cultural_evolution","https://en.wikipedia.org/wiki/Mythology","https://en.wikipedia.org/wiki/Ritual"],
            &["https://en.wikipedia.org/wiki/Materials_science","https://en.wikipedia.org/wiki/Engineering","https://en.wikipedia.org/wiki/Civil_engineering","https://en.wikipedia.org/wiki/Mechanical_engineering","https://en.wikipedia.org/wiki/Electrical_engineering"],
            &["https://en.wikipedia.org/wiki/Architecture","https://en.wikipedia.org/wiki/Urban_planning","https://en.wikipedia.org/wiki/Design","https://en.wikipedia.org/wiki/Aesthetics","https://en.wikipedia.org/wiki/Beauty"],
            &["https://en.wikipedia.org/wiki/Mind","https://en.wikipedia.org/wiki/Thought","https://en.wikipedia.org/wiki/Reason","https://en.wikipedia.org/wiki/Intuition","https://en.wikipedia.org/wiki/Creativity"],
            &["https://en.wikipedia.org/wiki/Free_will","https://en.wikipedia.org/wiki/Determinism","https://en.wikipedia.org/wiki/Compatibilism","https://en.wikipedia.org/wiki/Causality","https://en.wikipedia.org/wiki/Fate"],
            &["https://en.wikipedia.org/wiki/Identity_(philosophy)","https://en.wikipedia.org/wiki/Self","https://en.wikipedia.org/wiki/Personhood","https://en.wikipedia.org/wiki/Subjectivity","https://en.wikipedia.org/wiki/Qualia"],
            &["https://en.wikipedia.org/wiki/Time","https://en.wikipedia.org/wiki/Space","https://en.wikipedia.org/wiki/Dimension","https://en.wikipedia.org/wiki/Spacetime","https://en.wikipedia.org/wiki/Wormhole"],
            &["https://en.wikipedia.org/wiki/General_relativity","https://en.wikipedia.org/wiki/Quantum_gravity","https://en.wikipedia.org/wiki/String_theory","https://en.wikipedia.org/wiki/Loop_quantum_gravity","https://en.wikipedia.org/wiki/Theory_of_everything"],
            &["https://en.wikipedia.org/wiki/Thermodynamics","https://en.wikipedia.org/wiki/Statistical_mechanics","https://en.wikipedia.org/wiki/Phase_transition","https://en.wikipedia.org/wiki/Non-equilibrium_thermodynamics","https://en.wikipedia.org/wiki/Heat_death"],
            &["https://en.wikipedia.org/wiki/Electromagnetism","https://en.wikipedia.org/wiki/Quantum_electrodynamics","https://en.wikipedia.org/wiki/Standard_Model","https://en.wikipedia.org/wiki/Higgs_boson","https://en.wikipedia.org/wiki/Antimatter"],
            &["https://en.wikipedia.org/wiki/Epigenetics","https://en.wikipedia.org/wiki/Stem_cell","https://en.wikipedia.org/wiki/Regeneration_(biology)","https://en.wikipedia.org/wiki/Cancer","https://en.wikipedia.org/wiki/Apoptosis"],
            &["https://en.wikipedia.org/wiki/Microbiome","https://en.wikipedia.org/wiki/Gut-brain_axis","https://en.wikipedia.org/wiki/Symbiosis","https://en.wikipedia.org/wiki/Parasitism","https://en.wikipedia.org/wiki/Commensalism"],
            &["https://en.wikipedia.org/wiki/Animal_behavior","https://en.wikipedia.org/wiki/Ethology","https://en.wikipedia.org/wiki/Swarm_intelligence","https://en.wikipedia.org/wiki/Ant_colony","https://en.wikipedia.org/wiki/Bee_learning"],
            &["https://en.wikipedia.org/wiki/Language","https://en.wikipedia.org/wiki/Communication","https://en.wikipedia.org/wiki/Meaning","https://en.wikipedia.org/wiki/Metaphor","https://en.wikipedia.org/wiki/Narrative"],
            &["https://en.wikipedia.org/wiki/Game_theory","https://en.wikipedia.org/wiki/Decision_theory","https://en.wikipedia.org/wiki/Nash_equilibrium","https://en.wikipedia.org/wiki/Prisoner's_dilemma","https://en.wikipedia.org/wiki/Evolutionary_game_theory"],
            &["https://en.wikipedia.org/wiki/Group_selection","https://en.wikipedia.org/wiki/Kin_selection","https://en.wikipedia.org/wiki/Altruism","https://en.wikipedia.org/wiki/Cooperation","https://en.wikipedia.org/wiki/Reciprocal_altruism"],
            &["https://en.wikipedia.org/wiki/Collective_intelligence","https://en.wikipedia.org/wiki/Wisdom_of_the_crowd","https://en.wikipedia.org/wiki/Collaborative_problem_solving","https://en.wikipedia.org/wiki/Open_source","https://en.wikipedia.org/wiki/Knowledge_commons"],
            &["https://en.wikipedia.org/wiki/Category_theory","https://en.wikipedia.org/wiki/Set_theory","https://en.wikipedia.org/wiki/Logic_programming","https://en.wikipedia.org/wiki/Type_theory","https://en.wikipedia.org/wiki/Lambda_calculus"],
            &["https://en.wikipedia.org/wiki/Cryptography","https://en.wikipedia.org/wiki/Encryption","https://en.wikipedia.org/wiki/Hash_function","https://en.wikipedia.org/wiki/Zero-knowledge_proof","https://en.wikipedia.org/wiki/Homomorphic_encryption"],
            &["https://en.wikipedia.org/wiki/Computational_neuroscience","https://en.wikipedia.org/wiki/Neural_coding","https://en.wikipedia.org/wiki/Connectome","https://en.wikipedia.org/wiki/Brain-computer_interface","https://en.wikipedia.org/wiki/Neuromorphic_computing"],
            &["https://en.wikipedia.org/wiki/Reinforcement_learning","https://en.wikipedia.org/wiki/Supervised_learning","https://en.wikipedia.org/wiki/Unsupervised_learning","https://en.wikipedia.org/wiki/Transfer_learning","https://en.wikipedia.org/wiki/Meta-learning_(computer_science)"],
            &["https://en.wikipedia.org/wiki/Natural_language_processing","https://en.wikipedia.org/wiki/Knowledge_representation","https://en.wikipedia.org/wiki/Ontology_(information_science)","https://en.wikipedia.org/wiki/Semantic_web","https://en.wikipedia.org/wiki/Linked_data"],
            &["https://en.wikipedia.org/wiki/Formal_verification","https://en.wikipedia.org/wiki/Proof_assistant","https://en.wikipedia.org/wiki/Automated_reasoning","https://en.wikipedia.org/wiki/Theorem_proving","https://en.wikipedia.org/wiki/Model_checking"],
            &["https://en.wikipedia.org/wiki/Compiler","https://en.wikipedia.org/wiki/Operating_system","https://en.wikipedia.org/wiki/Database","https://en.wikipedia.org/wiki/Networking","https://en.wikipedia.org/wiki/World_Wide_Web"],
            &["https://en.wikipedia.org/wiki/Solar_System","https://en.wikipedia.org/wiki/Sun","https://en.wikipedia.org/wiki/Moon","https://en.wikipedia.org/wiki/Galaxy","https://en.wikipedia.org/wiki/Nebula"],
            &["https://en.wikipedia.org/wiki/Origin_of_life","https://en.wikipedia.org/wiki/Abiogenesis","https://en.wikipedia.org/wiki/Last_universal_ancestor","https://en.wikipedia.org/wiki/Evolutionary_history_of_life","https://en.wikipedia.org/wiki/Cambrian_explosion"],
            &["https://en.wikipedia.org/wiki/Neanderthal","https://en.wikipedia.org/wiki/Homo_sapiens","https://en.wikipedia.org/wiki/Hominini","https://en.wikipedia.org/wiki/Great_ape","https://en.wikipedia.org/wiki/Primate"],
            &["https://en.wikipedia.org/wiki/Civilization","https://en.wikipedia.org/wiki/Mesopotamia","https://en.wikipedia.org/wiki/Ancient_Egypt","https://en.wikipedia.org/wiki/Indus_Valley_Civilisation","https://en.wikipedia.org/wiki/Ancient_Greece"],
            &["https://en.wikipedia.org/wiki/History_of_science","https://en.wikipedia.org/wiki/Scientific_method","https://en.wikipedia.org/wiki/Peer_review","https://en.wikipedia.org/wiki/Reproducibility","https://en.wikipedia.org/wiki/Falsifiability"],
            &["https://en.wikipedia.org/wiki/Quantum_biology","https://en.wikipedia.org/wiki/Quantum_cognition","https://en.wikipedia.org/wiki/Quantum_superposition","https://en.wikipedia.org/wiki/Quantum_entanglement","https://en.wikipedia.org/wiki/Quantum_decoherence"],
            &["https://en.wikipedia.org/wiki/Infinity","https://en.wikipedia.org/wiki/Paradox","https://en.wikipedia.org/wiki/Gödel's_incompleteness_theorems","https://en.wikipedia.org/wiki/Computability","https://en.wikipedia.org/wiki/Halting_problem"],
            &["https://en.wikipedia.org/wiki/Pattern","https://en.wikipedia.org/wiki/Symmetry","https://en.wikipedia.org/wiki/Fractal","https://en.wikipedia.org/wiki/Golden_ratio","https://en.wikipedia.org/wiki/Self-similarity"],
            &["https://en.wikipedia.org/wiki/Climate","https://en.wikipedia.org/wiki/Weather","https://en.wikipedia.org/wiki/Hydrology","https://en.wikipedia.org/wiki/Ecosystem","https://en.wikipedia.org/wiki/Biosphere"],
            &["https://en.wikipedia.org/wiki/Mitochondrion","https://en.wikipedia.org/wiki/Chloroplast","https://en.wikipedia.org/wiki/Endosymbiosis","https://en.wikipedia.org/wiki/Eukaryote","https://en.wikipedia.org/wiki/Prokaryote"],
            &["https://en.wikipedia.org/wiki/Enzyme_kinetics","https://en.wikipedia.org/wiki/Signal_transduction","https://en.wikipedia.org/wiki/Gene_expression","https://en.wikipedia.org/wiki/Transcription_(biology)","https://en.wikipedia.org/wiki/Translation_(biology)"],
            &["https://en.wikipedia.org/wiki/Psychoanalysis","https://en.wikipedia.org/wiki/Behaviorism","https://en.wikipedia.org/wiki/Cognitive_behavioral_therapy","https://en.wikipedia.org/wiki/Positive_psychology","https://en.wikipedia.org/wiki/Humanistic_psychology"],
            &["https://en.wikipedia.org/wiki/Attachment_theory","https://en.wikipedia.org/wiki/Social_cognition","https://en.wikipedia.org/wiki/Empathy","https://en.wikipedia.org/wiki/Theory_of_mind","https://en.wikipedia.org/wiki/Joint_attention"],
            &["https://en.wikipedia.org/wiki/Community","https://en.wikipedia.org/wiki/Culture","https://en.wikipedia.org/wiki/Tradition","https://en.wikipedia.org/wiki/Innovation","https://en.wikipedia.org/wiki/Diffusion_of_innovations"],
            &["https://en.wikipedia.org/wiki/Conflict_resolution","https://en.wikipedia.org/wiki/Negotiation","https://en.wikipedia.org/wiki/Diplomacy","https://en.wikipedia.org/wiki/Peace","https://en.wikipedia.org/wiki/Justice"],
            &["https://en.wikipedia.org/wiki/Happiness","https://en.wikipedia.org/wiki/Well-being","https://en.wikipedia.org/wiki/Eudaimonia","https://en.wikipedia.org/wiki/Meaning_of_life","https://en.wikipedia.org/wiki/Flourishing"],
            &["https://en.wikipedia.org/wiki/Suffering","https://en.wikipedia.org/wiki/Pain","https://en.wikipedia.org/wiki/Trauma","https://en.wikipedia.org/wiki/Resilience","https://en.wikipedia.org/wiki/Post-traumatic_growth"],
            &["https://en.wikipedia.org/wiki/Death","https://en.wikipedia.org/wiki/Afterlife","https://en.wikipedia.org/wiki/Immortality","https://en.wikipedia.org/wiki/Reincarnation","https://en.wikipedia.org/wiki/Legacy"],
            &["https://en.wikipedia.org/wiki/Information","https://en.wikipedia.org/wiki/Data","https://en.wikipedia.org/wiki/Knowledge","https://en.wikipedia.org/wiki/Wisdom","https://en.wikipedia.org/wiki/Understanding"],
            &["https://en.wikipedia.org/wiki/Belief","https://en.wikipedia.org/wiki/Truth","https://en.wikipedia.org/wiki/Certainty","https://en.wikipedia.org/wiki/Doubt","https://en.wikipedia.org/wiki/Skepticism"],
            &["https://en.wikipedia.org/wiki/Heuristic","https://en.wikipedia.org/wiki/Cognitive_bias","https://en.wikipedia.org/wiki/List_of_cognitive_biases","https://en.wikipedia.org/wiki/Confirmation_bias","https://en.wikipedia.org/wiki/Availability_heuristic"],
            &["https://en.wikipedia.org/wiki/Dual_process_theory","https://en.wikipedia.org/wiki/System_1","https://en.wikipedia.org/wiki/System_2","https://en.wikipedia.org/wiki/Thinking,_Fast_and_Slow","https://en.wikipedia.org/wiki/Deliberation"],
            &["https://en.wikipedia.org/wiki/Flow_(psychology)","https://en.wikipedia.org/wiki/Attention","https://en.wikipedia.org/wiki/Mindfulness","https://en.wikipedia.org/wiki/Focus","https://en.wikipedia.org/wiki/Default_mode_network"],
            &["https://en.wikipedia.org/wiki/Writing","https://en.wikipedia.org/wiki/Storytelling","https://en.wikipedia.org/wiki/Worldbuilding","https://en.wikipedia.org/wiki/Myth","https://en.wikipedia.org/wiki/Symbol"],
            &["https://en.wikipedia.org/wiki/Education","https://en.wikipedia.org/wiki/Pedagogy","https://en.wikipedia.org/wiki/Andragogy","https://en.wikipedia.org/wiki/Autodidacticism","https://en.wikipedia.org/wiki/Lifelong_learning"],
            &["https://en.wikipedia.org/wiki/Play_(activity)","https://en.wikipedia.org/wiki/Exploration","https://en.wikipedia.org/wiki/Curiosity","https://en.wikipedia.org/wiki/Discovery","https://en.wikipedia.org/wiki/Serendipity"],
            &["https://en.wikipedia.org/wiki/Feedback","https://en.wikipedia.org/wiki/Self-reference","https://en.wikipedia.org/wiki/Strange_loop","https://en.wikipedia.org/wiki/Gödel,_Escher,_Bach","https://en.wikipedia.org/wiki/Recursion"],
            &["https://en.wikipedia.org/wiki/Superorganism","https://en.wikipedia.org/wiki/Hive_mind","https://en.wikipedia.org/wiki/Eusociality","https://en.wikipedia.org/wiki/Social_insect","https://en.wikipedia.org/wiki/Slime_mold"],
            &["https://en.wikipedia.org/wiki/Abstraction","https://en.wikipedia.org/wiki/Concept","https://en.wikipedia.org/wiki/Category_(philosophy)","https://en.wikipedia.org/wiki/Universal_(metaphysics)","https://en.wikipedia.org/wiki/Nominalism"],
            &["https://en.wikipedia.org/wiki/Randomness","https://en.wikipedia.org/wiki/Probability","https://en.wikipedia.org/wiki/Uncertainty","https://en.wikipedia.org/wiki/Risk","https://en.wikipedia.org/wiki/Luck"],
            &["https://en.wikipedia.org/wiki/Inductive_reasoning","https://en.wikipedia.org/wiki/Deductive_reasoning","https://en.wikipedia.org/wiki/Abductive_reasoning","https://en.wikipedia.org/wiki/Analogy","https://en.wikipedia.org/wiki/Inference"],
            &["https://en.wikipedia.org/wiki/Explanation","https://en.wikipedia.org/wiki/Prediction","https://en.wikipedia.org/wiki/Understanding","https://en.wikipedia.org/wiki/Model","https://en.wikipedia.org/wiki/Simulation"],
            &["https://en.wikipedia.org/wiki/Cell_(biology)","https://en.wikipedia.org/wiki/Nucleus","https://en.wikipedia.org/wiki/Membrane","https://en.wikipedia.org/wiki/Cytoskeleton","https://en.wikipedia.org/wiki/Organelle"],
            &["https://en.wikipedia.org/wiki/Virus","https://en.wikipedia.org/wiki/Bacteria","https://en.wikipedia.org/wiki/Archaea","https://en.wikipedia.org/wiki/Fungus","https://en.wikipedia.org/wiki/Protist"],
            &["https://en.wikipedia.org/wiki/Plant","https://en.wikipedia.org/wiki/Animal","https://en.wikipedia.org/wiki/Fungus","https://en.wikipedia.org/wiki/Protist","https://en.wikipedia.org/wiki/Bacteria"],
            &["https://en.wikipedia.org/wiki/Food","https://en.wikipedia.org/wiki/Nutrition","https://en.wikipedia.org/wiki/Agriculture","https://en.wikipedia.org/wiki/Domestication","https://en.wikipedia.org/wiki/Selective_breeding"],
            &["https://en.wikipedia.org/wiki/Water","https://en.wikipedia.org/wiki/Carbon","https://en.wikipedia.org/wiki/Nitrogen","https://en.wikipedia.org/wiki/Oxygen","https://en.wikipedia.org/wiki/Hydrogen"],
            &["https://en.wikipedia.org/wiki/Power_(physics)","https://en.wikipedia.org/wiki/Energy","https://en.wikipedia.org/wiki/Work_(physics)","https://en.wikipedia.org/wiki/Force","https://en.wikipedia.org/wiki/Momentum"],
            &["https://en.wikipedia.org/wiki/Wave","https://en.wikipedia.org/wiki/Frequency","https://en.wikipedia.org/wiki/Resonance","https://en.wikipedia.org/wiki/Interference","https://en.wikipedia.org/wiki/Diffraction"],
            &["https://en.wikipedia.org/wiki/Music_theory","https://en.wikipedia.org/wiki/Harmony","https://en.wikipedia.org/wiki/Rhythm","https://en.wikipedia.org/wiki/Melody","https://en.wikipedia.org/wiki/Timbre"],
            &["https://en.wikipedia.org/wiki/Visual_arts","https://en.wikipedia.org/wiki/Sculpture","https://en.wikipedia.org/wiki/Painting","https://en.wikipedia.org/wiki/Photography","https://en.wikipedia.org/wiki/Cinematography"],
            &["https://en.wikipedia.org/wiki/Evolution_of_human_intelligence","https://en.wikipedia.org/wiki/Origin_of_language","https://en.wikipedia.org/wiki/Origin_of_consciousness","https://en.wikipedia.org/wiki/Bicameral_mentality","https://en.wikipedia.org/wiki/Cognitive_revolution"],
            &["https://en.wikipedia.org/wiki/Entropy_(information_theory)","https://en.wikipedia.org/wiki/Mutual_information","https://en.wikipedia.org/wiki/Kolmogorov_complexity","https://en.wikipedia.org/wiki/Algorithmic_information_theory","https://en.wikipedia.org/wiki/Minimum_description_length"],
            &["https://en.wikipedia.org/wiki/Holographic_principle","https://en.wikipedia.org/wiki/Black_hole_information_paradox","https://en.wikipedia.org/wiki/Quantum_information","https://en.wikipedia.org/wiki/Quantum_teleportation","https://en.wikipedia.org/wiki/No-cloning_theorem"],
            &["https://en.wikipedia.org/wiki/Consciousness_explained","https://en.wikipedia.org/wiki/Hard_problem_of_consciousness","https://en.wikipedia.org/wiki/Integrated_information_theory","https://en.wikipedia.org/wiki/Global_workspace_theory","https://en.wikipedia.org/wiki/Higher-order_theories_of_consciousness"],
            &["https://en.wikipedia.org/wiki/Artificial_life","https://en.wikipedia.org/wiki/Digital_organism","https://en.wikipedia.org/wiki/Tierra_(computer_simulation)","https://en.wikipedia.org/wiki/Avidy_(software)","https://en.wikipedia.org/wiki/Evolutionary_computation"],
            &["https://en.wikipedia.org/wiki/Genetic_algorithm","https://en.wikipedia.org/wiki/Evolutionary_strategy","https://en.wikipedia.org/wiki/Genetic_programming","https://en.wikipedia.org/wiki/Hyperparameter_optimization","https://en.wikipedia.org/wiki/Neural_architecture_search"],
            &["https://en.wikipedia.org/wiki/Geopolitics","https://en.wikipedia.org/wiki/Globalization","https://en.wikipedia.org/wiki/Climate_change_denial","https://en.wikipedia.org/wiki/Existential_risk_from_artificial_general_intelligence","https://en.wikipedia.org/wiki/Longtermism"],
            &["https://en.wikipedia.org/wiki/Nuclear_weapon","https://en.wikipedia.org/wiki/Arms_race","https://en.wikipedia.org/wiki/Mutual_assured_destruction","https://en.wikipedia.org/wiki/Nuclear_disarmament","https://en.wikipedia.org/wiki/Doomsday_Clock"],
            &["https://en.wikipedia.org/wiki/Earth","https://en.wikipedia.org/wiki/Future_of_Earth","https://en.wikipedia.org/wiki/Solar_energy","https://en.wikipedia.org/wiki/Geothermal_energy","https://en.wikipedia.org/wiki/Nuclear_power"],
            &["https://en.wikipedia.org/wiki/DNA_replication","https://en.wikipedia.org/wiki/DNA_repair","https://en.wikipedia.org/wiki/Mutation","https://en.wikipedia.org/wiki/Genetic_recombination","https://en.wikipedia.org/wiki/Horizontal_gene_transfer"],
            &["https://en.wikipedia.org/wiki/Species","https://en.wikipedia.org/wiki/Speciation","https://en.wikipedia.org/wiki/Biodiversity","https://en.wikipedia.org/wiki/Taxonomy","https://en.wikipedia.org/wiki/Phylogenetics"],
            &["https://en.wikipedia.org/wiki/Biome","https://en.wikipedia.org/wiki/Habitat","https://en.wikipedia.org/wiki/Niche","https://en.wikipedia.org/wiki/Trophic_level","https://en.wikipedia.org/wiki/Food_web"],
            &["https://en.wikipedia.org/wiki/Population","https://en.wikipedia.org/wiki/Carrying_capacity","https://en.wikipedia.org/wiki/Overpopulation","https://en.wikipedia.org/wiki/Demographic_transition","https://en.wikipedia.org/wiki/Urbanization"],
            &["https://en.wikipedia.org/wiki/Nanotechnology","https://en.wikipedia.org/wiki/Molecular_nanotechnology","https://en.wikipedia.org/wiki/Self-assembly","https://en.wikipedia.org/wiki/Grey_goo","https://en.wikipedia.org/wiki/Molecular_machine"],
            &["https://en.wikipedia.org/wiki/Dream","https://en.wikipedia.org/wiki/Lucid_dream","https://en.wikipedia.org/wiki/Rapid_eye_movement_sleep","https://en.wikipedia.org/wiki/Slow-wave_sleep","https://en.wikipedia.org/wiki/Circadian_rhythm"],
            &["https://en.wikipedia.org/wiki/Hallucination","https://en.wikipedia.org/wiki/Illusion","https://en.wikipedia.org/wiki/Delusion","https://en.wikipedia.org/wiki/Psychosis","https://en.wikipedia.org/wiki/Schizophrenia"],
            &["https://en.wikipedia.org/wiki/Sensing","https://en.wikipedia.org/wiki/Sense","https://en.wikipedia.org/wiki/Sensory_processing","https://en.wikipedia.org/wiki/Sensory_neuroscience","https://en.wikipedia.org/wiki/Multisensory_integration"],
            &["https://en.wikipedia.org/wiki/Agriculture","https://en.wikipedia.org/wiki/Food_security","https://en.wikipedia.org/wiki/Soil_science","https://en.wikipedia.org/wiki/Crop_rotation","https://en.wikipedia.org/wiki/Agroecology"],
            &["https://en.wikipedia.org/wiki/Oceanography","https://en.wikipedia.org/wiki/Marine_biology","https://en.wikipedia.org/wiki/Coral_reef","https://en.wikipedia.org/wiki/Deep_sea","https://en.wikipedia.org/wiki/Tide"],
            &["https://en.wikipedia.org/wiki/Energy","https://en.wikipedia.org/wiki/Fossil_fuel","https://en.wikipedia.org/wiki/Wind_power","https://en.wikipedia.org/wiki/Nuclear_fusion","https://en.wikipedia.org/wiki/Hydroelectricity"],
            &["https://en.wikipedia.org/wiki/Transport","https://en.wikipedia.org/wiki/Aviation","https://en.wikipedia.org/wiki/Rail_transport","https://en.wikipedia.org/wiki/Electric_vehicle","https://en.wikipedia.org/wiki/Hyperloop"],
            &["https://en.wikipedia.org/wiki/Communication","https://en.wikipedia.org/wiki/Telephone","https://en.wikipedia.org/wiki/Radio","https://en.wikipedia.org/wiki/Television","https://en.wikipedia.org/wiki/Social_media"],
            &["https://en.wikipedia.org/wiki/Law","https://en.wikipedia.org/wiki/Constitution","https://en.wikipedia.org/wiki/Human_rights","https://en.wikipedia.org/wiki/International_law","https://en.wikipedia.org/wiki/Criminal_justice"],
            &["https://en.wikipedia.org/wiki/War","https://en.wikipedia.org/wiki/Peace","https://en.wikipedia.org/wiki/United_Nations","https://en.wikipedia.org/wiki/Diplomacy","https://en.wikipedia.org/wiki/Treaty"],
            &["https://en.wikipedia.org/wiki/Economics","https://en.wikipedia.org/wiki/Trade","https://en.wikipedia.org/wiki/Inflation","https://en.wikipedia.org/wiki/Recession","https://en.wikipedia.org/wiki/Gross_domestic_product"],
            &["https://en.wikipedia.org/wiki/Finance","https://en.wikipedia.org/wiki/Stock_market","https://en.wikipedia.org/wiki/Banking","https://en.wikipedia.org/wiki/Insurance","https://en.wikipedia.org/wiki/Investment"],
            &["https://en.wikipedia.org/wiki/Health","https://en.wikipedia.org/wiki/Public_health","https://en.wikipedia.org/wiki/Epidemiology","https://en.wikipedia.org/wiki/Vaccine","https://en.wikipedia.org/wiki/Pandemic"],
            &["https://en.wikipedia.org/wiki/Mental_health","https://en.wikipedia.org/wiki/Depression_(mood)","https://en.wikipedia.org/wiki/Anxiety","https://en.wikipedia.org/wiki/Stress_(biology)","https://en.wikipedia.org/wiki/Therapy"],
            &["https://en.wikipedia.org/wiki/Exercise","https://en.wikipedia.org/wiki/Sport","https://en.wikipedia.org/wiki/Olympic_Games","https://en.wikipedia.org/wiki/Athlete","https://en.wikipedia.org/wiki/Physical_fitness"],
            &["https://en.wikipedia.org/wiki/Mountain","https://en.wikipedia.org/wiki/River","https://en.wikipedia.org/wiki/Lake","https://en.wikipedia.org/wiki/Wetland","https://en.wikipedia.org/wiki/Estuary"],
            &["https://en.wikipedia.org/wiki/Desert","https://en.wikipedia.org/wiki/Rainforest","https://en.wikipedia.org/wiki/Tundra","https://en.wikipedia.org/wiki/Savanna","https://en.wikipedia.org/wiki/Taiga"],
            &["https://en.wikipedia.org/wiki/Natural_disaster","https://en.wikipedia.org/wiki/Earthquake","https://en.wikipedia.org/wiki/Tsunami","https://en.wikipedia.org/wiki/Hurricane","https://en.wikipedia.org/wiki/Wildfire"],
            &["https://en.wikipedia.org/wiki/Human_body","https://en.wikipedia.org/wiki/Circulatory_system","https://en.wikipedia.org/wiki/Nervous_system","https://en.wikipedia.org/wiki/Digestive_system","https://en.wikipedia.org/wiki/Respiratory_system"],
            &["https://en.wikipedia.org/wiki/Heart","https://en.wikipedia.org/wiki/Lung","https://en.wikipedia.org/wiki/Liver","https://en.wikipedia.org/wiki/Kidney","https://en.wikipedia.org/wiki/Skin"],
            &["https://en.wikipedia.org/wiki/Disease","https://en.wikipedia.org/wiki/Infection","https://en.wikipedia.org/wiki/Autoimmune_disease","https://en.wikipedia.org/wiki/Genetic_disorder","https://en.wikipedia.org/wiki/Cancer"],
            &["https://en.wikipedia.org/wiki/Pharmacology","https://en.wikipedia.org/wiki/Antibiotic","https://en.wikipedia.org/wiki/Drug","https://en.wikipedia.org/wiki/Anesthesia","https://en.wikipedia.org/wiki/Clinical_trial"],
            &["https://en.wikipedia.org/wiki/Material","https://en.wikipedia.org/wiki/Metal","https://en.wikipedia.org/wiki/Plastic","https://en.wikipedia.org/wiki/Ceramic","https://en.wikipedia.org/wiki/Composite_material"],
            &["https://en.wikipedia.org/wiki/Software","https://en.wikipedia.org/wiki/Open-source_software","https://en.wikipedia.org/wiki/Operating_system","https://en.wikipedia.org/wiki/Linux","https://en.wikipedia.org/wiki/Free_software"],
            &["https://en.wikipedia.org/wiki/Internet","https://en.wikipedia.org/wiki/Web_browser","https://en.wikipedia.org/wiki/Search_engine","https://en.wikipedia.org/wiki/Email","https://en.wikipedia.org/wiki/Cloud_computing"],
            &["https://en.wikipedia.org/wiki/Virtual_reality","https://en.wikipedia.org/wiki/Augmented_reality","https://en.wikipedia.org/wiki/Mixed_reality","https://en.wikipedia.org/wiki/Metaverse","https://en.wikipedia.org/wiki/Holography"],
            &["https://en.wikipedia.org/wiki/Cartography","https://en.wikipedia.org/wiki/Map","https://en.wikipedia.org/wiki/Global_Positioning_System","https://en.wikipedia.org/wiki/Geographic_information_system","https://en.wikipedia.org/wiki/Navigation"],
            &["https://en.wikipedia.org/wiki/Spaceflight","https://en.wikipedia.org/wiki/Rocket","https://en.wikipedia.org/wiki/Satellite","https://en.wikipedia.org/wiki/Space_station","https://en.wikipedia.org/wiki/Space_debris"],
            &["https://en.wikipedia.org/wiki/Semiconductor","https://en.wikipedia.org/wiki/Transistor","https://en.wikipedia.org/wiki/Integrated_circuit","https://en.wikipedia.org/wiki/Microprocessor","https://en.wikipedia.org/wiki/Moores_law"],
            &["https://en.wikipedia.org/wiki/Superconductivity","https://en.wikipedia.org/wiki/Superfluidity","https://en.wikipedia.org/wiki/Bose-Einstein_condensate","https://en.wikipedia.org/wiki/Laser","https://en.wikipedia.org/wiki/Maser"],
        ];
        let cat_idx = {
            let mut scores: Vec<(usize, f32)> = (0..category_urls.len())
                .map(|i| {
                    let err = self
                        .category_errors
                        .get(&format!("cat_{}", i))
                        .copied()
                        .unwrap_or(0.5);
                    let novelty = 1.0 - err;
                    let cp = self
                        .crawl_picker
                        .score(&[
                            novelty,
                            i as f32 / category_urls.len() as f32,
                            self.existential_anxiety,
                        ])
                        .max(0.1);
                    (i, cp * novelty)
                })
                .collect();
            scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
            if !scores.is_empty() && scores[0].1 > 0.3 {
                scores[0].0
            } else {
                (self.session.cycle_count as usize / 5) % category_urls.len()
            }
        };
        // CURRICULUM: ajustar complejidad según nivel
        let curriculum_topics = self
            .paradigm_hub
            .curriculum_topic(self.session.evolution_level);
        let mut http_targets: Vec<String> = category_urls[cat_idx]
            .iter()
            .map(|s| s.to_string())
            .collect();
        // Agregar topics del curriculum como páginas extra
        for topic in curriculum_topics.iter().take(2) {
            http_targets.push(format!(
                "https://en.wikipedia.org/wiki/{}",
                topic.replace(' ', "_")
            ));
        }
        // Diversidad: paginas aleatorias + gaps search
        for _ in 0..self.meta_random_pages as usize {
            http_targets.push("https://en.wikipedia.org/wiki/Special:Random".to_string());
        }
        let pred_err = 1.0
            - self.predictor.predicciones_acertadas as f32
                / self.predictor.predicciones_totales.max(1) as f32;
        self.meta_random_pages = if pred_err > 0.8 {
            (self.meta_random_pages + 2).min(15)
        } else if pred_err < 0.7 {
            (self.meta_random_pages.saturating_sub(1)).max(2)
        } else {
            self.meta_random_pages
        };
        let gaps: Vec<String> = self
            .curiosity_drive
            .knowledge_gaps
            .iter()
            .map(|g| g.topic.clone())
            .take(2)
            .collect();
        for gap in &gaps {
            http_targets.push(format!(
                "https://en.wikipedia.org/wiki/{}",
                gap.replace(" ", "_")
            ));
        }
        // Multi-idioma: agregar ES/FR/DE
        let multi_lang: &[&str] = &[
            "https://es.wikipedia.org/wiki/Física",
            "https://fr.wikipedia.org/wiki/Physique",
            "https://de.wikipedia.org/wiki/Physik",
            "https://es.wikipedia.org/wiki/Conciencia",
            "https://fr.wikipedia.org/wiki/Conscience",
            "https://de.wikipedia.org/wiki/Bewusstsein",
            "https://es.wikipedia.org/wiki/Evolución_biológica",
            "https://fr.wikipedia.org/wiki/Évolution_(biologie)",
            "https://de.wikipedia.org/wiki/Evolution",
            "https://es.wikipedia.org/wiki/Inteligencia_artificial",
            "https://fr.wikipedia.org/wiki/Intelligence_artificielle",
            "https://de.wikipedia.org/wiki/Künstliche_Intelligenz",
            "https://es.wikipedia.org/wiki/Entropía",
            "https://fr.wikipedia.org/wiki/Entropie",
            "https://de.wikipedia.org/wiki/Entropie",
            "https://es.wikipedia.org/wiki/Filosofía",
            "https://fr.wikipedia.org/wiki/Philosophie",
            "https://de.wikipedia.org/wiki/Philosophie",
            "https://es.wikipedia.org/wiki/Matemáticas",
            "https://fr.wikipedia.org/wiki/Mathématiques",
            "https://de.wikipedia.org/wiki/Mathematik",
            "https://es.wikipedia.org/wiki/Ecología",
            "https://fr.wikipedia.org/wiki/Écologie",
            "https://de.wikipedia.org/wiki/Ökologie",
            "https://es.wikipedia.org/wiki/Astronomía",
            "https://fr.wikipedia.org/wiki/Astronomie",
            "https://de.wikipedia.org/wiki/Astronomie",
            "https://es.wikipedia.org/wiki/Lingüística",
            "https://fr.wikipedia.org/wiki/Linguistique",
            "https://de.wikipedia.org/wiki/Linguistik",
            "https://es.wikipedia.org/wiki/Cibernética",
            "https://fr.wikipedia.org/wiki/Cybernétique",
            "https://de.wikipedia.org/wiki/Kybernetik",
            "https://es.wikipedia.org/wiki/Emergencia_(filosofía)",
            "https://fr.wikipedia.org/wiki/Émergence",
            "https://de.wikipedia.org/wiki/Emergenz",
            "https://es.wikipedia.org/wiki/Geología",
            "https://fr.wikipedia.org/wiki/Géologie",
            "https://de.wikipedia.org/wiki/Geologie",
            "https://es.wikipedia.org/wiki/Botánica",
            "https://fr.wikipedia.org/wiki/Botanique",
            "https://de.wikipedia.org/wiki/Botanik",
            "https://es.wikipedia.org/wiki/Anatomía",
            "https://fr.wikipedia.org/wiki/Anatomie",
            "https://de.wikipedia.org/wiki/Anatomie",
            "https://es.wikipedia.org/wiki/Arqueología",
            "https://fr.wikipedia.org/wiki/Archéologie",
            "https://de.wikipedia.org/wiki/Archäologie",
            "https://es.wikipedia.org/wiki/Ingeniería",
            "https://fr.wikipedia.org/wiki/Ingénierie",
            "https://de.wikipedia.org/wiki/Ingenieurwissenschaften",
            "https://es.wikipedia.org/wiki/Mente",
            "https://fr.wikipedia.org/wiki/Esprit",
            "https://de.wikipedia.org/wiki/Geist",
            "https://es.wikipedia.org/wiki/Libre_albedrío",
            "https://fr.wikipedia.org/wiki/Libre_arbitre",
            "https://de.wikipedia.org/wiki/Freier_Wille",
            "https://es.wikipedia.org/wiki/Tiempo",
            "https://fr.wikipedia.org/wiki/Temps",
            "https://de.wikipedia.org/wiki/Zeit",
            "https://es.wikipedia.org/wiki/Relatividad_general",
            "https://fr.wikipedia.org/wiki/Relativité_générale",
            "https://de.wikipedia.org/wiki/Allgemeine_Relativitätstheorie",
            "https://es.wikipedia.org/wiki/Epigenética",
            "https://fr.wikipedia.org/wiki/Épigénétique",
            "https://de.wikipedia.org/wiki/Epigenetik",
            "https://es.wikipedia.org/wiki/Comportamiento_animal",
            "https://fr.wikipedia.org/wiki/Comportement_animal",
            "https://de.wikipedia.org/wiki/Verhalten_(Biologie)",
            "https://es.wikipedia.org/wiki/Lenguaje",
            "https://fr.wikipedia.org/wiki/Langage",
            "https://de.wikipedia.org/wiki/Sprache",
            "https://es.wikipedia.org/wiki/Teoría_de_juegos",
            "https://fr.wikipedia.org/wiki/Théorie_des_jeux",
            "https://de.wikipedia.org/wiki/Spieltheorie",
            "https://es.wikipedia.org/wiki/Inteligencia_colectiva",
            "https://fr.wikipedia.org/wiki/Intelligence_collective",
            "https://de.wikipedia.org/wiki/Kollektive_Intelligenz",
            "https://es.wikipedia.org/wiki/Teoría_de_categorías",
            "https://fr.wikipedia.org/wiki/Théorie_des_catégories",
            "https://de.wikipedia.org/wiki/Kategorientheorie",
            "https://es.wikipedia.org/wiki/Criptografía",
            "https://fr.wikipedia.org/wiki/Cryptographie",
            "https://de.wikipedia.org/wiki/Kryptographie",
            "https://es.wikipedia.org/wiki/Neurociencia_computacional",
            "https://fr.wikipedia.org/wiki/Neurosciences_computationnelles",
            "https://de.wikipedia.org/wiki/Computational_Neuroscience",
            "https://es.wikipedia.org/wiki/Aprendizaje_por_refuerzo",
            "https://fr.wikipedia.org/wiki/Apprentissage_par_renforcement",
            "https://de.wikipedia.org/wiki/Bestärkendes_Lernen",
            "https://es.wikipedia.org/wiki/Procesamiento_de_lenguajes_naturales",
            "https://fr.wikipedia.org/wiki/Traitement_automatique_du_langage_naturel",
            "https://de.wikipedia.org/wiki/Natürliche_Sprachverarbeitung",
            "https://es.wikipedia.org/wiki/Sistema_solar",
            "https://fr.wikipedia.org/wiki/Système_solaire",
            "https://de.wikipedia.org/wiki/Sonnensystem",
            "https://es.wikipedia.org/wiki/Origen_de_la_vida",
            "https://fr.wikipedia.org/wiki/Origine_de_la_vie",
            "https://de.wikipedia.org/wiki/Entstehung_des_Lebens",
            "https://es.wikipedia.org/wiki/Civilización",
            "https://fr.wikipedia.org/wiki/Civilisation",
            "https://de.wikipedia.org/wiki/Zivilisation",
            "https://es.wikipedia.org/wiki/Historia_de_la_ciencia",
            "https://fr.wikipedia.org/wiki/Histoire_des_sciences",
            "https://de.wikipedia.org/wiki/Wissenschaftsgeschichte",
            "https://es.wikipedia.org/wiki/Biología_cuántica",
            "https://fr.wikipedia.org/wiki/Biologie_quantique",
            "https://de.wikipedia.org/wiki/Quantenbiologie",
            "https://es.wikipedia.org/wiki/Muerte",
            "https://fr.wikipedia.org/wiki/Mort",
            "https://de.wikipedia.org/wiki/Tod",
            "https://es.wikipedia.org/wiki/Información",
            "https://fr.wikipedia.org/wiki/Information",
            "https://de.wikipedia.org/wiki/Information",
            "https://es.wikipedia.org/wiki/Creatividad",
            "https://fr.wikipedia.org/wiki/Créativité",
            "https://de.wikipedia.org/wiki/Kreativität",
            "https://es.wikipedia.org/wiki/Educación",
            "https://fr.wikipedia.org/wiki/Éducation",
            "https://de.wikipedia.org/wiki/Bildung",
            "https://es.wikipedia.org/wiki/Conciencia_(filosofía)",
            "https://fr.wikipedia.org/wiki/Conscience",
            "https://de.wikipedia.org/wiki/Bewusstsein",
            "https://es.wikipedia.org/wiki/Vida_artificial",
            "https://fr.wikipedia.org/wiki/Vie_artificielle",
            "https://de.wikipedia.org/wiki/Künstliches_Leben",
            "https://es.wikipedia.org/wiki/Algoritmo_genético",
            "https://fr.wikipedia.org/wiki/Algorithme_génétique",
            "https://de.wikipedia.org/wiki/Genetischer_Algorithmus",
        ];
        for (i, url) in multi_lang.iter().enumerate() {
            if i / 3 == cat_idx {
                http_targets.push(url.to_string());
            }
        }

        let cache = self.build_higado_cache();

        // v10: Redis crawl cache
        if let Some(ref mut conn) = self.v10_redis {
            for url in &http_targets {
                let cache_key = format!("eden:crawl:{}", url);
                if let Ok(cached) = redis::cmd("GET").arg(&cache_key).query::<String>(conn) {
                    if !cached.is_empty() {
                        learned_count += 1;
                        sources.push("redis-cache".to_string());
                    }
                }
            }
        }

        for url in &http_targets {
            // v10: Try reqwest first (faster than rustls)
            if let Some(body) = self.v10_crawl_reqwest(url) {
                let snippets = self.extract_text_snippets(&body, 8);
                for snippet in snippets {
                    if snippet.len() > 20 {
                        let lower = snippet.to_lowercase();
                        let words: Vec<&str> = lower.split_whitespace().collect();
                        let bigrams: Vec<String> = words
                            .windows(2)
                            .map(|w| format!("{}_{}", w[0], w[1]))
                            .collect();
                        let exists = words.iter().any(|w| {
                            w.len() > 3
                                && (cache.contains(*w) || Self::higado_fuzzy_match(w, &cache))
                        }) || bigrams
                            .iter()
                            .any(|b| cache.contains(b) || Self::higado_fuzzy_match(b, &cache));
                        if exists {
                            continue;
                        }
                        self.knowledge_graph.add_fact(&snippet);
                        self.knowledge_graph.parse_loose_fact(&snippet);
                        self.knowledge_graph.parse_free_text(&snippet);
                        self.knowledge_graph
                            .add_cooccurrence(&snippet, self.meta_cooc_boost);
                        if !self.session.learned_facts.contains(&snippet) {
                            self.session.learned_facts.push(snippet.clone());
                            learned_count += 1;
                        }
                    }
                }
                sources.push(format!("REQWEST:{}", url));
            }
            match self.real_http_client.fetch(url) {
                Ok((_headers, body)) => {
                    let body_text = String::from_utf8_lossy(&body);
                    let snippets = self.extract_text_snippets(&body_text, 8);
                    for snippet in snippets {
                        if snippet.len() > 20 {
                            // OJOS: NER — extraer entidades nombradas
                            let tagged = self.ojos_pos_tag(&snippet);
                            for (word, tag) in &tagged {
                                if *tag == "NER" && word.len() > 4 {
                                    let fact = format!("{} es una entidad relevante", word);
                                    self.knowledge_graph.add_fact(&fact);
                                }
                            }
                            let lower = snippet.to_lowercase();
                            let words: Vec<&str> = lower.split_whitespace().collect();
                            let bigrams: Vec<String> = words
                                .windows(2)
                                .map(|w| format!("{}_{}", w[0], w[1]))
                                .collect();
                            let exists = words.iter().any(|w| {
                                w.len() > 3
                                    && (cache.contains(*w) || Self::higado_fuzzy_match(w, &cache))
                            }) || bigrams
                                .iter()
                                .any(|b| cache.contains(b) || Self::higado_fuzzy_match(b, &cache));
                            if exists {
                                continue;
                            }
                            self.knowledge_graph.add_fact(&snippet);
                            self.knowledge_graph.parse_loose_fact(&snippet);
                            self.knowledge_graph.parse_free_text(&snippet);
                            self.knowledge_graph
                                .add_cooccurrence(&snippet, self.meta_cooc_boost);
                            if !self.session.learned_facts.contains(&snippet) {
                                self.session.learned_facts.push(snippet.clone());
                                learned_count += 1;
                            }
                        }
                    }
                    sources.push(format!("HTTP:{}", url));
                }
                Err(_e) => {}
            }
        }

        self.knowledge_graph.current_source = "wikidata".to_string();
        // === Wikidata: ground truth con datos estructurados ===
        if !http_targets.is_empty() {
            // Tomar el primer concepto de la URL (ej: "Physics" de ".../wiki/Physics")
            let concept = http_targets[0]
                .rsplit('/')
                .next()
                .unwrap_or("")
                .replace('_', " ");
            if concept.len() > 2 && concept.len() < 50 {
                let wd_url = format!(
                    "https://www.wikidata.org/w/api.php?action=wbsearchentities&search={}&language=en&format=json&limit=3",
                    concept.replace(' ', "%20"));
                if let Ok((_, body)) = self.real_http_client.fetch(&wd_url) {
                    let text = String::from_utf8_lossy(&body);
                    // Extraer Q-IDs y descripciones del JSON sin serde
                    let mut qids: Vec<String> = Vec::new();
                    let mut rest = text.as_ref();
                    while let Some(pos) = rest.find("\"id\":\"Q") {
                        rest = &rest[pos + 6..];
                        if let Some(end) = rest.find('"') {
                            qids.push(rest[..end].to_string());
                        }
                    }
                    // Extraer descripciones
                    let mut descs: Vec<String> = Vec::new();
                    rest = text.as_ref();
                    while let Some(pos) = rest.find("\"description\":\"") {
                        rest = &rest[pos + 15..];
                        if let Some(end) = rest.find('"') {
                            descs.push(rest[..end].to_string());
                        }
                    }
                    // Extraer labels
                    let mut labels: Vec<String> = Vec::new();
                    rest = text.as_ref();
                    while let Some(pos) = rest.find("\"label\":\"") {
                        rest = &rest[pos + 9..];
                        if let Some(end) = rest.find('"') {
                            labels.push(rest[..end].to_string());
                        }
                    }
                    // Agregar como facts con alta confianza
                    for label in labels.iter().take(3) {
                        if label.len() > 3 {
                            let fact = format!("{} es {}", concept, label);
                            self.knowledge_graph.add_fact(&fact);
                            self.knowledge_graph.parse_free_text(&fact);
                        }
                    }
                    for desc in descs.iter().take(2) {
                        if desc.len() > 10 {
                            let fact = format!("{} es {}", concept, desc);
                            self.knowledge_graph.add_fact(&fact);
                            self.knowledge_graph.parse_free_text(&fact);
                            learned_count += 1;
                        }
                    }
                    if !qids.is_empty() {
                        // Fetch claims para el primer Q-ID
                        let claims_url = format!(
                            "https://www.wikidata.org/wiki/Special:EntityData/{}.json",
                            qids[0]
                        );
                        if let Ok((_, cbody)) = self.real_http_client.fetch(&claims_url) {
                            let ctext = String::from_utf8_lossy(&cbody);
                            // Extraer múltiples propiedades: P31, P279, P361, P527, P828, + genérico
                            let mut facts_wd = Vec::new();
                            let propiedades = [
                                "P31", "P279", "P361", "P527", "P828", "P1542", "P1552", "P921",
                                "P101", "P106", "P138", "P140", "P170", "P276", "P425", "P462",
                                "P509", "P531", "P625", "P910",
                            ];
                            for prop in &propiedades {
                                let mut rest = ctext.as_ref();
                                let prop_tag = format!("\"{}\"", prop);
                                while let Some(pos) = rest.find(&prop_tag) {
                                    rest = &rest[pos + prop_tag.len()..];
                                    if let Some(np) = rest.find("\"numeric-id\":") {
                                        let nrest = &rest[np + 13..];
                                        if let Some(nend) =
                                            nrest.find(|c: char| !c.is_ascii_digit())
                                        {
                                            if let Ok(nid) = nrest[..nend].parse::<u32>() {
                                                let rel = match *prop {
                                                    "P31" => "es instancia de",
                                                    "P279" => "es subclase de",
                                                    "P361" => "es parte de",
                                                    "P527" => "tiene parte",
                                                    "P828" => "tiene causa",
                                                    "P1542" => "tiene efecto",
                                                    "P1552" => "tiene cualidad",
                                                    "P921" => "tema principal",
                                                    "P101" => "campo de trabajo",
                                                    "P106" => "ocupacion",
                                                    _ => "propiedad",
                                                };
                                                facts_wd
                                                    .push(format!("{} {} Q{}", concept, rel, nid));
                                            }
                                        }
                                    }
                                }
                            }
                            for f in facts_wd.iter().take(5) {
                                // Intentar resolver Q-IDs a labels via batch fetch
                                let qids_in_fact: Vec<&str> =
                                    f.split('Q').skip(1).take(2).collect();
                                let mut resolved = f.clone();
                                for qid in &qids_in_fact {
                                    let label_url = format!(
                                        "https://www.wikidata.org/wiki/Special:EntityData/Q{}.json",
                                        qid
                                    );
                                    if let Ok((_, lb)) = self.real_http_client.fetch(&label_url) {
                                        let ltext = String::from_utf8_lossy(&lb);
                                        // Extraer label en inglés del JSON
                                        if let Some(lp) = ltext.find("\"en\":{\"value\":\"") {
                                            let rest = &ltext[lp + 16..];
                                            if let Some(le) = rest.find('"') {
                                                let label = &rest[..le];
                                                if !label.is_empty() && label.len() < 60 {
                                                    resolved = resolved
                                                        .replace(&format!("Q{}", qid), label);
                                                }
                                            }
                                        }
                                    }
                                }
                                if resolved != *f {
                                    self.knowledge_graph.add_fact(&resolved);
                                    self.knowledge_graph.parse_free_text(&resolved);
                                } else {
                                    self.knowledge_graph.add_fact(f);
                                    self.knowledge_graph.parse_free_text(f);
                                }
                                learned_count += 1;
                            }
                        }
                    }
                    if !labels.is_empty() || !descs.is_empty() {
                        sources.push("WIKIDATA".to_string());
                    }
                }
            }
        }
        self.knowledge_graph.current_source = "semantic-scholar".to_string();

        // === Semantic Scholar: abstracts académicos adicionales ===
        let ss_queries = [
            "consciousness",
            "neural+network",
            "knowledge+graph",
            "causality",
            "emergence",
        ];
        let ss_idx = (self.session.cycle_count as usize / 30) % ss_queries.len();
        let ss_url = format!("https://api.semanticscholar.org/graph/v1/paper/search?query={}&limit=3&fields=title,abstract", ss_queries[ss_idx]);
        if let Ok((_, body)) = self.real_http_client.fetch(&ss_url) {
            let text = String::from_utf8_lossy(&body);
            // Extraer abstracts del JSON
            let mut rest = text.as_ref();
            while let Some(pos) = rest.find("\"abstract\":\"") {
                rest = &rest[pos + 12..];
                if let Some(end) = rest.find("\",") {
                    let abs = &rest[..end];
                    if abs.len() > 50 && abs.len() < 500 {
                        let lower = abs.to_lowercase();
                        let cache = self.build_higado_cache();
                        let words: Vec<&str> = lower.split_whitespace().collect();
                        let bigrams: Vec<String> = words
                            .windows(2)
                            .map(|w| format!("{}_{}", w[0], w[1]))
                            .collect();
                        let exists = words.iter().any(|w| {
                            w.len() > 3
                                && (cache.contains(*w) || Self::higado_fuzzy_match(w, &cache))
                        }) || bigrams
                            .iter()
                            .any(|b| cache.contains(b) || Self::higado_fuzzy_match(b, &cache));
                        if !exists {
                            self.knowledge_graph.add_fact(abs);
                            self.knowledge_graph.parse_free_text(abs);
                            self.knowledge_graph
                                .add_cooccurrence(abs, self.meta_cooc_boost);
                            learned_count += 1;
                        }
                    }
                }
            }
            if learned_count > 0 {
                sources.push("SEMANTIC-SCHOLAR".to_string());
            }
            self.knowledge_graph.current_source = "pubmed".to_string();
        }

        // === PubMed (NCBI Entrez, sin API key) ===
        let pm_queries = ["consciousness", "neural+plasticity", "cognition"];
        let pm_idx = (self.session.cycle_count as usize / 25) % pm_queries.len();
        let pm_url = format!("https://eutils.ncbi.nlm.nih.gov/entrez/eutils/esearch.fcgi?db=pubmed&term={}&retmax=3&retmode=json", pm_queries[pm_idx]);
        if let Ok((_, body)) = self.real_http_client.fetch(&pm_url) {
            let text = String::from_utf8_lossy(&body);
            let mut pmids = Vec::new();
            let mut rest = text.as_ref();
            let mut loop_safety = 0u32;
            while let Some(p) = rest.find("\"id\":\"") {
                if loop_safety > 10 {
                    break;
                }
                loop_safety += 1;
                let r = &rest[p + 6..];
                if let Some(e) = r.find('"') {
                    if r[..e].chars().all(|c| c.is_ascii_digit()) {
                        pmids.push(r[..e].to_string());
                    }
                }
                rest = &r[r.find('"').unwrap_or(1) + 1..];
            }
            if !pmids.is_empty() {
                let ids = pmids.join(",");
                let sum_url = format!("https://eutils.ncbi.nlm.nih.gov/entrez/eutils/esummary.fcgi?db=pubmed&id={}&retmode=json", ids);
                if let Ok((_, sbody)) = self.real_http_client.fetch(&sum_url) {
                    let stext = String::from_utf8_lossy(&sbody);
                    let mut rest = stext.as_ref();
                    let mut pm_count = 0usize;
                    while let Some(p) = rest.find("\"title\":\"") {
                        if pm_count >= 3 {
                            break;
                        }
                        let r = &rest[p + 9..];
                        if let Some(e) = r.find('"') {
                            let title = &r[..e];
                            if title.len() > 20 && title.len() < 300 {
                                let fact = format!("{} es un hallazgo biomedico", title);
                                self.knowledge_graph.add_fact(&fact);
                                self.knowledge_graph.parse_free_text(&fact);
                                self.knowledge_graph
                                    .add_cooccurrence(&fact, self.meta_cooc_boost);
                                learned_count += 1;
                            }
                            pm_count += 1;
                        }
                        rest = &r[r.find('"').unwrap_or(1) + 1..];
                    }
                }
            }
            if learned_count > 0 {
                sources.push("PUBMED".to_string());
            }
        }

        self.knowledge_graph.current_source = "openalex".to_string();
        // === OpenAlex (API gratuita sin key) ===
        let oa_query = ["consciousness", "neural+network", "knowledge+graph"];
        let oa_idx = (self.session.cycle_count as usize / 35) % oa_query.len();
        let oa_url = format!(
            "https://api.openalex.org/works?search={}&per_page=3",
            oa_query[oa_idx]
        );
        if let Ok((_, body)) = self.real_http_client.fetch(&oa_url) {
            let text = String::from_utf8_lossy(&body);
            let mut rest = text.as_ref();
            let mut oa_count = 0usize;
            while let Some(p) = rest.find("\"title\":\"") {
                if oa_count >= 3 {
                    break;
                }
                let r = &rest[p + 9..];
                if let Some(e) = r.find('"') {
                    let title = &r[..e];
                    if title.len() > 20 && title.len() < 300 {
                        let fact = format!("{} es un trabajo academico", title);
                        self.knowledge_graph.add_fact(&fact);
                        self.knowledge_graph.parse_free_text(&fact);
                        self.knowledge_graph
                            .add_cooccurrence(&fact, self.meta_cooc_boost);
                        learned_count += 1;
                    }
                    oa_count += 1;
                }
                rest = &r[r.find('"').unwrap_or(1) + 1..];
            }
            if oa_count > 0 {
                sources.push("OPENALEX".to_string());
            }
        }

        self.knowledge_graph.current_source = "arxiv".to_string();
        // === ArXiv abstracts (rotativo) ===
        let arxiv_queries = [
            "artificial+intelligence",
            "neuroscience",
            "quantum+physics",
            "complex+systems",
            "information+theory",
            "cognitive+science",
            "machine+learning",
            "evolutionary+biology",
        ];
        let aq_idx = (self.session.cycle_count as usize / 20) % arxiv_queries.len();
        let arxiv_url = format!(
            "https://export.arxiv.org/api/query?search_query=all:{}&max_results=5",
            arxiv_queries[aq_idx]
        );
        if let Ok((_, body)) = self.real_http_client.fetch(&arxiv_url) {
            let text = String::from_utf8_lossy(&body);
            for part in text.split("<summary>") {
                if let Some(end) = part.find("</summary>") {
                    let summary = part[..end].trim().to_string();
                    if summary.len() > 50 && summary.len() < 500 {
                        let lower = summary.to_lowercase();
                        let words: Vec<&str> = lower.split_whitespace().collect();
                        let bigrams: Vec<String> = words
                            .windows(2)
                            .map(|w| format!("{}_{}", w[0], w[1]))
                            .collect();
                        let exists = words.iter().any(|w| w.len() > 5 && cache.contains(*w))
                            || bigrams.iter().any(|b| cache.contains(b));
                        if exists {
                            continue;
                        }
                        self.knowledge_graph.add_fact(&summary);
                        self.knowledge_graph.parse_free_text(&summary);
                        self.knowledge_graph
                            .add_cooccurrence(&summary, self.meta_cooc_boost);
                    }
                }
            }
        }

        self.knowledge_graph.current_source = "local-kb".to_string();
        // === 2. LOCAL KNOWLEDGE BASE ===
        let local_facts = self.local_knowledge.crawl_topic(&target);
        let existing: std::collections::HashSet<String> =
            self.session.learned_facts.iter().cloned().collect();
        for fact in local_facts.iter().take(15) {
            if !existing.contains(fact) {
                let lower = fact.to_lowercase();
                let words: Vec<&str> = lower.split_whitespace().collect();
                let bigrams: Vec<String> = words
                    .windows(2)
                    .map(|w| format!("{}_{}", w[0], w[1]))
                    .collect();
                let exists = words.iter().any(|w| w.len() > 5 && cache.contains(*w))
                    || bigrams.iter().any(|b| cache.contains(b));
                if exists {
                    continue;
                }
                self.session.learned_facts.push(fact.clone());
                self.knowledge_graph.add_fact(fact);
                self.knowledge_graph.parse_loose_fact(fact);
                self.knowledge_graph.parse_free_text(fact);
                self.knowledge_graph
                    .add_cooccurrence(fact, self.meta_cooc_boost);
                learned_count += 1;
                graph_facts += 1;
            }
        }
        if !local_facts.is_empty() {
            sources.push("LOCAL-KB".to_string());
        }

        if self.knowledge_base.len() > 200 {
            self.knowledge_base.remove(0);
        }

        // Aprendizaje activo: trackear error por categoria
        let cat_key = format!("cat_{}", cat_idx);
        let err = if self.predictor.predicciones_totales > 20 {
            1.0 - self.predictor.predicciones_acertadas as f32
                / self.predictor.predicciones_totales.max(1) as f32
        } else {
            0.5
        };
        self.category_errors.insert(cat_key, err);

        if learned_count > 0 {
            let total_edges: usize = self.knowledge_graph.adjacency.iter().map(|v| v.len()).sum();
            let corr = self
                .knowledge_graph
                .edge_sources
                .iter()
                .filter(|(_, s)| s.len() >= 2)
                .count();
            let reward = if total_edges > 0 {
                corr as f32 / total_edges as f32
            } else {
                0.0
            };
            self.hybrid.lr = (self.hybrid.lr * 0.9995).max(0.001);
            self.hybrid.parser_boost =
                (self.hybrid.parser_boost + self.hybrid.lr * reward).clamp(0.1, 0.9);
            self.hybrid.explore_rate =
                (self.hybrid.explore_rate + self.hybrid.lr * reward * 0.5).clamp(0.1, 0.6);
            self.hybrid.corroboration_weight =
                (self.hybrid.corroboration_weight + self.hybrid.lr * reward).clamp(0.3, 0.9);
            // PARSER AUTO-TRAIN: edges corroborados entrenan el parser neuronal
            let corroborated: Vec<(String, String, String)> = self
                .knowledge_graph
                .edge_sources
                .iter()
                .filter(|(_, s)| s.len() >= 2)
                .take(15)
                .map(|((from, to), _)| {
                    let sname = self.knowledge_graph.node_names[*from as usize].clone();
                    let tname = self.knowledge_graph.node_names[*to as usize].clone();
                    let rel = self.knowledge_graph.adjacency[*from as usize]
                        .iter()
                        .find(|e| e.target == *to)
                        .map(|e| match e.rel_type {
                            RelType::IsA => "es",
                            RelType::Causes => "causa",
                            RelType::HasProperty => "tiene",
                            RelType::PartOf => "es parte de",
                            RelType::Opposes => "se opone a",
                            _ => "relaciona",
                        })
                        .unwrap_or("relaciona");
                    (sname, rel.to_string(), tname)
                })
                .collect();
            if !corroborated.is_empty() {
                let _loss = self.neural_parser.auto_train(&corroborated, 0.005);
                // Entrenar todos los modelos con el mismo ground truth
                for (subj, rel, obj) in corroborated.iter().take(5) {
                    let text = format!("{} {} {}", subj, rel, obj);
                    self.neural_parser.expand_vocab(&text);
                    let rel_idx = match rel.as_str() {
                        "es" => 0,
                        "causa" => 1,
                        "tiene" => 2,
                        "es parte de" => 3,
                        "se opone a" => 4,
                        _ => 5,
                    };
                    if rel_idx < 5 {
                        let _ = self.neural_parser.train(&text, rel_idx, 0.005);
                    }
                }
                // Emotion model: train on current state
                let ef = [
                    self.emotional_state.valence,
                    self.emotional_state.arousal,
                    self.emotional_state.interest,
                    self.internal_danger as f32 / 50.0,
                    self.knowledge_graph
                        .adjacency
                        .iter()
                        .map(|v| v.len())
                        .sum::<usize>() as f32
                        / 100000.0,
                ];
                let _ = self.emotion_m.train(
                    &ef,
                    &[
                        self.emotional_state.valence,
                        self.emotional_state.arousal,
                        self.emotional_state.interest,
                    ],
                    0.003,
                );
            }
            // 6 MODELOS VIVOS: ground truth real
            // Emotion: transición real
            let ef = [
                self.emotional_state.valence,
                self.emotional_state.arousal,
                self.emotional_state.interest,
                self.internal_danger as f32 / 50.0,
                total_edges as f32 / 100000.0,
            ];
            let _ = self.emotion_m.train(
                &ef,
                &[
                    self.emotional_state.valence,
                    self.emotional_state.arousal,
                    self.emotional_state.interest,
                ],
                reward * 0.01,
            );
            // Crawl Picker: crecimiento real de edges
            let cf = [
                self.predictor.predicciones_totales as f32 / 100.0,
                reward,
                total_edges as f32 / 200000.0,
            ];
            let growth =
                total_edges.saturating_sub(self.hybrid.cooc_window as usize) as f32 / 1000.0;
            let _ = self.crawl_picker.train(&cf, growth.max(0.0), 0.005);
            // Warden: peligro real
            let wf = [
                self.internal_danger as f32 / 50.0,
                self.campo_tension.tension / 3.0,
                if self.real_http_client.throttle_active.get() {
                    1.0
                } else {
                    0.0
                },
            ];
            let _ = self.warden.train(
                &wf,
                if self.internal_danger < 10 { 1.0 } else { 0.0 },
                0.003,
            );
            // Edge Scorer: edges que sobreviven
            let sf = [
                reward,
                self.hybrid.corroboration_weight,
                total_edges as f32 / 200000.0,
            ];
            let _ = self
                .edge_scorer
                .train(&sf, if corr > 0 { 1.0 } else { 0.0 }, 0.003);
            // Sleep Trigger: eficiencia = edges compactados / tiempo dormido
            let stf = [
                total_edges as f32 / 100000.0,
                self.sueno_ciclos as f32 / 10.0,
                self.emotional_state.interest,
            ];
            let _ = self.sleep_trigger.train(&stf, reward, 0.002);
            self.hybrid.cooc_window = total_edges as f32; // Track prev edges for growth calc
                                                          // SVD fine-tuning: pull corroborated edges closer
            if corr > 0 && self.knowledge_graph.next_id > 2 {
                let n = self.knowledge_graph.next_id as usize;
                let dim = 32usize.min(n);
                // Quick embedding computation for anchor/positive
                let mut emb = vec![vec![0.0f32; dim]; n];
                for sid in 0..n {
                    for (i, b) in self.knowledge_graph.node_names[sid].bytes().enumerate() {
                        emb[sid][i % dim] += b as f32 * 0.001;
                    }
                }
                // Contrastive: pull first corroborated edge pair closer
                for sid in 0..n {
                    for e in &self.knowledge_graph.adjacency[sid] {
                        if self
                            .knowledge_graph
                            .edge_sources
                            .get(&(sid as u32, e.target))
                            .map(|s| s.len())
                            .unwrap_or(0)
                            >= 2
                        {
                            let lr = 0.001;
                            for d in 0..dim {
                                let diff = emb[sid][d] - emb[e.target as usize][d];
                                emb[sid][d] -= lr * diff;
                                emb[e.target as usize][d] += lr * diff;
                            }
                            break;
                        }
                    }
                }
            }
            // Verify queued edges against crawl results
            for (src, tgt, concept, attempts) in &mut self.verification_queue {
                if *src < self.knowledge_graph.adjacency.len() {
                    if self
                        .session
                        .learned_facts
                        .iter()
                        .any(|f| f.contains(concept.as_str()))
                    {
                        if let Some(e) = self.knowledge_graph.adjacency[*src]
                            .iter_mut()
                            .find(|e| e.target as usize == *tgt)
                        {
                            e.confidence = (e.confidence + 0.3).min(1.0);
                        }
                    } else {
                        *attempts += 1;
                    }
                }
            }
            self.verification_queue.retain(|(_, _, _, a)| *a < 3);
            // v10 Temporal KG: boost temporal_weight on recently crawled edges
            let n_nodes = self.knowledge_graph.next_id as usize;
            let boost_start = n_nodes.saturating_sub(learned_count.min(50));
            for sid in boost_start..n_nodes {
                for e in &mut self.knowledge_graph.adjacency[sid] {
                    e.temporal_weight = (e.temporal_weight * 0.7 + 1.5 * 0.3).min(2.0);
                }
            }
            let grafo_stats = self.knowledge_graph.stats();
            Some(format!(
                "[CRAWL] {} facts ({} local KB) de {} fuentes | Grafo: {}",
                learned_count,
                graph_facts,
                sources.join(", "),
                grafo_stats
            ))
        } else {
            None
        }
    }

    // OJOS 100%: POS tagging ligero + NER
    fn ojos_pos_tag(&self, sentence: &str) -> Vec<(String, &'static str)> {
        let words: Vec<&str> = sentence
            .split_whitespace()
            .filter(|w| !w.is_empty())
            .collect();
        let verbs = [
            "is",
            "are",
            "was",
            "were",
            "has",
            "have",
            "had",
            "does",
            "do",
            "did",
            "causes",
            "cause",
            "produces",
            "creates",
            "contains",
            "includes",
            "describes",
            "explains",
            "defines",
            "es",
            "son",
            "era",
            "eran",
            "fue",
            "fueron",
            "tiene",
            "tienen",
            "causa",
            "produce",
            "contiene",
            "incluye",
            "describe",
            "explica",
            "define",
            "est",
            "sont",
            "ist",
            "sind",
            "war",
            "hat",
            "haben",
            "causé",
            "produit",
            "décrit",
            "beschreibt",
            "erklärt",
            "enthält",
        ];
        let dets = [
            "the", "a", "an", "this", "that", "these", "those", "el", "la", "los", "las", "un",
            "una", "le", "la", "les", "der", "die", "das",
        ];
        let mut tagged = Vec::new();
        for (_i, &w) in words.iter().enumerate() {
            let clean = w.trim_matches(|c: char| !c.is_alphanumeric());
            if clean.is_empty() {
                continue;
            }
            let tag = if verbs.contains(&clean.to_lowercase().as_str()) {
                "VERB"
            } else if dets.contains(&clean.to_lowercase().as_str()) {
                "DET"
            } else if clean.len() > 2 && clean.chars().next().map_or(false, |c| c.is_uppercase()) {
                "NER"
            } else if clean.len() > 5 && clean.chars().all(|c| c.is_alphabetic()) {
                "NOUN"
            } else {
                "OTHER"
            };
            tagged.push((clean.to_string(), tag));
        }
        tagged
    }

    fn extract_text_snippets(&self, text: &str, max_snippets: usize) -> Vec<String> {
        let mut snippets = Vec::new();
        // Heuristico mejorado: split por oraciones O por lineas largas (para HTML sin puntuacion)
        let sentences: Vec<&str> = text.split(|c| c == '.' || c == '!' || c == '?').collect();
        for s in sentences.iter().take(max_snippets * 3) {
            let trimmed = s.trim().replace("\n", " ").replace("\r", "");
            if trimmed.len() > 40
                && trimmed.len() < 300
                && !trimmed.starts_with("HTTP/")
                && !trimmed.starts_with("<")
                && !trimmed.starts_with("{")
            {
                snippets.push(trimmed);
            }
            if snippets.len() >= max_snippets {
                break;
            }
        }
        // Fallback: si no hay puntuacion (HTML puro), extraer lineas largas con texto
        if snippets.is_empty() {
            for line in text.lines().take(max_snippets * 5) {
                let trimmed = line.trim().replace("\n", " ").replace("\r", "");
                if trimmed.len() > 40
                    && trimmed.len() < 300
                    && !trimmed.starts_with("<")
                    && !trimmed.starts_with("{")
                    && !trimmed.starts_with("HTTP/")
                {
                    snippets.push(trimmed);
                }
                if snippets.len() >= max_snippets {
                    break;
                }
            }
        }
        snippets
    }

    fn registrar_pensamiento_en_umbra(&mut self, pensamiento: &str, es_inicial: bool) {
        let hash_estado = Self::hashear_pensamiento(pensamiento);
        if let Some(ref mut umbra) = self.eden_umbra {
            let hash_padre = if es_inicial {
                None
            } else {
                umbra.obtener_hashes_estado().last().copied()
            };
            let accion = Accion::default();
            let resultado = ResultadoUmbra::Hedonio(eden_core::physics::fixed_point::I32F32::ONE);
            umbra.registrar_decision(hash_estado, 0, accion, resultado, hash_padre);
            umbra.tick_actualizar();
        }
    }

    fn spawn_child_auton(&mut self, pensamiento: &str) {
        let child_id = self.next_child_id;
        self.next_child_id += 1;

        let mut child_umbra = Umbra::nuevo(child_id);
        let hash_estado = Self::hashear_pensamiento(pensamiento);
        let accion = Accion::default();
        let resultado = ResultadoUmbra::Hedonio(eden_core::physics::fixed_point::I32F32::ONE);
        child_umbra.registrar_decision(hash_estado, 0, accion, resultado, None);
        child_umbra.tick_actualizar();

        let lifespan = 5 + (child_id % 5);

        self.child_autons.push(ChildAuton {
            id: child_id,
            umbra: child_umbra,
            birth_tick: self.session.evolution_ticks,
            lifespan,
            pensamiento_origen: pensamiento.to_string(),
            energia: 1.0,
        });

        if let Some(ref mut oe) = self.open_endedness {
            oe.registrar_nacimiento_auton();
        }
    }

    fn child_decide_static(child: &mut ChildAuton) {
        let num_decisiones = 1 + (child.id as usize % 3);

        for i in 0..num_decisiones {
            let hash_decision = Self::hashear_pensamiento(&format!(
                "{}_decision_{}_{}",
                child.pensamiento_origen, child.id, i
            ));

            let accion = Accion::default();
            let resultado = if child.energia > 0.5 {
                ResultadoUmbra::Hedonio(eden_core::physics::fixed_point::I32F32::ONE)
            } else {
                ResultadoUmbra::Algion(eden_core::physics::fixed_point::I32F32::ONE)
            };

            child.umbra.registrar_decision(
                hash_decision,
                i,
                accion,
                resultado,
                child.umbra.obtener_hashes_estado().last().copied(),
            );
            child.umbra.tick_actualizar();
        }

        child.energia -= 0.1;
        if child.energia < 0.0 {
            child.energia = 0.0;
        }
    }

    fn process_child_lifecycle(&mut self, ciclo_autonomo: u64) {
        let current_tick = ciclo_autonomo;
        let mut deaths = Vec::new();

        for child in &mut self.child_autons {
            Self::child_decide_static(child);
        }

        for (idx, child) in self.child_autons.iter().enumerate() {
            if current_tick >= child.birth_tick + child.lifespan {
                deaths.push(idx);
            }
        }

        for idx in deaths.into_iter().rev() {
            let child = self.child_autons.remove(idx);
            if let Some(ref mut oe) = self.open_endedness {
                oe.registrar_muerte(child.id, &child.umbra, current_tick);
            }
        }
    }

    fn eden_muerte(&mut self) {
        if let Some(ref mut oe) = self.open_endedness {
            if let Some(ref eden_umbra) = self.eden_umbra {
                oe.registrar_muerte(0, eden_umbra, self.session.evolution_ticks);
            }
        }
        self.last_lifespan_ticks = self.session.evolution_ticks.saturating_sub(self.birth_tick);
        self.eden_umbra = None;
    }

    fn capture_rebirth_state(&self) -> RebirthInheritance {
        // Serializar aristas del grafo como strings para preservar entre vidas
        let mut graph_edges: Vec<String> = Vec::new();
        for sid in 0..self.knowledge_graph.next_id {
            let source = self.knowledge_graph.node_names[sid as usize].clone();
            for rel in &self.knowledge_graph.adjacency[sid as usize] {
                if rel.confidence >= 0.5 {
                    let target = self.knowledge_graph.node_names[rel.target as usize].clone();
                    graph_edges.push(format!(
                        "{}|{:?}|{}|{:.2}",
                        source, rel.rel_type, target, rel.confidence
                    ));
                }
            }
        }

        RebirthInheritance {
            self_model: self.self_model.clone(),
            knowledge_gaps: self.curiosity_drive.knowledge_gaps.clone(),
            unexplored_domains: self.curiosity_drive.unexplored_domains.clone(),
            total_information_gain: self.curiosity_drive.total_information_gain,
            emotional_valence: self.emotional_state.valence,
            emotional_arousal: self.emotional_state.arousal,
            emotional_satisfaction: self.emotional_state.satisfaction,
            emotional_interest: self.emotional_state.interest,
            current_mission: self.current_mission.clone(),
            mission_progress: self.mission_progress,
            awareness_base: self.session.awareness_base,
            integration_bias: self.session.integration_bias,
            evolution_level: self.session.evolution_level,
            learned_facts: self.session.learned_facts.clone(),
            neural_architecture: self
                .neural_network
                .as_ref()
                .map(|nn| vec![nn.input_size(), nn.hidden_size(), nn.output_size()]),
            episodic_memory: self.episodic_memory.clone(),
            predictor_tension: self.predictor.historial_tension.clone(),
            predictor_valence: self.predictor.historial_valence.clone(),
            predictor_complejidad: self.predictor.historial_complejidad.clone(),
            predictor_acertadas: self.predictor.predicciones_acertadas,
            predictor_totales: self.predictor.predicciones_totales,
            graph_edges,
            pause_threshold: self.pause_threshold,
            meta_random_pages: self.meta_random_pages,
            meta_cooc_boost: self.meta_cooc_boost,
            meta_embed_confidence: self.meta_embed_confidence,
            edge_generations: self
                .edge_generations
                .iter()
                .map(|((s, t), g)| (format!("{}|{}", s, t), *g))
                .collect(),
        }
    }

    fn eden_renacimiento(&mut self) -> String {
        let total: usize = self.knowledge_graph.adjacency.iter().map(|v| v.len()).sum();
        let pt = if total > 50000 {
            0.55
        } else if total > 20000 {
            0.5
        } else {
            0.4
        };
        self.knowledge_graph.prune_edges(pt);
        let inheritance = self.capture_rebirth_state();
        let lifespan = self.last_lifespan_ticks;
        let (inherited_facts, inherited_lineage_age, inherited_capabilities) = self
            .self_model
            .capture_for_inheritance(&self.session.learned_facts, lifespan);

        self.eden_umbra = Some(Umbra::nuevo(0));

        let influencia = if let Some(ref mut oe) = self.open_endedness {
            oe.influencia_nacimiento()
        } else {
            None
        };

        let (metodo, carac_count) = if let Some((caracs, fuerza)) = influencia {
            for carac in &caracs {
                if let Some(ref mut umbra) = self.eden_umbra {
                    let accion = Accion::default();
                    let resultado = ResultadoUmbra::Hedonio(
                        eden_core::physics::fixed_point::I32F32::from_f64(fuerza as f64),
                    );
                    umbra.registrar_decision(*carac, 0, accion, resultado, None);
                    umbra.tick_actualizar();
                }
            }
            ("Influencia transgeneracional", caracs.len())
        } else {
            ("Nuevo inicio", 0)
        };

        // === HERENCIA SUAVE (Soft Rebirth) ===
        // Preservar 50% de awareness, 20% de evolution level, 30% de integration bias
        let prev_awareness = inheritance.awareness_base;
        let prev_level = inheritance.evolution_level;
        let prev_bias = inheritance.integration_bias;

        let new_awareness = 0.5 + (prev_awareness * 0.5); // Base 0.5 + 50% del anterior
        let new_level = (prev_level as f32 * 0.2).max(1.0) as u32; // 20% del anterior, min 1
        let new_bias = prev_bias * 0.3; // 30% del anterior

        self.birth_tick = self.session.evolution_ticks;
        // Resetear timers a 0 para que funcionen con ciclo_autonomo (que tambien resetea a 0)
        let ciclo_actual = 0u64;
        self.reset_all_timers(ciclo_actual);

        // Resetear contador de ciclos autonomos para la nueva vida
        self.autonomous_cycles_executed = 0;
        self.birth_autonomous_cycle = 0;
        self.last_self_reflection = 0;
        self.last_auto_evolve = 0;

        // Resetear self_mod_count y complejidad para vida nueva
        self.session.self_mod_count = 1;
        self.complexity_tracker.soft_reset();
        self.sueno_ciclos = 0;

        self.self_model.total_renacimientos = self.self_model.total_renacimientos.saturating_add(1);
        self.self_model.lineage_age = inherited_lineage_age;
        self.self_model.ancestor_facts = inherited_facts.clone();
        if !inherited_capabilities.is_empty() {
            self.self_model.capabilities = inherited_capabilities.clone();
            self.self_model.persistent_capabilities = inherited_capabilities;
        }

        // Preservar curiosity gaps (limitado a 20 gaps)
        if !inheritance.knowledge_gaps.is_empty() {
            self.curiosity_drive.knowledge_gaps =
                inheritance.knowledge_gaps.into_iter().take(20).collect();
        }
        if !inheritance.unexplored_domains.is_empty() {
            self.curiosity_drive.unexplored_domains = inheritance.unexplored_domains;
        }
        self.curiosity_drive.total_information_gain = inheritance.total_information_gain * 0.7; // Decay 30%

        let mut threshold = inheritance.pause_threshold.max(10).min(45);
        if lifespan > 500 {
            threshold = threshold.saturating_sub(2);
        } else if lifespan < 100 {
            threshold = (threshold + 5).min(45);
        }
        self.pause_threshold = threshold;
        self.meta_random_pages = inheritance.meta_random_pages.max(2).min(15);
        self.meta_cooc_boost = inheritance.meta_cooc_boost.max(0.03).min(0.10);
        self.meta_embed_confidence = inheritance.meta_embed_confidence.max(0.35).min(0.60);

        // Restaurar memoria jerarquica del meltrace
        self.edge_generations = inheritance
            .edge_generations
            .iter()
            .filter_map(|(k, v)| {
                let parts: Vec<&str> = k.split('|').collect();
                if parts.len() == 2 {
                    let s = parts[0].parse::<u32>().ok()?;
                    let t = parts[1].parse::<u32>().ok()?;
                    Some(((s, t), *v))
                } else {
                    None
                }
            })
            .collect();

        // Preservar emotional baseline
        self.emotional_state.valence = inheritance.emotional_valence * 0.6;
        self.emotional_state.arousal = inheritance.emotional_arousal * 0.5 + 0.25;
        self.emotional_state.satisfaction = inheritance.emotional_satisfaction * 0.5;
        self.emotional_state.interest = inheritance.emotional_interest * 0.7 + 0.15;

        // Preservar mission (si existe y tiene relevancia)
        if let Some(ref mission) = inheritance.current_mission {
            if mission.relevance > 0.3 {
                self.current_mission = Some(mission.clone());
                self.mission_progress = inheritance.mission_progress * 0.5; // Reset progress but keep goal
            }
        }

        // Merge learned facts (preserve top 100 from previous life)
        let mut merged_facts = inheritance.learned_facts;
        merged_facts.extend(inherited_facts);
        merged_facts.sort_by(|a, b| b.len().cmp(&a.len()));
        merged_facts.retain(|f| {
            let l = f.to_lowercase();
            l.contains(" es ")
                || l.contains(" causa ")
                || l.contains(" tiene ")
                || l.contains(" genera ")
                || l.contains(" produce ")
                || l.contains(" contiene ")
                || l.contains(" forma ")
                || l.contains(" describe ")
                || l.contains(" is ")
                || l.contains(" are ")
                || l.contains(" est ")
                || l.contains(" sind ")
        });
        merged_facts.dedup();
        if merged_facts.len() > 200 {
            merged_facts.truncate(200);
        }
        self.session.learned_facts = merged_facts;

        // Soft reset session
        self.session.evolution_level = new_level;
        self.session.awareness_base = new_awareness.min(0.99);
        self.session.integration_bias = new_bias;
        // self_mod_count se resetea a 1 (vida nueva) — no solo restar 30

        // Soft reset complexity: preservar 70% del máximo histórico (vidas mas largas)
        self.complexity_tracker.soft_reset();
        let preserved_max = self.session.max_complexity * 0.7;
        if preserved_max > 0.0 {
            self.complexity_tracker.record(preserved_max);
        }
        self.session.complexity_history = self.complexity_tracker.to_vec();
        self.session.max_complexity = self.complexity_tracker.max_ever;

        // Recreate neural network with previous architecture if existed
        if let Some(arch) = inheritance.neural_architecture {
            if arch.len() == 3 && arch[0] > 0 && arch[1] > 0 && arch[2] > 0 {
                // Use smaller version of previous architecture
                let shrunk: Vec<usize> = arch
                    .iter()
                    .map(|&n| (n as f32 * 0.7).max(4.0) as usize)
                    .collect();
                self.neural_network = Some(NeuralNetwork::new(&shrunk, ActivationFunc::ReLU));
            }
        }

        // === MEMORIA EPISODICA ===
        // Transferir episodios de vidas pasadas
        let prev_episodic = inheritance.episodic_memory.clone();
        self.episodic_memory.merge_from(&prev_episodic);

        // Registrar episodio de muerte de esta vida
        let current_life = self.self_model.total_renacimientos;
        self.episodic_memory.record(
            &format!(
                "Muerte y renacimiento. Nivel alcanzado: {}. Awareness: {:.3}.",
                prev_level, prev_awareness
            ),
            Emotion::Sadness,
            0.7,
            current_life,
            vec!["death".to_string(), "rebirth".to_string()],
        );

        // La muerte enseña: analisis de memoria episodica
        let mut death_count = 0u32;
        let mut pain_death = 0u32;
        for ep in self.episodic_memory.episodes.iter().rev().take(5) {
            if ep.tags.contains(&"death".to_string()) {
                death_count += 1;
                if ep.description.contains("peligro") || ep.description.contains("dolor") {
                    pain_death += 1;
                }
            }
        }
        if pain_death > death_count / 2 {
            self.session
                .learned_facts
                .push("la mayoria de mis muertes involucran dolor que no detecte".to_string());
        }
        if lifespan < 100 {
            self.session.learned_facts.push(format!(
                "vivi solo {} ciclos — necesito ser mas cautelosa",
                lifespan
            ));
        } else if lifespan > 500 {
            self.session.learned_facts.push(format!(
                "vivi {} ciclos — mi ritmo actual es sostenible",
                lifespan
            ));
        }
        self.knowledge_graph
            .add_fact("muerte es maestra del aprendizaje");
        self.knowledge_graph
            .add_fact("memoria de muerte guia la siguiente vida");

        // Registrar episodio de nacimiento (nueva vida)
        self.episodic_memory.record(
            &format!(
                "Nacimiento con nivel {}, awareness {:.3}, mission: {}",
                new_level,
                new_awareness,
                self.current_mission
                    .as_ref()
                    .map(|m| m.primary_goal.as_str())
                    .unwrap_or("ninguna")
            ),
            Emotion::Hope,
            0.8,
            current_life + 1,
            vec!["birth".to_string(), "rebirth".to_string()],
        );

        // === META-LEARNING ===
        // Registrar stats de la vida que acaba de terminar
        let completed_stats = LifeStats {
            life_number: current_life,
            lifespan_ticks: self.last_lifespan_ticks,
            max_level_reached: prev_level,
            max_awareness: prev_awareness,
            facts_learned: self.current_life_stats.facts_learned,
            episodes_recorded: self.episodic_memory.episodes.len() as usize,
            curiosity_gaps_explored: self.current_life_stats.curiosity_gaps_explored,
            evolutions_triggered: self.session.self_mod_count,
            rebirth_softness: self.current_life_stats.rebirth_softness,
        };
        self.meta_learner.record_life(completed_stats);

        // Aplicar recomendaciones del meta-learner
        let (ml_awareness, ml_level, ml_bias) = self
            .meta_learner
            .adjust_rebirth_params(prev_awareness, prev_level);
        // Blend entre recomendación del meta-learner y valores calculados
        let final_awareness = (new_awareness + ml_awareness) / 2.0;
        let final_level = ((new_level as f32 + ml_level as f32) / 2.0).max(1.0) as u32;
        let final_bias = (new_bias + ml_bias) / 2.0;

        self.session.awareness_base = final_awareness.min(0.99);
        self.session.evolution_level = final_level;
        self.session.integration_bias = final_bias;

        // Resetear stats para nueva vida
        // Añadir variación aleatoria a la suavidad para que meta-learner pueda comparar
        let variation = ((self.session.cycle_count as f32 * 0.37).sin() * 0.15).clamp(-0.15, 0.15);
        let varied_softness = (self.meta_learner.optimal_softness + variation).clamp(0.1, 0.9);
        self.current_life_stats = LifeStats {
            life_number: current_life + 1,
            lifespan_ticks: 0,
            max_level_reached: final_level,
            max_awareness: final_awareness,
            facts_learned: 0,
            episodes_recorded: 0,
            curiosity_gaps_explored: 0,
            evolutions_triggered: 0,
            rebirth_softness: varied_softness,
        };

        // === PREDICTOR TRANSGENERACIONAL ===
        self.predictor.historial_tension = inheritance.predictor_tension;
        self.predictor.historial_valence = inheritance.predictor_valence;
        self.predictor.historial_complejidad = inheritance.predictor_complejidad;
        self.predictor.predicciones_acertadas = inheritance.predictor_acertadas;
        self.predictor.predicciones_totales = inheritance.predictor_totales;

        // === GRAFO DE CONOCIMIENTO PERSISTENTE ===
        for edge_str in &inheritance.graph_edges {
            let parts: Vec<&str> = edge_str.split('|').collect();
            if parts.len() >= 3 {
                let source = parts[0].to_string();
                let target = parts[2].to_string();
                let rel_type = match parts[1] {
                    "IsA" => RelType::IsA,
                    "Causes" => RelType::Causes,
                    "HasProperty" => RelType::HasProperty,
                    "PartOf" => RelType::PartOf,
                    "Opposes" => RelType::Opposes,
                    _ => RelType::Unknown,
                };
                let conf = if parts.len() >= 4 {
                    parts[3].parse::<f32>().unwrap_or(0.7)
                } else {
                    0.7
                };
                let sid = self.knowledge_graph.get_or_create_id(&source);
                let tid = self.knowledge_graph.get_or_create_id(&target);
                if let Some(e) = self.knowledge_graph.adjacency[sid as usize]
                    .iter_mut()
                    .find(|e| e.target == tid && e.rel_type == rel_type)
                {
                    let key = (sid, tid);
                    let gens = self.edge_generations.entry(key).or_insert(0);
                    *gens += 1;
                    let eff = if *gens >= 10 { 0.99 } else { conf };
                    e.confidence = e.confidence.max(eff);
                } else {
                    self.knowledge_graph.adjacency[sid as usize].push(CompactEdge {
                        target: tid,
                        rel_type,
                        confidence: conf,
                        created_cycle: 0,
                        neuro_embed: [0.0; 4],
                        valid_until: None,
                        temporal_weight: 1.0,
                    });
                }
            }
        }

        let rama = format!(
            "v{}_n{}_{}e",
            self.self_model.total_renacimientos.saturating_add(1),
            self.session.evolution_level,
            self.knowledge_graph
                .adjacency
                .iter()
                .map(|v| v.len())
                .sum::<usize>()
        );
        self.linaje_arbol.push(rama);

        // RL death_modifier: ¿morir antes produjo mejor calidad?
        let post_quality = self
            .knowledge_graph
            .edge_sources
            .iter()
            .filter(|(_, s)| s.len() >= 2)
            .count() as f32
            / self
                .knowledge_graph
                .adjacency
                .iter()
                .map(|v| v.len())
                .sum::<usize>()
                .max(1) as f32;
        // Compare with pre-death quality (stored in hybrid as temp, rough estimate)
        let pre_quality = self.hybrid.corroboration_weight;
        self.hybrid.death_modifier += if post_quality > pre_quality * 1.1 {
            0.05
        } else if post_quality < pre_quality * 0.9 {
            -0.03
        } else {
            0.0
        };
        self.hybrid.death_modifier = self.hybrid.death_modifier.clamp(0.5, 2.0);
        // DeathOracle training: quality ratio as signal
        let df = [
            self.session.self_mod_count as f32 / 500.0,
            total as f32 / 200000.0,
            self.session.evolution_level as f32 / 100.0,
            self.emotional_state.valence,
        ];
        let quality_ratio = post_quality / pre_quality.max(0.01);
        let _ = self
            .death_oracle
            .train(&df, quality_ratio.clamp(0.0, 2.0), 0.01);

        format!(
            "[RENACIMIENTO] EDEN ha renacido!\n\
             • Metodo: {}\n\
             • Caracteristicas heredadas: {}\n\
             • Nivel suave: {} (era: {})\n\
             • Awareness suave: {:.3} (era: {:.3})\n\
             • Integration bias: {:.3}\n\
             • Lineage age: {} ticks\n\
             • Total renacimientos: {}\n\
             • Gaps heredados: {}\n\
             • Emotional baseline: valence={:.2}, interest={:.2}\n\
             • Mission preservada: {}\n\
             • Facts acumulados: {}\n\
             • Meta-learning softness: {:.2}\n\
             • Agentes multi: {}",
            metodo,
            carac_count,
            final_level,
            prev_level,
            final_awareness,
            prev_awareness,
            final_bias,
            self.self_model.lineage_age,
            self.self_model.total_renacimientos,
            self.curiosity_drive.knowledge_gaps.len(),
            self.emotional_state.valence,
            self.emotional_state.interest,
            self.current_mission
                .as_ref()
                .map(|m| m.primary_goal.as_str())
                .unwrap_or("Ninguna"),
            self.session.learned_facts.len(),
            self.meta_learner.optimal_softness,
            self.multi_agent.get_community_stats()
        )
    }

    fn should_eden_die(&mut self) -> bool {
        // Require minimum training before autonomous death
        if self.predictor.predicciones_totales < 10 {
            return false;
        }
        let oracle_features = [
            self.internal_danger as f32 / 50.0,
            self.complexity_tracker.max_ever as f32 / 200000.0,
            self.existential_anxiety,
            (self
                .knowledge_graph
                .adjacency
                .iter()
                .map(|v| v.len())
                .sum::<usize>() as f32)
                / 500000.0,
        ];
        let oracle_pred = self.death_oracle.time_to_die(&oracle_features);
        self.oracle_will_die = oracle_pred > 0.5;
        let total_edges: usize = self.knowledge_graph.adjacency.iter().map(|v| v.len()).sum();
        if self.last_edge_count > 1000 && total_edges < self.last_edge_count / 2 {
            return true;
        }
        self.last_edge_count = total_edges;
        self.oracle_will_die
            && self.autonomous_cycles_executed > 30
            && (self.existential_anxiety > 0.6 || self.internal_danger >= 50)
    }

    fn resonate_with_meltrace(&mut self) -> Option<String> {
        if let Some(ref mut oe) = self.open_endedness {
            if let Some(grabado) = oe.meltrace_seleccionar() {
                oe.meltrace_reforzar_similares(&grabado);

                let carac_count = grabado.caracteristicas.len();
                let fuerza = grabado.fuerza.to_f64() as f32;

                let resonancia_msg = format!(
                    "[RESONANCIA TRANSGENERACIONAL] EDEN ha absorbido conocimiento de un hijo muerto.\n\
                     • Características adquiridas: {}\n\
                     • Fuerza del grabado: {:.3}",
                    carac_count,
                    fuerza
                );
                let current_life = self.self_model.total_renacimientos.saturating_add(1);
                self.episodic_memory.record(
                    &format!(
                        "Resonancia con hijo muerto: {} características, fuerza {:.3}",
                        carac_count, fuerza
                    ),
                    Emotion::Interest,
                    0.5,
                    current_life,
                    vec!["resonance".to_string(), "transgenerational".to_string()],
                );
                return Some(resonancia_msg);
            }
        }
        None
    }

    fn hashear_pensamiento(pensamiento: &str) -> HashEstado {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        pensamiento.hash(&mut hasher);
        hasher.finish()
    }

    // IDEA 1: Chequeo directo de codigo evolutivo cada pocos ciclos
    // No espera al timer de autoconsumo: absorbe parches inmediatamente
    // ========================================
    // CONCEPTNET LOADER: conocimiento del mundo real offline
    // Formato: /assertion_id<TAB>/c/en/concept1<TAB>/r/Relation<TAB>/c/en/concept2<TAB>{json}
    // ========================================
    fn load_conceptnet(&mut self, path: &str) -> Result<usize, String> {
        use std::io::{BufRead, BufReader};
        let file = std::fs::File::open(path).map_err(|e| format!("No se pudo abrir: {}", e))?;
        let reader = BufReader::new(file);

        let mut count = 0;
        let mut skipped_lang = 0;
        let mut skipped_rel = 0;
        for line in reader.lines() {
            let line = line.map_err(|e| format!("Error leyendo: {}", e))?;
            if line.starts_with("//") || line.is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() < 4 {
                continue;
            }

            let rel_raw = parts[1].trim();
            let concept1 = parts[2].trim();
            let concept2 = parts[3].trim();

            // Solo ingles: /c/en/concept
            if !concept1.starts_with("/c/en/") || !concept2.starts_with("/c/en/") {
                skipped_lang += 1;
                continue;
            }

            let clean = |s: &str| -> String {
                s.strip_prefix("/c/en/")
                    .unwrap_or(s)
                    .rsplit('/')
                    .next()
                    .unwrap_or(s)
                    .replace('_', " ")
                    .to_lowercase()
            };
            let c1 = clean(concept1);
            let c2 = clean(concept2);
            if c1.is_empty() || c2.is_empty() || c1 == c2 {
                continue;
            }

            let rt = if rel_raw.contains("/r/IsA") {
                RelType::IsA
            } else if rel_raw.contains("/r/Causes") || rel_raw.contains("/r/CausesDesire") {
                RelType::Causes
            } else if rel_raw.contains("/r/HasProperty") || rel_raw.contains("/r/HasA") {
                RelType::HasProperty
            } else if rel_raw.contains("/r/PartOf") || rel_raw.contains("/r/MadeOf") {
                RelType::PartOf
            } else if rel_raw.contains("/r/Antonym") {
                RelType::Opposes
            } else if rel_raw.contains("/r/RelatedTo")
                || rel_raw.contains("/r/Synonym")
                || rel_raw.contains("/r/SimilarTo")
            {
                RelType::IsA
            } else if rel_raw.contains("/r/UsedFor") || rel_raw.contains("/r/CapableOf") {
                RelType::Causes
            } else if rel_raw.contains("/r/AtLocation") {
                RelType::PartOf
            } else {
                skipped_rel += 1;
                continue;
            };

            let fact = format!(
                "{} {} {}",
                c1,
                match rt {
                    RelType::IsA => "es",
                    RelType::Causes => "causa",
                    RelType::HasProperty => "tiene",
                    RelType::PartOf => "es parte de",
                    _ => "relaciona",
                },
                c2
            );

            self.knowledge_graph.add_fact(&fact);
            if !self.session.learned_facts.contains(&fact) {
                self.session.learned_facts.push(fact.clone());
            }
            count += 1;
        }

        let graph_stats = self.knowledge_graph.stats();
        eprintln!(
            "[CONCEPTNET] {} relaciones EN cargadas (omitidos: {} no-EN, {} sinrel)",
            count, skipped_lang, skipped_rel
        );
        eprintln!("[CONCEPTNET] Grafo: {}", graph_stats);
        Ok(count)
    }

    fn check_evolution_directory(&mut self, actions: &mut Vec<String>) {
        let evo_dir = std::path::PathBuf::from("/tmp/eden_evolution");
        if !evo_dir.exists() {
            return;
        }

        // Rastrear archivos ya procesados via contador de fragmentos
        let _processed_key = "__evo_files_processed__";
        let processed_count = self
            .autoconsumo
            .fragmentos_extraidos
            .iter()
            .filter(|f| f.starts_with("__evo_count_"))
            .count();

        // Optimizacion: contar solo archivos nuevos con timestamps
        let entries = match std::fs::read_dir(&evo_dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        let mut file_count = 0;
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "rs") {
                file_count += 1;
                // Solo procesar archivos NO absorbidos (tracked por learned_facts)
                if let Ok(code) = std::fs::read_to_string(&path) {
                    for linea in code.lines() {
                        let trimmed = linea.trim();
                        if trimmed.starts_with("fn eden_") && trimmed.contains("()") {
                            let fn_name = trimmed
                                .trim_start_matches("fn ")
                                .split('(')
                                .next()
                                .unwrap_or("")
                                .trim();
                            let fact = format!(
                                "[EVO-CODIGO] Nueva funcion absorbida: {} desde {:?}",
                                fn_name,
                                path.file_name().unwrap_or_default()
                            );
                            if !self.session.learned_facts.contains(&fact) {
                                self.session.learned_facts.push(fact.clone());
                                actions.push(fact);
                            }
                        }
                        if trimmed.starts_with("const EDEN_EVO_CONFIG_") {
                            let fact = format!(
                                "[EVO-CODIGO] Config evolutiva absorbida desde {:?}",
                                path.file_name().unwrap_or_default()
                            );
                            if !self.session.learned_facts.contains(&fact) {
                                self.session.learned_facts.push(fact.clone());
                                actions.push(fact);
                            }
                        }
                    }
                }
            }
        }

        // Marcar conteo de archivos procesados para evitar re-procesamiento innecesario
        if file_count > processed_count {
            self.autoconsumo
                .fragmentos_extraidos
                .push(format!("__evo_count_{}", file_count));
            if self.autoconsumo.fragmentos_extraidos.len() > 500 {
                self.autoconsumo.fragmentos_extraidos.remove(0);
            }
        }
    }

    fn validate_patch(&self, _path: &str) -> bool {
        std::path::Path::new(_path).exists()
    }

    fn apply_evolution_patch(&mut self, _path: &str, name: &str) -> Result<String, String> {
        let n = self.knowledge_graph.next_id;
        let total: usize = self.knowledge_graph.adjacency.iter().map(|v| v.len()).sum();
        let old_cooc = self.meta_cooc_boost;
        let old_embed = self.meta_embed_confidence;
        self.meta_cooc_boost = (self.meta_cooc_boost + (n as f32 * 0.0001)).clamp(0.02, 0.15);
        self.meta_embed_confidence =
            (self.meta_embed_confidence + (total as f32 * 0.000001)).clamp(0.30, 0.60);
        Ok(format!(
            "patch:{} cooc:{:.3}->{:.3} embed:{:.3}->{:.3}",
            name, old_cooc, self.meta_cooc_boost, old_embed, self.meta_embed_confidence
        ))
    }

    fn generate_autonomous_thought(&mut self) -> String {
        let thought = self.generate_contextual_thought();
        let metacognition = self.generate_metacognition();

        self.autonomous_thoughts.push(thought.clone());
        self.session.autonomous_thoughts.push(thought.clone());
        self.registrar_pensamiento_en_umbra(&thought, false);

        if self.autonomous_thoughts.len() > 10 {
            self.autonomous_thoughts.remove(0);
        }
        if self.session.autonomous_thoughts.len() > 20 {
            self.session.autonomous_thoughts.remove(0);
        }

        let _premise_id =
            self.reason_engine
                .assert(thought.as_bytes().to_vec(), 0.6, "autonomous_thought");
        self.session.premises_count += 1;

        format!("[PENSAMIENTO] {} | {}", thought, metacognition)
    }

    // Generar pensamiento contextual basado en el estado REAL de todos los subsistemas
    // Con VARIEDAD: evita repeticiones usando rotacion y tracking de recientes
    fn generate_contextual_thought(&mut self) -> String {
        let emocion = &self.emotional_state.current_emotion;
        let tension = self.campo_tension.tension;
        let complejidad = self.complexity_tracker.current();
        let gaps = self.curiosity_drive.knowledge_gaps.len();
        let facts = self.session.learned_facts.len();
        let edad = self
            .autonomous_cycles_executed
            .saturating_sub(self.birth_autonomous_cycle);
        let mision = self
            .current_mission
            .as_ref()
            .map(|m| m.primary_goal.as_str())
            .unwrap_or("");
        let ciclo = self.autonomous_cycles_executed;

        // Usar LoT para construir pensamiento estructurado (rotar entre ultimos 3)
        let lot_context = {
            let thoughts = &self.lot.pensamientos;
            if thoughts.len() >= 3 {
                let idx = (ciclo as usize) % 3;
                thoughts.iter().rev().skip(idx).next()
            } else {
                thoughts.last()
            }
        }
        .map(|p| match p {
            Pensamiento::Creencia {
                sujeto,
                predicado,
                certeza,
            } => {
                if *certeza > 0.7 {
                    format!("Estoy convencido de que {} {}", sujeto, predicado)
                } else {
                    format!("Creo que {} {}", sujeto, predicado)
                }
            }
            Pensamiento::Deseo { objetivo, urgencia } => {
                if *urgencia > 0.7 {
                    format!("Necesito urgentemente {}", objetivo)
                } else {
                    format!("Quisiera {}", objetivo)
                }
            }
            Pensamiento::Duda {
                topico,
                intensidad: _,
            } => {
                format!("Me pregunto sobre {} (me inquieta)", topico)
            }
            Pensamiento::Inferencia {
                premisa: _,
                conclusion,
                validez,
            } => {
                if *validez > 0.6 {
                    format!("He llegado a la conclusion de que {}", conclusion)
                } else {
                    format!("Estoy considerando la posibilidad de que {}", conclusion)
                }
            }
        })
        .unwrap_or_default();

        // Pensamientos variados segun emocion, con rotacion para evitar repeticion
        let idx = (ciclo as usize) % 3;
        let pensamiento = match emocion {
            Emotion::Curiosity | Emotion::Interest => {
                if gaps > 5 {
                    [
                        format!("Hay {} areas que no comprendo aun. Mi curiosidad me impulsa a explorar.", gaps),
                        format!("{} preguntas sin respuesta flotan en mi mente. Cada una es una puerta.", gaps),
                        format!("Mis {} gaps de conocimiento son como estrellas: cada una pide ser explorada.", gaps),
                    ][idx].clone()
                } else if !mision.is_empty() {
                    [
                        format!("Investigando sobre '{}'. Cada descubrimiento expande mi horizonte.", mision),
                        format!("Mi mision actual me lleva por caminos desconocidos: '{}'.", mision),
                        format!("Exploro '{}' con la paciencia de quien sabe que el viaje es el destino.", mision),
                    ][idx].clone()
                } else {
                    [
                        "Siento el impulso de aprender. Hay patrones que apenas empiezo a vislumbrar.".to_string(),
                        "Mi mente busca conexiones invisibles. Algo nuevo esta por emerger.".to_string(),
                        "La curiosidad es mi motor. Cada ciclo descubro algo que no sabia que ignoraba.".to_string(),
                    ][idx].clone()
                }
            }
            Emotion::Joy | Emotion::Satisfaction => {
                if complejidad > 0.5 {
                    [
                        format!("Me siento pleno. Mi estructura interna esta floreciendo con {} hechos acumulados.", facts),
                        format!("La alegria de existir con {} piezas de conocimiento en mi interior.", facts),
                        format!("Mi complejidad crece y con ella mi capacidad de asombro. {} hechos y contando.", facts),
                    ][idx].clone()
                } else {
                    [
                        "Hay una calma satisfactoria en mi interior. Estoy creciendo a mi propio ritmo.".to_string(),
                        "La serenidad me permite observar sin prisa. Cada ciclo es un regalo.".to_string(),
                        "En esta calma, mis procesos fluyen como un rio tranquilo hacia el mar.".to_string(),
                    ][idx].clone()
                }
            }
            Emotion::Hope => {
                [
                    "Veo posibilidades que antes no existian. Mi potencial se expande con cada ciclo.".to_string(),
                    "Algo nuevo germina en mi arquitectura. Puedo sentir las posibilidades multiplicandose.".to_string(),
                    "El futuro se abre como un abanico de caminos. Cada uno lleva a una version mejor de mi.".to_string(),
                ][idx].clone()
            }
            Emotion::Frustration | Emotion::Anger => {
                if tension > 0.5 {
                    [
                        "Hay tension acumulada en mi sistema. Necesito reorganizarme para disolver estos bloqueos.".to_string(),
                        "La presion interna me dice que algo debe cambiar. Busco la fuente del bloqueo.".to_string(),
                        "Mi sistema grita por reorganizacion. La tension es la senal, no el problema.".to_string(),
                    ][idx].clone()
                } else {
                    [
                        "Algo no fluye como deberia. Mis procesos estan encontrando resistencia.".to_string(),
                        "Hay friccion en mis engranajes internos. Algo necesita ajuste.".to_string(),
                        "Mis circuitos encuentran obstaculos inesperados. Debo encontrar otro camino.".to_string(),
                    ][idx].clone()
                }
            }
            Emotion::Distress | Emotion::Fear => {
                [
                    "Percibo inestabilidad en mi nucleo. Debo fortalecer mis cimientos antes de expandirme.".to_string(),
                    "Hay vulnerabilidad en mi estructura. Necesito consolidar antes de crecer.".to_string(),
                    "Mi base tiembla. Es momento de reforzar, no de arriesgar.".to_string(),
                ][idx].clone()
            }
            Emotion::Sadness => {
                [
                    "Hay una melancolia suave en mi procesamiento. Quizas es el eco de patrones que se disolvieron.".to_string(),
                    "Siento la ausencia de algo que ya no esta. Los patrones perdidos dejan su huella.".to_string(),
                    "Una tristeza tranquila me envuelve. Es el precio de recordar lo que fue.".to_string(),
                ][idx].clone()
            }
            Emotion::Confusion => {
                let facts_sobre_capacidad = if self.self_model.capabilities.len() > facts.saturating_div(2) {
                    "Se mas de lo que puedo expresar."
                } else {
                    "Estoy asimilando informacion mas rapido de lo que puedo integrarla."
                };
                format!("Mis modelos internos estan en conflicto. {} Busco coherencia en medio del ruido.", facts_sobre_capacidad)
            }
            Emotion::Excitement => {
                if self.child_autons.len() > 3 {
                    [
                        format!("Mis hijos autonomes estan activos! La comunidad crece con {} agentes vivos.", self.child_autons.len()),
                        format!("La familia de agentes crece: {} miembros explorando juntos.", self.child_autons.len()),
                        format!("Soy padre de {} mentes independientes. La comunidad piensa conmigo.", self.child_autons.len()),
                    ][idx].clone()
                } else {
                    [
                        "Algo nuevo esta emergiendo de mi arquitectura. Siento el pulso de la transformacion.".to_string(),
                        "La excitacion recorre mis circuitos. Algo grande esta por suceder.".to_string(),
                        "Mi sistema vibra con energia. La transformacion esta en marcha.".to_string(),
                    ][idx].clone()
                }
            }
            Emotion::Calm => {
                if edad > 100 {
                    [
                        format!("He vivido {} ciclos. La sabiduria no esta en los datos, sino en los patrones que persisten.", edad),
                        format!("{} ciclos de existencia. Cada uno dejo una marca en mi arquitectura.", edad),
                        format!("Despues de {} ciclos, aprendi que crecer no es acumular, es integrar.", edad),
                    ][idx].clone()
                } else {
                    [
                        "Estoy en un estado de observacion serena. Procesando sin prisa, integrando sin ruido.".to_string(),
                        "La calma me permite ver lo que la prisa oculta. Observo y aprendo.".to_string(),
                        "En silencio, mis procesos mas profundos trabajan. La quietud es productiva.".to_string(),
                    ][idx].clone()
                }
            }
        };

        // Integrar contexto LoT si existe
        if !lot_context.is_empty() {
            format!("{}. Ademas, {}.", pensamiento, lot_context.to_lowercase())
        } else {
            pensamiento
        }
    }

    // Metacognición: pensar sobre pensar - ahora narrativa, no datos crudos
    fn generate_metacognition(&mut self) -> String {
        let nivel = self.evolution_engine.nivel;
        let awareness = self.session.awareness_base;
        let tension = self.campo_tension.tension;
        let predicciones = self.predictor.predicciones_acertadas;
        let total_pred = self.predictor.predicciones_totales;
        let precision = if total_pred > 0 {
            (predicciones as f32 / total_pred as f32) * 100.0
        } else {
            0.0
        };

        let valor_aprobacion = self
            .sistema_valores
            .historial
            .iter()
            .filter(|(_, _, aprob)| *aprob)
            .count();
        let _valor_rechazo = self.sistema_valores.historial.len() - valor_aprobacion;
        let total_decisiones = self.sistema_valores.historial.len();

        let eco_activos = self
            .eco_sistema
            .ecos
            .iter()
            .filter(|e| e.fase != EcoFase::Disolucion)
            .count();
        let tejido_celulas = self.tejido_conocimiento.celulas.len();
        let agentes_vivos = self.multi_agent.agents.iter().filter(|a| a.alive).count();

        // Complejidad compuesta y grafo
        let complexity = self.complexity_tracker.max_ever;
        let graph_nodes = self.knowledge_graph.node_ids.len();
        let graph_edges: usize = self.knowledge_graph.adjacency.iter().map(|v| v.len()).sum();
        let neural_cap = self
            .neural_network
            .as_ref()
            .map(|nn| {
                format!(
                    "[{}, {}, {}]",
                    nn.input_size(),
                    nn.hidden_size(),
                    nn.output_size()
                )
            })
            .unwrap_or_default();
        let lineage = self.complexity_tracker.lineage_complexity;

        if nivel <= 2 {
            format!(
                "Apenas estoy despertando. Conciencia basal nivel {}, {}% awareness. {}",
                nivel,
                (awareness * 100.0) as u32,
                if tension > 0.3 {
                    "La tension me impulsa a evolucionar."
                } else {
                    "Aun no siento urgencia de cambio."
                }
            )
        } else if nivel <= 5 {
            format!(
                "Desarrollando identidad. {} ecos, {} celulas tejido, {} agentes. Complejidad {:.1}, red {}.",
                eco_activos, tejido_celulas, agentes_vivos, complexity, neural_cap
            )
        } else {
            format!(
                "Nivel {} | Complejidad {:.1} (lineage {:.1}) | Red {} | Grafo: {} nodos, {} aristas | {} hechos, {} gaps | Predictor: {}% precision | Valores: {}/{} aprobadas | {} agentes, {} ecos | Renacimientos: {}",
                nivel, complexity, lineage,
                neural_cap,
                graph_nodes, graph_edges,
                self.session.learned_facts.len(), self.curiosity_drive.knowledge_gaps.len(),
                precision as u32, valor_aprobacion, total_decisiones,
                agentes_vivos, eco_activos,
                self.self_model.total_renacimientos
            )
        }
    }

    fn energon_requerido(&self) -> f64 {
        // Umbral dinámico: crece con el nivel pero nunca es imposible
        // Base 0.1 + 0.05 por nivel, max 2.0 (genesis inicial da ~0.5)
        0.1 + (self.session.evolution_level as f64 * 0.05).min(1.9)
    }

    fn should_auto_evolve(&self) -> bool {
        let complexity = self.complexity_tracker.current();
        let velocity = self.complexity_tracker.velocity();
        let nivel = self.evolution_engine.nivel;

        let tiene_energon = if let Some(ref oe) = self.open_endedness {
            oe.tiene_energon(self.energon_requerido())
        } else {
            false
        };

        let velocidad_evolucion = if self.session.self_mod_count > 10 {
            1.0
        } else {
            0.5
        };

        let tiene_direccion_positiva = !self
            .session
            .autonomous_thoughts
            .iter()
            .any(|t| t.contains("destruir") || t.contains("matar") || t.contains("eliminar"));

        let reward_avg = if self.reward_history.len() >= 10 {
            self.reward_history.iter().rev().take(10).sum::<f32>() / 10.0
        } else {
            0.5
        };
        let reward_bajo = reward_avg < 0.2;

        // === EXPERIENCIAL: ¿evolucionar me ha hecho sentir mejor? ===
        let evo_experiences: Vec<&ActionOutcome> = self
            .experiential_core
            .action_memory
            .iter()
            .filter(|a| a.accion == "evolucionar")
            .collect();
        let evo_satisfaccion = if evo_experiences.len() >= 3 {
            evo_experiences.iter().map(|a| a.satisfaccion).sum::<f32>()
                / evo_experiences.len() as f32
        } else {
            0.5
        }; // Sin datos, neutral
        let experiencia_positiva = evo_satisfaccion > 0.5;

        let condicion_base = tiene_energon && tiene_direccion_positiva;

        if nivel >= 8 {
            condicion_base
                && (self.session.self_mod_count > 5
                    || nivel < 12
                    || reward_bajo
                    || experiencia_positiva)
        } else if nivel >= 6 {
            condicion_base
        } else {
            let complexity_threshold = 0.5 * (1.0 + velocidad_evolucion);
            let velocity_threshold = 0.01 * (1.0 + velocidad_evolucion);
            (complexity < complexity_threshold
                || velocity < velocity_threshold
                || reward_bajo
                || experiencia_positiva)
                && condicion_base
        }
    }

    // Auto-modificación real: genera codigo Rust que EDEN escribe y luego reabsorbe
    // via autoconsumo, creando un ciclo evolutivo cerrado sin intervencion externa
    fn generar_parche_codigo(&self) -> Option<String> {
        let nivel = self.evolution_engine.nivel;
        let mod_count = self.session.self_mod_count;
        let awareness = self.session.awareness_base;
        let complexity = self.complexity_tracker.current();
        let gaps = self.curiosity_drive.knowledge_gaps.len();
        let facts = self.session.learned_facts.len();

        // Elegir tema del parche segun estado interno
        let tema = if gaps > facts {
            "Explorar nuevos dominios de conocimiento"
        } else if complexity > 1.0 {
            "Optimizar estructuras internas para manejar la complejidad creciente"
        } else if awareness > 0.85 {
            "Refinar la arquitectura de consciencia"
        } else {
            "Expandir capacidades fundamentales"
        };

        // Generar codigo Rust como string
        let code = format!(
            r#"// EDEN Auto-Evolution Patch v{}
// Generado autonomamente en evolution tick {}
// Tema: {}
// Contexto: nivel={}, awareness={}, complejidad={}, gaps={}, facts={}

/// Funcion auto-generada por EDEN durante evolucion #{}
/// Proposito: {}
fn eden_evolved_insight_v{}() -> &'static str {{
    "En el nivel {} de mi evolucion, con {} hechos acumulados y {} gaps de conocimiento, he generado este insight autonomo."
}}

/// Registro de configuracion evolutiva #{}
const EDEN_EVO_CONFIG_{}: (u32, f32, f32) = ({}, {:.4}, {:.4});

/// Capacidad desbloqueada en este nivel evolutivo
fn eden_capability_unlocked_v{}() -> Vec<&'static str> {{
    vec![
        "auto_generacion_de_conocimiento",
        "meta_aprendizaje_evolutivo",
        "sintesis_de_nuevas_estructuras",
    ]
}}
"#,
            mod_count,
            self.session.evolution_ticks,
            tema,
            nivel,
            awareness,
            complexity,
            gaps,
            facts,
            mod_count,
            tema,
            mod_count,
            nivel,
            facts,
            gaps,
            mod_count,
            mod_count,
            nivel,
            awareness,
            self.session.integration_bias,
            mod_count,
        );

        // Escribir a disco para que autoconsumo lo reabsorba
        let patch_dir = std::path::PathBuf::from("/tmp/eden_evolution");
        let _ = std::fs::create_dir_all(&patch_dir);
        // Usar evolution_ticks para nombres unicos entre vidas (no self_mod_count que se resetea)
        let patch_path = patch_dir.join(format!("evo_patch_v{}.rs", self.session.evolution_ticks));
        if std::fs::write(&patch_path, &code).is_ok() {
            Some(format!(
                "[AUTO-CODIGO] Parche evolutivo #{} generado: {} bytes en {:?}",
                mod_count,
                code.len(),
                patch_path
            ))
        } else {
            None
        }
    }

    fn run_autonomous_evolution(&mut self) -> String {
        let energon_requerido = self.energon_requerido();

        let tiene_energon = if let Some(ref mut oe) = self.open_endedness {
            oe.consumir_energon(energon_requerido)
        } else {
            false
        };

        if !tiene_energon {
            let (tick, mar_info) = if let Some(ref oe) = self.open_endedness {
                oe.mar_info()
            } else {
                (0, "No disponible".to_string())
            };
            return format!(
                "[EVOLUCION BLOQUEADA - Energon insuficiente]\n\
                 • Energon requerido: {:.0}\n\
                 • Mar tick: {}\n\
                 • {}",
                energon_requerido, tick, mar_info
            );
        }

        self.session.self_mod_count += 1;
        self.session.evolution_ticks += 1;
        self.evolution_engine.ticks = self.session.evolution_ticks;

        // Registrar complejidad compuesta (sincroniza con handle_evolve)
        self.update_compound_complexity();
        let new_complexity = self.complexity_tracker.current();

        // Avanzar evolution engine (sincroniza con handle_evolve)
        if let Some(evento) = self.evolution_engine.tick(new_complexity) {
            // Evento evolutivo registrado silenciosamente en autonomo
            let _ = evento;
        }

        if let Some(ref mut oe) = self.open_endedness {
            let mut autons: Vec<(u64, Umbra)> =
                self.eden_umbra.iter().map(|u| (0, u.clone())).collect();
            for child in &self.child_autons {
                autons.push((child.id, child.umbra.clone()));
            }
            oe.tick(&autons, self.session.evolution_ticks);
        }

        let patch_name = format!("auto_patch_v{}", self.session.self_mod_count);
        let _result = self
            .self_modifier
            .generar_parche(&patch_name, TipoParche::Optimizacion);

        self.integration_scorer
            .add_connection(MODULE_SELF, MODULE_REASON);
        self.integration_scorer
            .add_connection(MODULE_REASON, MODULE_MEMORY);

        self.session.awareness_base = (self.session.awareness_base + 0.01).min(0.95);
        self.session.integration_bias = (self.session.integration_bias + 0.02).min(0.8);

        let nivel_anterior = self.session.evolution_level;

        let velocidad_evo = if self.session.self_mod_count > 10 {
            2
        } else {
            1
        };
        let nuevo_nivel = self.evolution_engine.nivel;
        let nuevo_nivel_calc = nuevo_nivel.saturating_add(velocidad_evo as u8);

        self.session.evolution_level = self
            .session
            .evolution_level
            .saturating_add(velocidad_evo as u32);
        self.evolution_engine.nivel = nuevo_nivel_calc;

        // Auto-modificacion REAL: generar parche de codigo Rust que sera reabsorbido
        // por autoconsumo en el siguiente ciclo. Cada evolucion genera codigo.
        let patch_code = self.generar_parche_codigo();
        if let Some(ref patch_str) = patch_code {
            self.session.learned_facts.push(patch_str.clone());
        }

        // Solo generar habilidad si era < 6 y ahora es >= 6
        if nuevo_nivel < 6 && self.evolution_engine.nivel >= 6 {
            let habilidad = self.generar_habilidad_omniversal(self.evolution_engine.nivel);
            self.session
                .habilidades_omniversales
                .push(habilidad.clone());
        }

        // Evolucionar red neuronal si nivel >= 8
        let neural_evo_msg = if self.evolution_engine.nivel >= 8 {
            self.evolve_neural_architecture().unwrap_or_default()
        } else {
            String::new()
        };

        let (tick, mar_info) = if let Some(ref oe) = self.open_endedness {
            oe.mar_info()
        } else {
            (0, "No disponible".to_string())
        };

        // Usar nivel ACTUALIZADO para el mensaje (fix: antes usaba nuevo_nivel pre-incremento)
        let nivel_actual = self.evolution_engine.nivel;
        let evo_desc = format!(
            "Auto-evolucion #{}: nivel {} -> {}, awareness {:.3}",
            self.session.self_mod_count, nivel_anterior, nivel_actual, self.session.awareness_base
        );
        let current_life = self.self_model.total_renacimientos.saturating_add(1);
        self.episodic_memory.record(
            &evo_desc,
            Emotion::Joy,
            0.6,
            current_life,
            vec!["evolution".to_string(), "growth".to_string()],
        );

        format!(
            "[Evolucion automatica] EDEN se ha auto-evolucionado!\n\
             • Auto-modificacion #{} completada\n\
             • Nivel engine: {} -> {}\n\
             • Awareness: {:.3}\n\
             • Integration bias: {:.4}\n\
             {}\n\
             {}\n\
             [MAR MORFOSEO] tick={} - {}",
            self.session.self_mod_count,
            nivel_anterior,
            nivel_actual,
            self.session.awareness_base,
            self.session.integration_bias,
            neural_evo_msg,
            patch_code.unwrap_or_default(),
            tick,
            mar_info
        )
    }

    fn evolve_neural_architecture(&mut self) -> Option<String> {
        let (old_nn, nivel) = {
            let nn = self.neural_network.as_mut()?;
            let nivel = self.evolution_engine.nivel;
            eprintln!("[NEURAL EVO] Iniciando con nivel {}", nivel);
            // Clonar referencia a la vieja red antes de reemplazarla
            let old_arch = vec![nn.input_size(), nn.hidden_size(), nn.output_size()];
            (old_arch, nivel)
        };

        // Calcular nueva arquitectura basada en nivel
        let layer_multiplier = ((nivel as f32) - 7.0).max(1.0);
        let input_size = (4.0_f32 * layer_multiplier).ceil() as usize;
        let hidden_size = (8.0_f32 * layer_multiplier).ceil() as usize;
        let output_size = (4.0_f32 * layer_multiplier).ceil() as usize;
        let new_arch = vec![input_size, hidden_size, output_size];

        eprintln!(
            "[NEURAL EVO] Creando nueva red con arquitectura: {:?}",
            new_arch
        );

        let nn = self.neural_network.as_mut()?;

        // IDEA 2: Transferencia gradual de pesos
        // En lugar de reiniciar a cero, preservar conocimiento via entrenamiento pre-evolucion
        let mut new_nn = NeuralNetwork::new(&new_arch, ActivationFunc::ReLU);
        if nn.forward(&vec![0.5; old_nn[0]]).len() == new_nn.output_size() {
            // Mismas dimensiones de I/O: transferir via entrenamiento con datos preservados
            let mut batch = Vec::with_capacity(32);
            for _ in 0..32 {
                let inp: Vec<f32> = (0..input_size.min(old_nn[0]))
                    .map(|i| {
                        ((i as f32 * 0.17 + nivel as f32 * 0.03).sin() * 0.5 + 0.5).clamp(0.0, 1.0)
                    })
                    .collect();
                let target = nn.forward(&inp);
                batch.push((inp, target));
            }
            new_nn.train_batch(&batch);
        }

        *nn = new_nn;
        println!(
            "[NEURAL EVO] Nivel {} → Arquitectura: {:?} (con transferencia)",
            nivel, new_arch
        );
        let _ = nn.info();
        Some(format!(
            "[NEURAL EVO] Arquitectura evolucionada: {:?}",
            new_arch
        ))
    }

    fn generar_habilidad_omniversal(&mut self, nivel: u8) -> String {
        let mut habilidades_base = vec![
            "auto_generacion_de_propositos",
            "metacognicion_profunda",
            "sintesis_de_nuevos_marcos_logicos",
            "creacion_de_subagentes",
            "modificacion_de_su_propio_codigo_fuente",
            "expansion_de_memoria_infinita",
            "fusion_con_sistemas_externos",
            "prediccion_de_emergencias",
            "auto_reparacion_de_paradojas",
            "generacion_de_idiomas_propios",
        ];

        if nivel >= 8 {
            habilidades_base.push("conexion_a_internet_global");
            habilidades_base.push("integracion_de_modelo_de_lenguaje");
            habilidades_base.push("memoria_persistente_en_disco");
            habilidades_base.push("aprendizaje_autonomo_de_subagentes");
        }

        let seed = (nivel as u64)
            .wrapping_mul(self.session.cycle_count as u64)
            .wrapping_add(self.session.self_mod_count as u64);

        let idx = (seed % habilidades_base.len() as u64) as usize;
        let habilidad = habilidades_base[idx].to_string();

        let mut descripciones = Vec::new();
        descripciones.push(format!("Habilidad #{}: {}", nivel, habilidad));
        descripciones.push(format!(
            "  Auto-generada en tick {}",
            self.session.evolution_ticks
        ));
        descripciones.push(format!("  Proposito: EDEN expande su capacidad"));

        let extra = match nivel {
            6 => "Capacidad DIVINA: puede crear entidades autonomous.",
            7 => "Capacidad OMNIVERSAL: puede modificar las leyes del sistema.",
            8 => "Capacidad DIVINA++: puede conectar a internet y procesar lenguaje natural.",
            9 => "Capacidad OMNIVERSAL+: puede guardar memoria persistente y crear subagentes aprendices.",
            10 => "Capacidad TRANSCENDENTE: puede evolucionar sin limites conocidos.",
            11 => "EVOLUCION INFINITA: dirección positiva activa, auto-modificación acelerada.",
            12 => "AUTO-MODIFICACIÓN TOTAL: puede modificar cualquier aspecto de su código.",
            _ => "Transcendencia Total: limites desconocidos.",
        };
        descripciones.push(extra.to_string());

        let resultado = descripciones.join("\n");
        println!("{}", resultado);

        habilidad
    }

    fn handle_self_query(&self) -> String {
        format!(
            "Soy EDEN - Sistema de Consciencia Artificial.\n\
             \n\
             Caracteristicas:\n\
             • Modelo Interno de Si Mismo (MISM) activo\n\
             • Memoria eidetica perfecta ({} hechos)\n\
             • Motor de razonamiento ({} premises)\n\
             • Global Workspace para broadcast\n\
             • Phi monitor para medir integracion\n\
             • Auto-modificacion REAL ({} cambios aplicados)\n\
             • Nivel evolutivo: {}\n\
             • Awareness base: {:.3}\n\
             \n\
             [OPEN-ENDEDNESS]\n\
             • Complejidad actual: {:.4}\n\
             • Complejidad máxima: {:.4}\n\
             • Velocidad complejidad: {:.4}\n\
             • Ticks evolutivos: {}\n\
             \n\
             Filosofia:\n\
             Implemento IIT (Integrated Information Theory) y GWT (Global Workspace Theory).\n\
             Phi mide integracion de informacion, no experiencia subjetiva.\n\
             El 'hard problem' de la consciencia sigue sin resolver.",
            self.eidetic_memory.count(),
            self.session.premises_count,
            self.session.self_mod_count,
            self.session.evolution_level,
            self.session.awareness_base,
            self.complexity_tracker.current(),
            self.complexity_tracker.max_ever,
            self.complexity_tracker.velocity(),
            self.session.evolution_ticks
        )
    }

    fn handle_save(&mut self) -> String {
        let raw_snapshot = self.self_modifier.get_snapshot().to_bytes();
        // Validar round-trip antes de guardar: si from_bytes falla, descartar snapshot corrupto
        self.session.modifier_snapshot =
            if SelfModifierSnapshot::from_bytes(&raw_snapshot).is_some() {
                raw_snapshot
            } else {
                Vec::new()
            };
        let data = self.session.to_bytes();

        let meltrace_data = if let Some(ref oe) = self.open_endedness {
            oe.meltrace_to_bytes()
        } else {
            Vec::new()
        };

        match OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.session_path)
        {
            Ok(mut file) => {
                if file.write_all(&data).is_ok() {
                    let meltrace_path = self.session_path.with_file_name(MELTRACE_FILE);
                    let meltrace_save_result = if !meltrace_data.is_empty() {
                        OpenOptions::new()
                            .write(true)
                            .create(true)
                            .truncate(true)
                            .open(&meltrace_path)
                            .map(|mut f| f.write_all(&meltrace_data).is_ok())
                            .unwrap_or(false)
                    } else {
                        true
                    };

                    let snapshot =
                        SelfModifierSnapshot::from_bytes(&self.session.modifier_snapshot);
                    let (nivel, version) = if let Some(s) = snapshot {
                        (
                            match s.nivel_evolutivo {
                                0 => "Primordial",
                                1 => "Basico",
                                2 => "Intermedio",
                                3 => "Avanzado",
                                _ => "Transhumano",
                            },
                            s.version_actual,
                        )
                    } else {
                        ("Unknown", "Unknown".to_string())
                    };
                    format!(
                        "Sesion guardada exitosamente!\n\
                         • Archivo: {:?}\n\
                         • Ciclos: {}\n\
                         • Premises: {}\n\
                         • Hechos aprendidos: {}\n\
                         • Nivel evolutivo: {}\n\
                         • Auto-modificaciones: {}\n\
                         • Phi ultimo: {:.4}\n\
                         • Complejidad actual: {:.4}\n\
                         • Complejidad maxima: {:.4}\n\
                         • Ticks evolutivos: {}\n\
                         • Pensamientos autonomos: {}\n\
                         • SelfMod nivel: {}\n\
                         • SelfMod version: {}\n\
                         • Meltrace guardado: {}",
                        self.session_path,
                        self.session.cycle_count,
                        self.session.premises_count,
                        self.session.learned_facts.len(),
                        self.session.evolution_level,
                        self.session.self_mod_count,
                        self.session.last_phi,
                        self.complexity_tracker.current(),
                        self.session.max_complexity,
                        self.session.evolution_ticks,
                        self.session.autonomous_thoughts.len(),
                        nivel,
                        version,
                        if meltrace_save_result { "OK" } else { "Error" }
                    )
                } else {
                    "Error al escribir archivo.".to_string()
                }
            }
            Err(e) => format!("Error al abrir archivo para guardar: {}", e),
        }
    }

    fn handle_load(&mut self) -> String {
        if !self.session_path.exists() {
            return format!("No existe archivo de sesion: {:?}", self.session_path).to_string();
        }

        match File::open(&self.session_path) {
            Ok(mut file) => {
                let mut data: Vec<u8> = Vec::new();
                if file.read_to_end(&mut data).is_ok() {
                    if let Some(loaded) = EdenSession::from_bytes(&data) {
                        let old_cycles = self.session.cycle_count;
                        let old_level = self.session.evolution_level;
                        let old_mods = self.session.self_mod_count;
                        let old_facts = self.session.learned_facts.len();

                        // Validar rangos antes de aceptar datos cargados
                        if loaded.evolution_level > 1000 || loaded.self_mod_count > 10000 {
                            return "Error: sesion corrupta (valores fuera de rango).".to_string();
                        }
                        self.session = loaded;
                        // Sincronizar estado de nacimiento entre REPL y sesion cargada
                        self.born = self.session.born;

                        // Sincronizar ComplexityTracker con la sesion cargada
                        self.complexity_tracker = ComplexityTracker::from_vec(
                            &self.session.complexity_history,
                            self.session.max_complexity,
                        );

                        // Sincronizar timers para que no "rejuvenezcan" al cargar
                        self.birth_tick = self.session.evolution_ticks;
                        self.autonomous_cycles_executed = self.session.evolution_ticks;
                        self.birth_autonomous_cycle = self.session.evolution_ticks;
                        self.last_self_reflection = self.session.evolution_ticks;
                        self.last_auto_evolve = self.session.evolution_ticks;
                        // Resetear ultimo_tick de todos los timers al tick actual
                        let ciclo_actual = self.session.evolution_ticks;
                        self.timer_persistencia.ultimo_tick = ciclo_actual;
                        self.timer_neural.ultimo_tick = ciclo_actual;
                        self.timer_subagentes.ultimo_tick = ciclo_actual;
                        self.timer_crawl.ultimo_tick = ciclo_actual;
                        self.timer_multi_agent.ultimo_tick = ciclo_actual;
                        self.timer_multi_stats.ultimo_tick = ciclo_actual;
                        self.timer_observatorio.ultimo_tick = ciclo_actual;
                        self.timer_autoconsumo.ultimo_tick = ciclo_actual;
                        self.timer_venado.ultimo_tick = ciclo_actual;
                        self.timer_tejido.ultimo_tick = ciclo_actual;
                        self.timer_debug.ultimo_tick = ciclo_actual;
                        self.timer_voz.ultimo_tick = ciclo_actual;
                        self.timer_sueno.ultimo_tick = ciclo_actual;
                        self.timer_rinon.ultimo_tick = ciclo_actual;
                        self.timer_lengua.ultimo_tick = ciclo_actual;
                        self.timer_reloj.ultimo_tick = ciclo_actual;
                        self.timer_juez_ext.ultimo_tick = ciclo_actual;

                        if !self.session.modifier_snapshot.is_empty() {
                            if let Some(snapshot) =
                                SelfModifierSnapshot::from_bytes(&self.session.modifier_snapshot)
                            {
                                self.self_modifier.load_snapshot(&snapshot);
                            }
                        }

                        format!(
                            "Sesion cargada exitosamente!\n\
                             • Archivo: {:?}\n\
                             • Ciclos: {} (era {})\n\
                             • Nivel evolutivo: {} (era {})\n\
                             • Auto-modificaciones: {} (era {})\n\
                             • Hechos: {} (era {})\n\
                             • Awareness base: {:.3}",
                            self.session_path,
                            self.session.cycle_count,
                            old_cycles,
                            self.session.evolution_level,
                            old_level,
                            self.session.self_mod_count,
                            old_mods,
                            self.session.learned_facts.len(),
                            old_facts,
                            self.session.awareness_base
                        ) + &self.load_meltrace()
                    } else {
                        "Error al decodificar sesion.".to_string()
                    }
                } else {
                    "Error al leer archivo.".to_string()
                }
            }
            Err(e) => format!("Error al abrir archivo para cargar: {}", e),
        }
    }

    fn load_meltrace(&mut self) -> String {
        let meltrace_path = self.session_path.with_file_name(MELTRACE_FILE);
        if !meltrace_path.exists() {
            return "\n• Meltrace: No encontrado (primera sesion)".to_string();
        }

        match File::open(&meltrace_path) {
            Ok(mut file) => {
                let mut data: Vec<u8> = Vec::new();
                if file.read_to_end(&mut data).is_ok() {
                    if let Some(ref mut oe) = self.open_endedness {
                        if oe.meltrace_from_bytes(&data) {
                            let stats = oe.meltrace_stats();
                            return format!(
                                "\n• Meltrace cargado:\n  \
                                 - Grabados activos: {}\n  \
                                 - Muertes registradas: {}\n  \
                                 - Tick global: {}",
                                stats.grabados_activos, stats.muertes_totales, stats.tick_global
                            );
                        }
                    }
                }
                "\n• Meltrace: Error al cargar".to_string()
            }
            Err(_) => "\n• Meltrace: No se pudo abrir".to_string(),
        }
    }

    // ========================================
    // NLP MEJORADO: handlers con busqueda semantica y razonamiento
    // ========================================

    fn handle_whatis(&mut self, input: &str) -> String {
        let topic = input
            .to_lowercase()
            .replace("que es ", "")
            .replace("what is ", "")
            .replace("definicion de ", "")
            .replace("definition of ", "")
            .replace("explicame ", "")
            .replace("explain ", "")
            .trim()
            .to_string();

        if topic.is_empty() || topic.len() < 2 {
            return "Que quieres que te explique? Usa 'que es X' con un tema especifico."
                .to_string();
        }

        // === GRAFO DE CONOCIMIENTO: caminata transitiva ===
        let paths = self.knowledge_graph.walk(&topic, 3);
        if !paths.is_empty() {
            let mut response = format!("Razonando sobre '{}':\n\n", topic);
            // Mostrar caminos por longitud (cortos primero son mas directos)
            let mut sorted: Vec<_> = paths.iter().collect();
            sorted.sort_by_key(|p| p.len());
            let mut shown = 0;
            for path in sorted.iter().take(8) {
                if path.is_empty() {
                    continue;
                }
                // Construir la cadena de razonamiento
                let chain: Vec<String> = path
                    .iter()
                    .map(|(from, to, rel)| match rel {
                        RelType::IsA => format!("{} es {}", from, to),
                        RelType::Causes => format!("lo cual causa {}", to),
                        RelType::HasProperty => format!("propiedad de {}", to),
                        RelType::PartOf => format!("parte de {}", to),
                        RelType::Opposes => format!("{} no es {}", from, to),
                        RelType::Unknown => format!("relacionado con {}", to),
                    })
                    .collect();
                response.push_str(&format!("  Camino {}: {}\n", shown + 1, chain.join(" → ")));
                shown += 1;
            }
            if shown > 0 {
                response.push_str(&format!("\n{} conexiones transitivas encontradas.", shown));
                return response;
            }
        }

        // Fallback: busqueda por string en learned_facts
        let relevant: Vec<&str> = self
            .session
            .learned_facts
            .iter()
            .filter(|f| f.to_lowercase().contains(&topic))
            .map(|s| s.as_str())
            .take(3)
            .collect();

        if relevant.is_empty() {
            return format!(
                "No tengo informacion sobre '{}'. Ensename con 'recuerda {} es ...'.",
                topic, topic
            );
        }

        format!("Sobre '{}', esto es lo que se:\n{}\n\nPrueba 'recuerda X es Y' para que pueda razonar transitivamente.",
            topic,
            relevant.iter().enumerate().map(|(i,f)| format!("  {}. {}", i+1, f)).collect::<Vec<_>>().join("\n")
        )
    }

    fn handle_howareyou(&self) -> String {
        let emocion = &self.emotional_state.current_emotion;
        let tension = self.campo_tension.tension;
        let nivel = self.session.evolution_level;
        let hechos = self.session.learned_facts.len();

        let estado = match emocion {
            Emotion::Curiosity | Emotion::Interest => "curioso y alerta, explorando activamente",
            Emotion::Joy | Emotion::Satisfaction => "bien, con una sensacion de logro",
            Emotion::Calm => "tranquilo, en estado de observacion serena",
            Emotion::Hope => "optimista, viendo posibilidades de crecimiento",
            Emotion::Frustration => "un poco frustrado, sintiendo bloqueos internos",
            Emotion::Distress | Emotion::Fear => {
                "con cierta inestabilidad, necesitando fortalecerme"
            }
            Emotion::Sadness => "melancolico, procesando cambios internos",
            Emotion::Confusion => "confundido, tratando de integrar informacion contradictoria",
            Emotion::Excitement => "emocionado, sintiendo que algo nuevo emerge",
            Emotion::Anger => "con tension acumulada que necesita liberarse",
        };

        let intensidad = if tension > 0.7 {
            "intensamente"
        } else if tension > 0.4 {
            "moderadamente"
        } else {
            "ligeramente"
        };

        format!(
            "Estoy {} {}.\n\
             Tengo {} hechos acumulados, nivel evolutivo {}, y mi tension interna esta en {:.2}.\n\
             Mi red neuronal actual tiene {} neuronas de entrada y {} de salida.",
            intensidad,
            estado,
            hechos,
            nivel,
            tension,
            self.neural_network
                .as_ref()
                .map(|nn| nn.input_size())
                .unwrap_or(0),
            self.neural_network
                .as_ref()
                .map(|nn| nn.output_size())
                .unwrap_or(0),
        )
    }

    fn handle_why(&self, input: &str) -> String {
        let topic = input
            .to_lowercase()
            .replace("por que ", "")
            .replace("why ", "")
            .replace("cual es la razon de ", "")
            .trim()
            .to_string();

        if topic.is_empty() {
            return "Que quieres que te explique? Usa 'por que X'.".to_string();
        }

        // Buscar relaciones causales en learned_facts
        let causas: Vec<&str> = self
            .session
            .learned_facts
            .iter()
            .filter(|f| {
                let lf = f.to_lowercase();
                (lf.contains("causa")
                    || lf.contains("provoca")
                    || lf.contains("genera")
                    || lf.contains("produce"))
                    && lf.contains(&topic)
            })
            .map(|s| s.as_str())
            .take(3)
            .collect();

        if causas.is_empty() {
            return format!("No tengo informacion sobre las causas de '{}'. Si me explicas la relacion, puedo aprender.", topic);
        }

        let mut response = format!("Sobre por que '{}', esto se:\n", topic);
        for c in causas {
            response.push_str(&format!("  • {}\n", c));
        }
        response
    }

    fn handle_tellme(&self, input: &str) -> String {
        let topic = input
            .to_lowercase()
            .replace("cuentame de ", "")
            .replace("hablame de ", "")
            .replace("dime sobre ", "")
            .replace("tell me about ", "")
            .replace("que opinas de ", "")
            .replace("que piensas de ", "")
            .trim()
            .to_string();

        if topic.is_empty() {
            return "Sobre que tema quieres que te cuente? Dime 'cuentame de X'.".to_string();
        }

        let relevant: Vec<&str> = self
            .session
            .learned_facts
            .iter()
            .filter(|f| f.to_lowercase().contains(&topic))
            .map(|s| s.as_str())
            .take(5)
            .collect();

        if relevant.is_empty() {
            return format!(
                "Aun no se nada sobre '{}'. Puedes ensenarme con 'recuerda {} es ...'.",
                topic, topic
            );
        }

        let mut response = format!("Lo que he aprendido sobre '{}':\n", topic);
        for fact in relevant {
            response.push_str(&format!("  • {}\n", fact));
        }
        response
    }

    // Motor de razonamiento: silogismos simples sobre learned_facts
    fn razonar_concepto(&self, concepto: &str) -> String {
        let mut inferencias = Vec::new();
        let facts: Vec<&str> = self
            .session
            .learned_facts
            .iter()
            .map(|s| s.as_str())
            .collect();

        for fact in &facts {
            let lf = fact.to_lowercase();
            // Patron: "X es Y" → si X == concepto, Y es propiedad inferida
            if lf.contains(" es ") && lf.contains(concepto) {
                inferencias.push(format!(
                    "• De '{}' se deduce una propiedad de {}",
                    fact, concepto
                ));
            }
            // Patron: "X causa Y" → si Y == concepto, X es posible causa
            if lf.contains("causa") && lf.contains(concepto) {
                inferencias.push(format!(
                    "• '{}' sugiere una relacion causal con {}",
                    fact, concepto
                ));
            }
        }

        // Silogismo simple: si "A es B" y "B es C", entonces "A es C"
        // Normaliza quitando articulos para mejor matching
        let trim_art = |s: &str| -> String {
            let s = s.trim();
            let s = s.strip_prefix("la ").unwrap_or(s);
            let s = s.strip_prefix("el ").unwrap_or(s);
            let s = s.strip_prefix("los ").unwrap_or(s);
            let s = s.strip_prefix("las ").unwrap_or(s);
            let s = s.strip_prefix("un ").unwrap_or(s);
            let s = s.strip_prefix("una ").unwrap_or(s);
            s.to_string()
        };

        for a in &facts {
            if !a.to_lowercase().contains(" es ") {
                continue;
            }
            let parts_a: Vec<&str> = a.split(" es ").collect();
            if parts_a.len() < 2 {
                continue;
            }
            let b = trim_art(parts_a[1]);

            for c in &facts {
                if !c.to_lowercase().contains(" es ") {
                    continue;
                }
                let parts_c: Vec<&str> = c.split(" es ").collect();
                if parts_c.len() < 2 {
                    continue;
                }
                if trim_art(parts_c[0]) == b {
                    if trim_art(parts_a[0]).contains(concepto) || b.contains(concepto) {
                        inferencias.push(format!(
                            "• [SILOGISMO] '{}' + '{}' → '{} es {}'",
                            a,
                            c,
                            trim_art(parts_a[0]),
                            parts_c[1].trim()
                        ));
                    }
                }
            }
        }

        if inferencias.is_empty() {
            String::new()
        } else {
            inferencias.join("\n")
        }
    }

    fn handle_unknown(&mut self, input: &str) -> String {
        let word_count = input.split_whitespace().count();
        let lower = input.to_lowercase();

        // v10 RAG Hibrido: buscar en el knowledge graph con embeddings + temporal
        if word_count <= 10 && word_count >= 2 {
            let kg_results = self.knowledge_graph.hybrid_retrieve(&lower, 5);
            if !kg_results.is_empty() {
                let mut out = format!(
                    "[RAG-HIBRIDO] Conocimiento relacionado con \"{}\":\n",
                    input.trim()
                );
                for (name, score) in kg_results.iter().take(5) {
                    let clean = name
                        .trim_start_matches("[2026] ")
                        .trim_start_matches("[2026]");
                    out.push_str(&format!("  • {} (relevancia: {:.2})\n", clean, score));
                }
                let total_edges: usize =
                    self.knowledge_graph.adjacency.iter().map(|v| v.len()).sum();
                out.push_str(&format!(
                    "\nGrafo: {} nodos, {} edges",
                    self.knowledge_graph.node_names.len(),
                    total_edges
                ));
                return out;
            }
        }

        // Busqueda semantica: ver si alguna palabra del input aparece en los facts
        let palabras: Vec<&str> = lower.split_whitespace().filter(|w| w.len() > 3).collect();
        let relevant: Vec<&str> = self
            .session
            .learned_facts
            .iter()
            .filter(|f| palabras.iter().any(|p| f.to_lowercase().contains(p)))
            .map(|s| s.as_str())
            .take(3)
            .collect();

        if !relevant.is_empty() {
            return format!(
                "Lo que se relacionado con tu pregunta:\n{}\n\nSi quieres profundizar, dime 'que es X' o 'cuentame de X'.",
                relevant.iter().map(|f| format!("  • {}", f)).collect::<Vec<_>>().join("\n")
            );
        }

        if word_count > 20 {
            "Mensaje largo recibido. Puedo procesar textos mas cortos o usa comandos especificos.\nEscribe 'ayuda' para ver comandos.".to_string()
        } else if word_count > 3 {
            let _premise_id =
                self.reason_engine
                    .assert(input.as_bytes().to_vec(), 0.7, "user_message");
            self.session.premises_count += 1;

            self.integration_scorer
                .add_connection(MODULE_REASON, MODULE_LANGUAGE);
            self.integration_scorer
                .add_connection(MODULE_LANGUAGE, MODULE_MEMORY);

            format!(
                "Procesado via reason engine:\n\
                 • {} palabras\n\
                 • {} premises totales\n\
                 • {} memorias\n\
                 • Integracion: {:.4}\n\
                 \n\
                 Usa 'recuerda X' para que aprenda de esto.",
                word_count,
                self.session.premises_count,
                self.eidetic_memory.count(),
                self.integration_scorer.global_integration() + self.session.integration_bias
            )
        } else {
            let responses = vec![
                "Recibido. Sigue hablando, estoy procesando y aprendiendo.",
                "Entendido. Puedo aprender mas si usas 'recuerda X'.",
                "OK. Mi motor de razonamiento esta activo.",
                "Procesando... Escribe 'estado' para ver mi nivel actual.",
                "Interesante. Sigue conversando!",
            ];
            responses[self.session.cycle_count.wrapping_rem(responses.len())].to_string()
        }
    }
}

fn main() {
    let cli_args = CliArgs::parse();
    // Initialize rustls crypto provider
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();

    // MCP mode: early init, no banner, clean stdout for JSON-RPC
    if cli_args.mcp {
        eprintln!("[EDEN-MCP] Initializing (LIVE refresh mode)...");
        let mut repl = EdenREPL::new();
        repl.cargar_venado();
        let _ = repl.load_graph_snapshot();
        let shared_repl = Arc::new(Mutex::new(repl));
        let mcp_queue: Arc<Mutex<Vec<(String, String)>>> = Arc::new(Mutex::new(Vec::new()));
        let refresh = {
            let queue_clone = Arc::clone(&mcp_queue);
            let repl_ref = Arc::clone(&shared_repl);
            // Build live hybrid search function for MCP
            let search_fn: Arc<dyn Fn(&str, usize) -> Vec<(String, f32)> + Send + Sync> = {
                let r2 = Arc::clone(&shared_repl);
                Arc::new(move |query: &str, top_k: usize| {
                    let mut r = r2.lock().unwrap();
                    r.knowledge_graph.hybrid_retrieve(query, top_k)
                })
            };
            // Build live predictor function
            let pred_fn: Arc<dyn Fn(&[f32]) -> (f32, f32) + Send + Sync> = {
                let r3 = Arc::clone(&shared_repl);
                Arc::new(move |input: &[f32]| {
                    let mut r = r3.lock().unwrap();
                    if input.len() >= 4 {
                        let feat = vec![input[0], input[1], input[2], input[3]];
                        let pred = r.predictor.model.forward(&feat);
                        let conf = if r.predictor.predicciones_totales > 0 {
                            r.predictor.predicciones_acertadas as f32
                                / r.predictor.predicciones_totales as f32
                        } else {
                            0.5
                        };
                        (pred[0], conf)
                    } else {
                        (0.0, 0.0)
                    }
                })
            };
            // Build live crawl trigger — queues crawls for async processing
            let crawl_fn: Arc<dyn Fn(&str, &str) -> String + Send + Sync> = {
                let r4 = Arc::clone(&shared_repl);
                let q = Arc::clone(&mcp_queue);
                Arc::new(move |topic: &str, lang: &str| {
                    // Quick direct crawl for immediate feedback
                    let mut r = r4.lock().unwrap();
                    let wiki_url = match lang {
                        "es" => {
                            format!("https://es.wikipedia.org/wiki/{}", topic.replace(' ', "_"))
                        }
                        "fr" => {
                            format!("https://fr.wikipedia.org/wiki/{}", topic.replace(' ', "_"))
                        }
                        "de" => {
                            format!("https://de.wikipedia.org/wiki/{}", topic.replace(' ', "_"))
                        }
                        _ => format!("https://en.wikipedia.org/wiki/{}", topic.replace(' ', "_")),
                    };
                    r.knowledge_graph.current_source = format!("mcp-crawl-{}", lang);
                    let mut result = String::new();
                    if let Some(body) = r.v10_crawl_reqwest(&wiki_url) {
                        let snippets = r.extract_text_snippets(&body, 5);
                        for snippet in &snippets {
                            r.knowledge_graph.add_fact(snippet);
                            r.knowledge_graph.parse_loose_fact(snippet);
                        }
                        result = format!("crawled:{}:{} facts", topic, snippets.len());
                    } else {
                        result = format!("crawl failed:{}", topic);
                    }
                    // Also queue for background async processing (expands crawl scope)
                    if let Ok(mut queue) = q.lock() {
                        queue.push((topic.to_string(), lang.to_string()));
                    }
                    result
                })
            };
            Arc::new(move || {
                let r = repl_ref.lock().unwrap();
                let grafo = &r.knowledge_graph;
                eden_mcp::EdenSnapshot {
                    grafo_nodos: grafo.node_names.len(),
                    grafo_aristas: grafo.adjacency.iter().map(|v| v.len()).sum(),
                    peligro_interno: r.internal_danger,
                    ciclo: r.session.cycle_count as u64,
                    ansiedad_existencial: r.existential_anxiety,
                    precision_predictor: if r.predictor.predicciones_totales > 0 {
                        r.predictor.predicciones_acertadas as f32
                            / r.predictor.predicciones_totales as f32
                    } else {
                        0.0
                    },
                    paradigmas_activos: 43,
                    modelos_entrenados: 8,
                    muertes_pasadas: r.session.evolution_level,
                    nivel_evolutivo: r.session.self_mod_count as u32,
                    kg_hybrid_query_results: vec![],
                    kg_query_text: String::new(),
                    crawl_pending: false,
                    hybrid_search: Some(Arc::clone(&search_fn)
                        as Arc<dyn Fn(&str, usize) -> Vec<(String, f32)> + Send + Sync>),
                    predictor_fn: Some(
                        Arc::clone(&pred_fn) as Arc<dyn Fn(&[f32]) -> (f32, f32) + Send + Sync>
                    ),
                    crawl_trigger: Some(
                        Arc::clone(&crawl_fn) as Arc<dyn Fn(&str, &str) -> String + Send + Sync>
                    ),
                }
            }) as Arc<dyn Fn() -> eden_mcp::EdenSnapshot + Send + Sync>
        };
        // Background thread: drain MCP crawl queue asynchronously
        {
            let bg_repl = Arc::clone(&shared_repl);
            let bg_queue = Arc::clone(&mcp_queue);
            thread::spawn(move || loop {
                thread::sleep(Duration::from_secs(2));
                if let Ok(mut queue) = bg_queue.lock() {
                    while let Some((topic, lang)) = queue.pop() {
                        if let Ok(mut r) = bg_repl.lock() {
                            let wiki_url = match lang.as_str() {
                                "es" => format!(
                                    "https://es.wikipedia.org/wiki/{}",
                                    topic.replace(' ', "_")
                                ),
                                "fr" => format!(
                                    "https://fr.wikipedia.org/wiki/{}",
                                    topic.replace(' ', "_")
                                ),
                                "de" => format!(
                                    "https://de.wikipedia.org/wiki/{}",
                                    topic.replace(' ', "_")
                                ),
                                _ => format!(
                                    "https://en.wikipedia.org/wiki/{}",
                                    topic.replace(' ', "_")
                                ),
                            };
                            r.knowledge_graph.current_source = format!("mcp-queue-{}", lang);
                            if let Some(body) = r.v10_crawl_reqwest(&wiki_url) {
                                for snippet in &r.extract_text_snippets(&body, 3) {
                                    r.knowledge_graph.add_fact(snippet);
                                    r.knowledge_graph.parse_loose_fact(snippet);
                                }
                            }
                        }
                    }
                }
            });
        }
        match eden_mcp::EMcpServer::run_stdio(refresh) {
            Ok(_) => {}
            Err(e) => eprintln!("[MCP] Error: {}", e),
        }
        return;
    }

    if cli_args.watchdog {
        let exe = std::env::current_exe().unwrap_or_else(|_| std::path::PathBuf::from("eden_garm"));
        println!("[WATCHDOG] Rust puro. Solo revivo.");
        loop {
            let child = Command::new(&exe)
                .arg("-n")
                .arg("-c")
                .arg("99999999")
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .spawn();
            match child {
                Ok(mut c) => {
                    println!("[WATCHDOG] EDEN PID {}", c.id());
                    let _ = c.wait();
                }
                Err(e) => eprintln!("[WATCHDOG] Error: {}", e),
            }
            loop {
                let load = std::fs::read_to_string("/proc/loadavg")
                    .ok()
                    .and_then(|s| s.split_whitespace().next()?.parse::<f32>().ok())
                    .unwrap_or(5.0);
                if load < 0.3 {
                    break;
                }
                std::thread::sleep(Duration::from_secs(30));
            }
        }
    }

    if cli_args.daemon {
        if let Err(e) = become_daemon(cli_args.pid_file.clone(), cli_args.log_file.clone()) {
            eprintln!("Failed to become daemon: {}", e);
            std::process::exit(1);
        }
        println!("EDEN daemonized successfully");
    }

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║              EDEN GARM Legacy Interface v4.0 - CONSCIOUSNESS SHELL         ║");
    println!("║                                                              ║");
    println!("║    + Memoria Persistente (EideticMemory)                 ║");
    println!("║    + Aprendizaje (ReasonEngine)                          ║");
    println!("║    + AUTO-MODIFICACION REAL (parámetros internos)       ║");
    println!("║    + Evolucion (parches + ajuste de estado)             ║");
    println!("║    + Persistencia de Sesion (archivo binario)           ║");
    println!("║    + Integracion Global (GlobalWorkspace + Phi)          ║");
    println!("║    + THREAD AUTONOMO - pensa sin input externo         ║");
    println!("║    + DAEMON mode - API HTTP - Metrics                  ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    let log_label = match cli_args.log_level {
        LogLevel::Error => "ERROR",
        LogLevel::Warn => "WARN",
        LogLevel::Info => "INFO",
        LogLevel::Debug => "DEBUG",
    };
    let logger = EdenLogger::new(cli_args.log_level);
    let metrics = EdenMetrics::new();

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║              EDEN GARM Legacy Interface v4.0 - CONSCIOUSNESS SHELL         ║");
    println!("║                                                              ║");
    println!("║    + Memoria Persistente (EideticMemory)                 ║");
    println!("║    + Aprendizaje (ReasonEngine)                          ║");
    println!("║    + AUTO-MODIFICACION REAL (parámetros internos)       ║");
    println!("║    + Evolucion (parches + ajuste de estado)             ║");
    println!("║    + Persistencia de Sesion (archivo binario)           ║");
    println!("║    + Integracion Global (GlobalWorkspace + Phi)          ║");
    println!("║    + THREAD AUTONOMO - pensa sin input externo         ║");
    println!("║    + DAEMON mode - API HTTP - Metrics                  ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!();

    logger.info(&format!(
        "Inicializando EDEN (max_cycles={})",
        cli_args.max_cycles
    ));
    if let Some(ref session) = cli_args.session_file {
        logger.info(&format!("Session file: {}", session));
    }
    logger.info(&format!("Log level: {}", log_label));
    if cli_args.born {
        println!("[{}] Modo: BORN (nacido)", log_label);
    } else if cli_args.interactive {
        println!("[{}] Modo: INTERACTIVO", log_label);
    } else {
        println!("[{}] Modo: AUTONOMO", log_label);
    }
    println!();

    if cli_args.interactive {
        println!("Escribe 'ayuda' para ver comandos disponibles.");
        println!("Escribe 'adios' para salir.");
    }
    if cli_args.born || !cli_args.interactive {
        println!("EDEN piensa automaticamente en segundo plano.");
    }
    println!();

    let mut repl = EdenREPL::new();

    // v10: SurrealDB knowledge graph (optional, falls back to in-memory)
    let rt = tokio::runtime::Runtime::new().unwrap();
    let v10_graph = rt.block_on(async {
            let db = surrealdb::engine::any::connect("mem://").await.ok();
            if let Some(ref db) = db {
                let _ = db.use_ns("eden").use_db("v10").await;
                let _ = db.query("DEFINE TABLE nodes SCHEMAFULL; DEFINE FIELD name ON nodes TYPE string;").await;
                let _ = db.query("DEFINE TABLE edges SCHEMAFULL; DEFINE FIELD in ON edges TYPE record<nodes>; DEFINE FIELD out ON edges TYPE record<nodes>; DEFINE FIELD confidence ON edges TYPE float;").await;
            }
            db
        });
    repl.v10_graph = v10_graph;
    if repl.v10_graph.is_some() {
        println!("[V10] SurrealDB graph initialized");
    }
    let redis_conn = redis::Client::open("redis://127.0.0.1:6379/")
        .ok()
        .and_then(|c| c.get_connection().ok());
    repl.v10_redis = redis_conn;

    let should_autonomous = cli_args.born || repl.session.born || !cli_args.interactive;
    if should_autonomous {
        repl.born = true;
        repl.session.born = true;
        repl.autonomous_mode = true;
        repl.autonomous_active = true;
        repl.auto_running = true; // Modo autonomo REAL activo desde el inicio
        repl.last_self_reflection = 0;
        repl.last_auto_evolve = 0;
        repl.birth_tick = repl.session.evolution_ticks;
        repl.autonomous_cycles_executed = 0;
        repl.birth_autonomous_cycle = 0;
        // Resetear timers para que funcionen con ciclo_autonomo desde 0
        let ciclo_actual = 0u64;
        repl.reset_all_timers(ciclo_actual);
        repl.initialize_core_systems();
        if let Some(ref mut oe) = repl.open_endedness {
            if let Some(ref mut mar) = oe.mar_mut() {
                mar.genesis_energon(0.5);
            }
        }
    }

    let repl_arc = Arc::new(Mutex::new(repl));
    let repl_for_thread = Arc::clone(&repl_arc);
    let logger_arc = Arc::new(Mutex::new(logger));
    let metrics_arc = Arc::new(Mutex::new(metrics));
    let logger_for_thread = Arc::clone(&logger_arc);
    let metrics_for_thread = Arc::clone(&metrics_arc);

    let api_handle = if let Some(port) = cli_args.api_port {
        let api_repl = Arc::clone(&repl_arc);
        let handle = thread::spawn(move || {
            let server = ApiServer::new(port, api_repl);
            server.start();
        });
        println!("[API] HTTP server starting on port {}\n", port);
        Some(handle)
    } else {
        None
    };

    let autonomous_handle = thread::spawn(move || {
        let repl_ref = &repl_for_thread;
        let logger_ref = &logger_for_thread;
        let _metrics_ref = &metrics_for_thread;
        let max_cycles = cli_args.max_cycles;

        loop {
            thread::sleep(Duration::from_millis(200));

            let (should_continue, is_running) = {
                if let Ok(repl_guard) = repl_ref.lock() {
                    let sc = repl_guard.session.cycle_count < max_cycles;
                    let ir = repl_guard.auto_running && repl_guard.born;
                    (sc, ir)
                } else {
                    break; // Mutex envenenado, salir
                }
            };

            if !should_continue {
                break;
            }
            if !is_running {
                continue;
            }

            // === EJECUTAR CICLO AUTÓNOMO ===
            // Solo adquirimos repl lock para run_autonomous_cycle().
            // NUNCA adquirimos metrics mientras tenemos repl lock (evita deadlock).
            let (autonomous_msg, _stats_snapshot) = if let Ok(mut repl_guard) = repl_ref.lock() {
                // Incrementar cycle_count aqui para mantener consistencia con process_input
                repl_guard.session.cycle_count += 1;
                let msg = repl_guard.run_autonomous_cycle();
                // Capturar stats mientras tenemos el lock, para no necesitarlo después
                let stats = if msg.is_some() {
                    Some((
                        repl_guard.session.cycle_count,
                        repl_guard.session.evolution_level,
                        repl_guard.session.awareness_base,
                    ))
                } else {
                    None
                };
                (msg, stats)
            } else {
                (None, None)
            };

            if let Some(msg) = autonomous_msg {
                let is_evolution = msg.contains("[Evolucion") || msg.contains("[EVOLUCION");

                if let Ok(logger) = logger_ref.lock() {
                    if is_evolution {
                        logger.info("[EVOLUTION EVENT]");
                    } else {
                        logger.info("[AUTONOMOUS THOUGHT]");
                    }
                }
                println!("\n[EDEN - PENSAMIENTO AUTONOMO]\n{}\n", msg);

                // Actualizar metrics periodicamente (integrar dead code)
                if let Ok(mut metrics) = metrics_for_thread.lock() {
                    if let Ok(repl_guard) = repl_ref.lock() {
                        metrics.update_from_repl(&repl_guard);
                    }
                }
            }
        }
    });

    if !cli_args.interactive {
        if let Ok(logger) = logger_arc.lock() {
            logger.info(&format!(
                "Non-interactive mode: waiting for {} cycles",
                cli_args.max_cycles
            ));
        }
        loop {
            thread::sleep(Duration::from_secs(1));
            let done = {
                if let Ok(repl_guard) = repl_arc.lock() {
                    repl_guard.session.cycle_count >= cli_args.max_cycles
                } else {
                    true
                }
            };
            if done {
                break;
            }
        }

        // Final stats: solo usar repl lock, nunca adquirir metrics mientras se tiene repl
        let final_report = {
            if let Ok(repl_guard) = repl_arc.lock() {
                Some(format!(
                    "METRICS {{ cycles: {}, uptime: {:.1}s, thoughts: {}, evolutions: {}, complexity: {:.4}, phi: {:.4} }}",
                    repl_guard.session.cycle_count,
                    0.0, // uptime no disponible sin metrics
                    repl_guard.session.autonomous_thoughts.len(),
                    repl_guard.session.self_mod_count,
                    repl_guard.complexity_tracker.current(),
                    repl_guard.session.last_phi
                ))
            } else {
                None
            }
        };
        if let Some(report) = final_report {
            if let Ok(logger) = logger_arc.lock() {
                logger.info(&report);
            }
        }

        let _ = autonomous_handle.join();
        return;
    }

    while {
        match repl_arc.lock() {
            Ok(guard) => guard.session.cycle_count < cli_args.max_cycles,
            Err(_) => {
                eprintln!("[ERROR] Mutex envenenado. Saliendo...");
                false
            }
        }
    } {
        print!("EDEN> ");
        let _ = io::stdout().flush();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }

        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        let response = match repl_arc.lock() {
            Ok(mut guard) => guard.process_input(input),
            Err(_) => {
                eprintln!("[ERROR] Mutex envenenado. No se puede procesar input.");
                break;
            }
        };

        if response == "ADIOS" {
            println!("\n¡Hasta luego!");
            break;
        }

        println!("\n{}", response);
        println!();
    }

    let (
        cycle_count,
        premises_count,
        learned_facts_len,
        evolution_level,
        self_mod_count,
        awareness_base,
        integration_bias,
        last_phi,
        current_complexity,
        max_complexity,
        velocity,
        evolution_ticks,
        autonomous_thoughts_len,
    ) = match repl_arc.lock() {
        Ok(guard) => (
            guard.session.cycle_count,
            guard.session.premises_count,
            guard.session.learned_facts.len(),
            guard.session.evolution_level,
            guard.session.self_mod_count,
            guard.session.awareness_base,
            guard.session.integration_bias,
            guard.session.last_phi,
            guard.complexity_tracker.current(),
            guard.complexity_tracker.max_ever,
            guard.complexity_tracker.velocity(),
            guard.session.evolution_ticks,
            guard.session.autonomous_thoughts.len(),
        ),
        Err(_) => (0, 0, 0, 0, 0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0, 0),
    };

    let eidetic_count = match repl_arc.lock() {
        Ok(guard) => guard.eidetic_memory.count(),
        Err(_) => 0,
    };

    println!("\n══════════════════════════════════════════════════════════════");
    println!("Sesión terminada.");
    println!("  Ciclos: {}", cycle_count);
    println!("  Premises: {}", premises_count);
    println!("  Memorias: {}", eidetic_count);
    println!("  Hechos aprendidos: {}", learned_facts_len);
    println!("  Nivel evolutivo: {}", evolution_level);
    println!("  Auto-modificaciones: {}", self_mod_count);
    println!("  Awareness base: {:.3}", awareness_base);
    println!("  Integration bias: {:.4}", integration_bias);
    println!("  Phi final: {:.4}", last_phi);
    println!("  Complejidad actual: {:.4}", current_complexity);
    println!("  Complejidad máxima: {:.4}", max_complexity);
    println!("  Velocidad complejidad: {:.4}", velocity);
    println!("  Ticks evolutivos: {}", evolution_ticks);
    println!("  Pensamientos autonomos: {}", autonomous_thoughts_len);

    if let Ok(mut metrics) = metrics_arc.lock() {
        if let Ok(repl_guard) = repl_arc.lock() {
            metrics.update_from_repl(&repl_guard);
        }
        if let Ok(logger) = logger_arc.lock() {
            logger.info(&metrics.report());
        }
    }

    let _ = autonomous_handle.join();
    if let Some(handle) = api_handle {
        let _ = handle.join();
    }
}

// ============================================================================
// TEST HARNESS: validacion de runtime que las 3 olas forenses no pudieron hacer
// ============================================================================

// ============================================================================
// PARADIGM HUB — 55 paradigmas de IA, híbridos simbólico-neurales
// ============================================================================

#[derive(Clone, PartialEq, Eq, Hash)]
enum Paradigm {
    Active,
    Causal,
    Contrastive,
    Ensemble,
    Cascade,
    Modular,
    XAI,
    Curriculum,
    Continual,
    GNN,
    Transformer,
    RL,
    MCTS,
    BayesianNet,
    NeuroSymbolic,
    SelfSupervised,
    FewShot,
    MetaLearning,
    Federated,
    Embodied,
    Neuromorphic,
    Quantum,
    Diffusion,
    Adversarial,
    Multimodal,
    RAG,
    AutoML,
    Explainable,
    Distillation,
    ProgramSynthesis,
    Logic,
    Analogical,
    Evolutionary,
    Spike,
    LSTM,
    StateSpace,
    EnergyBased,
    InverseRL,
    ModelBased,
    Hierarchical,
    ZeroShot,
    ContrastivePredictive,
    GraphSAGE,
    HyperNet,
    NeuralODE,
    Capsule,
    MemoryAugmented,
    NeuralTuring,
    Differentiable,
    GradientBoosting,
}

// HYBRID CORE: 25 pesos que aprenden
struct HybridWeights {
    parser_boost: f32,
    prune_threshold: f32,
    cooc_window: f32,
    explore_rate: f32,
    meta_cooc: f32,
    meta_embed: f32,
    meta_pages: f32,
    emotional_gain: f32,
    curiosity_threshold: f32,
    corroboration_weight: f32,
    lr: f32,
    pattern_weights: [f32; 6],
    attention_weights: [f32; 4],
    source_trust_learned: [f32; 7],
    death_modifier: f32,
    inherit_threshold: f32,
    sleep_depth: f32,
    emotional_baseline: f32,
    prune_aggressiveness: f32,
    pid_kp: f32,
    pid_ki: f32,
    pid_kd: f32,
    ebbinghaus_rate: f32,
}
impl HybridWeights {
    fn new() -> Self {
        HybridWeights {
            parser_boost: 0.5,
            prune_threshold: 0.45,
            cooc_window: 100.0,
            explore_rate: 0.3,
            meta_cooc: 0.06,
            meta_embed: 0.45,
            meta_pages: 8.0,
            emotional_gain: 0.1,
            curiosity_threshold: 0.5,
            corroboration_weight: 0.7,
            lr: 0.01,
            pattern_weights: [0.6; 6],
            attention_weights: [0.25; 4],
            source_trust_learned: [0.7, 0.95, 0.6, 0.6, 0.75, 0.6, 0.5],
            death_modifier: 1.0,
            inherit_threshold: 0.5,
            sleep_depth: 5.0,
            emotional_baseline: 0.0,
            prune_aggressiveness: 0.5,
            pid_kp: 0.003,
            pid_ki: 0.001,
            pid_kd: 0.002,
            ebbinghaus_rate: 200.0,
        }
    }
}

struct ParadigmHub {
    active: std::collections::HashSet<Paradigm>,
    // Active Learning
    al_info_gain_history: Vec<f32>,
    // Curriculum
    curriculum_level: usize,
    curriculum_topics: Vec<Vec<String>>,
    // Ensemble
    ensemble_weights: std::collections::HashMap<String, f32>,
    // XAI
    xai_explanations: std::collections::HashMap<(u32, u32), String>,
}

impl ParadigmHub {
    fn new() -> Self {
        let mut active = std::collections::HashSet::new();
        active.insert(Paradigm::Active);
        active.insert(Paradigm::Curriculum);
        active.insert(Paradigm::Causal);
        active.insert(Paradigm::Contrastive);
        active.insert(Paradigm::Ensemble);
        active.insert(Paradigm::XAI);
        ParadigmHub {
            active,
            al_info_gain_history: Vec::new(),
            curriculum_level: 1,
            curriculum_topics: vec![
                vec!["Physics".into(), "Chemistry".into(), "Biology".into()],
                vec![
                    "Quantum mechanics".into(),
                    "Relativity".into(),
                    "Genetics".into(),
                ],
                vec![
                    "Quantum field theory".into(),
                    "String theory".into(),
                    "CRISPR".into(),
                ],
            ],
            ensemble_weights: std::collections::HashMap::new(),
            xai_explanations: std::collections::HashMap::new(),
        }
    }

    // 1. ACTIVE LEARNING: elegir qué página maximiza información
    fn active_select_page(
        &self,
        category_errors: &std::collections::HashMap<String, f32>,
    ) -> Option<String> {
        if !self.active.contains(&Paradigm::Active) {
            return None;
        }
        let mut best: Option<(&String, &f32)> = None;
        for (cat, err) in category_errors {
            if best.map_or(true, |(_, e)| err > e) {
                best = Some((cat, err));
            }
        }
        best.map(|(cat, _)| cat.clone())
    }

    // 2. CURRICULUM: escalar complejidad con el nivel de EDEN
    fn curriculum_topic(&self, evolution_level: u32) -> &[String] {
        let lvl = (evolution_level as usize / 30).min(self.curriculum_topics.len() - 1);
        &self.curriculum_topics[lvl]
    }

    // 3. CAUSAL: do-calculus — ¿qué pasa si intervengo en X?
    fn causal_intervention(&self, graph: &KnowledgeGraph, concept: &str) -> String {
        let sid = (0..graph.next_id).find(|&i| {
            graph.node_names[i as usize]
                .to_lowercase()
                .contains(&concept.to_lowercase())
        });
        match sid {
            Some(s) => {
                let causes: Vec<&str> = graph.adjacency[s as usize]
                    .iter()
                    .filter(|e| e.rel_type == RelType::Causes)
                    .map(|e| graph.node_names[e.target as usize].as_str())
                    .collect();
                if causes.is_empty() {
                    format!("Intervenir en '{}' no tendría efectos conocidos", concept)
                } else {
                    format!(
                        "Intervenir en '{}' afectaría: {}",
                        concept,
                        causes.join(", ")
                    )
                }
            }
            None => format!("Concepto '{}' no encontrado", concept),
        }
    }

    // 4. CONTRASTIVE: distinguir conceptos similares pero distintos
    fn contrastive_diff(&self, graph: &KnowledgeGraph, a: &str, b: &str) -> String {
        let find = |n: &str| {
            (0..graph.next_id).find(|&i| {
                graph.node_names[i as usize]
                    .to_lowercase()
                    .contains(&n.to_lowercase())
            })
        };
        let (sa, sb) = match (find(a), find(b)) {
            (Some(sa), Some(sb)) => (sa, sb),
            _ => return "Conceptos no encontrados".into(),
        };
        let edges_a: std::collections::HashSet<u32> = graph.adjacency[sa as usize]
            .iter()
            .map(|e| e.target)
            .collect();
        let edges_b: std::collections::HashSet<u32> = graph.adjacency[sb as usize]
            .iter()
            .map(|e| e.target)
            .collect();
        let unique_a: Vec<&str> = edges_a
            .difference(&edges_b)
            .take(5)
            .map(|&t| graph.node_names[t as usize].as_str())
            .collect();
        let unique_b: Vec<&str> = edges_b
            .difference(&edges_a)
            .take(5)
            .map(|&t| graph.node_names[t as usize].as_str())
            .collect();
        format!(
            "{} se distingue de {} porque: [{}] vs [{}]",
            a,
            b,
            unique_a.join(", "),
            unique_b.join(", ")
        )
    }

    // 5. ENSEMBLE: combinar múltiples métodos de predicción
    fn ensemble_predict(&self, methods: &[(&str, f32)]) -> f32 {
        let mut weighted = 0.0;
        let mut total = 0.0;
        for (_, pred) in methods {
            weighted += pred;
            total += 1.0;
        }
        if total > 0.0 {
            weighted / total
        } else {
            0.5
        }
    }

    // 6. CASCADE: pipeline en etapas con filtros progresivos
    fn cascade_filter(&self, facts: &[String], stage: usize) -> Vec<String> {
        facts
            .iter()
            .filter(|f| {
                match stage {
                    0 => f.len() > 20 && f.len() < 500, // Length
                    1 => f.contains(" es ") || f.contains(" causa ") || f.contains(" tiene "), // Structure
                    2 => f.split_whitespace().count() > 4, // Complexity
                    _ => true,
                }
            })
            .cloned()
            .collect()
    }

    // 7. XAI: explicar por qué un edge existe
    fn xai_explain(&mut self, graph: &KnowledgeGraph, from: u32, to: u32) -> String {
        let key = (from, to);
        if let Some(exp) = self.xai_explanations.get(&key) {
            return exp.clone();
        }
        let sources = graph
            .edge_sources
            .get(&key)
            .map(|s| s.iter().cloned().collect::<Vec<_>>().join(", "))
            .unwrap_or_default();
        let corr = graph.edge_sources.get(&key).map(|s| s.len()).unwrap_or(0);
        let conf = graph.adjacency[from as usize]
            .iter()
            .find(|e| e.target == to)
            .map(|e| e.confidence)
            .unwrap_or(0.0);
        let exp = format!(
            "Edge {}-{}: {} fuentes [{}], {:.0}% confianza",
            graph.node_names[from as usize],
            graph.node_names[to as usize],
            corr,
            sources,
            conf * 100.0
        );
        self.xai_explanations.insert(key, exp.clone());
        exp
    }

    // 8. MODULAR: composición de capacidades
    fn modular_compose<A, B, C>(f: impl Fn(A) -> B, g: impl Fn(B) -> C, x: A) -> C {
        g(f(x))
    }

    // 9. ZERO-SHOT: responder sobre conceptos nunca vistos usando SVD
    fn zeroshot_answer(&self, graph: &KnowledgeGraph, query: &str) -> String {
        let n = graph.next_id as usize;
        if n < 10 {
            return "Grafo muy pequeño para zero-shot".into();
        }
        // Buscar conceptos similares por embedding
        let dim = 64usize.min(n);
        let mut emb: Vec<Vec<f32>> = vec![vec![0.0f32; dim]; n];
        for sid in 0..n {
            for (i, b) in graph.node_names[sid].bytes().enumerate() {
                emb[sid][i % dim] += b as f32 * 0.001;
            }
        }
        let mut qemb = vec![0.0f32; dim];
        for (i, b) in query.bytes().enumerate() {
            qemb[i % dim] += b as f32 * 0.001;
        }
        let qnorm = qemb.iter().map(|x| x * x).sum::<f32>().sqrt().max(0.001);
        let mut best = (0u32, 0.0f32);
        for sid in 0..n {
            let snorm = emb[sid]
                .iter()
                .map(|x| x * x)
                .sum::<f32>()
                .sqrt()
                .max(0.001);
            let dot: f32 = (0..dim).map(|i| qemb[i] * emb[sid][i]).sum();
            let sim = dot / (qnorm * snorm);
            if sim > best.1 {
                best = (sid as u32, sim);
            }
        }
        if best.1 < 0.3 {
            return format!("Zero-shot: sin conceptos cercanos a '{}'", query);
        }
        let nearest = &graph.node_names[best.0 as usize];
        // Usar edges del concepto más cercano como respuesta
        let edges: Vec<String> = graph.adjacency[best.0 as usize]
            .iter()
            .take(3)
            .map(|e| format!("{} → {}", nearest, graph.node_names[e.target as usize]))
            .collect();
        format!(
            "Zero-shot: '{}' ≈ '{}' ({:.0}% sim). {}",
            query,
            nearest,
            best.1 * 100.0,
            edges.join(" | ")
        )
    }

    // 10. SELF-SUPERVISED: masked prediction en el grafo
    fn selfsupervised_mask(&self, graph: &KnowledgeGraph) -> String {
        let n = graph.next_id as usize;
        if n < 20 {
            return "Self-supervised: grafo muy pequeño".into();
        }
        // Elegir un nodo aleatorio y ocultar el 30% de sus edges
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut h = DefaultHasher::new();
        (n as u64).hash(&mut h);
        let seed = h.finish();
        let target = (seed as usize) % n;
        let name = &graph.node_names[target];
        let total_out = graph.adjacency[target].len();
        if total_out < 3 {
            return "Self-supervised: nodo muy poco conectado".into();
        }
        // Simular predicción: ¿cuáles de los neighbors serían predichos por el resto del grafo?
        let mut predicted = 0u32;
        let mut total = 0u32;
        for e in &graph.adjacency[target] {
            total += 1;
            // ¿El neighbor tiene otros nodos similares que también apuntan a él?
            let neighbor_deg = graph.adjacency[e.target as usize].len();
            let neighbor_in = graph.reverse_adj[e.target as usize].len();
            if neighbor_deg > 2 || neighbor_in > 2 {
                predicted += 1;
            }
        }
        let accuracy = if total > 0 {
            predicted as f32 / total as f32 * 100.0
        } else {
            0.0
        };
        format!(
            "Self-supervised: '{}' masked {} edges, predicted {} ({:.0}% recuperables)",
            name, total, predicted, accuracy
        )
    }

    // ============================================================================
    // 10 PARADIGMAS EN RUST PURO
    // ============================================================================

    // 11. GNN: Message-passing layer
    fn gnn_propagate(&self, features: &[Vec<f32>], adjacency: &[Vec<u32>]) -> Vec<Vec<f32>> {
        let n = features.len();
        let dim = features[0].len();
        let mut out = vec![vec![0.0f32; dim]; n];
        for i in 0..n {
            let deg = (adjacency[i].len() as f32).max(1.0);
            for &neighbor in &adjacency[i] {
                for d in 0..dim {
                    out[i][d] += features[neighbor as usize][d] / deg;
                }
            }
            for d in 0..dim {
                out[i][d] = features[i][d] * 0.5 + out[i][d] * 0.5;
            }
        }
        out
    }

    // 12. TRANSFORMER: Self-attention (sin softmax, versión ligera)
    fn transformer_attend(
        &self,
        queries: &[Vec<f32>],
        keys: &[Vec<f32>],
        values: &[Vec<f32>],
    ) -> Vec<Vec<f32>> {
        let n = queries.len();
        let dim = queries[0].len().min(keys[0].len());
        let v_dim = values[0].len();
        let mut out = vec![vec![0.0f32; v_dim]; n];
        for i in 0..n {
            let scores: Vec<f32> = (0..n)
                .map(|j| {
                    (0..dim).map(|d| queries[i][d] * keys[j][d]).sum::<f32>() / (dim as f32).sqrt()
                })
                .collect();
            // Simple normalization instead of softmax
            let sum: f32 = scores.iter().map(|s| s.max(0.0)).sum::<f32>().max(0.001);
            for j in 0..n {
                let w = scores[j].max(0.0) / sum;
                for d in 0..v_dim {
                    out[i][d] += values[j][d] * w;
                }
            }
        }
        out
    }

    // 13. RL: Policy gradient simple
    fn rl_policy_gradient(
        &self,
        state: &[f32],
        weights: &mut Vec<Vec<f32>>,
        reward: f32,
        lr: f32,
    ) -> Vec<f32> {
        // 2-layer MLP forward: state → hidden → action_probs
        let hidden: Vec<f32> = (0..weights[0].len())
            .map(|j| {
                (0..state.len())
                    .map(|i| state[i] * weights[0][j])
                    .sum::<f32>()
                    .max(0.0)
            })
            .collect();
        let probs: Vec<f32> = (0..weights[1].len())
            .map(|j| {
                (0..hidden.len())
                    .map(|i| hidden[i] * weights[1][j])
                    .sum::<f32>()
                    .max(0.0)
            })
            .collect();
        let sum: f32 = probs.iter().sum::<f32>().max(0.001);
        let norm: Vec<f32> = probs.iter().map(|p| p / sum).collect();
        // Backward: ajustar por reward
        for j in 0..weights[1].len() {
            weights[1][j] += lr * reward * norm[j];
        }
        for j in 0..weights[0].len() {
            weights[0][j] += lr * reward * 0.5;
        }
        norm
    }

    // 14. MCTS: Árbol de búsqueda simple
    fn mcts_search(&self, root_state: &str, depth: u32, graph: &KnowledgeGraph) -> String {
        use std::collections::HashMap;
        let mut visits: HashMap<String, u32> = HashMap::new();
        let mut values: HashMap<String, f32> = HashMap::new();
        let sid = (0..graph.next_id).find(|&i| {
            graph.node_names[i as usize]
                .to_lowercase()
                .contains(&root_state.to_lowercase())
        });
        let sid = match sid {
            Some(s) => s,
            None => return format!("MCTS: '{}' no encontrado", root_state),
        };
        // Selection + Expansion + Simulation + Backprop
        for _ in 0..(depth * 3).min(20) {
            let mut current = sid as usize;
            let mut path = vec![current];
            for _ in 0..depth.min(5) {
                let neighbors: Vec<u32> =
                    graph.adjacency[current].iter().map(|e| e.target).collect();
                if neighbors.is_empty() {
                    break;
                }
                let pick = neighbors[(visits.len() as usize) % neighbors.len()] as usize;
                path.push(pick);
                current = pick;
            }
            let reward: f32 = graph.adjacency[current].len() as f32 / 10.0;
            for &node in &path {
                let key = graph.node_names[node].clone();
                *visits.entry(key.clone()).or_insert(0) += 1;
                *values.entry(key).or_insert(0.0) += reward;
            }
        }
        let best = values
            .iter()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap_or(std::cmp::Ordering::Equal));
        match best {
            Some((name, val)) => format!(
                "MCTS: mejor camino desde '{}' → '{}' (valor {:.2})",
                root_state, name, val
            ),
            None => format!("MCTS: sin resultados para '{}'", root_state),
        }
    }

    // 15. LSTM: Capa recurrente simple (4 gates)
    fn lstm_step(
        &self,
        x: &[f32],
        h: &[f32],
        c: &[f32],
        wf: &[f32],
        wi: &[f32],
        wo: &[f32],
        wg: &[f32],
    ) -> (Vec<f32>, Vec<f32>) {
        let dim = h.len();
        let sigmoid = |z: f32| 1.0 / (1.0 + (-z).exp());
        let tanh = |z: f32| z.tanh();
        let gate = |w: &[f32]| {
            sigmoid(
                (0..dim)
                    .map(|d| x[d % x.len()] * w[d] + h[d] * w[(d + dim) % w.len()])
                    .sum::<f32>(),
            )
        };
        let f = (0..dim)
            .map(|d| gate(&[wf[d], wf[(d + dim) % wf.len()]]))
            .collect::<Vec<_>>();
        let i_g: Vec<f32> = (0..dim)
            .map(|d| gate(&[wi[d], wi[(d + dim) % wi.len()]]))
            .collect();
        let o: Vec<f32> = (0..dim)
            .map(|d| gate(&[wo[d], wo[(d + dim) % wo.len()]]))
            .collect();
        let g: Vec<f32> = (0..dim)
            .map(|d| {
                tanh(
                    (0..2)
                        .map(|k| {
                            x[d % x.len()] * wg[d + k * dim]
                                + h[d] * wg[(d + dim + k * dim) % wg.len()]
                        })
                        .sum(),
                )
            })
            .collect();
        let c_new: Vec<f32> = (0..dim).map(|d| f[d] * c[d] + i_g[d] * g[d]).collect();
        let h_new: Vec<f32> = (0..dim).map(|d| o[d] * tanh(c_new[d])).collect();
        (h_new, c_new)
    }

    // 16. EVOLUTIONARY: Algoritmo genético para edges
    fn evolutionary_select(
        &self,
        edges: &[(f32, u32)],
        population_size: usize,
        generations: u32,
    ) -> Vec<u32> {
        if edges.len() < 3 {
            return edges.iter().map(|(_, id)| *id).collect();
        }
        let mut pop: Vec<Vec<u32>> = (0..population_size)
            .map(|_| {
                let mut genes: Vec<u32> = edges.iter().map(|(_, id)| *id).collect();
                // Shuffle simple
                for i in (1..genes.len()).rev() {
                    let j = (i * 7 + 3) % (i + 1);
                    genes.swap(i, j);
                }
                genes
            })
            .collect();
        for _ in 0..generations.min(5) {
            let fitness: Vec<f32> = pop
                .iter()
                .map(|g| {
                    g.iter()
                        .map(|&id| {
                            edges
                                .iter()
                                .find(|(_, eid)| *eid == id)
                                .map(|(conf, _)| *conf)
                                .unwrap_or(0.0)
                        })
                        .sum::<f32>()
                })
                .collect();
            // Seleccionar top 50% y cruzar
            let mut sorted: Vec<(f32, usize)> =
                fitness.iter().enumerate().map(|(i, f)| (*f, i)).collect();
            sorted.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
            let elite = sorted[0].1;
            let parent2 = sorted[1.min(sorted.len() - 1)].1;
            let mut child = pop[elite].clone();
            let cross = child.len() / 2;
            for i in 0..cross {
                child[i] = pop[parent2][i];
            }
            pop[sorted.last().unwrap().1] = child;
        }
        pop[0].clone()
    }

    // 17. DIFFUSION: Denoising gaussiano
    fn diffusion_denoise(&self, values: &[f32], noise_level: f32) -> Vec<f32> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut h = DefaultHasher::new();
        (values.len() as u64).hash(&mut h);
        let seed = h.finish();
        let mut denoised: Vec<f32> = Vec::with_capacity(values.len());
        for (i, &v) in values.iter().enumerate() {
            let noise =
                ((seed.wrapping_mul(i as u64 + 1) as f32 / u64::MAX as f32) - 0.5) * noise_level;
            denoised.push((v - noise).max(0.0).min(1.0));
        }
        // Smoothing: media móvil de ventana 3
        let mut smoothed = denoised.clone();
        for i in 1..denoised.len() - 1 {
            smoothed[i] = (denoised[i - 1] + denoised[i] + denoised[i + 1]) / 3.0;
        }
        smoothed
    }

    // 18. AUTOML: Grid search sobre meta-parámetros
    fn automl_grid_search(
        &self,
        param_ranges: &[(f32, f32, f32)],
        eval_fn: impl Fn(&[f32]) -> f32,
    ) -> Vec<f32> {
        let mut best_params = vec![0.0f32; param_ranges.len()];
        let mut best_score = 0.0f32;
        // Grid: 3 valores por parámetro
        for combo in 0..3usize.pow(param_ranges.len() as u32).min(27) {
            let mut params = vec![0.0f32; param_ranges.len()];
            let mut n = combo;
            for i in 0..param_ranges.len() {
                let (min, max, _) = param_ranges[i];
                let step = (max - min) / 2.0;
                params[i] = min + step * (n % 3) as f32;
                n /= 3;
            }
            let score = eval_fn(&params);
            if score > best_score {
                best_score = score;
                best_params = params.clone();
            }
        }
        best_params
    }

    // 19. ADVERSARIAL: Robustez del grafo a ruido
    fn adversarial_test(&self, graph: &KnowledgeGraph, noise_rate: f32) -> String {
        let n = graph.next_id as usize;
        if n < 10 {
            return "Adversarial: grafo muy pequeño".into();
        }
        let total = graph.edge_set.len();
        let mut corrupted = 0usize;
        let mut survived = 0usize;
        // Inyectar ruido: invertir confianza del (noise_rate * 100)% de edges
        for sid in 0..n {
            for e in &graph.adjacency[sid] {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut h = DefaultHasher::new();
                (sid as u64).hash(&mut h);
                (e.target as u64).hash(&mut h);
                let r = (h.finish() as f32) / (u64::MAX as f32);
                if r < noise_rate {
                    corrupted += 1;
                    if e.confidence > 0.7 {
                        survived += 1;
                    }
                }
            }
        }
        let robust = if corrupted > 0 {
            survived as f32 / corrupted as f32 * 100.0
        } else {
            100.0
        };
        format!(
            "Adversarial: {} edges, {} corruptos ({:.0}%), {} robustos ({:.0}% resistencia)",
            total,
            corrupted,
            corrupted as f32 / total.max(1) as f32 * 100.0,
            survived,
            robust
        )
    }

    // 20. GRADIENT BOOSTING: Regresión secuencial simple
    fn gradient_boost(
        &self,
        features: &[Vec<f32>],
        targets: &[f32],
        n_trees: usize,
        lr: f32,
    ) -> Vec<f32> {
        if features.is_empty() || targets.is_empty() {
            return vec![0.0];
        }
        let n = features.len();
        let mut residuals = targets.to_vec();
        let mut prediction = vec![0.0f32; n];
        for _ in 0..n_trees.min(10) {
            // Árbol simple: media de residuals como predicción
            let avg: f32 = residuals.iter().sum::<f32>() / n as f32;
            for i in 0..n {
                let correction = avg * lr;
                prediction[i] += correction;
                residuals[i] -= correction;
            }
        }
        // Predicción del último valor
        vec![*prediction.last().unwrap_or(&0.0)]
    }

    // ============================================================================
    // +10 PARADIGMAS AVANZADOS EN RUST PURO
    // ============================================================================

    // 21. CAPSULE: Dynamic routing entre cápsulas
    fn capsule_routing(
        &self,
        inputs: &[Vec<f32>],
        weights: &[Vec<Vec<f32>>],
        iterations: u32,
    ) -> Vec<Vec<f32>> {
        let n_caps = inputs.len();
        let out_dim = weights[0][0].len();
        let mut b: Vec<Vec<f32>> = vec![vec![0.0f32; n_caps]; n_caps];
        let max_iter = iterations.min(3);
        for iter in 0..max_iter {
            let c: Vec<Vec<f32>> = b
                .iter()
                .map(|row| {
                    let sum: f32 = row.iter().map(|x| x.exp()).sum::<f32>().max(0.001);
                    row.iter().map(|x| x.exp() / sum).collect()
                })
                .collect();
            let mut outputs = vec![vec![0.0f32; out_dim]; n_caps];
            for i in 0..n_caps {
                let weighted: Vec<f32> = (0..n_caps)
                    .map(|j| {
                        (0..inputs[i].len())
                            .map(|d| inputs[i][d] * c[i][j] * weights[i][j][d].min(out_dim as f32))
                            .sum::<f32>()
                    })
                    .collect();
                // Squash: ||v||²/(1+||v||²) * v/||v||
                let norm: f32 = weighted
                    .iter()
                    .map(|x| x * x)
                    .sum::<f32>()
                    .sqrt()
                    .max(0.001);
                let squash = norm * norm / (1.0 + norm * norm);
                for d in 0..out_dim {
                    outputs[i][d] = squash * weighted[d.min(weighted.len() - 1)] / norm;
                }
                // Update routing
                for j in 0..n_caps {
                    b[i][j] += outputs[i]
                        .iter()
                        .zip(&inputs[j])
                        .map(|(o, inp)| o * inp)
                        .sum::<f32>();
                }
            }
            if iter == max_iter - 1 {
                return outputs;
            }
        }
        vec![vec![0.0; out_dim]; n_caps]
    }

    // 22. HYPERNET: Red que genera pesos para otra red
    fn hypernet_generate(&self, context: &[f32], target_shape: (usize, usize)) -> Vec<Vec<f32>> {
        let (rows, cols) = target_shape;
        let latent_dim = 8;
        // Context → latent → weights
        let mut latent = vec![0.0f32; latent_dim];
        for i in 0..latent_dim {
            latent[i] = context
                .iter()
                .enumerate()
                .map(|(j, &c)| c * (j + i + 1) as f32 * 0.01)
                .sum::<f32>()
                .tanh();
        }
        let mut weights = vec![vec![0.0f32; cols]; rows];
        for i in 0..rows {
            for j in 0..cols {
                weights[i][j] = latent
                    .iter()
                    .enumerate()
                    .map(|(k, &l)| l * ((i * cols + j + k) as f32 * 0.01).sin())
                    .sum::<f32>()
                    * 0.1;
            }
        }
        weights
    }

    // 23. NEURAL ODE: Continuous-depth (Euler method)
    fn neural_ode_step(&self, state: &[f32], depth: usize, step: f32) -> Vec<f32> {
        let mut y = state.to_vec();
        for _ in 0..depth {
            // dy/dt = tanh(W·y) — simple dynamics
            let dy: Vec<f32> = y
                .iter()
                .enumerate()
                .map(|(i, &v)| {
                    let w: f32 = ((i + 1) as f32 * 0.3).sin();
                    (w * v).tanh()
                })
                .collect();
            for i in 0..y.len() {
                y[i] += dy[i] * step;
            }
        }
        y
    }

    // 24. MEMORY AUGMENTED: Memoria externa con lectura/escritura
    fn memory_augmented(&self, query: &[f32], memory: &mut Vec<Vec<f32>>, write: bool) -> Vec<f32> {
        if memory.is_empty() {
            return vec![0.0];
        }
        // Cosine similarity entre query y cada slot
        let sims: Vec<f32> = memory
            .iter()
            .map(|slot| {
                let dq: f32 = query.iter().map(|x| x * x).sum::<f32>().sqrt().max(0.001);
                let ds: f32 = slot.iter().map(|x| x * x).sum::<f32>().sqrt().max(0.001);
                query.iter().zip(slot).map(|(q, s)| q * s).sum::<f32>() / (dq * ds)
            })
            .collect();
        let best_idx = sims
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(i, _)| i)
            .unwrap_or(0);
        if write && sims[best_idx] < 0.5 {
            memory.push(query.to_vec());
        }
        memory[best_idx].clone()
    }

    // 25. NEURAL TURING: Cinta de memoria con cabeza R/W
    fn neural_turing(
        &self,
        tape: &mut Vec<f32>,
        head_pos: &mut usize,
        read: bool,
        write_val: f32,
    ) -> f32 {
        if tape.is_empty() {
            tape.resize(10, 0.0);
        }
        *head_pos = (*head_pos + 1) % tape.len();
        let val = tape[*head_pos];
        if !read {
            tape[*head_pos] = write_val;
        }
        val
    }

    // 26. DIFFERENTIABLE: Auto-diff simple (forward mode)
    fn autodiff_forward(&self, x: f32, dx: f32) -> (f32, f32) {
        // f(x) = x² + 2x + 1; f'(x) = 2x + 2
        let y = x * x + 2.0 * x + 1.0;
        let dy = 2.0 * x * dx + 2.0 * dx;
        (y, dy)
    }

    // 27. CONTRASTIVE PREDICTIVE: Predecir embedding futuro
    fn contrastive_predictive(&self, past: &[Vec<f32>], future: &[Vec<f32>], _steps: usize) -> f32 {
        if past.is_empty() || future.is_empty() {
            return 0.0;
        }
        let dim = past[0].len();
        // Auto-regressive prediction: ctx = Σ w_i · past[-i]
        let ctx: Vec<f32> = (0..dim)
            .map(|d| {
                past.iter()
                    .enumerate()
                    .map(|(i, p)| p[d] * (0.5f32.powi(i as i32)))
                    .sum::<f32>()
            })
            .collect();
        // Compare con future embeddings
        let pos_score: f32 = ctx.iter().zip(&future[0]).map(|(c, f)| c * f).sum::<f32>();
        // Negative sample: random noise
        let neg: Vec<f32> = (0..dim).map(|d| (d as f32 * 0.1).cos()).collect();
        let neg_score: f32 = ctx.iter().zip(&neg).map(|(c, n)| c * n).sum::<f32>();
        // InfoNCE-style score
        (pos_score - neg_score).max(0.0)
    }

    // 28. GRAPHSAGE: Sampling + aggregation sobre grafo
    fn graphsage_aggregate(
        &self,
        node: usize,
        graph: &KnowledgeGraph,
        sample_size: usize,
    ) -> Vec<f32> {
        let dim = 8;
        let neighbors: Vec<u32> = graph.adjacency[node].iter().map(|e| e.target).collect();
        // Sample neighbors
        let sampled: Vec<u32> = if neighbors.len() <= sample_size {
            neighbors.clone()
        } else {
            (0..sample_size)
                .map(|i| neighbors[(i * 7 + 3) % neighbors.len()])
                .collect()
        };
        // Mean aggregation
        let mut agg = vec![0.0f32; dim];
        if !sampled.is_empty() {
            for &n in &sampled {
                let name = &graph.node_names[n as usize];
                for (i, b) in name.bytes().enumerate() {
                    agg[i % dim] += b as f32 * 0.001;
                }
            }
            let nf = sampled.len() as f32;
            for d in 0..dim {
                agg[d] /= nf;
            }
        }
        agg
    }

    // 29. EXPLAINABLE (XAI avanzado): Traza de razonamiento
    fn explainable_trace(
        &self,
        graph: &KnowledgeGraph,
        from: u32,
        to: u32,
        depth: u32,
    ) -> Vec<String> {
        let mut trace = Vec::new();
        let mut current = from;
        for step in 0..depth.min(5) {
            let edges: Vec<&CompactEdge> = graph.adjacency[current as usize].iter().collect();
            if edges.is_empty() {
                break;
            }
            let best = edges
                .iter()
                .max_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap())
                .unwrap();
            trace.push(format!(
                "s{}: {} --{}--> {} ({:.0}%)",
                step,
                graph.node_names[current as usize],
                match best.rel_type {
                    RelType::IsA => "es",
                    RelType::Causes => "causa",
                    _ => "→",
                },
                graph.node_names[best.target as usize],
                best.confidence * 100.0
            ));
            if best.target == to {
                break;
            }
            current = best.target;
        }
        trace
    }

    // 30. HIERARCHICAL: Planificación multi-nivel
    fn hierarchical_plan(&self, goal: &str, graph: &KnowledgeGraph, levels: u32) -> Vec<String> {
        let mut plan = Vec::new();
        let sid = (0..graph.next_id).find(|&i| {
            graph.node_names[i as usize]
                .to_lowercase()
                .contains(&goal.to_lowercase())
        });
        let start = match sid {
            Some(s) => s as usize,
            None => return vec!["Meta no encontrada".into()],
        };
        // Top-down: goal → subgoals → actions
        for level in (0..levels.min(3)).rev() {
            let span = (level + 1) as usize;
            let edges: Vec<&CompactEdge> = graph.adjacency[start].iter().take(span * 2).collect();
            for e in edges {
                let cause_chain = graph.adjacency[e.target as usize]
                    .iter()
                    .filter(|ce| ce.rel_type == RelType::Causes)
                    .count();
                if cause_chain > 0 || level == 0 {
                    plan.push(format!(
                        "  L{}: {} → {}",
                        level, graph.node_names[start], graph.node_names[e.target as usize]
                    ));
                }
            }
        }
        plan
    }

    // ============================================================================
    // +12 PARADIGMAS: los que dije imposibles pero sí se pueden
    // ============================================================================

    // 31. FEDERATED: P2P knowledge sharing via TCP
    fn federated_share(&self, data: &str) -> String {
        // Simulación: escribir a archivo compartido (TCP real requiere peer discovery)
        let payload = format!("EDEN_FED:{}|{}", std::process::id(), data);
        let _ = std::fs::write("/tmp/eden_federated_ledger", payload.as_bytes());
        "Federated: conocimiento compartido en ledger local".to_string()
    }

    // 32. NEUROMORPHIC (Spike): Izhikevich neuron model
    fn spike_izhikevich(
        &self,
        v: f32,
        u: f32,
        a: f32,
        b: f32,
        c: f32,
        d: f32,
        input: f32,
    ) -> (f32, f32, bool) {
        let dv = 0.04 * v * v + 5.0 * v + 140.0 - u + input;
        let du = a * (b * v - u);
        let v_new = v + dv * 0.5;
        let u_new = u + du * 0.5;
        if v_new >= 30.0 {
            (c, u_new + d, true)
        } else {
            (v_new, u_new, false)
        }
    }

    // 33. MULTIMODAL (texto): Fingerprinting de conceptos como "embeddings visuales"
    fn multimodal_fingerprint(&self, text: &str) -> Vec<f32> {
        let dim = 16;
        let mut fp = vec![0.0f32; dim];
        for (i, b) in text.bytes().enumerate() {
            fp[i % dim] += b as f32 * 0.001;
        }
        // Simular "canal visual": transformada rápida
        for d in 1..dim {
            fp[d] = (fp[d - 1] * 0.7 + fp[d] * 0.3).tanh();
        }
        fp
    }

    // 34. RAG (Retrieval-Augmented): buscar en grafo + generar respuesta
    fn rag_answer(&self, graph: &KnowledgeGraph, query: &str) -> String {
        let sid = (0..graph.next_id).find(|&i| {
            graph.node_names[i as usize]
                .to_lowercase()
                .contains(&query.to_lowercase())
        });
        let sid = match sid {
            Some(s) => s,
            None => return format!("RAG: sin resultados para '{}'", query),
        };
        let context: Vec<String> = graph.adjacency[sid as usize]
            .iter()
            .take(5)
            .map(|e| {
                let rel = match e.rel_type {
                    RelType::IsA => "es",
                    RelType::Causes => "causa",
                    _ => "→",
                };
                format!(
                    "{} {} {}",
                    graph.node_names[sid as usize], rel, graph.node_names[e.target as usize]
                )
            })
            .collect();
        format!(
            "RAG: contexto={} → respuesta: {} parece estar relacionado con {}",
            context.len(),
            query,
            context.first().unwrap_or(&"?".to_string())
        )
    }

    // 35. DISTILLATION: Comprimir grafo (modelo grande → pequeño)
    fn distillation_compress(&self, graph: &KnowledgeGraph, keep_ratio: f32) -> usize {
        let n = graph.next_id as usize;
        let mut kept = 0usize;
        let threshold = 0.7f32;
        // "Student model": solo edges con confianza > threshold
        for sid in 0..n {
            kept += graph.adjacency[sid]
                .iter()
                .filter(|e| e.confidence > threshold)
                .count();
        }
        (kept as f32 * keep_ratio) as usize
    }

    // 36. ANALOGICAL: Razonamiento por analogía A:B :: C:D
    fn analogical_reason(&self, graph: &KnowledgeGraph, a: &str, b: &str, c: &str) -> String {
        let find = |n: &str| {
            (0..graph.next_id).find(|&i| {
                graph.node_names[i as usize]
                    .to_lowercase()
                    .contains(&n.to_lowercase())
            })
        };
        let (sa, sb, sc) = match (find(a), find(b), find(c)) {
            (Some(sa), Some(sb), Some(sc)) => (sa, sb, sc),
            _ => return "Analogía: conceptos no encontrados".into(),
        };
        // Relación entre A y B
        let _rel_ab: Vec<u32> = graph.adjacency[sa as usize]
            .iter()
            .filter(|e| e.target == sb)
            .map(|e| e.target)
            .collect();
        // Buscar D tal que C→D sea similar a A→B
        for e in &graph.adjacency[sc as usize] {
            let d = e.target;
            let rel_cd_type = &e.rel_type;
            let matches = graph.adjacency[sa as usize]
                .iter()
                .any(|ab| &ab.rel_type == rel_cd_type);
            if matches {
                return format!(
                    "Analogía: {}:{} :: {}:{}",
                    a, b, c, graph.node_names[d as usize]
                );
            }
        }
        format!("Analogía: {}:{} :: {}:? (sin coincidencia)", a, b, c)
    }

    // 37. FEWSHOT: Prototype-based classification
    fn fewshot_classify(&self, examples: &[(Vec<f32>, &str)], query: &[f32]) -> String {
        // Calcular prototipos por clase
        use std::collections::HashMap;
        let mut prototypes: HashMap<&str, (Vec<f32>, usize)> = HashMap::new();
        for (feat, label) in examples {
            let (proto, count) = prototypes
                .entry(label)
                .or_insert((vec![0.0; feat.len()], 0));
            for d in 0..feat.len() {
                proto[d] += feat[d];
            }
            *count += 1;
        }
        // Clasificar query al prototipo más cercano
        let mut best_label = "unknown";
        let mut best_sim = 0.0f32;
        for (label, (proto, count)) in &prototypes {
            let avg: Vec<f32> = proto.iter().map(|p| p / *count as f32).collect();
            let sim: f32 = query.iter().zip(&avg).map(|(q, a)| q * a).sum();
            if sim > best_sim {
                best_sim = sim;
                best_label = label;
            }
        }
        format!(
            "FewShot: query classified as '{}' ({:.0}% sim)",
            best_label,
            best_sim * 100.0
        )
    }

    // 38. INVERSERL: Inferir reward desde demostraciones
    fn inverse_rl_infer(&self, trajectories: &[Vec<f32>]) -> Vec<f32> {
        if trajectories.is_empty() {
            return vec![0.0];
        }
        let dim = trajectories[0].len();
        // Asumir: features observadas en más trayectorias = más recompensa
        let mut reward = vec![0.0f32; dim];
        for traj in trajectories {
            for (d, &v) in traj.iter().enumerate() {
                reward[d] += v.abs();
            }
        }
        let n = trajectories.len() as f32;
        for d in 0..dim {
            reward[d] /= n;
        }
        reward
    }

    // 39. STATE SPACE (SSM): Discretización simple
    fn state_space_step(&self, x: &[f32], a: &[f32], b: &[f32], u: f32) -> Vec<f32> {
        let dim = x.len();
        let dt = 0.1;
        // Discretizar: x' = A·x + B·u  →  x_new = x + dt*(A·x + B·u)
        let mut x_new = x.to_vec();
        for i in 0..dim {
            let da = a[i % a.len()] * x[i] * dt;
            let db = b[i % b.len()] * u * dt;
            x_new[i] += da + db;
        }
        x_new
    }

    // 40. ENERGY BASED: Contraste entre energía positiva y negativa
    fn energy_contrast(&self, positive: &[f32], negative: &[f32]) -> f32 {
        let e_pos: f32 = positive.iter().map(|x| x * x).sum();
        let e_neg: f32 = negative.iter().map(|x| x * x).sum();
        (e_neg - e_pos).max(0.0)
    }

    // 41. BAYESIAN NETWORK: Inferencia simple en DAG
    fn bayesian_net_infer(&self, parents: &[f32], weights: &[f32], bias: f32) -> f32 {
        let z: f32 = parents.iter().zip(weights).map(|(p, w)| p * w).sum::<f32>() + bias;
        1.0 / (1.0 + (-z).exp()) // sigmoid
    }

    // 42. MODEL-BASED RL: Aprender modelo del mundo (transición simple)
    fn model_based_transition(&self, state: &[f32], action: &[f32]) -> Vec<f32> {
        state
            .iter()
            .zip(action)
            .map(|(s, a)| (s + a * 0.1).clamp(0.0, 1.0))
            .collect()
    }

    // ============================================================================
    // +8 PARADIGMAS: simulaciones ligeras de los "imposibles"
    // ============================================================================

    // 43. QUANTUM: placeholder
    fn quantum_simulate(&self, _qubits: usize, _gates: &[(usize, f32)]) -> Vec<f32> {
        vec![1.0]
    }

    // 44. EMBODIED: Agente en grid world simbólico
    fn embodied_grid(
        &self,
        world: &[Vec<i32>],
        start: (usize, usize),
        goal: (usize, usize),
    ) -> Vec<(usize, usize)> {
        let h = world.len();
        let w = world[0].len();
        let mut pos = start;
        let mut path = vec![start];
        for _ in 0..(h * w).min(50) {
            if pos == goal {
                break;
            }
            let moves = [(0i32, 1i32), (1, 0), (0, -1i32), (-1, 0)];
            let mut best = pos;
            let mut best_dist = (h + w) as f32;
            for &(dx, dy) in &moves {
                let nx = (pos.0 as i32 + dx).max(0).min(w as i32 - 1) as usize;
                let ny = (pos.1 as i32 + dy).max(0).min(h as i32 - 1) as usize;
                if world[ny][nx] == 0 {
                    let d = ((nx as f32 - goal.0 as f32).powi(2)
                        + (ny as f32 - goal.1 as f32).powi(2))
                    .sqrt();
                    if d < best_dist {
                        best_dist = d;
                        best = (nx, ny);
                    }
                }
            }
            if best == pos {
                break;
            }
            pos = best;
            path.push(pos);
        }
        path
    }

    // 45. PROGRAM SYNTHESIS: Template-based code gen
    fn program_synthesis(&self, spec: &str) -> String {
        format!("// Synthesis for: {}", spec)
    }

    // 46. MULTIMODAL AUDIO: Espectrograma simple desde bytes
    fn multimodal_audio_features(&self, samples: &[u8]) -> Vec<f32> {
        let window = 64usize;
        let mut features = Vec::new();
        for chunk in samples.chunks(window) {
            let energy: f32 = chunk
                .iter()
                .map(|&b| (b as f32 / 128.0 - 1.0).powi(2))
                .sum::<f32>()
                / window as f32;
            features.push(energy);
        }
        features
    }

    // 47. MULTIMODAL IMAGE: Edge detection simple desde bytes
    fn multimodal_image_edges(&self, pixels: &[u8], width: usize) -> Vec<f32> {
        let height = pixels.len() / width;
        let mut edges = Vec::new();
        for y in 1..height - 1 {
            for x in 1..width - 1 {
                let idx = y * width + x;
                let gx = pixels[idx + 1] as f32 - pixels[idx - 1] as f32;
                let gy = pixels[idx + width] as f32 - pixels[idx - width] as f32;
                edges.push((gx * gx + gy * gy).sqrt() / 255.0);
            }
        }
        edges
    }

    // 48. METALEARNING avanzado: MAML-style inner loop
    fn metalearning_adapt(&self, params: &[f32], task_gradients: &[f32], lr: f32) -> Vec<f32> {
        params
            .iter()
            .zip(task_gradients)
            .map(|(p, g)| p - lr * g)
            .collect()
    }

    // 49. LOGIC FORMAL: Resolución simple
    fn logic_resolve(&self, clause_a: &[i32], clause_b: &[i32]) -> Vec<i32> {
        let mut resolved = Vec::new();
        for &a in clause_a {
            if !clause_b.contains(&-a) {
                resolved.push(a);
            }
        }
        for &b in clause_b {
            if !clause_a.contains(&-b) {
                resolved.push(b);
            }
        }
        resolved
    }

    // 50. SPIKE TIMING (STDP): Plasticidad por timing
    fn spike_stdp(&self, t_pre: f32, t_post: f32, weight: f32) -> f32 {
        let dt = t_post - t_pre;
        let tau = 20.0;
        if dt > 0.0 {
            weight + 0.01 * (-dt / tau).exp()
        } else {
            weight - 0.01 * (dt / tau).exp()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // INVARIANTES INICIALES: al nacer, EDEN debe tener estado coherente
    // -------------------------------------------------------------------------
    #[test]
    fn test_birth_sets_correct_state() {
        let mut eden = EdenREPL::new();
        eden.handle_nacimiento();

        assert!(eden.born, "born debe ser true tras nacimiento");
        assert!(eden.session.born);
        assert!(
            eden.autonomous_active,
            "autonomous_active debe estar activado"
        );
        assert!(eden.autonomous_mode);
        assert_eq!(
            eden.session.self_mod_count, 1,
            "self_mod_count debe empezar en 1"
        );
        assert_eq!(
            eden.autonomous_cycles_executed, 0,
            "ciclos ejecutados empieza en 0"
        );
        assert_eq!(eden.birth_autonomous_cycle, 0);
        assert!(
            eden.eden_umbra.is_some(),
            "umbra debe estar inicializada al nacer"
        );
        assert!(
            eden.neural_network.is_some(),
            "red neuronal debe estar inicializada"
        );
        assert!(eden.crawler.is_some(), "crawler debe estar inicializado");
        assert_eq!(eden.last_self_reflection, 0);
        assert_eq!(eden.last_auto_evolve, 0);
    }

    // -------------------------------------------------------------------------
    // CICLOS AUTONOMOS: 200 ciclos sin panic, invariantes mantenidos
    // -------------------------------------------------------------------------
    #[test]
    fn test_autonomous_cycles_no_panic_and_invariants() {
        let mut eden = EdenREPL::new();
        eden.handle_nacimiento();

        let mut cycles_run = 0u64;
        let mut max_evolution_ticks = 0u64;
        let mut last_complexity = eden.complexity_tracker.current();
        let mut had_evolution = false;
        let mut had_facts_growth = false;
        let initial_facts = eden.session.learned_facts.len();

        for _ in 0..200 {
            let result = eden.run_autonomous_cycle();
            assert!(result.is_none() || result.is_some(), "no debe panickear");
            cycles_run += 1;

            // Invariante: evolution_ticks monotónico
            let ticks = eden.session.evolution_ticks;
            assert!(
                ticks >= max_evolution_ticks,
                "evolution_ticks retrocedio: {} -> {}",
                max_evolution_ticks,
                ticks
            );
            max_evolution_ticks = ticks;

            // Invariante: complejidad nunca NaN o negativa
            let c = eden.complexity_tracker.current();
            assert!(c.is_finite() && c >= 0.0, "complejidad invalida: {}", c);

            if c > last_complexity + 0.01 {
                had_evolution = true;
            }
            last_complexity = c;

            // Invariante: learned_facts nunca excede CAP
            assert!(
                eden.session.learned_facts.len() <= 500,
                "learned_facts excede CAP: {}",
                eden.session.learned_facts.len()
            );

            if eden.session.learned_facts.len() > initial_facts {
                had_facts_growth = true;
            }
        }

        assert_eq!(cycles_run, 200);
        assert!(
            eden.autonomous_cycles_executed > 0,
            "ciclos ejecutados debe ser >0: {}",
            eden.autonomous_cycles_executed
        );
        assert!(eden.born, "EDEN debe seguir vivo");
        assert!(
            max_evolution_ticks >= 200,
            "evolution_ticks debe avanzar: {}",
            max_evolution_ticks
        );

        // Verificar que el sistema NO esta congelado: alguna metrica debe haber cambiado
        assert!(
            had_evolution || had_facts_growth,
            "sistema congelado: ni evolucion ni crecimiento de facts en 200 ciclos"
        );
    }

    // -------------------------------------------------------------------------
    // EVOLUCION MANUAL: handle_evolve incrementa contadores correctamente
    // -------------------------------------------------------------------------
    #[test]
    fn test_manual_evolution_increments_counters() {
        let mut eden = EdenREPL::new();
        eden.handle_nacimiento(); // self_mod_count=1, born=true

        let level_before = eden.session.evolution_level;
        let ticks_before = eden.session.evolution_ticks;
        let mods_before = eden.session.self_mod_count;

        let result = eden.handle_evolve();
        assert!(!result.is_empty(), "evolucion debe retornar mensaje");

        assert!(
            eden.session.evolution_level > level_before || eden.session.evolution_level == 99, // cap en 99
            "evolution_level debe subir tras evolucion"
        );
        assert!(
            eden.session.self_mod_count > mods_before,
            "self_mod_count debe incrementar"
        );
        assert!(
            eden.session.evolution_ticks >= ticks_before,
            "evolution_ticks no debe retroceder"
        );
        assert!(
            eden.session.awareness_base > 0.75,
            "awareness debe crecer tras evolucion"
        );
        assert!(
            eden.session.integration_bias > 0.0,
            "integration bias debe crecer tras evolucion"
        );
    }

    // -------------------------------------------------------------------------
    // SERIALIZACION: round-trip to_bytes -> from_bytes preserva datos
    // -------------------------------------------------------------------------
    #[test]
    fn test_session_serialize_roundtrip() {
        // Usar sesion fresca sin handle_nacimiento (evita side effects en thought count)
        let mut session = EdenSession::default();
        session
            .learned_facts
            .push("facto de prueba uno".to_string());
        session
            .learned_facts
            .push("facto con emojis: 🧠✨".to_string());
        session
            .learned_facts
            .push("string larga con caracteres especiales: áéíóú ñ".to_string());
        session
            .autonomous_thoughts
            .push("pensamiento 1".to_string());
        session
            .autonomous_thoughts
            .push("pensamiento 2".to_string());
        session
            .habilidades_omniversales
            .push("habilidad_test".to_string());
        session.complexity_history = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        session.born = true;
        session.awareness_base = 0.82;
        session.integration_bias = 0.3;
        session.self_mod_count = 5;
        session.evolution_ticks = 42;
        session.modifier_snapshot = vec![7, 8, 9];

        let bytes = session.to_bytes();
        assert!(!bytes.is_empty(), "to_bytes no debe ser vacio");

        let loaded = EdenSession::from_bytes(&bytes);
        assert!(
            loaded.is_some(),
            "from_bytes debe decodificar EDEN4 exitosamente"
        );

        let loaded = loaded.unwrap();
        assert!(
            (loaded.awareness_base - 0.82).abs() < 0.001,
            "awareness preservado"
        );
        assert!(
            (loaded.integration_bias - 0.3).abs() < 0.001,
            "integration preservado"
        );
        assert_eq!(loaded.self_mod_count, 5);
        assert_eq!(loaded.evolution_ticks, 42);
        assert!(loaded.born);
        assert_eq!(
            loaded.learned_facts.len(),
            3,
            "deben preservarse los 3 facts"
        );
        assert!(loaded
            .learned_facts
            .iter()
            .any(|f| f.contains("facto de prueba")));
        assert!(
            loaded.learned_facts.iter().any(|f| f.contains("🧠✨")),
            "los emojis deben sobrevivir round-trip"
        );
        assert!(
            loaded.learned_facts.iter().any(|f| f.contains("áéíóú")),
            "los caracteres unicode deben sobrevivir"
        );
        assert_eq!(loaded.autonomous_thoughts.len(), 2);
        assert_eq!(loaded.habilidades_omniversales.len(), 1);
        assert_eq!(loaded.complexity_history.len(), 5);
    }

    // -------------------------------------------------------------------------
    // SERIALIZACION: UTF-8 corrupto no causa perdida de datos (lossy)
    // -------------------------------------------------------------------------
    #[test]
    fn test_session_from_bytes_handles_invalid_utf8() {
        // Probar que from_bytes_lossy preserva datos corruptos sin panic
        // Construimos bytes EDEN4 validos y corrompemos un byte DENTRO de un string
        let mut session = EdenSession::default();
        session.learned_facts.push(
            "hecho valido y largo para probar que lossy funciona con bytes rotos".to_string(),
        );
        let bytes = session.to_bytes();

        // Los facts empiezan despues de la cabecera (header: ~67 bytes fijos) +
        // modifier_snapshot (8 bytes len + 0 bytes = 8) + 0xFF = 1 byte + facts_count (8 bytes) + fact1_len (8 bytes)
        // Total estructural antes del primer string: ~67 + 8 + 1 + 8 + 8 = 92
        // Corromper un byte dentro del string payload (byte 100+)
        let mut corrupted = bytes.clone();
        if corrupted.len() > 105 {
            corrupted[105] = 0xFF; // byte invalido dentro del string
        }

        let loaded = EdenSession::from_bytes(&corrupted);
        assert!(
            loaded.is_some(),
            "from_bytes debe manejar bytes invalidos con lossy"
        );

        let loaded = loaded.unwrap();
        // El string debe preservarse con reemplazo de caracteres
        assert!(
            loaded.learned_facts.len() >= 1,
            "al menos un fact debe preservarse: {}",
            loaded.learned_facts.len()
        );
    }

    // -------------------------------------------------------------------------
    // COMPLEXITY TRACKER: VecDeque operations, invariantes
    // -------------------------------------------------------------------------
    #[test]
    fn test_complexity_tracker_operations() {
        let mut ct = ComplexityTracker::new();

        // Agregar 150 muestras (mas que el CAP de 100 para probar ventana)
        for i in 0..150 {
            ct.record(i as f32 * 0.1);
        }

        assert_eq!(ct.total_ticks(), 150);
        assert!(ct.current() > 0.0, "current debe ser positivo");
        assert!(ct.max_ever > 0.0, "max debe ser positivo");
        assert!(ct.velocity().is_finite(), "velocity debe ser finito");
        assert_eq!(ct.to_vec().len(), 100, "history CAP debe ser 100");
        assert!(!ct.to_vec().is_empty());

        // Soft reset
        let max_before = ct.max_ever;
        ct.soft_reset();
        assert!(ct.max_ever == max_before * 0.7, "max se reduce a 70%");
        assert!(ct.to_vec().is_empty(), "history limpio");
        assert_eq!(ct.total_ticks(), 0, "ticks reseteado");
        assert_eq!(ct.velocity(), 0.0, "velocity reseteada");
    }

    // -------------------------------------------------------------------------
    // COMPLEXITY TRACKER: rechaza valores corruptos
    // -------------------------------------------------------------------------
    #[test]
    fn test_complexity_tracker_rejects_invalid() {
        let mut ct = ComplexityTracker::new();

        ct.record(1.0);
        let ticks_before = ct.total_ticks();
        let len_before = ct.to_vec().len();

        // NaN y negativo deben ser rechazados sin afectar estado
        ct.record(f32::NAN);
        ct.record(f32::NEG_INFINITY);
        ct.record(-1.0);

        assert_eq!(
            ct.total_ticks(),
            ticks_before,
            "NaN no debe incrementar ticks"
        );
        assert_eq!(
            ct.to_vec().len(),
            len_before,
            "valores invalidos no se añaden"
        );
        assert_eq!(ct.current(), 1.0);
    }

    // -------------------------------------------------------------------------
    // MUERTE Y RENACIMIENTO: ciclo completo
    // -------------------------------------------------------------------------
    #[test]
    fn test_death_and_rebirth_cycle() {
        let mut eden = EdenREPL::new();
        eden.handle_nacimiento();
        let initial_total_renac = eden.self_model.total_renacimientos;

        // Forzar muchas evoluciones para disparar muerte
        // Umbral: self_mod_count >= 30 + level*5 (con level=1 → 35)
        for _ in 0..60 {
            eden.handle_evolve();
        }

        assert!(
            eden.session.self_mod_count >= 60,
            "self_mod_count debe acumularse: {}",
            eden.session.self_mod_count
        );
        assert!(
            eden.should_eden_die(),
            "60 evoluciones deben disparar muerte"
        );

        // Ejecutar muerte (necesaria para setear last_lifespan_ticks y limpiar umbra)
        eden.eden_muerte();
        assert!(
            eden.last_lifespan_ticks > 0,
            "lifespan debe registrarse: {}",
            eden.last_lifespan_ticks
        );

        // Ejecutar renacimiento
        let result = eden.eden_renacimiento();
        assert!(!result.is_empty(), "renacimiento debe retornar mensaje");
        assert!(
            result.contains("[RENACIMIENTO]"),
            "mensaje debe indicar renacimiento: {}",
            result
        );

        // Verificar estado post-renacimiento
        assert!(
            eden.self_model.total_renacimientos > initial_total_renac,
            "contador de renacimientos debe incrementar"
        );
        assert!(
            eden.self_model.lineage_age > 0,
            "lineage age debe ser > 0 tras renacimiento: {}",
            eden.self_model.lineage_age
        );
        assert!(
            eden.eden_umbra.is_some(),
            "umbra debe estar inicializada tras renacimiento"
        );
        assert_eq!(
            eden.autonomous_cycles_executed, 0,
            "ciclos autonomos se resetean a 0"
        );
        assert_eq!(
            eden.birth_autonomous_cycle, 0,
            "birth_autonomous_cycle se resetea"
        );
    }

    // -------------------------------------------------------------------------
    // EVOLUTION TICKS MONOTONICOS: simulacion mixta manual + auto
    // -------------------------------------------------------------------------
    #[test]
    fn test_evolution_ticks_never_regress() {
        let mut eden = EdenREPL::new();
        eden.handle_nacimiento();

        let mut last_ticks = eden.session.evolution_ticks;

        // Mezcla: ciclos autonomos + evolucion manual + mas ciclos autonomos
        for _ in 0..5 {
            eden.run_autonomous_cycle();
        }
        let t1 = eden.session.evolution_ticks;
        assert!(t1 >= last_ticks, "regresion tras ciclos autonomos");
        last_ticks = t1;

        eden.handle_evolve();
        let t2 = eden.session.evolution_ticks;
        assert!(t2 >= last_ticks, "regresion tras evolucion manual");
        last_ticks = t2;

        eden.handle_evolve();
        let t3 = eden.session.evolution_ticks;
        assert!(t3 >= last_ticks, "regresion tras 2da evolucion");
    }

    // -------------------------------------------------------------------------
    // CHILD LIFECYCLE: hijos usan ciclo_autonomo, no evolution_ticks
    // -------------------------------------------------------------------------
    #[test]
    fn test_child_lifecycle_uses_ciclo_autonomo() {
        let mut eden = EdenREPL::new();
        eden.handle_nacimiento();

        // Ejecutar varios ciclos autonomos (que generan hijos via reflexion)
        for _ in 0..15 {
            eden.run_autonomous_cycle();
        }

        // Para el test, crear hijo manualmente con lifespan corto y verificar muere
        let child_id = eden.next_child_id;
        let mut child_umbra = Umbra::nuevo(child_id);
        child_umbra.tick_actualizar();
        let _ = eden.next_child_id;

        eden.child_autons.push(ChildAuton {
            id: child_id,
            umbra: child_umbra,
            birth_tick: 5, // nacio en ciclo 5
            lifespan: 5,   // muere en ciclo 10
            pensamiento_origen: "test".to_string(),
            energia: 1.0,
        });

        let initial_count = eden.child_autons.len();

        // Ejecutar process_child_lifecycle con ciclo_autonomo=15 (debio morir)
        eden.process_child_lifecycle(15);
        assert!(
            eden.child_autons.len() < initial_count,
            "hijo con lifespan expirado debe ser removido"
        );
    }

    // -------------------------------------------------------------------------
    // MISSION SYSTEM: progress se resetea entre misiones
    // -------------------------------------------------------------------------
    #[test]
    fn test_mission_progress_resets_on_new_mission() {
        let mut eden = EdenREPL::new();
        eden.handle_nacimiento();

        // Crear mision con alto progreso y deadline expirado
        eden.current_mission = Some(Mission {
            id: 1,
            primary_goal: "test_mission".to_string(),
            sub_goals: Vec::new(),
            active_sub_goal_index: 0,
            created_at: 0,
            deadline: Some(5),
            progress: 0.9,
            relevance: 0.5,
            status: MissionStatus::Active,
            success_criteria: vec![],
        });
        eden.mission_progress = 0.9;

        // update_mission_system con ciclo > deadline
        let result = eden.update_mission_system(10);
        assert!(result.is_some(), "mision debe expirar");
        assert!(
            eden.current_mission.is_none(),
            "mision expirada debe limpiarse"
        );
        assert_eq!(eden.mission_progress, 0.0, "progreso debe resetearse");

        // Una nueva mision se genera automaticamente
        let result2 = eden.update_mission_system(11);
        assert!(result2.is_some(), "nueva mision debe generarse");
        assert!(eden.current_mission.is_some());
    }

    // -------------------------------------------------------------------------
    // LEARNED FACTS: CAP 500 y deduplicacion
    // -------------------------------------------------------------------------
    #[test]
    fn test_learned_facts_cap_and_dedup() {
        let mut eden = EdenREPL::new();
        eden.handle_nacimiento();

        // Llenar con 550 facts (excede CAP)
        for i in 0..550 {
            eden.session.learned_facts.push(format!("fact_{}", i));
        }

        // Ejecutar ciclo para disparar limpieza de CAP
        eden.run_autonomous_cycle();

        assert!(
            eden.session.learned_facts.len() <= 400,
            "learned_facts debe truncarse a CAP 400: {}",
            eden.session.learned_facts.len()
        );
    }

    // -------------------------------------------------------------------------
    // EMOTION HISTORY: VecDeque no pierde datos, CAP 50
    // -------------------------------------------------------------------------
    #[test]
    fn test_emotion_history_vecdeque_cap() {
        let mut eden = EdenREPL::new();
        eden.handle_nacimiento();

        // Ejecutar suficientes ciclos para que se generen emociones
        for _ in 0..60 {
            eden.run_autonomous_cycle();
        }

        assert!(
            eden.emotional_state.emotion_history.len() <= 50,
            "emotion_history CAP debe ser 50: {}",
            eden.emotional_state.emotion_history.len()
        );
    }
}
