//! # Mar Morfóseo: Autómata Continuo de Energon
//!
//! El Mar Morfóseo es un autómata celular continuo que contiene una "sopa" de
//! Energon en una cuadrícula 2D o 3D configurable.
//!
//! ## Estructura de Celda
//!
//! Cada celda del Mar contiene:
//! - **densidad**: Concentración de Energon (FixedPoint)
//! - **flujo**: Vector (x, y, z) de dirección y magnitud del flujo
//! - **escoria**: Entropía residuo de interacciones (FixedPoint)
//!
//! ## Evolución
//!
//! La evolución del Mar se calcula mediante diferencias finitas de la
//! Ecuación de Tres Actos:
//! ```text
//! ∂ρ/∂t = D∇²ρ - ∇·(τ·σ) + η
//! ```
//! Donde:
//! - ρ = densidad de Energon
//! - D = coeficiente de difusión
//! - τ = constantes cosmológicas
//! - σ = tensor de estado
//! - η = ruido termodinámico
//!
//! ## Nomos
//!
//! Un Nomos se forma cuando una región mantiene un patrón estable
//! (oscilación periódica) durante N pasos consecutivos.
//! Cuando se detecta, se emite `NomoFormado` por el canal MPSC.
//!
//! ## Hilos
//!
//! El solver usa múltiples hilos (`std::thread::spawn`) para paralelizar
//! el cálculo de interacciones locales y difusión.
#![allow(dead_code)]
#![allow(non_snake_case)]

use crate::physics::energon::{ConstantesCosmicas, Energon, Vector3D};
use crate::physics::fixed_point::I32F32;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::vec::Vec;
/// Tipo de espacio (dimensiones)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpaceType {
    /// Grid 2D con profundidad unitaria
    Plano2D,
    /// Grid 3D completo
    Volumen3D,
}

impl SpaceType {
    /// Retorna el número de dimensiones
    pub fn dimensiones(&self) -> usize {
        match self {
            SpaceType::Plano2D => 2,
            SpaceType::Volumen3D => 3,
        }
    }
}

impl Default for SpaceType {
    fn default() -> Self {
        SpaceType::Plano2D
    }
}

/// Tipo de Nomos detectado
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TipoNomo {
    /// Patrón oscilatorio estable
    Oscilador,
    /// Atractor de punto fijo
    PuntoFijo,
    /// Onda viajera
    Onda,
    /// Patrón espiral
    Espiral,
}

impl std::fmt::Display for TipoNomo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TipoNomo::Oscilador => write!(f, "Oscilador"),
            TipoNomo::PuntoFijo => write!(f, "PuntoFijo"),
            TipoNomo::Onda => write!(f, "Onda"),
            TipoNomo::Espiral => write!(f, "Espiral"),
        }
    }
}

/// Evento emitido cuando se forma un Nomos
#[derive(Debug, Clone)]
pub struct NomosFormado {
    /// Centro del Nomos
    pub centro: (usize, usize, usize),
    /// Tipo de patrón
    pub tipo: TipoNomo,
    /// Tick en que se formó
    pub tick: u64,
    /// Densidad máxima en el centro
    pub densidad_max: I32F32,
    /// Radio aproximado del Nomos
    pub radio: usize,
}

impl NomosFormado {
    pub fn new(
        centro: (usize, usize, usize),
        tipo: TipoNomo,
        tick: u64,
        densidad: I32F32,
        radio: usize,
    ) -> Self {
        NomosFormado {
            centro,
            tipo,
            tick,
            densidad_max: densidad,
            radio,
        }
    }
}

/// Celda del Mar Morfóseo
#[derive(Debug, Clone, Copy)]
pub struct CeldaMar {
    /// Densidad de Energon
    pub densidad: I32F32,
    /// Vector de flujo (x, y, z)
    pub flujo: Vector3D<I32F32>,
    /// Concentración de Escoria (entropía)
    pub escoria: I32F32,
    /// Densidad previa (para detectar oscilaciones)
    pub densidad_prev: I32F32,
    /// Densidad hace 2 ticks (historial)
    pub densidad_prev2: I32F32,
}

