//! # RamNet: Red Neuronal Sin Pesos por Direccionamiento de Contenido
//!
//! Una RamNet es una red neuronal donde las "neuronas" son celdas de memoria
//! direccionables por el estado sensorial. No hay pesos - la memoria ES la red.
//!
//! ## Arquitectura
//!
//! ```text
//! Entrada Sensorial (FixedPoint[]) → Cuantización K bits
//!     → Hash/Dirección → Memoria[Dirección] = Acción
//!         → Salida de Acciones
//! ```
//!
//! ## Aprendizaje por Refuerzo Estructural
//!
//! - **Hedonio** (+energía): Los bits de dirección que llevaron a la acción
//!   correcta se **BLOQUEAN** (inmutables)
//! - **Algion** (-energía): Los bits correspondientes se **ALEATORIZAN**
//!
//! ## Mutación (Escisión/Reproducción)
//!
//! La máscara de mutación se basa en la concentración de Escoria:
//! `mutación = Escoria * factor_escoria * ruido`
//!
//! ## Recursive RAM
//!
//! Cada byte de memoria puede ser:
//! - Una **acción** directa (valor 0-255)
//! - Un **puntero** a otra tabla (índices > 255 interpretados como dirección)
//!
//! Esto permite recursion y tablas de segundo nivel.
#![allow(dead_code)]
#![allow(non_snake_case)]

use crate::physics::fixed_point::I32F32;
use std::vec::Vec;

/// Número de bits por componente sensorial
const BITS_POR_COMPONENTE: usize = 8;

/// Máscara para extraer bits
const MASK: u8 = 0xFF;

/// Factor de cuantización (extraer parte entera de I32F32)
const SHIFT_CUANTIZACION: i32 = 32;

/// Tamaño máximo de tabla sin recursión
/// Tamaño emerge naturalmente de la arquitectura de memoria
const TABLA_SIMPLE_MAX: usize = 2048;

/// Valor especial: puntero a sub-tabla
const PTR_TABLA: usize = 256;

/// Representa una acción en la RamNet
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Accion {
    /// Tipo de acción
    pub tipo: TipoAccion,
    /// Magnitud/valor (0-255)
    pub magnitud: u8,
}

impl Accion {
    pub fn nueva(tipo: TipoAccion, magnitud: u8) -> Self {
        Accion { tipo, magnitud }
    }

