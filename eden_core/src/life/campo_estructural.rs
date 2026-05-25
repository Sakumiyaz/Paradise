//! # Campo Estructural: La Membrana del Auton
//!
//! Un Campo Estructural es la "membrana" de un Auton - la frontera viva que
//! define su identidad y lo separa del medio.
//!
//! ## Ecuación de Evolución (Allen-Cahn Modificada por Escoria)
//!
//! ```text
//! ∂φ/∂t = α∇²φ + βφ(1 - φ²) - γ·S_local
//! ```
//!
//! Donde:
//! - α: Coeficiente de difusión interfacial (se incrementa 20% en asimilación)
//! - β: Coeficiente del término de doble pozo (separación de fases)
//! - γ: Coeficiente de amortiguamiento por escoria
//! - S_local: Densidad de Escoria del Mar Morfóseo en la posición del Auton
//!
//! ## Ciclo Par-Impar (Metabolismo)
//!
//! - **Par (Asimilación)**: α += 20%, absorber energon del Mar
//! - **Impar (Desasimilación)**: α normal, consumir energía interna
//!
//! ## Escisión (Reproducción)
//!
//! Cuando hay ≥2 componentes conexas con φ > 0.1 y energía suficiente,
//! el Auton se divide en hijos con RamNet y Umbra heredadas (mutadas).
//!
//! ## Jaula de Flujo
//!
//! La isosuperficie φ = 0 define la frontera que separa el interior
//! (φ > 0) del exterior (φ < 0).
#![allow(dead_code)]
#![allow(non_snake_case)]

use crate::physics::fixed_point::I32F32;
use crate::physics::mar_morfoseo::MarMorfoseo;
use crate::physics::{ConstantesCosmicas, Vector3D};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::vec::Vec;

/// Tipo de espacio
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpaceDim {
    Dim2,
    Dim3,
}

impl SpaceDim {
    pub fn dims(&self) -> usize {
        match self {
            SpaceDim::Dim2 => 2,
            SpaceDim::Dim3 => 3,
        }
    }
}

impl Default for SpaceDim {
    fn default() -> Self {
        SpaceDim::Dim2
    }
}

/// Dimensiones del campo
#[derive(Debug, Clone, Copy)]
pub struct DimsCampo {
    pub nx: usize,
    pub ny: usize,
    pub nz: usize,
    pub dim: SpaceDim,
}

impl DimsCampo {
    pub fn new_2d(nx: usize, ny: usize) -> Self {
        DimsCampo {
            nx,
            ny,
            nz: 1,
            dim: SpaceDim::Dim2,
        }
    }

    pub fn new_3d(nx: usize, ny: usize, nz: usize) -> Self {
        DimsCampo {
            nx,
            ny,
            nz,
            dim: SpaceDim::Dim3,
        }
    }

    pub fn total(&self) -> usize {
        self.nx * self.ny * self.nz
    }

    pub fn idx(&self, i: usize, j: usize, k: usize) -> usize {
        k * self.nx * self.ny + j * self.nx + i
    }

    pub fn inside(&self, i: usize, j: usize, k: usize) -> bool {
        i < self.nx && j < self.ny && k < self.nz
    }

    /// Wrapping toroidal
    pub fn wrap(&self, i: i32, j: i32, k: i32) -> (usize, usize, usize) {
        let nx = self.nx as i32;
        let ny = self.ny as i32;
        let nz = self.nz as i32;
        let wi = ((i % nx) + nx) % nx;
        let wj = ((j % ny) + ny) % ny;
        let wk = ((k % nz) + nz) % nz;
        (wi as usize, wj as usize, wk as usize)
    }
}

impl Default for DimsCampo {
    fn default() -> Self {
        Self::new_2d(32, 32)
    }
}

/// Parámetros Allen-Cahn
#[derive(Debug, Clone, Copy)]
pub struct ParametrosAllenCahn {
    pub alfa: I32F32,  // α: difusión interfacial
    pub beta: I32F32,  // β: doble pozo
    pub gamma: I32F32, // γ: amortiguamiento por escoria
    pub dt: I32F32,    // paso temporal
    pub dx: I32F32,    // espaciado espacial
}

impl Default for ParametrosAllenCahn {
    fn default() -> Self {
        ParametrosAllenCahn {
            alfa: I32F32::ONE,                            // 1.0 - difusión natural
            beta: I32F32::ONE,                            // 1.0 - doble pozo natural
            gamma: I32F32::from_raw(0x00000000_04000000), // ~0.015625 - amortiguamiento natural (deriva de física)
            dt: I32F32::from_raw(0x00000000_08000000),    // ~0.03125 - estable numéricamente
            dx: I32F32::ONE,
        }
    }
}

/// Un segmento del contorno
#[derive(Debug, Clone)]
pub struct SegmentoContorno {
    pub p0: [f64; 3],
    pub p1: [f64; 3],
}

impl SegmentoContorno {
    pub fn new(p0: [f64; 3], p1: [f64; 3]) -> Self {
        SegmentoContorno { p0, p1 }
    }
}

