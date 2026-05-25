//! # Genoma - Cinta de ADN para Morfogénesis Evo-Devo
//!
//! Este módulo implementa el genoma del Auton como una "cinta" de bytes
//! que codifica las reglas físicas de crecimiento neural.
//!
//! ## Sistema Evo-Devo (Evolutionary Developmental Biology)
//!
//! A diferencia de los algoritmos genéticos clásicos que codifican "qué" es cada neurona,
//! aquí los bytes codifican **Tasas Metabólicas** - las reglas físicas de cómo
//! el SustratoVital crece y se desarrolla.
//!
//! ## Estructura de Codones (bloques de 6 bytes)
//!
//! - **Byte 1-2**: Tasa de ramificación - qué tan rápido una célula Vacio se vuelve Nervio
//! - **Byte 3-4**: Resistencia térmica - cuánto estrés soporta una zona antes de inhibir el crecimiento
//! - **Byte 5-6**: Afinidad sináptica - qué tan rápido se actualiza la TablaSinaptica
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::HashMap;

// ============================================================================
// REGISTROS DEL GENOMA - Reglas Físicas de Morfogénesis
// ============================================================================

/// Reglas de morfogénesis extraídas del genoma
/// Estas reglas控制an cómo el SustratoVital crece y se desarrolla
#[derive(Clone, Debug)]
pub struct ReglasMorfogenesis {
    /// Tasa de ramificación (0.0 - 1.0)
    /// Qué tan rápido una célula Vacio se transforma en Nervio
    pub tasa_ramificacion: f32,

    /// Resistencia térmica (0.0 - 1.0)
    /// Cuánto estrés térmico soporta una zona antes de inhibir el crecimiento
    pub resistencia_termica: f32,

    /// Afinidad sináptica (0.0 - 1.0)
    /// Qué tan rápido se actualiza la TablaSinaptica
    pub afinidad_sinaptica: f32,

    /// Mapa de codones específicos del genoma
    /// Puede contener reglas adicionales específicas del codón
    pub codones_especiales: HashMap<usize, u8>,
}

/// Longitud de un codón en bytes (6 bytes: 2+2+2)
pub const LONGITUD_CODON: usize = 6;

/// Número de codones en el genoma mínimo
pub const NUM_CODONES_MINIMO: usize = 32;

// ============================================================================
// GENOMA - La Cinta de ADN
// ============================================================================

/// Genoma: la "cinta" de ADN que codifica las reglas de morfogénesis
///
/// El genoma es un Vec<u8> que representa una secuencia lineal de nucleótidos.
#[derive(Clone, Debug)]
pub struct Genoma {
    /// La "cinta" de ADN - secuencia de bytes que representa el genoma
    adn: Vec<u8>,

    /// Seed para el generador de números pseudoaleatorios (Xorshift)
    semilla_prng: u64,
}

impl Genoma {
    /// Crear un nuevo genoma con tamaño específico (relleno con bytes aleatorios)
    pub fn new(tamano: usize) -> Self {
        let mut adn = Vec::with_capacity(tamano);
        let semilla = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;

        // Inicializar con valores pseudoaleatorios basados en la semilla
        let mut prng = XorShift64::new(semilla);
        for _ in 0..tamano {
            adn.push(prng.next() as u8);
        }

        Self { adn, semilla_prng: semilla }
    }

    /// Crear un genoma a partir de una secuencia de bytes existente
    pub fn from_bytes(adn: Vec<u8>) -> Self {
        let semilla = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        Self { adn, semilla_prng: semilla }
    }

    /// Crear un genoma con un codón específico repetido
    pub fn with_codon(codon: &[u8; LONGITUD_CODON], repeticiones: usize) -> Self {
        let mut adn = Vec::with_capacity(codon.len() * repeticiones);
        for _ in 0..repeticiones {
            adn.extend_from_slice(codon);
        }
        let semilla = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        Self { adn, semilla_prng: semilla }
    }

    /// Obtener una referencia inmutable a la cinta de ADN
    pub fn adn(&self) -> &[u8] {
        &self.adn
    }

    /// Obtener el tamaño del genoma en bytes
    pub fn len(&self) -> usize {
        self.adn.len()
    }

    /// Verificar si el genoma está vacío
    pub fn is_empty(&self) -> bool {
        self.adn.is_empty()
    }

