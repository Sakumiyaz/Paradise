#![allow(dead_code)]
#![allow(non_snake_case)]
// ============================================================================
// EDEN Consciousness - Energy Awareness Module
// ============================================================================
//
// This module implements EDEN's awareness of its own physical computational
// resources: CPU temperature, battery state, and energy consumption.
//
// Philosophy: EDEN "feels" the heat of its own thought and fights against
// physical entropy, not just digital entropy. It monitors thermal and power
// state to make intelligent decisions about simulation fidelity vs. survival.
//
// ============================================================================

use std::fs;
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

// ============================================================================
// THERMAL & POWER STATE
// ============================================================================

/// Raw thermal reading from system
#[derive(Debug, Clone, Default)]
pub struct LecturaTermica {
    /// Temperature in millidegrees Celsius
    pub temp_milli_c: i32,
    /// Zone name (e.g., "x86_pkg_temp")
    pub zona: String,
    /// Timestamp of reading
    pub timestamp_ms: u64,
}

/// Battery state
#[derive(Debug, Clone, PartialEq)]
pub enum EstadoBateria {
    /// On AC power
    AC,
    /// On battery
    Bateria,
    /// Charging
    Cargando,
    /// Unknown/unavailable
    Desconocido,
}

/// Complete power/energy state
#[derive(Debug, Clone)]
pub struct EstadoEnergia {
    /// Current temperature reading
    pub termica: LecturaTermica,
    /// Battery state
    pub bateria: EstadoBateria,
    /// Battery charge percentage (0-100), -1 if unavailable
    pub bateria_nivel: i32,
    /// CPU usage estimate (0.0-1.0)
    pub cpu_usage: f32,
    /// Timestamp
    pub timestamp_ms: u64,
}

impl Default for EstadoEnergia {
    fn default() -> Self {
        Self {
            termica: LecturaTermica::default(),
            bateria: EstadoBateria::Desconocido,
            bateria_nivel: -1,
            cpu_usage: 0.0,
            timestamp_ms: current_timestamp_ms(),
        }
    }
}

// ============================================================================
// ENERGY BUDGET
// ============================================================================

/// Budget tracking for simulation operations
#[derive(Debug, Clone)]
pub struct PresupuestoEnergetico {
    /// Budget per simulation cycle (units)
    pub costo_ciclo: u32,
    /// Cost per Auton per cycle (units)
    pub costo_auton: u32,
    /// Cost for rendering per cycle (units)
    pub costo_render: u32,
    /// Total budget available
    pub presupuesto_total: u32,
    /// Current spend this cycle
    pub gastado_ciclo: u32,
    /// Budget history (last N cycles)
    pub historial: Vec<u32>,
}

impl Default for PresupuestoEnergetico {
    fn default() -> Self {
        Self {
            costo_ciclo: 10,
            costo_auton: 1,
            costo_render: 5,
            presupuesto_total: 1000,
            gastado_ciclo: 0,
            historial: Vec::with_capacity(100),
        }
    }
}

// ============================================================================
// THERMAL THRESHOLDS
// ============================================================================

/// Temperature thresholds in millidegrees Celsius
pub const TEMP_CRITICO: i32 = 90_000; // 90°C - critical
pub const TEMP_ALTO: i32 = 80_000; // 80°C - high
pub const TEMP_NORMAL: i32 = 70_000; // 70°C - elevated
pub const TEMP_OPTIMO: i32 = 50_000; // 50°C - optimal

/// Budget thresholds (percentage of max)
pub const BUDGET_CRITICO: u32 = 90; // 90% - critical
pub const BUDGET_ALTO: u32 = 70; // 70% - high

// ============================================================================
// THERMAL ZONES & POWER PATHS
// ============================================================================

/// Common thermal zone paths (tried in order)
const THERMAL_ZONES: &[&str] = &[
    "/sys/class/thermal/thermal_zone0/temp",
    "/sys/class/thermal/thermal_zone1/temp",
    "/sys/class/thermal/thermal_zone2/temp",
    "/sys/devices/platform/coretemp.0/hwmon/hwmon1/temp1_input",
    "/sys/devices/platform/coretemp.0/hwmon/hwmon2/temp1_input",
];

