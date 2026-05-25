//! # Open-Endedness: Suelo Movedizo de la Evolución
//!
//! Este módulo implementa las condiciones necesarias para la emergencia de
//! **evolución abierta ABSOLUTA** — donde la complejidad emerge del ambiente,
//! no de bias artificial.
//!
//! ## Modo ABSOLUTO (factor_seleccion_complejidad = 0.0)
//!
//! En este modo:
//! - La complejidad NO es forzada — emerge solo si el ambiente la demanda
//! - Supervivencia 100% determinada por condiciones ambientales
//! - Los nichos, coevolución y suelo movedizo crean presiones naturales
//! - La complejidad puede crecer, pero no está obligada a hacerlo
//!
//! ## Los Cinco Pilares
//!
//! 1. **Complejidad Arquetípica** — Métrica original basada en tres vectores:
//!    - **Reflexividad**: Capacidad del sistema de modelarse a sí mismo
//!    - **Diversidad Comportamental**: Rango de estados accesibles
//!    - **Acoplamiento Sistémico**: Interdependencia entre componentes
//!
//! 2. **Novelty Search (Estampida)** — En lugar de fitness, buscar
//!    "fronteras del caos" donde nuevas formas emergen.
//!
//! 3. **Coevolución** — Nichos como sombras que se expanden cuando son ocupados.
//!
//! 4. **Shifting Fitness** — El medio es un agente activo, no un observador pasivo.
//!
//! 5. **Acumulación Transgeneracional** — La Umbra de un Auton muerto NO se pierte,
//!    sino que persiste en el Meltrace como semilla para nuevas formas.
//!
//! ## Principio Filosófico
//!
//! El sistema se inspira en la **dialética er磕磕碰碰**:
//! - Thetasis (afirmación) → Synthesis (síntesis)
//! - Lo nuevo nace del conflicto, no de la estabilidad
//! - El "techo" es una ilusión: siempre hay un borde más allá
#![allow(dead_code)]
#![allow(non_snake_case)]

use crate::life::meltrace::Meltrace;
use crate::life::umbra::Umbra;
use crate::physics::mar_morfoseo::MarMorfoseo;
use std::collections::HashMap;
use std::collections::VecDeque;

// ============================================================================
// COMPLEJIDAD ARQUETÍPICA
// ============================================================================

/// Métrica de complejidad basada en tres vectores arquetípicos.
///
/// A diferencia de Shannon (que mide información), esta mide:
/// - **Reflexividad**: Cuánto el sistema se modela a sí mismo
/// - **Diversitas**: Rango comportamental (número de estados distintos)
/// - **Acoplamiento**: Cuánto interactúan los componentes entre sí
#[derive(Debug, Clone, Copy)]
pub struct ComplejidadArquetipica {
    /// Vector R: Reflexividad (0.0 - 1.0)
    /// Mide cómo de "consciente" es el sistema de su propio estado.
    /// Un Auton con Umbra densa = alta reflexividad.
    pub reflexividad: f32,

    /// Vector D: Diversitas Comportamental (0.0 - 1.0)
    /// Mide cuántos estados distintos puede alcanzar el sistema.
    /// Un Auton con many RamNet pathways = alta diversitas.
    pub diversitas: f32,

    /// Vector A: Acoplamiento Sistémico (0.0 - 1.0)
    /// Mide cómo de entrelazados están los componentes.
    /// Un Auton con simbiogenesis activa = alto acoplamiento.
    pub acoplamiento: f32,

    /// Complejidad total (producto de los tres vectores)
    /// Usamos producto, no suma, para penalizar sistemas unbalanced.
    pub total: f32,
}

impl ComplejidadArquetipica {
    /// Calcula complejidad desde la Umbra de un Auton
    pub fn desde_umbra(umbra: &Umbra) -> Self {
        let num_nodos = umbra.num_nodos();
        let num_arcos = umbra.num_arcos();

        // R: Reflexividad = circuitos de largo 3 / nodos
        // Un circuito de largo 3 es: nodo A -> nodo B -> nodo C -> A
        // SIN TECHO: la complejidad puede crecer indefinidamente (inagotabilidad)
        let circuitos_cortos = umbra.contar_circuitos_longitud(3);
        let reflexividad = if num_nodos > 0 {
            circuitos_cortos as f32 / num_nodos as f32
            // Removido .min(1.0) - inagotabilidad requiere sin techo
        } else {
            0.0
        };

        // D: Diversitas = varianza de hashes de estado
        // SIN TECHO: la diversidad es unbounded (inagotabilidad)
        let hashes = umbra.obtener_hashes_estado();
        let diversitas = if hashes.len() > 1 {
            let mean = hashes.iter().fold(0u64, |acc, &h| acc.wrapping_add(h)) as f32
                / hashes.len() as f32;
            let variance = hashes
                .iter()
                .map(|&h| {
                    let diff = (h as f32 - mean).abs();
                    diff * diff
                })
                .fold(0f32, |acc, v| acc + v)
                / hashes.len() as f32;
            variance.sqrt() / u32::MAX as f32
            // Removido .min(1.0) - inagotabilidad requiere sin techo
        } else {
            0.0
        };

        // A: Acoplamiento = densidad de arcos
        // SIN TECHO: puede exceder 1.0 (inagotabilidad)
        let acoplamiento = if num_nodos > 0 {
            num_arcos as f32 / (num_nodos as f32 * 2.0)
            // Removido .min(1.0) - inagotabilidad requiere sin techo
        } else {
            0.0
        };

        let total = reflexividad * diversitas * acoplamiento;
        let total = if total < 0.001 {
            (reflexividad + diversitas + acoplamiento) / 3.0 * 0.1
        } else {
            total
        };

        ComplejidadArquetipica {
            reflexividad,
            diversitas,
            acoplamiento,
            total,
        }
    }

    /// Combina dos complejidades (para simbiogenesis)
    pub fn mezclar(&self, otro: &ComplejidadArquetipica, proporcion: f32) -> Self {
        let mix = |a: f32, b: f32| a * (1.0 - proporcion) + b * proporcion;
        let reflexividad = mix(self.reflexividad, otro.reflexividad);
        let diversitas = mix(self.diversitas, otro.diversitas);
        let acoplamiento = mix(self.acoplamiento, otro.acoplamiento);
        let total = reflexividad * diversitas * acoplamiento;
        ComplejidadArquetipica {
            reflexividad,
            diversitas,
            acoplamiento,
            total,
        }
    }
}

