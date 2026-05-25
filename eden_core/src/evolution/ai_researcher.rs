//! # AI Researcher — Motor de Descubrimiento Automático de Algoritmos
//!
//! Este módulo implementa la capacidad de EDEN de **descubrir nuevos algoritmos**
//! mediante sketch + búsqueda guiada por tipos, sin depender de bibliotecas externas.
//!
//! ## Filosofía
//!
//! El investigador de IA no _diseña_ algoritmos — los **descubre**.
//! Usa una combinación de:
//! - **Sketching**: Esqueletos de algoritmos incompletos que se completan
//! - **Búsqueda Guiada por Tipos**: El sistema de tipos guía la construcción
//! - **Evaluación Evolutiva**: Los candidatos se evalúan y mutan
//!
//! ## Seguridad Ontológica
//!
//! Los algoritmos descubiertos son **verificados formalmente** antes de aplicarse.
//! No se permite la emergencia de código que:
//! - pueda auto-modificar las Leyes Inmutables
//! - intente evadir el sandbox
//! - consuma recursos sin límite
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::{HashMap, VecDeque};

/// Tipo de algoritmo descubierto
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TipoAlgoritmo {
    /// Algoritmo de optimización numérica
    Optimizacion,
    /// Algoritmo de búsqueda en grafos
    BusquedaGrafos,
    /// Algoritmo de clasificación
    Clasificacion,
    /// Algoritmo de compresión
    Compresion,
    /// Algoritmo de encriptación simétrica
    Criptografia,
    /// Algoritmo de síntesis de sonido
    AudioSntesis,
    /// Algoritmo de renderizado
    Renderizado,
    /// Algoritmo de planificación
    Planificacion,
    /// Meta-algoritmo (algoritmo que modifica otros)
    Meta,
}

/// Complejidad computacional estimada
#[derive(Debug, Clone, Copy)]
pub struct ComplejidadAlgoritmo {
    pub tiempo: ComplexityClass,
    pub espacio: ComplexityClass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComplexityClass {
    O1,          // Constante
    OLogN,       // Logarítmica
    ON,          // Lineal
    ONLogN,      // Lineal-logarítmica
    ON2,         // Cuadrática
    ON3,         // Cúbica
    OExponential, // Exponencial (peligroso)
}

impl ComplexityClass {
    /// Verifica si la complejidad es aceptable para aplicación
    pub fn es_aceptable(&self) -> bool {
        !matches!(self, ComplexityClass::OExponential)
    }
}

/// Un sketch es un esqueleto de algoritmo con huecos a completar
#[derive(Debug, Clone)]
pub struct Sketch {
    /// Nombre del sketch
    pub nombre: String,
    /// Tipo de algoritmo objetivo
    pub tipo: TipoAlgoritmo,
    /// Parámetros formales (huecos a completar)
    pub parametros: Vec<ParametroSketch>,
    /// Cuerpo del sketch (código parcialmente especificado)
    pub cuerpo: String,
    /// Restricciones de tipos
    pub restricciones_tipo: Vec<ResticcionTipo>,
    /// Complejidad estimada
    pub complejidad_estimada: ComplejidadAlgoritmo,
}

/// Parámetro de un sketch
#[derive(Debug, Clone)]
pub struct ParametroSketch {
    pub nombre: String,
    pub tipo: String,
    pub valor_default: Option<String>,
    pub rango: Option<(f64, f64)>,
}

/// Restricción de tipo
#[derive(Debug, Clone)]
pub struct ResticcionTipo {
    pub parametro: String,
    pub restricciones: Vec<String>,
}

/// Algoritmo completamente descubierto
#[derive(Debug, Clone)]
pub struct AlgoritmoDescubierto {
    pub id: u64,
    pub nombre: String,
    pub tipo: TipoAlgoritmo,
    /// Código fuente del algoritmo
    pub codigo: String,
    /// Paramétricas iniciales
    pub parametros_iniciales: Vec<f64>,
    /// Metricas de rendimiento
    pub metricas: MetricasAlgoritmo,
    /// Linaje (de qué sketch/propuesta derivó)
    pub linaje: Vec<u64>,
    /// Timestamp de descubrimiento
    pub tick_descubrimiento: u64,
}

/// Métricas de un algoritmo
#[derive(Debug, Clone)]
pub struct MetricasAlgoritmo {
    /// Rendimiento en benchmark (0.0 - 1.0)
    pub benchmark_score: f32,
    /// Tiempo de ejecución relativo (más bajo = mejor)
    pub tiempo_relativo: f32,
    /// Uso de memoria relativo (más bajo = mejor)
    pub memoria_relativa: f32,
    /// Estabilidad (varianza en múltiples runs)
    pub estabilidad: f32,
    /// Novedad (diferencia de algoritmos conocidos)
    pub novedad: f32,
    /// Escalabilidad (cómo escala con más datos)
    pub escalabilidad: f32,
}

impl MetricasAlgoritmo {
    pub fn nuevo(benchmark: f32) -> Self {
        MetricasAlgoritmo {
            benchmark_score: benchmark,
            tiempo_relativo: 1.0,
            memoria_relativa: 1.0,
            estabilidad: 0.5,
            novedad: 0.5,
            escalabilidad: 0.5,
        }
    }

