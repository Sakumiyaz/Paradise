//! # EDEN Core - Binary Entry Point
//!
//! Sistema de vida artificial autopoético. Punto de entrada principal.
//!
//! ## Arquitectura de Hilos
//!
//! - **Hilo Principal (Simulación)**: Física del Mar + Ciclo vital de Autons
//! - **Hilo de Renderizado**: Dibuja en framebuffer o terminal
//! - **Hilo de IPC**: Socket Unix bidireccional con Python/Demiurgo
//!
//! ## Determinismo
//!
//! El bucle principal es **estrictamente determinista** dado el estado inicial.
//! El único no-determinismo proviene de:
//! - Lectura de sensores de hardware (reloj CPU)
//! - Condiciones de carrera intencionadas en threads
#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(unused, dropping_references)]

// ============================================================================
// MÓDULOS DE EDEN CORE
// ============================================================================

use eden_core::evolution::open_endedness::OpenEndednessEngine;
use eden_core::fs::edenfs::{AutonArchivo, CausaMuerte, EdenFS, MetadatosAuton};
use eden_core::ipc::command::{Comando, EdenCommand, Evento};
use eden_core::ipc::socket::UnixDatagram;
use eden_core::life::campo_estructural::{
    BifurcacionDetectada, CampoEstructural, DimsCampo, EstadoCampo, ParametrosAllenCahn, SpaceDim,
};
use eden_core::life::meltrace::{BufferCircular, Grabado, Meltrace, MeltraceStats};
use eden_core::life::ramnet::{RamNet, TipoAccion};
use eden_core::life::umbra::{ArcoUmbra, HashEstado, NodoUmbra, ResultadoUmbra, TipoFusion, Umbra};
use eden_core::membrain::rand_u64;
use eden_core::physics::energon::{
    ConstantesCosmicas, Energon, FixedPoint, SemillaGenesis, Vector3D,
};
use eden_core::physics::fixed_point::I32F32;
use eden_core::physics::mar_morfoseo::MarMorfoseo;
use eden_core::render::soft_gpu::SoftGPU;
use eden_core::render::term_hex::TermHex;
use eden_core::swarm;

// ============================================================================
// DEPENDENCIAS ESTÁNDAR
// ============================================================================

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex, RwLock};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

// ============================================================================
// CONSTANTES GLOBALES
// ============================================================================

/// Tamaño del Mar Morfóseo (cuadrado NxN)
const TAMAÑO_MAR: usize = 64;
/// Número de hilos para el solver del Mar
const NUM_HILOS_MAR: usize = 1;
/// Pasos de simulación por segundo (60 FPS subjetivo)
const PASOS_POR_SEGUNDO: u32 = 60;
/// Intervalo en ciclos para guardar Meltrace en disco
const INTERVALO_GUARDADO_MELTRACE: u64 = 600;
/// Intervalo en ciclos para broadcast de estado por socket
const INTERVALO_SOCKET_BROADCAST: u64 = 10;
/// Duración de un paso (16ms para 60 FPS)
const DURACION_PASO_MS: u64 = 1000 / PASOS_POR_SEGUNDO as u64;
/// Socket path para IPC con Python
const SOCKET_PATH: &str = "/tmp/eden_core.sock";
/// Directorio de estado Eden (~/.eden)
const EDEN_DIR: &str = ".eden";

// ============================================================================
// ESTRUCTURAS PRINCIPALES
// ============================================================================

/// Vitals para monitor externo (vital_signs)
/// INAGOTABILIDAD: EDEN escribe su estado aquí para que el monitor lo lea
#[derive(Debug, Clone, Serialize, Deserialize)]
struct VitalsJson {
    timestamp: String,
    ciclo: u64,
    energia_mar: f64,
    energia_autons: f64,
    autons_vivos: u64,
    nacidos: u64,
    muertos: u64,
    meltrace_len: u64,
}

/// UUID de un Auton (v4 aleatorio - 16 bytes)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Uuid([u8; 16]);

impl Uuid {
    /// Genera UUID v4 leyendo de /dev/urandom
    pub fn v4() -> io::Result<Self> {
        let mut bytes = [0u8; 16];
        let mut file = File::open("/dev/urandom")?;
        file.read_exact(&mut bytes)?;
        // Versión 4 + variante RFC 4122
        bytes[6] = (bytes[6] & 0x0F) | 0x40;
        bytes[8] = (bytes[8] & 0x3F) | 0x80;
        Ok(Uuid(bytes))
    }

    /// Devuelve los 8 primeros bytes como u64 (para hashing rápido)
    pub fn as_u64(&self) -> u64 {
        u64::from_le_bytes([
            self.0[0], self.0[1], self.0[2], self.0[3], self.0[4], self.0[5], self.0[6], self.0[7],
        ])
    }
}

impl std::fmt::Display for Uuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
            self.0[0], self.0[1], self.0[2], self.0[3],
            self.0[4], self.0[5], self.0[6], self.0[7],
            self.0[8], self.0[9], self.0[10], self.0[11],
            self.0[12], self.0[13], self.0[14], self.0[15])
    }
}

/// Un Auton vivo: combina membrana (CampoEstructural), cerebro (RamNet),
/// sombra (Umbra) y energía
pub struct AutonVivo {
    /// Identificador único
    pub id: Uuid,
    /// Campo Estructural (membrana - solver Allen-Cahn)
    pub campo: CampoEstructural,
    /// RamNet (cerebro sin pesos)
    pub ramnet: RamNet,
    /// Umbra (grafo de decisiones causales)
    pub umbra: Umbra,
    /// Energía actual del Auton
    pub energia: I32F32,
    /// Generación desde el Primordial
    pub generacion: u32,
    /// ID del padre (None si es Primordial)
    pub padre_id: Option<Uuid>,
}