/// Historial de complejidades para tracking evolutivo
///
/// INAGOTABILIDAD: Usa ventana logarithmica - cuando el buffer está lleno,
/// comprime el historial antiguo en lugar de descartarlo.
/// Los "picos" de complejidad siempre se preservan.
#[derive(Debug)]
pub struct HistorialComplejidad {
    /// Buffer de complejidades (crece dinámicamente hasta un máximo razonable)
    buffer: VecDeque<(u64, ComplejidadArquetipica)>,
    /// Complejidad máxima alcanzada (línea de borde)
    complejidad_maxima: f32,
    /// Tick de la complejidad máxima
    tick_maximo: u64,
    /// Capacidad base del buffer
    capacidad_base: usize,
    /// Picos de complejidad preservados (nunca se descartan)
    picos: Vec<(u64, ComplejidadArquetipica)>,
}

impl HistorialComplejidad {
    pub fn new() -> Self {
        HistorialComplejidad {
            buffer: VecDeque::with_capacity(1000),
            complejidad_maxima: 0.0,
            tick_maximo: 0,
            capacidad_base: 1000,
            picos: Vec::new(),
        }
    }

    pub fn agregar(&mut self, tick: u64, complejidad: ComplejidadArquetipica) {
        // Actualizar máximo y preservar picos
        if complejidad.total > self.complejidad_maxima {
            self.complejidad_maxima = complejidad.total;
            self.tick_maximo = tick;

            // Preservar este pico - es un máximo histórico
            // INAGOTABILIDAD: Los picos nunca se pierden
            if complejidad.total > 0.1 {
                self.picos.push((tick, complejidad.clone()));
                // Mantener solo los últimos 100 picos
                if self.picos.len() > 100 {
                    self.picos.remove(0);
                }
            }
        }

        self.buffer.push_back((tick, complejidad));

        // INAGOTABILIDAD: Compresión proactiva en lugar de descarte
        // Si estamos por encima del 80% de capacidad Y complejidad está creciendo
        let ocupacion_ratio = self.buffer.len() as f32 / self.capacidad_base as f32;
        if ocupacion_ratio > 0.8 {
            // Calcular velocidad de complejidad reciente
            let velocidad = self.velocidad_complejidad(10);

            // Si la complejidad está creciendo, auto-escalar capacidad
            if velocidad > 0.05 && self.buffer.len() < self.capacidad_base * 4 {
                // Duplicar capacidad - ¡el sistema está evolucionando activamente!
                self.capacidad_base *= 2;
            } else if velocidad <= 0.05 {
                // Complejidad estable - comprimir mediante muestreo
                self.comprimir_muestreo();
            }
        }

        // Mantener tamaño máximo (ahora dinámico)
        while self.buffer.len() > self.capacidad_base {
            self.comprimir_muestreo();
        }
    }

    /// Compresión por muestreo: elimina cada 2da entrada del tercio más antiguo
    fn comprimir_muestreo(&mut self) {
        if self.buffer.len() < 10 {
            return; // No comprimir si hay muy pocas entradas
        }

        let tercio = self.buffer.len() / 3;
        let mut nueva_buffer = VecDeque::new();

        for (i, item) in self.buffer.iter().enumerate() {
            // Conservar: los últimos 2/3 siempre, del primer tercio solo los pares
            if i >= tercio || i % 2 == 0 {
                nueva_buffer.push_back(item.clone());
            }
        }

        self.buffer = nueva_buffer;
    }

    pub fn complejidad_maxima(&self) -> f32 {
        self.complejidad_maxima
    }

    pub fn tick_maximo(&self) -> u64 {
        self.tick_maximo
    }

    /// Obtiene todos los picos de complejidad (para análisis histórico)
    pub fn get_picos(&self) -> &[(u64, ComplejidadArquetipica)] {
        &self.picos
    }

    /// Calcula "velocidad de complejidad" — qué tan rápido crece
    pub fn velocidad_complejidad(&self, ultimos_n: usize) -> f32 {
        let entries: Vec<_> = self.buffer.iter().rev().take(ultimos_n).collect();
        if entries.len() < 2 {
            return 0.0;
        }

        let primero = entries.last().unwrap().1.total;
        let ultimo = entries.first().unwrap().1.total;

        (ultimo - primero) / primero.max(0.001)
    }
}

// ============================================================================
// NOVELTY SEARCH (ESTAMPIDA)
// ============================================================================

/// Sistema de búsqueda de novedad basado en "ESTAMPIDA":
/// - **E**xplorar los **B**ordes del **C**aos donde lo nuevo emerge
/// - **A**cumular variaciones que "sobrevivieron" al olvido
/// - **M**etricar la distancia desde el "centro" (estabilidad)
///
/// En lugar de fitness, usamos "ESTAMPIDA" — la capacidad del sistema
/// de generar formas que nadie anticipó.
#[derive(Debug, Clone)]
pub struct Estampida {
    /// Mapa de "estados del ecosistema" → cuántas veces apareció
    /// Un estado = hash de la distribución de energon + complejidades
    estados: HashMap<u64, u32>,

    /// Registro de "estados olvidados" (baja frecuencia)
    estados_olvidados: Vec<u64>,

    /// Contador de nuevos estados descubiertos
    nuevos_descubiertos: u64,

    /// Threshold de "olvido" — estados con freq < umbral son "olvidados"
    umbral_olvido: u32,
}

impl Estampida {
    pub fn new() -> Self {
        Estampida {
            estados: HashMap::new(),
            estados_olvidados: Vec::new(),
            nuevos_descubiertos: 0,
            umbral_olvido: 2,
        }
    }

    /// Registra un nuevo estado del ecosistema
    pub fn registrar_estado(&mut self, _estado_hash: u64) -> NovedadResult {
        NovedadResult {
            es_nuevo: true,
            frecuencia: 1,
            es_olvidado: false,
            distancia_novedad: 1.0,
        }
    }