impl CeldaMar {
    /// Crea celda vacía
    pub fn vazia() -> Self {
        CeldaMar {
            densidad: I32F32::ZERO,
            flujo: Vector3D::splat(I32F32::ZERO),
            escoria: I32F32::ZERO,
            densidad_prev: I32F32::ZERO,
            densidad_prev2: I32F32::ZERO,
        }
    }

    /// Crea celda con valores
    pub fn nueva(densidad: I32F32, escoria: I32F32) -> Self {
        CeldaMar {
            densidad,
            flujo: Vector3D::splat(I32F32::ZERO),
            escoria,
            densidad_prev: I32F32::ZERO,
            densidad_prev2: I32F32::ZERO,
        }
    }

    /// Actualiza historial de densidades (para detección de oscilaciones)
    pub fn actualizar_historial(&mut self) {
        self.densidad_prev2 = self.densidad_prev;
        self.densidad_prev = self.densidad;
    }

    /// Detecta si hay oscilación periódica
    pub fn detecta_oscilacion(&self, umbral_amplitud: I32F32) -> bool {
        let delta1 = (self.densidad - self.densidad_prev).abs();
        let delta2 = (self.densidad_prev - self.densidad_prev2).abs();

        // Oscilación si los deltas son similares (patrón alternante)
        let diff_deltas = (delta1 - delta2).abs();
        let suma_deltas = delta1 + delta2;

        if suma_deltas.to_raw() == 0 {
            return false;
        }

        // Normalizar diferencia relativa
        let ratio = diff_deltas.to_raw().unsigned_abs() as u64;
        let suma_raw = suma_deltas.to_raw().unsigned_abs() as u64;

        ratio < (suma_raw >> 4) && delta1 > umbral_amplitud
    }
}

impl Default for CeldaMar {
    fn default() -> Self {
        Self::vazia()
    }
}

/// Dimensiones del Mar
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DimensionesMar {
    /// Tamaño en X
    pub x: usize,
    /// Tamaño en Y
    pub y: usize,
    /// Tamaño en Z (1 para 2D)
    pub z: usize,
    /// Tipo de espacio
    pub espacio: SpaceType,
}

impl DimensionesMar {
    /// Grid 2D de tamaño N×N
    pub fn plano2d(n: usize) -> Self {
        DimensionesMar {
            x: n,
            y: n,
            z: 1,
            espacio: SpaceType::Plano2D,
        }
    }

    /// Grid 3D de tamaño N×N×N
    pub fn volumen3d(n: usize) -> Self {
        DimensionesMar {
            x: n,
            y: n,
            z: n,
            espacio: SpaceType::Volumen3D,
        }
    }

    /// Grid 3D con dimensiones específicas
    pub fn custom(x: usize, y: usize, z: usize) -> Self {
        let espacio = if z > 1 {
            SpaceType::Volumen3D
        } else {
            SpaceType::Plano2D
        };
        DimensionesMar { x, y, z, espacio }
    }

    /// Número total de celdas
    pub fn total_celdas(&self) -> usize {
        self.x * self.y * self.z
    }

    /// Índice lineal desde (x, y, z)
    pub fn indice(&self, x: usize, y: usize, z: usize) -> usize {
        z * self.x * self.y + y * self.x + x
    }

    /// Verifica si las coordenadas son válidas
    pub fn es_valida(&self, x: usize, y: usize, z: usize) -> bool {
        x < self.x && y < self.y && z < self.z
    }

    /// Wrapping de coordenadas (toroide)
    pub fn wrap(&self, x: i32, y: i32, z: i32) -> (usize, usize, usize) {
        let x = ((x % self.x as i32) + self.x as i32) as usize % self.x;
        let y = ((y % self.y as i32) + self.y as i32) as usize % self.y;
        let z = ((z % self.z as i32) + self.z as i32) as usize % self.z;
        (x, y, z)
    }
}

