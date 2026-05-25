//! # PneumaBonds: Enlaces Pneumáticos
//!
//! Los Enlaces Pneumáticos son conexiones persistentes que se forman entre
//! Auton cuando sus Umbrae resuenan durante períodos prolongados.
//!
//! ## Formación
//!
//! Un enlace se forma cuando:
//! - Dos Auton permanecen a distancia < RADIO_COMUNICACION durante N ciclos
//! - Sus fases de Umbra están alineadas (diferencia de hash < UMBRAL_FASE)
//!
//! ## Efectos
//!
//! 1. **Compartir percepción**: Un Auton puede consultar la RamNet del otro
//!    con penalización de latencia
//! 2. **Transferencia de energía**: Flujo lento de energía del más cargado
//!    al menos cargado (altruismo emergente)
//! 3. **Herencia compartida**: Los hijos de ambos pueden heredar fragmentos
//!    de ambas RamNets (intercambio genético horizontal)
//!
//! ## Ruptura
//!
//! Los enlaces se rompen si:
//! - La distancia aumenta más allá de RADIO_COMUNICACION
//! - Uno de los Auton sufre mutación drástica en su Umbra
#![allow(dead_code)]
#![allow(non_snake_case)]

use crate::life::umbra::{HashEstado, Umbra};
use std::collections::VecDeque;
use std::vec::Vec;

/// Radio de comunicación para formación de enlaces (en unidades normalizadas 0..1)
pub const RADIO_COMUNICACION: f64 = 0.15;

/// Número de ciclos necesarios para formar un enlace
pub const CICLOS_RESONANCIA: u32 = 50;

/// Umbral de diferencia de hash para considerar fases alineadas
pub const UMBRAL_FASE: u64 = 0x1000; // 4096 diferencias máximo

/// Umbral de energía para transferencia (mínimo para transferir)
pub const UMBRAL_ENERGIA_TRANSFERENCIA: i64 = 0x00000020_00000000; // 32.0 en I32F32

/// Tasa de transferencia de energía (por ciclo)
pub const TASA_TRANSFERENCIA: i64 = 0x00000001_00000000; // 1.0 por ciclo

/// Mutación drástica: diferencia de hash mayor a este valor
pub const UMBRAL_MUTACION_DRASTICA: u64 = 0x10000; // 65536

/// Representa un enlace neumático entre dos Auton
#[derive(Debug, Clone)]
pub struct PneumaBond {
    /// ID del primer Auton
    pub id_auton_a: u64,
    /// ID del segundo Auton
    pub id_auton_b: u64,
    /// Hash de fase del Auton A al formar el enlace
    pub fase_a: HashEstado,
    /// Hash de fase del Auton B al formar el enlace
    pub fase_b: HashEstado,
    /// Ciclos que llevan cerca (para formación)
    pub ciclos_cerca: u32,
    /// Ciclos que el enlace ha estado activo
    pub ciclos_activo: u32,
    /// Energía transferida total (para estadísticas)
    pub energia_transferida_total: i64,
    /// Timestamp de formación (tick)
    pub tick_formacion: u64,
    /// Si el enlace está activo
    pub activo: bool,
}

impl PneumaBond {
    /// Crea un nuevo enlace
    pub fn nuevo(
        id_auton_a: u64,
        id_auton_b: u64,
        fase_a: HashEstado,
        fase_b: HashEstado,
        tick: u64,
    ) -> Self {
        PneumaBond {
            id_auton_a,
            id_auton_b,
            fase_a,
            fase_b,
            ciclos_cerca: 0,
            ciclos_activo: 0,
            energia_transferida_total: 0,
            tick_formacion: tick,
            activo: false,
        }
    }

    /// Verifica si las fases están alineadas
    pub fn fases_alineadas(&self) -> bool {
        let diff = if self.fase_a > self.fase_b {
            self.fase_a - self.fase_b
        } else {
            self.fase_b - self.fase_a
        };
        diff < UMBRAL_FASE
    }