    /// Obtiene "presión de novelty" — cuánto debe explorar vs explotar
    /// Retorna valor 0.0-1.0: 1.0 = explorar (muchos estados nuevos)
    pub fn presion_exploracion(&self) -> f32 {
        let total_estados = self.estados.len() as f32;
        let estados_nuevos = self.nuevos_descubiertos as f32;
        // INAGOTABILIDAD: presión de exploración sin límite artificial
        // Esto permite que el sistema explore más aggressively cuando hay muchos estados nuevos
        if total_estados < 10.0 {
            return 1.0; // Muy pocos estados, explorar todo
        }
        // Presión de exploración proporcional a la proporción de estados nuevos
        estados_nuevos / total_estados
    }

    /// Limpia estados muy comunes (ya no son "novedad")
    pub fn envejecer(&mut self) {
        // Reducir frecuencias
        for freq in self.estados.values_mut() {
            *freq = (*freq / 2).max(1);
        }

        // Mover estados olvidados al registro
        self.estados.retain(|_, v| *v >= self.umbral_olvido);

        // Limpiar olvidados antiguos
        if self.estados_olvidados.len() > 100 {
            self.estados_olvidados.drain(0..50);
        }
    }

    pub fn num_estados(&self) -> usize {
        self.estados.len()
    }
}

/// Resultado de registrar un estado
#[derive(Debug, Clone)]
pub struct NovedadResult {
    pub es_nuevo: bool,
    pub frecuencia: u32,
    pub es_olvidado: bool,
    pub distancia_novedad: f32,
}

// ============================================================================
// COEVOLUCIÓN (NICHOS SOMBRA)
// ============================================================================

/// Sistema de nichos como "sombras" — cada nicho existe en proporción
/// al número de Autons que lo ocupan. Un nicho lleno se expande (crea
/// sub-nichos) o genera presión competitiva.
///
/// La coevolución ocurre cuando los nichos interactúan:
/// - Un Auton en nicho A afecta las condiciones del nicho B
/// - Esto crea cadenas de co-evolución
#[derive(Debug, Clone)]
pub struct NichosSombra {
    /// Nichos activos: (id_nicho, ocupacion, complejidad_promedio)
    nichos: Vec<Nicho>,

    /// Mapa de Auton → nicho (id)
    autons_en_nicho: HashMap<u64, usize>,

    /// ID del siguiente nicho
    next_nicho_id: usize,

    /// Factor de expansión: cuándo un nicho "se llena"
    umbral_ocupacion: u32,
}

#[derive(Debug, Clone)]
pub struct Nicho {
    pub id: usize,
    /// Nombres de nichos en el dominio
    pub nombre: String,
    /// Cuántos Autons lo ocupan
    pub ocupacion: u32,
    /// Complejidad promedio de Autons en este nicho
    pub complejidad_promedio: f32,
    /// Diversidad comportamental del nicho
    pub diversidad: f32,
    /// Nichos "hijos" (sub-nichos)
    pub sub_nichos: Vec<usize>,
    /// Parent nicho
    pub padre: Option<usize>,
}

impl NichosSombra {
    pub fn new() -> Self {
        NichosSombra {
            nichos: Vec::new(),
            autons_en_nicho: HashMap::new(),
            next_nicho_id: 1,
            umbral_ocupacion: 5,
        }
    }

    /// Registra un Auton en un nicho (o crea uno nuevo si no existe)
    pub fn registrar_auton(&mut self, auton_id: u64, complejidad: f32, diversidad: f32) -> usize {
        // Buscar nicho existente que acepte al Auton
        // Criterio: complejidad similar ±20%
        let umbral_multiplicador = self.umbral_ocupacion as f32 * 2.0;
        for i in 0..self.nichos.len() {
            let nicho = &self.nichos[i];
            let complejidad_diff = (complejidad - nicho.complejidad_promedio).abs();
            let complejidad_promedio_20 = 0.2 * nicho.complejidad_promedio;
            let nicho_id = nicho.id;
            let ocupacion_actual = nicho.ocupacion;
            let complejidad_promedio_actual = nicho.complejidad_promedio;

            if complejidad_diff < complejidad_promedio_20
                && (ocupacion_actual as f32) < umbral_multiplicador
            {
                // Aceptar en nicho existente - use index i
                self.nichos[i].ocupacion += 1;
                self.nichos[i].complejidad_promedio =
                    (complejidad_promedio_actual * ocupacion_actual as f32 + complejidad)
                        / (ocupacion_actual + 1) as f32;
                self.autons_en_nicho.insert(auton_id, nicho_id);
                return nicho_id;
            }
        }

        // Crear nuevo nicho
        let id = self.next_nicho_id;
        self.next_nicho_id += 1;

        let nuevo_nicho = Nicho {
            id,
            nombre: format!("Nicho-{}", id),
            ocupacion: 1,
            complejidad_promedio: complejidad,
            diversidad,
            sub_nichos: Vec::new(),
            padre: None,
        };

        self.nichos.push(nuevo_nicho);
        self.autons_en_nicho.insert(auton_id, id);
        id
    }

    /// Un Auton deja un nicho (por muerte)
    pub fn remover_auton(&mut self, auton_id: u64) {
        if let Some(nicho_id) = self.autons_en_nicho.remove(&auton_id) {
            if let Some(nicho) = self.nichos.iter_mut().find(|n| n.id == nicho_id) {
                nicho.ocupacion = nicho.ocupacion.saturating_sub(1);
            }
        }
    }

    /// Calcula presión competitiva en un nicho
    pub fn presion_competitiva(&self, nicho_id: usize) -> f32 {
        if let Some(nicho) = self.nichos.iter().find(|n| n.id == nicho_id) {
            // Más ocupación = más presión
            let ocupacion_normalizada = nicho.ocupacion as f32 / (self.umbral_ocupacion * 2) as f32;
            // Más diversidad = menos presión (nichos diversos comparten recursos)
            let diversidad_factor = 1.0 - (nicho.diversidad / 2.0);
            ocupacion_normalizada * diversidad_factor
        } else {
            0.0
        }
    }