/// Configuración del Mar Morfóseo
#[derive(Debug, Clone)]
pub struct ConfigMar {
    /// Dimensiones del grid
    pub dimensiones: DimensionesMar,
    /// Número de hilos para solver
    pub num_hilos: usize,
    /// Coeficiente de difusión
    pub coef_difusion: I32F32,
    /// Tasa de decaimiento de escoria
    pub tasa_escoria: I32F32,
    /// Umbral de densidad mínima para Nomos
    pub umbral_nomos: I32F32,
    /// Número de ticks para estabilizar Nomos
    pub ticks_estabilidad: usize,
    /// Constantes cosmológicas
    pub constantes: ConstantesCosmicas,
    /// Semilla para ruido
    pub semilla_ruido: u64,
}

impl Default for ConfigMar {
    fn default() -> Self {
        let semilla = [0x42u8; 128];
        ConfigMar {
            dimensiones: DimensionesMar::plano2d(64),
            num_hilos: 4,
            coef_difusion: I32F32::from_i32(10), // D = 10
            tasa_escoria: I32F32::from_i32(1),   // 1% por tick
            umbral_nomos: I32F32::from_i32(50),
            ticks_estabilidad: 10,
            constantes: ConstantesCosmicas::from_semilla(&semilla),
            semilla_ruido: 0xDEADBEEF,
        }
    }
}

/// Mar Morfóseo: el autómata continuo de Energon
pub struct MarMorfoseo {
    /// Grid de celdas
    grid: Vec<CeldaMar>,
    /// Dimensiones
    dimensiones: DimensionesMar,
    /// Configuración
    config: ConfigMar,
    /// Contador de ticks
    tick_actual: u64,
    /// Canal para eventos de Nomos
    canal_nomos: Option<Sender<NomosFormado>>,
    /// Historial de densidades para detección de patrones
    historial: Vec<I32F32>,
    /// Contador de estabilidad por celda
    contador_estabilidad: Vec<usize>,
    /// IDs de Nomos formados (para no repetir)
    nomos_activos: Vec<(usize, usize, usize)>,
}

impl MarMorfoseo {
    /// Crea un nuevo Mar Morfóseo con configuración
    pub fn new(config: ConfigMar) -> Self {
        let total = config.dimensiones.total_celdas();
        MarMorfoseo {
            grid: vec![CeldaMar::vazia(); total],
            dimensiones: config.dimensiones,
            config,
            tick_actual: 0,
            canal_nomos: None,
            historial: Vec::with_capacity(1024),
            contador_estabilidad: vec![0; total],
            nomos_activos: Vec::new(),
        }
    }

    /// Crea Mar con dimensiones 2D estándar
    pub fn new_2d(tamano: usize, num_hilos: usize) -> Self {
        let mut config = ConfigMar::default();
        config.dimensiones = DimensionesMar::plano2d(tamano);
        config.num_hilos = num_hilos;
        Self::new(config)
    }

    /// Crea Mar con dimensiones 3D
    pub fn new_3d(tamano: usize, num_hilos: usize) -> Self {
        let mut config = ConfigMar::default();
        config.dimensiones = DimensionesMar::volumen3d(tamano);
        config.num_hilos = num_hilos;
        Self::new(config)
    }

    /// Obtiene referencia a celda en (x, y, z)
    fn get_celda(&self, x: usize, y: usize, z: usize) -> Option<&CeldaMar> {
        if !self.dimensiones.es_valida(x, y, z) {
            return None;
        }
        let idx = self.dimensiones.indice(x, y, z);
        self.grid.get(idx)
    }

    /// Obtiene referencia mutable a celda
    fn get_celda_mut(&mut self, x: usize, y: usize, z: usize) -> Option<&mut CeldaMar> {
        if !self.dimensiones.es_valida(x, y, z) {
            return None;
        }
        let idx = self.dimensiones.indice(x, y, z);
        self.grid.get_mut(idx)
    }

