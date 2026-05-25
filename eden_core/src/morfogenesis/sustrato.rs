//! # SustratoVital - Memoria Cruda para el Cerebro de Eden
//!
//! Espacio de memoria lineal ultra-eficiente para morfogénesis celular.
//! Sin abstracciones de matriz 2D - mapeo directo (x, y, z) → índice plano.
//!
//! # Sistema de Flujo de Resonancia Química
//! Cada célula Nervio tiene una carga (u8 0-255) y la TablaSinaptica
//! controla el flujo de carga entre células adyacentes.
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::HashMap;
use crate::consciousness::NivelTermico;

/// Estados celulares - exactamente 1 byte cada uno
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EstadoCelular {
    Vacio = 0,
    NervioPrimario = 1,
    NervioSecundario = 2,
    SoporteEnergético = 3,
}

impl Default for EstadoCelular {
    fn default() -> Self {
        EstadoCelular::Vacio
    }
}

/// SustratoVital: memoria lineal masiva para el cerebro de Eden
pub struct SustratoVital {
    ancho: usize,
    alto: usize,
    profundidad: usize,
    /// Datos crudos - vector plano de estados celulares
    datos: Vec<EstadoCelular>,
    /// Carga de cada célula (0-255) - solo significativo para Nervio
    cargas: Vec<u8>,
}

impl SustratoVital {
    /// Crear nuevo sustrato con dimensiones especificadas
    pub fn new(ancho: usize, alto: usize, profundidad: usize) -> Self {
        let capacidad = ancho * alto * profundidad;
        Self {
            ancho,
            alto,
            profundidad,
            datos: vec![EstadoCelular::Vacio; capacidad],
            cargas: vec![0u8; capacidad],
        }
    }

    /// Crear sustrato con tamaño preallocado (sin inicializar a Vacio)
    pub fn with_capacity(ancho: usize, alto: usize, profundidad: usize) -> Self {
        let capacidad = ancho * alto * profundidad;
        Self {
            ancho,
            alto,
            profundidad,
            datos: Vec::with_capacity(capacidad),
            cargas: Vec::with_capacity(capacidad),
        }
    }

    /// Tamaño total en bytes aproximados (un byte por celda)
    pub fn len_bytes(&self) -> usize {
        self.datos.len()
    }

    /// Número de celdas totales
    pub fn len(&self) -> usize {
        self.datos.len()
    }

    /// Vérifica si está vacío
    pub fn is_empty(&self) -> bool {
        self.datos.is_empty()
    }

    /// Dimensiones del sustrato
    pub fn dimensiones(&self) -> (usize, usize, usize) {
        (self.ancho, self.alto, self.profundidad)
    }

    /// Mapeo (x, y, z) → índice plano
    /// i = x + (y * ancho) + (z * ancho * alto)
    #[inline]
    pub fn index(&self, x: usize, y: usize, z: usize) -> Option<usize> {
        if x >= self.ancho || y >= self.alto || z >= self.profundidad {
            return None;
        }
        Some(x + y * self.ancho + z * self.ancho * self.alto)
    }

    /// Obtener estado en coordenadas (x, y, z)
    #[inline]
    pub fn get(&self, x: usize, y: usize, z: usize) -> Option<EstadoCelular> {
        self.index(x, y, z).map(|i| self.datos[i])
    }

    /// Establecer estado en coordenadas (x, y, z)
    #[inline]
    pub fn set(&mut self, x: usize, y: usize, z: usize, estado: EstadoCelular) -> bool {
        if let Some(i) = self.index(x, y, z) {
            self.datos[i] = estado;
            true
        } else {
            false
        }
    }

    /// Obtener estado en índice plano
    #[inline]
    pub fn get_index(&self, idx: usize) -> Option<EstadoCelular> {
        self.datos.get(idx).copied()
    }

    /// Establecer estado en índice plano
    #[inline]
    pub fn set_index(&mut self, idx: usize, estado: EstadoCelular) -> bool {
        if idx < self.datos.len() {
            self.datos[idx] = estado;
            true
        } else {
            false
        }
    }