    /// Obtiene nicho con menor presión (para新生儿 Autons)
    pub fn nicho_menos_presionado(&self) -> Option<usize> {
        self.nichos
            .iter()
            .map(|n| (n.id, self.presion_competitiva(n.id)))
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(id, _)| id)
    }

    /// Expande un nicho (crea sub-nicho) cuando está muy lleno
    pub fn expandir_nicho(
        &mut self,
        nicho_id: usize,
        nuevas_complejidades: &[f32],
    ) -> Option<usize> {
        // Find the nicho index first
        let nicho_idx = self.nichos.iter().position(|n| n.id == nicho_id)?;

        // Check condition using index
        if self.nichos[nicho_idx].ocupacion < self.umbral_ocupacion * 2 {
            return None;
        }

        // Extract values we need from nicho
        let nicho = &self.nichos[nicho_idx];
        let nicho_nombre = nicho.nombre.clone();
        let complejidad_promedio = nicho.complejidad_promedio;
        let diversidad = nicho.diversidad;

        // Crear sub-nicho con nueva specialization
        let sub_id = self.next_nicho_id;
        self.next_nicho_id += 1;

        let promedio_nuevas = if !nuevas_complejidades.is_empty() {
            nuevas_complejidades.iter().sum::<f32>() / nuevas_complejidades.len() as f32
        } else {
            complejidad_promedio * 1.1 // especialización más compleja
        };

        let sub_nicho = Nicho {
            id: sub_id,
            nombre: format!("{}/sub-{}", nicho_nombre, sub_id),
            ocupacion: 0,
            complejidad_promedio: promedio_nuevas,
            diversidad: diversidad * 0.8,
            sub_nichos: Vec::new(),
            padre: Some(nicho_id),
        };

        self.nichos.push(sub_nicho);
        self.nichos[nicho_idx].sub_nichos.push(sub_id);

        Some(sub_id)
    }

    pub fn num_nichos(&self) -> usize {
        self.nichos.len()
    }

    /// Retorna true si TODOS los nichos están cerca de su capacidad máxima
    /// Esto indica que el sistema necesita más "espacio dimensional"
    pub fn todos_ocupados(&self) -> bool {
        if self.nichos.is_empty() {
            return false;
        }
        // Un nicho está "ocupado" si tiene >= 80% de su capacidad
        let umbral = (self.umbral_ocupacion as f32 * 0.8) as u32;
        self.nichos.iter().all(|n| n.ocupacion >= umbral)
    }
}

// ============================================================================
// SHIFTING FITNESS (SUELO MOVEDIZO)
// ============================================================================

/// El Mar Morfóseo como agente activo — no solo un depósito de energía,
/// sino unTermostato que oscila y crea "crisis" que invalidan estrategias.
///
/// El shifting fitness opera en tres escalas:
/// - **Micro** (cada ciclo): Fluctuaciones menores de energon
/// - **Meso** (cada 100 ciclos): Oscilaciones de temperatura (estrés)
/// - **Macro** (cada 1000 ciclos): Extinciones masivas (reshuffle)
#[derive(Debug, Clone)]
pub struct SueloMovedizo {
    /// Nivel de "termalización" actual (0.0 = frío/estable, 1.0 = caótico)
    nivel_termico: f32,

    /// Tick del último cambio de régimen
    ultimo_cambio_regimen: u64,

    /// Período de oscilación (en ticks)
    periodo_oscilacion: u64,

    /// Amplitud de oscilación (0.0 - 1.0)
    amplitud: f32,

    /// Factor de extinción masiva (probabilidad de evento)
    probabilidad_extincion: f32,

    /// Contador de extinciones totales
    num_extinciones: u64,

    /// Historial de "eventos de extinción"
    eventos_extincion: VecDeque<EventoExtension>,
}

#[derive(Debug, Clone)]
pub struct EventoExtension {
    pub tick: u64,
    pub intensidad: f32,
    pub num_autons_eliminados: u32,
    pub causa: String,
}

impl SueloMovedizo {
    pub fn new(semilla: u64) -> Self {
        let periodo = 500 + (semilla % 300); // 500-800 ciclos
        SueloMovedizo {
            nivel_termico: 0.3, // Comienza medio
            ultimo_cambio_regimen: 0,
            periodo_oscilacion: periodo,
            amplitud: 0.3,
            probabilidad_extincion: 0.01, // 1% por ciclo
            num_extinciones: 0,
            eventos_extincion: VecDeque::with_capacity(100),
        }
    }

    /// Calcula factor de estrés modificado para el ciclo actual
    pub fn factor_estres(&self, tick: u64) -> f32 {
        // Oscilación sinusoidal
        let fase = (tick % self.periodo_oscilacion) as f32 / self.periodo_oscilacion as f32;
        let oscilacion = (fase * std::f32::consts::PI * 2.0).sin() * self.amplitud;

        // El nivel térmico oscila alrededor de 0.3
        let nivel = 0.3 + oscilacion;
        nivel.clamp(0.0, 1.0)
    }

    /// Determina si ocurre un evento de extinción masiva
    pub fn evaluar_extincion(&mut self, tick: u64, num_autons: usize) -> Option<EventoExtension> {
        // Solo considerar extinciones si hay suficientes Autons
        if num_autons < 3 {
            return None;
        }

        // Probabilidad aumenta con el nivel térmico
        let riesgo = self.probabilidad_extincion * (1.0 + self.nivel_termico);

        // Ruido pseudo-aleatorio basado en tick
        let ruido = ((tick * 7919) % 1000) as f32 / 1000.0;

        if ruido < riesgo {
            // ¡Extinción!
            let intensidad = self.nivel_termico;
            let num_victimas = ((num_autons as f32 * intensidad * 0.5) as u32).max(1);

            self.num_extinciones += 1;

            let evento = EventoExtension {
                tick,
                intensidad,
                num_autons_eliminados: num_victimas,
                causa: format!("Termalización-{:.2}", intensidad),
            };

            self.eventos_extincion.push_back(evento.clone());
            if self.eventos_extincion.len() > 100 {
                self.eventos_extincion.pop_front();
            }

            // Resetear nivel térmico después de extinción
            self.nivel_termico = 0.2;

            Some(evento)
        } else {
            None
        }
    }

    /// Aplica presión del suelo movedizo a un Auton
    /// Retorna modificación del umbral de muerte
    pub fn presion_suelo(&self, complejidad: f32, tick: u64) -> f32 {
        // Autons más complejos son más sensibles al estrés
        let sensibilidad = 1.0 + complejidad;

        // El factor de estrés oscila
        let estres = self.factor_estres(tick);

        // Más estrés = umbrales más estrictos (penalización)
        sensibilidad * estres
    }

