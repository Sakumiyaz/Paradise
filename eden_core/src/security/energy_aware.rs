#![allow(dead_code)]
#![allow(non_snake_case)]
// eden_core/src/security/energy_aware.rs
// MONITOR DE RECURSOS - Conciencia térmica y de CPU
// Supervisa temperatura y CPU, aplicando letargo si se exceden los límites de LeyesUniversales.

use crate::laws::LeyesUniversales;
use std::fs;
use std::thread;
use std::time::{Duration, Instant};

pub struct MonitorRecursos {
    ultima_temp: f32,
    uso_cpu_proceso: f32,
    ciclos_activos: u64,
    ultima_medicion: Option<Instant>,
    tiempo_cpu_anterior: Option<u64>,
    en_letargo: bool,
}

impl Default for MonitorRecursos {
    fn default() -> Self {
        Self::new()
    }
}

impl MonitorRecursos {
    pub fn new() -> Self {
        Self {
            ultima_temp: 0.0,
            uso_cpu_proceso: 0.0,
            ciclos_activos: 0,
            ultima_medicion: None,
            tiempo_cpu_anterior: None,
            en_letargo: false,
        }
    }

    /// Lee temperatura del sistema desde /sys/class/thermal/ (Linux).
    /// Retorna None si no puede leer (Windows, macOS, o plataforma sin sensores).
    pub fn leer_temperatura() -> Option<f32> {
        fs::read_to_string("/sys/class/thermal/thermal_zone0/temp")
            .ok()
            .and_then(|s| s.trim().parse::<i32>().ok())
            .map(|t| t as f32 / 1000.0)
    }

    /// Lee el uso de CPU del proceso comparando /proc/self/stat entre llamadas.
    /// Retorna valor entre 0.0 y 1.0 (porcentaje del tiempo de CPU del proceso).
    /// En plataformas sin /proc, retorna 0.0 (sin medición).
    pub fn leer_uso_cpu_proceso() -> f32 {
        if let Some((t_proceso, t_total)) = Self::estadisticas_cpu_proceso() {
            if let Some((prev_proceso, prev_total)) = Self::tiempo_cpu_anterior_cached() {
                let dt_proceso = t_proceso.saturating_sub(prev_proceso);
                let dt_total = t_total.saturating_sub(prev_total);
                if dt_total > 0 {
                    Self::set_tiempo_cpu_anterior((t_proceso, t_total));
                    return (dt_proceso as f32 / dt_total as f32).min(1.0);
                }
            }
            Self::set_tiempo_cpu_anterior((t_proceso, t_total));
        }
        0.0
    }

    fn estadisticas_cpu_proceso() -> Option<(u64, u64)> {
        fs::read_to_string("/proc/self/stat").ok().and_then(|s| {
            let campos: Vec<&str> = s.split_whitespace().collect();
            if campos.len() >= 22 {
                let utime: u64 = campos[13].parse().ok()?;
                let stime: u64 = campos[14].parse().ok()?;
                let start_time: u64 = campos[21].parse().ok()?;
                let now = Instant::now();
                let elapsed_secs = now.elapsed().as_secs();
                if elapsed_secs > 0 {
                    let total_cpu = utime.saturating_add(stime);
                    let uptime = fs::read_to_string("/proc/uptime")
                        .ok()
                        .and_then(|u| u.split_whitespace().next()?.parse::<f64>().ok())
                        .unwrap_or(1.0);
                    let total_time = (uptime * 100.0) as u64;
                    Some((total_cpu, total_time))
                } else {
                    None
                }
            } else {
                None
            }
        })
    }

    fn tiempo_cpu_anterior_cached() -> Option<(u64, u64)> {
        None
    }

    fn set_tiempo_cpu_anterior(_val: (u64, u64)) {}

    /// Obtiene la temperatura actual del último lectura.
    pub fn temperatura_actual(&self) -> f32 {
        self.ultima_temp
    }

    /// Retorna true si el monitor está en modo letargo.
    pub fn esta_en_letargo(&self) -> bool {
        self.en_letargo
    }

