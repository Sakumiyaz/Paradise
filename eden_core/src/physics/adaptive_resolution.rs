//! # Resolución Adaptativa del Mar Morfóseo
//!
//! Sistema de quadtree para refinamiento automático basado en complejidad.
//!
//! ## Concepto
//!
//! El Mar Morfóseo puede subdividirse dinámicamente en regiones de mayor resolución
//! cuando la densidad de Auton o variación de Energon supera un umbral.
#![allow(dead_code)]
#![allow(non_snake_case)]

use crate::physics::energon::Vector3D;
use crate::physics::fixed_point::I32F32;
use std::collections::HashMap;

// ============================================================================
// CONSTANTES
// ============================================================================

/// Nivel máximo de subdivisión (profundidad del quadtree)
pub const MAX_NIVEL_SUBDIVISION: u32 = 6;

/// Tamaño mínimo de celda en la más alta resolución
pub const TAMANO_CELDA_MINIMO: usize = 16;

/// Umbral de densidad de Auton para subdivisión
pub const UMBRAL_DENSIDAD_AUTON: usize = 3;

/// Umbral de variación de Energon para subdivisión
pub const UMBRAL_VARIACION_ENERGON: I32F32 = I32F32::from_i32(10);

/// Factor de fusión para celdas vecinas similares
pub const UMBRAL_SIMILITUD_FUSION: I32F32 = I32F32::from_i32(2);

// ============================================================================
// TIPOS DE CELDA ADAPTATIVA
// ============================================================================

/// Posición en la quadtree (coordenadas lógicas)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PosicionQuad {
    /// Nivel en el quadtree (0 = raíz)
    pub nivel: u32,
    /// Coordenada X en este nivel
    pub x: u32,
    /// Coordenada Y en este nivel
    pub y: u32,
}

impl PosicionQuad {
    /// Crea posición en nivel 0 (raíz)
    pub fn raiz(x: u32, y: u32) -> Self {
        PosicionQuad { nivel: 0, x, y }
    }

    /// Calcula el índice lineal para esta posición
    pub fn indice_lineal(&self) -> u64 {
        let nivel_mask = (1u32 << self.nivel) - 1;
        (self.nivel as u64) << 62
            | ((self.x & nivel_mask) as u64) << 31
            | (self.y & nivel_mask) as u64
    }

    /// Obtiene la posición del padre en el nivel superior
    pub fn padre(&self) -> Option<PosicionQuad> {
        if self.nivel == 0 {
            None
        } else {
            Some(PosicionQuad {
                nivel: self.nivel - 1,
                x: self.x >> 1,
                y: self.y >> 1,
            })
        }
    }

    /// Obtiene las 4 posiciones hijo en el nivel siguiente
    pub fn hijos(&self) -> [PosicionQuad; 4] {
        let siguiente = self.nivel + 1;
        let base_x = self.x << 1;
        let base_y = self.y << 1;
        [
            PosicionQuad {
                nivel: siguiente,
                x: base_x,
                y: base_y,
            },
            PosicionQuad {
                nivel: siguiente,
                x: base_x + 1,
                y: base_y,
            },
            PosicionQuad {
                nivel: siguiente,
                x: base_x,
                y: base_y + 1,
            },
            PosicionQuad {
                nivel: siguiente,
                x: base_x + 1,
                y: base_y + 1,
            },
        ]
    }
}

/// Estado de subdivision de una celda
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EstadoSubdivision {
    /// Hoja - no subdivided
    Hoja,
    /// Nodo subdividido con 4 hijos
    Nodo([PosicionQuad; 4]),
}

/// Datos de una celda en la quadtree
#[derive(Debug, Clone)]
pub struct CeldaAdaptativa {
    /// Posición en quadtree
    pub posicion: PosicionQuad,
    /// Estado de subdivisión
    pub subdivision: EstadoSubdivision,
    /// Densidad de Energon
    pub densidad: I32F32,
    /// Flujo de Energon
    pub flujo: Vector3D<I32F32>,
    /// Escoria
    pub escoria: I32F32,
    /// Densidad previa (para variación)
    pub densidad_prev: I32F32,
    /// Historial de densidades (para detectar oscilaciones)
    pub historial_densidades: Vec<I32F32>,
    /// Contenido de Auton IDs en esta celda
    pub auton_ids: Vec<u64>,
    /// Densidad de Auton en esta celda
    pub densidad_auton: usize,
    /// Última actualización
    pub tick_actualizacion: u64,
    /// Si ha sido refinada forzosamente por el Demiurgo
    pub refinamiento_forzado: bool,
}

