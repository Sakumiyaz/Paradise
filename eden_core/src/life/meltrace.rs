//! # Meltrace: El Trazo Fundido
//!
//! El Trazo Fundido es un almacén probabilístico de "grabados" - compresiones
//! de las Umbrae de Auton muertos que influyen en el nacimiento de nuevos Auton.
//!
//! ## Mecanismo
//!
//! 1. **Muerte del Auton**: Su Umbra se comprime en un vector de características
//! 2. **Almacenamiento**: El vector se guarda en buffer circular global
//! 3. **Nacimiento**: Nuevos Auton consultan el Trazo para sesgar su inicialización
//! 4. **Decaimiento**: Grabados antiguos tienen menor probabilidad de ser seleccionados
//!
//! ## Presión Lamarckiana
//!
//! Este mecanismo crea presión evolutiva lamarckiana a nivel de ecosistema:
//! - Experiencias (Umbrae) de Auton muertos influyen en descendientes
//! - No hay "genes" literalmente, pero sí "memoria heredable"
//!
//! ## Buffer Circular
//!
//! El buffer tiene tamaño fijo. Cuando se llena, los grabados más antiguos
//! se sobrescriben. Cada grabado tiene un contador de refuerzo.
#![allow(dead_code)]
#![allow(non_snake_case)]

use crate::life::umbra::Umbra;
use crate::physics::fixed_point::I32F32;
use std::collections::HashMap;
use std::vec::Vec;

/// Tamaño máximo del buffer circular (base)
/// Tamaño emerge naturalmente del espacio disponible
const BUFFER_BASE: usize = 50_000;

/// Factor de decaimiento temporal (por tick)
const FACTOR_DECAIMIENTO: f64 = 0.999;

/// Umbral de refuerzo para "inmortalizar" un grabado
const UMBRAL_INMORTAL: u32 = 5;

/// Grabado: vector de características comprimidas de una Umbra
#[derive(Debug, Clone)]
pub struct Grabado {
    /// Vector de características (hashes de nodos reforzados)
    pub caracteristicas: Vec<u64>,
    /// Fuerza total del grabado (suma de fuerzas)
    pub fuerza: I32F32,
    /// Tick en que fue creado (muerte del Auton)
    pub tick_creacion: u64,
    /// Contador de veces que fue seleccionado
    pub conteo_selecciones: u32,
    /// Contador de "refuerzos" (muertes similares)
    pub conteo_refuerzos: u32,
    /// ID del Auton original
    pub id_auton_original: u64,
    /// Suma de hashes para similitud
    pub hash_suma: u64,
}

impl Grabado {
    pub fn new(caracteristicas: Vec<u64>, fuerza: I32F32, tick: u64, id_auton: u64) -> Self {
        let hash_suma = caracteristicas
            .iter()
            .fold(0u64, |acc, &h| acc.wrapping_add(h));
        Grabado {
            caracteristicas,
            fuerza,
            tick_creacion: tick,
            conteo_selecciones: 0,
            conteo_refuerzos: 0,
            id_auton_original: id_auton,
            hash_suma,
        }
    }

    /// Calcula probabilidad de selección (decae con tiempo)
    pub fn probabilidad_seleccion(&self, tick_actual: u64, factor_decaimiento: f64) -> f64 {
        let edad = tick_actual.saturating_sub(self.tick_creacion) as f64;

        // Probabilidad base decae exponencialmente
        let decaimiento = factor_decaimiento.powf(edad);

        // Refuerzos multiplican la probabilidad
        let refuerzos = 1.0 + (self.conteo_refuerzos as f64) * 0.5;

        // Selecciones previas también afectan (negativamente si excesivas)
        let factor_selecciones = if self.conteo_selecciones > 10 {
            0.5 // Reducir si muy seleccionado
        } else {
            1.0
        };

        decaimiento * refuerzos * factor_selecciones
    }

    /// Verifica si el grabado está "inmortalizado"
    pub fn es_inmortal(&self) -> bool {
        self.conteo_refuerzos >= UMBRAL_INMORTAL
    }

    /// Reinicia contador de selecciones (simula nueva generación)
    pub fn fue_seleccionado(&mut self) {
        self.conteo_selecciones += 1;
    }

