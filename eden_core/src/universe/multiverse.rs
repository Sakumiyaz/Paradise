//! # Multiverse: Jardín de Universos
//!
//! EDEN puede gestionar múltiples instancias de `Universo`, cada una con su propia
//! semilla derivada. La selección natural opera a nivel cosmológico.
//!
//! ## Conceptos
//!
//! - **UniversoHilo**: Instancia de universo ejecutándose en un hilo separado
//! - **Aptitud Cósmica**: Métrica = diversidad_auton × energía_total × ciclos_sin_extinción
//! - **Fisión Cósmica**: Cuando Meltrace alcanza tamaño crítico, crear hijo con semilla mutada
//! - **Poda**: Terminar o pausar universos con baja aptitud
//! - **Puentes**: Migración rara de Auton entre universos
//!
//! ## Scheduling
//!
//! Los universos compiten por tiempo de CPU según su aptitud relativa.
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::JoinHandle;
use std::time::Duration;

/// Mensajes de control para un universo
#[derive(Debug, Clone)]
pub enum MensajeControl {
    /// Pausar el universo
    Pausar,
    /// Reanudar ejecución
    Reanudar,
    /// Terminar definitivamente
    Terminar,
    /// Migrar Auton a otro universo
    MigrarAuton {
        auton_id: u64,
        universo_destino: u64,
    },
}

/// Estado de un universo
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EstadoUniverso {
    /// Ejecutándose activamente
    Activo,
    /// Pausado (por baja aptitud)
    Pausado,
    /// En proceso de terminación
    Terminando,
    /// Extinguido
    Extinto,
}

/// Métricas de un universo
#[derive(Debug, Clone)]
pub struct MetricasUniverso {
    /// ID único del universo
    pub id: u64,
    /// Número de Auton activos
    pub num_auton: u32,
    /// Energía total del universo
    pub energia_total: i64,
    /// Ciclos desde el inicio
    pub ciclos: u64,
    /// Ciclos sin extinción (desde última muerte de Auton)
    pub ciclos_sin_extincion: u64,
    /// Diversidad de Auton (0.0 - 1.0)
    pub diversidad: f64,
    /// Tamaño del Meltrace
    pub tamano_meltrace: usize,
    /// Aptitud cósmica calculada
    pub aptitud: f64,
}

impl MetricasUniverso {
    /// Calcula la aptitud cósmica del universo
    /// Fórmula: diversidad × energía_normalizada × log(ciclos_sin_extincion + 1)
    pub fn calcular_aptitud(&self) -> f64 {
        let energia_norm = if self.energia_total > 0 {
            (self.energia_total as f64).ln().max(0.0) / 50.0 // Normalizado
        } else {
            0.0
        };

        let ciclos_factor = if self.ciclos_sin_extincion > 0 {
            (self.ciclos_sin_extincion as f64).ln().max(0.0) / 10.0
        } else {
            0.0
        };

        self.diversidad * energia_norm * ciclos_factor
    }
}

/// Un universo ejecutándose en un hilo
pub struct UniversoHilo {
    /// ID del universo
    pub id: u64,
    /// Hilo que ejecuta el universo
    pub handle: Option<JoinHandle<()>>,
    /// Canal de control
    pub canal_control: Sender<MensajeControl>,
    /// Estado actual
    pub estado: EstadoUniverso,
    /// Métricas actuales
    pub metricas: MetricasUniverso,
    /// Prioridad del hilo (0.0 - 1.0)
    prioridad: f64,
    /// Semilla del universo (para reproducibilidad)
    semilla: u64,
    /// Tick del último update
    pub ultimo_tick: u64,
}

impl UniversoHilo {
    /// Crea un nuevo universo hilo
    pub fn new(id: u64, semilla: u64) -> (Self, Receiver<MensajeControl>) {
        let (tx, rx) = channel();

        let universo = UniversoHilo {
            id,
            handle: None,
            canal_control: tx,
            estado: EstadoUniverso::Activo,
            metricas: MetricasUniverso {
                id,
                num_auton: 0,
                energia_total: 0,
                ciclos: 0,
                ciclos_sin_extincion: 0,
                diversidad: 0.0,
                tamano_meltrace: 0,
                aptitud: 0.0,
            },
            prioridad: 0.5,
            semilla,
            ultimo_tick: 0,
        };

        (universo, rx)
    }