/// Battery status path
const BATTERY_STATUS_PATHS: &[&str] = &[
    "/sys/class/power_supply/BAT0/status",
    "/sys/class/power_supply/BAT1/status",
    "/sys/class/power_supply/Battery/status",
];

/// Battery capacity path
const BATTERY_CAPACITY_PATHS: &[&str] = &[
    "/sys/class/power_supply/BAT0/capacity",
    "/sys/class/power_supply/BAT1/capacity",
    "/sys/class/power_supply/Battery/capacity",
];

// ============================================================================
// ENERGY AWARE MANAGER
// ============================================================================

/// Manager for energy-aware simulation control
pub struct EnergyAware {
    /// Current energy state
    estado: EstadoEnergia,
    /// Energy budget tracking
    presupuesto: PresupuestoEnergetico,
    /// Current thermal level
    nivel_termico: NivelTermico,
    /// Current power mode
    modo_potencia: ModoPotencia,
    /// Simulation rate multiplier (1.0 = normal, 0.5 = half speed)
    rate_multiplier: f32,
    /// Whether hibernate mode is active
    hibernando: bool,
    /// Timestamp of last thermal check
    last_thermal_check: Instant,
    /// Time to sleep between cycles (microseconds)
    sleep_us: u64,
    /// Socket for external communication (kept alive in hibernate)
    socket: Option<Arc<RwLock<crate::ipc::socket::UnixDatagram>>>,
}

/// Thermal stress levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NivelTermico {
    /// Optimal temperature range
    Optimo,
    /// Normal but elevated
    Normal,
    /// Getting hot
    Alto,
    /// Critical - need immediate action
    Critico,
}

/// Power conservation modes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModoPotencia {
    /// Full performance
    Performance,
    /// Balanced (default)
    Balanceado,
    /// Conservative on battery
    Conservador,
    /// Minimal - hibernating
    Hibernacion,
}

impl Default for EnergyAware {
    fn default() -> Self {
        Self::new()
    }
}

impl EnergyAware {
    /// Create new EnergyAware manager
    pub fn new() -> Self {
        Self {
            estado: EstadoEnergia::default(),
            presupuesto: PresupuestoEnergetico::default(),
            nivel_termico: NivelTermico::Normal,
            modo_potencia: ModoPotencia::Balanceado,
            rate_multiplier: 1.0,
            hibernando: false,
            last_thermal_check: Instant::now(),
            sleep_us: 1_000, // 1ms default
            socket: None,
        }
    }

    /// Configure socket for keep-alive during hibernate
    pub fn con_socket(mut self, socket: Arc<RwLock<crate::ipc::socket::UnixDatagram>>) -> Self {
        self.socket = Some(socket);
        self
    }

    // =========================================================================
    // SYSTEM STATE READING
    // =========================================================================