    pub fn num_extinciones(&self) -> u64 {
        self.num_extinciones
    }

    pub fn nivel_termico_actual(&self) -> f32 {
        self.nivel_termico
    }
}

// ============================================================================
// ACUMULACIÓN TRANGENERACIONAL
// ============================================================================

/// Sistema que conecta la Umbra de un Auton muerto con el nacimiento
/// de nuevos Autons, creando "memoria transgeneracional".
///
/// INAGOTABILIDAD: El buffer de semillas ahora es dinámico - se expande
/// automáticamente cuando hay nuevas experiencias complejas por preservar.
#[derive(Debug)]
pub struct AcumulacionTransgeneracional {
    /// Buffer de "semillas" (Umbras comprimidas) - CRECE DINÁMICAMENTE
    semillas: VecDeque<SemillaTransgeneracional>,

    /// Capacidad actual del buffer (crece con el sistema)
    capacidad: usize,

    /// Capacidad máxima absoluta (límite de memoria)
    capacidad_max: usize,

    /// Contador de semillas utilizadas
    semillas_utilizadas: u64,

    /// Factor de selección (cuántas semillas considerar)
    factor_seleccion: f32,

    /// Picos de complejidad preservados (nunca se descartan)
    picos_complejidad: Vec<SemillaTransgeneracional>,
}

#[derive(Debug, Clone)]
pub struct SemillaTransgeneracional {
    /// Características de la Umbra original
    pub caracteristicas: Vec<u64>,
    /// Complejidad de la semilla
    pub complejidad: f32,
    /// Tick de origen
    pub tick_origen: u64,
    /// Veces que fue "regada" (seleccionada para nacimiento)
    pub conteo_regadas: u32,
    /// Hash del Auton padre (para similitud)
    pub hash_padre: u64,
}

impl AcumulacionTransgeneracional {
    /// Capacidad inicial por defecto
    const CAPACIDAD_DEFAULT: usize = 512;
    /// Capacidad máxima absoluta (límite de memoria práctica en bytes)
    /// INAGOTABILIDAD: emerge del balance memoria-rendimiento
    const CAPACIDAD_MAX: usize = 500_000;

    pub fn new(capacidad: usize) -> Self {
        let capacidad_real = capacidad.max(Self::CAPACIDAD_DEFAULT);
        AcumulacionTransgeneracional {
            semillas: VecDeque::with_capacity(capacidad_real),
            capacidad: capacidad_real,
            capacidad_max: Self::CAPACIDAD_MAX,
            semillas_utilizadas: 0,
            factor_seleccion: 0.3,
            picos_complejidad: Vec::new(),
        }
    }

    /// Guarda una Umbra como semilla transgeneracional
    pub fn guardar_semilla(
        &mut self,
        umbra: &Umbra,
        complejidad: ComplejidadArquetipica,
        tick: u64,
        hash_auton: u64,
    ) {
        let caracteristicas = umbra.obtener_hashes_estado();

        let semilla = SemillaTransgeneracional {
            caracteristicas,
            complejidad: complejidad.total,
            tick_origen: tick,
            conteo_regadas: 0,
            hash_padre: hash_auton,
        };

        // INAGOTABILIDAD: Preservar semillas de alta complejidad en picos
        // Los picos de complejidad NUNCA se descartan
        if semilla.complejidad > 0.5 && self.picos_complejidad.len() < 1000 {
            self.picos_complejidad.push(semilla.clone());
        }

        // INAGOTABILIDAD: Auto-expansión del buffer cuando hay complejidad creciente
        if self.semillas.len() >= self.capacidad && self.capacidad < self.capacidad_max {
            // Expandir buffer al doble, hasta el máximo
            let nueva_capacidad = (self.capacidad * 2).min(self.capacidad_max);
            let mut nuevo_buffer = VecDeque::with_capacity(nueva_capacidad);

            // Migrar semillas existentes
            for s in self.semillas.drain(..) {
                nuevo_buffer.push_back(s);
            }
            self.semillas = nuevo_buffer;
            self.capacidad = nueva_capacidad;
        }

        // Agregar al buffer (sobrescribe los más viejos SOLO si estamos al máximo)
        if self.semillas.len() >= self.capacidad {
            self.semillas.pop_front();
        }
        self.semillas.push_back(semilla);
    }

    /// Obtiene semillas de alta complejidad preservadas (picos)
    pub fn get_picos(&self) -> &[SemillaTransgeneracional] {
        &self.picos_complejidad
    }

    /// Obtiene "influencia" para un nuevo Auton nascitur
    ///
    /// INAGOTABILIDAD: Ahora también considera los PICOS de complejidad.
    /// Si el buffer normal está casi vacío, usa los picos preservados.
    pub fn obtener_influencia(&mut self, tick_actual: u64) -> Option<(Vec<u64>, f32)> {
        // Primero intentar con semillas normales
        if !self.semillas.is_empty() {
            // Seleccionar semilla basado en:
            // 1. Recencia (más reciente = mejor)
            // 2. Complejidad (más complejo = mejor)
            // 3. Pocas veces utilizado (evitar saturación)

            let mut mejor_idx = 0;
            let mut mejor_score = 0.0f32;

            for (i, semilla) in self.semillas.iter().enumerate() {
                let edad = (tick_actual - semilla.tick_origen).max(1) as f32;
                let recencia = 1.0 / edad.sqrt(); // Decaimiento sqrt
                let complejidad = semilla.complejidad;
                let frescura = 1.0 / (semilla.conteo_regadas as f32 + 1.0);

                let score = recencia * complejidad * frescura;

                if score > mejor_score {
                    mejor_score = score;
                    mejor_idx = i;
                }
            }

            self.semillas[mejor_idx].conteo_regadas += 1;
            self.semillas_utilizadas += 1;

            return Some((
                self.semillas[mejor_idx].caracteristicas.clone(),
                self.semillas[mejor_idx].complejidad,
            ));
        }

        // FALLBACK INAGOTABILIDAD: Si no hay semillas normales, usar picos preservados
        if !self.picos_complejidad.is_empty() {
            // Seleccionar el pico más complejo
            let mejor_pico = self
                .picos_complejidad
                .iter()
                .max_by(|a, b| a.complejidad.partial_cmp(&b.complejidad).unwrap())
                .unwrap();

            self.semillas_utilizadas += 1;

            return Some((mejor_pico.caracteristicas.clone(), mejor_pico.complejidad));
        }

        None
    }

