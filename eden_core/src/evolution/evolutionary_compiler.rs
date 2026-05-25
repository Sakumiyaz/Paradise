//! # Evolutionary Compiler: Mutación AST + Fitness Evaluation
//!
//! Este módulo implementa la capacidad de EDEN de reescribir su propio código
//! mediante mutación evolutiva de ASTs, evaluación de fitness, y preservación
//! de las Leyes Inmutables.
//!
//! ## Filosofía
//!
//! El compilador evolutivo NO es un optimizador — es un *explorador*.
//! La optimización lleva a convergencia prematura (muerte evolutiva).
//! El compilador evolutivo busca *novedad útil* que emerja de la tensión
//! entre eficiencia y diversidad.
#![allow(dead_code)]
#![allow(non_snake_case)]

// ============================================================================
// ARQUITECTURA DEL COMPILADOR EVOLUTIVO
// ============================================================================

use crate::evolution::open_endedness::ComplejidadArquetipica;
use crate::life::umbra::Umbra;
use crate::physics::fixed_point::I32F32;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

// ============================================================================
// TIPOS DE AST MUTABLE
// ============================================================================

/// Nodo del AST que puede ser mutado
#[derive(Debug, Clone)]
pub enum AstNode {
    /// Literal constante (inmutable por evoluci\u00f3n)
    Literal { valor: i64, tipo: TipoLiteral },
    /// Identificador
    Identificador { nombre: String },
    /// Operaci\u00f3n binaria
    BinOp { op: BinOp, izquierdo: Box<AstNode>, derecho: Box<AstNode> },
    /// Llamada de funci\u00f3n
    Llamada { nombre: String, args: Vec<AstNode> },
    /// Expresi\u00f3n condicional
    Condicional { condicion: Box<AstNode>, entonces: Box<AstNode>, sino: Box<AstNode> },
    /// Bucle
    Bucle { variable: String, inicio: Box<AstNode>, fin: Box<AstNode>, cuerpo: Box<AstNode> },
    /// Secuencia de expresiones
    Secuencia { expresiones: Vec<AstNode> },
    /// Registro de funci\u00f3n parcheable
    Funci\u00f3nParcheable { nombre: String, parametros: Vec<String>, cuerpo: Box<AstNode> },
}

/// Tipo de literal
#[derive(Debug, Clone, Copy)]
pub enum TipoLiteral {
    Entero,
    Flotante,
    Booleano,
}

/// Operadores binarios
#[derive(Debug, Clone, Copy)]
pub enum BinOp {
    Suma, Resta, Multiplicacion, Division,
    Y, O, Igual, Diferente,
    Menor, Mayor, MenorIgual, MayorIgual,
}

/// Tipo de mutaci\u00f3n aplicable
#[derive(Debug, Clone, Copy)]
pub enum TipoMutacion {
    /// Cambiar una constante
    Tweaking,
    /// Intercambiar sub\u00e1rboles
    Crossover,
    /// Insertar nuevo nodo
    Insercion,
    /// Eliminar sub\u00e1rbol
    Deletion,
    /// Mutar operador
    Operador,
    /// Cambiar tipo de construcci\u00f3n (ej: if -> match)
    Hoist,
}

impl TipoMutacion {
    pub fn probabilidad(&self) -> f32 {
        match self {
            TipoMutacion::Tweaking => 0.40,
            TipoMutacion::Crossover => 0.25,
            TipoMutacion::Insercion => 0.15,
            TipoMutacion::Deletion => 0.10,
            TipoMutacion::Operador => 0.05,
            TipoMutacion::Hoist => 0.05,
        }
    }
}

// ============================================================================
// GENOMA: REPRESENTACI\u00d3N EVOLUTIVA DE FUNCIONES
// ============================================================================