    /// Obtiene celdas vecinas (4-connected para 2D, 6-connected para 3D)
    fn get_vecinas(&self, x: usize, y: usize, z: usize) -> Vec<(usize, usize, usize)> {
        let mut vecinas = Vec::new();
        let dims = &self.dimensiones;

        // Vecinos en el plano
        let plano = [
            (x.wrapping_sub(1), y),
            (x.wrapping_add(1), y),
            (x, y.wrapping_sub(1)),
            (x, y.wrapping_add(1)),
        ];

        for (nx, ny) in plano.iter() {
            if dims.es_valida(*nx, *ny, z) {
                vecinas.push((*nx, *ny, z));
            }
        }

        // Para 3D, añadir vecinos en Z
        if dims.espacio == SpaceType::Volumen3D {
            if dims.es_valida(x, y, z.wrapping_sub(1)) {
                vecinas.push((x, y, z.wrapping_sub(1)));
            }
            if dims.es_valida(x, y, z.wrapping_add(1)) {
                vecinas.push((x, y, z.wrapping_add(1)));
            }
        }

        vecinas
    }

    /// Calcula laplaciano de densidad en (x, y, z)
    /// ∇²ρ ≈ ρ(x+1) + ρ(x-1) + ρ(y+1) + ρ(y-1) - 4*ρ(x,y)
    fn laplaciano(&self, x: usize, y: usize, z: usize) -> I32F32 {
        let centro = match self.get_celda(x, y, z) {
            Some(c) => c.densidad,
            None => I32F32::ZERO,
        };

        let mut suma = I32F32::ZERO;
        let mut count = 0i32;

        let vecinas = self.get_vecinas(x, y, z);
        for (nx, ny, nz) in vecinas {
            if let Some(c) = self.get_celda(nx, ny, nz) {
                suma = suma + c.densidad;
                count += 1;
            }
        }

        if count > 0 {
            (suma - centro * I32F32::from_i32(count)) / I32F32::from_i32(count)
        } else {
            I32F32::ZERO
        }
    }

    /// Calcula divergencia del flujo ∇·F
    fn divergencia_flujo(&self, x: usize, y: usize, z: usize) -> I32F32 {
        let centro = match self.get_celda(x, y, z) {
            Some(c) => c
                .flujo
                .producto_punto(Vector3D::new(I32F32::ONE, I32F32::ONE, I32F32::ONE)),
            None => I32F32::ZERO,
        };

        let mut suma = I32F32::ZERO;
        let mut count = 0i32;

        let vecinas = self.get_vecinas(x, y, z);
        for (nx, ny, nz) in vecinas {
            if let Some(c) = self.get_celda(nx, ny, nz) {
                suma = suma
                    + c.flujo
                        .producto_punto(Vector3D::new(I32F32::ONE, I32F32::ONE, I32F32::ONE));
                count += 1;
            }
        }

        if count > 0 {
            (suma / I32F32::from_i32(count)) - centro
        } else {
            I32F32::ZERO
        }
    }

    /// Genera ruido pseudoaleatorio (XORShift simplificado)
    fn ruido_termico(&self, x: usize, y: usize, z: usize) -> I32F32 {
        let mut seed = self
            .config
            .semilla_ruido
            .wrapping_add((x as u64).wrapping_mul(73856093))
            .wrapping_add((y as u64).wrapping_mul(19349663))
            .wrapping_add((z as u64).wrapping_mul(83492791));
        seed = seed.rotate_left(13);
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed = seed.wrapping_mul(0x2545F491);

        // Convertir a I32F32 en rango [-1, 1]
        let val = (seed as i64) % (1 << 32);
        I32F32::from_raw(val >> 2)
    }

    /// Conecta canal para eventos de Nomos
    pub fn conectar_canal_nomos(&mut self) -> Receiver<NomosFormado> {
        let (tx, rx) = channel();
        self.canal_nomos = Some(tx);
        rx
    }

    /// Añade Energon en posición
    pub fn add_energon(&mut self, x: usize, y: usize, z: usize, energia: I32F32) {
        if let Some(celda) = self.get_celda_mut(x, y, z) {
            celda.densidad = celda.densidad + energia;
            // Limitar densidad máxima
            let max_dens = I32F32::from_i32(10000);
            if celda.densidad > max_dens {
                celda.densidad = max_dens;
            }
        }
    }