    /// Verifica si hubo mutación drástica desde la formación
    pub fn mutacion_drastica(&self, fase_actual_a: HashEstado, fase_actual_b: HashEstado) -> bool {
        let diff_a = if fase_actual_a > self.fase_a {
            fase_actual_a - self.fase_a
        } else {
            self.fase_a - fase_actual_a
        };
        let diff_b = if fase_actual_b > self.fase_b {
            fase_actual_b - self.fase_b
        } else {
            self.fase_b - fase_actual_b
        };
        diff_a > UMBRAL_MUTACION_DRASTICA || diff_b > UMBRAL_MUTACION_DRASTICA
    }

    /// Activa el enlace
    pub fn activar(&mut self) {
        self.activo = true;
    }

    /// Avanza un ciclo (incrementa contadores si está activo)
    pub fn avanzar_ciclo(&mut self) {
        if self.activo {
            self.ciclos_activo += 1;
        } else {
            self.ciclos_cerca += 1;
        }
    }

    /// Añade energía transferida
    pub fn agregar_transferencia(&mut self, cantidad: i64) {
        self.energia_transferida_total += cantidad;
    }

    /// Comprueba si este enlace conecta a los Auton dados
    pub fn conecta(&self, id_a: u64, id_b: u64) -> bool {
        (self.id_auton_a == id_a && self.id_auton_b == id_b)
            || (self.id_auton_a == id_b && self.id_auton_b == id_a)
    }
}

/// Estado del seguimiento de resonancia entre dos Auton
#[derive(Debug, Clone)]
pub struct ResonanciaSeguimiento {
    pub id_auton_a: u64,
    pub id_auton_b: u64,
    pub ciclos_cerca: u32,
    pub fase_a: HashEstado,
    pub fase_b: HashEstado,
    pub ultima_distancia: f64,
}

impl ResonanciaSeguimiento {
    pub fn nuevo(id_a: u64, id_b: u64, fase_a: HashEstado, fase_b: HashEstado) -> Self {
        ResonanciaSeguimiento {
            id_auton_a: id_a,
            id_auton_b: id_b,
            ciclos_cerca: 0,
            fase_a,
            fase_b,
            ultima_distancia: f64::MAX,
        }
    }

    /// Calcula diferencia de fase
    pub fn diferencia_fase(&self) -> u64 {
        if self.fase_a > self.fase_b {
            self.fase_a - self.fase_b
        } else {
            self.fase_b - self.fase_a
        }
    }

    /// Verifica si las fases están alineadas
    pub fn fases_alineadas(&self) -> bool {
        self.diferencia_fase() < UMBRAL_FASE
    }

    /// Actualiza el seguimiento para un ciclo
    pub fn actualizar(
        &mut self,
        cerca: bool,
        fase_a: HashEstado,
        fase_b: HashEstado,
        distancia: f64,
    ) {
        self.fase_a = fase_a;
        self.fase_b = fase_b;
        self.ultima_distancia = distancia;

        if cerca {
            self.ciclos_cerca += 1;
        } else {
            self.ciclos_cerca = 0;
        }
    }

    /// Comprueba si está listo para formar enlace
    pub fn listo_para_enlace(&self) -> bool {
        self.ciclos_cerca >= CICLOS_RESONANCIA && self.fases_alineadas()
    }
}

/// Gestor de enlaces pneumáticos para un Auton
#[derive(Debug, Clone)]
pub struct PneumaBonds {
    /// Enlaces activos
    pub enlaces: Vec<PneumaBond>,
    /// Historial de enlaces formados/rotos (para estadísticas)
    pub historial: VecDeque<PneumaBond>,
    /// Seguimientos de resonancia activos (no se serializa)
    pub seguimientos: Vec<ResonanciaSeguimiento>,
    /// Tick actual
    tick_actual: u64,
    /// Máximo de seguimientos a mantener
    max_seguimientos: usize,
    /// Máximo de historial a mantener
    max_historial: usize,
}

impl Default for PneumaBonds {
    fn default() -> Self {
        Self::nuevo()
    }
}

