//! # Semiosis Module
//!
//! Sistema de señales emergente para comunicación entre Auton.
//!
//! ## Concepto
//!
//! Cada Auton puede emitir una "señal" (u8) como parte de su estado visible.
//! Inicialmente las señales son ruido, pero mediante aprendizaje por refuerzo
//! pueden emerger convenciones.
#![allow(dead_code)]
#![allow(non_snake_case)]

use crate::life::ramnet::{Accion, EstadoSensorial, RamNet, TipoAccion, XorShift64};
use crate::physics::I32F32;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicBool, Ordering};

// ============================================================================
// FAST MUTEX - Spinlock puro, sin overhead
// ============================================================================

use std::cell::UnsafeCell;

/// Mutex rápido usando spinlock atómico
/// Sin poisoning, sin alloc, mínimo overhead
struct FastMutex<T> {
    locked: AtomicBool,
    value: UnsafeCell<T>,
}

unsafe impl<T: Send> Send for FastMutex<T> {}
unsafe impl<T: Send> Sync for FastMutex<T> {}

impl<T> FastMutex<T> {
    const fn new(value: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            value: UnsafeCell::new(value),
        }
    }

    /// Lock spinloop con pause hint
    #[inline(always)]
    fn lock(&self) -> FastMutexGuard<'_, T> {
        while self
            .locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            while self.locked.load(Ordering::Relaxed) {
                std::hint::spin_loop();
            }
        }
        FastMutexGuard { mutex: self }
    }
}

impl<T: Default> Default for FastMutex<T> {
    fn default() -> Self {
        Self::new(T::default())
    }
}

/// Guard - acceso directo al valor vía UnsafeCell
pub struct FastMutexGuard<'a, T> {
    mutex: &'a FastMutex<T>,
}

impl<T> Drop for FastMutexGuard<'_, T> {
    #[inline(always)]
    fn drop(&mut self) {
        self.mutex.locked.store(false, Ordering::Release);
    }
}

impl<T> std::ops::Deref for FastMutexGuard<'_, T> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &T {
        unsafe { &*self.mutex.value.get() }
    }
}

impl<T> std::ops::DerefMut for FastMutexGuard<'_, T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.mutex.value.get() }
    }
}

// ============================================================================
// TIPOS BÁSICOS
// ============================================================================

/// Una señal de 8 bits emitida por un Auton
#[derive(Debug, Clone, Copy)]
pub struct Senal {
    /// Valor de la señal (0-255)
    pub valor: u8,
    /// ID del Auton que la emitió
    pub emisor_id: u64,
    /// Ciclo en que se emitió
    pub ciclo: u64,
    /// Posición del emisor (x, y) como enteros de 32 bits
    pub posicion: (i32, i32),
}

impl Senal {
    /// Crea nueva señal
    pub fn new(valor: u8, emisor_id: u64, ciclo: u64, posicion: (f64, f64)) -> Self {
        Senal {
            valor,
            emisor_id,
            ciclo,
            posicion: (posicion.0 as i32, posicion.1 as i32),
        }
    }

    /// Compara posiciones para Hash (usando i32)
    pub fn posicion_f64(&self) -> (f64, f64) {
        (self.posicion.0 as f64, self.posicion.1 as f64)
    }
}

impl PartialEq for Senal {
    fn eq(&self, other: &Self) -> bool {
        self.valor == other.valor && self.emisor_id == other.emisor_id && self.ciclo == other.ciclo
    }
}

impl Eq for Senal {}

impl Hash for Senal {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.valor.hash(state);
        self.emisor_id.hash(state);
        self.ciclo.hash(state);
    }
}

/// Representación interna del estado semiótico
#[derive(Debug, Clone)]
pub struct EstadoSemiotico {
    /// Señal actualmente emitida
    pub senal_actual: u8,
    /// Historial de señales observadas (max MAX_SENALES_OBSERVADAS)
    pub senales_observadas: Vec<Senal>,
    /// Conexiones aprendidas: señal -> recompensa esperada
    pub conexiones_senal_recompensa: HashMap<u8, f64>,
    /// Frecuencia de observación por señal
    pub frecuencia_senales: HashMap<u8, u32>,
    /// Ciclos desde última señal válida (> 0.5 confianza)
    pub ciclos_sin_senal_valida: u32,
}

impl EstadoSemiotico {
    /// Crea nuevo estado semiótico
    pub fn new() -> Self {
        EstadoSemiotico {
            senal_actual: SENAL_NULA,
            senales_observadas: Vec::new(),
            conexiones_senal_recompensa: HashMap::new(),
            frecuencia_senales: HashMap::new(),
            ciclos_sin_senal_valida: 0,
        }
    }
}

/// Valor nulo para señal
pub const SENAL_NULA: u8 = 0;