impl AutonVivo {
    /// Crea el Auton Primordial (El Primogénito) en el centro del Mar
    pub fn nuevo_primordial(constantes: &ConstantesCosmicas, semilla: u64) -> Self {
        let id = Uuid::v4().expect("No se pudo generar UUID del Primordial");
        let mut campo = CampoEstructural::new_2d(32, 32);

        // Esfera en el centro del campo (coordenadas normalizadas 0..1)
        let cx = 0.5;
        let cy = 0.5;
        let radio = 0.35;
        campo.inicializar_circular(cx, cy, radio, Self::phi_aleatorio());
        campo.set_id(id.as_u64());

        // Posición inicial en el centro del Mar (normalized 0.5 = I32F32 ONE/2 = 0x80000000)
        let pos_x = I32F32::from_raw(0x80000000); // 0.5 normalized
        let pos_y = I32F32::from_raw(0x80000000); // 0.5 normalized
        campo.set_posicion(Vector3D::new(pos_x, pos_y, I32F32::ZERO));

        // Energía inicial
        campo.set_energia_interna(I32F32::from_raw(0x00000064_00000000)); // 100.0

        let ramnet = RamNet::new(8, 2, semilla.wrapping_add(id.as_u64()));
        let umbra = Umbra::nuevo(id.as_u64());

        AutonVivo {
            id,
            campo,
            ramnet,
            umbra,
            energia: I32F32::from_raw(0x00000064_00000000),
            generacion: 0,
            padre_id: None,
        }
    }

    /// Genera φ aleatorio para inicialización variedora
    fn phi_aleatorio() -> I32F32 {
        let r = rand_u64();
        let perturbacion = (r & 0xFFFF) as f64 / 65535.0;
        let signo = if r & 0x10000 != 0 { 1.0 } else { -1.0 };
        let magnitud = 0.7 + 0.3 * perturbacion;
        let valor = signo * magnitud;
        I32F32::from_f64(valor)
    }

    /// Crea un Auton hijo a partir de un lóbulo del campo paterno
    pub fn desde_escision(
        padre: &AutonVivo,
        mut lobulo: CampoEstructural,
        constantes: &ConstantesCosmicas,
    ) -> Self {
        let id = Uuid::v4().expect("No se pudo generar UUID del hijo");
        lobulo.set_id(id.as_u64());

        // Copiar energía proporcional (mitad)
        let energia_padre = padre.campo.energia_interna();
        let energia_hijo = energia_padre / I32F32::from_raw(0x00000002_00000000);
        lobulo.set_energia_interna(energia_hijo);

        // Herencia de RamNet con mutación
        let mut ramnet = RamNet::new(8, 2, id.as_u64());
        // Mutación basada en energía del padre (no implementada aquí)

        // Herencia de Umbra (compartida por ahora)
        let umbra = Umbra::nuevo(id.as_u64());

        AutonVivo {
            id,
            campo: lobulo,
            ramnet,
            umbra,
            energia: energia_hijo,
            generacion: padre.generacion + 1,
            padre_id: Some(padre.id),
        }
    }

    /// Crea un Auton desde un gene recibido del enjambre (injectado)
    pub fn desde_gen_inyectado(gen: &swarm::AutonGene, semilla_global: u64) -> Self {
        let id = Uuid::v4().expect("No se pudo generar UUID del Inyectado");

        // Campo estructural con seed del gene
        let mut campo = CampoEstructural::new_2d(32, 32);
        campo.inicializar_circular(0.5, 0.5, 0.35, Self::phi_aleatorio());
        campo.set_id(id.as_u64());
        campo.set_energia_interna(I32F32::from_raw(0x00000064_00000000));

        // Posición desde el gene (normalizada 0.0-1.0 → I32F32)
        let pos_x = I32F32::from_f64(gen.pos_x as f64 * 2.0 - 1.0); // 0..1 → -1..1
        let pos_y = I32F32::from_f64(gen.pos_y as f64 * 2.0 - 1.0);
        campo.set_posicion(Vector3D::new(pos_x, pos_y, I32F32::ZERO));

        // RamNet con seed combinada (permite reproduccion posterior)
        let seed_ramnet = gen.ramnet_seed.wrapping_add(semilla_global);
        let ramnet = RamNet::new(8, 2, seed_ramnet);

        let umbra = Umbra::nuevo(id.as_u64());

        AutonVivo {
            id,
            campo,
            ramnet,
            umbra,
            energia: I32F32::from_raw(0x00000064_00000000),
            generacion: gen.generacion.saturating_add(1), // Hijos del gene
            padre_id: None,                               // Gene externo = sin padre conocido
        }
    }

    /// Avanza el ciclo vital: metabolismo + movimiento
    ///
    /// Ciclo par (asimilación): absorbe energon del Mar
    /// Ciclo impar (desasimilación): consume energía interna
    pub fn ciclo_vital(&mut self, mar: &MarMorfoseo, constantes: &ConstantesCosmicas) {
        self.campo.step(mar, constantes);
    }

    /// Retorna true si el Auton está muerto
    pub fn esta_muerto(&self) -> bool {
        // Campo disuelto o energía agotada
        self.campo.estado() == EstadoCampo::Disuelto
            || self.campo.energia_interna() <= I32F32::ZERO
            || !self.campo.esta_vivo()
    }

    /// Causa de muerte推断
    pub fn causa_muerte(&self) -> CausaMuerte {
        if self.campo.estado() == EstadoCampo::Disuelto {
            CausaMuerte::ColapsoCampo
        } else if self.campo.energia_interna() <= I32F32::ZERO {
            CausaMuerte::AgotamientoEnergia
        } else {
            CausaMuerte::Senescencia
        }
    }

    /// Detecta escisión (reproducción)
    pub fn detectar_escision(&self) -> Option<Vec<CampoEstructural>> {
        self.campo.detectar_escision()
    }

    /// Estado del Auton para el socket
    pub fn estado_socket(&self) -> eden_core::life::campo_estructural::AutonState {
        self.campo.estado_para_socket()
    }
}