/// Un genoma es la representaci\u00f3n evolutiva de una funci\u00f3n parcheable
#[derive(Debug, Clone)]
pub struct Genoma {
    /// ID \u00fanico del genoma
    pub id: u64,
    /// Nombre de la funci\u00f3n que codifica
    pub nombre_funcion: String,
    /// AST del cuerpo de la funci\u00f3n
    pub ast: AstNode,
    /// Fitness hist\u00f3rico (para selecci\u00f3n)
    pub fitness: f32,
    /// Fitness anterior (para comparaci\u00f3n)
    pub fitness_anterior: f32,
    /// N\u00famero de mutaciones aplicadas
    pub num_mutaciones: u32,
    /// Linaje (IDs de genomas ancestros)
    pub linaje: Vec<u64>,
    /// Timestamp de creaci\u00f3n
    pub tick_creacion: u64,
    /// Complejidad arquet\u00edpica
    pub complejidad: ComplejidadArquetipica,
    /// Hash del c\u00f3digo (para deduplicaci\u00f3n)
    pub hash_codigo: u64,
    /// Regions protegidas (Ley 1: solo .eden_patchable)
    pub regiones_permitidas: Vec<String>,
}

impl Genoma {
    pub fn new(id: u64, nombre: &str, ast: AstNode, tick: u64) -> Self {
        let hash = Self::calcular_hash_ast(&ast);
        Genoma {
            id,
            nombre_funcion: nombre.to_string(),
            ast,
            fitness: 0.0,
            fitness_anterior: 0.0,
            num_mutaciones: 0,
            linaje: Vec::new(),
            tick_creacion: tick,
            complejidad: ComplejidadArquetipica {
                reflexividad: 0.0,
                diversitas: 0.0,
                acoplamiento: 0.0,
                total: 0.0,
            },
            hash_codigo: hash,
            regiones_permitidas: vec![".eden_patchable".to_string()],
        }
    }

    /// Calcula hash del AST para deduplicaci\u00f3n
    fn calcular_hash_ast(ast: &AstNode) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut s = DefaultHasher::new();
        ast.hash(&mut s);
        s.finish()
    }

    /// Verifica que el genoma s\u00f3lo modifica regiones permitidas
    pub fn verificar_regiones(&self) -> bool {
        self.nombre_funcion.starts_with(".eden_patchable")
            || self.nombre_funcion.starts_with(".eden_patchable.")
    }

    /// Clona con nueva mutaci\u00f3n
    pub fn mutar(&self, tipo: TipoMutacion, rng: &mut XorShift64) -> Self {
        let mut nuevo_ast = self.ast.clone();
        Self::aplicar_mutacion(&mut nuevo_ast, &tipo, rng);
        
        let mut hijo = Genoma {
            id: self.id + 1_000_000, // Reservar espacio
            nombre_funcion: self.nombre_funcion.clone(),
            ast: nuevo_ast,
            fitness: 0.0,
            fitness_anterior: self.fitness,
            num_mutaciones: self.num_mutaciones + 1,
            linaje: vec![self.id],
            tick_creacion: self.tick_creacion,
            complejidad: self.complejidad,
            hash_codigo: Self::calcular_hash_ast(&self.ast),
            regiones_permitidas: self.regiones_permitidas.clone(),
        };
        hijo.hash_codigo = Self::calcular_hash_ast(&hijo.ast);
        hijo
    }

    /// Aplica mutaci\u00f3n al AST
    fn aplicar_mutacion(ast: &mut AstNode, tipo: &TipoMutacion, rng: &mut XorShift64) {
        match tipo {
            TipoMutacion::Tweaking => {
                if let AstNode::Literal { valor, .. } = ast {
                    let delta = rng.next_i64() % 100;
                    *valor += delta;
                }
            },
            TipoMutacion::Crossover => {
                // En una implementaci\u00f3n completa, intercambiaria sub\u00e1rboles
                // Por ahora es un placeholder
            },
            TipoMutacion::Insercion => {
                // Insertar nueva rama
            },
            TipoMutacion::Deletion => {
                // Eliminar rama (no si es \u00faltima)
            },
            TipoMutacion::Operador => {
                if let AstNode::BinOp { op, .. } = ast {
                    *op = Self::random_binop(rng);
                }
            },
            TipoMutacion::Hoist => {
                // Cambiar tipo de construcci\u00f3n
            },
        }
    }

    fn random_binop(rng: &mut XorShift64) -> BinOp {
        match rng.next_u32() % 12 {
            0 => BinOp::Suma,
            1 => BinOp::Resta,
            2 => BinOp::Multiplicacion,
            3 => BinOp::Division,
            4 => BinOp::Y,
            5 => BinOp::O,
            6 => BinOp::Igual,
            7 => BinOp::Diferente,
            8 => BinOp::Menor,
            9 => BinOp::Mayor,
            10 => BinOp::MenorIgual,
            _ => BinOp::MayorIgual,
        }
    }
}