impl CeldaAdaptativa {
    /// Crea celda en posición raíz
    pub fn nueva_raiz(x: u32, y: u32) -> Self {
        CeldaAdaptativa {
            posicion: PosicionQuad::raiz(x, y),
            subdivision: EstadoSubdivision::Hoja,
            densidad: I32F32::ZERO,
            flujo: Vector3D::splat(I32F32::ZERO),
            escoria: I32F32::ZERO,
            densidad_prev: I32F32::ZERO,
            historial_densidades: Vec::with_capacity(8),
            auton_ids: Vec::new(),
            densidad_auton: 0,
            tick_actualizacion: 0,
            refinamiento_forzado: false,
        }
    }

    /// Crea celda en posición específica
    pub fn nueva(posicion: PosicionQuad) -> Self {
        CeldaAdaptativa {
            posicion,
            subdivision: EstadoSubdivision::Hoja,
            densidad: I32F32::ZERO,
            flujo: Vector3D::splat(I32F32::ZERO),
            escoria: I32F32::ZERO,
            densidad_prev: I32F32::ZERO,
            historial_densidades: Vec::with_capacity(8),
            auton_ids: Vec::new(),
            densidad_auton: 0,
            tick_actualizacion: 0,
            refinamiento_forzado: false,
        }
    }

    /// Calcula variación de densidad
    pub fn variacion_densidad(&self) -> I32F32 {
        (self.densidad - self.densidad_prev).abs()
    }

    /// Verifica si necesita subdivisión
    pub fn necesita_subdivision(&self) -> bool {
        // Verificar nivel máximo
        if self.posicion.nivel >= MAX_NIVEL_SUBDIVISION {
            return false;
        }

        // Verificar si ya está subdividida
        if !matches!(self.subdivision, EstadoSubdivision::Hoja) {
            return false;
        }

        // Verificar densidad de Auton
        if self.densidad_auton >= UMBRAL_DENSIDAD_AUTON {
            return true;
        }

        // Verificar variación de Energon
        if self.variacion_densidad() > UMBRAL_VARIACION_ENERGON {
            return true;
        }

        // Verificar refinamiento forzado
        if self.refinamiento_forzado {
            return true;
        }

        false
    }

    /// Verifica si puede fusionarse con hermanos
    pub fn puede_fusionarse(&self) -> bool {
        if matches!(self.subdivision, EstadoSubdivision::Nodo(_)) {
            return false;
        }

        // Solo fusionar si variación es muy baja y sin Auton
        self.variacion_densidad() < UMBRAL_SIMILITUD_FUSION
            && self.densidad_auton == 0
            && !self.refinamiento_forzado
    }

    /// Actualiza historial de densidades
    pub fn actualizar_historial(&mut self) {
        self.historial_densidades.push(self.densidad);
        if self.historial_densidades.len() > 8 {
            self.historial_densidades.remove(0);
        }
        self.densidad_prev = self.densidad;
    }

    /// Agrega Auton a esta celda
    pub fn agregar_auton(&mut self, id: u64) {
        if !self.auton_ids.contains(&id) {
            self.auton_ids.push(id);
            self.densidad_auton = self.auton_ids.len();
        }
    }

    /// Remueve Auton de esta celda
    pub fn remover_auton(&mut self, id: u64) {
        self.auton_ids.retain(|&x| x != id);
        self.densidad_auton = self.auton_ids.len();
    }

    /// Calcula tamaño de celda efectivo basado en nivel
    pub fn tamanoEfectivo(&self, tamano_base: usize) -> usize {
        let nivel_inverso = MAX_NIVEL_SUBDIVISION - self.posicion.nivel;
        tamano_base >> nivel_inverso
    }
}

// ============================================================================
// GRID ADAPTATIVO (QUADTREE)
// ============================================================================