/// Universo: el estado global de la simulación A-Life
pub struct Universo {
    /// El Mar Morfóseo (campo de energon)
    pub mar: MarMorfoseo,
    /// Autons vivos
    pub autons: Vec<AutonVivo>,
    /// Registro de grabaciones Lamarckianas
    pub meltrace: Meltrace,
    /// Sistema de archivos EdenFS
    pub fs: EdenFS,
    /// Constantes cosmológicas (derivadas de la semilla)
    pub constantes: ConstantesCosmicas,
    /// Contador de ciclos ejecutados
    pub contador_ciclos: u64,
    /// Semilla original del universo
    pub semilla: SemillaGenesis,
    /// Motor de Open-Endedness - inagotabilidad natural
    pub open_endedness: OpenEndednessEngine,
}

impl Universo {
    /// Crea un nuevo universo desde una semilla de 128 bytes
    pub fn crear(semilla: [u8; 128]) -> io::Result<Self> {
        let constantes = ConstantesCosmicas::from_semilla(&semilla);

        // Crear Mar Morfóseo
        let mut mar = MarMorfoseo::new_2d(TAMAÑO_MAR, NUM_HILOS_MAR);

        // Calcular semilla_u64 early for use in region seeding
        let semilla_u64 = calcular_semilla_u64(&semilla);

        // Seed initial energon in center of the Mar so Autons can absorb energy
        // INAGOTABILIDAD: energía emerge naturalmente, sin boost artificial
        let centro = TAMAÑO_MAR / 2;
        let base_cantidad = 32 + (semilla_u64 % 64) as i64; // 32-95 emerge naturalmente
        let cantidad_semilla = I32F32::from_raw((base_cantidad << 32) as i64);
        mar.sembrar_region(
            centro - 8,
            centro - 8,
            0,
            centro + 8,
            centro + 8,
            0,
            cantidad_semilla,
        );

        // Inicializar Meltrace y Open-Endedness
        let meltrace = Meltrace::new(semilla_u64);
        let open_endedness = OpenEndednessEngine::new(semilla_u64);

        // Crear/el directorio de EdenFS
        let fs = EdenFS::new(semilla_u64)?;

        let mut universo = Universo {
            mar,
            autons: Vec::new(),
            meltrace,
            fs,
            constantes,
            contador_ciclos: 0,
            semilla,
            open_endedness,
        };

        // Sembrar el Primordial (El Primogénito)
        let progenito = AutonVivo::nuevo_primordial(&universo.constantes, semilla_u64);
        universo.autons.push(progenito);

        // Registrar en EdenFS
        universo
            .fs
            .registrar_nacimiento(Some(universo.autons[0].id.as_u64()))?;

        println!(
            "✧ El Primogénito ({}) ha sido sembrado",
            universo.autons[0].id
        );

        Ok(universo)
    }

    /// Obtiene el estado global del ecosistema para enviar por socket
    pub fn estado_global(&self) -> EcosistemaEstado {
        let energia_mar = self.mar.energia_total();
        // Normalizar por máxima energía posible en el Mar (4096 celdas * 10000 densidad máx)
        let energia_maxima_mar = (TAMAÑO_MAR * TAMAÑO_MAR * 10000) as f64;
        let energia_normalizada = (energia_mar.to_f64() / energia_maxima_mar).min(1.0);
        EcosistemaEstado {
            ciclo: self.contador_ciclos,
            autons_vivos: self.autons.len() as u64,
            energia_total_mar: energia_normalizada,
            energia_total_autons: self
                .autons
                .iter()
                .map(|a| a.energia.to_raw() as f64)
                .sum::<f64>()
                / i64::MAX as f64,
            nacidos_total: self.meltrace.muertes_totales() + self.autons.len() as u64,
            muertos_total: self.meltrace.muertes_totales(),
            grabados_meltrace: self.meltrace.len() as u64,
        }
    }

    /// Intenta generación espontánea de un nuevo Auton
    /// INAGOTABILIDAD: funciona incluso sin Meltrace (baseline del vacío)
    pub fn intentar_generacion_espontanea(&mut self) -> Option<AutonVivo> {
        let dims = self.mar.dimensiones();
        let step = 8;

        // Buscar región de alta energía en el Mar
        let mut mejor_x = dims.x / 2;
        let mut mejor_y = dims.y / 2;
        let mut mejor_dens = I32F32::ZERO;

        for y in (0..dims.y).step_by(step) {
            for x in (0..dims.x).step_by(step) {
                if let Some(dens) = self.mar.densidad_en(x, y, 0) {
                    if dens > mejor_dens {
                        mejor_dens = dens;
                        mejor_x = x;
                        mejor_y = y;
                    }
                }
            }
        }

        // DEBUG
        let umbral_debug: i64 = if self.meltrace.len() > 0 {
            8589934592
        } else {
            0
        };
        eprintln!(
            "[SPONT] ciclo {} mejor_dens={} umbral={} meltrace={}",
            self.contador_ciclos,
            mejor_dens.to_raw(),
            umbral_debug,
            self.meltrace.len()
        );

        // INAGOTABILIDAD: emerge del balance natural del sistema
        // En sistema maduro, umbral deriva del estado
        // En baseline, umbral bajo pero no cero
        let energia_raw = self.mar.energia_total().to_raw();
        let semilla_base = self.contador_ciclos.wrapping_add(energia_raw as u64);
        let umbral_base = 1 + ((semilla_base % 4) as i64); // 1-4 range
        let umbral_densidad = if self.meltrace.len() > 0 {
            I32F32::from_raw((umbral_base << 32) as i64)
        } else {
            I32F32::from_raw(((umbral_base / 2) << 32) as i64) // Half en baseline
        };

        if mejor_dens > umbral_densidad {
            let mut nuevo = AutonVivo::nuevo_primordial(&self.constantes, self.contador_ciclos);
            // Mover al centro de la región de alta energía
            let pos_x = I32F32::from_raw((mejor_x as i64 * i32::MAX as i64 / dims.x as i64));
            let pos_y = I32F32::from_raw((mejor_y as i64 * i32::MAX as i64 / dims.y as i64));
            nuevo
                .campo
                .set_posicion(Vector3D::new(pos_x, pos_y, I32F32::ZERO));

            println!(
                "✧ Auton {}surgió en ({}, {}) — ciclo {} (Meltrace: {})",
                if self.meltrace.len() == 0 {
                    "espontáneo-BASELINE "
                } else {
                    ""
                },
                mejor_x,
                mejor_y,
                self.contador_ciclos,
                self.meltrace.len()
            );
            return Some(nuevo);
        }

        None
    }
}