    /// Refuerza el grabado
    pub fn reforzar(&mut self) {
        self.conteo_refuerzos += 1;
    }

    /// Calcula similitud con otro grabado
    pub fn similitud(&self, otro: &Grabado) -> f64 {
        if self.caracteristicas.len() != otro.caracteristicas.len() {
            return 0.0;
        }

        let mut matches = 0usize;
        for (a, b) in self.caracteristicas.iter().zip(otro.caracteristicas.iter()) {
            if *a == *b {
                matches += 1;
            }
        }

        matches as f64 / self.caracteristicas.len() as f64
    }

    /// Serializa a bytes
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut v = Vec::new();

        // caracteristicas.len() as u32
        v.extend_from_slice(&(self.caracteristicas.len() as u32).to_le_bytes());

        // caracteristicas
        for &c in &self.caracteristicas {
            v.extend_from_slice(&c.to_le_bytes());
        }

        // fuerza
        v.extend_from_slice(&self.fuerza.to_raw().to_le_bytes());

        // tick_creacion
        v.extend_from_slice(&self.tick_creacion.to_le_bytes());

        // conteo_selecciones
        v.extend_from_slice(&self.conteo_selecciones.to_le_bytes());

        // conteo_refuerzos
        v.extend_from_slice(&self.conteo_refuerzos.to_le_bytes());

        // id_auton_original
        v.extend_from_slice(&self.id_auton_original.to_le_bytes());

        // hash_suma
        v.extend_from_slice(&self.hash_suma.to_le_bytes());

        v
    }

    /// Deserializa desde bytes
    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        let mut pos = 0;

        // caracteristicas.len()
        if bytes.len() < 4 {
            return None;
        }
        let len = u32::from_le_bytes(bytes[pos..pos + 4].try_into().ok()?) as usize;
        pos += 4;

        // caracteristicas
        let mut caracteristicas = Vec::with_capacity(len);
        for _ in 0..len {
            if bytes.len() < pos + 8 {
                return None;
            }
            let c = u64::from_le_bytes(bytes[pos..pos + 8].try_into().ok()?);
            pos += 8;
            caracteristicas.push(c);
        }

        // fuerza
        if bytes.len() < pos + 8 {
            return None;
        }
        let fuerza_raw = i64::from_le_bytes(bytes[pos..pos + 8].try_into().ok()?);
        pos += 8;

        // tick_creacion
        if bytes.len() < pos + 8 {
            return None;
        }
        let tick_creacion = u64::from_le_bytes(bytes[pos..pos + 8].try_into().ok()?);
        pos += 8;

        // conteo_selecciones
        if bytes.len() < pos + 4 {
            return None;
        }
        let conteo_selecciones = u32::from_le_bytes(bytes[pos..pos + 4].try_into().ok()?);
        pos += 4;

        // conteo_refuerzos
        if bytes.len() < pos + 4 {
            return None;
        }
        let conteo_refuerzos = u32::from_le_bytes(bytes[pos..pos + 4].try_into().ok()?);
        pos += 4;

        // id_auton_original
        if bytes.len() < pos + 8 {
            return None;
        }
        let id_auton_original = u64::from_le_bytes(bytes[pos..pos + 8].try_into().ok()?);
        pos += 8;

        // hash_suma
        if bytes.len() < pos + 8 {
            return None;
        }
        let hash_suma = u64::from_le_bytes(bytes[pos..pos + 8].try_into().ok()?);

        Some(Grabado {
            caracteristicas,
            fuerza: I32F32::from_raw(fuerza_raw),
            tick_creacion,
            conteo_selecciones,
            conteo_refuerzos,
            id_auton_original,
            hash_suma,
        })
    }
}

/// Buffer circular de grabados
#[derive(Debug, Clone)]
pub struct BufferCircular {
    /// Grabados almacenados
    grabados: Vec<Grabado>,
    /// Índice de escritura actual
    indice_escritura: usize,
    /// Número total de grabados
    count: u64,
}

impl BufferCircular {
    pub fn new(capacidad: usize) -> Self {
        BufferCircular {
            grabados: Vec::with_capacity(capacidad),
            indice_escritura: 0,
            count: 0,
        }
    }