/// Grid adaptativo basado en quadtree
pub struct GridAdaptativo {
    /// Raíces del quadtree (una por cada celda de nivel 0)
    celdas: HashMap<PosicionQuad, CeldaAdaptativa>,
    /// Dimensión base del grid (debe ser potencia de 2)
    dim_base: usize,
    /// Nivel actual máximo alcanzado
    nivel_maximo_alcanzado: u32,
    /// Contador de subdivisiones totales
    total_subdivisiones: u64,
    /// Contador de fusiones totales
    total_fusiones: u64,
}

impl GridAdaptativo {
    /// Crea nuevo grid adaptativo
    pub fn new(dim_base: usize) -> Self {
        // Verificar que dim_base es potencia de 2
        assert!(
            dim_base.is_power_of_two(),
            "dim_base debe ser potencia de 2"
        );

        let mut celdas = HashMap::new();

        // Crear nivel 0 (raíz) - una celda por cada unidad
        for y in 0..dim_base {
            for x in 0..dim_base {
                let pos = PosicionQuad::raiz(x as u32, y as u32);
                celdas.insert(pos, CeldaAdaptativa::nueva_raiz(x as u32, y as u32));
            }
        }

        GridAdaptativo {
            celdas,
            dim_base,
            nivel_maximo_alcanzado: 0,
            total_subdivisiones: 0,
            total_fusiones: 0,
        }
    }

    /// Obtiene celda en posición
    pub fn get(&self, pos: &PosicionQuad) -> Option<&CeldaAdaptativa> {
        self.celdas.get(pos)
    }

    /// Obtiene celda mutable en posición
    pub fn get_mut(&mut self, pos: &PosicionQuad) -> Option<&mut CeldaAdaptativa> {
        self.celdas.get_mut(pos)
    }

    /// Obtiene todas las celdas hoja
    pub fn celdas_hoja(&self) -> Vec<&CeldaAdaptativa> {
        self.celdas
            .values()
            .filter(|c| matches!(c.subdivision, EstadoSubdivision::Hoja))
            .collect()
    }

    /// Subdivide una celda en 4 hijos
    pub fn subdividir(&mut self, pos: &PosicionQuad) -> bool {
        // Obtener celda padre
        let celda = match self.celdas.get_mut(pos) {
            Some(c) => c,
            None => return false,
        };

        if !celda.necesita_subdivision() {
            return false;
        }

        let hijos = celda.posicion.hijos();
        let nuevo_nivel = hijos[0].nivel;
        let subdivision_nueva = EstadoSubdivision::Nodo(hijos);

        // Actualizar el padre a nodo
        celda.subdivision = subdivision_nueva;

        self.nivel_maximo_alcanzado = self.nivel_maximo_alcanzado.max(nuevo_nivel);
        self.total_subdivisiones += 1;

        // Ahora crear las celdas hijo
        // Necesitamos hacer esto fuera del borrow de celda
        for hijo in &hijos {
            self.celdas.insert(*hijo, CeldaAdaptativa::nueva(*hijo));
        }

        true
    }

    /// Fusiona una celda con sus hermanos (si todos pueden fusionarse)
    pub fn fusionar(&mut self, pos: &PosicionQuad) -> bool {
        let padre = match pos.padre() {
            Some(p) => p,
            None => return false,
        };

        // Verificar que todos los hermanos pueden fusionarse
        let hermanos = padre.hijos();
        let todos_pueden = hermanos.iter().all(|h| {
            self.celdas
                .get(h)
                .map(|c| c.puede_fusionarse())
                .unwrap_or(false)
        });

        if !todos_pueden {
            return false;
        }

        // Remover todos los hermanos
        for hermano in &hermanos {
            self.celdas.remove(hermano);
        }

        // Actualizar el padre a hoja
        if let Some(celda_padre) = self.celdas.get_mut(&padre) {
            celda_padre.subdivision = EstadoSubdivision::Hoja;
        }

        self.total_fusiones += 1;
        true
    }

    /// Encuentra la celda hoja que contiene una posición física
    pub fn encontrar_celda_containig(&self, px: f64, py: f64) -> Option<PosicionQuad> {
        let mut pos = PosicionQuad::raiz(
            px as u32 % self.dim_base as u32,
            py as u32 % self.dim_base as u32,
        );

        // Buscar descendiendo mientras sea nodo
        loop {
            if let Some(celda) = self.celdas.get(&pos) {
                match celda.subdivision {
                    EstadoSubdivision::Nodo(hijos) => {
                        // Elegir hijo basándose en posición
                        let child_x = ((px * 2.0) as u32) % 2;
                        let child_y = ((py * 2.0) as u32) % 2;
                        let idx = (child_x as usize) * 2 + (child_y as usize);
                        pos = hijos[idx];
                    }
                    EstadoSubdivision::Hoja => return Some(pos),
                }
            } else {
                return None;
            }
        }
    }