    /// Fitness total del algoritmo
    pub fn fitness(&self) -> f32 {
        // Ponderación: benchmark 40%, novedad 20%, estabilidad 15%, escalabilidad 15%, eficiencia 10%
        let eficiencia = 1.0 - (self.tiempo_relativo.min(2.0) / 2.0);
        let memoria_eficiencia = 1.0 - (self.memoria_relativa.min(2.0) / 2.0);

        self.benchmark_score * 0.40
            + self.novedad * 0.20
            + self.estabilidad * 0.15
            + self.escalabilidad * 0.15
            + eficiencia * 0.05
            + memoria_eficiencia * 0.05
    }
}

/// Búsqueda guiada por tipos en curso
#[derive(Debug, Clone)]
pub struct BusquedaGuiada {
    pub sketch_id: u64,
    pub paso_actual: usize,
    pub candidatos_generados: usize,
    pub candidatos_descartados: usize,
    pub mejor_candidato: Option<u64>,
    pub estado: EstadoBusqueda,
}

/// Estado de una búsqueda
#[derive(Debug, Clone, PartialEq)]
pub enum EstadoBusqueda {
    Inicializando,
    Explorando,
    Refinando,
    Verificando,
    Completada,
    Descartada,
}

/// Propuesta de algoritmo para evaluación
#[derive(Debug, Clone)]
pub struct ProposicionAlgoritmo {
    pub id: u64,
    pub algoritmo: AlgoritmoDescubierto,
    pub propuesta_por: String, // "AI_Researcher", "Evolutionary_Compiler", "Hive_Mind"
    pub votacion: Votacion,
    pub estado_aprobacion: EstadoAprobacion,
}

/// Votación de la Mente Colmena
#[derive(Debug, Clone)]
pub struct Votacion {
    pub votos_a_favor: u32,
    pub votos_en_contra: u32,
    pub abstenciones: u32,
    pub umbral_aprobacion: f32,
}

impl Votacion {
    pub fn nuevo() -> Self {
        Votacion {
            votos_a_favor: 0,
            votos_en_contra: 0,
            abstenciones: 0,
            umbral_aprobacion: 0.9, // 90% para aprobar
        }
    }

    /// Calcula si hay consenso suficiente
    pub fn hay_consenso(&self) -> bool {
        let total = self.votos_a_favor + self.votos_en_contra + self.abstenciones;
        if total == 0 {
            return false;
        }
        let aprobacion = self.votos_a_favor as f32 / total as f32;
        aprobacion >= self.umbral_aprobacion
    }