    /// Obtener carga en coordenadas (x, y, z)
    #[inline]
    pub fn carga(&self, x: usize, y: usize, z: usize) -> Option<u8> {
        self.index(x, y, z).map(|i| self.cargas[i])
    }

    /// Establecer carga en coordenadas (x, y, z)
    #[inline]
    pub fn set_carga(&mut self, x: usize, y: usize, z: usize, valor: u8) -> bool {
        if let Some(i) = self.index(x, y, z) {
            self.cargas[i] = valor;
            true
        } else {
            false
        }
    }

    /// Obtener carga en índice plano
    #[inline]
    pub fn carga_index(&self, idx: usize) -> Option<u8> {
        self.cargas.get(idx).copied()
    }

    /// Establecer carga en índice plano
    #[inline]
    pub fn set_carga_index(&mut self, idx: usize, valor: u8) -> bool {
        if idx < self.cargas.len() {
            self.cargas[idx] = valor;
            true
        } else {
            false
        }
    }

    /// Obtener puntero mutable a los datos crudos (para simbiogénesis unsafe)
    /// 
    /// # Advertencia
    /// Este método es unsafe por diseño - permite acceso directo a memoria
    /// para implementar Transferencia Genética Horizontal entre Autons.
    pub unsafe fn datos_ptr(&mut self) -> *mut Vec<EstadoCelular> {
        &mut self.datos as *mut Vec<EstadoCelular>
    }

    /// Obtener puntero mutable a las cargas (para simbiogénesis unsafe)
    pub unsafe fn cargas_ptr(&mut self) -> *mut Vec<u8> {
        &mut self.cargas as *mut Vec<u8>
    }

    /// Índices de los 26 vecinos (cubo 3x3x3 excluyendo el centro)
    /// Orden: iteración en capas (xy plano, luego z)
    #[inline]
    pub fn vecinos_26(&self, x: usize, y: usize, z: usize) -> Vec<usize> {
        let mut idxs = Vec::with_capacity(26);
        for dz in -1..=1 {
            for dy in -1..=1 {
                for dx in -1..=1 {
                    if dx == 0 && dy == 0 && dz == 0 {
                        continue;
                    }
                    let nx = x as isize + dx;
                    let ny = y as isize + dy;
                    let nz = z as isize + dz;
                    if nx >= 0 && nx < self.ancho as isize
                        && ny >= 0 && ny < self.alto as isize
                        && nz >= 0 && nz < self.profundidad as isize
                    {
                        if let Some(i) = self.index(
                            nx as usize, ny as usize, nz as usize
                        ) {
                            idxs.push(i);
                        }
                    }
                }
            }
        }
        idxs
    }

    /// Conteo de vecinos por tipo específico
    pub fn contar_vecinos(&self, x: usize, y: usize, z: usize, tipo: EstadoCelular) -> usize {
        self.vecinos_26(x, y, z)
            .iter()
            .filter(|&&i| self.datos[i] == tipo)
            .count()
    }

    /// Verificar si la zona tiene temperatura baja (baja conciencia_termica)
    /// Usa el trait TermicoReader para consultar el sistema de conciencia
    fn temperatura_baja<T: TermicoReader>(&self, x: usize, y: usize, z: usize, lector: &T) -> bool {
        // Mapear coordenadas 3D a una zona de temperatura
        // Dividimos el sustrato en regiones para consultar la temperatura
        let region_x = x / (self.ancho / 4).max(1);
        let region_y = y / (self.alto / 4).max(1);
        let region_z = z / (self.profundidad / 4).max(1);
        let zona = (region_x + region_y * 4 + region_z * 16) % 64;
        lector.temperatura_zona(zona) == NivelTermico::Optimo
            || lector.temperatura_zona(zona) == NivelTermico::Normal
    }
}

/// Trait para leer datos de temperatura/conciencia_termica desde el módulo consciousness
pub trait TermicoReader: Send + Sync {
    /// Obtener nivel térmico de una zona específica (0-63)
    fn temperatura_zona(&self, zona: usize) -> NivelTermico;