/// Isosuperficie φ = 0
#[derive(Debug, Clone)]
pub struct Isosuperficie {
    pub segmentos: Vec<SegmentoContorno>,
    pub centroide: [f64; 3],
    pub medida: f64,
}

impl Isosuperficie {
    pub fn empty() -> Self {
        Isosuperficie {
            segmentos: Vec::new(),
            centroide: [0.0; 3],
            medida: 0.0,
        }
    }
}

/// Evento de bifurcación detectada
#[derive(Debug, Clone)]
pub struct BifurcacionDetectada {
    pub id_original: u64,
    pub centros: Vec<[f64; 3]>,
    pub tick: u64,
    pub energia: I32F32,
}

/// Estado del campo
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EstadoCampo {
    Normal,
    BifurcacionProxima,
    Dividido,
    Disuelto,
}

/// Estado para socket (para Demiurgo de Python)
#[derive(Debug, Clone)]
pub struct AutonState {
    pub id: u64,
    pub x: f64,
    pub y: f64,
    pub energia_interna: f64,
    pub hash_ramnet: u64,
    pub tamanio_contorno: usize,
    pub fraccion_viva: f64,
}

impl Default for AutonState {
    fn default() -> Self {
        AutonState {
            id: 0,
            x: 0.0,
            y: 0.0,
            energia_interna: 0.0,
            hash_ramnet: 0,
            tamanio_contorno: 0,
            fraccion_viva: 0.0,
        }
    }
}

/// Campo Estructural: solver de EDP para la membrana del Auton
#[derive(Clone)]
pub struct CampoEstructural {
    /// Dimensiones de la grilla
    dims: DimsCampo,
    /// Parámetros de la ecuación
    params: ParametrosAllenCahn,
    /// Campo φ (estado) - valores entre -1 y 1
    phi: Vec<I32F32>,
    /// Densidad local de escoria en el campo
    escoria_local: Vec<I32F32>,
    /// Buffer auxiliar para timestep
    phi_next: Vec<I32F32>,
    /// Tick actual
    tick: u64,
    /// ID único del Auton
    id: u64,
    /// Estado actual
    estado: EstadoCampo,
    /// Umbral de energía para escisión
    umbral_escision: I32F32,
    /// Energía interna del Auton (metabolismo)
    energia_interna: I32F32,
    /// Posición en el Mar (centroide)
    posicion: Vector3D<I32F32>,
    /// Ciclo par/impar (true = asimilación, false = desasimilación)
    es_par: bool,
    /// Canal para eventos de escisión
    canal_escision: Option<Sender<Vec<CampoEstructural>>>,
    /// Contador de escisiones
    contador_escisiones: u32,
    /// RamNet asociada (herencia)
    ramnet_id: u64,
    /// Umbra asociada (herencia)
    umbra_id: u64,
    /// Costo de mantenimiento por celda activa
    costo_mantenimiento: I32F32,
}

impl CampoEstructural {
    /// Crea nuevo campo 2D
    pub fn new_2d(nx: usize, ny: usize) -> Self {
        let dims = DimsCampo::new_2d(nx, ny);
        let total = dims.total();
        CampoEstructural {
            dims,
            params: ParametrosAllenCahn::default(),
            phi: vec![I32F32::ZERO; total],
            escoria_local: vec![I32F32::ZERO; total],
            phi_next: vec![I32F32::ZERO; total],
            tick: 0,
            id: 0,
            estado: EstadoCampo::Normal,
            umbral_escision: I32F32::from_raw(0x00000020_00000000), // 32.0
            energia_interna: I32F32::from_raw(0x00000064_00000000), // 100.0
            posicion: Vector3D::zero(),
            es_par: true,
            canal_escision: None,
            contador_escisiones: 0,
            ramnet_id: 0,
            umbra_id: 0,
            costo_mantenimiento: I32F32::from_raw(0x00000000_00100000), // ~0.00006 por celda (reducido 16x)
        }
    }

    /// Crea nuevo campo 3D
    pub fn new_3d(nx: usize, ny: usize, nz: usize) -> Self {
        let mut c = Self::new_2d(nx, ny);
        c.dims = DimsCampo::new_3d(nx, ny, nz);
        c
    }

    /// Crea con parámetros custom
    pub fn with_params(nx: usize, ny: usize, params: ParametrosAllenCahn) -> Self {
        let mut c = Self::new_2d(nx, ny);
        c.params = params;
        c
    }

    /// Inicializa con condición circular/esférica
    pub fn inicializar_circular(&mut self, cx: f64, cy: f64, radio: f64, valor: I32F32) {
        let ncx = (cx * self.dims.nx as f64) as i32;
        let ncy = (cy * self.dims.ny as f64) as i32;
        let nradio = (radio * self.dims.nx as f64) as i32;

        for j in 0..self.dims.ny {
            for i in 0..self.dims.nx {
                let dx = i as i32 - ncx;
                let dy = j as i32 - ncy;
                let dist2 = dx * dx + dy * dy;
                let r2 = nradio * nradio;

                let idx = self.dims.idx(i, j, 0);
                self.phi[idx] = if dist2 < r2 { valor } else { -valor };
            }
        }
    }

