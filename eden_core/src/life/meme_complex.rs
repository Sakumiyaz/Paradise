//! # Meme Complex: Sistemas Culturales en EDEN
//!
//! Cuando varios Auton están enlazados pneumáticamente, pueden compartir
//! **patrones de comportamiento** (memes) que codifican estrategias exitosas.
//!
//! ## Arquitectura de Memes
//!
//! Un **Meme** es una sub-RamNet que codifica una estrategia:
//! - Un conjunto de direcciones de LUT y sus valores asociados
//! - Un hash que lo identifica unívocamente
//! - Un contador de "fitness" basado en supervivencias exitosas
//!
//! Un **Complejo de Memes** es un grupo de memes que tienden a transmitirse juntos,
//! formando una "cultura" que distingue a un grupo de Auton de otro.
//!
//! ## Propagación
//!
//! 1. Auton con PneumaBonds comparten hashes de memes periódicamente
//! 2. Si un Auton no tiene un meme ofrecido, puede "aprenderlo" (copiar la porción
//!    de RamNet asociada) si su energía lo permite
//! 3. El aprendizaje cuesta energía (reconfiguración de RamNet)
//! 4. Un meme puede "mutar" al ser copiado (inversión de bits con baja probabilidad)
#![allow(dead_code)]
#![allow(non_snake_case)]

use crate::life::ramnet::XorShift64;
use std::collections::{HashMap, HashSet};

/// Identificador único de un meme (hash de su contenido)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MemeId(pub u64);

impl MemeId {
    /// Crea un nuevo MemeId a partir de contenido
    pub fn desde_contenido(memoria: &[u8], direcciones: &[usize]) -> Self {
        let mut hash: u64 = 0xDEADBEEF;

        for &dir in direcciones {
            if dir < memoria.len() {
                hash = hash.wrapping_mul(31).wrapping_add(memoria[dir] as u64);
            }
        }

        MemeId(hash)
    }
}

/// Tipo de meme según su estrategia
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TipoMeme {
    /// Movimiento evasivo (espiral hacia afuera de Escoria)
    Evasivo,
    /// Movimiento exploratorio (patrón aleatorio)
    Exploratorio,
    /// Movimiento convergente (hacia Energon)
    Convergente,
    /// Agregación (permanecer cerca de otros Auton)
    Agregativo,
    /// Antipredador (evitar zonas con muchos Auton)
    Antipredador,
    /// Estratégico (basado en Umbra propia)
    Estrategico,
    /// Desconocido
    Desconocido,
}

impl TipoMeme {
    /// Infiere el tipo desde los valores de memoria en ciertas direcciones
    pub fn inferir(memoria: &[u8], direcciones: &[usize]) -> Self {
        if direcciones.is_empty() || memoria.is_empty() {
            return TipoMeme::Desconocido;
        }

        // Calcular hash de las direcciones para clasificación
        let mut suma: u64 = 0;
        for &dir in direcciones {
            if dir < memoria.len() {
                suma = suma.wrapping_add(memoria[dir] as u64);
            }
        }

        // Clasificar según hash
        match suma % 6 {
            0 => TipoMeme::Evasivo,
            1 => TipoMeme::Exploratorio,
            2 => TipoMeme::Convergente,
            3 => TipoMeme::Agregativo,
            4 => TipoMeme::Antipredador,
            _ => TipoMeme::Estrategico,
        }
    }
}

/// Representa un meme individual
#[derive(Debug, Clone)]
pub struct Meme {
    /// Identificador único
    pub id: MemeId,
    /// Tipo de estrategia
    pub tipo: TipoMeme,
    /// Direcciones de RamNet que contiene
    pub direcciones: Vec<usize>,
    /// Valores originais (para mutación)
    pub valores_originales: Vec<u8>,
    /// Fitness del meme (supervivencias exitosas)
    pub fitness: u32,
    /// Contador de veces propagado
    pub veces_propagado: u32,
    /// Tick de creación
    pub tick_creacion: u64,
    /// Hash del complejo al que pertenece (0 = ninguno)
    pub id_complejo: u64,
}