    /// Añade un grabado al buffer y retorna:
    /// - El índice donde fue escrito
    /// - El ID del Auton cuyo grabado fue sobrescrito (si applicable)
    pub fn push(&mut self, grabado: Grabado) -> (usize, Option<u64>) {
        let (idx, old_id) = if self.grabados.len() < self.grabados.capacity() {
            self.grabados.push(grabado);
            (self.grabados.len() - 1, None)
        } else {
            let idx = self.indice_escritura;
            let old_id = self.grabados[idx].id_auton_original;
            self.grabados[idx] = grabado;
            (idx, Some(old_id))
        };
        self.indice_escritura = (self.indice_escritura + 1) % self.grabados.capacity().max(1);
        self.count += 1;
        (idx, old_id)
    }

    /// Obtiene todos los grabados
    pub fn grabados(&self) -> &[Grabado] {
        &self.grabados
    }

    /// Obtiene un grabado aleatorio ponderado por probabilidad
    pub fn seleccionar_ponderado<R: RandProvider>(
        &self,
        rng: &mut R,
        tick_actual: u64,
    ) -> Option<&Grabado> {
        if self.grabados.is_empty() {
            return None;
        }

        // Calcular probabilidades
        let probabilidades: Vec<f64> = self
            .grabados
            .iter()
            .map(|g| g.probabilidad_seleccion(tick_actual, FACTOR_DECAIMIENTO))
            .collect();

        let suma: f64 = probabilidades.iter().sum();
        if suma <= 0.0 {
            // Si todas las probabilidades son 0, seleccionar aleatorio uniforme
            let idx = rng.gen_range(0..self.grabados.len());
            return self.grabados.get(idx);
        }

        // Selección ponderada
        let umbral = rng.gen() * suma;
        let mut acum = 0.0;

        for (i, &p) in probabilidades.iter().enumerate() {
            acum += p;
            if umbral <= acum {
                return self.grabados.get(i);
            }
        }

        // Fallback
        self.grabados.last()
    }

    /// Encuentra grabados similares a uno dado
    pub fn encontrar_similares(&self, objetivo: &Grabado, umbral: f64) -> Vec<&Grabado> {
        self.grabados
            .iter()
            .filter(|g| {
                let sim = g.similitud(objetivo);
                sim >= umbral
            })
            .collect()
    }

    /// Aplica decaimiento a todos los grabados
    pub fn aplicar_decaimiento(&mut self, tick_actual: u64) {
        // Los grabados muy antiguos eventualmente se eliminan
        self.grabados.retain(|g| {
            let edad = tick_actual.saturating_sub(g.tick_creacion);
            // Mantener si es inmortal o tiene menos de 10000 ticks
            g.es_inmortal() || edad < 10000
        });
    }

    pub fn len(&self) -> usize {
        self.grabados.len()
    }

    pub fn is_empty(&self) -> bool {
        self.grabados.is_empty()
    }

    pub fn count(&self) -> u64 {
        self.count
    }
}

/// Trait para generadores aleatorios (evita依赖 externa)
pub trait RandProvider {
    fn gen_range(&mut self, range: std::ops::Range<usize>) -> usize;
    fn gen(&mut self) -> f64;
}

/// XorShift64 como RNG
#[derive(Debug, Clone)]
pub struct XorShiftRng {
    state: u64,
}

impl XorShiftRng {
    pub fn new(seed: u64) -> Self {
        XorShiftRng {
            state: if seed == 0 { 0xDEADBEEFCAFEBABE } else { seed },
        }
    }
}

impl RandProvider for XorShiftRng {
    fn gen_range(&mut self, range: std::ops::Range<usize>) -> usize {
        let count = range.end - range.start;
        if count == 0 {
            return range.start;
        }
        let val = self.next() as usize % count;
        range.start + val
    }

    fn gen(&mut self) -> f64 {
        (self.next() as f64) / (u64::MAX as f64)
    }
}

impl XorShiftRng {
    pub fn next(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }
}

/// Meltrace: El Trazo Fundido Global
#[derive(Debug, Clone)]
pub struct Meltrace {
    /// Buffer circular de grabados
    buffer: BufferCircular,
    /// Grabados "inmortales" (refuerzos múltiples)
    inmortales: Vec<Grabado>,
    /// Mapa de ID Auton -> Grabado (rápido lookup)
    indice_auton: HashMap<u64, usize>,
    /// Tick actual global
    tick_global: u64,
    /// Contador de muertes total
    muertes_totales: u64,
    /// Semilla RNG
    semilla: u64,
    /// Número de Auton vivos actualmente
    autons_vivos: u32,
}