impl PneumaBonds {
    /// Crea un nuevo gestor de enlaces
    pub fn nuevo() -> Self {
        PneumaBonds {
            enlaces: Vec::new(),
            historial: VecDeque::with_capacity(100),
            seguimientos: Vec::with_capacity(50),
            tick_actual: 0,
            max_seguimientos: 50,
            max_historial: 100,
        }
    }

    /// Avanza el tick
    pub fn avanzar_tick(&mut self) {
        self.tick_actual += 1;
    }

    /// Obtiene el tick actual
    pub fn tick(&self) -> u64 {
        self.tick_actual
    }

    /// Obtiene número de enlaces activos
    pub fn num_enlaces_activos(&self) -> usize {
        self.enlaces.iter().filter(|e| e.activo).count()
    }

    /// Busca enlace entre dos Auton
    pub fn buscar_enlace(&self, id_a: u64, id_b: u64) -> Option<&PneumaBond> {
        self.enlaces
            .iter()
            .find(|e| e.conecta(id_a, id_b) && e.activo)
    }

    /// Busca enlace mutable entre dos Auton
    pub fn buscar_enlace_mut(&mut self, id_a: u64, id_b: u64) -> Option<&mut PneumaBond> {
        self.enlaces
            .iter_mut()
            .find(|e| e.conecta(id_a, id_b) && e.activo)
    }

    /// Obtiene todos los enlaces de un Auton
    pub fn enlaces_de(&self, id_auton: u64) -> Vec<&PneumaBond> {
        self.enlaces
            .iter()
            .filter(|e| e.activo && (e.id_auton_a == id_auton || e.id_auton_b == id_auton))
            .collect()
    }

    /// Procesa un encuentro entre dos Auton
    ///
    /// Retorna Some(event) si se forma un nuevo enlace
    pub fn procesar_encuentro(
        &mut self,
        id_a: u64,
        id_b: u64,
        fase_a: HashEstado,
        fase_b: HashEstado,
        distancia: f64,
        umbra_a: &Umbra,
        umbra_b: &Umbra,
    ) -> Option<String> {
        // Ignorar si ya tenemos enlace activo
        if self.buscar_enlace(id_a, id_b).is_some() {
            return None;
        }

        // Verificar si ya estamos rastreando esta resonancia
        if let Some(seg) = self.seguimientos.iter_mut().find(|s| {
            (s.id_auton_a == id_a && s.id_auton_b == id_b)
                || (s.id_auton_a == id_b && s.id_auton_b == id_a)
        }) {
            let cerca = distancia < RADIO_COMUNICACION;
            seg.actualizar(cerca, fase_a, fase_b, distancia);

            // Avanzar el seguimiento si están cerca
            if cerca {
                seg.ciclos_cerca += 1;
            }

            // Verificar si podemos formar enlace
            if seg.ciclos_cerca >= CICLOS_RESONANCIA && seg.fases_alineadas() {
                return Some(self.formar_enlace(id_a, id_b, fase_a, fase_b, umbra_a, umbra_b));
            }
        } else {
            // Crear nuevo seguimiento
            if self.seguimientos.len() >= self.max_seguimientos {
                self.seguimientos.remove(0);
            }
            self.seguimientos
                .push(ResonanciaSeguimiento::nuevo(id_a, id_b, fase_a, fase_b));
        }

        None
    }

    /// Forma un nuevo enlace
    fn formar_enlace(
        &mut self,
        id_a: u64,
        id_b: u64,
        fase_a: HashEstado,
        fase_b: HashEstado,
        _umbra_a: &Umbra,
        _umbra_b: &Umbra,
    ) -> String {
        let mut enlace = PneumaBond::nuevo(id_a, id_b, fase_a, fase_b, self.tick_actual);
        enlace.activar();

        self.enlaces.push(enlace.clone());

        // Limpiar seguimiento
        self.seguimientos.retain(|s| {
            !(s.id_auton_a == id_a && s.id_auton_b == id_b)
                && !(s.id_auton_a == id_b && s.id_auton_b == id_a)
        });

        format!(
            "{{\"tipo\":\"EnlaceFormado\",\"id1\":\"{:016x}\",\"id2\":\"{:016x}\",\"tick\":{}}}",
            id_a, id_b, self.tick_actual
        )
    }