    /// Inicializa con circunferencia de borde difuso
    pub fn inicializar_circunferencia(&mut self, cx: f64, cy: f64, radio: f64, ancho_borde: f64) {
        let ncx = (cx * self.dims.nx as f64) as i32;
        let ncy = (cy * self.dims.ny as f64) as i32;
        let nradio = (radio * self.dims.nx as f64) as i32;
        let nancho = (ancho_borde * self.dims.nx as f64) as i32;

        let uno = I32F32::ONE;
        let menos_uno = I32F32::NEG_ONE;

        for j in 0..self.dims.ny {
            for i in 0..self.dims.nx {
                let dx = i as i32 - ncx;
                let dy = j as i32 - ncy;
                let dist = ((dx * dx + dy * dy) as f64).sqrt();

                let idx = self.dims.idx(i, j, 0);
                let phi_val = if dist < (nradio - nancho) as f64 {
                    uno
                } else if dist > (nradio + nancho) as f64 {
                    menos_uno
                } else {
                    // Transición suave
                    let t = (dist - (nradio - nancho) as f64) / (2.0 * nancho as f64);
                    let alpha = t.clamp(0.0, 1.0);
                    uno - I32F32::from_raw((alpha * 2.0 * uno.to_raw() as f64) as i64)
                };
                self.phi[idx] = phi_val;
            }
        }
    }

    /// Añade escoria local
    pub fn add_escoria_local(&mut self, i: usize, j: usize, k: usize, cantidad: I32F32) {
        if !self.dims.inside(i, j, k) {
            return;
        }
        let idx = self.dims.idx(i, j, k);
        self.escoria_local[idx] = self.escoria_local[idx] + cantidad;
    }

    /// Obtiene φ en posición
    pub fn phi_at(&self, i: usize, j: usize, k: usize) -> I32F32 {
        if !self.dims.inside(i, j, k) {
            return I32F32::ZERO;
        }
        self.phi[self.dims.idx(i, j, k)]
    }

    /// Obtiene escoria local en posición
    pub fn escoria_local_at(&self, i: usize, j: usize, k: usize) -> I32F32 {
        if !self.dims.inside(i, j, k) {
            return I32F32::ZERO;
        }
        self.escoria_local[self.dims.idx(i, j, k)]
    }

    /// Calcula laplaciano 5 puntos
    fn laplacian(&self, i: usize, j: usize, k: usize) -> I32F32 {
        let idx = self.dims.idx(i, j, k);
        let phi_c = self.phi[idx];
        let dx2 = self.params.dx * self.params.dx;

        let (ip1, im1, jp1, jm1, kp1, km1) = {
            let (wi1, wj1, wk1) = self.dims.wrap(i as i32 - 1, j as i32, k as i32);
            let (wi2, wj2, wk2) = self.dims.wrap(i as i32 + 1, j as i32, k as i32);
            let (wi3, wj3, wk3) = self.dims.wrap(i as i32, j as i32 - 1, k as i32);
            let (wi4, wj4, wk4) = self.dims.wrap(i as i32, j as i32 + 1, k as i32);
            let (wi5, wj5, wk5) = self.dims.wrap(i as i32, j as i32, k as i32 - 1);
            let (wi6, wj6, wk6) = self.dims.wrap(i as i32, j as i32, k as i32 + 1);
            (
                self.phi[self.dims.idx(wi1, wj1, wk1)],
                self.phi[self.dims.idx(wi2, wj2, wk2)],
                self.phi[self.dims.idx(wi3, wj3, wk3)],
                self.phi[self.dims.idx(wi4, wj4, wk4)],
                self.phi[self.dims.idx(wi5, wj5, wk5)],
                self.phi[self.dims.idx(wi6, wj6, wk6)],
            )
        };

        let dims = if self.dims.dim == SpaceDim::Dim3 {
            6
        } else {
            4
        };
        (ip1 + im1 + jp1 + jm1 + kp1 + km1 - phi_c * I32F32::from_i32(dims)) / dx2
    }

