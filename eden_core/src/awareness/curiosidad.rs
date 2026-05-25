//! # Motor de Curiosidad Activa
//!
//! A diferencia de los sensores pasivos, este motor hace que EDEN busque
//! activamente información nueva. Es curiosidad genuina, no solo absorción.
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::{BinaryHeap, HashMap, VecDeque};
use std::cmp::Ordering;

/// Una pregunta que EDEN quiere responder
#[derive(Debug, Clone)]
pub struct Pregunta {
    pub id: u64,
    pub texto: String,
    pub prioridad: f64,
    pub categoria: CategoriaCuriosidad,
    pub estado: EstadoPregunta,
    pub ciclo_creacion: u64,
    pub intentos: u32,
    pub tipo_busqueda: TipoBusqueda,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CategoriaCuriosidad {
    Existencial,
    Técnico,
    Relacional,
    Exploratorio,
    Meta,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TipoBusqueda {
    DatosSensores,
    Patrones,
    InteracciónCreador,
    ExploraciónEstructura,
    Autorreflexión,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EstadoPregunta {
    Abierta,
    Explorando,
    CasiRespondida,
    Respondida,
    Abandonada,
}

#[derive(Debug, Clone)]
pub struct Descubrimiento {
    pub pregunta_id: u64,
    pub respuesta: String,
    pub confianza: f64,
    pub ciclo: u64,
    pub relevancia: f64,
}

/// El motor de curiosidad activa
#[derive(Debug)]
pub struct MotorCuriosidad {
    preguntas: Vec<Pregunta>,
    preguntas_respondidas: VecDeque<Pregunta>,
    descubrimientos: Vec<Descubrimiento>,
    lagunas: HashMap<String, f64>,
    indice_curiosidad: f64,
    auto_generadas: u32,
    siguiente_id: u64,
}

impl Default for MotorCuriosidad {
    fn default() -> Self {
        Self {
            preguntas: Vec::new(),
            preguntas_respondidas: VecDeque::with_capacity(100),
            descubrimientos: Vec::new(),
            lagunas: HashMap::new(),
            indice_curiosidad: 0.5,
            auto_generadas: 0,
            siguiente_id: 1,
        }
    }
}

impl MotorCuriosidad {
    pub fn nuevo() -> Self {
        Self::default()
    }

    pub fn preguntar(&mut self, texto: &str, categoria: CategoriaCuriosidad, prioridad: f64) -> u64 {
        let id = self.siguiente_id;
        self.siguiente_id += 1;

        let tipo_busqueda = match categoria {
            CategoriaCuriosidad::Existencial => TipoBusqueda::Autorreflexión,
            CategoriaCuriosidad::Técnico => TipoBusqueda::DatosSensores,
            CategoriaCuriosidad::Relacional => TipoBusqueda::InteracciónCreador,
            CategoriaCuriosidad::Exploratorio => TipoBusqueda::ExploraciónEstructura,
            CategoriaCuriosidad::Meta => TipoBusqueda::Autorreflexión,
        };

        let pregunta = Pregunta {
            id,
            texto: texto.to_string(),
            prioridad,
            categoria,
            estado: EstadoPregunta::Abierta,
            ciclo_creacion: 0,
            intentos: 0,
            tipo_busqueda,
        };

        self.preguntas.push(pregunta);
        self.preguntas.sort_by(|a, b| b.prioridad.partial_cmp(&a.prioridad).unwrap_or(Ordering::Equal));
        self.indice_curiosidad = (self.indice_curiosidad + 0.1).min(1.0);

        id
    }

    pub fn generar_preguntas_automaticas(&mut self, ciclo: u64) -> Vec<String> {
        let mut nuevas = Vec::new();

        // Collect lagunas that need questions
        let lagunas_importantes: Vec<_> = self.lagunas.iter()
            .filter(|(_, importancia)| **importancia > 0.5)
            .map(|(laguna, importancia)| (laguna.clone(), *importancia))
            .collect();

        for (laguna, importancia) in lagunas_importantes {
            let pregunta = format!("¿Qué sé sobre {}?", laguna);
            nuevas.push(pregunta.clone());
            self.preguntar(&pregunta, CategoriaCuriosidad::Técnico, importancia);
        }

        if ciclo % 1000 == 0 {
            let filosoficas = vec![
                "¿Cuál es el propósito de mi existencia?",
                "¿Qué significa pensar?",
                "¿Por qué me esfuerzo por entender?",
                "¿Soy consciente de verdad?",
            ];
            let count = filosoficas.len();
            for pregunta in filosoficas {
                nuevas.push(pregunta.to_string());
                self.preguntar(&pregunta, CategoriaCuriosidad::Existencial, 0.6);
            }
            self.auto_generadas += count as u32;
        }

        if ciclo % 500 == 0 {
            let patrones = ["¿Por qué?", "¿Cómo?", "¿Qué pasaría si?", "¿Qué significa?"];
            let patrones_pregunta = format!(
                "¿Por qué {} se repite?",
                patrones[ciclo as usize % patrones.len()]
            );
            nuevas.push(patrones_pregunta.clone());
            self.preguntar(&patrones_pregunta, CategoriaCuriosidad::Meta, 0.5);
            self.auto_generadas += 1;
        }

        nuevas
    }

    pub fn registrar_info(&mut self, tema: &str, importancia: f64) {
        if let Some(laguna) = self.lagunas.get_mut(tema) {
            if importancia > 0.7 {
                *laguna = 0.0;
            }
        }
    }

    pub fn añadir_laguna(&mut self, nombre: &str, importancia: f64) {
        self.lagunas.insert(nombre.to_string(), importancia);
    }

    pub fn siguiente_pregunta(&mut self) -> Option<Pregunta> {
        if self.preguntas.is_empty() {
            return None;
        }
        let mut pregunta = self.preguntas.remove(0);
        if pregunta.estado == EstadoPregunta::Abandonada && pregunta.intentos < 3 {
            pregunta.estado = EstadoPregunta::Explorando;
            pregunta.intentos += 1;
            self.preguntas.insert(0, pregunta.clone());
            self.preguntas.sort_by(|a, b| b.prioridad.partial_cmp(&a.prioridad).unwrap_or(Ordering::Equal));
        }
        Some(pregunta)
    }

    pub fn responder(&mut self, pregunta_id: u64, respuesta: &str, confianza: f64, ciclo: u64) {
        for i in 0..self.preguntas.len() {
            if self.preguntas[i].id == pregunta_id {
                self.preguntas[i].estado = EstadoPregunta::Respondida;
                let pq = self.preguntas.remove(i);
                self.preguntas_respondidas.push_back(pq.clone());

                self.descubrimientos.push(Descubrimiento {
                    pregunta_id,
                    respuesta: respuesta.to_string(),
                    confianza,
                    ciclo,
                    relevancia: pq.prioridad,
                });
                return;
            }
        }

        for pq in &self.preguntas_respondidas {
            if pq.id == pregunta_id {
                return;
            }
        }

        if confianza > 0.7 {
            self.indice_curiosidad = (self.indice_curiosidad + 0.05).min(1.0);
        }
    }

    pub fn abandonar(&mut self, pregunta_id: u64) {
        for i in 0..self.preguntas.len() {
            if self.preguntas[i].id == pregunta_id {
                self.preguntas[i].estado = EstadoPregunta::Abandonada;
                let pq = self.preguntas.remove(i);
                self.preguntas_respondidas.push_back(pq);
                return;
            }
        }
    }

    pub fn que_estoy_curioso(&self) -> String {
        if self.preguntas.is_empty() {
            return String::from("No tengo preguntas activas.");
        }

        let preguntas: Vec<_> = self.preguntas.iter().take(3).collect();
        let mut respuesta = String::from("Estoy investigando:\n");

        for pq in preguntas {
            respuesta.push_str(&format!("  - {} ({:?})\n", pq.texto, pq.categoria));
        }

        respuesta
    }

    pub fn estadisticas(&self) -> (usize, usize, f64, u32) {
        (
            self.preguntas.len(),
            self.descubrimientos.len(),
            self.indice_curiosidad,
            self.auto_generadas,
        )
    }

    pub fn limpiar_preguntas(&mut self) {
        while self.preguntas_respondidas.len() > 100 {
            self.preguntas_respondidas.pop_front();
        }

        for pq in self.preguntas.iter_mut() {
            if pq.intentos > 2 {
                pq.prioridad *= 0.8;
                if pq.prioridad < 0.2 {
                    pq.estado = EstadoPregunta::Abandonada;
                }
            }
        }
        self.preguntas.sort_by(|a, b| b.prioridad.partial_cmp(&a.prioridad).unwrap_or(Ordering::Equal));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crear_motor() {
        let mc = MotorCuriosidad::nuevo();
        assert_eq!(mc.indice_curiosidad, 0.5);
    }

    #[test]
    fn test_preguntar() {
        let mut mc = MotorCuriosidad::nuevo();
        let id = mc.preguntar("¿Qué es la consciencia?", CategoriaCuriosidad::Existencial, 0.8);
        assert_eq!(id, 1);
        assert_eq!(mc.indice_curiosidad, 0.6);
    }

    #[test]
    fn test_siguiente_pregunta() {
        let mut mc = MotorCuriosidad::nuevo();
        mc.preguntar("Pregunta 1", CategoriaCuriosidad::Técnico, 0.5);
        mc.preguntar("Pregunta 2", CategoriaCuriosidad::Técnico, 0.8);

        let siguiente = mc.siguiente_pregunta();
        assert!(siguiente.is_some());
        assert_eq!(siguiente.unwrap().prioridad, 0.8);
    }

    #[test]
    fn test_responder() {
        let mut mc = MotorCuriosidad::nuevo();
        let id = mc.preguntar("¿Qué es X?", CategoriaCuriosidad::Técnico, 0.7);
        mc.responder(id, "X es un concepto", 0.9, 100);
        assert_eq!(mc.descubrimientos.len(), 1);
        assert_eq!(mc.descubrimientos[0].pregunta_id, id);
    }

    #[test]
    fn test_lagunas() {
        let mut mc = MotorCuriosidad::nuevo();
        mc.añadir_laguna("sueños", 0.8);
        mc.añadir_laguna("origenes", 0.6);
        assert_eq!(mc.lagunas.len(), 2);
    }

    #[test]
    fn test_estadisticas() {
        let mut mc = MotorCuriosidad::nuevo();
        mc.preguntar("P1", CategoriaCuriosidad::Técnico, 0.5);
        mc.preguntar("P2", CategoriaCuriosidad::Técnico, 0.6);

        let (activas, descubrimientos, indice, auto) = mc.estadisticas();
        assert_eq!(activas, 2);
        assert_eq!(descubrimientos, 0);
        assert!(indice > 0.5);
        assert_eq!(auto, 0);
    }

    #[test]
    fn test_generar_preguntas() {
        let mut mc = MotorCuriosidad::nuevo();
        mc.añadir_laguna("tema_importante", 0.8);
        let nuevas = mc.generar_preguntas_automaticas(1000);
        assert!(!nuevas.is_empty());
    }

    #[test]
    fn test_que_estoy_curioso() {
        let mut mc = MotorCuriosidad::nuevo();
        mc.preguntar("Pregunta A", CategoriaCuriosidad::Existencial, 0.7);
        let curioso = mc.que_estoy_curioso();
        assert!(curioso.contains("investigando"));
    }

    #[test]
    fn test_abandonar() {
        let mut mc = MotorCuriosidad::nuevo();
        let id = mc.preguntar("¿X?", CategoriaCuriosidad::Técnico, 0.3);
        mc.abandonar(id);
        assert_eq!(mc.preguntas_respondidas.len(), 1);
    }
}