    /// Obtiene estadísticas del grid
    pub fn stats(&self) -> GridAdaptativoStats {
        let hojas = self.celdas_hoja();
        let mut celdas_por_nivel: HashMap<u32, usize> = HashMap::new();

        for celda in &hojas {
            *celdas_por_nivel.entry(celda.posicion.nivel).or_insert(0) += 1;
        }

        GridAdaptativoStats {
            total_celdas: self.celdas.len(),
            celdas_hoja: hojas.len(),
            nivel_maximo: self.nivel_maximo_alcanzado,
            total_subdivisiones: self.total_subdivisiones,
            total_fusiones: self.total_fusiones,
            celdas_por_nivel,
        }
    }

    /// Agrega Energon en posición física
    pub fn agregar_energon(&mut self, px: f64, py: f64, cantidad: I32F32) {
        if let Some(pos) = self.encontrar_celda_containig(px, py) {
            if let Some(celda) = self.celdas.get_mut(&pos) {
                celda.densidad = celda.densidad + cantidad;
            }
        }
    }

    /// Obtiene la densidad en una posición física
    pub fn get_densidad_en(&self, px: f64, py: f64) -> I32F32 {
        if let Some(pos) = self.encontrar_celda_containig(px, py) {
            if let Some(celda) = self.celdas.get(&pos) {
                return celda.densidad;
            }
        }
        I32F32::ZERO
    }

    /// Fuerza refinamiento en una región
    pub fn forzar_refinamiento(&mut self, x0: f64, y0: f64, x1: f64, y1: f64) -> usize {
        let mut count = 0;

        for px in (x0 as i32)..=(x1 as i32) {
            for py in (y0 as i32)..=(y1 as i32) {
                if let Some(pos) = self.encontrar_celda_containig(px as f64, py as f64) {
                    if let Some(celda) = self.celdas.get_mut(&pos) {
                        celda.refinamiento_forzado = true;
                        count += 1;
                    }
                }
            }
        }

        count
    }

    /// Procesa refínamientos pendientes
    pub fn procesar_refinamientos(&mut self) -> u64 {
        let mut subdivididas = 0u64;

        // Procesar todas las celdas que necesitan subdivision
        let posiciones: Vec<PosicionQuad> = self.celdas.keys().cloned().collect();

        for pos in posiciones {
            if let Some(celda) = self.celdas.get(&pos) {
                if celda.necesita_subdivision() {
                    subdivididas += 1;
                    self.subdividir(&pos);
                }
            }
        }

        subdivididas
    }
}

impl Default for GridAdaptativo {
    fn default() -> Self {
        Self::new(1024)
    }
}

/// Estadísticas del grid adaptativo
#[derive(Debug, Clone)]
pub struct GridAdaptativoStats {
    pub total_celdas: usize,
    pub celdas_hoja: usize,
    pub nivel_maximo: u32,
    pub total_subdivisiones: u64,
    pub total_fusiones: u64,
    pub celdas_por_nivel: HashMap<u32, usize>,
}

// ============================================================================
// ADMINISTRADOR DE RESOLUCIÓN ADAPTATIVA
// ============================================================================

/// Administrador de la resolución adaptativa del Mar
pub struct AdaptiveResolutionManager {
    /// Grid adaptativo
    grid: GridAdaptativo,
    /// Tick actual
    tick_actual: u64,
    /// Frecuencia de evaluación de subdivisión
    ticks_entre_evaluacion: u64,
    /// Historial de subdivisiones por tick
    historial_subdivisiones: HashMap<u64, u64>,
}

impl AdaptiveResolutionManager {
    /// Crea nuevo manager
    pub fn new(dim_base: usize) -> Self {
        AdaptiveResolutionManager {
            grid: GridAdaptativo::new(dim_base),
            tick_actual: 0,
            ticks_entre_evaluacion: 10,
            historial_subdivisiones: HashMap::new(),
        }
    }