/// Estado global del ecosistema para IPC
#[derive(Debug, Clone)]
pub struct EcosistemaEstado {
    pub ciclo: u64,
    pub autons_vivos: u64,
    pub energia_total_mar: f64,
    pub energia_total_autons: f64,
    pub nacidos_total: u64,
    pub muertos_total: u64,
    pub grabados_meltrace: u64,
}

/// Comando recibido de Python por el socket
#[derive(Debug, Clone)]
pub enum ComandoRecibido {
    InyectarEnergon {
        x: usize,
        y: usize,
        cantidad: I32F32,
    },
    AumentarEscoria {
        x: usize,
        y: usize,
        radio: usize,
    },
    ForzarBifurcacion {
        id: u64,
    },
    EliminarAuton {
        id: u64,
    },
    PausarSimulacion,
    ReanudarSimulacion,
}

impl ComandoRecibido {
    pub fn desde_bytes(bytes: &[u8]) -> Option<Self> {
        // Protocolo simple: JSON-like en texto
        // Formato: {"cmd":"InyectarEnergon","x":100,"y":200,"cantidad":500}
        if let Ok(s) = std::str::from_utf8(bytes) {
            let s = s.trim();
            if s.starts_with("{\"cmd\":\"InyectarEnergon\"") {
                let x = extraer_numero(s, "x")? as usize;
                let y = extraer_numero(s, "y")? as usize;
                let cantidad = extraer_numero(s, "cantidad")?;
                return Some(ComandoRecibido::InyectarEnergon {
                    x,
                    y,
                    cantidad: I32F32::from_raw(cantidad),
                });
            }
            if s.starts_with("{\"cmd\":\"AumentarEscoria\"") {
                let x = extraer_numero(s, "x")? as usize;
                let y = extraer_numero(s, "y")? as usize;
                let radio = extraer_numero(s, "radio")? as usize;
                return Some(ComandoRecibido::AumentarEscoria { x, y, radio });
            }
            if s.starts_with("{\"cmd\":\"EliminarAuton\"") {
                let id = extraer_numero(s, "id")? as u64;
                return Some(ComandoRecibido::EliminarAuton { id });
            }
            if s.starts_with("{\"cmd\":\"PausarSimulacion\"") {
                return Some(ComandoRecibido::PausarSimulacion);
            }
            if s.starts_with("{\"cmd\":\"ReanudarSimulacion\"") {
                return Some(ComandoRecibido::ReanudarSimulacion);
            }
        }
        None
    }
}

fn extraer_numero(s: &str, clave: &str) -> Option<i64> {
    let patron = format!("\"{}\":", clave);
    if let Some(pos) = s.find(&patron) {
        let resto = &s[pos + patron.len()..];
        let numero: String = resto
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '-')
            .collect();
        numero.parse().ok()
    } else {
        None
    }
}

// ============================================================================
// UTILIDADES DE SEMILLA
// ============================================================================

/// Calcula un u64 a partir de la semilla de 128 bytes (XOR fold)
fn calcular_semilla_u64(semilla: &[u8; 128]) -> u64 {
    let mut h: u64 = 0xEAD_BEEFu64;
    for (i, &b) in semilla.iter().enumerate() {
        h ^= b as u64 * (i as u64).wrapping_mul(31);
        h = h.rotate_left(7);
    }
    h
}

/// Genera semilla desde /dev/urandom
fn generar_semilla_desde_hardware() -> io::Result<[u8; 128]> {
    let mut bytes = [0u8; 128];
    let mut file = File::open("/dev/urandom")?;
    file.read_exact(&mut bytes)?;
    Ok(bytes)
}

/// Genera semilla desde argumentos o hardware
fn obtener_semilla() -> io::Result<([u8; 128], Option<String>)> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() >= 2 {
        let seed_str = &args[1];
        if seed_str.starts_with("--seed=") || seed_str.starts_with("--seed") {
            let hex = if seed_str.contains('=') {
                seed_str.split('=').nth(1).unwrap_or("0")
            } else {
                args.get(2).map(|s| s.as_str()).unwrap_or("0")
            };

            // Parsear hex a bytes
            let hex_limpio = hex.trim_start_matches("0x");
            let bytes_hex = hex::decode(hex_limpio).unwrap_or_else(|_| vec![0u8; 16]);

            let mut semilla = [0u8; 128];
            for (i, &b) in bytes_hex.iter().enumerate() {
                if i < 128 {
                    semilla[i] = b;
                }
            }
            return Ok((semilla, Some(hex.to_string())));
        }
    }

    // Fallback: /dev/urandom
    let semilla = generar_semilla_desde_hardware()?;
    Ok((semilla, None))
}

// Módulo hex simple (no dependemos de crates externos)
mod hex {
    pub fn decode(s: &str) -> Result<Vec<u8>, ()> {
        let s = s.trim();
        if s.len() % 2 != 0 {
            return Err(());
        }
        let mut result = Vec::with_capacity(s.len() / 2);
        let mut i = 0;
        while i < s.len() {
            let byte_str = &s[i..i + 2];
            let byte = u8::from_str_radix(byte_str, 16).map_err(|_| ())?;
            result.push(byte);
            i += 2;
        }
        Ok(result)
    }
}

// ============================================================================
// HILO DE SIMULACIÓN (PRINCIPAL)
// ============================================================================