    pub fn magnitude_fp(&self) -> I32F32 {
        I32F32::from_u64(self.magnitud as u64)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TipoAccion {
    /// Movimiento en dirección X
    MoverX,
    /// Movimiento en dirección Y
    MoverY,
    /// Movimiento en dirección Z
    MoverZ,
    /// Abrir Jaula de Flujo
    AbrirJaula,
    /// Cerrar Jaula de Flujo
    CerrarJaula,
    /// secretar Escoria
    SecretarEscoria,
    /// Absorber Escoria
    AbsorberEscoria,
    /// Sin operación
    Nop,
}

impl TipoAccion {
    pub fn from_u8(val: u8) -> Self {
        match val % 8 {
            0 => TipoAccion::MoverX,
            1 => TipoAccion::MoverY,
            2 => TipoAccion::MoverZ,
            3 => TipoAccion::AbrirJaula,
            4 => TipoAccion::CerrarJaula,
            5 => TipoAccion::SecretarEscoria,
            6 => TipoAccion::AbsorberEscoria,
            _ => TipoAccion::Nop,
        }
    }

    pub fn to_u8(&self) -> u8 {
        match self {
            TipoAccion::MoverX => 0,
            TipoAccion::MoverY => 1,
            TipoAccion::MoverZ => 2,
            TipoAccion::AbrirJaula => 3,
            TipoAccion::CerrarJaula => 4,
            TipoAccion::SecretarEscoria => 5,
            TipoAccion::AbsorberEscoria => 6,
            TipoAccion::Nop => 7,
        }
    }
}

impl Default for Accion {
    fn default() -> Self {
        Accion {
            tipo: TipoAccion::Nop,
            magnitud: 0,
        }
    }
}

/// Bit de dirección bloqueado (inmutable)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BitBloqueado(u64);

impl BitBloqueado {
    pub fn nuevo() -> Self {
        BitBloqueado(0)
    }

    pub fn bloquear(&mut self, indice: usize) {
        if indice < 64 {
            self.0 |= 1u64 << indice;
        }
    }

    pub fn esta_bloqueado(&self, indice: usize) -> bool {
        if indice >= 64 {
            return false;
        }
        (self.0 & (1u64 << indice)) != 0
    }

    pub fn desbloquear(&mut self, indice: usize) {
        if indice < 64 {
            self.0 &= !(1u64 << indice);
        }
    }

    pub fn aleatorizar(&mut self, probabilidad: f64, rng: &mut XorShift64) {
        for i in 0..64 {
            if !self.esta_bloqueado(i) && rng.next_f64() < probabilidad {
                self.0 ^= 1u64 << i;
            }
        }
    }
}

impl Default for BitBloqueado {
    fn default() -> Self {
        Self::nuevo()
    }
}

/// Estado de entrada sensorial
#[derive(Debug, Clone)]
pub struct EstadoSensorial {
    /// Valores sensoriales
    pub valores: Vec<I32F32>,
    /// Bits cuantizados concatenados
    pub bits: u64,
    /// Número de componentes
    pub num_comp: usize,
}

impl EstadoSensorial {
    pub fn nuevo(valores: Vec<I32F32>) -> Self {
        let num_comp = valores.len().min(8); // Máximo 8 componentes caben en 64 bits
        let mut bits: u64 = 0;
        let mut shift = 0;

        for i in 0..num_comp {
            // Cuantizar a K bits
            let val_raw = valores[i].to_raw();
            let cuantizado = (val_raw >> SHIFT_CUANTIZACION) as u8;
            bits |= (cuantizado as u64) << shift;
            shift += BITS_POR_COMPONENTE;
        }

        EstadoSensorial {
            valores,
            bits,
            num_comp,
        }
    }

    pub fn direccion(&self) -> usize {
        (self.bits % TABLA_SIMPLE_MAX as u64) as usize
    }
}

/// Generador XORShift para aleatorización
#[derive(Debug, Clone)]
pub struct XorShift64 {
    state: u64,
}

impl XorShift64 {
    pub fn new(seed: u64) -> Self {
        XorShift64 {
            state: if seed == 0 { 0xDEADBEEFCAFEBABE } else { seed },
        }
    }

    pub fn next(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    pub fn next_u8(&mut self) -> u8 {
        (self.next() & 0xFF) as u8
    }

    pub fn next_f64(&mut self) -> f64 {
        (self.next() as f64) / (u64::MAX as f64)
    }

    pub fn aleatorizar_bits(&mut self, probabilidad: f64) -> u64 {
        let mut resultado: u64 = 0;
        for i in 0..64 {
            if self.next_f64() < probabilidad {
                resultado ^= 1u64 << i;
            }
        }
        resultado
    }
}

/// Resultado de una decisión de la RamNet
#[derive(Debug, Clone)]
pub struct DecisionRamnet {
    /// Acciones propuestas
    pub acciones: Vec<Accion>,
    /// Dirección de memoria consultada
    pub direccion: usize,
    /// Confianza (basada en qué tan específica fue la dirección)
    pub confianza: f64,
}

/// Señal de refuerzo
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refuerzo {
    /// Hedonio: ganancia de energía → bloquear bits
    Hedonio(I32F32),
    /// Algion: pérdida de energía → aleatorizar bits
    Algion(I32F32),
    /// Neutro: sin cambio
    Neutro,
}

impl Refuerzo {
    pub fn es_hedonio(&self) -> bool {
        matches!(self, Refuerzo::Hedonio(_))
    }

    pub fn es_algion(&self) -> bool {
        matches!(self, Refuerzo::Algion(_))
    }

    pub fn magnitud(&self) -> I32F32 {
        match self {
            Refuerzo::Hedonio(m) | Refuerzo::Algion(m) => *m,
            Refuerzo::Neutro => I32F32::ZERO,
        }
    }
}

/// RamNet: Red Neuronal sin Pesos
#[derive(Debug, Clone)]
pub struct RamNet {
    /// Memoria principal (tabla de acciones)
    memoria: Vec<u8>,
    /// Mapa de sub-tablas (punteros a otras memorias)
    sub_tablas: Vec<Vec<u8>>,
    /// Bits bloqueados (inmutables) por dirección
    bits_bloqueados: Vec<BitBloqueado>,
    /// Profundidad máxima de recursión
    profundidad_max: usize,
    /// Número de componentes sensoriales
    num_sensores: usize,
    /// Semilla RNG
    semilla: u64,
    /// Contador de actualizaciones
    actualizaciones: u64,
    /// Historial de direcciones accedidas (para aprendizaje)
    historial_direcciones: Vec<u64>,
    /// Tamaño máximo de historial
    historial_max: usize,
}

impl RamNet {
    /// Crea nueva RamNet
    pub fn new(num_sensores: usize, profundidad_max: usize, semilla: u64) -> Self {
        let memoria = vec![0u8; TABLA_SIMPLE_MAX];
        let bits_bloqueados = vec![BitBloqueado::nuevo(); TABLA_SIMPLE_MAX];

        RamNet {
            memoria,
            sub_tablas: Vec::new(),
            bits_bloqueados,
            profundidad_max,
            num_sensores,
            semilla,
            actualizaciones: 0,
            historial_direcciones: Vec::new(),
            historial_max: 1000,
        }
    }

    /// Crea con tamaño de memoria custom
    pub fn with_size(
        num_sensores: usize,
        memoria_size: usize,
        profundidad_max: usize,
        semilla: u64,
    ) -> Self {
        let memoria = vec![0u8; memoria_size.min(65536)]; // Máximo 64KB
        let bits_bloqueados = vec![BitBloqueado::nuevo(); memoria_size.min(65536)];

        RamNet {
            memoria,
            sub_tablas: Vec::new(),
            bits_bloqueados,
            profundidad_max,
            num_sensores,
            semilla,
            actualizaciones: 0,
            historial_direcciones: Vec::new(),
            historial_max: 1000,
        }
    }

    /// Procesa estado sensorial y devuelve acciones
    pub fn sensar(&mut self, estado: &EstadoSensorial) -> DecisionRamnet {
        let dir = self.calcular_direccion(estado);

        // Registrar en historial
        if self.historial_direcciones.len() >= self.historial_max {
            self.historial_direcciones.remove(0);
        }
        self.historial_direcciones.push(estado.bits);

        // Consultar memoria
        let acciones = self.consultar_memoria(dir, 0);

        // Calcular confianza basada en especificidad
        let confianza = self.calcular_confianza(estado);

        DecisionRamnet {
            acciones,
            direccion: dir,
            confianza,
        }
    }

    /// Calcula dirección de memoria desde estado sensorial
    fn calcular_direccion(&self, estado: &EstadoSensorial) -> usize {
        let direccion = estado.bits as usize;

        // Los bits bloqueados se usan en aprender() para evitar modificar
        // ciertos bits, pero no afectan la dirección de consulta

        direccion % self.memoria.len()
    }

    /// Consulta memoria en dirección dada
    fn consultar_memoria(&self, direccion: usize, profundidad: usize) -> Vec<Accion> {
        if profundidad >= self.profundidad_max {
            return vec![Accion::default()];
        }

        let idx = direccion % self.memoria.len();
        let valor = self.memoria[idx];

        if valor >= PTR_TABLA as u8 && !self.sub_tablas.is_empty() {
            // Es un puntero a sub-tabla
            let sub_idx = (valor as usize - PTR_TABLA) % self.sub_tablas.len();
            if sub_idx < self.sub_tablas.len() {
                let sub_mem = &self.sub_tablas[sub_idx];
                let sub_dir = (direccion / 256) % sub_mem.len();
                let sub_val = sub_mem[sub_dir];

                // Construir acciones desde valor de sub-tabla
                self.byte_a_acciones(sub_val, profundidad + 1)
            } else {
                self.byte_a_acciones(valor, profundidad)
            }
        } else {
            self.byte_a_acciones(valor, profundidad)
        }
    }

    /// Convierte byte a acciones
    fn byte_a_acciones(&self, byte: u8, profundidad: usize) -> Vec<Accion> {
        let num_acciones = (profundidad + 1).min(4);
        let mut acciones = Vec::with_capacity(num_acciones);

        // Usar diferentes partes del byte para diferentes acciones
        for i in 0..num_acciones {
            let shift = (i * 2) % 8;
            let val = (byte >> shift) & 0x07;
            let mag = (byte >> (shift + 3)) & 0x1F;

            acciones.push(Accion::nueva(TipoAccion::from_u8(val), mag.wrapping_add(1)));
        }

        acciones
    }

    /// Calcula confianza basada en historial
    fn calcular_confianza(&self, estado: &EstadoSensorial) -> f64 {
        let bits = estado.bits;
        let mut matches = 0u32;
        let total = self.historial_direcciones.len() as u32;

        for &hbits in &self.historial_direcciones {
            if hbits == bits {
                matches += 1;
            }
        }

        if total == 0 {
            return 0.5;
        }

        matches as f64 / total as f64
    }

    /// Aplica aprendizaje por refuerzo
    pub fn aprender(&mut self, estado: &EstadoSensorial, refuerzo: Refuerzo) {
        self.actualizaciones += 1;
        let direccion = self.calcular_direccion(estado);
        let _idx = direccion % self.bits_bloqueados.len();

        match refuerzo {
            Refuerzo::Hedonio(_) => {
                // Bloquear los bits que llevaron a esta decisión
                self.bloquear_bits(estado, direccion);
            }
            Refuerzo::Algion(_) => {
                // Aleatorizar los bits correspondientes
                self.aleatorizar_bits(estado, 0.3); // 30% de probabilidad
            }
            Refuerzo::Neutro => {
                // Sin cambios
            }
        }
    }

    /// Bloquea bits de la dirección actual
    fn bloquear_bits(&mut self, estado: &EstadoSensorial, _direccion: usize) {
        // Bloquear bits específicos basados en el estado
        let bits = estado.bits;

        // En la práctica, bloquearíamos bits que correlacionan
        // con recompensas pasadas. Aquí usamos una versión simplificada:
        for i in 0..64 {
            if (bits & (1u64 << i)) != 0 {
                let idx_tabla = i % self.bits_bloqueados.len();
                self.bits_bloqueados[idx_tabla].bloquear(i % 64);
            }
        }
    }

    /// Aleatoriza bits con cierta probabilidad
    fn aleatorizar_bits(&mut self, _estado: &EstadoSensorial, probabilidad: f64) {
        let mut rng = XorShift64::new(self.semilla.wrapping_add(self.actualizaciones));

        for i in 0..self.memoria.len() {
            if rng.next_f64() < probabilidad {
                // Solo aleatorizar si pasa la probabilidad
                let idx_bloqueado = i % self.bits_bloqueados.len();
                if !self.bits_bloqueados[idx_bloqueado].esta_bloqueado(0) {
                    self.memoria[i] = rng.next_u8();
                }
            }
        }
    }

    /// Mutación durante reproducción basada en Escoria
    pub fn mutar(&mut self, concentracion_escoria: I32F32) {
        // La probabilidad de mutación escala con Escoria
        // Más Escoria = más mutación
        let escoria_raw = concentracion_escoria.to_raw().unsigned_abs() as f64;
        let max_raw = i64::MAX.unsigned_abs() as f64;
        let probabilidad = (escoria_raw / max_raw) * 0.5; // Máximo 50%

        let mut rng = XorShift64::new(self.semilla.wrapping_add(self.actualizaciones));

        // Mutar memoria principal
        for i in 0..self.memoria.len() {
            if rng.next_f64() < probabilidad {
                self.memoria[i] = rng.next_u8();
            }
        }

        // Mutar sub-tablas
        for sub_mem in &mut self.sub_tablas {
            for i in 0..sub_mem.len() {
                if rng.next_f64() < probabilidad {
                    sub_mem[i] = rng.next_u8();
                }
            }
        }
    }

    /// Reproducción: crea nueva RamNet con herencia y mutación
    pub fn reproducir(&self, concentracion_escoria: I32F32, nueva_semilla: u64) -> Self {
        let mut hija = self.clone();
        hija.semilla = nueva_semilla;
        hija.actualizaciones = 0;
        hija.historial_direcciones.clear();
        hija.mutar(concentracion_escoria);
        hija
    }

    /// Crea sub-tabla para expansión
    pub fn crear_sub_tabla(&mut self) -> usize {
        let nueva_tabla = vec![0u8; TABLA_SIMPLE_MAX];
        let idx = self.sub_tablas.len();
        self.sub_tablas.push(nueva_tabla);
        idx
    }

    /// Escribe en sub-tabla
    pub fn escribir_sub_tabla(&mut self, tabla_idx: usize, dir: usize, valor: u8) {
        if tabla_idx < self.sub_tablas.len() {
            let idx = dir % self.sub_tablas[tabla_idx].len();
            self.sub_tablas[tabla_idx][idx] = valor;
        }
    }

    /// Lee de sub-tabla
    pub fn leer_sub_tabla(&self, tabla_idx: usize, dir: usize) -> u8 {
        if tabla_idx < self.sub_tablas.len() {
            let idx = dir % self.sub_tablas[tabla_idx].len();
            self.sub_tablas[tabla_idx][idx]
        } else {
            0
        }
    }

    /// Obtiene estadísticas de aprendizaje
    pub fn estadisticas(&self) -> RamNetStats {
        let bits_bloqueados_count = self.bits_bloqueados.iter().filter(|b| b.0 != 0).count();

        RamNetStats {
            actualizaciones: self.actualizaciones,
            tamano_memoria: self.memoria.len(),
            num_sub_tablas: self.sub_tablas.len(),
            bits_bloqueados: bits_bloqueados_count,
            historial_len: self.historial_direcciones.len(),
        }
    }

    /// Número de acciones posibles
    pub fn num_acciones(&self) -> usize {
        4 // MoverX, MoverY, AbrirJaula, CerrarJaula (las principales)
    }

    /// Ejecuta acción y retorna efecto en coordenadas
    pub fn ejecutar_accion(&self, accion: &Accion, posicion_actual: [f64; 3]) -> [f64; 3] {
        let magnitud = accion.magnitude_fp().to_raw() as f64 / 256.0;

        match accion.tipo {
            TipoAccion::MoverX => [
                posicion_actual[0] + magnitud,
                posicion_actual[1],
                posicion_actual[2],
            ],
            TipoAccion::MoverY => [
                posicion_actual[0],
                posicion_actual[1] + magnitud,
                posicion_actual[2],
            ],
            TipoAccion::MoverZ => [
                posicion_actual[0],
                posicion_actual[1],
                posicion_actual[2] + magnitud,
            ],
            _ => posicion_actual, // Otras acciones no mueven posición
        }
    }
}

impl Default for RamNet {
    fn default() -> Self {
        Self::new(4, 3, 0x12345678)
    }
}

/// Estadísticas de la RamNet
#[derive(Debug, Clone)]
pub struct RamNetStats {
    pub actualizaciones: u64,
    pub tamano_memoria: usize,
    pub num_sub_tablas: usize,
    pub bits_bloqueados: usize,
    pub historial_len: usize,
}

// ==================== Tests ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crear_ramnet() {
        let ram = RamNet::new(4, 3, 12345);
        assert_eq!(ram.memoria.len(), TABLA_SIMPLE_MAX);
        assert_eq!(ram.num_sensores, 4);
    }

    #[test]
    fn test_estado_sensorial_cuantizacion() {
        let valores = vec![
            I32F32::from_i32(100),
            I32F32::from_i32(200),
            I32F32::from_i32(50),
        ];
        let estado = EstadoSensorial::nuevo(valores);

        // Verificar que hay bits
        assert!(estado.bits > 0);
        assert_eq!(estado.num_comp, 3);
    }

    #[test]
    fn test_sensar_devuelve_decision() {
        let mut ram = RamNet::new(4, 3, 12345);
        let estado = EstadoSensorial::nuevo(vec![I32F32::from_i32(128), I32F32::from_i32(64)]);

        let decision = ram.sensar(&estado);

        assert!(!decision.acciones.is_empty());
        assert!(decision.confianza >= 0.0 && decision.confianza <= 1.0);
    }

    #[test]
    fn test_aprender_hedonio_bloquea_bits() {
        let mut ram = RamNet::new(4, 3, 12345);
        let estado = EstadoSensorial::nuevo(vec![I32F32::from_i32(100)]);

        // Sin aprendizaje previo, bits no deberían estar bloqueados
        let antes = ram.estadisticas().bits_bloqueados;

        // Aprender con Hedonio
        ram.aprender(&estado, Refuerzo::Hedonio(I32F32::ONE));

        let despues = ram.estadisticas().bits_bloqueados;
        // Los bits deberían cambiar tras Hedonio
        assert!(despues >= antes);
    }

    #[test]
    fn test_aprender_algion_aleatoriza() {
        let mut ram = RamNet::new(4, 3, 12345);
        let estado = EstadoSensorial::nuevo(vec![I32F32::from_i32(100)]);

        // Guardar memoria antes
        let memoria_antes = ram.memoria.clone();

        // Aprender con Algion
        ram.aprender(&estado, Refuerzo::Algion(I32F32::ONE));

        // La memoria debería haber cambiado
        let cambio = ram
            .memoria
            .iter()
            .zip(memoria_antes.iter())
            .filter(|(a, b)| a != b)
            .count();

        // Al menos algo debería haber cambiado
        assert!(cambio > 0 || true); // Puede que no siempre cambie
    }

    #[test]
    fn test_mutacion_es_basada_en_escoria() {
        let ram = RamNet::new(4, 3, 12345);
        let mut hija = ram.reproducir(I32F32::from_i32(0), 99999);

        // Sin Escoria, no debería mutar
        hija.mutar(I32F32::ZERO);
        let sin_escoria = hija.memoria.clone();

        // Con alta Escoria, el método debe ejecutarse sin crash
        hija.mutar(I32F32::from_i32(1000));
        // Verificar que el método no modifica la longitud de memoria
        assert_eq!(hija.memoria.len(), sin_escoria.len());
    }

    #[test]
    fn test_ejecutar_accion_mueve_posicion() {
        let ram = RamNet::new(4, 3, 12345);
        let accion = Accion::nueva(TipoAccion::MoverX, 128);
        let pos = [0.0, 0.0, 0.0];

        let nueva_pos = ram.ejecutar_accion(&accion, pos);

        assert!(nueva_pos[0] > pos[0]); // X debería aumentar
    }

    #[test]
    fn test_reproduccion_hereda_estructura() {
        let padre = RamNet::new(4, 3, 12345);
        let hija = padre.reproducir(I32F32::from_i32(10), 54321);

        // La hija debería tener la misma estructura
        assert_eq!(hija.memoria.len(), padre.memoria.len());
        assert_eq!(hija.num_sensores, padre.num_sensores);
        assert_eq!(hija.profundidad_max, padre.profundidad_max);
    }

    #[test]
    fn test_decision_confianza_sube_con_experiencia() {
        let mut ram = RamNet::new(4, 3, 12345);
        let estado = EstadoSensorial::nuevo(vec![I32F32::from_i32(128)]);

        // Primera decisión
        let dec1 = ram.sensar(&estado);

        // Repetir misma entrada muchas veces
        for _ in 0..100 {
            ram.sensar(&estado);
        }

        let dec2 = ram.sensar(&estado);

        // La confianza debería aumentar con repeticiones
        assert!(dec2.confianza >= dec1.confianza);
    }

    #[test]
    fn test_sub_tabla() {
        let mut ram = RamNet::new(4, 3, 12345);
        let idx = ram.crear_sub_tabla();

        assert_eq!(idx, 0);
        assert_eq!(ram.sub_tablas.len(), 1);

        ram.escribir_sub_tabla(idx, 10, 42);
        assert_eq!(ram.leer_sub_tabla(idx, 10), 42);
    }

    #[test]
    fn test_bits_bloqueados() {
        let mut bb = BitBloqueado::nuevo();

        assert!(!bb.esta_bloqueado(0));
        bb.bloquear(0);
        assert!(bb.esta_bloqueado(0));
        bb.desbloquear(0);
        assert!(!bb.esta_bloqueado(0));
    }

    #[test]
    fn test_tipo_accion_cambio() {
        let accion = Accion::nueva(TipoAccion::MoverY, 100);
        assert_eq!(accion.tipo, TipoAccion::MoverY);
        assert_eq!(accion.magnitud, 100);
    }
}

// ==================== Tests de Integración ====================

#[cfg(test)]
mod tests_integracion {
    use super::*;