    /// Avanza un tick
    pub fn tick(&mut self) {
        self.tick_actual += 1;

        // Procesar refínamientos cada N ticks
        if self.tick_actual % self.ticks_entre_evaluacion == 0 {
            let subdivididas = self.grid.procesar_refinamientos();
            self.historial_subdivisiones
                .insert(self.tick_actual, subdivididas);
        }
    }

    /// Obtiene referencia al grid
    pub fn grid(&self) -> &GridAdaptativo {
        &self.grid
    }

    /// Obtiene referencia mutable al grid
    pub fn grid_mut(&mut self) -> &mut GridAdaptativo {
        &mut self.grid
    }

    /// Obtiene estadísticas
    pub fn stats(&self) -> AdaptiveStats {
        let grid_stats = self.grid.stats();
        AdaptiveStats {
            tick: self.tick_actual,
            grid_stats,
        }
    }

    /// Establece frecuencia de evaluación
    pub fn set_ticks_evaluacion(&mut self, ticks: u64) {
        self.ticks_entre_evaluacion = ticks;
    }
}

impl Default for AdaptiveResolutionManager {
    fn default() -> Self {
        Self::new(1024)
    }
}

/// Estadísticas del sistema adaptativo
#[derive(Debug, Clone)]
pub struct AdaptiveStats {
    pub tick: u64,
    pub grid_stats: GridAdaptativoStats,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_posicion_quad_raiz() {
        let pos = PosicionQuad::raiz(5, 10);
        assert_eq!(pos.nivel, 0);
        assert_eq!(pos.x, 5);
        assert_eq!(pos.y, 10);
    }

    #[test]
    fn test_posicion_hijos() {
        let pos = PosicionQuad::raiz(0, 0);
        let hijos = pos.hijos();

        assert_eq!(hijos.len(), 4);
        assert_eq!(
            hijos[0],
            PosicionQuad {
                nivel: 1,
                x: 0,
                y: 0
            }
        );
        assert_eq!(
            hijos[1],
            PosicionQuad {
                nivel: 1,
                x: 1,
                y: 0
            }
        );
        assert_eq!(
            hijos[2],
            PosicionQuad {
                nivel: 1,
                x: 0,
                y: 1
            }
        );
        assert_eq!(
            hijos[3],
            PosicionQuad {
                nivel: 1,
                x: 1,
                y: 1
            }
        );
    }

    #[test]
    fn test_posicion_padre() {
        let pos = PosicionQuad {
            nivel: 2,
            x: 3,
            y: 5,
        };
        let padre = pos.padre();

        assert!(padre.is_some());
        let p = padre.unwrap();
        assert_eq!(p.nivel, 1);
        assert_eq!(p.x, 1);
        assert_eq!(p.y, 2);
    }

    #[test]
    fn test_posicion_raiz_no_tiene_padre() {
        let pos = PosicionQuad::raiz(5, 10);
        assert!(pos.padre().is_none());
    }

    #[test]
    fn test_grid_adaptativo_creacion() {
        let grid = GridAdaptativo::new(64);
        let stats = grid.stats();

        // 64x64 = 4096 celdas en nivel 0
        assert_eq!(stats.total_celdas, 4096);
        assert_eq!(stats.celdas_hoja, 4096);
        assert_eq!(stats.nivel_maximo, 0);
    }

    #[test]
    fn test_subdividir() {
        let mut grid = GridAdaptativo::new(2);
        let pos = PosicionQuad::raiz(0, 0);

        // Forzar refinamiento
        {
            let celda = grid.get_mut(&pos).unwrap();
            celda.refinamiento_forzado = true;
        }

        let resultado = grid.subdividir(&pos);
        assert!(resultado);

        let stats = grid.stats();
        // Start with 2x2 = 4 cells, subdivide (0,0): 4 + 4 children - 0 parent removed = 8
        assert_eq!(stats.total_celdas, 8);
        // Only (0,1), (1,0), (1,1) remain as leaves at level 0, plus 4 children at level 1 = 7 leaves
        assert_eq!(stats.celdas_hoja, 7);
        assert!(stats.nivel_maximo >= 1);
    }