    /// Procesa enlaces activos (transferencia de energía, etc)
    ///
    /// Retorna eventos de energía transferida
    pub fn procesar_enlaces_activos(
        &mut self,
        energia_a: i64,
        energia_b: i64,
        id_a: u64,
        id_b: u64,
    ) -> Option<String> {
        if let Some(enlace) = self.buscar_enlace_mut(id_a, id_b) {
            enlace.avanzar_ciclo();

            // Transferencia de energía: del que tiene más al que tiene menos
            if (energia_a > UMBRAL_ENERGIA_TRANSFERENCIA && energia_b < energia_a)
                || (energia_b > UMBRAL_ENERGIA_TRANSFERENCIA && energia_a < energia_b)
            {
                let transferencia = TASA_TRANSFERENCIA;
                let (de_quien, a_quien, monto) = if energia_a > energia_b {
                    (id_a, id_b, transferencia)
                } else {
                    (id_b, id_a, transferencia)
                };

                enlace.agregar_transferencia(monto);

                return Some(format!(
                    "{{\"tipo\":\"EnergiaTransferida\",\"de\":\"{:016x}\",\"a\":\"{:016x}\",\"monto\":{}}}",
                    de_quien, a_quien, monto
                ));
            }
        }
        None
    }

    /// Verifica y rompe enlaces que ya no cumplen criterios
    ///
    /// Retorna eventos de enlaces rotos
    pub fn verificar_rupturas(
        &mut self,
        id_auton: u64,
        fase_actual: HashEstado,
        otra_fase: HashEstado,
        distancia: f64,
    ) -> Vec<String> {
        let mut eventos = Vec::new();

        // Verificar enlaces de este Auton
        for enlace in self.enlaces.iter_mut() {
            if !enlace.activo {
                continue;
            }

            let (_id_otro, _fase_este, fase_otro) = if enlace.id_auton_a == id_auton {
                (enlace.id_auton_b, fase_actual, enlace.fase_b)
            } else if enlace.id_auton_b == id_auton {
                (enlace.id_auton_a, fase_actual, enlace.fase_a)
            } else {
                continue;
            };

            // Verificar distancia
            if distancia >= RADIO_COMUNICACION * 1.5 {
                enlace.activo = false;
                eventos.push(format!(
                    "{{\"tipo\":\"EnlaceRoto\",\"id1\":\"{:016x}\",\"id2\":\"{:016x}\",\"causa\":\"distancia\",\"tick\":{}}}",
                    id_auton, otra_fase, self.tick_actual
                ));
                continue;
            }

            // Verificar mutación drástica
            if enlace.mutacion_drastica(fase_actual, fase_otro) {
                enlace.activo = false;
                eventos.push(format!(
                    "{{\"tipo\":\"EnlaceRoto\",\"id1\":\"{:016x}\",\"id2\":\"{:016x}\",\"causa\":\"mutacion\",\"tick\":{}}}",
                    id_auton, otra_fase, self.tick_actual
                ));
            }
        }

        // Mover enlaces rotos al historial
        let mut indices_rotos = Vec::new();
        for (i, e) in self.enlaces.iter().enumerate() {
            if !e.activo {
                indices_rotos.push(i);
            }
        }
        // Remove in reverse order to preserve indices
        for i in indices_rotos.into_iter().rev() {
            if self.historial.len() >= self.max_historial {
                self.historial.pop_front();
            }
            self.historial.push_back(self.enlaces.remove(i));
        }

        eventos
    }

    /// Obtiene estadísticas de enlaces
    pub fn estadisticas(&self) -> PneumaBondsStats {
        PneumaBondsStats {
            num_enlaces_activos: self.num_enlaces_activos(),
            num_seguimientos: self.seguimientos.len(),
            num_historial: self.historial.len(),
            energia_total_transferida: self
                .enlaces
                .iter()
                .map(|e| e.energia_transferida_total)
                .sum(),
        }
    }
}