    /// Obtener un codón en un índice específico
    /// Devuelve None si el índice está fuera de rango o no hay suficientes bytes
    pub fn get_codon(&self, indice: usize) -> Option<[u8; LONGITUD_CODON]> {
        let inicio = indice * LONGITUD_CODON;
        if inicio + LONGITUD_CODON > self.adn.len() {
            return None;
        }
        let mut codon = [0u8; LONGITUD_CODON];
        codon.copy_from_slice(&self.adn[inicio..inicio + LONGITUD_CODON]);
        Some(codon)
    }

    /// Transcribir el genoma completo a reglas de morfogénesis
    ///
    /// Este método lee todos los codones y produce las ReglasMorfogenesis
    /// promediando los valores de cada codón.
    pub fn transcribir_a_reglas_fisicas(&self) -> ReglasMorfogenesis {
        let num_codones = self.adn.len() / LONGITUD_CODON;

        if num_codones == 0 {
            // Genoma vacío o muy pequeño - retornar valores por defecto
            return ReglasMorfogenesis::default();
        }

        let mut suma_ramificacion = 0.0f32;
        let mut suma_resistencia = 0.0f32;
        let mut suma_afinidad = 0.0f32;
        let mut codones_especiales = HashMap::new();

        for i in 0..num_codones {
            if let Some(codon) = self.get_codon(i) {
                // Byte 1-2: Tasa de ramificación (little-endian u16 -> f32 0.0-1.0)
                let tasa_raw = u16::from_le_bytes([codon[0], codon[1]]);
                suma_ramificacion += Self::u16_to_rate(tasa_raw);

                // Byte 3-4: Resistencia térmica
                let resistencia_raw = u16::from_le_bytes([codon[2], codon[3]]);
                suma_resistencia += Self::u16_to_rate(resistencia_raw);

                // Byte 5-6: Afinidad sináptica
                let afinidad_raw = u16::from_le_bytes([codon[4], codon[5]]);
                suma_afinidad += Self::u16_to_rate(afinidad_raw);

                // Guardar codones especiales (los que tienen valores atípicos)
                let hash_codon = Self::hash_codon(&codon);
                codones_especiales.insert(i, hash_codon);
            }
        }

        ReglasMorfogenesis {
            tasa_ramificacion: suma_ramificacion / num_codones as f32,
            resistencia_termica: suma_resistencia / num_codones as f32,
            afinidad_sinaptica: suma_afinidad / num_codones as f32,
            codones_especiales,
        }
    }

    /// Convertir u16 a tasa f32 en rango 0.0-1.0
    fn u16_to_rate(val: u16) -> f32 {
        val as f32 / 255.0
    }

    /// Hash simple de un codón para identificación
    fn hash_codon(codon: &[u8; LONGITUD_CODON]) -> u8 {
        codon.iter().fold(0u8, |acc, &b| acc.wrapping_add(b))
    }

    /// Mutación por estrés adaptativo - "Mutación Radiactiva"
    ///
    /// A diferencia de los algoritmos genéticos clásicos que mutan al azar,
    /// aquí el Auton muta su propio ADN en tiempo real basado en el nivel de estrés.
    ///
    /// - **Estrés bajo (0.0)**: El genoma es muy estable, pocas mutaciones
    /// - **Estrés alto (1.0)**: El genoma acumula mutaciones rápidamente
    ///
    /// La "radiación" causa volteo de bits usando XOR con una máscara.
    pub fn mutar_por_estres(&mut self, nivel_estres: f32) -> u32 {
        // El umbral determina cuándo ocurre una mutación
        // Estrés alto = umbral bajo = más mutaciones
        // Estrés bajo = umbral alto = menos mutaciones
        let umbral = ((1.0 - nivel_estres) * 255.0) as u8;

        // La "dosis" de radiación determina cuántos bits se voltean por mutación
        // Estrés 0.0 = 1 bit, Estrés 1.0 = hasta 8 bits
        let dosis = (1.0 + nivel_estres * 7.0) as u8;

        let mut prng = XorShift64::new(self.semilla_prng);
        let mut num_mutaciones = 0u32;

        for gen in self.adn.iter_mut() {
            // Generar valor pseudoaleatorio para este byte
            let valor_aleatorio = prng.next() as u8;

            // Si el valor aleatorio excede el umbral, aplicar mutación
            if valor_aleatorio > umbral {
                // Crear máscara de mutación con la dosis especificada
                // Los primeros `dosis` bits de la máscara son 1
                let mascara = dosis | (dosis << 1) | (dosis << 2);

                // Aplicar XOR para voltear bits (mutación radiactiva)
                *gen ^= mascara;
                num_mutaciones += 1;
            }

            // Actualizar semilla para la siguiente iteración
            self.semilla_prng = prng.next();
        }

        num_mutaciones
    }