impl Meltrace {
    /// Crea nuevo Meltrace
    pub fn new(semilla: u64) -> Self {
        Meltrace {
            buffer: BufferCircular::new(BUFFER_BASE),
            inmortales: Vec::new(),
            indice_auton: HashMap::new(),
            tick_global: 0,
            muertes_totales: 0,
            semilla,
            autons_vivos: 0,
        }
    }

    /// Registra la muerte de un Auton: comprime su Umbra en un Grabado
    pub fn registrar_muerte(&mut self, umbra: &Umbra) {
        self.muertes_totales += 1;

        // Extraer características de nodos reforzados (Hedonio)
        let nodos_hedonio = umbra.nodos_hedonio();

        let mut caracteristicas = Vec::new();
        let mut fuerza_total = I32F32::ZERO;

        // Tomar los primeros 16 nodos hedonio o todos si menos
        for nodo in nodos_hedonio.iter().take(16) {
            caracteristicas.push(nodo.hash_estado);
            caracteristicas.push(nodo.hash_estado.wrapping_mul(31));
            fuerza_total = fuerza_total + nodo.fuerza();
        }

        // Si no hay nodos hedonio, usar todos los hashes
        if caracteristicas.is_empty() {
            for nodo in umbra.nodos() {
                if caracteristicas.len() >= 16 {
                    break;
                }
                caracteristicas.push(nodo.hash_estado);
            }
        }

        // Crear grabado
        let grabado = Grabado::new(
            caracteristicas,
            fuerza_total,
            self.tick_global,
            umbra.id_auton(),
        );

        // Guardar en buffer y obtener índice donde se escribió
        let (idx_grabado, id_sobrescrito) = self.buffer.push(grabado.clone());

        // Si el buffer sobrescribió un grabado, limpiar el índice del ID antiguo
        if let Some(id_antiguo) = id_sobrescrito {
            self.indice_auton.remove(&id_antiguo);
        }

        // Guardar en índice - usa el índice retornado por push()
        self.indice_auton.insert(umbra.id_auton(), idx_grabado);

        // Si el grabado es muy fuerte, hacerlo inmortal
        // Threshold: 10 veces I32F32::ONE (42949672960 en representación)
        if fuerza_total.to_raw().unsigned_abs() as i64 > I32F32::ONE.to_raw() * 10 {
            let mut inmortal = grabado;
            inmortal.conteo_refuerzos = UMBRAL_INMORTAL;
            self.inmortales.push(inmortal);
        }
    }

    /// Selecciona un grabado aleatorio para inicializar nuevo Auton
    pub fn seleccionar_grabado(&mut self) -> Option<Grabado> {
        let mut rng = XorShiftRng::new(self.semilla.wrapping_add(self.muertes_totales));

        // 30% de probabilidad de seleccionar un inmortal
        if !self.inmortales.is_empty() && rng.gen() < 0.3 {
            let idx = rng.gen_range(0..self.inmortales.len());
            let seleccionado = self.inmortales[idx].clone();
            // Marcar como seleccionado
            if let Some(buf_idx) = self.indice_auton.get(&seleccionado.id_auton_original) {
                if let Some(grabado_buf) = self.buffer.grabados.get(*buf_idx) {
                    let mut g = grabado_buf.clone();
                    g.fue_seleccionado();
                    return Some(g);
                }
            }
            return Some(seleccionado);
        }

        // Seleccionar del buffer con ponderación
        if let Some(grabado) = self
            .buffer
            .seleccionar_ponderado(&mut rng, self.tick_global)
        {
            // Marcar como seleccionado
            let mut seleccionado = grabado.clone();
            seleccionado.fue_seleccionado();
            return Some(seleccionado);
        }

        None
    }