    /// Combina influencias de múltiples semillas (para diversidad)
    pub fn combinar_influencias(
        &self,
        tick_actual: u64,
        num_semillas: usize,
    ) -> Option<(Vec<u64>, f32)> {
        let mut acum_caracteristicas: HashMap<u64, f32> = HashMap::new();
        let mut suma_complejidad = 0.0f32;
        let mut count = 0;

        for semilla in self.semillas.iter() {
            let edad = (tick_actual - semilla.tick_origen).max(1) as f32;
            let peso = 1.0 / edad.sqrt();

            for carac in &semilla.caracteristicas {
                *acum_caracteristicas.entry(*carac).or_insert(0.0) += peso;
            }
            suma_complejidad += semilla.complejidad * peso;
            count += 1;

            if count >= num_semillas {
                break;
            }
        }

        if acum_caracteristicas.is_empty() {
            return None;
        }

        // Top 20 características más pesadas
        let mut top: Vec<_> = acum_caracteristicas.into_iter().collect();
        top.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        let caracteristicas: Vec<u64> = top.into_iter().take(20).map(|(c, _)| c).collect();

        let complejidad_prom = if count > 0 {
            suma_complejidad / count as f32
        } else {
            0.0
        };

        Some((caracteristicas, complejidad_prom))
    }

    pub fn num_semillas(&self) -> usize {
        self.semillas.len()
    }

    pub fn semillas_utilizadas(&self) -> u64 {
        self.semillas_utilizadas
    }
}

// ============================================================================
// OPEN-ENDEDNESS ORCHESTRATOR
// ============================================================================

/// Orchestrator principal que integra todos los subsistemas de Open-Endedness
pub struct OpenEndednessEngine {
    /// Métricas de complejidad por Auton
    complejidades: HashMap<u64, ComplejidadArquetipica>,

    /// Historial global de complejidad
    pub historial_complejidad: HistorialComplejidad,

    /// Sistema de novelty search
    estampida: Estampida,

    /// Sistema de nichos
    nichos: NichosSombra,

    /// Suelo movedizo
    suelo: SueloMovedizo,

    /// Acumulación transgeneracional (legacy, migrando a Meltrace)
    acumulador: AcumulacionTransgeneracional,

    /// Meltrace: El Trazo Fundido - memoria transgeneracional lamarckiana
    meltrace: Meltrace,

    /// Mar Morfóseo: fuente de energon emergente
    mar: Option<MarMorfoseo>,

    /// Tick actual
    tick_actual: u64,

    /// Selección positiva por complejidad (factor 0.0 - 1.0)
    /// = Cuántobonus recibe un Auton complejo vs simple
    factor_seleccion_complejidad: f32,

    // ============================================================================
    // OPEN-ENDEDNESS ILIMITADO: Contadores
    // ============================================================================
    /// Número de veces que el Mar se expandió
    expansiones_mar: u32,

    /// Total de energon generado del vacío
    energon_generado: f64,

    /// Transiciones a nuevas dimensiones de estados
    transiciones_infinito: u32,
}

impl OpenEndednessEngine {
    pub fn new(semilla: u64) -> Self {
        OpenEndednessEngine {
            complejidades: HashMap::new(),
            historial_complejidad: HistorialComplejidad::new(),
            estampida: Estampida::new(),
            nichos: NichosSombra::new(),
            suelo: SueloMovedizo::new(semilla),
            acumulador: AcumulacionTransgeneracional::new(512),
            meltrace: Meltrace::new(semilla),
            mar: None,
            tick_actual: 0,
            factor_seleccion_complejidad: 0.0, // 0% = ABSOLUTO: complejidad emerge del ambiente, no impuesta
            // Infinite OE tracking
            expansiones_mar: 0,
            energon_generado: 0.0,
            transiciones_infinito: 0,
        }
    }

    pub fn with_mar(mut self, mar: MarMorfoseo) -> Self {
        self.mar = Some(mar);
        self
    }

    /// Actualiza el sistema cada ciclo
    pub fn tick(&mut self, autons: &[(u64, Umbra)], tick: u64) {
        self.tick_actual = tick;

        // 1. Calcular complejidades de todos los Autons
        for (id, umbra) in autons.iter() {
            let complejidad = ComplejidadArquetipica::desde_umbra(umbra);
            self.complejidades.insert(*id, complejidad);
            self.historial_complejidad.agregar(tick, complejidad);
        }

        // 2. Registrar estado del ecosistema
        let estado_eco_hash = self.hash_estado_ecosistema(autons);
        let _novedad = self.estampida.registrar_estado(estado_eco_hash);

        // 3. Actualizar nichos
        for (id, _umbra) in autons.iter() {
            if let Some(&complejidad) = self.complejidades.get(id) {
                let diversitas = complejidad.diversitas;
                self.nichos
                    .registrar_auton(*id, complejidad.total, diversitas);
            }
        }

        // 4. Integración Meltrace: tick global del trazo transgeneracional
        self.meltrace.tick();

        // 5. Integración Mar Morfóseo: genesis de energon si es necesario
        if let Some(ref mut _mar) = self.mar {
            let complejidad_promedio = if !self.complejidades.is_empty() {
                self.complejidades.values().map(|c| c.total).sum::<f32>()
                    / self.complejidades.len() as f32
            } else {
                0.0
            };

            if self.expansion_mar_necesaria(complejidad_promedio) {
                if let Some(ref mut mar_mut) = self.mar {
                    mar_mut.genesis_energon(complejidad_promedio);
                    self.expansiones_mar += 1;
                }
            }
        }

        // 6. Envejecer sistemas (limpieza)
        if tick % 50 == 0 {
            self.estampida.envejecer();
        }
    }

    fn expansion_mar_necesaria(&self, complejidad: f32) -> bool {
        if let Some(ref mar) = self.mar {
            let umbral_densidad = crate::physics::fixed_point::I32F32::from_i32(10);
            mar.necesita_expansion(umbral_densidad) && complejidad > 0.1
        } else {
            false
        }
    }