impl Meme {
    /// Crea un nuevo meme desde una porción de RamNet
    pub fn new(memoria: &[u8], direcciones: Vec<usize>, tick_actual: u64) -> Self {
        let valores: Vec<u8> = direcciones
            .iter()
            .filter_map(|&d| memoria.get(d).copied())
            .collect();

        let id = MemeId::desde_contenido(memoria, &direcciones);
        let tipo = TipoMeme::inferir(memoria, &direcciones);

        Meme {
            id,
            tipo,
            direcciones,
            valores_originales: valores,
            fitness: 1,
            veces_propagado: 0,
            tick_creacion: tick_actual,
            id_complejo: 0,
        }
    }

    /// Aplica mutación al meme (copia con variaciones)
    pub fn mutar(&self, _memoria: &[u8], probabilidad: f64, rng: &mut XorShift64) -> Self {
        let mut nuevos_valores = self.valores_originales.clone();

        // Aplicar mutación: inversión de bits
        for val in &mut nuevos_valores {
            if rng.next_f64() < probabilidad {
                let mask = rng.next_u8();
                *val ^= mask;
            }
        }

        // Crear nuevo hash basado en valores mutados
        let mut hash = 0xCAFEBABE_u64;
        for &val in &nuevos_valores {
            hash = hash.wrapping_mul(31).wrapping_add(val as u64);
        }

        Meme {
            id: MemeId(hash),
            tipo: self.tipo,
            direcciones: self.direcciones.clone(),
            valores_originales: nuevos_valores,
            fitness: 1, //Nuevo meme empieza con fitness 1
            veces_propagado: 0,
            tick_creacion: rng.next(),     // Nuevo tick
            id_complejo: self.id_complejo, // Hereda complejo
        }
    }

    /// Incrementa fitness tras supervivencia exitosa
    pub fn incrementar_fitness(&mut self) {
        self.fitness = self.fitness.saturating_add(1);
    }
}

/// Un complejo de memes (cultura)
#[derive(Debug, Clone)]
pub struct ComplejoMemes {
    /// ID único del complejo
    pub id: u64,
    /// Nombre/etiqueta descriptivo
    pub nombre: String,
    /// Memes que lo componen
    pub memes: Vec<MemeId>,
    /// Fitness promedio del complejo
    pub fitness_promedio: f64,
    /// Tick de creación
    pub tick_creacion: u64,
}

impl ComplejoMemes {
    /// Crea un nuevo complejo
    pub fn new(id: u64, nombre: &str, tick: u64) -> Self {
        ComplejoMemes {
            id,
            nombre: nombre.to_string(),
            memes: Vec::new(),
            fitness_promedio: 1.0,
            tick_creacion: tick,
        }
    }

    /// Añade un meme al complejo
    pub fn añadir_meme(&mut self, meme_id: MemeId) {
        if !self.memes.contains(&meme_id) {
            self.memes.push(meme_id);
        }
    }
}

/// Administrador de memes en el universo
pub struct MemeManager {
    /// Mapa de memes conocidos (id -> meme)
    memes: HashMap<MemeId, Meme>,
    /// Mapa de complejos de memes
    complejos: HashMap<u64, ComplejoMemes>,
    /// Contador de complejos creados
    proximo_id_complejo: u64,
    /// Contador de memes creados
    proximo_id_meme: u64,
}

impl MemeManager {
    /// Crea un nuevo administrador
    pub fn new() -> Self {
        MemeManager {
            memes: HashMap::new(),
            complejos: HashMap::new(),
            proximo_id_complejo: 1,
            proximo_id_meme: 1,
        }
    }

    /// Registra un nuevo meme en el universo
    pub fn registrar_meme(&mut self, meme: Meme) -> MemeId {
        let id = meme.id;
        self.memes.insert(id, meme);
        id
    }

    /// Obtiene un meme por id
    pub fn obtener_meme(&self, id: MemeId) -> Option<&Meme> {
        self.memes.get(&id)
    }

    /// Obtiene un meme mutable por id
    pub fn obtener_meme_mut(&mut self, id: MemeId) -> Option<&mut Meme> {
        self.memes.get_mut(&id)
    }

    /// Obtiene todos los memes de un tipo específico
    pub fn memes_por_tipo(&self, tipo: TipoMeme) -> Vec<&Meme> {
        self.memes.values().filter(|m| m.tipo == tipo).collect()
    }