    /// Obtener temperatura global del sistema
    fn temperatura_global(&self) -> NivelTermico;
}

/// Iterador plano para recorrido eficiente del sustrato
pub struct IterSustrato<'a> {
    sustrato: &'a SustratoVital,
    idx: usize,
}

impl<'a> Iterator for IterSustrato<'a> {
    type Item = (usize, EstadoCelular);

    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.sustrato.datos.len() {
            let idx = self.idx;
            let estado = self.sustrato.datos[idx];
            self.idx += 1;
            Some((idx, estado))
        } else {
            None
        }
    }
}

impl<'a> IntoIterator for &'a SustratoVital {
    type Item = (usize, EstadoCelular);
    type IntoIter = IterSustrato<'a>;

    fn into_iter(self) -> Self::IntoIter {
        IterSustrato { sustrato: self, idx: 0 }
    }
}

impl SustratoVital {
    /// Iterar morfogénesis: aplicar reglas de crecimiento celular
    ///
    /// Regla de Tensión Superficial:
    /// - Célula Vacia → NervioPrimario si:
    ///   1. Exactamente 3 vecinos NervioPrimario
    ///   2. Conciencia térmica BAJA en esa zona (temperatura óptima o normal)
    /// - Si la temperatura es ALTA o CRÍTICA, el crecimiento se inhibe
    pub fn iterar_morfogenesis<T: TermicoReader>(&mut self, lector_termico: &T) {
        let mut cambios = Vec::with_capacity(self.datos.len() / 100);

        for z in 0..self.profundidad {
            for y in 0..self.alto {
                for x in 0..self.ancho {
                    let idx = match self.index(x, y, z) {
                        Some(i) => i,
                        None => continue,
                    };

                    let estado_actual = self.datos[idx];

                    // Solo procesamos célidez vacías
                    if estado_actual != EstadoCelular::Vacio {
                        continue;
                    }

                    let nervios_vecinos = self.contar_vecinos(
                        x, y, z, EstadoCelular::NervioPrimario
                    );

                    // Regla: exactamente 3 vecinos nervio primario
                    if nervios_vecinos == 3 {
                        // Verificar temperatura baja (baja conciencia_termica)
                        if self.temperatura_baja(x, y, z, lector_termico) {
                            cambios.push((idx, EstadoCelular::NervioPrimario));
                        }
                    }
                }
            }
        }

        // Aplicar cambios al final (evitar modificar mientras leemos)
        for (idx, nuevo_estado) in cambios {
            self.datos[idx] = nuevo_estado;
        }
    }

    /// Morfogénesis básica sin sensor térmico (para testing)
    pub fn iterar_morfogenesis_simple(&mut self) {
        let mut cambios = Vec::with_capacity(self.datos.len() / 100);

        for z in 0..self.profundidad {
            for y in 0..self.alto {
                for x in 0..self.ancho {
                    let idx = match self.index(x, y, z) {
                        Some(i) => i,
                        None => continue,
                    };

                    if self.datos[idx] != EstadoCelular::Vacio {
                        continue;
                    }

                    let nervios_vecinos = self.contar_vecinos(
                        x, y, z, EstadoCelular::NervioPrimario
                    );

                    if nervios_vecinos == 3 {
                        cambios.push((idx, EstadoCelular::NervioPrimario));
                    }
                }
            }
        }

        for (idx, nuevo_estado) in cambios {
            self.datos[idx] = nuevo_estado;
        }
    }
}

// =============================================================================
// TABLA SINÁPTICA - Flujo de Resonancia Química
// =============================================================================

/// TablaSinaptica: HashMap de fuerzas de flujo entre células Nervio
///
/// Clave: (Índice_Origen, Índice_Destino)
/// Valor: Fuerza del flujo (i16, negativo = inhibidor)
///
/// # Plasticidad
/// - Si la célula B ya estaba cargada (resonancia), se fortalece la conexión
/// - Si B estaba vacía, se debilita (olvido)
#[derive(Clone, Debug, Default)]
pub struct TablaSinaptica {
    flujos: HashMap<(usize, usize), i16>,
}