    #[test]
    fn test_no_subdivide_si_no_necesita() {
        let mut grid = GridAdaptativo::new(64);
        let pos = PosicionQuad::raiz(0, 0);

        // Sin forzar ni Auton ni variación
        let resultado = grid.subdividir(&pos);
        assert!(!resultado);
    }

    #[test]
    fn test_agregar_auton() {
        let mut celda = CeldaAdaptativa::nueva_raiz(0, 0);

        assert_eq!(celda.densidad_auton, 0);

        celda.agregar_auton(42);
        assert_eq!(celda.densidad_auton, 1);

        celda.agregar_auton(43);
        assert_eq!(celda.densidad_auton, 2);

        // No duplica
        celda.agregar_auton(42);
        assert_eq!(celda.densidad_auton, 2);
    }

    #[test]
    fn test_remover_auton() {
        let mut celda = CeldaAdaptativa::nueva_raiz(0, 0);

        celda.agregar_auton(42);
        celda.agregar_auton(43);
        assert_eq!(celda.densidad_auton, 2);

        celda.remover_auton(42);
        assert_eq!(celda.densidad_auton, 1);
    }

    #[test]
    fn test_necesita_subdivision_por_auton() {
        let mut celda = CeldaAdaptativa::nueva_raiz(0, 0);

        // Añadir Auton por encima del umbral
        for i in 0..UMBRAL_DENSIDAD_AUTON {
            celda.agregar_auton(i as u64);
        }

        assert!(celda.necesita_subdivision());
    }

    #[test]
    fn test_necesita_subdivision_por_variacion() {
        let mut celda = CeldaAdaptativa::nueva_raiz(0, 0);
        celda.densidad_prev = I32F32::ZERO;
        celda.densidad = UMBRAL_VARIACION_ENERGON + I32F32::ONE;

        assert!(celda.necesita_subdivision());
    }

    #[test]
    fn test_variacion_densidad() {
        let mut celda = CeldaAdaptativa::nueva_raiz(0, 0);
        celda.densidad = I32F32::from_i32(100);
        celda.densidad_prev = I32F32::from_i32(80);

        let variacion = celda.variacion_densidad();
        assert_eq!(variacion, I32F32::from_i32(20));
    }

    #[test]
    fn test_tamano_efectivo() {
        let celda = CeldaAdaptativa::nueva(PosicionQuad {
            nivel: 2,
            x: 0,
            y: 0,
        });

        let tamano = celda.tamanoEfectivo(1024);
        // 1024 >> (6-2) = 1024 >> 4 = 64
        assert_eq!(tamano, 64);
    }

    #[test]
    fn test_estadisticas_grid() {
        let grid = GridAdaptativo::new(32);
        let stats = grid.stats();

        assert_eq!(stats.total_celdas, 1024); // 32x32
        assert_eq!(stats.celdas_hoja, 1024);
        assert_eq!(stats.total_subdivisiones, 0);
    }

    #[test]
    fn test_adaptive_manager_creacion() {
        let manager = AdaptiveResolutionManager::new(256);
        let stats = manager.stats();

        assert_eq!(stats.tick, 0);
        assert_eq!(stats.grid_stats.total_celdas, 65536); // 256x256
    }

    #[test]
    fn test_adaptive_manager_tick() {
        let mut manager = AdaptiveResolutionManager::new(64);

        manager.tick();
        manager.tick();

        let stats = manager.stats();
        assert_eq!(stats.tick, 2);
    }

    #[test]
    fn test_forzar_refinamiento() {
        let mut grid = GridAdaptativo::new(64);

        // Forzar región 0,0 a 10,10
        let count = grid.forzar_refinamiento(0.0, 0.0, 10.0, 10.0);
        assert!(count > 0);
    }

    #[test]
    fn testfusion_no_si_tiene_auton() {
        let mut celda = CeldaAdaptativa::nueva_raiz(0, 0);
        celda.agregar_auton(1);

        assert!(!celda.puede_fusionarse());
    }

    #[test]
    fn test_fusion_con_baja_variacion() {
        let mut celda = CeldaAdaptativa::nueva_raiz(0, 0);
        celda.densidad = I32F32::from_i32(100);
        celda.densidad_prev = I32F32::from_i32(99);
        // Variación = 1, menor que UMBRAL_SIMILITUD_FUSION = 2

        assert!(celda.puede_fusionarse());
    }
}