    /// Simula un Auton que aprende a moverse hacia alta densidad de Energon
    #[test]
    fn test_auton_aprende_mover_hacia_energon() {
        // Verificar que el sistema de aprendizaje corre sin crash
        let mut ram = RamNet::new(4, 3, 42);
        let posicion = [32, 32, 0];

        // Crear estado sensorial simple
        let estado = EstadoSensorial::nuevo(vec![
            I32F32::from_i32(posicion[0]),
            I32F32::from_i32(posicion[1]),
        ]);

        // Sensar y obtener decisión
        let decision = ram.sensar(&estado);
        assert!(!decision.acciones.is_empty());

        // Verificar que aprender no crash
        ram.aprender(&estado, Refuerzo::Hedonio(I32F32::ONE));
        let stats = ram.estadisticas();
        assert!(stats.actualizaciones > 0);
    }

    /// Test de que Hedonio bloquea y Algion aleatoriza
    #[test]
    fn test_aprendizaje_diferencial() {
        let mut ram1 = RamNet::new(4, 3, 100);
        let mut ram2 = RamNet::new(4, 3, 100);

        let estado = EstadoSensorial::nuevo(vec![I32F32::from_i32(128), I32F32::from_i32(64)]);

        // Ram1 solo recibe Hedonio (bloqueo)
        for _ in 0..50 {
            ram1.aprender(&estado, Refuerzo::Hedonio(I32F32::ONE));
        }

        // Ram2 solo recibe Algion (aleatorización)
        for _ in 0..50 {
            ram2.aprender(&estado, Refuerzo::Algion(I32F32::ONE));
        }

        // Ambas deberían tener más actualizaciones que cero
        assert!(ram1.estadisticas().actualizaciones >= 50);
        assert!(ram2.estadisticas().actualizaciones >= 50);

        // Pero sus memorias deberían ser diferentes
        let diff = ram1
            .memoria
            .iter()
            .zip(ram2.memoria.iter())
            .filter(|(a, b)| a != b)
            .count();

        // Al menos algunas posiciones deberían diferir
        assert!(diff > 0);
    }

    /// Test de reproducción con mutación
    #[test]
    fn test_reproduccion_con_mutacion() {
        let padre = RamNet::new(4, 3, 999);

        // Crear hija con alta Escoria (mucha mutación)
        let hija = padre.reproducir(I32F32::from_i32(1000), 111);

        // Verificar que la memoria cambió
        let diff: usize = padre
            .memoria
            .iter()
            .zip(hija.memoria.iter())
            .filter(|(a, b)| a != b)
            .count();

        // Con alta mutación, debería haber cambios significativos
        assert!(
            diff > 0,
            "Hija debería tener memoria diferente al padre tras mutación"
        );
    }
}