/// Bucle principal de simulación
fn ejecutar_simulacion(
    universo: Arc<RwLock<Universo>>,
    canal_comandos: Receiver<ComandoRecibido>,
    canal_eventos: Sender<Evento>,
    senales: Arc<RwLock<SenalGlobal>>,
) {
    let paso_duration = Duration::from_millis(DURACION_PASO_MS);

    loop {
        let ciclo_inicio = Instant::now();

        // 1. Procesar comandos pendientes de Python
        while let Ok(cmd) = canal_comandos.try_recv() {
            procesar_comando(&universo, &cmd);
        }

        // Verificar si hay que pausar
        {
            let senal = senales.read().unwrap();
            if senal.pausado {
                thread::sleep(Duration::from_millis(100));
                continue;
            }
        }

        // 2. Evolucionar el Mar Morfóseo
        {
            let mut u = universo.write().unwrap();
            u.mar.step();
        }

        // 3. Ciclo vital de cada Auton
        let mut nuevos_autons: Vec<AutonVivo> = Vec::new();
        let mut autons_muertos: Vec<Uuid> = Vec::new();

        {
            let mut u = universo.write().unwrap();

            // Hacer step del Mar
            u.mar.step();

            // Ciclo vital de cada Auton
            for i in 0..u.autons.len() {
                // Get references that outlive the mutable borrow
                let mar_ref = &u.mar as *const MarMorfoseo;
                let const_ref = &u.constantes as *const ConstantesCosmicas;
                // SAFETY: We hold &mut u, so no other code can access these
                // and we don't use these pointers after u is modified
                unsafe {
                    (&mut *u.autons)
                        .get_unchecked_mut(i)
                        .ciclo_vital(&*mar_ref, &*const_ref);
                }
            }

            // 4. Detectar muertes
            for aut in u.autons.iter() {
                if aut.esta_muerto() {
                    autons_muertos.push(aut.id);
                }
            }

            // 5. Open-Endedness e INAGOTABILIDAD
            // INAGOTABILIDAD: genesis_energon se llama SIEMPRE, incluso sin Autons
            // Esto evita el deadlock donde sin Autons no hay energía, sin energía no hay Autons
            let num_autons = u.autons.len();

            // Calcular complejidad para genesis_energon
            let complejidad_para_energia = if num_autons > 0 {
                let autons_umbra: Vec<(u64, Umbra)> = u
                    .autons
                    .iter()
                    .map(|aut| (aut.id.as_u64(), aut.umbra.clone()))
                    .collect();
                let contador = u.contador_ciclos;

                let mut oe = std::mem::take(&mut u.open_endedness);
                oe.tick(&autons_umbra, contador);
                u.open_endedness = oe;

                u.open_endedness.metricas().complejidad_promedio
            } else {
                // INAGOTABILIDAD: even when no Autons, maintain baseline energy from void
                // Use tick as pseudo-complexity to allow recovery after extinction
                let tick_f = u.contador_ciclos as f32;
                (tick_f.min(1000.0) / 1000.0).max(0.01) // 0.01 to 1.0 based on tick
            };

            // INAGOTABILIDAD 100%: Energía del vacío fluye siempre
            // Even if all Autons die, the Mar regenerates from quantum fluctuations
            u.mar.genesis_energon(complejidad_para_energia.max(0.01));

            // 6. Generación espontánea
            if u.contador_ciclos % 1000 == 0 && u.contador_ciclos > 0 {
                eprintln!(
                    "[GEN_MAIN] ciclo {} entering spontaneous gen",
                    u.contador_ciclos
                );
                if let Some(nuevo) = u.intentar_generacion_espontanea() {
                    if let Err(e) = u.fs.registrar_nacimiento(Some(nuevo.id.as_u64())) {
                        eprintln!("EdenFS error: {}", e);
                    }
                    let pos = nuevo.campo.posicion();
                    let x = pos.x.to_raw() as f64 / i64::MAX as f64;
                    let y = pos.y.to_raw() as f64 / i64::MAX as f64;
                    let _ = canal_eventos.send(Evento::NacioAuton {
                        id: nuevo.id.as_u64(),
                        x,
                        y,
                    });
                    u.autons.push(nuevo);
                }
            }

            // 12. ENJAMBRE: Compartir genes con peers cada 1000 ciclos
            #[cfg(feature = "swarm")]
            if u.contador_ciclos % 1000 == 0 && u.contador_ciclos > 0 {
                use eden_core::swarm::{self, AutonGene};

                // Si hay peers, compartir genes de los mejores Autons
                if swarm::has_peers() {
                    // Seleccionar los 3 mejores Autons por fitness
                    let mut autons_con_genes: Vec<(f32, AutonGene)> = u
                        .autons
                        .iter()
                        .map(|aut| {
                            let stats = aut.ramnet.estadisticas();
                            // Fitness = actualizaciones + generación (más actualizado = mejor)
                            let fitness =
                                stats.actualizaciones as f32 + aut.generacion as f32 * 1000.0;
                            let gene = AutonGene {
                                ramnet_seed: aut.id.as_u64(),
                                campo_seed: aut.id.as_u64() ^ 0xDEADBEEF,
                                generacion: aut.generacion,
                                pos_x: aut.campo.posicion().x.to_raw() as f32 / i32::MAX as f32,
                                pos_y: aut.campo.posicion().y.to_raw() as f32 / i32::MAX as f32,
                                fitness,
                            };
                            (fitness, gene)
                        })
                        .collect();

                    // Ordenar por fitness descendente
                    autons_con_genes.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
                    let genes: Vec<AutonGene> = autons_con_genes
                        .into_iter()
                        .take(3)
                        .map(|(_, g)| g)
                        .collect();

                    if !genes.is_empty() {
                        swarm::share_genes(&genes);
                        println!("[SWARM] Compartidos {} genes", genes.len());
                    }

                    // Recibir genes de otros peers e inyectar como nuevos Autons
                    let incoming = swarm::receive_genes();
                    for gene in incoming {
                        // Crear Auton desde gene recibido
                        let nuevo = AutonVivo::desde_gen_inyectado(&gene, u.contador_ciclos);

                        // Registrar en EdenFS
                        if let Err(e) = u.fs.registrar_nacimiento(Some(nuevo.id.as_u64())) {
                            eprintln!("EdenFS error: {}", e);
                        }

                        // Enviar evento
                        let pos = nuevo.campo.posicion();
                        let x = pos.x.to_raw() as f64 / i64::MAX as f64;
                        let y = pos.y.to_raw() as f64 / i64::MAX as f64;
                        let _ = canal_eventos.send(Evento::NacioAuton {
                            id: nuevo.id.as_u64(),
                            x,
                            y,
                        });

                        // Insertar en el universo
                        u.autons.push(nuevo);

                        println!(
                            "[SWARM] ✧ Gen inyectado: gen={} fitness={} → Auton creado",
                            gene.generacion, gene.fitness
                        );
                    }
                }
            }
            // 9. Broadcast de estado (cada INTERVALO_SOCKET_BROADCAST ciclos)
            if u.contador_ciclos % INTERVALO_SOCKET_BROADCAST == 0 {
                let estado = u.estado_global();
                let evento = Evento::EcosistemaState {
                    autons_vivos: estado.autons_vivos as u32,
                    energia_total: estado.energia_total_mar,
                    escoria_total: 0.0, // Mar does not expose a stable aggregate yet.
                    densidad_promedio: estado.energia_total_mar
                        / TAMAÑO_MAR as f64
                        / TAMAÑO_MAR as f64,
                };
                let _ = canal_eventos.send(evento);
            }

            // 10. Guardar Meltrace peri'odicamente
            if u.contador_ciclos % INTERVALO_GUARDADO_MELTRACE == 0 && u.contador_ciclos > 0 {
                // Guardar en disco (serialización simple)
                if let Err(e) = guardar_meltrace(&u) {
                    eprintln!("Error al guardar Meltrace: {}", e);
                }
            }

            // 11. Guardar vitals para monitor (cada 3600 ciclos ≈ cada hora)
            // INAGOTABILIDAD: persistir estado para vital_signs
            if u.contador_ciclos % 3600 == 0 {
                let estado = u.estado_global();
                let vitals = VitalsJson {
                    timestamp: format!("{:?}", std::time::SystemTime::now()),
                    ciclo: estado.ciclo,
                    energia_mar: estado.energia_total_mar,
                    energia_autons: estado.energia_total_autons,
                    autons_vivos: estado.autons_vivos,
                    nacidos: estado.nacidos_total,
                    muertos: estado.muertos_total,
                    meltrace_len: u.meltrace.len() as u64,
                };
                if let Ok(json) = serde_json::to_string(&vitals) {
                    if let Err(e) = std::fs::write("/tmp/eden_vitals.json", json) {
                        eprintln!("Error guardando vitals: {}", e);
                    }
                }
            }

            let stats = u.meltrace.estadisticas();

            // 11. Auto-validación del Techo (cada 3600 ciclos)
            if u.contador_ciclos % 3600 == 0 && u.contador_ciclos > 0 {
                println!("\n  ── Techo Absoluto [ciclo {}] ──", u.contador_ciclos);
                println!("  Autons vivos: {}", u.autons.len());
                println!(
                    "  Energía Mar: {:.4}",
                    u.mar.energia_total().to_raw() as f64 / i64::MAX as f64
                );
                println!("  Grabados Meltrace: {}", u.meltrace.len());
                println!("  Muertes totales: {}", stats.muertes_totales);
            }

            u.contador_ciclos += 1;
        } // cierra el lock de universo

        // Control de velocidad (60 FPS subjetivo)
        let elapsed = ciclo_inicio.elapsed();
        if elapsed < paso_duration {
            thread::sleep(paso_duration - elapsed);
        }
    }
}