impl TablaSinaptica {
    /// Crear nueva tabla sináptica vacía
    pub fn new() -> Self {
        Self {
            flujos: HashMap::new(),
        }
    }

    /// Obtener fuerza de flujo entre dos células
    pub fn fuerza(&self, origen: usize, destino: usize) -> i16 {
        self.flujos.get(&(origen, destino)).copied().unwrap_or(0)
    }

    /// Establecer fuerza de flujo directamente
    pub fn set_fuerza(&mut self, origen: usize, destino: usize, valor: i16) {
        self.flujos.insert((origen, destino), valor);
    }

    /// Actualizar plasticidad según si hubo resonancia
    ///
    /// Si `resuena` (B ya estaba cargada): fortalecer +10
    /// Si no resonó (B estaba vacía): debilitar -1
    pub fn actualizar_plasticidad(&mut self, origen: usize, destino: usize, resuena: bool) {
        let entrada = self.flujos.entry((origen, destino)).or_insert(0);
        if resuena {
            *entrada = entrada.saturating_add(10);
        } else {
            *entrada = entrada.saturating_sub(1);
        }
    }

    /// Normalizar todas las fuerzas al rango [-32768, 32767]
    pub fn normalizar(&mut self) {
        for valor in self.flujos.values_mut() {
            *valor = (*valor).clamp(-32768, 32767);
        }
    }

    /// Número de conexiones registradas
    pub fn num_conexiones(&self) -> usize {
        self.flujos.len()
    }

    /// Eliminar conexiones con fuerza cero o negativa muy baja
    pub fn podar_conexiones_debiles(&mut self, umbral: i16) {
        self.flujos.retain(|_, &mut v| v > umbral);
    }

    /// Iterador sobre todas las conexiones
    pub fn iter(&self) -> impl Iterator<Item = ((usize, usize), i16)> + '_ {
        self.flujos.iter().map(|(k, &v)| (*k, v))
    }

    /// Obtener los N flujos más intensos ordenados por fuerza absoluta
    pub fn obtener_top_flujos(&self, n: usize) -> Vec<(usize, usize, i16)> {
        let mut flujos_ordenados: Vec<(usize, usize, i16)> = self
            .flujos
            .iter()
            .map(|(&k, &v)| (k.0, k.1, v))
            .collect();

        // Ordenar por fuerza descendente
        flujos_ordenados.sort_by(|a, b| b.2.cmp(&a.2));

        // Tomar los top N
        flujos_ordenados.truncate(n);
        flujos_ordenados
    }

    /// Pre-cargar un flujo desde una espora (Necromancia Digital)
    pub fn precargar_flujo(&mut self, origen: usize, destino: usize, fuerza: i16) {
        // Solo precargar si la fuerza es positiva
        if fuerza > 0 {
            let actual = self.flujos.entry((origen, destino)).or_insert(0);
            // Solo mejorar si la nueva fuerza es mayor que la actual
            if fuerza > *actual {
                *actual = fuerza;
            }
        }
    }
}

// =============================================================================
// PENSAMIENTO - Flujo de Resonancia Química (El "Pensar")
// =============================================================================

