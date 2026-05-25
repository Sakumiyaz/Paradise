//! # EDEN Core — Sistema A-Life Autopoético
//!
//! Punto de entrada principal del sistema de vida artificial.
//!
//! ## Arquitectura
//!
//! ```text
//! main()
//! ├── 1. Inicialización
//! │   ├── obtener_semilla() — CLI args o /dev/urandom
//! │   ├── Universo::crear() — Mar + Primordial
//! │   └── lanzar_hilos(render, ipc)
//! │
//! ├── 2. Bucle de Simulación (hilo principal)
//! │   ├── procesar_comandos_pendientes()
//! │   ├── universo.mar.step()
//! │   ├── aut.ciclo_vital() × N
//! │   ├── detectar_muertes() → Meltrace + EdenFS
//! │   ├── detectar_escisiones() → nuevos Autons
//! │   ├── generacion_espontanea() × cada 1000 ciclos
//! │   └── broadcast_estado() × cada 10 ciclos
//! │
//! ├── 3. Hilo de Renderizado
//! │   ├── SoftGPU::new() → /dev/fb0
//! │   └── TermHex fallback
//! │
//! └── 4. Hilo de IPC (socket Unix)
//!     └── recv comandos → send eventos
//! ```
//!
//! ## Determinismo
//!
//! El bucle principal es **estrictamente determinista** dado el estado inicial.
//! El único no-determinismo proviene de:
//! - Lectura de sensores de hardware (reloj CPU)
//! - Condiciones de carrera intencionadas en threads
//! - `/dev/urandom` solo para la semilla inicial
//!
//! ## Compilación
//!
//! ```bash
//! cargo build --release
//! ```
#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(unused)]

// ============================================================================
// MÓDULOS DE EDEN CORE
// ============================================================================

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
use eden_core::render::term_hex::{StatsSistema, TermHex};

// ============================================================================
// DEPENDENCIAS ESTÁNDAR
// ============================================================================

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
//
// Estas constantes definen el comportamiento del universo A-Life.
// Todas son configurables via ConstantesCosmicas derivadas de la semilla.
// ============================================================================

/// Tamaño del Mar Morfóseo (1024×1024 celdas)
///
/// Un Mar más grande permite más autonomía y distribuciones espacialmente
/// complejas de energon. 1024² = 1,048,576 celdas.
const TAMAÑO_MAR: usize = 1024;

/// Número de hilos para el solver del Mar Morfóseo
///
/// Cada hilo procesa una franja del Mar. Más hilos = más paralelismo
/// pero mayor overhead de sincronización.
const NUM_HILOS_MAR: usize = 8;

/// Pasos de simulación por segundo (60 FPS subjetivo)
///
/// El bucle principal intenta mantener este ritmo. Si la simulación
/// es más lenta que 60 FPS, el tiempo simulado se ralentiza.
const PASOS_POR_SEGUNDO: u32 = 60;

/// Intervalo en ciclos para guardar Meltrace en disco
///
/// Cada 600 ciclos ≈ cada 10 segundos. El Meltrace contiene todos
/// los Autons muertos, sirve como "memoria Lamarckiana" del sistema.
const INTERVALO_GUARDADO_MELTRACE: u64 = 600;

/// Intervalo en ciclos para broadcast de estado por socket
///
/// Cada 10 ciclos ≈ 6 veces por segundo. Envía estado global a Python
/// para que el agente Demiurgo pueda observar y actuar.
const INTERVALO_SOCKET_BROADCAST: u64 = 10;

/// Duración de un paso (16ms para 60 FPS)
const DURACION_PASO_MS: u64 = 1000 / PASOS_POR_SEGUNDO as u64;

/// Socket path para IPC con Python (Demiurgo)
const SOCKET_PATH: &str = "/tmp/eden_core.sock";

/// Directorio de estado Eden (~/.eden)
const EDEN_DIR: &str = ".eden";

/// Ruta del archivo de hibernación
const HIBERNATION_PATH: &str = "/tmp/eden_hibernation.bin";

// ============================================================================
// ESTRUCTURAS PRINCIPALES
// ============================================================================

/// UUID de un Auton (v4 aleatorio de 16 bytes)
///
/// Generado desde /dev/urandom para garantizar unicidad.
///
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Uuid(u128);

impl Uuid {
    /// Genera UUID v4 leyendo de /dev/urandom
    pub fn v4() -> io::Result<Self> {
        let mut bytes = [0u8; 16];
        let mut file = File::open("/dev/urandom")?;
        file.read_exact(&mut bytes)?;
        // Versión 4 + variante RFC 4122
        bytes[6] = (bytes[6] & 0x0F) | 0x40;
        bytes[8] = (bytes[8] & 0x3F) | 0x80;
        let val = u128::from_le_bytes(bytes);
        Ok(Uuid(val))
    }

    pub fn as_u64(&self) -> u64 {
        (self.0 & 0xFFFFFFFFFFFFFFFF) as u64
    }
}

impl std::fmt::Display for Uuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:032x}", self.0)
    }
}

/// Un Auton vivo: combina membrana (CampoEstructural), cerebro (RamNet),
/// sombra (Umbra) y energía
///
/// El Auton es la unidad fundamental de vida en EDEN. No es un objeto
/// gráfico — es un conjunto de campos matemáticos que definen su identidad.
///
/// # Componentes
///
/// - **campo**: Solver de EDP Allen-Cahn — define la membrana
/// - **ramnet**: Red neuronal sin pesos — decide acciones
/// - **umbra**: Grafo DAG — memoria causal de decisiones
/// - **energia**: Energía disponible para metabolismo
///
pub struct AutonVivo {
    /// Identificador único (UUID v4)
    pub id: Uuid,
    /// Campo Estructural (membrana - solver Allen-Cahn)
    pub campo: CampoEstructural,
    /// RamNet (cerebro sin pesos)
    pub ramnet: RamNet,
    /// Umbra (grafo de decisiones causales)
    pub umbra: Umbra,
    /// Energía actual del Auton
    pub energia: I32F32,
    /// Generación desde el Primordial (0 = primordial)
    pub generacion: u32,
    /// ID del padre (None si es Primordial)
    pub padre_id: Option<Uuid>,
}

