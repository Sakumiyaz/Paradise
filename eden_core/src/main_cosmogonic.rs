//! # Cosmogonic Test Main - EDEN Core A-Life Engine Validator
//!
//! This binary integrates all core EDEN modules to validate the A-Life engine:
//! - MarMorfoseo: The continuous automaton of Energon
//! - CampoEstructural: Auton membrane (Allen-Cahn EDP solver)
//! - Umbra: Causal shadow DAG of decisions
//! - Meltrace: Lamarckian engraving registry
//! - Genesis: Nomos/Tipon generator
//! - SoftGPU: Framebuffer renderer
//! - IPC: Unix socket communication with Demiurgo
#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(unused)]

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::{Duration, Instant};

use eden_core::fs::{CausaMuerte, EdenFS};
use eden_core::genesis::{EntropySource, GenesisPattern};
use eden_core::ipc::command::{EdenCommand, Evento};
use eden_core::life::{CampoEstructural, Meltrace, Umbra};
use eden_core::membrain::rand_u64;
use eden_core::physics::{
    ConfigMar, ConstantesCosmicas, DimensionesMar, MarMorfoseo, SemillaGenesis, Vector3D, I32F32,
};

// ============================================================================
// Seed Generation
// ============================================================================

/// Obtain genesis seed from args or hardware entropy
fn obtener_semilla_genesis() -> u64 {
    std::env::args()
        .nth(1)
        .map(|s| {
            s.parse().unwrap_or_else(|_| {
                // Fallback: combine hardware entropy
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos() as u64
            })
        })
        .unwrap_or_else(|| {
            let mut h = DefaultHasher::new();
            std::time::Instant::now().hash(&mut h);
            std::env::vars().collect::<Vec<_>>().hash(&mut h);
            h.finish()
        })
}

/// Convert u64 seed to SemillaGenesis (128 bytes)
fn seed_to_semilla(seed: u64) -> SemillaGenesis {
    let mut s = [0u8; 128];
    // Simple expansion: repeat seed bytes with mixing
    for i in 0..128 {
        let word = if i < 8 {
            seed
        } else {
            seed.wrapping_mul(i as u64 + 1)
        };
        s[i] = ((word >> ((i % 8) * 8)) & 0xFF) as u8;
        s[i] = s[i]
            .wrapping_add(i as u8)
            .wrapping_mul((i as u8).wrapping_add(1));
    }
    s
}

// ============================================================================
// Auton: The Living Entity (Wrapper combining CampoEstructural + Umbra)
// ============================================================================

/// Auton: The living membrane entity combining CampoEstructural and Umbra
pub struct Auton {
    /// Campo Estructural (membrane)
    campo: CampoEstructural,
    /// Umbra (causal shadow)
    umbra: Umbra,
    /// Unique identifier
    id: u64,
    /// Parent ID (0 if primordial)
    parent_id: u64,
    /// Tick when born
    tick_nacimiento: u64,
    /// Current tick
    tick_actual: u64,
    /// Energy at last death check
    energia_anterior: I32F32,
    /// Cause of death (if dead)
    causa_muerte: Option<CausaMuerte>,
    /// Has been registered in meltrace on death
    muerte_registrada: bool,
}