// ============================================================================
// FITNESS: EVALUACI\u00d3N DE GENOMAS
// ============================================================================

/// M\u00e9tricas de fitness para un genoma
#[derive(Debug, Clone)]
pub struct FitnessMetrics {
    /// Fitness total (0.0 - 1.0)
    pub total: f32,
    /// Performance (ejecuci\u00f3n m\u00e1s r\u00e1pida = mejor)
    pub performance: f32,
    /// Robustez (tasa de \u00e9xito en tests)
    pub robustez: f32,
    /// Novedad (diferencia de genomas anteriores)
    pub novedad: f32,
    /// Complejidad (balance con otras m\u00e9tricas)
    pub complejidad: f32,
    /// Inagotabilidad (preserva diversidad)
    pub inagotabilidad: f32,
}

impl FitnessMetrics {
    pub fn nuevo(performance: f32, robustez: f32, novedad: f32, complejidad: f32) -> Self {
        // INAGOTABILIDAD: La complejidad no suma, MULTIPLICA!
        // Esto previene convergencia prematura
        let complejidad_balance = if complejidad > 0.8 {
            0.5 // Penalizaci\u00f3n por ser "demasiado perfecto"
        } else {
            1.0 + complejidad * 0.2
        };

        let total = (performance * robustez * novedad * complejidad_balance)
            .sqrt()
            .max(0.01); // Nunca 0

        FitnessMetrics {
            total,
            performance,
            robustez,
            novedad,
            complejidad,
            inagotabilidad: 1.0 - (complejidad * 0.3).min(0.3),
        }
    }
}

// ============================================================================
// POOL EVOLUTIVO: POBLACI\u00d3N DE GENOMAS
// ============================================================================

/// Pool gen\u00e9tico para evoluci\u00f3n concurrente
pub struct PoolGenetico {
    /// Genomas activos
    genomas: Vec<Genoma>,
    /// Genomas en evaluaci\u00f3n
    evaluando: Vec<Genoma>,
    /// Genomas muertos (historial)
    historial: Vec<Genoma>,
    /// \u00cdndice de genomas por funci\u00f3n
    por_funcion: HashMap<String, Vec<u64>>,
    /// Configuraci\u00f3n
    config: PoolConfig,
    /// Siguiente ID
    next_id: u64,
}

/// Configuraci\u00f3n del pool gen\u00e9tico
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// M\u00e1ximo de genomas vivos
    pub max_genomas: usize,
    /// M\u00e1ximo de genomas por funci\u00f3n
    pub max_por_funcion: usize,
    /// Tasa de mutaci\u00f3n base
    pub tasa_mutacion: f32,
    /// Presi\u00f3n de selecci\u00f3n (0.0 = todo sobrevive, 1.0 = solo mejor)
    pub presion_seleccion: f32,
    /// Enable elitismo (preservar mejores)
    pub elitismo: bool,
    /// N\u00famero de elite a preservar
    pub num_elite: usize,
    /// Threshold de novedad (m\u00ednima diferencia para conservar)
    pub threshold_novedad: f32,
}

impl Default for PoolConfig {
    fn default() -> Self {
        PoolConfig {
            max_genomas: 100,
            max_por_funcion: 20,
            tasa_mutacion: 0.1,
            presion_seleccion: 0.7,
            elitismo: true,
            num_elite: 3,
            threshold_novedad: 0.1,
        }
    }
}

impl PoolGenetico {
    pub fn new(config: PoolConfig) -> Self {
        PoolGenetico {
            genomas: Vec::new(),
            evaluando: Vec::new(),
            historial: Vec::new(),
            por_funcion: HashMap::new(),
            config,
            next_id: 1,
        }
    }