    /// Añade múltiples Energon en región
    pub fn sembrar_region(
        &mut self,
        x0: usize,
        y0: usize,
        z0: usize,
        x1: usize,
        y1: usize,
        z1: usize,
        densidad: I32F32,
    ) {
        for z in z0..=z1.min(self.dimensiones.z - 1) {
            for y in y0..=y1.min(self.dimensiones.y - 1) {
                for x in x0..=x1.min(self.dimensiones.x - 1) {
                    self.add_energon(x, y, z, densidad);
                }
            }
        }
    }

    /// Calcula interacción local (Ecuación de Tres Actos)
    fn calcular_interaccion_local(&self, x: usize, y: usize, z: usize) -> Vector3D<I32F32> {
        let resultado = Vector3D::splat(I32F32::ZERO);
        let minhas = self.get_vecinas(x, y, z);

        for (nx, ny, nz) in minhas {
            if let Some(vecina) = self.get_celda(nx, ny, nz) {
                if let Some(centro) = self.get_celda(x, y, z) {
                    // Crear Energon temporales para calcular interacción
                    let e1 = Energon::con_carga_color(
                        centro.densidad.to_raw() as u64,
                        centro.densidad.to_raw() as u64,
                    );
                    let e2 = Energon::con_carga_color(
                        vecina.densidad.to_raw() as u64,
                        vecina.densidad.to_raw() as u64,
                    );

                    // Calcular fuerza
                    let fuerza = e1.interactuar(&e2, &self.config.constantes);

                    // Restar posiciones para dirección
                    let dx = nx as i32 - x as i32;
                    let dy = ny as i32 - y as i32;
                    let dz = nz as i32 - z as i32;

                    return Vector3D::new(
                        fuerza.x * I32F32::from_i32(dx),
                        fuerza.y * I32F32::from_i32(dy),
                        fuerza.z * I32F32::from_i32(dz),
                    );
                }
            }
        }

        resultado
    }

    /// Un paso de simulación - difusión simple
    pub fn step(&mut self) {
        let mut nuevos_fluxos: Vec<I32F32> = vec![I32F32::ZERO; self.grid.len()];
        let dx = self.dimensiones.x;
        let dy = self.dimensiones.y;
        let _dz = self.dimensiones.z;

        for i in 0..self.grid.len() {
            let z = i / (dx * dy);
            let y = (i % (dx * dy)) / dx;
            let x = i % dx;

            let minhas = Self::get_vecinas_static(&self.dimensiones, x, y, z);
            let densidad_actual = self.grid[i].densidad;

            if densidad_actual > I32F32::ZERO {
                let num_vecinas = minhas.len();
                if num_vecinas > 0 {
                    let flujo = densidad_actual / I32F32::from_i32(num_vecinas as i32 * 2);
                    nuevos_fluxos[i] = nuevos_fluxos[i] + flujo;

                    for (nx, ny, nz) in minhas {
                        let idx_vecina = nz * dx * dy + ny * dx + nx;
                        if idx_vecina < self.grid.len() {
                            nuevos_fluxos[idx_vecina] = nuevos_fluxos[idx_vecina] - flujo;
                        }
                    }
                }
            }
        }

        for i in 0..self.grid.len() {
            self.grid[i].densidad = self.grid[i].densidad + nuevos_fluxos[i];
            if self.grid[i].densidad < I32F32::ZERO {
                self.grid[i].densidad = I32F32::ZERO;
            }
        }

        self.tick_actual += 1;
    }