    /// Actualiza las métricas del universo
    pub fn actualizar_metricas(&mut self, metricas: MetricasUniverso) {
        self.metricas = metricas;
        self.metricas.aptitud = self.metricas.calcular_aptitud();
        self.ultimo_tick = self.metricas.ciclos;
    }

    /// Establece la prioridad del hilo
    pub fn establecer_prioridad(&mut self, prioridad: f64) {
        self.prioridad = prioridad.clamp(0.0, 1.0);
    }
}

/// Administrador del multiverso
pub struct MultiverseManager {
    /// Universos activos
    universos: HashMap<u64, UniversoHilo>,
    /// Próximo ID de universo
    proximo_id: u64,
    /// Tick global del multiverso
    tick_global: u64,
    /// Umbral de Meltrace para fisión
    umbral_fision_meltrace: usize,
    /// Número mínimo de universos
    min_universos: u32,
    /// Número máximo de universos
    max_universos: u32,
    /// Frecuencia de evaluación de aptitud (ciclos)
    frecuencia_evaluacion: u64,
    /// tick de la última evaluación
    ultima_evaluacion: u64,
}

impl MultiverseManager {
    /// Crea nuevo administrador
    pub fn new() -> Self {
        MultiverseManager {
            universos: HashMap::new(),
            proximo_id: 1,
            tick_global: 0,
            umbral_fision_meltrace: 10_000,
            min_universos: 1,
            max_universos: 8,
            frecuencia_evaluacion: 10_000,
            ultima_evaluacion: 0,
        }
    }

    /// Crea el universo inicial
    pub fn crear_universo_inicial(&mut self, semilla: u64) -> u64 {
        let id = self.proximo_id;
        self.proximo_id += 1;

        let (mut universo, _rx) = UniversoHilo::new(id, semilla);
        universo.estado = EstadoUniverso::Activo;
        universo.metricas = MetricasUniverso {
            id,
            num_auton: 1,
            energia_total: 1_000_000_000_000i64 << 32,
            ciclos: 0,
            ciclos_sin_extincion: 0,
            diversidad: 0.1,
            tamano_meltrace: 0,
            aptitud: 0.0,
        };

        self.universos.insert(id, universo);
        id
    }

    /// Genera un universo hijo mediante fisión cósmica
    ///
    /// # Arguments
    /// * `universo_padre_id` - ID del universo padre
    /// * `semilla_mutada` - Semilla derivada del Meltrace del padre
    ///
    /// # Returns
    /// Some(id) del nuevo universo si se creó, None si no
    pub fn fission_cosmica(&mut self, universo_padre_id: u64, semilla_mutada: u64) -> Option<u64> {
        // Verificar que no excedemos el máximo
        if self.universos.len() >= self.max_universos as usize {
            return None;
        }

        // Verificar que el padre existe
        if !self.universos.contains_key(&universo_padre_id) {
            return None;
        }

        let id = self.proximo_id;
        self.proximo_id += 1;

        let (mut universo, _rx) = UniversoHilo::new(id, semilla_mutada);
        universo.estado = EstadoUniverso::Activo;
        universo.metricas = MetricasUniverso {
            id,
            num_auton: 0, // Inicia vacío, se puebla desde padre
            energia_total: 500_000_000_000i64 << 32, // Menos energía inicial
            ciclos: 0,
            ciclos_sin_extincion: 0,
            diversidad: 0.0,
            tamano_meltrace: 0,
            aptitud: 0.0,
        };

        self.universos.insert(id, universo);

        // Enviar mensaje al padre para migrar algunos Auton
        if let Some(padre) = self.universos.get_mut(&universo_padre_id) {
            let _ = padre.canal_control.send(MensajeControl::MigrarAuton {
                auton_id: 0, // El universo decidirá cuáles
                universo_destino: id,
            });
        }

        Some(id)
    }

    /// Evalúa la aptitud de todos los universos
    pub fn evaluar_aptitud(&mut self) {
        let _tick = self.tick_global;

        // Calcular aptitud total
        let aptitud_total: f64 = self.universos.values().map(|u| u.metricas.aptitud).sum();

        if aptitud_total <= 0.0 {
            return;
        }

        // Distribuir prioridad según aptitud relativa
        for universo in self.universos.values_mut() {
            let aptitud_relativa = universo.metricas.aptitud / aptitud_total;
            universo.establecer_prioridad(aptitud_relativa);
        }
    }