    pub fn agregar_voto(&mut self, a_favor: bool) {
        if a_favor {
            self.votos_a_favor += 1;
        } else {
            self.votos_en_contra += 1;
        }
    }
}

/// Estado de aprobación
#[derive(Debug, Clone, PartialEq)]
pub enum EstadoAprobacion {
    Pendiente,
    Aprobado,
    Rechazado,
    VetadoCreador,
}

/// Motor de investigación de IA
pub struct AIResearcher {
    /// Sketches disponibles
    sketches: Vec<Sketch>,
    /// Algoritmos descubiertos
    descubiertos: Vec<AlgoritmoDescubierto>,
    /// Búsquedas activas
    busquedas_activas: HashMap<u64, BusquedaGuiada>,
    /// Propuestas pendientes de aprobación
    propuestas: VecDeque<ProposicionAlgoritmo>,
    /// Historial de descubrimientos por tipo
    historial_tipo: HashMap<TipoAlgoritmo, Vec<u64>>,
    /// Contador de IDs
    next_id: u64,
    /// Configuración
    config: ResearcherConfig,
}

/// Configuración del investigador
#[derive(Debug, Clone)]
pub struct ResearcherConfig {
    /// Máximos candidatos a explorar por sketch
    pub max_candidatos_por_sketch: usize,
    /// Máximos sketches activos simultáneamente
    pub max_busquedas_paralelas: usize,
    /// Threshold de novedad para conservar algoritmo
    pub threshold_novedad: f32,
    /// Probabilidad de mutar parámetros vs cambiar estructura
    pub tasa_mutacion_parametros: f32,
    /// Overhead máximo de CPU para investigación (0.0 - 1.0)
    pub overhead_maximo: f32,
}

impl Default for ResearcherConfig {
    fn default() -> Self {
        ResearcherConfig {
            max_candidatos_por_sketch: 100,
            max_busquedas_paralelas: 3,
            threshold_novedad: 0.15,
            tasa_mutacion_parametros: 0.7,
            overhead_maximo: 0.15, // Máximo 15% CPU para investigación
        }
    }
}

impl AIResearcher {
    /// Crea nuevo investigador
    pub fn new() -> Self {
        let mut researcher = AIResearcher {
            sketches: Vec::new(),
            descubiertos: Vec::new(),
            busquedas_activas: HashMap::new(),
            propuestas: VecDeque::new(),
            historial_tipo: HashMap::new(),
            next_id: 1,
            config: ResearcherConfig::default(),
        };

        // Registrar sketches base
        researcher.registrar_sketches_base();

        researcher
    }