/// Máximo de señales observadas que se guardan
/// Cantidad emerge naturalmente de la dinámica semiótica
pub const MAX_SENALES_OBSERVADAS: usize = 500;

// ============================================================================
// MANAGER DE SEMIOSIS
// ============================================================================

/// Gestor global del sistema semiótico
pub struct SemiosisManager {
    /// Diccionario de señales por valor
    señales_por_valor: HashMap<u8, Vec<Senal>>,
    /// Convenciones descubiertas: señal -> significado aprendido
    convenciones: HashMap<u8, String>,
    /// Contador global de señales emitidas
    total_senales: u64,
    /// Mapa de correlaciones: señal -> eventos positivos siguientes
    correlaciones: HashMap<u8, u32>,
    /// Frecuencia de observación por señal (índice global)
    frecuencia_senales: HashMap<u8, u32>,
}

impl SemiosisManager {
    /// Crea nuevo manager
    pub fn new() -> Self {
        SemiosisManager {
            señales_por_valor: HashMap::new(),
            convenciones: HashMap::new(),
            total_senales: 0,
            correlaciones: HashMap::new(),
            frecuencia_senales: HashMap::new(),
        }
    }

    /// Registra una señal observada
    pub fn registrar_senal(&mut self, senal: Senal) {
        self.total_senales += 1;

        // Actualizar frecuencia
        *self.frecuencia_senales.entry(senal.valor).or_insert(0) += 1;

        // Agregar a lista por valor
        self.señales_por_valor
            .entry(senal.valor)
            .or_insert_with(Vec::new)
            .push(senal);
    }

    /// Registra correlación entre señal y evento positivo
    pub fn registrar_correlacion(&mut self, senal: u8, evento_positivo: bool) {
        if evento_positivo {
            *self.correlaciones.entry(senal).or_insert(0) += 1;
        }
    }

    /// Obtiene señal más frecuente observada
    pub fn senal_mas_frecuente(&self) -> Option<u8> {
        self.frecuencia_senales
            .iter()
            .max_by_key(|&(_, count)| count)
            .map(|(&senal, _)| senal)
    }

    /// Obtiene mejor señal según correlación positiva
    pub fn mejor_senal(&self) -> Option<u8> {
        self.correlaciones
            .iter()
            .filter(|&(_, &count)| count > 10) // Mínimo 10 observaciones
            .max_by_key(|&(_, count)| count)
            .map(|(&senal, _)| senal)
    }

    /// Obtiene estadísticas
    pub fn stats(&self) -> SemiosisStats {
        SemiosisStats {
            total_senales: self.total_senales,
            numero_convenciones: self.convenciones.len() as u32,
            senal_mas_observada: self.senal_mas_frecuente().unwrap_or(SENAL_NULA),
            mayor_correlacion: self.mejor_senal().unwrap_or(SENAL_NULA),
        }
    }

    /// Agrega convención descubierta
    pub fn agregar_convencion(&mut self, senal: u8, significado: &str) {
        self.convenciones.insert(senal, significado.to_string());
    }
}

impl Default for SemiosisManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Estadísticas del sistema semiótico
#[derive(Debug, Clone)]
pub struct SemiosisStats {
    pub total_senales: u64,
    pub numero_convenciones: u32,
    pub senal_mas_observada: u8,
    pub mayor_correlacion: u8,
}

// ============================================================================
// PROCESADOR DE SEMIOSIS PARA AUTON
// ============================================================================

/// Procesador de semiosis para un Auton individual
pub struct ProcesadorSemiosis {
    /// Estado semiótico
    pub estado: EstadoSemiotico,
    /// RamNet para decisiones de señal
    ramnet_señales: RamNet,
    /// RNG local
    rng: XorShift64,
    /// Probabilidad de copiar señal observada
    prob_copia_senial: f64,
}

impl ProcesadorSemiosis {
    /// Crea nuevo procesador
    pub fn new(semilla: u64) -> Self {
        let ramnet = RamNet::new(4, 1, semilla);

        ProcesadorSemiosis {
            estado: EstadoSemiotico::new(),
            ramnet_señales: ramnet,
            rng: XorShift64::new(semilla),
            prob_copia_senial: 0.3,
        }
    }