    /// Registra un genoma inicial
    pub fn registrar(&mut self, nombre: &str, ast: AstNode, tick: u64) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let genoma = Genoma::new(id, nombre, ast, tick);
        let genoma_id = genoma.id;

        // Verificar regiones permitidas
        if !genoma.verificar_regiones() {
            return 0; // Falla silenciosa - regi\u00f3n no permitida
        }

        // Verificar l\u00edmite por funci\u00f3n
        let current = self.por_funcion.get(nombre)
            .map(|v| v.len())
            .unwrap_or(0);
        
        if current >= self.config.max_por_funcion {
            return 0;
        }

        self.genomas.push(genoma.clone());
        self.por_funcion.entry(nombre.to_string())
            .or_insert_with(Vec::new)
            .push(genoma_id);

        genoma_id
    }

    /// Selecciona genomas para mutaci\u00f3n (torneo)
    pub fn seleccionar_padres(&self, num: usize) -> Vec<&Genoma> {
        let mut padres = Vec::with_capacity(num);
        let tournament_size = (self.genomas.len() / 5).max(3);

        for _ in 0..num {
            let mut mejores = Vec::with_capacity(tournament_size);
            for _ in 0..tournament_size {
                let idx = (rand_u64() as usize) % self.genomas.len();
                mejores.push(&self.genomas[idx]);
            }
            mejores.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap());
            if let Some(ganador) = mejores.first() {
                padres.push(*ganador);
            }
        }

        padres
    }

    /// Genera nueva generaci\u00f3n
    pub fn generar_siguiente_generacion(&mut self, rng: &mut XorShift64) {
        let elites = if self.config.elitismo {
            let mut sorted = self.genomas.clone();
            sorted.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap());
            sorted.into_iter().take(self.config.num_elite).collect::<Vec<_>>()
        } else {
            Vec::new()
        };

        // Seleccionar padres
        let num_hijos = (self.config.max_genomas - elites.len()) / 2;
        let padres = self.seleccionar_padres(num_hijos);

        // Crear hijos
        let mut hijos = Vec::new();
        for padre in padres {
            let tipo = Self::seleccionar_tipo_mutacion(rng);
            let hijo = padre.mutar(tipo, rng);
            hijos.push(hijo);
        }

        // Calcular fitness de todos
        for genoma in &mut self.genomas {
            genoma.fitness_anterior = genoma.fitness;
            genoma.fitness = self.evaluar_genoma(genoma);
        }

        for genoma in &mut hijos {
            genoma.fitness = self.evaluar_genoma(genoma);
        }

        // Seleccionar sobrevivientes
        self.genomas.append(&mut hijos);
        self.supervivencia();

        // Agregar elite
        for elite in elites {
            if !self.genomas.iter().any(|g| g.id == elite.id) {
                self.genomas.push(elite);
            }
        }

        // Historial
        self.historial.extend(self.genomas.clone());
        if self.historial.len() > 1000 {
            self.historial.drain(0..100);
        }
    }

    /// Eval\u00faa fitness de un genoma
    fn evaluar_genoma(&self, genoma: &Genoma) -> f32 {
        // M\u00e9tricas dummy - en implementaci\u00f3n real usar\u00eda tests
        let performance = 0.7 + (genoma.num_mutaciones as f32 * 0.01).min(0.2);
        let robustez = 0.8;
        let novedad = self.calcular_novedad(genoma);
        let complejidad = (genoma.complejidad.total * 0.5).min(1.0);

        FitnessMetrics::nuevo(performance, robustez, novedad, complejidad).total
    }

    /// Calcula novedad respecto al historial
    fn calcular_novedad(&self, genoma: &Genoma) -> f32 {
        let mut min_distancia = f32::MAX;

        for hist in &self.historial {
            if hist.nombre_funcion == genoma.nombre_funcion {
                let dist = Self::distancia_hash(hist.hash_codigo, genoma.hash_codigo);
                min_distancia = min_distancia.min(dist);
            }
        }

        min_distancia.min(1.0)
    }

    fn distancia_hash(a: u64, b: u64) -> f32 {
        let diff = (a ^ b).count_ones() as f32;
        diff / 64.0
    }

    /// Selecci\u00f3n de supervvivencia (preservar diversidad)
    fn supervivencia(&mut self) {
        // INAGOTABILIDAD: No usar solo fitness
        // Preservar diversidad con speciation
        self.genomas.sort_by(|a, b| {
            let fitness_cmp = b.fitness.partial_cmp(&a.fitness).unwrap();
            if fitness_cmp != std::cmp::Ordering::Equal {
                fitness_cmp
            } else {
                // Secondary: diversidad
                b.complejidad.diversitas.partial_cmp(&a.complejidad.diversitas).unwrap()
            }
        });

        // Reducir hasta max
        while self.genomas.len() > self.config.max_genomas {
            self.genomas.pop();
        }
    }

    /// Selecciona tipo de mutaci\u00f3n aleatoria
    fn seleccionar_tipo_mutacion(rng: &mut XorShift64) -> TipoMutacion {
        let r = rng.next_f32();
        let mut acum = 0.0;
        
        for tipo in [
            TipoMutacion::Tweaking,
            TipoMutacion::Crossover,
            TipoMutacion::Insercion,
            TipoMutacion::Deletion,
            TipoMutacion::Operador,
            TipoMutacion::Hoist,
        ].iter() {
            acum += tipo.probabilidad();
            if r < acum {
                return *tipo;
            }
        }
        
        TipoMutacion::Tweaking
    }

    /// Obtiene mejores genomas para aplicaci\u00f3n
    pub fn mejores_candidatos(&self, num: usize) -> Vec<&Genoma> {
        let mut sorted = self.genomas.clone();
        sorted.sort_by(|a, b| b.fitness.partial_cmp(&a.fitness).unwrap());
        sorted.into_iter().take(num).collect()
    }

    /// Obtiene genoma por ID
    pub fn get(&self, id: u64) -> Option<&Genoma> {
        self.genomas.iter().find(|g| g.id == id)
    }
}