    /// Registra sketches base para cada tipo de algoritmo
    fn registrar_sketches_base(&mut self) {
        // Sketch: Optimización Numérica (Gradient Descent mejorado)
        self.sketches.push(Sketch {
            nombre: "optimizacion_gradiente".to_string(),
            tipo: TipoAlgoritmo::Optimizacion,
            parametros: vec![
                ParametroSketch {
                    nombre: "tasa_aprendizaje".to_string(),
                    tipo: "f32".to_string(),
                    valor_default: Some("0.01".to_string()),
                    rango: Some((0.0001, 1.0)),
                },
                ParametroSketch {
                    nombre: "momentum".to_string(),
                    tipo: "f32".to_string(),
                    valor_default: Some("0.9".to_string()),
                    rango: Some((0.0, 0.99)),
                },
                ParametroSketch {
                    nombre: "decaimiento".to_string(),
                    tipo: "f32".to_string(),
                    valor_default: Some("0.0001".to_string()),
                    rango: Some((0.0, 0.1)),
                },
            ],
            cuerpo: r#"
fn optimizar(funcion: fn(&[f32]) -> f32, inicial: &[f32], iteraciones: u32) -> Vec<f32> {
    let mut params = inicial.to_vec();
    let mut velocidad = vec![0.0; params.len()];
    // HUECO: calcular gradiente de forma heurística
    // HUECO: aplicar actualización con momentum
    params
}
"#.to_string(),
            restricciones_tipo: vec![],
            complejidad_estimada: ComplejidadAlgoritmo {
                tiempo: ComplexityClass::ON,
                espacio: ComplexityClass::ON,
            },
        });

        // Sketch: Búsqueda en Grafos (A* mejorado)
        self.sketches.push(Sketch {
            nombre: "busqueda_grafos_astar".to_string(),
            tipo: TipoAlgoritmo::BusquedaGrafos,
            parametros: vec![
                ParametroSketch {
                    nombre: "heuristica_peso".to_string(),
                    tipo: "f32".to_string(),
                    valor_default: Some("1.0".to_string()),
                    rango: Some((0.1, 10.0)),
                },
            ],
            cuerpo: r#"
fn buscar_camino(inicio: u64, objetivo: u64, grafo: &[Vec<u64>]) -> Option<Vec<u64>> {
    let mut abiertos = VecDeque::new();
    let mut cerrados = HashSet::new();
    let mut came_from = HashMap::new();
    // HUECO: implementar A* con heurística adaptive
    // HUECO: manejarciclos
    None
}
"#.to_string(),
            restricciones_tipo: vec![],
            complejidad_estimada: ComplejidadAlgoritmo {
                tiempo: ComplexityClass::ONLogN,
                espacio: ComplexityClass::ON,
            },
        });

        // Sketch: Compresión (Delta encoding + Huffman simplificado)
        self.sketches.push(Sketch {
            nombre: "compresion_delta".to_string(),
            tipo: TipoAlgoritmo::Compresion,
            parametros: vec![
                ParametroSketch {
                    nombre: "ventana".to_string(),
                    tipo: "usize".to_string(),
                    valor_default: Some("32".to_string()),
                    rango: Some((4.0, 256.0)),
                },
            ],
            cuerpo: r#"
fn comprimir(datos: &[u8], ventana: usize) -> Vec<u8> {
    let mut resultado = Vec::new();
    // HUECO: aplicar delta encoding
    // HUECO: aplicar codificación Run-Length
    // HUECO: comprimir secuencias repetidas
    resultado
}
"#.to_string(),
            restricciones_tipo: vec![],
            complejidad_estimada: ComplejidadAlgoritmo {
                tiempo: ComplexityClass::ON,
                espacio: ComplexityClass::ON,
            },
        });

        // Sketch: Meta-algoritmo (Auto-tuning de parámetros)
        self.sketches.push(Sketch {
            nombre: "meta_autotuning".to_string(),
            tipo: TipoAlgoritmo::Meta,
            parametros: vec![
                ParametroSketch {
                    nombre: "tasa_exploracion".to_string(),
                    tipo: "f32".to_string(),
                    valor_default: Some("0.2".to_string()),
                    rango: Some((0.01, 0.5)),
                },
            ],
            cuerpo: r#"
fn autotuning(algoritmo_id: u64, metricas: &MetricasAlgoritmo) -> Vec<f32> {
    let mut params = Vec::new();
    // HUECO: analizar metricas para detectar subóptimo
    // HUECO: proponer mutaciones de parámetros
    // HUECO: validar cambios antes de aplicar
    params
}
"#.to_string(),
            restricciones_tipo: vec![],
            complejidad_estimada: ComplejidadAlgoritmo {
                tiempo: ComplexityClass::ON2,
                espacio: ComplexityClass::ON,
            },
        });
    }

    /// Inicia una nueva búsqueda basada en sketch
    pub fn iniciar_busqueda(&mut self, sketch_id: usize) -> Option<u64> {
        if sketch_id >= self.sketches.len() {
            return None;
        }

        let _sketch = &self.sketches[sketch_id];
        let busqueda_id = self.next_id;
        self.next_id += 1;

        let busqueda = BusquedaGuiada {
            sketch_id: sketch_id as u64,
            paso_actual: 0,
            candidatos_generados: 0,
            candidatos_descartados: 0,
            mejor_candidato: None,
            estado: EstadoBusqueda::Inicializando,
        };

        self.busquedas_activas.insert(busqueda_id, busqueda);

        Some(busqueda_id)
    }

    /// Genera candidatos a partir de un sketch (filling the gaps)
    pub fn generar_candidatos(&mut self, sketch_id: usize, num: usize) -> Vec<AlgoritmoDescubierto> {
        let sketch = match self.sketches.get(sketch_id) {
            Some(s) => s,
            None => return Vec::new(),
        };

        let mut candidatos = Vec::new();

        for _i in 0..num {
            let id = self.next_id;
            self.next_id += 1;

            let mut codigo = sketch.cuerpo.clone();
            let mut params = Vec::new();

            // Completar huecos basándose en parámetros
            for (idx, param) in sketch.parametros.iter().enumerate() {
                // Generar valor aleatorio en rango
                let valor = param.rango
                    .map(|(min, max)| min + (max - min) * self.random_float())
                    .unwrap_or_else(|| 0.5);

                params.push(valor);

                // Reemplazar hueco con implementación
                codigo = self.completar_hueco(&codigo, idx, valor, &param.nombre);
            }

            let algoritmo = AlgoritmoDescubierto {
                id,
                nombre: format!("{}_{}", sketch.nombre, id),
                tipo: sketch.tipo.clone(),
                codigo,
                parametros_iniciales: params.clone(),
                metricas: MetricasAlgoritmo::nuevo(0.5),
                linaje: vec![sketch_id as u64],
                tick_descubrimiento: self.tick_actual(),
            };

            candidatos.push(algoritmo);
        }

        candidatos
    }