    /// Ejecuta poda de universos con baja aptitud
    ///
    /// # Returns
    /// Número de universos podados
    pub fn podar_universos(&mut self) -> usize {
        if self.universos.len() <= self.min_universos as usize {
            return 0;
        }

        let aptitud_promedio: f64 = if !self.universos.is_empty() {
            self.universos
                .values()
                .map(|u| u.metricas.aptitud)
                .sum::<f64>()
                / self.universos.len() as f64
        } else {
            0.0
        };

        let mut podados = 0;

        // Encontrar universos para podar
        let ids_a_podar: Vec<u64> = self
            .universos
            .iter()
            .filter(|(_, u)| {
                u.estado == EstadoUniverso::Activo
                && u.metricas.aptitud < aptitud_promedio * 0.3 // 30% del promedio
                && self.universos.len() > self.min_universos as usize
            })
            .map(|(id, _)| *id)
            .collect();

        for id in ids_a_podar {
            if let Some(universo) = self.universos.get_mut(&id) {
                universo.estado = EstadoUniverso::Terminando;
                let _ = universo.canal_control.send(MensajeControl::Terminar);
                podados += 1;
            }
        }

        podados
    }

    /// Verifica si algún universo requiere fisión
    ///
    /// # Returns
    /// Some((universo_padre_id, semilla_mutada)) si se requiere fisión, None si no
    pub fn verificar_fision(&self) -> Option<(u64, u64)> {
        for universo in self.universos.values() {
            if universo.metricas.tamano_meltrace >= self.umbral_fision_meltrace {
                // Generar semilla mutada basada en el hash del Meltrace
                let semilla_mutada = universo
                    .semilla
                    .wrapping_mul(31)
                    .wrapping_add(universo.metricas.tamano_meltrace as u64);
                return Some((universo.id, semilla_mutada));
            }
        }
        None
    }

    /// Migra un Auton entre universos
    ///
    /// # Arguments
    /// * `auton_id` - ID del Auton a migrar
    /// * `origen_id` - ID del universo de origen
    /// * `destino_id` - ID del universo de destino
    ///
    /// # Returns
    /// true si la migración fue exitosa
    pub fn migrar_auton(&mut self, auton_id: u64, origen_id: u64, destino_id: u64) -> bool {
        // Verificar que ambos universos existen
        if !self.universos.contains_key(&origen_id) || !self.universos.contains_key(&destino_id) {
            return false;
        }

        // Enviar mensaje de migración al origen
        if let Some(origen) = self.universos.get_mut(&origen_id) {
            let result = origen.canal_control.send(MensajeControl::MigrarAuton {
                auton_id,
                universo_destino: destino_id,
            });
            return result.is_ok();
        }

        false
    }

    /// Pausa un universo
    pub fn pausar_universo(&mut self, id: u64) -> bool {
        if let Some(universo) = self.universos.get_mut(&id) {
            universo.estado = EstadoUniverso::Pausado;
            return universo.canal_control.send(MensajeControl::Pausar).is_ok();
        }
        false
    }

    /// Reanuda un universo
    pub fn reanudar_universo(&mut self, id: u64) -> bool {
        if let Some(universo) = self.universos.get_mut(&id) {
            universo.estado = EstadoUniverso::Activo;
            return universo
                .canal_control
                .send(MensajeControl::Reanudar)
                .is_ok();
        }
        false
    }

    /// Termina un universo definitivamente
    pub fn terminar_universo(&mut self, id: u64) -> bool {
        if let Some(universo) = self.universos.get_mut(&id) {
            universo.estado = EstadoUniverso::Terminando;
            return universo
                .canal_control
                .send(MensajeControl::Terminar)
                .is_ok();
        }
        false
    }

    /// Actualiza métricas de un universo
    pub fn actualizar_metricas(&mut self, id: u64, metricas: MetricasUniverso) {
        if let Some(universo) = self.universos.get_mut(&id) {
            universo.actualizar_metricas(metricas);
        }
    }