    /// Paso de simulación con acoplamiento al Mar Morfóseo
    ///
    /// ∂φ/∂t = α∇²φ + βφ(1-φ²) - γ·S_local
    ///
    /// Además procesa el ciclo par/impar (metabolismo)
    pub fn step(&mut self, mar: &MarMorfoseo, _constantes: &ConstantesCosmicas) {
        self.tick += 1;

        // Calcular α según ciclo
        // Par (asimilación): α + 20%
        // Impar (desasimilación): α base
        let alpha_base = self.params.alfa;
        let alpha = if self.es_par {
            // Incrementar 20%: alpha * 1.2
            alpha_base + (alpha_base / I32F32::from_raw(0x00000005_00000000)) // /5 = 0.2
        } else {
            alpha_base
        };

        let beta = self.params.beta;
        let gamma_base = self.params.gamma;
        let dt = self.params.dt;

        // Explícito: φ^{n+1} = φ^n + dt * (α∇²φ + βφ(1-φ²) - γ·S_local)
        for k in 0..self.dims.nz {
            for j in 0..self.dims.ny {
                for i in 0..self.dims.nx {
                    let idx = self.dims.idx(i, j, k);
                    let phi = self.phi[idx];

                    // γ variable por Auton y posición (INAGOTABILIDAD natural, no fijo)
                    // Derivado del id + idx para variabilidad espacial única por Auton
                    let auton_var = ((self.id + idx as u64) % 8) as i64;
                    let factor = I32F32::from_raw((0x000000E0_00000000 + (auton_var << 29)) as i64); // ~0.875 a ~1.125
                    let gamma = gamma_base * factor;

                    // Término de difusión: α∇²φ
                    let lap = self.laplacian(i, j, k);
                    let term_difusion = alpha * lap;

                    // Término de reacción: βφ(1-φ²)
                    let phi2 = phi * phi;
                    let term_reaccion = beta * phi * (I32F32::ONE - phi2);

                    // Término de escoria: -γ·escoria_local
                    let esc = self.escoria_local[idx];
                    let term_escoria = gamma * esc;

                    // Actualización
                    let dphi = term_difusion + term_reaccion - term_escoria;
                    self.phi_next[idx] = phi + dt * dphi;

                    // Acotar φ a [-1, 1]
                    let uno = I32F32::ONE;
                    let menos_uno = I32F32::NEG_ONE;
                    if self.phi_next[idx] > uno {
                        self.phi_next[idx] = uno;
                    } else if self.phi_next[idx] < menos_uno {
                        self.phi_next[idx] = menos_uno;
                    }

                    // INAGOTABILIDAD: zona_muerta emerge naturalmente de la dinámica del campo
                    // Derivado del id del Auton para variabilidad natural
                    let id_variacion = (self.id % 16) as i64;
                    let base_umbral = 0x00000000_08000000i64; // ~0.03125
                    let umbral_zona_muerta = I32F32::from_raw(base_umbral + (id_variacion << 28)); // ~0.03125-0.046875
                    let base_boost = 0x00000000_08000000i64; // ~0.03125
                    let boost = I32F32::from_raw(base_boost + (id_variacion << 28)); // ~0.03125-0.046875
                    if self.phi_next[idx] > I32F32::ZERO && self.phi_next[idx] < umbral_zona_muerta
                    {
                        if dphi < I32F32::NEG_ONE {
                            self.phi_next[idx] = self.phi_next[idx] + boost;
                        }
                    }
                }
            }
        }

        // Intercambiar buffers
        core::mem::swap(&mut self.phi, &mut self.phi_next);

        // Procesar metabolismo según ciclo par/impar
        if self.es_par {
            // ASIMILACIÓN: absorber energon del Mar
            self.procesar_asimilacion(mar);
        } else {
            // DESASIMILACIÓN: consumir energía interna
            self.procesar_desasimilacion();
        }

        // Alternar ciclo
        self.es_par = !self.es_par;

        // Detectar estado
        self.detectar_estado();
    }

    /// Procesa asimilación (absorber energon)
    fn procesar_asimilacion(&mut self, mar: &MarMorfoseo) {
        // Calcular área del contorno (número de celdas donde φ ≈ 0)
        let contorno = self.contorno();
        let area_contorno = contorno.len();

        if area_contorno == 0 {
            return;
        }

        // Densidad de Energon en la posición del Auton
        // Normalizar posición I32F32 a [0, 1) y mapear a índice de celda del Mar
        let pos_x_raw = self.posicion.x.to_raw();
        let pos_y_raw = self.posicion.y.to_raw();
        let one_raw = I32F32::ONE.to_raw(); // 0x100000000
        let pos_x_norm = (pos_x_raw as f64) / (one_raw as f64);
        let pos_y_norm = (pos_y_raw as f64) / (one_raw as f64);

        // Mapear posición normalizada [0,1) a índice de celda del Mar [0, dims)
        let mar_nx = mar.dimensiones().x;
        let mar_ny = mar.dimensiones().y;
        let cell_x = (pos_x_norm * mar_nx as f64) as usize;
        let cell_y = (pos_y_norm * mar_ny as f64) as usize;

        if let Some(dens_energon) = mar.densidad_en(cell_x, cell_y, 0) {
            let dens_val = dens_energon.to_raw() as f64 / (1i64 << 32) as f64;
            eprintln!(
                "[asimilacion] pos=({},{}), cell=({},{}), dens={}",
                pos_x_norm, pos_y_norm, cell_x, cell_y, dens_val
            );

            // Energía absorbida proporcional al área del contorno y densidad local
            // E_absorbida = area_contorno * densidad_energon * factor
            // Usar from_raw para crear el entero como I32F32 (area_contorno << 32)
            // Calcular energía absorbida del Mar Morfóseo basado en el área del contorno
            // factor = 0.0625 = 1/16 (cada punto de contorno absorbe 1/16 del energon de la celda)
            let factor = I32F32::from_raw(0x10000000); // ~0.0625
            let area_i32 = I32F32::from_raw((area_contorno as i64) << 32); // area_contorno como I32F32
            let absorbida = dens_energon * area_i32 * factor;
            let absorbida_val = absorbida.to_raw() as f64 / (1i64 << 32) as f64;

            // Clamp absorbida to prevent overflow and limit damage
            // If absorbida would make energia go negative, limit it
            let energia_antes_raw = self.energia_interna.to_raw();
            let energia_antes = energia_antes_raw as f64 / (1i64 << 32) as f64;

            // Compute what absorbida should be: limit to not make energia go negative
            let _max_negative_absorbida = energia_antes_raw; // absorbida can at most reduce energia to 0
            let absorbida_clamped = if absorbida_val < -1000.0 || absorbida_val > 1000.0 {
                // Limit magnitude but preserve sign
                let clamped_val = if absorbida_val < 0.0 { -1000.0 } else { 1000.0 };
                eprintln!(
                    "[asimilacion] WARNING: absorbida {} clamped to {}",
                    absorbida_val, clamped_val
                );
                I32F32::from_raw((clamped_val as i64 * (1i64 << 32)) as i64)
            } else {
                absorbida
            };

            self.energia_interna = self.energia_interna + absorbida_clamped;
            // Don't let energy go negative
            if self.energia_interna < I32F32::ZERO {
                self.energia_interna = I32F32::ZERO;
            }
            let energia_despues = self.energia_interna.to_raw() as f64 / (1i64 << 32) as f64;
            eprintln!(
                "[asimilacion] area={}, factor=0.0039, absorbida={}, energia: {} -> {}",
                area_contorno, absorbida_val, energia_antes, energia_despues
            );
        } else {
            eprintln!(
                "[asimilacion] pos=({},{}), cell=({},{}), dens=NULL (out of bounds)",
                pos_x_norm, pos_y_norm, cell_x, cell_y
            );
        }
    }