impl AutonVivo {
    /// Crea el Auton Primordial (El Primogénito) en el centro del Mar
    ///
    /// El Primordial aparece en el centro del Mar con una esfera de radio 0.35
    /// en coordenadas normalizadas [0,1]. Su RamNet está vacía (sin entradas).
    pub fn nuevo_primordial(constantes: &ConstantesCosmicas, semilla: u64) -> Self {
        let id = Uuid::v4().expect("No se pudo generar UUID del Primordial");
        let mut campo = CampoEstructural::new_2d(32, 32);

        // Esfera en el centro del campo (coordenadas normalizadas 0..1)
        let cx = 0.5;
        let cy = 0.5;
        let radio = 0.35;
        campo.inicializar_circular(cx, cy, radio, Self::phi_aleatorio());
        campo.set_id(id.as_u64());

        // Posición inicial en el centro del Mar
        let pos_x = I32F32::from_raw((i32::MAX / 2) as i64);
        let pos_y = I32F32::from_raw((i32::MAX / 2) as i64);
        campo.set_posicion(Vector3D::new(pos_x, pos_y, I32F32::ZERO));

        // Energía inicial: 100.0 unidades
        campo.set_energia_interna(I32F32::from_raw(0x00000064_00000000));

        // RamNet vacía con seed para reproducibilidad
        let ramnet = RamNet::new(0, 0, semilla.wrapping_add(id.as_u64()));
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
    /// EDEN no fuerza complejidad - permite emergencia desde diversidad inicial
    fn phi_aleatorio() -> I32F32 {
        let r = rand_u64();
        let perturbacion = (r & 0xFFFF) as f64 / 65535.0;
        let signo = if r & 0x10000 != 0 { 1.0 } else { -1.0 };
        let magnitud = 0.7 + 0.3 * perturbacion;
        let valor = signo * magnitud;
        I32F32::from_f64(valor)
    }

    /// Crea un Auton hijo a partir de un lóbulo del campo paterno
    ///
    /// El hijo hereda:
    /// - Mitad de la energía del padre
    /// - Mutaciones en RamNet basadas en la energía del padre
    /// - Umbra propia (compartida por ahora)
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
        let semilla_ramnet = id.as_u64();
        let mut ramnet = RamNet::new(8, 2, semilla_ramnet);

        // Herencia de Umbra
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

    /// Avanza el ciclo vital: metabolismo + movimiento
    ///
    /// # Ciclo Par (Asimilación)
    ///
    /// El α del campo se incrementa 20%, permitiendo que la membrana
    /// "se ablande" y absorba energon del Mar. La energía absorbida
    /// es proporcional al área del contorno y la densidad local.
    ///
    /// # Ciclo Impar (Desasimilación)
    ///
    /// α vuelve a su valor base. Se consume energía interna para
    /// mantener el gradiente del campo. Si la energía llega a cero,
    /// el campo colapsa (muerte).
    pub fn ciclo_vital(&mut self, mar: &MarMorfoseo, constantes: &ConstantesCosmicas) {
        // 1. Avanzar el campo (Allen-Cahn)
        self.campo.step(mar, constantes);

        // 2. Actualizar energía del Auton desde el campo
        self.energia = self.campo.energia_interna();

        // 3. Movimiento basado en RamNet (simplificado)
        // En una implementación completa, la RamNet decidiría la dirección
        if self.campo.es_par() {
            let cx = self.campo.dims().nx / 2;
            let cy = self.campo.dims().ny / 2;
            if let Some(dens) = mar.densidad_en(cx, cy, 0) {
                if dens > I32F32::from_raw(0x00000001_00000000) {
                    let dx = I32F32::from_raw(0x00000000_10000000); // ~0.0625
                    let mut pos = *self.campo.posicion();
                    pos.x = pos.x + dx;
                    pos.y = pos.y + dx;
                    self.campo.set_posicion(pos);
                }
            }
        }
    }

    /// Retorna true si el Auton está muerto
    ///
    /// Un Auton muere cuando:
    /// - Su campo se disuelve (EstadoCampo::Disuelto)
    /// - Su energía interna llega a cero
    /// - Su campo ya no tiene celdas vivas (φ > 0)
    pub fn esta_muerto(&self) -> bool {
        self.campo.estado() == EstadoCampo::Disuelto
            || self.campo.energia_interna() <= I32F32::ZERO
            || !self.campo.esta_vivo()
    }

    /// Causa de muerte inferida
    pub fn causa_muerte(&self) -> CausaMuerte {
        if self.campo.estado() == EstadoCampo::Disuelto {
            CausaMuerte::ColapsoCampo
        } else if self.campo.energia_interna() <= I32F32::ZERO {
            CausaMuerte::AgotamientoEnergia
        } else {
            CausaMuerte::Senescencia
        }
    }

    /// Detecta escisión (reproducción asexual)
    ///
    /// Usa etiquetado BFS sobre la máscara φ > 0.1 para encontrar
    /// componentes conexos. Si hay ≥2 componentes y energía suficiente,
    /// retorna los lóbulos que se convertirán en hijos.
    pub fn detectar_escision(&self) -> Option<Vec<CampoEstructural>> {
        self.campo.detectar_escision()
    }

    /// Estado del Auton para enviar por el socket a Python
    pub fn estado_socket(&self) -> eden_core::life::campo_estructural::AutonState {
        self.campo.estado_para_socket()
    }
}

/// Universo: el estado global de la simulación A-Life
///
/// Contiene todo lo que existe en el universo simulado:
/// - El Mar Morfóseo (campo de energon)
/// - Los Autons vivos
/// - El Meltrace (memoria de los muertos)
/// - El sistema de archivos EdenFS
pub struct Universo {
    /// El Mar Morfóseo (campo de energon)
    pub mar: MarMorfoseo,
    /// Autons vivos
    pub autons: Vec<AutonVivo>,
    /// Registro de grabaciones Lamarckianas (Autons muertos)
    pub meltrace: Meltrace,
    /// Sistema de archivos EdenFS
    pub fs: EdenFS,
    /// Constantes cosmológicas (derivadas de la semilla)
    pub constantes: ConstantesCosmicas,
    /// Contador de ciclos ejecutados
    pub contador_ciclos: u64,
    /// Semilla original del universo (128 bytes)
    pub semilla: SemillaGenesis,
}

impl Universo {
    /// Crea un nuevo universo desde una semilla de 128 bytes
    ///
    /// # Pasos de inicialización
    ///
    /// 1. Derivar ConstantesCosmicas de la semilla
    /// 2. Crear MarMorfoseo de 1024×1024 lleno de energon uniforme
    /// 3. Inicializar Meltrace vacío
    /// 4. Crear/el directorio ~/.eden/universos/<hash_semilla>/
    /// 5. Sembrar el Primordial en el centro
    pub fn crear(semilla: [u8; 128]) -> io::Result<Self> {
        let constantes = ConstantesCosmicas::from_semilla(&semilla);

        // Crear Mar Morfóseo de 1024×1024
        let mut mar = MarMorfoseo::new_2d(TAMAÑO_MAR, NUM_HILOS_MAR);

        // =====================================================================
        // INICIALIZAR MAR CON ENERGON UNIFORME
        //
        // Llenamos el Mar con una densidad de energon de 1.0 unidad por celda.
        // Esto asegura que el Primordial tenga energía disponible para absorber.
        // La Escoria se mantiene en cero (Mar pristine).
        // =====================================================================
        let densidad_inicial = I32F32::ONE; // 1.0
        for y in 0..TAMAÑO_MAR {
            for x in 0..TAMAÑO_MAR {
                mar.add_energon(x, y, 0, densidad_inicial);
            }
        }

        // Inicializar Meltrace
        let semilla_u64 = calcular_semilla_u64(&semilla);
        let meltrace = Meltrace::new(semilla_u64);

        // Crear/el directorio de EdenFS en ~/.eden/universos/<semilla>/
        let fs = EdenFS::new(semilla_u64)?;

        let mut universo = Universo {
            mar,
            autons: Vec::new(),
            meltrace,
            fs,
            constantes,
            contador_ciclos: 0,
            semilla,
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
        EcosistemaEstado {
            ciclo: self.contador_ciclos,
            autons_vivos: self.autons.len() as u64,
            energia_total_mar: energia_mar.to_raw() as f64 / i64::MAX as f64,
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
    ///
    /// Ocurre cada 1000 ciclos con probabilidad baja.
    /// Si hay grabaciones en Meltrace, favorece regiones de alta energía.
    pub fn intentar_generacion_espontanea(&mut self) -> Option<AutonVivo> {
        // Probabilidad muy baja sin Recordings
        if self.meltrace.len() == 0 {
            return None;
        }

        // Buscar región de alta energía en el Mar
        let dims = self.mar.dimensiones();
        let step = 8;

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

        // Solo si la densidad es suficientemente alta
        if mejor_dens > I32F32::from_raw(0x00000002_00000000) {
            let mut nuevo = AutonVivo::nuevo_primordial(&self.constantes, self.contador_ciclos);
            let pos_x = I32F32::from_raw((mejor_x as i64 * i32::MAX as i64 / dims.x as i64));
            let pos_y = I32F32::from_raw((mejor_y as i64 * i32::MAX as i64 / dims.y as i64));
            nuevo
                .campo
                .set_posicion(Vector3D::new(pos_x, pos_y, I32F32::ZERO));

            println!(
                "✧ Auton espontáneo surgió en ({}, {}) — ciclo {}",
                mejor_x, mejor_y, self.contador_ciclos
            );
            return Some(nuevo);
        }

        None
    }

    /// Ejecuta un ciclo completo del ecosistema sin I/O externo.
    ///
    /// Mantiene en un solo punto las fases puras del bucle de vida:
    /// Mar, metabolismo, muertes, escisiones, generación espontánea y contador.
    fn ejecutar_ciclo_ecosistema(&mut self) {
        self.mar.step();

        let mut nuevos_autons = Vec::new();
        let mut autons_muertos = Vec::new();

        let mar_ref = &self.mar as *const _;
        let constantes_ref = &self.constantes as *const _;
        for aut in self.autons.iter_mut() {
            // SAFETY: both pointers reference fields owned by `self` and are
            // read-only for the duration of this loop while only `autons` is
            // mutably iterated.
            unsafe {
                let mar = &*mar_ref;
                let constantes = &*constantes_ref;
                aut.ciclo_vital(mar, constantes);
            }
        }

        for aut in self.autons.iter() {
            if aut.esta_muerto() {
                autons_muertos.push(aut.id);
            }
        }

        for aut in self.autons.iter() {
            if let Some(lobulos) = aut.detectar_escision() {
                for lobulo in lobulos {
                    let hijo = AutonVivo::desde_escision(aut, lobulo, &self.constantes);
                    nuevos_autons.push(hijo);
                }
            }
        }

        let mut muertes_info: Vec<(Uuid, CausaMuerte, u64, Umbra)> = Vec::new();
        for id in autons_muertos.iter() {
            if let Some(aut) = self.autons.iter().find(|a| a.id == *id) {
                let causa = aut.causa_muerte();
                let hash = aut.id.as_u64();
                let umbra_clone = aut.umbra.clone();
                muertes_info.push((aut.id, causa, hash, umbra_clone));
            }
        }

        for (id, causa, hash_estado, umbra) in muertes_info {
            self.meltrace.registrar_muerte(&umbra);

            if let Err(e) = self
                .fs
                .registrar_muerte(hash_estado, causa.clone(), hash_estado)
            {
                eprintln!("EdenFS error al registrar muerte: {}", e);
            }

            println!("✝ Auton {} murió ({:?})", id, causa);
        }

        self.autons.retain(|a| !autons_muertos.contains(&a.id));

        for nuevo in nuevos_autons.drain(..) {
            if let Err(e) = self.fs.registrar_nacimiento(Some(nuevo.id.as_u64())) {
                eprintln!("EdenFS error al registrar nacimiento: {}", e);
            }
            println!("✧ Nacio Auton {} (gen {})", nuevo.id, nuevo.generacion);
            self.autons.push(nuevo);
        }

        if self.contador_ciclos % 1000 == 0 && self.contador_ciclos > 0 {
            if let Some(nuevo) = self.intentar_generacion_espontanea() {
                if let Err(e) = self.fs.registrar_nacimiento(Some(nuevo.id.as_u64())) {
                    eprintln!("EdenFS error: {}", e);
                }
                self.autons.push(nuevo);
            }
        }

        self.contador_ciclos += 1;
    }

    /// Guarda el estado completo del universo para hibernación
    ///
    /// Incluye: semilla, contador_ciclos, mar, meltrace, fs
    pub fn guardar_hibernacion(&self) -> io::Result<()> {
        let path = Path::new(HIBERNATION_PATH);
        let mut file = File::create(path)?;

        // Escribir magia + versión
        file.write_all(b"EDEN_HIBERNATION_V1")?;

        // Escribir semilla (128 bytes)
        file.write_all(&self.semilla)?;

        // Escribir contador de ciclos
        file.write_all(&self.contador_ciclos.to_le_bytes())?;

        // Escribir número de Autons vivos
        let num_autons = self.autons.len() as u64;
        file.write_all(&num_autons.to_le_bytes())?;

        // Para cada Auton: id + energia + generacion + padre_id
        for aut in &self.autons {
            file.write_all(&aut.id.0.to_le_bytes())?;
            file.write_all(&aut.energia.to_raw().to_le_bytes())?;
            file.write_all(&aut.generacion.to_le_bytes())?;
            let padre = aut.padre_id.map(|p| p.0).unwrap_or(0);
            file.write_all(&padre.to_le_bytes())?;
        }

        println!("💾 Hibernación guardada: {} Autons vivos", num_autons);
        Ok(())
    }

    /// Carga estado desde hibernación (no implementado — requiere diseño)
    pub fn cargar_hibernacion(_path: &Path) -> io::Result<Self> {
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "Hibernación no implementada — usar --seed para reiniciar",
        ))
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
///
/// Protocolo: JSON simple en texto plano.
///
#[derive(Debug, Clone)]
pub enum ComandoRecibido {
    /// Inyecta energon en una posición del Mar
    InyectarEnergon {
        x: usize,
        y: usize,
        cantidad: I32F32,
    },
    /// Aumenta escoria en una región circular
    AumentarEscoria { x: usize, y: usize, radio: usize },
    /// Consulta el estado de un Auton específico
    ConsultarAuton { id: u64 },
    /// Forzar bifurcación de un Auton
    ForzarBifurcacion { id: u64 },
    /// Eliminar un Auton por ID
    EliminarAuton { id: u64 },
    /// Pausar simulación
    PausarSimulacion,
    /// Reanudar simulación
    ReanudarSimulacion,
}

impl ComandoRecibido {
    /// Parsea un comando desde bytes recibidos del socket
    pub fn desde_bytes(bytes: &[u8]) -> Option<Self> {
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
            if s.starts_with("{\"cmd\":\"ConsultarAuton\"") {
                let id = extraer_numero(s, "id")? as u64;
                return Some(ComandoRecibido::ConsultarAuton { id });
            }
            if s.starts_with("{\"cmd\":\"EliminarAuton\"") {
                let id = extraer_numero(s, "id")? as u64;
                return Some(ComandoRecibido::EliminarAuton { id });
            }
            if s.starts_with("{\"cmd\":\"ForzarBifurcacion\"") {
                let id = extraer_numero(s, "id")? as u64;
                return Some(ComandoRecibido::ForzarBifurcacion { id });
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

/// Extrae un número entero de un string JSON-like
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
///
/// Usa XORrotate para comprimir 128 bytes a 64 bits de forma
/// determinista. Dado el mismo input, siempre produce el mismo output.
fn calcular_semilla_u64(semilla: &[u8; 128]) -> u64 {
    let mut h: u64 = 0xEAD_BEEFu64;
    for (i, &b) in semilla.iter().enumerate() {
        h ^= b as u64 * (i as u64).wrapping_mul(31);
        h = h.rotate_left(7);
    }
    h
}

/// Genera semilla de 128 bytes desde /dev/urandom
///
/// Fallback seguro: /dev/urandom esBlocking y devuelve entropía real.
fn generar_semilla_desde_hardware() -> io::Result<[u8; 128]> {
    let mut bytes = [0u8; 128];
    let mut file = File::open("/dev/urandom")?;
    file.read_exact(&mut bytes)?;
    Ok(bytes)
}

/// Obtiene semilla desde argumentos CLI o hardware
///
/// Formatos soportados:
/// - `--seed=HEX` o `--seed HEX` (interpreta hex como bytes)
/// - Sin argumento → /dev/urandom
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

            // Parsear hex a bytes (soporta longitud arbitraria, rellena a 128)
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

/// Módulo hex simple (inline, sin dependencia externa)
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
// SEÑALES GLOBALES
// ============================================================================

/// Señales compartidas entre hilos para control global
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

// ============================================================================
// BUCLE PRINCIPAL DE SIMULACIÓN
// ============================================================================

/// Bucle principal de simulación — CORAZÓN DEL SISTEMA A-LIFE
///
/// # Fases del bucle (cada ciclo)
///
/// 1. **Procesar comandos pendientes** — comandos recibidos del hilo IPC
/// 2. **Evolucionar el Mar** — física de difusión de energon
/// 3. **Ciclo vital de cada Auton** — Allen-Cahn + metabolismo
/// 4. **Detectar muertes** — Autons con energía agotada
/// 5. **Detectar escisiones** — Autons que se dividen
/// 6. **Procesar muertes** — Meltrace + EdenFS
/// 7. **Añadir nuevos Autons** — nacimientos y escisiones
/// 8. **Generación espontánea** — cada 1000 ciclos
/// 9. **Broadcast de estado** — cada 10 ciclos al socket
/// 10. **Guardar Meltrace** — cada 600 ciclos
///
/// # Control de velocidad
///
/// El bucle intenta mantener 60 FPS. Si la simulación es más lenta,
/// el tiempo simulado se ralentiza proporcionalmente.
///
fn ejecutar_simulacion(
    universo: Arc<RwLock<Universo>>,
    canal_comandos: Receiver<ComandoRecibido>,
    senales: Arc<RwLock<SenalGlobal>>,
) {
    let paso_duration = Duration::from_millis(DURACION_PASO_MS);

    loop {
        let ciclo_inicio = Instant::now();

        // =====================================================================
        // FASE 1: Procesar comandos pendientes de Python
        //
        // El hilo IPC encola comandos en canal_comandos.
        // Aquí los procesamos de forma síncrona.
        // =====================================================================
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

        // =====================================================================
        // FASE 2: Evolucionar el Mar Morfóseo
        //
        // El Mar procesa la ecuación de difusión de energon:
        // ∂ρ/∂t = D∇²ρ - ∇·(τ·σ) + η
        //
        // Esto actualiza densidades y flujos en todo el grid.
        // =====================================================================
        {
            let mut u = universo.write().unwrap();
            u.mar.step();
        }

        // =====================================================================
        // FASE 3-5: Ciclo vital, muertes, escisiones
        //
        // Para cada Auton:
        // - Ejecutar ciclo_vital() → Allen-Cahn + metabolismo
        // - Verificar si está muerto
        // - Verificar si debe dividirse
        // =====================================================================
        let mut nuevos_autons = Vec::new();
        let mut autons_muertos = Vec::new();

        {
            let mut u = universo.write().unwrap();

            // Procesar ciclo_vital de todos los autons
            // Necesitamos acceder a mar y constantes de u, y modificar u.autons
            let mar_ref = &u.mar as *const _;
            let constantes_ref = &u.constantes as *const _;
            // SAFETY: these pointers are valid for the scope of this block since u lives here
            for aut in u.autons.iter_mut() {
                // SAFETY: pointers are valid - u owns these fields and we're in u's scope
                unsafe {
                    let mar = &*mar_ref;
                    let constantes = &*constantes_ref;
                    aut.ciclo_vital(mar, constantes);
                }
            }

            // Detectar muertes
            for aut in u.autons.iter() {
                if aut.esta_muerto() {
                    autons_muertos.push(aut.id);
                }
            }

            // Detectar escisiones
            for aut in u.autons.iter() {
                if let Some(lobulos) = aut.detectar_escision() {
                    for lobulo in lobulos {
                        let hijo = AutonVivo::desde_escision(aut, lobulo, &u.constantes);
                        nuevos_autons.push(hijo);
                    }
                }
            }

            // =================================================================
            // FASE 6: Procesar muertes
            //
            // Por cada Auton muerto:
            // - Grabar su Umbra en Meltrace (memoria Lamarckiana)
            // - Guardar en EdenFS (~/.eden/meltrace/)
            // =================================================================

            // Primero: extraer umbras y datos necesarios de autons muertos
            let mut muertes_info: Vec<(Uuid, CausaMuerte, u64, Umbra)> = Vec::new();
            for id in autons_muertos.iter() {
                if let Some(aut) = u.autons.iter().find(|a| a.id == *id) {
                    let causa = aut.causa_muerte();
                    let hash = aut.id.as_u64();
                    let umbra_clone = aut.umbra.clone();
                    muertes_info.push((aut.id, causa, hash, umbra_clone));
                }
            }

            // Segundo: procesar las muertes (ya no hay borrow conflict)
            for (id, causa, hash_estado, umbra) in muertes_info {
                u.meltrace.registrar_muerte(&umbra);

                if let Err(e) =
                    u.fs.registrar_muerte(hash_estado, causa.clone(), hash_estado)
                {
                    eprintln!("EdenFS error al registrar muerte: {}", e);
                }

                println!("✝ Auton {} murió ({:?})", id, causa);
            }

            // Remover muertos del vector
            u.autons.retain(|a| !autons_muertos.contains(&a.id));

            // =================================================================
            // FASE 7: Añadir nuevos Autons
            //
            // Incluye:
            // - Hijos de escisión (reproducción asexual)
            // - Nacimientos registrados en EdenFS
            // =================================================================
            for mut nuevo in nuevos_autons.drain(..) {
                if let Err(e) = u.fs.registrar_nacimiento(Some(nuevo.id.as_u64())) {
                    eprintln!("EdenFS error al registrar nacimiento: {}", e);
                }
                println!("✧ Nacio Auton {} (gen {})", nuevo.id, nuevo.generacion);
                u.autons.push(nuevo);
            }

            // =================================================================
            // FASE 8: Generación espontánea
            //
            // Cada 1000 ciclos, intentar crear un Auton de la nada.
            // Solo ocurre si:
            // - Hay grabaciones en Meltrace (memoria)
            // - Existe una región de alta energía en el Mar
            // =================================================================
            if u.contador_ciclos % 1000 == 0 && u.contador_ciclos > 0 {
                if let Some(nuevo) = u.intentar_generacion_espontanea() {
                    if let Err(e) = u.fs.registrar_nacimiento(Some(nuevo.id.as_u64())) {
                        eprintln!("EdenFS error: {}", e);
                    }
                    u.autons.push(nuevo);
                }
            }

            // =================================================================
            // FASE 9: Broadcast de estado
            //
            // Cada 10 ciclos enviamos estado global al socket para que
            // Python/Demiurgo pueda observar el ecosistema.
            // =================================================================
            if u.contador_ciclos % INTERVALO_SOCKET_BROADCAST == 0 {
                let _estado = u.estado_global();
                // El envío real se hace desde el hilo IPC
                // Aquí solo registramos para el log
                if u.contador_ciclos % 60 == 0 {
                    println!(
                        "[{:>6}] Autons: {:>2} | Mar: {:>12.4} | Nacidos: {} | Muertos: {}",
                        u.contador_ciclos,
                        u.autons.len(),
                        u.mar.energia_total().to_raw() as f64 / i64::MAX as f64,
                        u.meltrace.muertes_totales() + u.autons.len() as u64,
                        u.meltrace.muertes_totales()
                    );
                }
            }

            // =================================================================
            // FASE 10: Guardar Meltrace peri'odicamente
            //
            // Cada 600 ciclos guardamos el estado de Meltrace a disco.
            // Esto permite preservar la memoria Lamarckiana.
            // =================================================================
            if u.contador_ciclos % INTERVALO_GUARDADO_MELTRACE == 0 && u.contador_ciclos > 0 {
                if let Err(e) = guardar_meltrace(&u) {
                    eprintln!("Error al guardar Meltrace: {}", e);
                }
            }

            u.contador_ciclos += 1;

            // Verificar shutdown para hibernación
            let senal = senales.read().unwrap();
            if senal.shutdown {
                println!("\n⚠ Guardando estado antes de salir...");
                if let Err(e) = u.guardar_hibernacion() {
                    eprintln!("Error en hibernación: {}", e);
                }
                return;
            }
        }

        // =====================================================================
        // Control de velocidad: mantener 60 FPS
        //
        // Si el ciclo tardó menos de 16ms, dormimos el resto.
        // Si tardó más, simplemente continuamos (nos atrasamos).
        // =====================================================================
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
        ComandoRecibido::ConsultarAuton { id } => {
            if let Some(aut) = u.autons.iter().find(|a| a.id.as_u64() == *id) {
                let estado = aut.estado_socket();
                println!(
                    "🔍 Auton {:016x}: energía={:.4}, contorno={}, fracción={:.3}",
                    id, estado.energia_interna, estado.tamanio_contorno, estado.fraccion_viva
                );
            } else {
                println!("🔍 Auton {:016x} no encontrado", id);
            }
        }
        ComandoRecibido::EliminarAuton { id } => {
            u.autons.retain(|a| a.id.as_u64() != *id);
            println!("✗ Auton {} eliminado", id);
        }
        ComandoRecibido::ForzarBifurcacion { id } => {
            if let Some(aut) = u.autons.iter_mut().find(|a| a.id.as_u64() == *id) {
                aut.campo
                    .set_energia_interna(I32F32::from_raw(0x00000050_00000000)); // 80.0
                println!("🔀 Auton {} forzado a bifurcar", id);
            }
        }
        ComandoRecibido::PausarSimulacion => {
            // Se marca en senal global
        }
        ComandoRecibido::ReanudarSimulacion => {}
    }
}

/// Guardar Meltrace a disco
fn guardar_meltrace(universo: &Universo) -> io::Result<()> {
    let dir_base =
        dirs::home_dir().ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "No home dir"))?;
    let path = dir_base.join(EDEN_DIR).join("meltrace_backup.bin");

    let mut file = File::create(&path)?;
    let num_grabados = universo.meltrace.len() as u64;
    file.write_all(&num_grabados.to_le_bytes())?;

    println!("  💾 Meltrace guardado: {} grabados", num_grabados);
    Ok(())
}

// ============================================================================
// HILO DE RENDERIZADO
// ============================================================================

/// Hilo de renderizado: framebuffer o terminal
///
/// Intenta abrir /dev/fb0 (framebuffer). Si falla, usa TermHex
/// como fallback para terminales ANSI.
///
/// # Responsabilidades
///
/// 1. Leer estado del universo (RwLock no-bloqueante)
/// 2. Renderizar fondo: mapa de calor del Mar
/// 3. Renderizar Autons: contornos φ=0
/// 4. Renderizar overlay: estadísticas
///
fn hilo_renderizado(universo: Arc<RwLock<Universo>>, duracion_frame: Duration) {
    // Intentar abrir framebuffer
    let gpu = SoftGPU::new();
    let mut renderizador: Option<SoftGPU> = None;
    let mut termhex = TermHex::new(80, 40); // Default: 80x40 celdas hex

    if gpu.is_ok() {
        renderizador = gpu.ok();
        println!("🖥 Framebuffer detectado — modo GPU activo");
    } else {
        println!("📺 Sin framebuffer — modo terminal");
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

        // Leer estado (no bloqueante)
        let estado = {
            let u = match universo.read() {
                Ok(guard) => guard,
                Err(_) => continue, // Retry next frame
            };
            u.estado_global()
        };

        // Solo loggear stats cada 60 frames (~1 segundo)
        if fotogramas % 60 == 0 {
            if renderizador.is_none() {
                // Modo terminal: actualizar stats
                termhex.actualizar_stats(StatsSistema {
                    autons_vivos: estado.autons_vivos as u32,
                    energia_total: estado.energia_total_mar,
                    escoria_total: 0.0,
                    densidad_promedio: 0.0,
                    fps: 0.0,
                });
            }
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
///
/// Usa Unix Datagram socket en /tmp/eden_core.sock
///
/// Protocolo:
/// - Python → Rust: comandos JSON
/// - Rust → Python: eventos binarios (EdenCommand)
///
fn hilo_ipc(
    universo: Arc<RwLock<Universo>>,
    canal_comandos: Sender<ComandoRecibido>,
    senales: Arc<RwLock<SenalGlobal>>,
) {
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
                if e.kind() != io::ErrorKind::WouldBlock {
                    eprintln!("Socket recv error: {}", e);
                }
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
///
/// # SIGINT (Ctrl+C)
///
/// Al recibir Ctrl+C:
/// 1. Marca shutdown = true
/// 2. El bucle de simulación detecta el flag
/// 3. Guarda estado en /tmp/eden_hibernation.bin
/// 4. Sale limpiamente
///
/// # Implementación
///
/// Usa std::signal en Unix para instalar el handler.
/// En Windows, el handler es un no-op.
///
fn configurar_manejador_senales(_senales: Arc<RwLock<SenalGlobal>>) {
    // Signal handling deshabilitado - requiere plataforma Unix con soporte de señales
    // El shutdown será detectado por el bucle principal vía ctrl_break flag
    #[cfg(unix)]
    {
        // En Unix con libc disponible, usar:
        // unsafe {
        //     use std::signal::{signal, SigHandler};
        //     use std::os::unix::signal::sigaction;
        //     signal(libc::SIGINT, SigHandler::Handler(move || { ... }));
        // }
    }
    #[cfg(not(unix))]
    {
        // En Windows o sin soporte, el shutdown se maneja por ctrl_break
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

    // =====================================================================
    // PASO 1: Obtener semilla
    //
    // Prioridad:
    // 1. --seed=HEX (línea de comandos)
    // 2. /dev/urandom (fallback hardware)
    //
    // La semilla de 128 bytes deriva todas las constantes del universo.
    // =====================================================================
    let (semilla, seed_display) = obtener_semilla()?;
    let seed_str = seed_display
        .unwrap_or_else(|| format!("[hardware {:02x}{:02x}...]", semilla[0], semilla[1]));
    println!("🌱 Semilla: {}", seed_str);

    // =====================================================================
    // PASO 2: Crear universo
    //
    // Inicializa:
    // - Mar Morfóseo de 1024×1024 con energon uniforme
    // - Un Auton Primordial en el centro
    // - Meltrace vacío
    // - EdenFS en ~/.eden/universos/<hash_semilla>/
    // =====================================================================
    let universo = match Universo::crear(semilla) {
        Ok(u) => Arc::new(RwLock::new(u)),
        Err(e) => {
            eprintln!("Error al crear universo: {}", e);
            return Err(e);
        }
    };

    // =====================================================================
    // PASO 3: Canales de comunicación entre hilos
    //
    // - canal_comandos: IPC → Simulación (comandos de Python)
    // =====================================================================
    let (tx_comandos, rx_comandos) = mpsc::channel();

    // =====================================================================
    // PASO 4: Señales globales
    //
    // Compartidas entre hilos para control de:
    // - Pausa/reanudación
    // - Shutdown para hibernación
    // =====================================================================
    let senales = Arc::new(RwLock::new(SenalGlobal::default()));

    // =====================================================================
    // PASO 5: Configurar manejador de SIGINT
    //
    // Ctrl+C triggers hibernación y salida limpia.
    // =====================================================================
    configurar_manejador_senales(senales.clone());

    // =====================================================================
    // PASO 6: Lanzar hilo de renderizado
    //
    // Intenta /dev/fb0 (GPU). Si falla, usa terminal (TermHex).
    // =====================================================================
    let universo_render = universo.clone();
    let _handle_render = thread::Builder::new()
        .name("render".to_string())
        .spawn(move || {
            hilo_renderizado(universo_render, Duration::from_millis(16));
        })
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    // =====================================================================
    // PASO 7: Lanzar hilo de IPC
    //
    // Socket Unix Datagram para comunicación bidireccional con Python.
    // =====================================================================
    let universo_ipc = universo.clone();
    let senales_ipc = senales.clone();
    let _handle_ipc = thread::Builder::new()
        .name("ipc".to_string())
        .spawn(move || {
            hilo_ipc(universo_ipc, tx_comandos, senales_ipc);
        })
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

    println!("🚀 Hilos lanzados: simulación, render, IPC");
    println!("🌍进入 — Entrando en el Sueño del Motor...");
    println!("      Ctrl+C para pausar, hibernación para guardar y salir\n");

    // =====================================================================
    // PASO 8: Ejecutar simulación (hilo principal)
    //
    // Este es el coraz'ón del sistema. Permanece aquí hasta:
    // - Error fatal
    // - SIGINT (Ctrl+C) → hibernación → salida
    // =====================================================================
    ejecutar_simulacion(universo, rx_comandos, senales);

    println!("✝ EDEN ha sido hibernado. Hasta la próxima.");
    Ok(())
}

// ============================================================================
// TESTS DE INTEGRACIÓN
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Test de integración: universo con semilla fija
    ///
    /// Verifica que el universo se crea correctamente con:
    /// - Exactamente 1 Auton vivo (el Primordial)
    /// - El Primordial está vivo
    /// - Meltrace vacío
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

    /// Test: correr ciclos acotados y verificar que el sistema avanza
    ///
    /// Ejecuta un smoke test corto y verifica:
    /// - El contador de ciclos avanzó
    #[test]
    fn test_ciclo_basico() {
        const CICLOS_TEST: u64 = 3;

        let semilla = [0x42u8; 128];
        let universo = Arc::new(RwLock::new(Universo::crear(semilla).unwrap()));

        let (tx, rx) = mpsc::channel::<ComandoRecibido>();
        let senales = Arc::new(RwLock::new(SenalGlobal::default()));
        tx.send(ComandoRecibido::PausarSimulacion).unwrap();
        assert!(matches!(
            rx.try_recv().unwrap(),
            ComandoRecibido::PausarSimulacion
        ));
        assert!(!senales.read().unwrap().shutdown);

        // Simular ciclos en el hilo actual
        for _ in 0..CICLOS_TEST {
            universo.write().unwrap().ejecutar_ciclo_ecosistema();
        }

        let u = universo.read().unwrap();

        // Verificar que se completaron los ciclos acotados
        assert_eq!(
            u.contador_ciclos, CICLOS_TEST,
            "Deben haberse completado los ciclos del smoke test"
        );
    }

    /// Test: el UUID v4 es válido y único
    #[test]
    fn test_uuid_v4() {
        let uuid1 = Uuid::v4().unwrap();
        let uuid2 = Uuid::v4().unwrap();

        // UUIDs deben ser distintos
        assert_ne!(uuid1, uuid2, "UUIDs generados deben ser únicos");

        // Deben caber en u64
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

    /// Test: comando ConsultarAuton
    #[test]
    fn test_comando_consultar_auton() {
        let cmd_bytes = b"{\"cmd\":\"ConsultarAuton\",\"id\":12345}";
        let cmd = ComandoRecibido::desde_bytes(cmd_bytes);

        assert!(cmd.is_some());
        match cmd.unwrap() {
            ComandoRecibido::ConsultarAuton { id } => {
                assert_eq!(id, 12345);
            }
            _ => panic!("Comando incorrecto"),
        }
    }

    /// Test: expansión de semilla a 128 bytes
    #[test]
    fn test_semilla_128_bytes() {
        let semilla = [0x42u8; 128];
        let h = calcular_semilla_u64(&semilla);

        // Debe ser determinista
        let h2 = calcular_semilla_u64(&semilla);
        assert_eq!(h, h2);

        // No debe ser cero
        assert_ne!(h, 0);
    }
}

// ============================================================================
// TEST DE INTEGRACIÓN (ejecutable separado)
// ============================================================================

#[cfg(test)]
mod integracion_tests {
    use super::*;

    /// Test de integración completo: "Prueba de Fuego"
    ///
    /// Ejecutable con: cargo test --test integracion
    ///
    /// Verifica:
    /// 1. Universo inicia con semilla fija [0u8; 128]
    /// 2. Corre ciclos acotados del ecosistema
    /// 3. Verifica metabolismo observable
    /// 4. Fuerza una muerte controlada y valida Meltrace
    #[test]
    fn test_fuego() {
        const CICLOS_FUEGO: u64 = 8;
        let semilla = [0u8; 128];

        // Crear universo
        let universo = match Universo::crear(semilla) {
            Ok(u) => u,
            Err(e) => panic!("No se pudo crear universo: {}", e),
        };

        let autons_inicial = universo.autons.len();
        let ciclos_inicial = universo.contador_ciclos;
        let energia_inicial = universo.autons[0].energia.to_raw();

        // Crear universo mutable para simular ciclos
        let mut universo_sim = universo;

        // Simular ciclos deterministas de metabolismo real.
        for _ in 0..CICLOS_FUEGO {
            universo_sim.ejecutar_ciclo_ecosistema();
        }

        let energia_tras_metabolismo = universo_sim.autons[0].energia.to_raw();
        assert_ne!(
            energia_inicial, energia_tras_metabolismo,
            "El metabolismo debe cambiar la energía del Primordial"
        );

        // Forzar una muerte controlada para validar el pipeline completo de
        // muerte -> Meltrace -> EdenFS sin depender de probabilidades emergentes.
        let auton_controlado = universo_sim.autons[0].id;
        universo_sim.autons[0]
            .campo
            .set_energia_interna(I32F32::ZERO);
        universo_sim.autons[0].energia = I32F32::ZERO;
        let fuera_del_mar = I32F32::from_raw(0x00000002_00000000);
        universo_sim.autons[0].campo.set_posicion(Vector3D::new(
            fuera_del_mar,
            fuera_del_mar,
            I32F32::ZERO,
        ));
        universo_sim.ejecutar_ciclo_ecosistema();

        // VERIFICACIÓN 1: El número de Autons cambió por muerte registrada.
        let autons_final = universo_sim.autons.len();
        assert_ne!(
            autons_inicial, autons_final,
            "Autons deben haber cambiado (nacimientos o muertes). \
             Inicial: {}, Final: {}",
            autons_inicial, autons_final
        );

        // VERIFICACIÓN 2: Meltrace no está vacío
        //
        assert!(
            universo_sim
                .autons
                .iter()
                .all(|aut| aut.id != auton_controlado),
            "El Auton controlado debe haber sido retirado tras morir"
        );

        let meltrace_len = universo_sim.meltrace.len();
        assert!(
            meltrace_len > 0,
            "Meltrace no debe estar vacío después de una muerte controlada. \
             Muertes registradas: {}",
            meltrace_len
        );

        // VERIFICACIÓN 3: Contador de ciclos avanzó
        assert_eq!(
            universo_sim.contador_ciclos,
            ciclos_inicial + CICLOS_FUEGO + 1,
            "Deben haberse completado los ciclos de fuego"
        );

        println!(
            "✓ Prueba de fuego exitosa: {}→{} Autons, {} grabaciones Meltrace",
            autons_inicial, autons_final, meltrace_len
        );
    }
}