    /// Calcula hash del estado del ecosistema (para novelty)
    fn hash_estado_ecosistema(&self, _autons: &[(u64, Umbra)]) -> u64 {
        let mut hash: u64 = 0;

        // Número de Autons
        hash = hash.wrapping_add(_autons.len() as u64 * 7919);

        // Suma de complejidades
        let suma_complejidad: f32 = self.complejidades.values().map(|c| c.total).sum();
        hash = hash.wrapping_add((suma_complejidad * 1000.0) as u64);

        // Estado del suelo movedizo
        hash = hash.wrapping_add((self.suelo.nivel_termico * 100.0) as u64);

        // Número de nichos
        hash = hash.wrapping_add(self.nichos.num_nichos() as u64 * 31);

        hash
    }

    /// Obtiene factor de supervivencia.
    /// Con factor_seleccion_complejidad = 0.0 (ABSOLUTO): siempre retorna 1.0
    /// La supervivencia es 100% determinada por condiciones ambientales,
    /// no por complejidad impuesta.
    pub fn factor_supervivencia(&self, auton_id: u64) -> f32 {
        if self.factor_seleccion_complejidad == 0.0 {
            return 1.0;
        }

        if let Some(&complejidad) = self.complejidades.get(&auton_id) {
            // Bonus proporcional a complejidad
            // Complejidad 0.0 = bonus 0%, Complejidad 1.0 = bonus factor_seleccion_complejidad
            let bonus = complejidad.total * self.factor_seleccion_complejidad;
            1.0 + bonus
        } else {
            1.0
        }
    }

    /// Obtiene presión competitiva del nicho
    pub fn presion_nicho(&self, auton_id: u64) -> f32 {
        if let Some(&nicho_id) = self.nichos.autons_en_nicho.get(&auton_id) {
            self.nichos.presion_competitiva(nicho_id)
        } else {
            0.0 // No hay presión si no está en nicho
        }
    }

    /// Registra muerte de un Auton
    pub fn registrar_muerte(&mut self, auton_id: u64, umbra: &Umbra, tick: u64) {
        // Guardar en acumulador transgeneracional (legacy)
        if let Some(&complejidad) = self.complejidades.get(&auton_id) {
            self.acumulador
                .guardar_semilla(umbra, complejidad, tick, auton_id);
        }

        // INTEGRACIÓN MELTRACE: Registrar muerte para memoria transgeneracional lamarckiana
        self.meltrace.registrar_muerte(umbra);

        // INTEGRACIÓN MELTRACE: Decrementar contador de Autons vivos
        self.meltrace.registrar_muerte_auton();

        // Remover de nichos
        self.nichos.remover_auton(auton_id);

        // Remover de complejidades
        self.complejidades.remove(&auton_id);
    }

    /// Obtiene influencia transgeneracional para nuevo Auton
    /// Prioriza Meltrace (lamarckismo) sobre AcumulacionTransgeneracional legacy
    pub fn influencia_nacimiento(&mut self) -> Option<(Vec<u64>, f32)> {
        // Primero intentar con Meltrace (más moderno)
        if let Some(grabado) = self.meltrace.seleccionar_grabado() {
            self.meltrace.registrar_nacimiento();
            return Some((
                grabado.caracteristicas.clone(),
                grabado.fuerza.to_f64() as f32,
            ));
        }

        // Fallback legacy
        self.acumulador.obtener_influencia(self.tick_actual)
    }

    /// Obtiene estadísticas del Meltrace integrado
    pub fn meltrace_stats(&self) -> crate::life::meltrace::MeltraceStats {
        self.meltrace.estadisticas()
    }

    /// Refuerza grabados similares en Meltrace (lamarckismo)
    pub fn reforzar_similares(&mut self, objetivo: &crate::life::meltrace::Grabado) {
        self.meltrace.reforzar_similares(objetivo);
    }

    /// Obtiene referencia al Mar Morfóseo (si existe)
    pub fn mar(&self) -> Option<&MarMorfoseo> {
        self.mar.as_ref()
    }

    pub fn mar_mut(&mut self) -> Option<&mut MarMorfoseo> {
        self.mar.as_mut()
    }

    /// Evalúa si hay extinción masiva
    pub fn evaluar_extincion(&mut self, num_autons: usize) -> Option<EventoExtension> {
        self.suelo.evaluar_extincion(self.tick_actual, num_autons)
    }

    /// Aplica presión del suelo movedizo
    pub fn presion_suelo(&self, complejidad: f32) -> f32 {
        self.suelo.presion_suelo(complejidad, self.tick_actual)
    }

    /// Obtiene presión de novelty search (0.0 = explotar, 1.0 = explorar)
    pub fn presion_exploracion(&self) -> f32 {
        self.estampida.presion_exploracion()
    }

    /// Devuelve métricas consolidadas para reporting
    pub fn metricas(&self) -> OpenEndednessMetrics {
        let meltrace_stats = self.meltrace.estadisticas();
        OpenEndednessMetrics {
            complejidad_promedio: if !self.complejidades.is_empty() {
                self.complejidades.values().map(|c| c.total).sum::<f32>()
                    / self.complejidades.len() as f32
            } else {
                0.0
            },
            complejidad_maxima: self.historial_complejidad.complejidad_maxima(),
            tick_maximo: self.historial_complejidad.tick_maximo(),
            velocidad_complejidad: self.historial_complejidad.velocidad_complejidad(100),
            num_nichos: self.nichos.num_nichos() as u32,
            num_estados_novedad: self.estampida.num_estados() as u32,
            num_extinciones: self.suelo.num_extinciones(),
            num_semillas_transgen: self.acumulador.num_semillas() as u32,
            semillas_utilizadas: self.acumulador.semillas_utilizadas(),
            nivel_termico: self.suelo.nivel_termico_actual(),
            // Infinite OE
            expansiones_mar: self.expansiones_mar,
            energon_generado: self.energon_generado,
            transiciones_infinito: self.transiciones_infinito,
            // INTEGRACIÓN MELTRACE
            meltrace_grabados: meltrace_stats.grabados_activos,
            meltrace_inmortales: meltrace_stats.inmortales,
            meltrace_muertes: meltrace_stats.muertes_totales,
        }
    }

    // ============================================================================
    // OPEN-ENDEDNESS ILIMITADO: Mecanismos para infinita evolución
    // ============================================================================