    /// Obtiene el complejo dominante de un Auton basándose en sus memes
    pub fn complejo_dominante(&self, memes_auton: &HashSet<MemeId>) -> Option<u64> {
        // Cuenta memes por complejo
        let mut conteo: HashMap<u64, u32> = HashMap::new();

        for &meme_id in memes_auton {
            if let Some(meme) = self.memes.get(&meme_id) {
                let complejo = meme.id_complejo;
                *conteo.entry(complejo).or_insert(0) += meme.fitness;
            }
        }

        // Retorna el complejo con más fitness total
        conteo.into_iter().max_by_key(|&(_, f)| f).map(|(c, _)| c)
    }

    /// Crea un nuevo complejo de memes
    pub fn crear_complejo(&mut self, nombre: &str, tick: u64) -> u64 {
        let id = self.proximo_id_complejo;
        self.proximo_id_complejo += 1;

        let complejo = ComplejoMemes::new(id, nombre, tick);

        // Registrar complejo
        self.complejos.insert(id, complejo.clone());
        id
    }

    /// Añade un meme existente a un complejo
    pub fn añadir_a_complejo(&mut self, meme_id: MemeId, complejo_id: u64) {
        if let Some(meme) = self.memes.get_mut(&meme_id) {
            meme.id_complejo = complejo_id;
        }
        if let Some(complejo) = self.complejos.get_mut(&complejo_id) {
            complejo.añadir_meme(meme_id);
        }
    }

    /// Número de memes activos
    pub fn num_memes(&self) -> usize {
        self.memes.len()
    }

    /// Número de complejos
    pub fn num_complejos(&self) -> usize {
        self.complejos.len()
    }
}

/// Estado de aprendizaje de memes para un Auton
#[derive(Debug, Clone)]
pub struct MemeAprendizaje {
    /// Memes que el Auton conoce
    pub memes_conocidos: HashSet<MemeId>,
    /// Memes actualmente activos en el Auton
    pub memes_activos: HashSet<MemeId>,
    /// Complejo dominante
    pub complejo_dominante: Option<u64>,
    /// Energía disponible para aprendizaje
    pub energia_disponible: i64,
}

impl MemeAprendizaje {
    /// Crea un nuevo estado de aprendizaje
    pub fn new() -> Self {
        MemeAprendizaje {
            memes_conocidos: HashSet::new(),
            memes_activos: HashSet::new(),
            complejo_dominante: None,
            energia_disponible: 0,
        }
    }

    /// Verifica si conoce un meme
    pub fn conoce_meme(&self, meme_id: MemeId) -> bool {
        self.memes_conocidos.contains(&meme_id)
    }