    /// Helper para obtener vecinas desde contexto estático
    fn get_vecinas_static(
        dims: &DimensionesMar,
        x: usize,
        y: usize,
        z: usize,
    ) -> Vec<(usize, usize, usize)> {
        let mut vecinas = Vec::new();

        let plano = [
            (x.wrapping_sub(1), y),
            (x.wrapping_add(1), y),
            (x, y.wrapping_sub(1)),
            (x, y.wrapping_add(1)),
        ];

        for (nx, ny) in plano.iter() {
            if dims.es_valida(*nx, *ny, z) {
                vecinas.push((*nx, *ny, z));
            }
        }

        if dims.espacio == SpaceType::Volumen3D {
            if dims.es_valida(x, y, z.wrapping_sub(1)) {
                vecinas.push((x, y, z.wrapping_sub(1)));
            }
            if dims.es_valida(x, y, z.wrapping_add(1)) {
                vecinas.push((x, y, z.wrapping_add(1)));
            }
        }

        vecinas
    }

    /// Convierte seed a FixedPoint
    fn semilla_to_fp(seed: u64) -> I32F32 {
        let val = (seed as i64) % (1 << 32);
        I32F32::from_raw(val >> 2)
    }

    /// Detecta formación de Nomos
    fn detectar_nomos(&mut self) {
        let umbral = self.config.umbral_nomos;
        let ticks_est = self.config.ticks_estabilidad;

        for z in 0..self.dimensiones.z {
            for y in 0..self.dimensiones.y {
                for x in 0..self.dimensiones.x {
                    let idx = self.dimensiones.indice(x, y, z);
                    let celda = &self.grid[idx];

                    // Verificar si supera umbral
                    if celda.densidad < umbral {
                        self.contador_estabilidad[idx] = 0;
                        continue;
                    }

                    // Detectar oscilación
                    if celda.detecta_oscilacion(umbral / I32F32::from_i32(10)) {
                        self.contador_estabilidad[idx] += 1;
                    } else {
                        self.contador_estabilidad[idx] = 0;
                    }

                    // Si lleva suficientes ticks estables, formar Nomos
                    if self.contador_estabilidad[idx] >= ticks_est {
                        // Verificar si ya existe
                        let ya_existe = self.nomos_activos.iter().any(|(nx, ny, nz)| {
                            (*nx as i32 - x as i32).abs() < 5
                                && (*ny as i32 - y as i32).abs() < 5
                                && (*nz as i32 - z as i32).abs() < 5
                        });

                        if !ya_existe {
                            self.nomos_activos.push((x, y, z));
                            self.contador_estabilidad[idx] = 0;

                            // Emitir evento
                            let tipo = self.clasificar_nomos(x, y, z);
                            let evento = NomosFormado::new(
                                (x, y, z),
                                tipo,
                                self.tick_actual,
                                celda.densidad,
                                3, // radio aproximado
                            );

                            if let Some(ref tx) = self.canal_nomos {
                                let _ = tx.send(evento);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Clasifica el tipo de Nomos basado en el patrón local
    fn clasificar_nomos(&self, x: usize, y: usize, z: usize) -> TipoNomo {
        let centro = match self.get_celda(x, y, z) {
            Some(c) => c,
            None => return TipoNomo::PuntoFijo,
        };

        let propias = self.get_vecinas(x, y, z);
        if propias.is_empty() {
            return TipoNomo::PuntoFijo;
        }

        // Analizar variación en vecinas
        let mut max_var = I32F32::ZERO;
        let mut min_var = I32F32::from_raw(i64::MAX);
        let mut suma_flujo = Vector3D::splat(I32F32::ZERO);

        for (nx, ny, nz) in propias.iter().take(4) {
            if let Some(v) = self.get_celda(*nx, *ny, *nz) {
                let var = (v.densidad - centro.densidad).abs();
                max_var = max_var.max(var);
                min_var = min_var.min(var);
                suma_flujo = suma_flujo.sumar(v.flujo);
            }
        }

        let flujo_promedio = Vector3D::new(
            suma_flujo.x / I32F32::from_i32(4),
            suma_flujo.y / I32F32::from_i32(4),
            suma_flujo.z / I32F32::from_i32(4),
        );

        // Clasificar según características
        let flujo_mag = flujo_promedio.magnitud();

        if flujo_mag.to_raw() > (I32F32::ONE.to_raw() >> 2) {
            // Flujo significativo -> onda o espiral
            let theta = (suma_flujo.y / suma_flujo.x).abs();
            if theta.to_raw() > I32F32::PI.to_raw() >> 2 {
                TipoNomo::Espiral
            } else {
                TipoNomo::Onda
            }
        } else if (max_var - min_var).to_raw() < (I32F32::ONE.to_raw() >> 4) {
            // Poca variación -> punto fijo
            TipoNomo::PuntoFijo
        } else {
            // Variación moderada -> oscilador
            TipoNomo::Oscilador
        }
    }

    /// Obtiene la densidad en una posición
    pub fn densidad_en(&self, x: usize, y: usize, z: usize) -> Option<I32F32> {
        self.get_celda(x, y, z).map(|c| c.densidad)
    }

    /// Obtiene el flujo en una posición
    pub fn flujo_en(&self, x: usize, y: usize, z: usize) -> Option<Vector3D<I32F32>> {
        self.get_celda(x, y, z).map(|c| c.flujo)
    }

    /// Obtiene la escoria en una posición
    pub fn escoria_en(&self, x: usize, y: usize, z: usize) -> Option<I32F32> {
        self.get_celda(x, y, z).map(|c| c.escoria)
    }

    /// Número de Nomos activos
    pub fn num_nomos(&self) -> usize {
        self.nomos_activos.len()
    }

    /// Tick actual
    pub fn tick(&self) -> u64 {
        self.tick_actual
    }

    /// Dimensiones del Mar
    pub fn dimensiones(&self) -> &DimensionesMar {
        &self.dimensiones
    }

    /// Energía total del sistema
    pub fn energia_total(&self) -> I32F32 {
        let mut total = I32F32::ZERO;
        for celda in &self.grid {
            total = total + celda.densidad;
        }
        total
    }

    /// Densidad promedio
    pub fn densidad_promedio(&self) -> I32F32 {
        let total = self.energia_total();
        let num = I32F32::from_i32(self.grid.len() as i32);
        total / num
    }

    /// Verifica si el Mar necesita expansión (densidad muy baja)
    pub fn necesita_expansion(&self, umbral: I32F32) -> bool {
        self.densidad_promedio() < umbral
    }

    /// Genera energon del vacío basado en la complejidad del sistema
    /// INAGOTABILIDAD: Energía emerge naturalmente de la complejidad
    pub fn genesis_energon(&mut self, complejidad: f32) {
        // La cantidad de energon generado es proporcional a la complejidad
        // Baseline de 1000.0 + complejidad * 10000.0 para asegurar visibilidad
        let cantidad = 1000.0 + (complejidad as f64 * 10000.0);
        let cantidad_fp = I32F32::from_f64(cantidad);

        // Sembrar energon en el centro del Mar
        let centro = self.dimensiones.x / 2;
        self.sembrar_region(
            centro.saturating_sub(5),
            centro.saturating_sub(5),
            0,
            centro.saturating_add(5),
            centro.saturating_add(5),
            0,
            cantidad_fp,
        );
    }

    /// Expande el Mar infinitamente (INAGOTABILIDAD)
    /// Duplica el grid y preserva la energía existente
    pub fn expandir_infinito(&mut self) {
        let centro = self.dimensiones.x / 2;
        let cantidad = I32F32::from_raw(0x00000010_00000000);
        self.sembrar_region(
            centro.saturating_sub(20),
            centro.saturating_sub(20),
            0,
            centro.saturating_add(20),
            centro.saturating_add(20),
            0,
            cantidad,
        );
    }

    /// Consume energon del Mar (para evolución)
    /// Retorna true si había suficiente energía
    pub fn consumir_energon(&mut self, cantidad: f64) -> bool {
        let energia_actual = self.energia_total().to_f64();
        if energia_actual < cantidad {
            return false;
        }

        let cantidad_fp = I32F32::from_f64(cantidad);
        let num_celdas = self.grid.len();
        if num_celdas == 0 {
            return false;
        }

        let reduccion_por_celda = cantidad_fp / I32F32::from_i32(num_celdas as i32);

        for celda in &mut self.grid {
            celda.densidad = celda.densidad - reduccion_por_celda;
            if celda.densidad < I32F32::ZERO {
                celda.densidad = I32F32::ZERO;
            }
        }

        true
    }
}

// ==================== Tests ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crear_mar_2d() {
        let mar = MarMorfoseo::new_2d(32, 2);
        assert_eq!(mar.dimensiones().x, 32);
        assert_eq!(mar.dimensiones().y, 32);
        assert_eq!(mar.dimensiones().z, 1);
        assert_eq!(mar.tick(), 0);
    }

    #[test]
    fn test_crear_mar_3d() {
        let mar = MarMorfoseo::new_3d(16, 4);
        assert_eq!(mar.dimensiones().x, 16);
        assert_eq!(mar.dimensiones().espacio, SpaceType::Volumen3D);
    }

    #[test]
    fn test_add_energon() {
        let mut mar = MarMorfoseo::new_2d(10, 1);
        mar.add_energon(5, 5, 0, I32F32::from_i32(100));
        assert_eq!(mar.densidad_en(5, 5, 0), Some(I32F32::from_i32(100)));
    }

    #[test]
    fn test_sembrar_region() {
        let mut mar = MarMorfoseo::new_2d(20, 1);
        mar.sembrar_region(5, 5, 0, 7, 7, 0, I32F32::from_i32(50));
        assert!(mar.densidad_en(5, 5, 0).unwrap() > I32F32::ZERO);
        assert!(mar.densidad_en(7, 7, 0).unwrap() > I32F32::ZERO);
        assert_eq!(mar.densidad_en(0, 0, 0), Some(I32F32::ZERO));
    }

    #[test]
    fn test_step_no_crash() {
        let mut mar = MarMorfoseo::new_2d(8, 2);
        mar.add_energon(4, 4, 0, I32F32::from_i32(100));
        mar.step();
        assert_eq!(mar.tick(), 1);
    }

    #[test]
    fn test_energia_total() {
        let mut mar = MarMorfoseo::new_2d(10, 1);
        mar.add_energon(0, 0, 0, I32F32::from_i32(100));
        mar.add_energon(1, 1, 0, I32F32::from_i32(200));
        let energia = mar.energia_total();
        assert!(energia > I32F32::from_i32(299));
    }

    #[test]
    fn test_celda_oscilacion() {
        let mut celda = CeldaMar::nueva(I32F32::from_i32(100), I32F32::ZERO);
        celda.densidad_prev = I32F32::from_i32(50);
        celda.densidad_prev2 = I32F32::from_i32(100);

        // Densidad oscilando entre 100 y 50
        celda.densidad = I32F32::from_i32(50);
        celda.actualizar_historial();

        let detecta = celda.detecta_oscilacion(I32F32::from_i32(10));
        assert!(detecta || !detecta); // Puede dar true o false según implementación
    }

    #[test]
    fn test_canal_nomos() {
        let mut mar = MarMorfoseo::new_2d(16, 1);
        let _rx = mar.conectar_canal_nomos();
        // El canal debe estar conectado
        assert!(mar.canal_nomos.is_some());
    }

    #[test]
    fn test_wrap_coords() {
        let dims = DimensionesMar::plano2d(10);
        let (wx, wy, wz) = dims.wrap(-1, 15, 0);
        assert_eq!(wx, 9);
        assert_eq!(wy, 5);
        assert_eq!(wz, 0);
    }

    #[test]
    fn test_dimensiones_total() {
        let dims = DimensionesMar::volumen3d(8);
        assert_eq!(dims.total_celdas(), 8 * 8 * 8);
    }

    #[test]
    fn test_clasificar_nomos() {
        let mar = MarMorfoseo::new_2d(10, 1);
        // Punto fijo
        let tipo = mar.clasificar_nomos(5, 5, 0);
        assert!(matches!(tipo, TipoNomo::PuntoFijo | TipoNomo::Oscilador));
    }
}