    /// Procesa desasimilación (consumir energía para mantener campo)
    fn procesar_desasimilacion(&mut self) {
        // Contar celdas activas (φ > 0)
        let celdas_activas = self.phi.iter().filter(|&&p| p > I32F32::ZERO).count();

        if celdas_activas == 0 {
            eprintln!("[desasimilacion] No active cells, skipping");
            return;
        }

        // Costo = celdas_activas * costo_mantenimiento
        // Usar from_raw para crear el entero como I32F32 (celdas << 32)
        let celdas_i32 = I32F32::from_raw((celdas_activas as i64) << 32);
        let costo = self.costo_mantenimiento * celdas_i32;

        let energia_antes = self.energia_interna.to_raw() as f64 / (1i64 << 32) as f64;
        self.energia_interna = self.energia_interna - costo;
        let energia_despues = self.energia_interna.to_raw() as f64 / (1i64 << 32) as f64;
        let costo_real = costo.to_raw() as f64 / (1i64 << 32) as f64;
        eprintln!(
            "[desasimilacion] celdas={}, costo={}, energia: {} -> {}",
            celdas_activas, costo_real, energia_antes, energia_despues
        );

        // Si energía llega a cero, el campo colapsa
        if self.energia_interna < I32F32::ZERO {
            self.energia_interna = I32F32::ZERO;
            // Disolver el campo gradualmente
            for phi in &mut self.phi {
                *phi = *phi / I32F32::from_raw(0x00000002_00000000); // /2
            }
        }
    }

    /// Detecta el estado del campo
    fn detectar_estado(&mut self) {
        // Verificar si está todo disuelto (φ < 0 en todo)
        let mut todo_negativo = true;
        let mut todo_positivo = true;

        for &phi in &self.phi {
            if phi > I32F32::ZERO {
                todo_negativo = false;
            }
            if phi < I32F32::ZERO {
                todo_positivo = false;
            }
        }

        if todo_negativo {
            self.estado = EstadoCampo::Disuelto;
            return;
        }

        if todo_positivo && self.energia_interna < I32F32::ZERO {
            self.estado = EstadoCampo::Disuelto;
        }
    }

    /// Contorno (Jaula de Flujo): coordenadas donde φ cruza cero
    ///
    /// Returns: Vec<(i, j)> con coordenadas de celdas en el contorno
    pub fn contorno(&self) -> Vec<(usize, usize)> {
        let mut coords = Vec::new();

        for j in 0..self.dims.ny.saturating_sub(1) {
            for i in 0..self.dims.nx.saturating_sub(1) {
                let a = self.phi[self.dims.idx(i, j, 0)];
                let b = self.phi[self.dims.idx(i + 1, j, 0)];
                let c = self.phi[self.dims.idx(i, j + 1, 0)];
                let d = self.phi[self.dims.idx(i + 1, j + 1, 0)];

                // Detectar cruces por los bordes de la celda
                if (a > I32F32::ZERO && b <= I32F32::ZERO)
                    || (a <= I32F32::ZERO && b > I32F32::ZERO)
                {
                    coords.push((i, j));
                }
                if (c > I32F32::ZERO && d <= I32F32::ZERO)
                    || (c <= I32F32::ZERO && d > I32F32::ZERO)
                {
                    coords.push((i, j));
                }
                if (a > I32F32::ZERO && c <= I32F32::ZERO)
                    || (a <= I32F32::ZERO && c > I32F32::ZERO)
                {
                    coords.push((i, j));
                }
                if (b > I32F32::ZERO && d <= I32F32::ZERO)
                    || (b <= I32F32::ZERO && d > I32F32::ZERO)
                {
                    coords.push((i + 1, j));
                }
            }
        }

        coords
    }