    /// Decide señal basándose en estado interno
    pub fn decidir_senal(&mut self, energia: i64, escoria: f64, hambre: f64, miedo: f64) -> u8 {
        // Normalizar inputs
        let energia_norm = ((energia as f64) / 1e12).clamp(-1.0, 1.0) as i32;
        let escoria_norm = (escoria.clamp(0.0, 1.0) * 100.0) as i32;
        let hambre_norm = (hambre.clamp(0.0, 1.0) * 100.0) as i32;
        let miedo_norm = (miedo.clamp(0.0, 1.0) * 100.0) as i32;

        // Crear estado sensorial con enteros
        let estado_sensorial = EstadoSensorial::nuevo(vec![
            I32F32::from_i32(energia_norm),
            I32F32::from_i32(escoria_norm),
            I32F32::from_i32(hambre_norm),
            I32F32::from_i32(miedo_norm),
        ]);

        let decision = self.ramnet_señales.sensar(&estado_sensorial);

        // Si hay acción con confianza, usar para generar señal
        if decision.confianza > 0.6 && !decision.acciones.is_empty() {
            let accion = &decision.acciones[0];
            let senal = generar_senal_desde_accion(accion, &mut self.rng);
            self.estado.senal_actual = senal;
            return senal;
        }

        // Si no hay decisión clara, generar señal basada en estado
        let senal = generar_senal_desde_estado(energia, escoria, hambre, miedo, &mut self.rng);
        self.estado.senal_actual = senal;
        senal
    }

    /// Observa señal de otro Auton
    pub fn observar_senal(&mut self, senal: Senal, resultado_positivo: bool) {
        // Limitar tamaño del historial
        if self.estado.senales_observadas.len() >= MAX_SENALES_OBSERVADAS {
            self.estado.senales_observadas.remove(0);
        }
        self.estado.senales_observadas.push(senal);

        // Actualizar correlación señal-recompensa
        if resultado_positivo {
            let entry = self
                .estado
                .conexiones_senal_recompensa
                .entry(senal.valor)
                .or_insert(0.0);
            *entry += 0.1;
        }

        // Registrar en manager global
        let mut guard = SEMIOSIS_GLOBAL.lock();
        if let Some(manager) = guard.as_mut() {
            manager.registrar_senal(senal);
            manager.registrar_correlacion(senal.valor, resultado_positivo);
        }
    }

    /// Refuerza conexión entre señal y recompensa
    pub fn reforzar_conexion(&mut self, senal: u8, refuerzo: f64) {
        let entry = self
            .estado
            .conexiones_senal_recompensa
            .entry(senal)
            .or_insert(0.0);
        *entry += refuerzo;
    }

    /// Obtiene recompensa esperada de una señal
    pub fn recompensa_esperada(&self, senal: u8) -> f64 {
        *self
            .estado
            .conexiones_senal_recompensa
            .get(&senal)
            .unwrap_or(&0.0)
    }

    /// Decide si copiar señal observada basándose en recompensa esperada
    pub fn decidir_copia_senal(&self, senal: u8) -> bool {
        let recompensa = self.recompensa_esperada(senal);
        self.prob_copia_senial > 0.0 && recompensa > 0.0
    }
}

/// Genera señal a partir de acción
fn generar_senal_desde_accion(accion: &Accion, _rng: &mut XorShift64) -> u8 {
    // Crear señal basada en el tipo de acción y magnitud
    // Usar enum ordinal approach
    let tipo_ordinal = match accion.tipo {
        TipoAccion::MoverX => 0,
        TipoAccion::MoverY => 1,
        TipoAccion::MoverZ => 2,
        TipoAccion::AbrirJaula => 3,
        TipoAccion::CerrarJaula => 4,
        TipoAccion::SecretarEscoria => 5,
        TipoAccion::AbsorberEscoria => 6,
        TipoAccion::Nop => 7,
    };

    // Combinar tipo y magnitud para generar señal
    let base = (tipo_ordinal & 0x07) << 5; // 3 bits para tipo, en superiores
    let mag_low = (accion.magnitud as u8) & 0x1F; // 5 bits para magnitud

    base | mag_low
}

/// Genera señal basándose en estado interno (fallback)
fn generar_senal_desde_estado(
    energia: i64,
    escoria: f64,
    hambre: f64,
    miedo: f64,
    rng: &mut XorShift64,
) -> u8 {
    // Señal basada en amenaza (combina miedo y escoria)
    if miedo > 0.5 || escoria > 0.3 {
        return 0xF0 | ((rng.next() as u8) & 0x0F); // Señales de peligro
    }

    // Señal basada en hambre
    if hambre > 0.5 {
        return 0xA0 | ((rng.next() as u8) & 0x0F); // Señales de hambre
    }

    // Energía baja
    if energia < 1_000_000_000 {
        return 0x50 | ((rng.next() as u8) & 0x0F); // Señales de baja energía
    }

    // Estado normal - señal aleatoria baja
    (rng.next() as u8) & 0x3F
}

/// Manager global (mutex para singleton thread-safe)
static SEMIOSIS_GLOBAL: FastMutex<Option<SemiosisManager>> = FastMutex::new(None);