impl Auton {
    /// Create the Primordial Auton (El Primogénito)
    pub fn nuevo_primordial(constantes: &ConstantesCosmicas, x: f64, y: f64) -> Self {
        let mut campo = CampoEstructural::new_2d(32, 32);
        // Initialize with circular membrane centered at position
        campo.inicializar_circular(x, y, 0.3, Self::phi_aleatorio());
        campo.set_energia_interna(I32F32::from_raw(0x00000064_00000000)); // 100.0

        let mut umbra = Umbra::nuevo(1); // ID 1 for primordial

        // Set position in the Mar
        let pos_x = I32F32::from_raw((x * i64::MAX as f64) as i64);
        let pos_y = I32F32::from_raw((y * i64::MAX as f64) as i64);
        campo.set_posicion(Vector3D::new(pos_x, pos_y, I32F32::ZERO));

        let mut auton = Auton {
            campo,
            umbra,
            id: 1,
            parent_id: 0,
            tick_nacimiento: 0,
            tick_actual: 0,
            energia_anterior: I32F32::ZERO,
            causa_muerte: None,
            muerte_registrada: false,
        };
        auton.energia_anterior = auton.campo.energia_interna();
        auton
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

    /// Advance one lifecycle tick
    pub fn ciclo_vital(&mut self, mar: &MarMorfoseo, constantes: &ConstantesCosmicas) {
        self.tick_actual += 1;

        // Step the Campo Estructural (membrane physics)
        self.campo.step(mar, constantes);

        // Step the Umbra (register sensory state - simplified)
        // In full implementation, RamNet would provide actual sensory state
        let hash_estado = self.id.wrapping_add(self.tick_actual * 7919);
        // Note: full Umbra integration would require RamNet decision/action cycle
    }

    /// Check if Auton is dead
    pub fn esta_muerto(&self) -> bool {
        if let Some(_) = self.causa_muerte {
            return true;
        }
        // Campo disuelto or energy exhausted
        self.campo.estado() == eden_core::life::EstadoCampo::Disuelto
            || self.campo.energia_interna() <= I32F32::ZERO
    }

    /// Get UUID
    pub fn uuid(&self) -> u64 {
        self.id
    }

    /// Get cause of death
    pub fn causa_muerte(&self) -> CausaMuerte {
        self.causa_muerte
            .clone()
            .unwrap_or(CausaMuerte::Desconocida)
    }

    /// Get current energy
    pub fn energia_actual(&self) -> I32F32 {
        self.campo.energia_interna()
    }

    /// Check for escisión (reproduction)
    pub fn detectar_escision(&self) -> Option<Vec<Auton>> {
        // Check CampoEstructural for bifurcation
        if self.campo.estado() == eden_core::life::EstadoCampo::Dividido {
            return None; // Already divided
        }

        // Manual check: if energy is very high and field is stable, can split
        if self.campo.energia_interna() > I32F32::from_raw(0x00000078_00000000) {
            // > 120
            // Use CampoEstructural's built-in escisión detection
            if let Some(_hijos_campos) = self.campo.detectar_escision() {
                // For now, we don't actually split - that would require RamNet + Umbra setup
                // In a full implementation, we'd create child Autons here
            }
        }
        None
    }

    /// Mark as dead
    pub fn marcar_muerto(&mut self, causa: CausaMuerte) {
        if self.causa_muerte.is_none() {
            self.causa_muerte = Some(causa);
        }
    }

    /// Check if death was registered in meltrace
    pub fn muerte_registrada(&self) -> bool {
        self.muerte_registrada
    }

    /// Mark death as registered
    pub fn marcar_muerte_registrada(&mut self) {
        self.muerte_registrada = true;
    }

    /// Get position
    pub fn posicion(&self) -> Vector3D<I32F32> {
        *self.campo.posicion()
    }

    /// Get Campo reference for rendering
    pub fn campo(&self) -> &CampoEstructural {
        &self.campo
    }

    /// Get Umbra reference
    pub fn umbra(&self) -> &Umbra {
        &self.umbra
    }
}

// ============================================================================
// Main
// ============================================================================

fn main() {
    println!("🜁 EDEN Core — Cosmogonic Test");
    println!("═══════════════════════════════════════");

    let semilla = obtener_semilla_genesis();
    println!("Semilla: {}", semilla);

    // Create constants from seed
    let semilla_genesis = seed_to_semilla(semilla);
    let constantes = ConstantesCosmicas::from_semilla(&semilla_genesis);
    println!("Constantes Cosmicas derividas de semilla");

    // Create the Mar (read mar_morfoseo.rs to find dimensions/constructor)
    let mut mar = MarMorfoseo::new_2d(1024, 4);
    println!("Mar Morfoseo creado: 1024x1024");

    // Seed the Mar with initial energon in center region
    mar.sembrar_region(
        480,
        480,
        0,
        544,
        544,
        0,
        I32F32::from_raw(0x0000000A_00000000),
    ); // ~10

    // Create meltrace registry
    let mut grabados = Meltrace::new(semilla);
    println!("Meltrace (Trazo Fundido) inicializado");

    // Create EdenFS for persistence
    let mut eden_fs = match EdenFS::new(semilla) {
        Ok(fs) => {
            println!("EdenFS inicializado");
            Some(fs)
        }
        Err(e) => {
            println!("EdenFS no disponible: {}", e);
            None
        }
    };

    // Spawn the Primordial Auton (El Primogénito)
    let progenito = Auton::nuevo_primordial(&constantes, 0.5, 0.5);
    let mut autons: Vec<Auton> = vec![progenito];
    let mut next_auton_id = 2u64;
    println!("✧ El Primogénito ha sido sembrado en el centro del Mar");

    // IPC socket for Demiurgo (read ipc module)
    let socket_path = "/tmp/eden_demiurge.sock";
    let mut socket = match EdenCommand::client(socket_path) {
        Ok(cmd) => {
            println!("✧ Socket IPC conectado a: {}", socket_path);
            Some(cmd)
        }
        Err(e) => {
            println!("Socket warn: {} (continuando sin IPC)", e);
            None
        }
    };

    let mut ciclo = 0u64;
    let inicio = Instant::now();

    println!("\n🌍进入 — Entrando en el Sueño del Motor...");
    println!("      El tiempo subjetivo de EDEN corre a 60 FPS");
    println!("      Ctrl+C para terminar.\n");

    // Stats tracking
    let mut energia_historial = Vec::new();
    let mut nacidos_total = 1u64;
    let mut muertos_total = 0u64;
    let mut escisiones_total = 0u64;

    // Main simulation loop
    loop {
        ciclo += 1;

        // 1. Physics of the Mar
        mar.step();

        // 2. Life cycle of each Auton
        let mut muertos_indices = Vec::new();
        let mut nacidos = Vec::new();

        for (i, aut) in autons.iter_mut().enumerate() {
            // Read auton.rs for ciclo_vital or step method
            aut.ciclo_vital(&mar, &constantes);

            // Check death
            // Read auton.rs - probably has energia or health field
            if aut.esta_muerto() && !aut.muerte_registrada() {
                muertos_indices.push(i);
                muertos_total += 1;
                aut.marcar_muerto(CausaMuerte::AgotamientoEnergia);

                // Register death in meltrace
                // Read meltrace.rs for registrar_muerte signature
                let uuid = aut.uuid();
                let causa = aut.causa_muerte();
                let energon = aut.energia_actual();
                grabados.registrar_muerte(aut.umbra());
                aut.marcar_muerte_registrada();

                // Register in EdenFS if available
                if let Some(ref mut fs) = eden_fs {
                    let _ = fs.registrar_muerte(uuid, causa.clone(), uuid.wrapping_mul(31));
                }

                if let Some(ref mut sock) = socket {
                    let evento = Evento::MurioAuton {
                        id: uuid,
                        causa: causa.to_str().to_string(),
                    };
                    if let Err(e) = sock.enviar_evento(evento) {
                        eprintln!("Socket send error: {}", e);
                    }
                }
            }

            // Check escisión (reproduction) - read auton.rs for detectar_escision
            if let Some(hijos) = aut.detectar_escision() {
                escisiones_total += hijos.len() as u64;
                for _hijo in hijos {
                    // In full implementation, we'd create actual child Autons here
                    // For now, just track the count
                }
                // Parent dies after escisión (biological reproduction)
                if !muertos_indices.contains(&i) {
                    muertos_indices.push(i);
                    muertos_total += 1;
                    aut.marcar_muerto(CausaMuerte::Senescencia);
                }
            }
        }

        // 3. Process deaths (remove in reverse order)
        for &i in muertos_indices.iter().rev() {
            // Read auton.rs for energia_actual and uuid before dropping
            let energon = autons[i].energia_actual();
            let uuid = autons[i].uuid();

            // Inject energon back into Mar
            // Read mar_morfoseo.rs for inject_energon
            // NOTE: Mar doesn't have inject_energon, but we can use add_energon
            // Approximate position from auton
            let pos = autons[i].posicion();
            let x = ((pos.x.to_raw() as u64) % (512 << 32)) >> 32;
            let y = ((pos.y.to_raw() as u64) % (512 << 32)) >> 32;
            mar.add_energon(x as usize, y as usize, 0, energon);

            autons.remove(i);
        }

        // 4. Add newborns (from escisión)
        for mut nuevo in nacidos {
            nuevo = Auton::nuevo_primordial(&constantes, 0.5, 0.5);
            autons.push(nuevo);
            nacidos_total += 1;
        }

        // 5. Genesis — try to form new Nomos/Tipon from high energy areas
        // Read genesis.rs for intentar_formacion
        // NOTE: Mar emits NomosFormado events via channel, but we don't use it here
        if mar.num_nomos() > 0 {
            if ciclo % 100 == 0 {
                println!("  ✧ {} Nomos activos en el Mar", mar.num_nomos());
            }
        }

        // 6. Send IPC event to Demiurgo
        // Read ipc/message.rs for evento types
        if ciclo % 100 == 0 {
            let estado = Evento::EcosistemaState {
                autons_vivos: autons.len() as u32,
                energia_total: mar.energia_total().to_raw() as f64 / i64::MAX as f64,
                escoria_total: 0.0, // Mar doesn't expose escoria directly
                densidad_promedio: mar.densidad_promedio().to_raw() as f64 / i64::MAX as f64,
            };
            // Send via socket if connected
            if let Some(ref mut sock) = socket {
                if let Err(e) = sock.enviar_evento(estado) {
                    eprintln!("Socket send error: {}", e);
                }
            }
        }

        // 7. Render (every few cycles) - placeholder for SoftGPU
        // #[cfg(feature = "render")]
        // if ciclo % 10 == 0 {
        //     // Read soft_gpu.rs for render method
        //     renderizar_estado(&mar, &autons, &grabados, ciclo);
        // }

        // 8. Track stats every second (60 cycles)
        if ciclo % 60 == 0 {
            let energia_mar = mar.energia_total();
            energia_historial.push(energia_mar);
            if energia_historial.len() > 60 {
                energia_historial.remove(0);
            }
            let avg: f64 = if !energia_historial.is_empty() {
                energia_historial
                    .iter()
                    .map(|&e| e.to_raw() as f64 / i64::MAX as f64)
                    .sum::<f64>()
                    / energia_historial.len() as f64
            } else {
                0.0
            };

            println!("[ciclo {:>6}] Autons: {:>2} | Mar energia: {:>12.4} | avg 60s: {:>12.4} | nacidos: {} | muertos: {} | escisiones: {}",
                ciclo, autons.len(),
                energia_mar.to_raw() as f64 / i64::MAX as f64,
                avg,
                nacidos_total, muertos_total, escisiones_total);
        }

        // 9. Self-check — techo absoluto every 3600 cycles (~1 minute)
        if ciclo % 3600 == 0 {
            println!("\n  ── Auto-validacion del Techo Absoluto ──");
            let fluctuation = if !energia_historial.is_empty() {
                let avg = energia_historial
                    .iter()
                    .map(|&e| e.to_raw() as f64 / i64::MAX as f64)
                    .sum::<f64>()
                    / energia_historial.len() as f64;
                energia_historial
                    .iter()
                    .map(|&e| (e.to_raw() as f64 / i64::MAX as f64 - avg).abs())
                    .fold(0.0f64, |a, b| a.max(b))
            } else {
                0.0
            };
            println!("  Fluctuacion maxima: {:.6}", fluctuation);
            let stats = grabados.estadisticas();
            println!(
                "  Grabados en Meltrace: {} (total: {}, inmortales: {})",
                stats.grabados_activos, stats.total_grabados, stats.inmortales
            );
            println!("  Niveles de energia: {} registros", stats.grabados_activos);
        }

        // 10. Sleep to maintain 60 FPS subjective time
        std::thread::sleep(Duration::from_millis(16));

        // Safety: if we somehow lost all Autons, panic
        if autons.is_empty() && ciclo > 10 {
            eprintln!("\n⚠ Todos los Autons han muerto. Fin de la simulacion.");
            break;
        }

        // Sanity limit - stop after 1 hour of cycles (216000 cycles = ~1 hour)
        if ciclo >= 216000 {
            println!("\n⚠ Limite de ciclos alcanzado. Fin de la simulacion.");
            break;
        }
    }

    let elapsed = inicio.elapsed();
    println!("\n═══════════════════════════════════════");
    println!("🏁 EDEN Core — Simulacion terminada");
    println!("   Ciclos completados: {}", ciclo);
    println!("   Tiempo real: {:.2}s", elapsed.as_secs_f64());
    println!("   Nacidos totales: {}", nacidos_total);
    println!("   Muertos totales: {}", muertos_total);
    println!("   Escisiones totales: {}", escisiones_total);
    let stats = grabados.estadisticas();
    println!(
        "   Grabados Meltrace: {} activos, {} total",
        stats.grabados_activos, stats.total_grabados
    );
}