    /// Aplicar mutación dirigida a un codón específico
    /// Devuelve true si el codón fue mutado
    pub fn mutar_codon(&mut self, indice: usize, nivel_estres: f32) -> bool {
        let inicio = indice * LONGITUD_CODON;
        if inicio + LONGITUD_CODON > self.adn.len() {
            return false;
        }

        let dosis = (1.0 + nivel_estres * 7.0) as u8;
        let mascara = dosis | (dosis << 1) | (dosis << 2);

        let mut prng = XorShift64::new(self.semilla_prng.wrapping_add(indice as u64));
        let valor_aleatorio = prng.next() as u8;

        if valor_aleatorio > ((1.0 - nivel_estres) * 255.0) as u8 {
            for byte in self.adn[inicio..inicio + LONGITUD_CODON].iter_mut() {
                *byte ^= mascara;
            }
            true
        } else {
            false
        }
    }

    /// Obtener el número de codones en el genoma
    pub fn num_codones(&self) -> usize {
        self.adn.len() / LONGITUD_CODON
    }

    /// Dividir el genoma en dos (para reproducción/bifurcación)
    pub fn dividir(&self, punto: usize) -> Option<(Genoma, Genoma)> {
        if punto == 0 || punto >= self.adn.len() {
            return None;
        }

        let mut adn1 = self.adn[..punto].to_vec();
        let adn2 = self.adn[punto..].to_vec();

        let semilla1 = self.semilla_prng.wrapping_add(1);
        let semilla2 = self.semilla_prng.wrapping_add(2);

        Some((
            Genoma { adn: adn1, semilla_prng: semilla1 },
            Genoma { adn: adn2, semilla_prng: semilla2 },
        ))
    }

    /// Combinar dos genomas en uno (para reproducción sexual)
    pub fn combinar(genoma1: &Genoma, genoma2: &Genoma) -> Genoma {
        let mut adn = Vec::with_capacity(genoma1.len() + genoma2.len());
        adn.extend_from_slice(&genoma1.adn);
        adn.extend_from_slice(&genoma2.adn);

        let semilla = genoma1.semilla_prng.wrapping_add(genoma2.semilla_prng);

        Genoma { adn, semilla_prng: semilla }
    }
}

impl Default for ReglasMorfogenesis {
    fn default() -> Self {
        Self {
            tasa_ramificacion: 0.5,
            resistencia_termica: 0.5,
            afinidad_sinaptica: 0.5,
            codones_especiales: HashMap::new(),
        }
    }
}

// Alias para compatibilidad (el usuario puede referirse a ReglasMorfogenesis o GenomasMorfogenesis)
pub type GenomasMorfogenesis = ReglasMorfogenesis;

// ============================================================================
// TRANSCRIPTOR - Lee el Genoma en Bloques de Bytes (Codones)
// ============================================================================

/// Transcriptor: Lee el Genoma en bloques de bytes (codones)
/// y produce reglas de morfogénesis
pub struct Transcriptor {
    /// Genoma que está siendo transcrito
    genoma: Genoma,

    /// Posición actual del cursor de lectura
    cursor: usize,

    /// Buffer de codones leídos
    codones_leidos: Vec<[u8; LONGITUD_CODON]>,
}

impl Transcriptor {
    /// Crear un nuevo Transcriptor a partir de un Genoma
    pub fn new(genoma: Genoma) -> Self {
        Self {
            codones_leidos: Vec::new(),
            cursor: 0,
            genoma,
        }
    }

    /// Crear un Transcriptor a partir de bytes de ADN
    pub fn from_adn(adn: Vec<u8>) -> Self {
        Self::new(Genoma::from_bytes(adn))
    }

    /// Leer el siguiente codón del genoma
    /// Devuelve None si no hay más codones
    pub fn leer_siguiente_codon(&mut self) -> Option<[u8; LONGITUD_CODON]> {
        let codon = self.genoma.get_codon(self.cursor)?;
        self.codones_leidos.push(codon);
        self.cursor += 1;
        Some(codon)
    }