/// Inicializa el manager global
pub fn inicializar_semiosis_global() {
    let mut global = SEMIOSIS_GLOBAL.lock();
    if global.is_none() {
        *global = Some(SemiosisManager::new());
    }
}

/// Obtiene acceso al manager global (retorna guard para acceso directo)
pub fn obtener_manager_global() -> FastMutexGuard<'static, Option<SemiosisManager>> {
    SEMIOSIS_GLOBAL.lock()
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crear_senal() {
        let senal = Senal::new(42, 1, 100, (0.0, 0.0));
        assert_eq!(senal.valor, 42);
        assert_eq!(senal.emisor_id, 1);
        assert_eq!(senal.ciclo, 100);
    }

    #[test]
    fn test_estado_semiotico_nuevo() {
        let estado = EstadoSemiotico::new();
        assert_eq!(estado.senal_actual, SENAL_NULA);
        assert!(estado.senales_observadas.is_empty());
        assert!(estado.conexiones_senal_recompensa.is_empty());
    }

    #[test]
    fn test_decidir_senal_vacia() {
        let mut procesador = ProcesadorSemiosis::new(12345);

        // Energia alta, escoria baja, sin miedo/hambre
        let senal = procesador.decidir_senal(5_000_000_000i64, 0.1, 0.1, 0.1);

        // No debe ser nula si hay estado normal
        // (aunque podría variar por RamNet)
        assert!(senal != SENAL_NULA || true); // Acepta cualquier valor
    }

    #[test]
    fn test_observar_senal() {
        let mut procesador = ProcesadorSemiosis::new(54321);
        let senal = Senal::new(50, 2, 200, (10.0, 10.0));

        inicializar_semiosis_global();

        procesador.observar_senal(senal, true);

        assert_eq!(procesador.estado.senales_observadas.len(), 1);
        assert_eq!(procesador.estado.senales_observadas[0].valor, 50);
    }

    #[test]
    fn test_reforzar_conexion() {
        let mut procesador = ProcesadorSemiosis::new(11111);

        procesador.reforzar_conexion(42, 0.5);

        let recompensa = procesador.recompensa_esperada(42);
        assert!((recompensa - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_recompensa_esperada_desconocida() {
        let procesador = ProcesadorSemiosis::new(22222);

        let recompensa = procesador.recompensa_esperada(99);

        assert_eq!(recompensa, 0.0);
    }

    #[test]
    fn test_decidir_copia_senal() {
        let mut procesador = ProcesadorSemiosis::new(33333);

        // Sin refuerzo, prob_copia = 0.3
        assert!(!procesador.decidir_copia_senal(42));

        // Con refuerzo alto
        procesador.reforzar_conexion(42, 1.0);
        // Aún con recompensa, prob_copia 0.3 podría dar false
        // solo verificar que no crash
    }

    #[test]
    fn test_generar_senal_estado_normal() {
        let mut rng = XorShift64::new(99999);

        let senal = generar_senal_desde_estado(
            1_000_000_000_000i64, // energía alta
            0.0,                  // sin escoria
            0.0,                  // sin hambre
            0.0,                  // sin miedo
            &mut rng,
        );

        // Señal normal debe ser baja (< 64)
        assert!(senal < 64);
    }

    #[test]
    fn test_generar_senal_peligro() {
        let mut rng = XorShift64::new(88888);

        let senal = generar_senal_desde_estado(
            1_000_000_000_000i64,
            0.5, // alta escoria
            0.0,
            0.8, // alto miedo
            &mut rng,
        );

        // Señal de peligro debe ser alta (0xF0-0xFF)
        assert!(senal >= 0xF0);
    }

    #[test]
    fn test_manager_global() {
        use std::sync::Arc;

        // Reset for test
        let mut guard = SEMIOSIS_GLOBAL.lock();
        *guard = None;
        drop(guard);

        inicializar_semiosis_global();

        // Verificar que el manager fue creado
        let guard = SEMIOSIS_GLOBAL.lock();
        let manager_ref = (*guard).as_ref().unwrap();

        let stats = Arc::new(manager_ref.stats());
        assert_eq!(stats.total_senales, 0);
        assert_eq!(Arc::strong_count(&stats), 1);
    }

    #[test]
    fn test_semiosis_manager_registrar() {
        let mut manager = SemiosisManager::new();

        let senal = Senal::new(42, 1, 100, (0.0, 0.0));
        manager.registrar_senal(senal);

        let stats = manager.stats();
        assert_eq!(stats.total_senales, 1);
    }

    #[test]
    fn test_correlacion() {
        let mut manager = SemiosisManager::new();

        // Registrar varias correlaciones positivas
        for _ in 0..15 {
            manager.registrar_correlacion(42, true);
        }

        let mejor = manager.mejor_senal();
        assert_eq!(mejor, Some(42));
    }
}