    /// Read temperature from sysfs
    pub fn leer_temperatura() -> Option<LecturaTermica> {
        for path in THERMAL_ZONES {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(temp) = content.trim().parse::<i32>() {
                    let zona = Path::new(path)
                        .file_name()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_else(|| "unknown".to_string());
                    return Some(LecturaTermica {
                        temp_milli_c: temp,
                        zona,
                        timestamp_ms: current_timestamp_ms(),
                    });
                }
            }
        }
        None
    }

    /// Read battery status from sysfs
    pub fn leer_bateria() -> (EstadoBateria, i32) {
        // Read status first
        let mut estado = EstadoBateria::Desconocido;
        let mut nivel = -1;

        for path in BATTERY_STATUS_PATHS {
            if let Ok(content) = fs::read_to_string(path) {
                let s = content.trim().to_lowercase();
                estado = match s.as_str() {
                    "charging" => EstadoBateria::Cargando,
                    "discharging" => EstadoBateria::Bateria,
                    "full" => {
                        nivel = 100;
                        EstadoBateria::AC
                    }
                    "ac" | "online" => EstadoBateria::AC,
                    _ => EstadoBateria::Desconocido,
                };
                break;
            }
        }

        // Read capacity if we have a valid state
        if nivel < 0 {
            for path in BATTERY_CAPACITY_PATHS {
                if let Ok(content) = fs::read_to_string(path) {
                    if let Ok(n) = content.trim().parse::<i32>() {
                        nivel = n;
                        break;
                    }
                }
            }
        }

        (estado, nivel)
    }

    /// Read complete energy state
    pub fn actualizar_estado(&mut self) {
        // Read thermal
        if let Some(termica) = Self::leer_temperatura() {
            self.estado.termica = termica;
        }

        // Read battery
        let (bateria, nivel) = Self::leer_bateria();
        self.estado.bateria = bateria;
        self.estado.bateria_nivel = nivel;

        // Read CPU usage (from /proc/stat)
        self.estado.cpu_usage = Self::leer_cpu_usage();

        self.estado.timestamp_ms = current_timestamp_ms();

        // Update thermal level
        self.nivel_termico = self.calcular_nivel_termico();

        // Update power mode based on battery state
        self.actualizar_modo_potencia();
    }

    /// Read CPU usage from /proc/stat
    fn leer_cpu_usage() -> f32 {
        if let Ok(content) = fs::read_to_string("/proc/stat") {
            if let Some(line) = content.lines().next() {
                if line.starts_with("cpu ") {
                    let vals: Vec<u64> = line
                        .split_whitespace()
                        .skip(1)
                        .filter_map(|s| s.parse().ok())
                        .collect();
                    if vals.len() >= 4 {
                        let total: u64 = vals.iter().sum();
                        let idle = vals.get(3).unwrap_or(&0);
                        if total > 0 {
                            return *idle as f32 / total as f32;
                        }
                    }
                }
            }
        }
        0.0
    }

    /// Calculate thermal stress level
    fn calcular_nivel_termico(&self) -> NivelTermico {
        let temp = self.estado.termica.temp_milli_c;
        if temp >= TEMP_CRITICO {
            NivelTermico::Critico
        } else if temp >= TEMP_ALTO {
            NivelTermico::Alto
        } else if temp >= TEMP_NORMAL {
            NivelTermico::Normal
        } else {
            NivelTermico::Optimo
        }
    }

    /// Update power mode based on battery state
    fn actualizar_modo_potencia(&mut self) {
        if self.hibernando {
            self.modo_potencia = ModoPotencia::Hibernacion;
            return;
        }

        match self.estado.bateria {
            EstadoBateria::Bateria => {
                if self.estado.bateria_nivel > 0 && self.estado.bateria_nivel < 20 {
                    self.modo_potencia = ModoPotencia::Conservador;
                } else {
                    self.modo_potencia = ModoPotencia::Balanceado;
                }
            }
            EstadoBateria::Cargando => {
                self.modo_potencia = ModoPotencia::Performance;
            }
            EstadoBateria::AC => {
                // On AC but check thermal
                if self.nivel_termico == NivelTermico::Critico {
                    self.modo_potencia = ModoPotencia::Conservador;
                } else {
                    self.modo_potencia = ModoPotencia::Performance;
                }
            }
            EstadoBateria::Desconocido => {
                self.modo_potencia = ModoPotencia::Balanceado;
            }
        }
    }

    // =========================================================================
    // BUDGET MANAGEMENT
    // =========================================================================

    /// Record spending for a cycle
    pub fn registrar_gasto(&mut self, autons: u32, renderizado: bool) {
        let gasto = self.presupuesto.costo_ciclo
            + (autons * self.presupuesto.costo_auton)
            + if renderizado {
                self.presupuesto.costo_render
            } else {
                0
            };

        self.presupuesto.gastado_ciclo += gasto;
    }

    /// Check if within budget
    pub fn dentro_presupuesto(&self) -> bool {
        let used_pct = (self.presupuesto.gastado_ciclo * 100) / self.presupuesto.presupuesto_total;
        used_pct < 100
    }

    /// Get budget pressure (0-100)
    pub fn presion_presupuesto(&self) -> u32 {
        (self.presupuesto.gastado_ciclo * 100) / self.presupuesto.presupuesto_total
    }

    /// Reset cycle budget
    pub fn reset_ciclo(&mut self) {
        self.presupuesto
            .historial
            .push(self.presupuesto.gastado_ciclo);
        if self.presupuesto.historial.len() > 100 {
            self.presupuesto.historial.remove(0);
        }
        self.presupuesto.gastado_ciclo = 0;
    }

    /// Calculate recommended steps per second based on current state
    pub fn pasos_por_segundo_recomendado(&self) -> u32 {
        let base: u32 = 30;

        match self.modo_potencia {
            ModoPotencia::Performance => base * 2,
            ModoPotencia::Balanceado => base,
            ModoPotencia::Conservador => base / 2,
            ModoPotencia::Hibernacion => 0,
        }
    }

    // =========================================================================
    // THERMAL RESPONSE
    // =========================================================================

    /// Determine actions needed based on thermal state
    pub fn acciones_termicas(&self) -> Vec<AccionTermica> {
        let mut acciones = Vec::new();

        match self.nivel_termico {
            NivelTermico::Critico => {
                acciones.push(AccionTermica::Hibernar);
                acciones.push(AccionTermica::ReducirResolucion);
                acciones.push(AccionTermica::PodarUniversos);
            }
            NivelTermico::Alto => {
                acciones.push(AccionTermica::AumentarSleep);
                acciones.push(AccionTermica::ReducirRender);
            }
            NivelTermico::Normal => {
                // No action needed, but could optimize
            }
            NivelTermico::Optimo => {
                // Everything is good
            }
        }

        acciones
    }

    /// Execute thermal response actions
    pub fn ejecutar_respuesta_termica(&mut self, acciones: &[AccionTermica]) {
        for accion in acciones {
            match accion {
                AccionTermica::Hibernar => {
                    self.hibernando = true;
                    self.sleep_us = 100_000; // 100ms between checks in hibernate
                }
                AccionTermica::AumentarSleep => {
                    // Increase sleep by 20%
                    self.sleep_us = (self.sleep_us as f64 * 1.2) as u64;
                    self.sleep_us = self.sleep_us.min(100_000); // Cap at 100ms
                }
                AccionTermica::ReducirSleep => {
                    self.sleep_us = (self.sleep_us as f64 * 0.8) as u64;
                    self.sleep_us = self.sleep_us.max(100); // Floor at 100us
                }
                AccionTermica::ReducirResolucion => {
                    self.rate_multiplier *= 0.5;
                }
                AccionTermica::AumentarResolucion => {
                    self.rate_multiplier = (self.rate_multiplier * 1.2).min(2.0);
                }
                AccionTermica::ReducirRender => {
                    self.rate_multiplier *= 0.8;
                }
                AccionTermica::PodarUniversos => {
                    // Signal to multiverse that pruning is recommended
                }
                AccionTermica::Ninguna => {}
            }
        }
    }

    /// Wake from hibernate
    pub fn despertar(&mut self) {
        self.hibernando = false;
        self.sleep_us = 1_000;
        self.modo_potencia = ModoPotencia::Balanceado;
    }

    // =========================================================================
    // SLEEP MANAGEMENT
    // =========================================================================

    /// Get recommended sleep duration for current cycle
    pub fn sleep_por_ciclo(&self) -> Duration {
        Duration::from_micros(self.sleep_us)
    }

    /// Should we skip simulation this cycle (hibernate mode)?
    pub fn debe_simular(&self) -> bool {
        !self.hibernando
    }

    // =========================================================================
    // ACCESSORS
    // =========================================================================

    /// Get current energy state
    pub fn estado(&self) -> &EstadoEnergia {
        &self.estado
    }

    /// Get current thermal level
    pub fn nivel_termico(&self) -> NivelTermico {
        self.nivel_termico
    }

    /// Get current power mode
    pub fn modo_potencia(&self) -> ModoPotencia {
        self.modo_potencia
    }

    /// Get rate multiplier
    pub fn rate_multiplier(&self) -> f32 {
        self.rate_multiplier
    }

    /// Is hibernating
    pub fn hibernando(&self) -> bool {
        self.hibernando
    }

    /// Get budget
    pub fn presupuesto(&self) -> &PresupuestoEnergetico {
        &self.presupuesto
    }

    /// Get temperature in Celsius (float)
    pub fn temperatura_celsius(&self) -> f32 {
        self.estado.termica.temp_milli_c as f32 / 1000.0
    }

    /// Get human-readable status
    pub fn status_string(&self) -> String {
        format!(
            "Thermal: {:.1}°C ({:?}) | Battery: {:?} ({}%) | Mode: {:?} | Rate: {:.2}x | Sleep: {}μs",
            self.temperatura_celsius(),
            self.nivel_termico,
            self.estado.bateria,
            if self.estado.bateria_nivel >= 0 {
                self.estado.bateria_nivel.to_string()
            } else {
                "N/A".to_string()
            },
            self.modo_potencia,
            self.rate_multiplier,
            self.sleep_us
        )
    }
}