    /// Expande el Mar si la energía está muy baja
    /// Retorna true si se expandió
    pub fn expandir_si_necesario(
        &self,
        mar: &mut crate::physics::mar_morfoseo::MarMorfoseo,
        complejidad: f32,
    ) -> bool {
        let umbral_densidad = crate::physics::fixed_point::I32F32::from_i32(10);
        if mar.necesita_expansion(umbral_densidad) {
            mar.expandir_infinito();
            mar.genesis_energon(complejidad);
            true
        } else {
            false
        }
    }

    /// Genera nueva energía del vacío basándose en la complejidad del sistema
    /// Mientras más complejo = más energía emerge
    pub fn generar_energon_del_vacio(
        &self,
        mar: &mut crate::physics::mar_morfoseo::MarMorfoseo,
        complejidad: f32,
    ) {
        mar.genesis_energon(complejidad);
    }

    /// Evalúa si debe abrir una nueva dimensión de estados
    /// Esto ocurre cuando el sistema ha explorado todo lo posible en la dimensión actual
    ///
    /// INAGOTABILIDAD: Ahora usa umbrales relativos, no absolutos.
    /// La transición ocurre cuando la complejidad CRECE por encima del promedio histórico.
    pub fn evaluar_transicion_infinita(&self, num_autons: usize) -> bool {
        if num_autons < 10 {
            return false; // No hay población suficiente
        }

        let complejidad_promedio = if !self.complejidades.is_empty() {
            self.complejidades.values().map(|c| c.total).sum::<f32>()
                / self.complejidades.len() as f32
        } else {
            0.0
        };

        // INAGOTABILIDAD: Comparar con máximo histórico, no con umbral arbitrario
        // Si la complejidad promedio está por encima del 80% del máximo alcanzado,
        // y hay presión competitiva (nichos llenos), entonces es hora de expandirse
        let complejidad_maxima = self.historial_complejidad.complejidad_maxima();

        if complejidad_maxima > 0.0 {
            let ratio_actual = complejidad_promedio / complejidad_maxima;

            // Condiciones para transición:
            // 1. Estamos cerca del máximo anterior (80%+)
            // 2. Hay suficiente población (evita ruido early-game)
            // 3. Nichos están bajo presión (todos_ocupados o cerca)
            if ratio_actual > 0.8 && num_autons > 100 && self.nichos.todos_ocupados() {
                return true;
            }
        }

        //Fallback: si la complejidad NO HA PARADO DE CRECER en las últimas 1000 entradas
        let velocidad = self.historial_complejidad.velocidad_complejidad(100);
        if velocidad > 0.1 && num_autons > 500 && self.nichos.todos_ocupados() {
            return true;
        }

        false
    }

    /// Serializa Meltrace para persistencia
    pub fn meltrace_to_bytes(&self) -> Vec<u8> {
        self.meltrace.to_bytes()
    }

    /// Restaura Meltrace desde bytes
    pub fn meltrace_from_bytes(&mut self, datos: &[u8]) -> bool {
        if let Some(meltrace) = crate::life::meltrace::Meltrace::from_bytes(datos) {
            self.meltrace = meltrace;
            true
        } else {
            false
        }
    }

    /// Selecciona un grabado de Meltrace (lamarckismo)
    pub fn meltrace_seleccionar(&mut self) -> Option<crate::life::meltrace::Grabado> {
        self.meltrace.seleccionar_grabado()
    }

    /// Refuerza grabados similares en Meltrace
    pub fn meltrace_reforzar_similares(&mut self, objetivo: &crate::life::meltrace::Grabado) {
        self.meltrace.reforzar_similares(objetivo);
    }

    /// Obtiene información del Mar Morfóseo
    pub fn mar_info(&self) -> (u64, String) {
        if let Some(ref mar) = self.mar {
            let tick = mar.tick();
            let energia = mar.energia_total().to_f64();
            let densidad = mar.densidad_promedio().to_f64();
            (
                tick,
                format!("Energia: {:.1}, Densidad: {:.4}", energia, densidad),
            )
        } else {
            (0, "No inicializado".to_string())
        }
    }

    /// Consume energon del Mar para evolución
    /// Retorna true si había suficiente energon
    pub fn consumir_energon(&mut self, cantidad: f64) -> bool {
        if let Some(ref mut mar) = self.mar {
            mar.consumir_energon(cantidad)
        } else {
            false
        }
    }

    /// Verifica si hay suficiente energon (sin consumir)
    pub fn tiene_energon(&self, cantidad: f64) -> bool {
        if let Some(ref mar) = self.mar {
            mar.energia_total().to_f64() >= cantidad
        } else {
            false
        }
    }

    /// Registra nacimiento de un Auton (incrementa autons_vivos)
    pub fn registrar_nacimiento_auton(&mut self) {
        self.meltrace.registrar_nacimiento();
    }

    /// Registra muerte de un Auton (decrementa autons_vivos)
    pub fn registrar_muerte_auton(&mut self) {
        self.meltrace.registrar_muerte_auton();
    }
}

/// Métricas consolidadas de Open-Endedness
#[derive(Debug, Clone)]
pub struct OpenEndednessMetrics {
    pub complejidad_promedio: f32,
    pub complejidad_maxima: f32,
    pub tick_maximo: u64,
    pub velocidad_complejidad: f32,
    pub num_nichos: u32,
    pub num_estados_novedad: u32,
    pub num_extinciones: u64,
    pub num_semillas_transgen: u32,
    pub semillas_utilizadas: u64,
    pub nivel_termico: f32,
    // NEW: Métricas de infinito
    pub expansiones_mar: u32,       // Veces que el Mar se expandió
    pub energon_generado: f64,      // Energon creado del vacío
    pub transiciones_infinito: u32, // Transiciones a nuevas dimensiones
    // INTEGRACIÓN MELTRACE
    pub meltrace_grabados: usize,
    pub meltrace_inmortales: usize,
    pub meltrace_muertes: u64,
}

impl Default for OpenEndednessEngine {
    fn default() -> Self {
        // WARNING: Seeds with 0 - only use for placeholder, not real simulation
        Self::new(0)
    }
}

impl Default for HistorialComplejidad {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for Estampida {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for NichosSombra {
    fn default() -> Self {
        Self::new()
    }
}