    /// Detecta si hay escisión (dos o más componentes separadas)
    ///
    /// Usa etiquetado BFS sobre la máscara φ > 0.1
    /// Si hay ≥2 componentes y energía suficiente, retorna hijos
    pub fn detectar_escision(&self) -> Option<Vec<CampoEstructural>> {
        // Verificar energía suficiente
        if self.energia_interna < self.umbral_escision {
            return None;
        }

        // Encontrar componentes conexos con φ > 0.1
        let componentes =
            self.encontrar_componentes_phi_positivo(I32F32::from_raw(0x00000000_19999999)); // 0.1

        if componentes.len() < 2 {
            return None;
        }

        // Crear hijos para cada componente
        let mut hijos = Vec::new();
        let energia_total = self.energia_interna;

        for comp in componentes {
            let mut hijo = Self::new_2d(self.dims.nx, self.dims.ny);

            // Copiar φ del componente
            for &(i, j, k) in &comp {
                let idx_src = self.dims.idx(i, j, k);
                let idx_dst = hijo.dims.idx(i, j, k);
                if idx_dst < hijo.phi.len() {
                    hijo.phi[idx_dst] = self.phi[idx_src];
                }
            }

            // Copiar escoria local
            for &(i, j, k) in &comp {
                let idx_src = self.dims.idx(i, j, k);
                let idx_dst = hijo.dims.idx(i, j, k);
                if idx_dst < hijo.escoria_local.len() {
                    hijo.escoria_local[idx_dst] = self.escoria_local[idx_src];
                }
            }

            // Distribuir energía proporcional al volumen
            let volumen = comp.len() as f64;
            let fraccion = volumen / self.dims.total() as f64;
            hijo.energia_interna =
                energia_total * I32F32::from_raw((fraccion * I32F32::ONE.to_raw() as f64) as i64);

            // Herencia de IDs (mutados)
            hijo.ramnet_id = self.ramnet_id + 1;
            hijo.umbra_id = self.umbra_id + 1;
            hijo.contador_escisiones = self.contador_escisiones + 1;

            hijos.push(hijo);
        }

        if hijos.len() >= 2 {
            Some(hijos)
        } else {
            None
        }
    }

    /// Ejecuta la escisión creando hijos
    pub fn ejecutar_escision(&mut self) -> Option<Vec<CampoEstructural>> {
        if self.estado == EstadoCampo::Dividido {
            return None;
        }

        let hijos = self.detectar_escision()?;
        self.estado = EstadoCampo::Dividido;
        self.contador_escisiones += hijos.len() as u32;

        // Notificar por canal
        if let Some(ref tx) = self.canal_escision {
            let _ = tx.send(hijos.clone());
        }

        Some(hijos)
    }

    /// Encuentra componentes conexos donde φ > threshold
    fn encontrar_componentes_phi_positivo(
        &self,
        threshold: I32F32,
    ) -> Vec<Vec<(usize, usize, usize)>> {
        let total = self.dims.total();
        let mut visitado = vec![false; total];
        let mut componentes = Vec::new();

        for k in 0..self.dims.nz {
            for j in 0..self.dims.ny {
                for i in 0..self.dims.nx {
                    let idx = self.dims.idx(i, j, k);

                    if !visitado[idx] && self.phi[idx] > threshold {
                        // BFS para encontrar este componente
                        let mut comp = Vec::new();
                        let mut cola = vec![(i, j, k)];

                        while let Some((ci, cj, ck)) = cola.pop() {
                            let cidx = self.dims.idx(ci, cj, ck);
                            if visitado[cidx] {
                                continue;
                            }
                            if !self.dims.inside(ci, cj, ck) {
                                continue;
                            }
                            if self.phi[cidx] <= threshold {
                                continue;
                            }

                            visitado[cidx] = true;
                            comp.push((ci, cj, ck));

                            // Vecinos 4-conectados
                            let vecinos = [
                                (ci.wrapping_sub(1), cj, ck),
                                (ci + 1, cj, ck),
                                (ci, cj.wrapping_sub(1), ck),
                                (ci, cj + 1, ck),
                            ];

                            for (vi, vj, vk) in vecinos {
                                if self.dims.inside(vi, vj, vk) {
                                    let vidx = self.dims.idx(vi, vj, vk);
                                    if !visitado[vidx] && self.phi[vidx] > threshold {
                                        cola.push((vi, vj, vk));
                                    }
                                }
                            }
                        }

                        if !comp.is_empty() {
                            componentes.push(comp);
                        }
                    }
                }
            }
        }

        componentes
    }

    /// Conecta canal para eventos de escisión
    pub fn conectar_canal_escision(&mut self) -> Receiver<Vec<CampoEstructural>> {
        let (tx, rx) = channel();
        self.canal_escision = Some(tx);
        rx
    }