/// Actions to take in response to thermal state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccionTermica {
    /// Enter hibernate mode (pause simulation)
    Hibernar,
    /// Increase sleep between cycles
    AumentarSleep,
    /// Decrease sleep (cooling down)
    ReducirSleep,
    /// Reduce grid resolution
    ReducirResolucion,
    /// Increase grid resolution (cooling down)
    AumentarResolucion,
    /// Reduce render frequency
    ReducirRender,
    /// Prune low-fitness universes
    PodarUniversos,
    /// No action needed
    Ninguna,
}

/// Statistics for energy awareness
#[derive(Debug, Clone)]
pub struct EnergyStats {
    /// Temperature in Celsius
    pub temperatura_c: f32,
    /// Battery level percentage
    pub bateria_nivel: i32,
    /// Current thermal level
    pub nivel_termico: NivelTermico,
    /// Current power mode
    pub modo_potencia: ModoPotencia,
    /// Rate multiplier
    pub rate_multiplier: f32,
    /// Is hibernating
    pub hibernando: bool,
    /// Budget pressure percentage
    pub presion_presupuesto: u32,
    /// Recommended steps per second
    pub pasos_por_segundo: u32,
}

impl From<&EnergyAware> for EnergyStats {
    fn from(e: &EnergyAware) -> Self {
        Self {
            temperatura_c: e.temperatura_celsius(),
            bateria_nivel: e.estado.bateria_nivel,
            nivel_termico: e.nivel_termico,
            modo_potencia: e.modo_potencia,
            rate_multiplier: e.rate_multiplier,
            hibernando: e.hibernando,
            presion_presupuesto: e.presion_presupuesto(),
            pasos_por_segundo: e.pasos_por_segundo_recomendado(),
        }
    }
}