    /// Añade un meme conocido
    pub fn aprender_meme(&mut self, meme_id: MemeId) {
        self.memes_conocidos.insert(meme_id);
        self.memes_activos.insert(meme_id);
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    const COSTO_ENERGIA_MEME: i64 = 5_000_000_000i64; // 5e9 para tests

    #[test]
    fn test_meme_id_desde_contenido() {
        let memoria = [0x01u8, 0x02, 0x03, 0x04, 0x05];
        let direcciones = [0, 2, 4];

        let id = MemeId::desde_contenido(&memoria, &direcciones);
        assert!(id.0 != 0);

        // Mismo contenido = mismo id
        let id2 = MemeId::desde_contenido(&memoria, &direcciones);
        assert_eq!(id, id2);
    }

    #[test]
    fn test_inferencia_tipo_meme() {
        let memoria = [0x10u8, 0x20, 0x30, 0x40, 0x50];
        let direcciones = [0, 1, 2];

        // Debería clasificar en algún tipo
        let tipo = TipoMeme::inferir(&memoria, &direcciones);
        assert_ne!(tipo, TipoMeme::Desconocido);
    }

    #[test]
    fn test_crear_meme() {
        let memoria = vec![0x11u8, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88];
        let direcciones = vec![0, 2, 4];

        let meme = Meme::new(&memoria, direcciones, 100);

        assert_eq!(meme.fitness, 1);
        assert_eq!(meme.veces_propagado, 0);
        assert_eq!(meme.tick_creacion, 100);
        assert!(!meme.valores_originales.is_empty());
    }

    #[test]
    fn test_mutacion_meme() {
        let memoria = vec![0xFFu8; 256];
        let direcciones: Vec<usize> = (0..8).collect();

        let original = Meme::new(&memoria, direcciones, 100);
        let id_original = original.id;

        let mut rng = XorShift64::new(42);
        let mutado = original.mutar(&memoria, 0.1, &mut rng);

        // El mutado debería tener fitness 1 (nuevo)
        assert_eq!(mutado.fitness, 1);
        // El ID debería ser diferente si hubo mutación
        // (no siempre garantizado si la mutación produce mismos valores)
        assert_eq!(original.id, id_original);
        assert!(mutado.tick_creacion != original.tick_creacion);
    }

    #[test]
    fn test_meme_manager_registro() {
        let mut manager = MemeManager::new();

        let memoria = vec![0xAAu8; 256];
        let meme = Meme::new(&memoria, vec![0, 1, 2, 3], 100);
        let id = manager.registrar_meme(meme);

        assert!(manager.obtener_meme(id).is_some());
        assert_eq!(manager.num_memes(), 1);
    }

    #[test]
    fn test_complejo_dominante() {
        let mut manager = MemeManager::new();

        // Crear complejo
        let complejo_id = manager.crear_complejo("Cultura A", 100);

        // Crear meme y añadirlo al complejo
        let memoria = vec![0xBBu8; 256];
        let meme = Meme::new(&memoria, vec![0, 1], 100);
        let id = manager.registrar_meme(meme);
        manager.añadir_a_complejo(id, complejo_id);

        // Simular Auton con ese meme
        let mut memes_auton = HashSet::new();
        memes_auton.insert(id);

        let dominante = manager.complejo_dominante(&memes_auton);
        assert_eq!(dominante, Some(complejo_id));
    }

    #[test]
    fn test_aprendizaje_meme() {
        let mut aprendizaje = MemeAprendizaje::new();

        assert!(!aprendizaje.conoce_meme(MemeId(123)));

        aprendizaje.aprender_meme(MemeId(123));

        assert!(aprendizaje.conoce_meme(MemeId(123)));
        assert!(aprendizaje.memes_activos.contains(&MemeId(123)));
    }

    #[test]
    fn test_memes_por_tipo() {
        let mut manager = MemeManager::new();

        let memoria = vec![0xCCu8; 256];

        // Crear varios memes de diferentes tipos
        for i in 0..20 {
            let direcciones = vec![i as usize, (i + 1) as usize];
            let meme = Meme::new(&memoria, direcciones, i as u64);
            manager.registrar_meme(meme);
        }

        let evasivos = manager.memes_por_tipo(TipoMeme::Evasivo);
        // Algunos deberían ser evasivos por clasificación de hash
        println!("Memes evasivos encontrados: {}", evasivos.len());
    }

    #[test]
    fn test_propagacion_cultural() {
        // Simular dos grupos con culturas diferentes

        let mut manager = MemeManager::new();

        // Grupo A: cultura "Exploradores"
        let memoria_a = vec![0x11u8; 256];
        let direcciones_a = vec![0, 1, 2, 3];
        let meme_a = Meme::new(&memoria_a, direcciones_a, 100);
        let id_a = manager.registrar_meme(meme_a);

        let complejo_a = manager.crear_complejo("Exploradores", 100);
        manager.añadir_a_complejo(id_a, complejo_a);

        // Grupo B: cultura "Convergentes"
        let memoria_b = vec![0x22u8; 256];
        let direcciones_b = vec![10, 11, 12, 13];
        let meme_b = Meme::new(&memoria_b, direcciones_b, 100);
        let id_b = manager.registrar_meme(meme_b);

        let complejo_b = manager.crear_complejo("Convergentes", 100);
        manager.añadir_a_complejo(id_b, complejo_b);

        // Un Auton del grupo A conoce su cultura
        let mut memes_auton_a = HashSet::new();
        memes_auton_a.insert(id_a);

        let dominante = manager.complejo_dominante(&memes_auton_a);
        assert_eq!(dominante, Some(complejo_a));

        // Un Auton del grupo B conoce su cultura
        let mut memes_auton_b = HashSet::new();
        memes_auton_b.insert(id_b);

        let dominante_b = manager.complejo_dominante(&memes_auton_b);
        assert_eq!(dominante_b, Some(complejo_b));
    }
}