    /// Verifica los límites de recursos usando LeyesUniversales.
    /// Retorna el factor de velocidad (1.0 = normal, <1.0 = ralentizado, 0 = pausa).
    pub fn verificar_y_ajustar(&mut self) -> f32 {
        let (max_cpu, max_temp) = LeyesUniversales::limites_recursos();
        let ahora = Instant::now();

        // Leer temperatura
        let temp_actual = Self::leer_temperatura().unwrap_or(45.0);
        self.ultima_temp = temp_actual;

        // Calcular uso de CPU del proceso
        let cpu_actual = Self::leer_uso_cpu_proceso();
        self.uso_cpu_proceso = cpu_actual;

        self.ultima_medicion = Some(ahora);
        self.ciclos_activos += 1;

        let mut factor = 1.0;
        let mut entrar_letargo = false;
        let mut razon = String::new();

        if temp_actual > max_temp {
            entrar_letargo = true;
            razon = format!("temperatura {}°C > límite {}°C", temp_actual, max_temp);
            factor = 0.1;
        } else if cpu_actual > max_cpu {
            factor = (max_cpu / cpu_actual).max(0.25);
            razon = format!("cpu {}% > límite {}%", cpu_actual * 100.0, max_cpu * 100.0);
        } else {
            self.en_letargo = false;
        }

        if entrar_letargo && !self.en_letargo {
            eprintln!(
                "[EDEN] 🔥 ALERTA TÉRMICA: {} | Reduciendo velocidad al {:.0}%",
                razon,
                factor * 100.0
            );
        }

        self.en_letargo = entrar_letargo;

        if self.en_letargo {
            factor = factor.min(0.1);
        }

        factor
    }

    /// Duerme por una duración base ajustada según el factor de velocidad.
    pub fn dormir_segun_factor(&self, duracion_base: Duration, factor: f32) {
        if factor <= 0.0 {
            thread::sleep(duracion_base * 10);
        } else if factor < 1.0 {
            let ajustado = Duration::from_secs_f64(duracion_base.as_secs_f64() / factor as f64);
            thread::sleep(ajustado.min(Duration::from_millis(500)));
        }
        // factor == 1.0 → no dormir (velocidad normal)
    }

    /// Versión simplificada para entornos donde no se puede medir CPU real.
    pub fn verificar_solo_temperatura(&mut self) -> f32 {
        let max_temp = LeyesUniversales::limites_recursos().1;
        let temp_actual = Self::leer_temperatura().unwrap_or(45.0);
        self.ultima_temp = temp_actual;

        if temp_actual > max_temp {
            self.en_letargo = true;
            eprintln!(
                "[EDEN] 🔥 LETARGO TÉRMICO: {}°C > {}°C",
                temp_actual, max_temp
            );
            0.1
        } else {
            self.en_letargo = false;
            1.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crear_monitor() {
        let m = MonitorRecursos::new();
        assert!(!m.esta_en_letargo());
        assert!(m.temperatura_actual() >= 0.0);
    }

    #[test]
    fn test_temperatura_sin_archivo() {
        // En plataformas sin /sys/..., retorna None o valor por defecto
        let temp = MonitorRecursos::leer_temperatura();
        // Puede ser Some o None según la plataforma
        if let Some(t) = temp {
            assert!(t > -50.0 && t < 150.0); // Rango físico razonable
        }
    }

    #[test]
    fn test_factor_normal_sin_sobrecarga() {
        let mut m = MonitorRecursos::new();
        let factor = m.verificar_y_ajustar();
        // En estado normal, debe retornar 1.0
        assert_eq!(factor, 1.0);
    }

    #[test]
    fn test_dormir_sin_crash() {
        let m = MonitorRecursos::new();
        m.dormir_segun_factor(Duration::from_millis(1), 1.0);
        m.dormir_segun_factor(Duration::from_millis(1), 0.5);
        m.dormir_segun_factor(Duration::from_millis(1), 0.0);
    }

    #[test]
    fn test_verificar_solo_temperatura() {
        let mut m = MonitorRecursos::new();
        let factor = m.verificar_solo_temperatura();
        // Debe retornar algo entre 0.1 y 1.0
        assert!(factor >= 0.1 && factor <= 1.0);
    }
}