impl SustratoVital {
    /// Iterar pensamiento: distribuir carga entre células Nervio
    ///
    /// Cada célula cargada intenta transferir su carga a vecinos Nervio.
    /// La cantidad transferida depende de la fuerza en TablaSinaptica.
    ///
    /// Retorna: número de transferencias realizadas
    pub fn iterar_pensamiento(&mut self, tabla: &mut TablaSinaptica) -> usize {
        let mut transferencias = 0usize;
        let mut cambios_carga: Vec<(usize, i16)> = Vec::with_capacity(1024);

        // Recorrer todas las células
        for idx in 0..self.datos.len() {
            let estado = self.datos[idx];
            // Solo procesamos células nervio con carga significativa
            if estado != EstadoCelular::NervioPrimario 
                && estado != EstadoCelular::NervioSecundario {
                continue;
            }

            let carga_actual = self.cargas[idx];
            if carga_actual < 32 {
                continue; // Carga muy baja para transferir
            }

            // Obtener coordenadas para encontrar vecinos
            let (x, y, z) = self.coordenadas(idx);
            if x.is_none() { continue; }
            let (x, y, z) = (x.unwrap(), y.unwrap(), z.unwrap());

            // Procesar cada vecino
            for &vecino_idx in &self.vecinos_26(x, y, z) {
                let estado_vecino = self.datos[vecino_idx];
                if estado_vecino != EstadoCelular::NervioPrimario 
                    && estado_vecino != EstadoCelular::NervioSecundario {
                    continue;
                }

                let fuerza = tabla.fuerza(idx, vecino_idx);
                if fuerza <= 0 {
                    continue; // Sin conexión o inhibida
                }

                // Calcular cantidad a transferir (proporcional a fuerza y carga)
                let factor = (fuerza as u16 * carga_actual as u16) / 256;
                let a_transferir = (factor as u8).min(carga_actual);

                if a_transferir < 4 {
                    continue; // Mínimo significativo
                }

                // Verificar si el vecino ya tiene carga (resonancia)
                let carga_vecino = self.cargas[vecino_idx];
                let resuena = carga_vecino > 64;

                // Registrar transferencia
                cambios_carga.push((idx, -(a_transferir as i16)));
                cambios_carga.push((vecino_idx, a_transferir as i16));

                // Actualizar plasticidad según resonancia
                tabla.actualizar_plasticidad(idx, vecino_idx, resuena);

                transferencias += 1;
            }
        }

        // Aplicar cambios de carga
        for (idx, delta) in cambios_carga {
            let nueva_carga = (self.cargas[idx] as i16 + delta).clamp(0, 255);
            self.cargas[idx] = nueva_carga as u8;
        }

        transferencias
    }

    /// Obtener coordenadas (x, y, z) desde índice plano
    fn coordenadas(&self, idx: usize) -> (Option<usize>, Option<usize>, Option<usize>) {
        if idx >= self.datos.len() {
            return (None, None, None);
        }
        let z = idx / (self.ancho * self.alto);
        let resto = idx % (self.ancho * self.alto);
        let y = resto / self.ancho;
        let x = resto % self.ancho;
        (Some(x), Some(y), Some(z))
    }

    /// Versión simple de pensamiento sin tabla sináptica (testing)
    pub fn iterar_pensamiento_simple(&mut self) -> usize {
        let mut transferencias = 0usize;
        let mut cambios_carga: Vec<(usize, i16)> = Vec::with_capacity(1024);

        for idx in 0..self.datos.len() {
            let estado = self.datos[idx];
            if estado != EstadoCelular::NervioPrimario 
                && estado != EstadoCelular::NervioSecundario {
                continue;
            }

            let carga_actual = self.cargas[idx];
            if carga_actual < 64 {
                continue;
            }

            let (x, y, z) = match self.coordenadas(idx) {
                (Some(x), Some(y), Some(z)) => (x, y, z),
                _ => continue,
            };

            for &vecino_idx in &self.vecinos_26(x, y, z) {
                let estado_vecino = self.datos[vecino_idx];
                if estado_vecino == EstadoCelular::Vacio {
                    continue;
                }

                let a_transferir = (carga_actual / 4).min(carga_actual);
                cambios_carga.push((idx, -(a_transferir as i16)));
                cambios_carga.push((vecino_idx, a_transferir as i16));
                transferencias += 1;
            }
        }

        for (idx, delta) in cambios_carga {
            let nueva_carga = (self.cargas[idx] as i16 + delta).clamp(0, 255);
            self.cargas[idx] = nueva_carga as u8;
        }

        transferencias
    }

    /// Inyectar pensamiento (carga) en una zona específica
    pub fn inyectar_pensamiento(&mut self, x: usize, y: usize, z: usize, cantidad: u8) -> bool {
        if let Some(idx) = self.index(x, y, z) {
            if self.datos[idx] == EstadoCelular::NervioPrimario 
                || self.datos[idx] == EstadoCelular::NervioSecundario {
                self.cargas[idx] = self.cargas[idx].saturating_add(cantidad).min(255);
                return true;
            }
        }
        false
    }