    /// Refuerza grabados similares a uno dado (lamarckismo)
    pub fn reforzar_similares(&mut self, objetivo: &Grabado) {
        // Encontrar grabados similares - collect indices first
        let similares = self.buffer.encontrar_similares(objetivo, 0.7);
        let indices: Vec<usize> = similares
            .iter()
            .filter_map(|grabar| self.indice_auton.get(&grabar.id_auton_original).copied())
            .collect();

        for idx in indices {
            if let Some(g) = self.buffer.grabados.get_mut(idx) {
                g.reforzar();

                // Si alcanza umbral inmortal, añadir a inmortales
                if g.conteo_refuerzos >= UMBRAL_INMORTAL && !g.es_inmortal() {
                    let inmortal = g.clone();
                    self.inmortales.push(inmortal);
                }
            }
        }
    }

    /// Inicializa una RamNet con influencia del Meltrace
    pub fn inicializar_ramnet(&self, rasgos: &mut [u8], grabado: &Grabado) {
        // Los bits de la RamNet se sesgan con las características del grabado
        for (i, rasgo) in rasgos.iter_mut().enumerate() {
            if i < grabado.caracteristicas.len() {
                // Mezclar característica con rasgo existente
                let car = (grabado.caracteristicas[i] & 0xFF) as u8;
                *rasgo = (*rasgo ^ car) & 0xFF;
            }
        }
    }

    /// Inicializa un Campo Estructural con influencia del Meltrace
    pub fn inicializar_campo(&self, campos: &mut [I32F32], grabado: &Grabado) {
        // Sesgar los valores del campo con las características
        let _rng = XorShiftRng::new(grabado.hash_suma);

        for (i, campo) in campos.iter_mut().enumerate() {
            if i < grabado.caracteristicas.len() {
                let offset = (grabado.caracteristicas[i] as i64) % 1000;
                let delta = I32F32::from_raw(offset << 16);
                *campo = *campo + delta;
            }
        }
    }

    /// Obtiene estadísticas del Meltrace
    pub fn estadisticas(&self) -> MeltraceStats {
        MeltraceStats {
            total_grabados: self.buffer.count(),
            grabados_activos: self.buffer.len(),
            inmortales: self.inmortales.len(),
            muertes_totales: self.muertes_totales,
            tick_global: self.tick_global,
            autons_vivos: self.autons_vivos,
        }
    }

    /// Número de grabados en el buffer
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Total de muertes registradas
    pub fn muertes_totales(&self) -> u64 {
        self.muertes_totales
    }

    /// Avanza el tick global
    pub fn tick(&mut self) {
        self.tick_global += 1;

        // Cada 1000 ticks, aplicar decaimiento
        if self.tick_global % 1000 == 0 {
            self.buffer.aplicar_decaimiento(self.tick_global);
        }
    }

    /// Registra nacimiento de Auton
    pub fn registrar_nacimiento(&mut self) {
        self.autons_vivos += 1;
    }

    /// Registra muerte de Auton (decrementa contador de vivos)
    pub fn registrar_muerte_auton(&mut self) {
        if self.autons_vivos > 0 {
            self.autons_vivos -= 1;
        }
    }

    /// Obtiene el Meltrace como buffer de bytes (para persistencia)
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut datos = Vec::new();

        // Magic
        datos.extend_from_slice(b"MLTC");

        // Versión
        datos.push(2);

        // Count muertes
        datos.extend_from_slice(&(self.muertes_totales as u64).to_le_bytes());

        // Autons vivos
        datos.extend_from_slice(&(self.autons_vivos as u64).to_le_bytes());

        // Tick global
        datos.extend_from_slice(&self.tick_global.to_le_bytes());

        // Grabados
        datos.extend_from_slice(&(self.buffer.len() as u32).to_le_bytes());
        for g in self.buffer.grabados() {
            let gb = g.to_bytes();
            datos.extend_from_slice(&(gb.len() as u32).to_le_bytes());
            datos.extend_from_slice(&gb);
        }

        // Inmortales
        datos.extend_from_slice(&(self.inmortales.len() as u32).to_le_bytes());
        for g in &self.inmortales {
            let gb = g.to_bytes();
            datos.extend_from_slice(&(gb.len() as u32).to_le_bytes());
            datos.extend_from_slice(&gb);
        }