/// Estadísticas de PneumaBonds
#[derive(Debug, Clone)]
pub struct PneumaBondsStats {
    pub num_enlaces_activos: usize,
    pub num_seguimientos: usize,
    pub num_historial: usize,
    pub energia_total_transferida: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_formacion_enlace() {
        let mut bonds = PneumaBonds::nuevo();

        let id_a = 0xAAAA;
        let id_b = 0xBBBB;
        let fase_a = 100;
        let fase_b = 105; // Dentro del umbral (5 < 4096)

        let mut enlace_formado = false;

        // Simular encuentros cercanos durante CICLOS_RESONANCIA + 10 ciclos
        for _ in 0..CICLOS_RESONANCIA + 10 {
            bonds.avanzar_tick();
            let evento = bonds.procesar_encuentro(
                id_a,
                id_b,
                fase_a,
                fase_b,
                0.1,
                &Umbra::nuevo(id_a),
                &Umbra::nuevo(id_b),
            );

            if evento.is_some() && evento.as_ref().unwrap().contains("EnlaceFormado") {
                enlace_formado = true;
                break;
            }
        }

        assert!(
            enlace_formado,
            "Debería formarse enlace después de {} ciclos",
            CICLOS_RESONANCIA
        );
        assert_eq!(bonds.num_enlaces_activos(), 1);
    }

    #[test]
    fn test_fases_no_alineadas() {
        let mut bonds = PneumaBonds::nuevo();

        let id_a = 0xAAAA;
        let id_b = 0xBBBB;
        let fase_a = 100;
        let fase_b = 10000; // Muy lejos del umbral (9900 > 4096)

        // Intentar durante muchos ciclos
        for _ in 0..100 {
            bonds.avanzar_tick();
            let _evento = bonds.procesar_encuentro(
                id_a,
                id_b,
                fase_a,
                fase_b,
                0.1,
                &Umbra::nuevo(id_a),
                &Umbra::nuevo(id_b),
            );
        }

        // No debería formarse enlace
        assert_eq!(bonds.num_enlaces_activos(), 0);
    }

    #[test]
    fn test_transferencia_energia() {
        let mut bonds = PneumaBonds::nuevo();

        let id_a = 0xAAAA;
        let id_b = 0xBBBB;

        // Crear enlace directamente
        bonds
            .enlaces
            .push(PneumaBond::nuevo(id_a, id_b, 100, 100, 0));
        bonds.enlaces[0].activar();

        // Energia A > B
        let evento = bonds.procesar_enlaces_activos(
            0x00000050_00000000, // 80.0
            0x00000020_00000000, // 32.0
            id_a,
            id_b,
        );

        assert!(evento.is_some());
        let ev = evento.unwrap();
        assert!(ev.contains("EnergiaTransferida"));
    }

    #[test]
    fn test_ruptura_por_distancia() {
        let mut bonds = PneumaBonds::nuevo();

        let id_a = 0xAAAA;
        let id_b = 0xBBBB;

        // Crear enlace
        bonds
            .enlaces
            .push(PneumaBond::nuevo(id_a, id_b, 100, 100, 0));
        bonds.enlaces[0].activar();

        // Distancia muy grande - verificar que se rompe
        let eventos = bonds.verificar_rupturas(id_a, 100, 200, RADIO_COMUNICACION * 2.0);

        assert!(!eventos.is_empty());
        assert!(eventos[0].contains("distancia"));
        // El enlace fue removido del vector por verificar_rupturas
        assert_eq!(
            bonds.enlaces.len(),
            0,
            "El enlace debe ser removido tras romperse"
        );
    }

    #[test]
    fn test_ruptura_por_mutacion() {
        let mut bonds = PneumaBonds::nuevo();

        let id_a = 0xAAAA;
        let id_b = 0xBBBB;

        // Crear enlace
        bonds
            .enlaces
            .push(PneumaBond::nuevo(id_a, id_b, 100, 100, 0));
        bonds.enlaces[0].activar();

        // Mutación drástica: fase cambió de 100 a 200000
        let eventos = bonds.verificar_rupturas(id_a, 200000, 100, 0.1);

        assert!(!eventos.is_empty());
        assert!(eventos[0].contains("mutacion"));
    }
}