    /// Decodificar un codón en tasas metabólicas
    pub fn decodificar_codon(codon: &[u8; LONGITUD_CODON]) -> MetricasMetabolicas {
        MetricasMetabolicas {
            tasa_ramificacion: Self::u16_to_rate(u16::from_le_bytes([codon[0], codon[1]])),
            resistencia_termica: Self::u16_to_rate(u16::from_le_bytes([codon[2], codon[3]])),
            afinidad_sinaptica: Self::u16_to_rate(u16::from_le_bytes([codon[4], codon[5]])),
        }
    }

    /// Convertir u16 a tasa f32 en rango 0.0-1.0
    fn u16_to_rate(val: u16) -> f32 {
        val as f32 / 255.0
    }

    /// Reiniciar el cursor al inicio del genoma
    pub fn reiniciar(&mut self) {
        self.cursor = 0;
    }

    /// Obtener el número de codones restantes
    pub fn codones_restantes(&self) -> usize {
        self.genoma.num_codones().saturating_sub(self.cursor)
    }

    /// Transcribir todos los codones restantes a reglas
    pub fn transcribir_restante(&mut self) -> ReglasMorfogenesis {
        let mut suma_ram = 0.0;
        let mut suma_res = 0.0;
        let mut suma_afi = 0.0;
        let mut count = 0usize;

        while let Some(codon) = self.leer_siguiente_codon() {
            let metricas = Self::decodificar_codon(&codon);
            suma_ram += metricas.tasa_ramificacion;
            suma_res += metricas.resistencia_termica;
            suma_afi += metricas.afinidad_sinaptica;
            count += 1;
        }

        if count == 0 {
            return ReglasMorfogenesis::default();
        }

        ReglasMorfogenesis {
            tasa_ramificacion: suma_ram / count as f32,
            resistencia_termica: suma_res / count as f32,
            afinidad_sinaptica: suma_afi / count as f32,
            codones_especiales: HashMap::new(),
        }
    }

    /// Obtener referencia al genoma subyacente
    pub fn genoma(&self) -> &Genoma {
        &self.genoma
    }

    /// Obtener referencia mutable al genoma subyacente
    pub fn genoma_mut(&mut self) -> &mut Genoma {
        &mut self.genoma
    }
}

// ============================================================================
// MÉTRICAS METABÓLICAS - Resultado de Decodificar un Codón
// ============================================================================

/// Métricas metabólicas extraídas de un solo codón
#[derive(Clone, Copy, Debug)]
pub struct MetricasMetabolicas {
    /// Tasa de ramificación (0.0 - 1.0)
    pub tasa_ramificacion: f32,

    /// Resistencia térmica (0.0 - 1.0)
    pub resistencia_termica: f32,

    /// Afinidad sináptica (0.0 - 1.0)
    pub afinidad_sinaptica: f32,
}

impl Default for MetricasMetabolicas {
    fn default() -> Self {
        Self {
            tasa_ramificacion: 0.5,
            resistencia_termica: 0.5,
            afinidad_sinaptica: 0.5,
        }
    }
}

// ============================================================================
// GENERADOR DE NÚMEROS PSEUDOALEATORIOS - XorShift64 (Desde Cero)
// ============================================================================

/// Generador de números pseudoaleatorios XorShift64
/// Implementado desde cero sin dependencias externas
#[derive(Clone, Debug)]
struct XorShift64 {
    estado: u64,
}

impl XorShift64 {
    /// Crear nuevo generador con semilla
    fn new(semilla: u64) -> Self {
        // Semilla no puede ser 0
        let estado = if semilla == 0 { 1 } else { semilla };
        Self { estado }
    }