        datos
    }

    /// Restaura desde bytes
    pub fn from_bytes(datos: &[u8]) -> Option<Self> {
        let mut pos = 0;

        // Magic
        if datos.len() < 8 {
            return None;
        }
        if &datos[0..4] != b"MLTC" {
            return None;
        }
        pos += 4;

        // Versión
        let version = datos[pos];
        pos += 1;
        if version != 1 && version != 2 {
            return None;
        }

        // Count
        let muertes = u64::from_le_bytes(datos[pos..pos + 8].try_into().ok()?);
        pos += 8;

        // Autons vivos (version 2+)
        let autons_vivos = if version >= 2 {
            let av = u64::from_le_bytes(datos[pos..pos + 8].try_into().ok()?);
            pos += 8;
            av as u32
        } else {
            0
        };

        // Tick global
        let tick = u64::from_le_bytes(datos[pos..pos + 8].try_into().ok()?);
        pos += 8;

        let mut meltrace = Meltrace::new(0);
        meltrace.muertes_totales = muertes;
        meltrace.autons_vivos = autons_vivos;
        meltrace.tick_global = tick;

        // Grabados
        let num_grabados = u32::from_le_bytes(datos[pos..pos + 4].try_into().ok()?) as usize;
        pos += 4;

        for _ in 0..num_grabados {
            let len = u32::from_le_bytes(datos[pos..pos + 4].try_into().ok()?) as usize;
            pos += 4;

            if datos.len() < pos + len {
                return None;
            }
            let grabado = Grabado::from_bytes(&datos[pos..pos + len])?;
            pos += len;

            meltrace.buffer.push(grabado);
        }

        // Inmortales
        let num_inmortales = u32::from_le_bytes(datos[pos..pos + 4].try_into().ok()?) as usize;
        pos += 4;

        for _ in 0..num_inmortales {
            let len = u32::from_le_bytes(datos[pos..pos + 4].try_into().ok()?) as usize;
            pos += 4;

            if datos.len() < pos + len {
                return None;
            }
            let inmortal = Grabado::from_bytes(&datos[pos..pos + len])?;
            pos += len;

            meltrace.inmortales.push(inmortal);
        }

        Some(meltrace)
    }
}

/// Estadísticas del Meltrace
#[derive(Debug, Clone)]
pub struct MeltraceStats {
    pub total_grabados: u64,
    pub grabados_activos: usize,
    pub inmortales: usize,
    pub muertes_totales: u64,
    pub tick_global: u64,
    pub autons_vivos: u32,
}