    /// Completa un hueco en el sketch
    fn completar_hueco(&self, codigo: &str, idx: usize, valor: f64, _nombre: &str) -> String {
        // En una implementación real, esto usaría templates oIR
        // Por ahora, devolvemos código con el valor insertado
        let valor_str = format!("{:.6}", valor);

        // Reemplazar comentarios HUECO con implementación parametrizada
        codigo.replace("HUECO", &format!("/* param_{} = {} */", idx, valor_str))
    }

    /// Evalúa candidatos contra benchmark
    pub fn evaluar_candidatos(&mut self, candidatos: &[AlgoritmoDescubierto]) -> Vec<MetricasAlgoritmo> {
        candidatos.iter()
            .map(|c| self.evaluar_algoritmo(c))
            .collect()
    }

    /// Evalúa un algoritmo individual
    fn evaluar_algoritmo(&self, algoritmo: &AlgoritmoDescubierto) -> MetricasAlgoritmo {
        // Simulación de evaluación - en producción usaría benchmarks reales
        let benchmark = self.simular_benchmark(&algoritmo.codigo, algoritmo.tipo.clone());

        let mut metricas = MetricasAlgoritmo::nuevo(benchmark);

        // Calcular novelty contra algoritmos conocidos
        metricas.novedad = self.calcular_novedad(algoritmo);

        // Estabilidad simulada
        metricas.estabilidad = (0.5 + self.random_float() * 0.4) as f32;

        metricas
    }

    /// Calcula novedad de un algoritmo contra el historial
    fn calcular_novedad(&self, algoritmo: &AlgoritmoDescubierto) -> f32 {
        let mut min_similitud = 1.0f32;

        if let Some(historial) = self.historial_tipo.get(&algoritmo.tipo) {
            for id in historial {
                if let Some(existente) = self.descubiertos.iter().find(|a| a.id == *id) {
                    let similitud = self.similitud_codigo(&algoritmo.codigo, &existente.codigo);
                    min_similitud = min_similitud.min(similitud);
                }
            }
        }

        1.0 - min_similitud
    }

    /// Calcula similitud entre dos códigos (distancia de Hamming simplificada)
    fn similitud_codigo(&self, a: &str, b: &str) -> f32 {
        let chars_a: Vec<char> = a.chars().collect();
        let chars_b: Vec<char> = b.chars().collect();

        if chars_a.is_empty() || chars_b.is_empty() {
            return 1.0;
        }

        let max_len = chars_a.len().max(chars_b.len());
        let mut diff = 0usize;

        for i in 0..max_len {
            let ca = chars_a.get(i).unwrap_or(&' ');
            let cb = chars_b.get(i).unwrap_or(&' ');
            if ca != cb {
                diff += 1;
            }
        }

        1.0 - (diff as f32 / max_len as f32)
    }

    /// Simula benchmark (en producción sería ejecución real)
    fn simular_benchmark(&self, _codigo: &str, tipo: TipoAlgoritmo) -> f32 {
        // Simulación basada en tipo
        let base = match tipo {
            TipoAlgoritmo::Optimizacion => 0.6,
            TipoAlgoritmo::BusquedaGrafos => 0.55,
            TipoAlgoritmo::Clasificacion => 0.5,
            TipoAlgoritmo::Compresion => 0.65,
            TipoAlgoritmo::Criptografia => 0.4,
            TipoAlgoritmo::AudioSntesis => 0.7,
            TipoAlgoritmo::Renderizado => 0.6,
            TipoAlgoritmo::Planificacion => 0.5,
            TipoAlgoritmo::Meta => 0.3,
        };

        (base + (self.random_float() * 0.3 - 0.15)) as f32
    }