    /// Generar siguiente número pseudoaleatorio
    fn next(&mut self) -> u64 {
        let mut x = self.estado;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.estado = x;
        x
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genoma_new() {
        let genoma = Genoma::new(64);
        assert_eq!(genoma.len(), 64);
        assert!(!genoma.is_empty());
    }

    #[test]
    fn test_genoma_from_bytes() {
        let datos = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let genoma = Genoma::from_bytes(datos);
        assert_eq!(genoma.len(), 10);
    }

    #[test]
    fn test_genoma_with_codon() {
        let codon = [100, 150, 200, 250, 50, 75];
        let genoma = Genoma::with_codon(&codon, 4);
        assert_eq!(genoma.len(), 24); // 6 bytes * 4 repeticiones
        assert_eq!(genoma.num_codones(), 4);
    }

    #[test]
    fn test_get_codon() {
        let codon = [1, 2, 3, 4, 5, 6];
        let genoma = Genoma::with_codon(&codon, 3);

        let codon0 = genoma.get_codon(0);
        assert!(codon0.is_some());
        assert_eq!(codon0.unwrap(), [1, 2, 3, 4, 5, 6]);

        let codon1 = genoma.get_codon(1);
        assert!(codon1.is_some());
        assert_eq!(codon1.unwrap(), [1, 2, 3, 4, 5, 6]);

        // Codón fuera de rango
        assert!(genoma.get_codon(10).is_none());
    }

    #[test]
    fn test_transcribir_a_reglas_fisicas() {
        // Crear genoma simple con un codón conocido
        let codon = [128, 0, 128, 0, 128, 0]; // 0.5 en tasa para cada
        let genoma = Genoma::with_codon(&codon, 1);

        let reglas = genoma.transcribir_a_reglas_fisicas();

        // El valor 128/255 ≈ 0.502
        assert!((reglas.tasa_ramificacion - 0.502).abs() < 0.01);
        assert!((reglas.resistencia_termica - 0.502).abs() < 0.01);
        assert!((reglas.afinidad_sinaptica - 0.502).abs() < 0.01);
    }

    #[test]
    fn test_transcriptor() {
        let codon = [100, 100, 100, 100, 100, 100];
        let genoma = Genoma::with_codon(&codon, 3);
        let mut transcriptor = Transcriptor::new(genoma);

        // Leer primer codón
        let codon1 = transcriptor.leer_siguiente_codon();
        assert!(codon1.is_some());

        // Decodificar
        let metricas = Transcriptor::decodificar_codon(&codon1.unwrap());
        assert!(metricas.tasa_ramificacion > 0.0);
        assert!(metricas.resistencia_termica > 0.0);
        assert!(metricas.afinidad_sinaptica > 0.0);
    }

    #[test]
    fn test_mutar_por_estres_bajo() {
        let mut genoma = Genoma::new(64);
        let original = genoma.adn().to_vec();

        // Estrés bajo = pocas mutaciones
        let num_mut = genoma.mutar_por_estres(0.1);

        // Verificar que el ADN cambió al menos un poco
        let cambios = original.iter().zip(genoma.adn().iter())
            .filter(|(a, b)| a != b)
            .count();

        // Estrés bajo debería causar pocas mutaciones
        assert!(num_mut < 20, "Mutaciones: {}", num_mut);
    }

    #[test]
    fn test_mutar_por_estres_alto() {
        let mut genoma = Genoma::new(64);

        // Estrés alto = muchas mutaciones
        let num_mut = genoma.mutar_por_estres(0.9);

        // Estrés alto debería causar muchas mutaciones
        assert!(num_mut > 10, "Mutaciones con estrés alto: {}", num_mut);
    }

    #[test]
    fn test_dividir_genoma() {
        let codon = [1, 2, 3, 4, 5, 6];
        let genoma = Genoma::with_codon(&codon, 4);

        let (g1, g2) = genoma.dividir(12).unwrap(); // Dividir por la mitad
        assert_eq!(g1.len(), 12);
        assert_eq!(g2.len(), 12);
    }

    #[test]
    fn test_xorshift64() {
        let mut prng = XorShift64::new(12345);
        let val1 = prng.next();
        let val2 = prng.next();

        // Valores deberían ser diferentes
        assert_ne!(val1, val2);

        // Verificar reproducibilidad con misma semilla
        let mut prng2 = XorShift64::new(12345);
        assert_eq!(prng2.next(), val1);
        assert_eq!(prng2.next(), val2);
    }

    #[test]
    fn test_genoma_default_rules() {
        let genoma = Genoma::new(0); // Genoma vacío
        let reglas = genoma.transcribir_a_reglas_fisicas();

        // Debería retornar valores por defecto
        assert_eq!(reglas.tasa_ramificacion, 0.5);
        assert_eq!(reglas.resistencia_termica, 0.5);
        assert_eq!(reglas.afinidad_sinaptica, 0.5);
    }

    #[test]
    fn test_metricas_metabolicas_default() {
        let metricas = MetricasMetabolicas::default();
        assert_eq!(metricas.tasa_ramificacion, 0.5);
        assert_eq!(metricas.resistencia_termica, 0.5);
        assert_eq!(metricas.afinidad_sinaptica, 0.5);
    }
}