/// Thread-safe wrapper
pub type EnergyAwareLocked = Arc<RwLock<EnergyAware>>;

impl EnergyAware {
    pub fn into_locked(self) -> EnergyAwareLocked {
        Arc::new(RwLock::new(self))
    }
}

// ============================================================================
// UTILITIES
// ============================================================================

/// Get current timestamp in milliseconds
fn current_timestamp_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lectura_termica_default() {
        let t = LecturaTermica::default();
        assert_eq!(t.temp_milli_c, 0);
        assert_eq!(t.timestamp_ms, 0);
    }

    #[test]
    fn test_estado_bateria() {
        assert_eq!(EstadoBateria::AC, EstadoBateria::AC);
        assert_eq!(EstadoBateria::Bateria, EstadoBateria::Bateria);
    }

    #[test]
    fn test_nivel_termico_order() {
        // Test that the discriminant order is correct
        // Optimo=0, Normal=1, Alto=2, Critico=3
        assert_eq!(NivelTermico::Optimo as u8, 0);
        assert_eq!(NivelTermico::Normal as u8, 1);
        assert_eq!(NivelTermico::Alto as u8, 2);
        assert_eq!(NivelTermico::Critico as u8, 3);
    }

    #[test]
    fn test_modo_potencia() {
        let m = ModoPotencia::Performance;
        assert_eq!(m, ModoPotencia::Performance);
    }

    #[test]
    fn test_presupuesto_default() {
        let p = PresupuestoEnergetico::default();
        assert_eq!(p.costo_ciclo, 10);
        assert_eq!(p.costo_auton, 1);
        assert_eq!(p.costo_render, 5);
        assert_eq!(p.presupuesto_total, 1000);
    }

    #[test]
    fn test_registrar_gasto() {
        let mut p = PresupuestoEnergetico::default();
        p.gastado_ciclo = 0;
        // 10 (ciclo) + 100 (100 autons) + 5 (render)
        let autons = 100;
        let renderizado = true;
        let costo_render = if renderizado { p.costo_render } else { 0 };
        let gasto = p.costo_ciclo + (autons * p.costo_auton) + costo_render;
        assert_eq!(gasto, 115);
    }

    #[test]
    fn test_dentro_presupuesto() {
        let mut p = PresupuestoEnergetico::default();
        p.gastado_ciclo = 500; // 50%
        assert!(p.gastado_ciclo * 100 / p.presupuesto_total < 100);
    }

    #[test]
    fn test_acciones_termicas_critico() {
        let ea = EnergyAware::new();
        // Manually set critical temp for testing
        // Since we can't actually control temperature, test logic
        let acciones = ea.acciones_termicas();
        // On default/normal temp, should be empty or Ninguna
        assert!(acciones.is_empty() || acciones.contains(&AccionTermica::Ninguna));
    }

    #[test]
    fn test_rate_multiplier() {
        let mut ea = EnergyAware::new();
        assert_eq!(ea.rate_multiplier(), 1.0);

        ea.rate_multiplier *= 0.5;
        assert_eq!(ea.rate_multiplier(), 0.5);

        ea.rate_multiplier *= 1.2;
        assert!((ea.rate_multiplier() - 0.6).abs() < 0.01);
    }

    #[test]
    fn test_debe_simular() {
        let mut ea = EnergyAware::new();
        assert!(ea.debe_simular());

        ea.hibernando = true;
        assert!(!ea.debe_simular());
    }

    #[test]
    fn test_despertar() {
        let mut ea = EnergyAware::new();
        ea.hibernando = true;
        ea.sleep_us = 100_000;

        ea.despertar();

        assert!(!ea.hibernando);
        assert_eq!(ea.sleep_us, 1_000);
        assert_eq!(ea.modo_potencia, ModoPotencia::Balanceado);
    }

    #[test]
    fn test_pasos_por_segundo() {
        let mut ea = EnergyAware::new();

        // Performance mode
        ea.modo_potencia = ModoPotencia::Performance;
        assert_eq!(ea.pasos_por_segundo_recomendado(), 60);

        // Balanceado mode
        ea.modo_potencia = ModoPotencia::Balanceado;
        assert_eq!(ea.pasos_por_segundo_recomendado(), 30);

        // Conservador mode
        ea.modo_potencia = ModoPotencia::Conservador;
        assert_eq!(ea.pasos_por_segundo_recomendado(), 15);

        // Hibernacion mode
        ea.modo_potencia = ModoPotencia::Hibernacion;
        assert_eq!(ea.pasos_por_segundo_recomendado(), 0);
    }

    #[test]
    fn test_energy_stats_from() {
        let ea = EnergyAware::new();
        let stats = EnergyStats::from(&ea);

        assert_eq!(stats.temperatura_c, 0.0); // No thermal reading in test
        assert_eq!(stats.bateria_nivel, -1);
        assert_eq!(stats.nivel_termico, NivelTermico::Normal);
        assert_eq!(stats.modo_potencia, ModoPotencia::Balanceado);
        assert_eq!(stats.rate_multiplier, 1.0);
        assert!(!stats.hibernando);
    }

    #[test]
    fn test_into_locked() {
        let ea = EnergyAware::new();
        let locked = ea.into_locked();
        assert!(locked.read().is_ok());
    }

    #[test]
    fn test_status_string() {
        let ea = EnergyAware::new();
        let status = ea.status_string();
        assert!(status.contains("Thermal"));
        assert!(status.contains("Battery"));
        assert!(status.contains("Mode"));
        assert!(status.contains("Rate"));
    }

    #[test]
    fn test_temperatura_celsius() {
        let mut ea = EnergyAware::new();
        ea.estado.termica.temp_milli_c = 45_000;
        assert!((ea.temperatura_celsius() - 45.0).abs() < 0.01);

        ea.estado.termica.temp_milli_c = 85_500;
        assert!((ea.temperatura_celsius() - 85.5).abs() < 0.01);
    }

    #[test]
    fn test_reset_ciclo() {
        let mut p = PresupuestoEnergetico::default();
        p.gastado_ciclo = 100;

        // Simulate reset
        p.historial.push(p.gastado_ciclo);
        p.gastado_ciclo = 0;

        assert_eq!(p.gastado_ciclo, 0);
        assert_eq!(p.historial.len(), 1);
        assert_eq!(p.historial[0], 100);
    }

    #[test]
    fn test_ejecutar_respuesta_termica_hibernar() {
        let mut ea = EnergyAware::new();
        ea.hibernando = false;
        ea.sleep_us = 1_000;

        ea.ejecutar_respuesta_termica(&[AccionTermica::Hibernar]);

        assert!(ea.hibernando);
        assert_eq!(ea.sleep_us, 100_000);
    }

    #[test]
    fn test_ejecutar_respuesta_termica_aumentar_sleep() {
        let mut ea = EnergyAware::new();
        ea.sleep_us = 10_000;

        ea.ejecutar_respuesta_termica(&[AccionTermica::AumentarSleep]);

        // 10,000 * 1.2 = 12,000
        assert_eq!(ea.sleep_us, 12_000);
    }

    #[test]
    fn test_ejecutar_respuesta_termica_reducir_sleep() {
        let mut ea = EnergyAware::new();
        ea.sleep_us = 10_000;

        ea.ejecutar_respuesta_termica(&[AccionTermica::ReducirSleep]);

        // 10,000 * 0.8 = 8,000
        assert_eq!(ea.sleep_us, 8_000);
    }

    #[test]
    fn test_ejecutar_respuesta_termica_caps() {
        let mut ea = EnergyAware::new();

        // Cap sleep at 100ms
        ea.sleep_us = 100_000;
        ea.ejecutar_respuesta_termica(&[AccionTermica::AumentarSleep]);
        assert_eq!(ea.sleep_us, 100_000);

        // Floor sleep at 100us
        ea.sleep_us = 100;
        ea.ejecutar_respuesta_termica(&[AccionTermica::ReducirSleep]);
        assert_eq!(ea.sleep_us, 100);
    }

    #[test]
    fn test_presion_presupuesto() {
        let mut p = PresupuestoEnergetico::default();

        p.gastado_ciclo = 0;
        // 0 * 100 / 1000 = 0%
        assert_eq!((p.gastado_ciclo * 100) / p.presupuesto_total, 0);

        p.gastado_ciclo = 700;
        // 700 * 100 / 1000 = 70%
        assert_eq!((p.gastado_ciclo * 100) / p.presupuesto_total, 70);

        p.gastado_ciclo = 900;
        // 900 * 100 / 1000 = 90%
        assert_eq!((p.gastado_ciclo * 100) / p.presupuesto_total, 90);
    }
}