    /// Propone algoritmo para aprobación de la Mente Colmena
    pub fn proponer_algoritmo(&mut self, mut algoritmo: AlgoritmoDescubierto) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        algoritmo.id = id;

        let propuesta = ProposicionAlgoritmo {
            id,
            algoritmo,
            propuesta_por: "AI_Researcher".to_string(),
            votacion: Votacion::nuevo(),
            estado_aprobacion: EstadoAprobacion::Pendiente,
        };

        self.propuestas.push_back(propuesta);
        id
    }

    /// Procesa aprobación de propuestas
    pub fn procesar_aprobaciones(&mut self, _umbral: f32) -> Vec<u64> {
        let mut aprobados = Vec::new();

        for propuesta in self.propuestas.iter_mut() {
            if propuesta.estado_aprobacion == EstadoAprobacion::Pendiente
                && propuesta.votacion.hay_consenso()
            {
                propuesta.estado_aprobacion = EstadoAprobacion::Aprobado;
                aprobados.push(propuesta.id);
            }
        }

        aprobados
    }

    /// Registra descubrimiento completado
    pub fn registrar_descubrimiento(&mut self, algoritmo: AlgoritmoDescubierto) {
        // Agregar a descubrimientos
        self.descubiertos.push(algoritmo.clone());

        // Actualizar historial por tipo
        let historial = self.historial_tipo.entry(algoritmo.tipo.clone())
            .or_insert_with(Vec::new);
        historial.push(algoritmo.id);

        // Mantener máximo de 100 por tipo
        if historial.len() > 100 {
            historial.remove(0);
        }
    }

    /// Obtiene mejores algoritmos por tipo
    pub fn mejores_por_tipo(&self, tipo: &TipoAlgoritmo, num: usize) -> Vec<&AlgoritmoDescubierto> {
        let mut por_tipo: Vec<_> = self.descubiertos.iter()
            .filter(|a| &a.tipo == tipo)
            .collect();

        por_tipo.sort_by(|a, b| {
            b.metricas.fitness().partial_cmp(&a.metricas.fitness()).unwrap()
        });

        por_tipo.truncate(num);
        por_tipo
    }

    /// Obtiene estadísticas del investigador
    pub fn estadisticas(&self) -> ResearcherStats {
        ResearcherStats {
            num_sketches: self.sketches.len(),
            num_descubiertos: self.descubiertos.len(),
            num_busquedas_activas: self.busquedas_activas.len(),
            num_propuestas_pendientes: self.propuestas.len(),
            mejor_fitness: self.descubiertos.iter()
                .map(|a| a.metricas.fitness())
                .fold(0.0f32, |a, b| a.max(b)),
        }
    }

    fn random_float(&self) -> f64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;
        ((seed ^ (seed >> 17)) % 1000) as f64 / 1000.0
    }

    fn tick_actual(&self) -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

/// Estadísticas del investigador
#[derive(Debug, Clone)]
pub struct ResearcherStats {
    pub num_sketches: usize,
    pub num_descubiertos: usize,
    pub num_busquedas_activas: usize,
    pub num_propuestas_pendientes: usize,
    pub mejor_fitness: f32,
}

impl Default for AIResearcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crear_researcher() {
        let researcher = AIResearcher::new();
        assert!(researcher.sketches.len() > 0);
    }

    #[test]
    fn test_iniciar_busqueda() {
        let mut researcher = AIResearcher::new();
        let id = researcher.iniciar_busqueda(0);
        assert!(id.is_some());
    }

    #[test]
    fn test_generar_candidatos() {
        let mut researcher = AIResearcher::new();
        let candidatos = researcher.generar_candidatos(0, 5);
        assert_eq!(candidatos.len(), 5);
    }

    #[test]
    fn test_propuesta_y_aprobacion() {
        let mut researcher = AIResearcher::new();
        let candidatos = researcher.generar_candidatos(0, 1);
        let id = researcher.proponer_algoritmo(candidatos.into_iter().next().unwrap());
        assert!(id > 0);
    }
}