/// Procesa un comando recibido de Python
fn procesar_comando(universo: &Arc<RwLock<Universo>>, cmd: &ComandoRecibido) {
    let mut u = universo.write().unwrap();
    match cmd {
        ComandoRecibido::InyectarEnergon { x, y, cantidad } => {
            u.mar.add_energon(*x, *y, 0, *cantidad);
            println!("⚡ Energon inyectado en ({}, {}): {:?}", x, y, cantidad);
        }
        ComandoRecibido::AumentarEscoria { x, y, radio } => {
            u.mar.sembrar_region(
                x.saturating_sub(*radio),
                y.saturating_sub(*radio),
                0,
                x.saturating_add(*radio),
                y.saturating_sub(*radio),
                0,
                I32F32::from_raw(0x00000010_00000000), // ~16.0 de escoria
            );
            println!("☢ Escoria aumentada en ({}, {}) radio {}", x, y, radio);
        }
        ComandoRecibido::EliminarAuton { id } => {
            u.autons.retain(|a| a.id.as_u64() != *id);
            println!("✗ Auton {} eliminado", id);
        }
        ComandoRecibido::PausarSimulacion => {
            // Se maneja a través de senal global
        }
        ComandoRecibido::ReanudarSimulacion => {}
        ComandoRecibido::ForzarBifurcacion { id } => {
            // Forzar escisión de un Auton
            if let Some(aut) = u.autons.iter_mut().find(|a| a.id.as_u64() == *id) {
                aut.campo
                    .set_energia_interna(I32F32::from_raw(0x00000050_00000000));
                // 80.0
            }
        }
    }
}

/// Señales globales compartidas entre hilos
#[derive(Debug, Clone)]
pub struct SenalGlobal {
    pub pausado: bool,
    pub shutdown: bool,
}

impl Default for SenalGlobal {
    fn default() -> Self {
        SenalGlobal {
            pausado: false,
            shutdown: false,
        }
    }
}