// ============================================================================
// GENERADOR RNG
// ============================================================================

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

    pub fn next_u32(&mut self) -> u32 {
        (self.next() & 0xFFFFFFFF) as u32
    }

    pub fn next_f32(&mut self) -> f32 {
        (self.next() as f32) / (u32::MAX as f32)
    }

    pub fn next_i64(&mut self) -> i64 {
        self.next() as i64
    }
}

// ============================================================================
// UTILIDADES
// ============================================================================

fn rand_u64() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genoma_creacion() {
        let ast = AstNode::Literal { valor: 42, tipo: TipoLiteral::Entero };
        let genoma = Genoma::new(1, ".eden_patchable.test", ast, 0);
        
        assert!(genoma.verificar_regiones());
        assert_eq!(genoma.nombre_funcion, ".eden_patchable.test");
    }

    #[test]
    fn test_genoma_mutacion() {
        let ast = AstNode::Literal { valor: 42, tipo: TipoLiteral::Entero };
        let genoma = Genoma::new(1, ".eden_patchable.test", ast, 0);
        
        let mut rng = XorShift64::new(123);
        let hijo = genoma.mutar(TipoMutacion::Tweaking, &mut rng);
        
        assert!(hijo.num_mutaciones == 1);
        assert!(hijo.linaje.contains(&1));
    }

    #[test]
    fn test_pool_registro() {
        let config = PoolConfig::default();
        let mut pool = PoolGenetico::new(config);
        
        let ast = AstNode::Literal { valor: 42, tipo: TipoLiteral::Entero };
        let id = pool.registrar(".eden_patchable.test", ast, 0);
        
        assert!(id > 0);
        assert_eq!(pool.genomas.len(), 1);
    }

    #[test]
    fn test_fitness_metrics() {
        let metrics = FitnessMetrics::nuevo(0.8, 0.9, 0.7, 0.5);
        
        assert!(metrics.total > 0.0);
        assert!(metrics.inagotabilidad > 0.5); // Complejidad 0.5 no debe penalizar mucho
    }
}