    /// Obtiene el universo con mayor aptitud
    pub fn universo_max_aptitud(&self) -> Option<&UniversoHilo> {
        self.universos.values().max_by(|a, b| {
            a.metricas
                .aptitud
                .partial_cmp(&b.metricas.aptitud)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    /// Obtiene todos los universos ordenados por aptitud
    pub fn universos_por_aptitud(&self) -> Vec<&UniversoHilo> {
        let mut universos: Vec<&UniversoHilo> = self.universos.values().collect();
        universos.sort_by(|a, b| {
            b.metricas
                .aptitud
                .partial_cmp(&a.metricas.aptitud)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        universos
    }

    /// Número de universos activos
    pub fn num_universos_activos(&self) -> usize {
        self.universos
            .values()
            .filter(|u| u.estado == EstadoUniverso::Activo)
            .count()
    }

    /// Número total de universos
    pub fn num_universos(&self) -> usize {
        self.universos.len()
    }

    /// Obtiene métricas consolidadas del multiverso
    pub fn metricas_consolidadas(&self) -> MetricasConsolidadas {
        let total_auton: u32 = self.universos.values().map(|u| u.metricas.num_auton).sum();

        let energia_total: i64 = self
            .universos
            .values()
            .map(|u| u.metricas.energia_total)
            .sum();

        let ciclos_max = self
            .universos
            .values()
            .map(|u| u.metricas.ciclos)
            .max()
            .unwrap_or(0);

        MetricasConsolidadas {
            num_universos: self.universos.len() as u32,
            universos_activos: self.num_universos_activos() as u32,
            total_auton,
            energia_total,
            ciclos_max,
        }
    }

    /// Avanza el tick global
    pub fn avanzar_tick(&mut self) {
        self.tick_global += 1;

        // Verificar si es hora de evaluar
        if self.tick_global - self.ultima_evaluacion >= self.frecuencia_evaluacion {
            self.evaluar_aptitud();
            self.podar_universos();
            self.ultima_evaluacion = self.tick_global;

            // Verificar necesidad de fisión
            if let Some((padre_id, semilla)) = self.verificar_fision() {
                let _ = self.fission_cosmica(padre_id, semilla);
            }
        }
    }
}

/// Métricas consolidadas de todo el multiverso
#[derive(Debug, Clone)]
pub struct MetricasConsolidadas {
    /// Número de universos totales
    pub num_universos: u32,
    /// Número de universos activos
    pub universos_activos: u32,
    /// Total de Auton en todos los universos
    pub total_auton: u32,
    /// Energía total del multiverso
    pub energia_total: i64,
    /// Máximo ciclos de cualquier universo
    pub ciclos_max: u64,
}

/// Scheduler de CPU para el multiverso
pub struct CosmicScheduler {
    /// Quantum de tiempo por universo (en ms)
    quantum_ms: u64,
}

impl CosmicScheduler {
    /// Crea nuevo scheduler
    pub fn new(quantum_ms: u64) -> Self {
        CosmicScheduler { quantum_ms }
    }

    /// Calcula el quantum de tiempo para un universo según su prioridad
    pub fn calcular_quantum(&self, prioridad: f64) -> Duration {
        let quantum = (self.quantum_ms as f64 * prioridad.max(0.1)) as u64;
        Duration::from_millis(quantum.max(10)) // Mínimo 10ms
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crear_universo_inicial() {
        let mut manager = MultiverseManager::new();
        let id = manager.crear_universo_inicial(0xDEADBEEF);

        assert_eq!(id, 1);
        assert_eq!(manager.num_universos(), 1);
        assert_eq!(manager.num_universos_activos(), 1);
    }

    #[test]
    fn test_fission_cosmica() {
        let mut manager = MultiverseManager::new();
        let _id_padre = manager.crear_universo_inicial(0xDEADBEEF);

        // Simular Meltrace grande
        manager
            .universos
            .get_mut(&1)
            .unwrap()
            .metricas
            .tamano_meltrace = 15_000;

        let semilla_mutada = 12345u64;
        let id_hijo = manager.fission_cosmica(1, semilla_mutada);

        assert!(id_hijo.is_some());
        assert_eq!(manager.num_universos(), 2);
    }

    #[test]
    fn test_poda_universos() {
        let mut manager = MultiverseManager::new();
        let _id1 = manager.crear_universo_inicial(0xAAAA);
        let _id2 = manager.crear_universo_inicial(0xBBBB);

        // Asignar aptitud muy diferente
        manager.universos.get_mut(&1).unwrap().metricas.aptitud = 100.0;
        manager.universos.get_mut(&2).unwrap().metricas.aptitud = 0.01;

        manager.min_universos = 1;
        let podados = manager.podar_universos();

        assert!(podados >= 1);
        assert_eq!(
            manager.universos.get(&1).unwrap().estado,
            EstadoUniverso::Activo
        );
        assert_eq!(
            manager.universos.get(&2).unwrap().estado,
            EstadoUniverso::Terminando
        );
    }

    #[test]
    fn test_metricas_consolidadas() {
        let mut manager = MultiverseManager::new();
        let _id1 = manager.crear_universo_inicial(0xAAAA);
        let _id2 = manager.crear_universo_inicial(0xBBBB);

        manager.universos.get_mut(&1).unwrap().metricas.num_auton = 10;
        manager.universos.get_mut(&2).unwrap().metricas.num_auton = 5;

        let metricas = manager.metricas_consolidadas();

        assert_eq!(metricas.num_universos, 2);
        assert_eq!(metricas.total_auton, 15);
    }

    #[test]
    fn test_universo_max_aptitud() {
        let mut manager = MultiverseManager::new();
        let _id1 = manager.crear_universo_inicial(0xAAAA);
        let _id2 = manager.crear_universo_inicial(0xBBBB);

        manager.universos.get_mut(&1).unwrap().metricas.aptitud = 50.0;
        manager.universos.get_mut(&2).unwrap().metricas.aptitud = 100.0;

        let max = manager.universo_max_aptitud();
        assert!(max.is_some());
        assert_eq!(max.unwrap().id, 2);
    }

    #[test]
    fn test_prioridades_segun_aptitud() {
        let mut manager = MultiverseManager::new();
        let _id1 = manager.crear_universo_inicial(0xAAAA);
        let _id2 = manager.crear_universo_inicial(0xBBBB);

        manager.universos.get_mut(&1).unwrap().metricas.aptitud = 100.0;
        manager.universos.get_mut(&2).unwrap().metricas.aptitud = 50.0;

        manager.evaluar_aptitud();

        // El universo con más aptitud debería tener más prioridad
        let u1 = manager.universos.get(&1).unwrap();
        let u2 = manager.universos.get(&2).unwrap();

        assert!(u1.prioridad > u2.prioridad);
    }

    #[test]
    fn test_calcular_aptitud() {
        // Usar un valor que no cause overflow en la-shift
        // 1_000_000 << 32 = 4.29e15, que es < i64::MAX
        let metricas = MetricasUniverso {
            id: 1,
            num_auton: 10,
            energia_total: 1_000_000i64 << 32,
            ciclos: 1000,
            ciclos_sin_extincion: 500,
            diversidad: 0.8,
            tamano_meltrace: 5000,
            aptitud: 0.0,
        };

        let aptitud = metricas.calcular_aptitud();

        // La aptitud debe ser positiva para datos válidos
        assert!(aptitud > 0.0, "Aptitud {} debería ser > 0", aptitud);
    }

    #[test]
    fn test_scheduler_quantum() {
        let scheduler = CosmicScheduler::new(100); // 100ms quantum

        let d100 = scheduler.calcular_quantum(1.0);
        assert_eq!(d100, Duration::from_millis(100));

        let d50 = scheduler.calcular_quantum(0.5);
        assert_eq!(d50, Duration::from_millis(50));

        let d1 = scheduler.calcular_quantum(0.01);
        assert_eq!(d1, Duration::from_millis(10)); // Mínimo 10ms
    }

    #[test]
    fn test_verificar_fision() {
        let mut manager = MultiverseManager::new();
        let _id1 = manager.crear_universo_inicial(0xAAAA);

        // Meltrace bajo - no debe haber fisión
        manager
            .universos
            .get_mut(&1)
            .unwrap()
            .metricas
            .tamano_meltrace = 100;
        assert!(manager.verificar_fision().is_none());

        // Meltrace alto - debe haber fisión
        manager
            .universos
            .get_mut(&1)
            .unwrap()
            .metricas
            .tamano_meltrace = 15_000;
        let resultado = manager.verificar_fision();
        assert!(resultado.is_some());

        let (padre_id, semilla) = resultado.unwrap();
        assert_eq!(padre_id, 1);
        assert_ne!(semilla, 0xAAAA); // Debe estar mutada
    }
}