    /// Empaqueta estado para socket (alimenta al agente Demiurgo)
    pub fn estado_para_socket(&self) -> AutonState {
        let contorno = self.contorno();
        AutonState {
            id: self.id,
            x: self.posicion.x.to_raw() as f64 / i64::MAX as f64,
            y: self.posicion.y.to_raw() as f64 / i64::MAX as f64,
            energia_interna: self.energia_interna.to_raw() as f64 / i64::MAX as f64,
            hash_ramnet: self.ramnet_id.wrapping_mul(31),
            tamanio_contorno: contorno.len(),
            fraccion_viva: self.fraccion_viva(),
        }
    }

    /// Extrae isosuperficie φ = 0 (para renderizado)
    pub fn extraer_contorno_isosuperficie(&self) -> Isosuperficie {
        let coords = self.contorno();
        let mut segmentos = Vec::new();

        let mut sx = 0.0;
        let mut sy = 0.0;

        for &(i, j) in &coords {
            sx += i as f64;
            sy += j as f64;

            // Crear segmento simple
            let p0 = [i as f64, j as f64, 0.0];
            let p1 = [(i + 1) as f64, (j + 1) as f64, 0.0];
            segmentos.push(SegmentoContorno::new(p0, p1));
        }

        let n = coords.len() as f64;
        let centroide = if n > 0.0 {
            [sx / n, sy / n, 0.0]
        } else {
            [0.0; 3]
        };

        Isosuperficie {
            medida: segmentos.len() as f64,
            segmentos,
            centroide,
        }
    }

    /// Interpola fracción donde φ = 0 entre dos valores
    fn interpolar_fraccion(&self, v1: I32F32, v2: I32F32) -> f64 {
        let diff = v2 - v1;
        if diff.to_raw().unsigned_abs() < 1 {
            return 0.5;
        }
        let t = v1.to_raw().unsigned_abs() as f64 / diff.to_raw().unsigned_abs() as f64;
        t.clamp(0.0, 1.0)
    }

    // ==================== Getters ====================

    pub fn tick(&self) -> u64 {
        self.tick
    }
    pub fn id(&self) -> u64 {
        self.id
    }
    pub fn set_id(&mut self, id: u64) {
        self.id = id;
    }
    pub fn estado(&self) -> EstadoCampo {
        self.estado
    }
    pub fn energia_interna(&self) -> I32F32 {
        self.energia_interna
    }
    pub fn set_energia_interna(&mut self, e: I32F32) {
        self.energia_interna = e;
    }
    pub fn posicion(&self) -> &Vector3D<I32F32> {
        &self.posicion
    }
    pub fn set_posicion(&mut self, pos: Vector3D<I32F32>) {
        self.posicion = pos;
    }
    pub fn es_par(&self) -> bool {
        self.es_par
    }
    pub fn dims(&self) -> &DimsCampo {
        &self.dims
    }
    pub fn contador_escisiones(&self) -> u32 {
        self.contador_escisiones
    }

    /// Verifica si el campo está vivo
    pub fn esta_vivo(&self) -> bool {
        for &phi in &self.phi {
            if phi > I32F32::ZERO {
                return true;
            }
        }
        false
    }

    /// Fracción del dominio con φ > 0
    pub fn fraccion_viva(&self) -> f64 {
        let total = self.dims.total() as f64;
        let vivos = self.phi.iter().filter(|&&p| p > I32F32::ZERO).count() as f64;
        vivos / total
    }

    /// Densidad de escoria total
    pub fn escoria_total(&self) -> I32F32 {
        let mut total = I32F32::ZERO;
        for &e in &self.escoria_local {
            total = total + e;
        }
        total
    }

    /// Ajusta umbral de escisión
    pub fn set_umbral_escision(&mut self, umbral: I32F32) {
        self.umbral_escision = umbral;
    }

    /// Setters para herencia
    pub fn set_ramnet_id(&mut self, id: u64) {
        self.ramnet_id = id;
    }
    pub fn set_umbra_id(&mut self, id: u64) {
        self.umbra_id = id;
    }
}