    /// Obtener energía total del sustrato (suma de cargas)
    pub fn energia_total(&self) -> u64 {
        self.cargas.iter().map(|&c| c as u64).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_mapping() {
        let s = SustratoVital::new(10, 10, 10);
        // i = x + y*ancho + z*ancho*alto
        assert_eq!(s.index(0, 0, 0), Some(0));
        assert_eq!(s.index(1, 0, 0), Some(1));
        assert_eq!(s.index(0, 1, 0), Some(10));
        assert_eq!(s.index(0, 0, 1), Some(100));
        assert_eq!(s.index(5, 5, 1), Some(5 + 50 + 100)); // 155
    }

    #[test]
    fn test_estado_default() {
        let s = SustratoVital::new(4, 4, 4);
        assert_eq!(s.get(0, 0, 0), Some(EstadoCelular::Vacio));
    }

    #[test]
    fn test_vecinos_26() {
        let s = SustratoVital::new(3, 3, 3);
        // En el centro (1,1,1), debe haber 26 vecinos
        let vecinos = s.vecinos_26(1, 1, 1);
        assert_eq!(vecinos.len(), 26);
    }

    #[test]
    fn test_vecinos_borde() {
        let s = SustratoVital::new(3, 3, 3);
        // En esquina (0,0,0), debe tener menos vecinos
        let vecinos = s.vecinos_26(0, 0, 0);
        assert_eq!(vecinos.len(), 7); // 3x3x3 - 1 - centro = 26 - 1 = 18... no
        // Esquina: solo 7 vecinos posibles (dx,dy,dz ∈ {0,1} excluyendo 0,0,0)
    }

    #[test]
    fn test_contar_vecinos() {
        let mut s = SustratoVital::new(5, 5, 5);
        // Poner nervios alrededor del centro
        s.set(2, 2, 1, EstadoCelular::NervioPrimario);
        s.set(2, 2, 3, EstadoCelular::NervioPrimario);
        s.set(2, 1, 2, EstadoCelular::NervioPrimario);
        
        let conteo = s.contar_vecinos(2, 2, 2, EstadoCelular::NervioPrimario);
        assert_eq!(conteo, 3);
    }

    #[test]
    fn test_morfogenesis_simple() {
        let mut s = SustratoVital::new(5, 5, 5);
        // Configurar germen de nervio
        s.set(2, 2, 2, EstadoCelular::NervioPrimario);
        s.set(3, 2, 2, EstadoCelular::NervioPrimario);
        s.set(2, 3, 2, EstadoCelular::NervioPrimario);
        
        // Una célidez vacía adyacente a 3 nervios
        // En (1,2,2): tiene nervios en (2,2,2), (3,2,2) = 2, falta uno
        // En (2,1,2): tiene nervios en (2,2,2), (2,3,2) = 2, falta uno
        
        s.iterar_morfogenesis_simple();
        
        // No debería cambiar nada porque ninguna célidez tiene exactamente 3 vecinos
        assert_eq!(s.get(1, 2, 2), Some(EstadoCelular::Vacio));
    }

    #[test]
    fn test_tamano_4gb() {
        // 4GB = 4 * 1024 * 1024 * 1024 bytes = 4,294,967,296 bytes
        // Dimensiones aproximadas para ~4GB:
        // 1624^3 = 4,288,722,624 bytes ≈ 3.99GB (4GB exactos requieren 1625^3)

        let ancho = 1625usize;
        let alto = 1625;
        let profundidad = 1625;
        let total = ancho * alto * profundidad;
        let gb_x100 = total / (10_737_418_24 / 100); // bytes to GB * 100
        assert!(gb_x100 >= 399, "Debería ser ~4GB, era {} bytes ({}.{} GB)",
                total, gb_x100 / 100, gb_x100 % 100);
    }

    #[test]
    fn test_carga_celular() {
        let mut s = SustratoVital::new(5, 5, 5);
        s.set(2, 2, 2, EstadoCelular::NervioPrimario);
        s.set_carga(2, 2, 2, 200);
        assert_eq!(s.carga(2, 2, 2), Some(200));
        assert_eq!(s.carga_index(2 + 2 * 5 + 2 * 25), Some(200));
    }

    #[test]
    fn test_tabla_sinaptica_basica() {
        let mut tabla = TablaSinaptica::new();
        assert_eq!(tabla.fuerza(10, 20), 0);

        tabla.set_fuerza(10, 20, 100);
        assert_eq!(tabla.fuerza(10, 20), 100);
        assert_eq!(tabla.fuerza(20, 10), 0); // asimétrica
    }

    #[test]
    fn test_plasticidad_resonancia() {
        let mut tabla = TablaSinaptica::new();

        // Sin resonancia = debilitamiento
        tabla.actualizar_plasticidad(1, 2, false);
        tabla.actualizar_plasticidad(1, 2, false);
        assert_eq!(tabla.fuerza(1, 2), -2);

        // Con resonancia = fortalecimiento
        tabla.actualizar_plasticidad(1, 2, true);
        assert_eq!(tabla.fuerza(1, 2), 8); // -2 + 10 = 8
    }

    #[test]
    fn test_pensamiento_simple() {
        let mut s = SustratoVital::new(5, 5, 5);
        // Crear cadena de nervios para que haya transferencia
        s.set(2, 2, 2, EstadoCelular::NervioPrimario);
        s.set(2, 2, 3, EstadoCelular::NervioPrimario); // vecino en z+1
        s.set(2, 2, 1, EstadoCelular::NervioPrimario); // vecino en z-1
        s.set_carga(2, 2, 2, 200);

        let transferencias = s.iterar_pensamiento_simple();
        assert!(transferencias > 0, "Debe haber transferencias entre nervios adyacentes");

        // Carga debe haberse distribuido
        let energia = s.energia_total();
        assert_eq!(energia, 200); // Conservación de energía
    }

    #[test]
    fn test_inyectar_pensamiento() {
        let mut s = SustratoVital::new(5, 5, 5);
        s.set(2, 2, 2, EstadoCelular::NervioPrimario);

        let exito = s.inyectar_pensamiento(2, 2, 2, 100);
        assert!(exito);
        assert_eq!(s.carga(2, 2, 2), Some(100));

        // En célula vacía debe fallar
        let exito2 = s.inyectar_pensamiento(0, 0, 0, 100);
        assert!(!exito2);
    }

    #[test]
    fn test_pensamiento_con_tabla_sinaptica() {
        let mut s = SustratoVital::new(5, 5, 5);
        let mut tabla = TablaSinaptica::new();

        // Crear dos nervios adyacentes en el plano xy
        s.set(2, 2, 2, EstadoCelular::NervioPrimario);
        s.set(3, 2, 2, EstadoCelular::NervioPrimario); // adyacente en x+1
        s.set_carga(2, 2, 2, 255);
        s.set_carga(3, 2, 2, 100); // vecino ya tiene carga (resonancia)

        let idx1 = 2 + 2 * 5 + 2 * 25;
        let idx2 = 3 + 2 * 5 + 2 * 25;

        // Establecer fuerza de flujo positiva
        tabla.set_fuerza(idx1, idx2, 100);

        let transferencias = s.iterar_pensamiento(&mut tabla);
        assert!(transferencias > 0, "Debe haber transferencia");

        // La fuerza debe haberse incrementado por resonancia
        assert!(tabla.fuerza(idx1, idx2) > 100, "Plasticidad debe fortalecer conexión");
    }

    #[test]
    fn test_podar_conexiones_debiles() {
        let mut tabla = TablaSinaptica::new();
        tabla.set_fuerza(1, 2, 50);
        tabla.set_fuerza(3, 4, -5);
        tabla.set_fuerza(5, 6, 0);

        tabla.podar_conexiones_debiles(10);

        assert_eq!(tabla.num_conexiones(), 1);
        assert!(tabla.fuerza(1, 2) > 0);
    }
}