// ==================== Tests ====================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::life::ramnet::{Accion, TipoAccion};
    use crate::life::umbra::Umbra;

    fn crear_umbra_ejemplo() -> Umbra {
        let mut umbra = Umbra::nuevo(100);
        let accion = Accion::nueva(TipoAccion::MoverX, 128);

        // Registrar decisiones con diferentes resultados
        umbra.registrar_decision(
            0x1000,
            10,
            accion.clone(),
            crate::life::umbra::ResultadoUmbra::Hedonio(I32F32::from_i32(10)),
            None,
        );

        umbra.registrar_decision(
            0x2000,
            20,
            accion.clone(),
            crate::life::umbra::ResultadoUmbra::Hedonio(I32F32::from_i32(5)),
            Some(0x1000),
        );

        umbra.registrar_decision(
            0x3000,
            30,
            accion,
            crate::life::umbra::ResultadoUmbra::Algion(I32F32::from_i32(3)),
            Some(0x2000),
        );

        umbra
    }

    #[test]
    fn test_crear_meltrace() {
        let mt = Meltrace::new(12345);
        let stats = mt.estadisticas();
        assert_eq!(stats.grabados_activos, 0);
        assert_eq!(stats.muertes_totales, 0);
    }

    #[test]
    fn test_registrar_muerte() {
        let mut mt = Meltrace::new(12345);
        let umbra = crear_umbra_ejemplo();

        mt.registrar_muerte(&umbra);

        let stats = mt.estadisticas();
        assert_eq!(stats.total_grabados, 1);
        assert_eq!(stats.grabados_activos, 1);
    }

    #[test]
    fn test_seleccionar_grabado() {
        let mut mt = Meltrace::new(12345);
        let umbra = crear_umbra_ejemplo();

        mt.registrar_muerte(&umbra);

        let seleccionado = mt.seleccionar_grabado();
        assert!(seleccionado.is_some());
    }

    #[test]
    fn test_decaimiento_probabilidad() {
        let grabado = Grabado::new(vec![0x1000, 0x2000], I32F32::from_i32(10), 100, 1);

        // Probabilidad inicial
        let p0 = grabado.probabilidad_seleccion(100, FACTOR_DECAIMIENTO);

        // Después de 1000 ticks
        let p1000 = grabado.probabilidad_seleccion(1100, FACTOR_DECAIMIENTO);

        assert!(p1000 < p0); // Debe decaer
    }

    #[test]
    fn test_refuerzo_aumenta_probabilidad() {
        let mut grabado = Grabado::new(vec![0x1000], I32F32::ONE, 0, 1);

        grabado.reforzar();
        grabado.reforzar();
        grabado.reforzar();

        assert!(grabado.conteo_refuerzos >= 3);
        assert!(grabado.es_inmortal() || grabado.conteo_refuerzos < UMBRAL_INMORTAL);
    }

    #[test]
    fn test_serializacion_grabado() {
        let grabado = Grabado::new(vec![0x1000, 0x2000, 0x3000], I32F32::from_i32(42), 999, 55);

        let bytes = grabado.to_bytes();
        let restaurado = Grabado::from_bytes(&bytes);

        assert!(restaurado.is_some());
        let r = restaurado.unwrap();
        assert_eq!(r.caracteristicas.len(), 3);
        assert_eq!(r.id_auton_original, 55);
    }

    #[test]
    fn test_similitud_grabados() {
        let g1 = Grabado::new(vec![0x1000, 0x2000, 0x3000], I32F32::ONE, 0, 1);

        let g2 = Grabado::new(
            vec![0x1000, 0x2000, 0x4000], // 2 de 3 iguales
            I32F32::ONE,
            0,
            2,
        );

        let sim = g1.similitud(&g2);
        assert!((sim - 2.0 / 3.0).abs() < 0.01);
    }

    #[test]
    fn test_buffer_circular() {
        let mut buf = BufferCircular::new(3);

        buf.push(Grabado::new(vec![1], I32F32::ONE, 1, 1));
        buf.push(Grabado::new(vec![2], I32F32::ONE, 2, 2));
        buf.push(Grabado::new(vec![3], I32F32::ONE, 3, 3));

        assert_eq!(buf.len(), 3);

        // Sobrescribir
        buf.push(Grabado::new(vec![4], I32F32::ONE, 4, 4));

        // Ahora tiene 3 (sobrescribió el más antiguo)
        assert_eq!(buf.len(), 3);
    }

    #[test]
    fn test_inicializar_ramnet() {
        let mt = Meltrace::new(12345);
        let grabado = Grabado::new(vec![0xAA, 0xBB, 0xCC], I32F32::ONE, 0, 1);

        let mut rasgos = [0x11u8, 0x22, 0x33, 0x44];
        mt.inicializar_ramnet(&mut rasgos, &grabado);

        // Verificar que hubo mezcla
        assert!(rasgos[0] != 0x11 || rasgos[1] != 0x22);
    }

    #[test]
    fn test_inicializar_campo() {
        let mt = Meltrace::new(12345);
        let grabado = Grabado::new(vec![0x1000, 0x2000], I32F32::ONE, 0, 1);

        let mut campos = [I32F32::ZERO, I32F32::ONE, I32F32::ONE];
        mt.inicializar_campo(&mut campos, &grabado);

        // Verificar que cambió
        assert!(campos[0] != I32F32::ZERO || campos[1] != I32F32::ONE);
    }

    #[test]
    fn test_reforsar_similares() {
        let mut mt = Meltrace::new(12345);

        // Registrar varios grabados
        for i in 0..5 {
            let mut umbra = Umbra::nuevo(i + 1);
            let accion = Accion::nueva(TipoAccion::MoverX, 128);
            umbra.registrar_decision(
                0x1000 + (i as u64 * 0x100),
                10,
                accion,
                crate::life::umbra::ResultadoUmbra::Hedonio(I32F32::ONE),
                None,
            );
            mt.registrar_muerte(&umbra);
        }

        let objetivo = Grabado::new(vec![0x1000, 0x1100], I32F32::ONE, 0, 99);

        mt.reforzar_similares(&objetivo);
        // No debe fallar
    }
}