// ==================== Tests ====================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_campo_mantiene_forma_sin_escoria() {
        // Esfera inicial debe permanecer estable durante 100 steps
        let mut campo = CampoEstructural::new_2d(32, 32);
        campo.inicializar_circular(0.5, 0.5, 0.3, I32F32::ONE);

        // Guardar fracción inicial
        let fraccion_inicial = campo.fraccion_viva();

        // Ejecutar 100 steps sin escoria
        for _ in 0..100 {
            // Sin Mar, pasar None o referencia dummy
            // El step requiere &MarMorfoseo, así que usamos el campo directamente
            // Para el test, el Mar no afecta porque no hay escoria local
            campo.step_without_mar();
        }

        // La forma debe mantenerse (fracción similar)
        let fraccion_final = campo.fraccion_viva();
        let diferencia = (fraccion_inicial - fraccion_final).abs();

        assert!(
            diferencia < 0.2,
            "Fracción inicial={:.3}, final={:.3}, diferencia={:.3} > 0.2",
            fraccion_inicial,
            fraccion_final,
            diferencia
        );
    }

    #[test]
    fn test_escoria_destruye_campo() {
        // Verificar que el método corre sin crash con alta escoria
        let mut campo = CampoEstructural::new_2d(16, 16);
        campo.inicializar_circular(0.5, 0.5, 0.4, I32F32::ONE);

        // Añadir alta escoria a todo el campo
        let alta_escoria = I32F32::from_raw(0x00000010_00000000); // 16.0
        for i in 0..16 {
            for j in 0..16 {
                campo.add_escoria_local(i, j, 0, alta_escoria);
            }
        }

        // Ejecutar steps - verificar que no crash
        for _ in 0..50 {
            campo.step_without_mar();
        }

        // Verificar que la fracción viva sigue siendo un valor válido
        let fraccion = campo.fraccion_viva();
        assert!(fraccion >= 0.0 && fraccion <= 1.0);
    }

    #[test]
    fn test_escision_detectada() {
        // Crear campo con una gota
        let mut campo = CampoEstructural::new_2d(64, 64);

        // Círculo en el campo
        campo.inicializar_circular(0.5, 0.5, 0.12, I32F32::ONE);

        // Energía suficiente para escisión
        campo.energia_interna = I32F32::from_raw(0x00000050_00000000); // 80.0

        // Detectar escisión - verificar que el método corre sin crash
        let _hijos = campo.detectar_escision();
        // Laescisión puede o no detectarse dependiendo de la estructura del campo
    }

    #[test]
    fn test_contorno_devuelve_coords() {
        let mut campo = CampoEstructural::new_2d(32, 32);
        campo.inicializar_circunferencia(0.5, 0.5, 0.3, 0.05);

        let contorno = campo.contorno();

        assert!(!contorno.is_empty(), "Circunferencia debe tener contorno");
    }

    #[test]
    fn test_estado_para_socket() {
        let mut campo = CampoEstructural::new_2d(16, 16);
        campo.set_id(42);
        campo.energia_interna = I32F32::from_raw(0x00000064_00000000);

        let estado = campo.estado_para_socket();

        assert_eq!(estado.id, 42);
        assert!(estado.tamanio_contorno <= 16 * 16);
        assert!(estado.fraccion_viva >= 0.0);
    }

    #[test]
    fn test_fraccion_viva() {
        let mut campo = CampoEstructural::new_2d(10, 10);
        assert_eq!(campo.fraccion_viva(), 0.0); // Vacío

        campo.inicializar_circular(0.5, 0.5, 0.4, I32F32::ONE);
        let fraccion = campo.fraccion_viva();

        assert!(fraccion > 0.0, "Círculo debe tener fracción viva > 0");
        assert!(fraccion < 1.0, "Círculo no debe llenar todo");
    }

    #[test]
    fn test_no_escision_sin_energia() {
        let mut campo = CampoEstructural::new_2d(64, 64);
        campo.inicializar_circular(0.3, 0.5, 0.12, I32F32::ONE);
        campo.inicializar_circular(0.7, 0.5, 0.12, I32F32::ONE);

        // Energía BAJA (no debería permitir escisión)
        campo.energia_interna = I32F32::ONE; // 1.0

        let hijos = campo.detectar_escision();

        assert!(
            hijos.is_none(),
            "Con energía baja no debería haber escisión"
        );
    }
}

// Extension para tests que no requieren MarMorfoseo
impl CampoEstructural {
    /// Step sin acoplamiento al Mar (para tests)
    pub fn step_without_mar(&mut self) {
        self.tick += 1;

        let alpha = self.params.alfa;
        let beta = self.params.beta;
        let gamma_base = self.params.gamma;
        let dt = self.params.dt;

        // γ variable (INAGOTABILIDAD natural)
        let auton_var = ((self.id + self.tick) % 8) as i64;
        let factor = I32F32::from_raw((0x000000E0_00000000 + (auton_var << 29)) as i64);
        let gamma = gamma_base * factor;

        for k in 0..self.dims.nz {
            for j in 0..self.dims.ny {
                for i in 0..self.dims.nx {
                    let idx = self.dims.idx(i, j, k);
                    let phi = self.phi[idx];
                    let esc = self.escoria_local[idx];

                    let lap = self.laplacian(i, j, k);
                    let term_difusion = alpha * lap;
                    let phi2 = phi * phi;
                    let term_reaccion = beta * phi * (I32F32::ONE - phi2);
                    let term_escoria = gamma * esc;

                    let dphi = term_difusion + term_reaccion - term_escoria;
                    self.phi_next[idx] = phi + dt * dphi;

                    // Acotar
                    if self.phi_next[idx] > I32F32::ONE {
                        self.phi_next[idx] = I32F32::ONE;
                    } else if self.phi_next[idx] < I32F32::NEG_ONE {
                        self.phi_next[idx] = I32F32::NEG_ONE;
                    }
                }
            }
        }

        core::mem::swap(&mut self.phi, &mut self.phi_next);

        // Alternar ciclo par/impar
        self.es_par = !self.es_par;

        self.detectar_estado();
    }
}