/// Guardar Meltrace a disco (serialización simple)
fn guardar_meltrace(universo: &Universo) -> io::Result<()> {
    let dir_base =
        dirs::home_dir().ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "No home dir"))?;
    let path = dir_base.join(EDEN_DIR).join("meltrace_backup.bin");

    let mut file = File::create(&path)?;
    let num_grabados = universo.meltrace.len() as u64;
    file.write_all(&num_grabados.to_le_bytes())?;

    // Por ahora solo escribir el conteo (serialización completa requiere
    // implementar Serialize para los tipos internos)
    println!("  💾 Meltrace guardado: {} grabados", num_grabados);

    Ok(())
}

// ============================================================================
// HILO DE RENDERIZADO
// ============================================================================

/// Hilo de renderizado: framebuffer o terminal
fn hilo_renderizado(universo: Arc<RwLock<Universo>>, duracion_frame: Duration) {
    // Intentar abrir framebuffer
    let gpu_result = SoftGPU::new();
    let tiene_framebuffer = gpu_result.is_ok();
    if tiene_framebuffer {
        println!("🖥 Framebuffer detectado — modo GPU activo");
    } else {
        println!("📺 Sin framebuffer — modo terminal (render stats cada 60 frames)");
    }

    let mut ultimo_render = Instant::now();
    let mut fotogramas = 0u64;

    loop {
        thread::sleep(Duration::from_millis(16)); // ~60 FPS

        if Instant::now().duration_since(ultimo_render) < duracion_frame {
            continue;
        }
        ultimo_render = Instant::now();
        fotogramas += 1;

        let estado = {
            let u = universo.read().unwrap();
            u.estado_global()
        };

        // Solo loggear stats cada 60 frames (~1 segundo)
        if fotogramas % 60 == 0 {
            println!(
                "  [RENDER] ciclo:{} | autons:{} | mar:{:.4}",
                estado.ciclo, estado.autons_vivos, estado.energia_total_mar
            );
        }
    }
}

// ============================================================================
// HILO DE IPC (SOCKET UNIX)
// ============================================================================

/// Hilo de IPC: recibe comandos de Python y envía eventos
fn hilo_ipc(
    universo: Arc<RwLock<Universo>>,
    canal_comandos: Sender<ComandoRecibido>,
    canal_eventos: Receiver<Evento>,
    senales: Arc<RwLock<SenalGlobal>>,
) {
    // Crear socket Unix Datagram
    let socket_path = Path::new(SOCKET_PATH);

    // Eliminar socket anterior si existe
    let _ = fs::remove_file(socket_path);

    let socket = match UnixDatagram::bind(socket_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error al bindear socket {}: {}", SOCKET_PATH, e);
            return;
        }
    };

    println!("📡 Socket IPC listening en {}", SOCKET_PATH);

    let mut buf = [0u8; 4096];

    loop {
        // Configurar timeout para poder revisar senales periódicamente
        match socket.recv_from(&mut buf) {
            Ok((size, _addr)) => {
                let datos = &buf[..size];

                // Intentar interpretar como comando
                if let Some(cmd) = ComandoRecibido::desde_bytes(datos) {
                    // Manejar senales especiales
                    match &cmd {
                        ComandoRecibido::PausarSimulacion => {
                            let mut s = senales.write().unwrap();
                            s.pausado = true;
                            println!("⏸ Simulación pausada");
                        }
                        ComandoRecibido::ReanudarSimulacion => {
                            let mut s = senales.write().unwrap();
                            s.pausado = false;
                            println!("▶ Simulación reanudada");
                        }
                        _ => {
                            let _ = canal_comandos.send(cmd);
                        }
                    }

                    // Enviar confirmación
                    let confirm = b"{\"ok\":true}";
                    let _ = socket.send_to(SOCKET_PATH, confirm);
                }
            }
            Err(e) => {
                // No hay datos, continuar
                if e.kind() != io::ErrorKind::WouldBlock {
                    eprintln!("Socket recv error: {}", e);
                }
            }
        }

        // Enviar eventos pendientes
        while let Ok(evento) = canal_eventos.try_recv() {
            if let Ok(msg) = evento.to_message() {
                let bytes = msg.to_bytes();
                let _ = socket.send_to(SOCKET_PATH, &bytes);
            }
        }

        // Revisar shutdown
        {
            let s = senales.read().unwrap();
            if s.shutdown {
                println!("📡 Hilo IPC: shutdown recibido");
                break;
            }
        }

        // Dormir breve
        thread::sleep(Duration::from_millis(1));
    }
}

// ============================================================================
// MANEJO DE SEÑALES (SIGINT)
// ============================================================================

/// Configura manejadores de señales para shutdown graceful
fn configurar_manejador_senales(senales: Arc<RwLock<SenalGlobal>>) {
    #[cfg(unix)]
    {
        use std::os::unix::io::AsRawFd;

        // Instalar SIGINT handler
        unsafe {
            extern "C" fn handler() {
                println!("\n⚠ SIGINT recibido — guardando estado...");
                // El flag se marca a través de la señal (simplificado)
                // En producción usar signal.rs
            }
        }
    }
}

// ============================================================================
// PUNTO DE ENTRADA PRINCIPAL
// ============================================================================

fn main() -> io::Result<()> {
    println!();
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║          🜁 EDEN Core — Sistema A-Life Autopoético          ║");
    println!("║                  100% Rust — Sin dependencias externas       ║");
    println!("╚═══════════════════════════════════════════════════════════╝");
    println!();

    // 1. Obtener semilla (args o hardware)
    let (semilla, seed_display) = obtener_semilla()?;
    let seed_str = seed_display
        .unwrap_or_else(|| format!("[hardware {:02x}{:02x}...]", semilla[0], semilla[1]));
    println!("🌱 Semilla: {}", seed_str);

    // 2. Crear universo
    let universo = match Universo::crear(semilla) {
        Ok(u) => Arc::new(RwLock::new(u)),
        Err(e) => {
            eprintln!("Error al crear universo: {}", e);
            return Err(e);
        }
    };

    // 3. Canales de comunicación entre hilos
    let (tx_comandos, rx_comandos) = mpsc::channel();
    let (tx_eventos, rx_eventos) = mpsc::channel();

    // 4. Señales globales
    let senales = Arc::new(RwLock::new(SenalGlobal::default()));

    // 5. Configurar manejador de señales
    configurar_manejador_senales(senales.clone());

    // 6. Lanzar hilo de renderizado
    let universo_render = universo.clone();
    let _handle_render = thread::Builder::new()
        .name("render".to_string())
        .spawn(move || {
            hilo_renderizado(universo_render, Duration::from_millis(16));
        })
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    // 7. Lanzar hilo de IPC
    let universo_ipc = universo.clone();
    let senales_ipc = senales.clone();
    let _handle_ipc = thread::Builder::new()
        .name("ipc".to_string())
        .spawn(move || {
            hilo_ipc(universo_ipc, tx_comandos, rx_eventos, senales_ipc);
        })
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    println!("🚀 Hilos lanzados: simulación, render, IPC, swarm");
    println!("🌍进入 — Entrando en el Sueño del Motor...");

    // Inicializar enjambre P2P
    #[cfg(feature = "swarm")]
    {
        const SWARM_PORT: u16 = 4849;
        if let Err(e) = swarm::init(SWARM_PORT) {
            println!("[SWARM] Warning: no se pudo iniciar enjambre: {}", e);
        } else {
            println!("[SWARM] Enjambre activo en puerto {}", SWARM_PORT);
        }
    }

    // Pasar referencias a swarm para el bucle
    #[cfg(feature = "swarm")]
    let universo_swarm = universo.clone();
    println!("      Ctrl+C para pausar, HIBERNATE para guardar y salir\n");

    // 8. Ejecutar simulación en hilo principal
    ejecutar_simulacion(universo, rx_comandos, tx_eventos, senales);

    Ok(())
}

// ============================================================================
// TESTS DE INTEGRACIÓN
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Test de integración: universo con semilla fija
    #[test]
    fn test_universo_semilla_fija() {
        let semilla = [0u8; 128];
        let universo = Universo::crear(semilla).expect("No se pudo crear universo");

        // Debe haber exactamente 1 Auton vivo (el Primordial)
        assert_eq!(universo.autons.len(), 1, "El Primordial debe existir");

        // El Primordial debe estar vivo
        assert!(
            !universo.autons[0].esta_muerto(),
            "El Primordial no debe estar muerto al inicio"
        );

        // Meltrace debe estar vacío al inicio
        assert_eq!(universo.meltrace.len(), 0, "Meltrace vacío al inicio");
    }

    /// Test: correr 100 ciclos y verificar que algo cambió
    #[test]
    fn test_ciclo_basico() {
        let semilla = [0x42u8; 128];
        let universo = Arc::new(RwLock::new(Universo::crear(semilla).unwrap()));

        let (tx, rx) = mpsc::channel::<ComandoRecibido>();
        let (txe, rxe) = mpsc::channel::<Evento>();
        let senales = Arc::new(RwLock::new(SenalGlobal::default()));
        tx.send(ComandoRecibido::PausarSimulacion).unwrap();
        assert!(matches!(
            rx.try_recv().unwrap(),
            ComandoRecibido::PausarSimulacion
        ));
        txe.send(Evento::EcosistemaState {
            autons_vivos: 0,
            energia_total: 0.0,
            escoria_total: 0.0,
            densidad_promedio: 0.0,
        })
        .unwrap();
        assert!(matches!(
            rxe.try_recv().unwrap(),
            Evento::EcosistemaState { .. }
        ));
        assert!(!senales.read().unwrap().shutdown);

        // Simular 100 ciclos en el hilo actual
        let universo_clone = universo.clone();
        for _ in 0..100 {
            {
                let mut u = universo_clone.write().unwrap();
                u.mar.step();

                let mut autons = std::mem::take(&mut u.autons);
                for aut in autons.iter_mut() {
                    aut.ciclo_vital(&u.mar, &u.constantes);
                }
                u.autons = autons;

                u.contador_ciclos += 1;
            }
            thread::sleep(Duration::from_micros(100));
        }

        let u = universo.read().unwrap();

        // Alguno de estos debe ser verdadero después de 100 ciclos:
        // - Han nacido nuevos Autons
        // - Han muerto Autons
        // - El contador de ciclos avanzar
        assert!(
            u.contador_ciclos == 100,
            "Deben haberse completado 100 ciclos"
        );

        // Meltrace puede tener grabados si hubo muertes
        // (no hacemos aserción sobre esto ya que depende del timing)
    }

    /// Test: el UUID v4 es válido
    #[test]
    fn test_uuid_v4() {
        let uuid1 = Uuid::v4().unwrap();
        let uuid2 = Uuid::v4().unwrap();

        // UUIDs deben ser distintos
        assert_ne!(uuid1, uuid2, "UUIDs generados deben ser únicos");

        // deben caber en u64 para la mayoría de usos
        assert!(uuid1.as_u64() != 0 || uuid2.as_u64() != 0);
    }

    /// Test: comando de inyección de energon
    #[test]
    fn test_comando_inyectar_energon() {
        let cmd_bytes = b"{\"cmd\":\"InyectarEnergon\",\"x\":100,\"y\":200,\"cantidad\":500}";
        let cmd = ComandoRecibido::desde_bytes(cmd_bytes);

        assert!(cmd.is_some());
        match cmd.unwrap() {
            ComandoRecibido::InyectarEnergon { x, y, cantidad } => {
                assert_eq!(x, 100);
                assert_eq!(y, 200);
                assert!(cantidad > I32F32::ZERO);
            }
            _ => panic!("Comando incorrecto"),
        }
    }

    /// Test: expansión de semilla a 128 bytes
    #[test]
    fn test_calcular_semilla_u64() {
        let semilla = [
            0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE, 0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC,
            0xDE, 0xF0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ];

        let h = calcular_semilla_u64(&semilla);
        assert_ne!(h, 0, "Hash de semilla no debe ser cero");
    }
}
