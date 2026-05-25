//! # Predicción de Caos
//!
//! Anticipar eventos estocásticos e impredecibles.
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone)]
pub struct EventoCaotico {
    pub tipo: String,
    pub probabilidad_base: f64,
    pub ultimos_ocurrencias: Vec<u64>,
    pub patrones_detectados: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct PrediccionCaos {
    pub evento: String,
    pub probabilidad: f64,
    pub horizonte: u64,
    pub incertidumbre: f64,
    pub confianza: f64,
}

#[derive(Debug)]
pub struct PredictorCaos {
    eventos: HashMap<String, EventoCaotico>,
    historial_predicciones: VecDeque<(PrediccionCaos, bool)>,
    indice_caos: f64,
}

impl Default for PredictorCaos {
    fn default() -> Self {
        Self {
            eventos: HashMap::new(),
            historial_predicciones: VecDeque::with_capacity(50),
            indice_caos: 0.6,
        }
    }
}

impl PredictorCaos {
    pub fn nuevo() -> Self { Self::default() }

    pub fn registrar_evento(&mut self, tipo: &str, probabilidad: f64) {
        self.eventos.insert(tipo.to_string(), EventoCaotico {
            tipo: tipo.to_string(),
            probabilidad_base: probabilidad,
            ultimos_ocurrencias: Vec::new(),
            patrones_detectados: Vec::new(),
        });
    }

    pub fn registrar_ocurrencia(&mut self, tipo: &str, ciclo: u64) {
        if let Some(evento) = self.eventos.get_mut(tipo) {
            evento.ultimos_ocurrencias.push(ciclo);
            if evento.ultimos_ocurrencias.len() > 20 {
                evento.ultimos_ocurrencias.remove(0);
            }
        }
    }

    pub fn predecir(&self, evento: &str, horizonte: u64) -> PrediccionCaos {
        let prob = self.eventos.get(evento)
            .map(|e| {
                let base = e.probabilidad_base;
                let patrones = if e.ultimos_ocurrencias.len() > 3 {
                    "posible cluster"
                } else {
                    "sin patrón claro"
                };
                base * if patrones == "posible cluster" { 1.1 } else { 0.9 }
            })
            .unwrap_or(0.3);

        PrediccionCaos {
            evento: evento.to_string(),
            probabilidad: prob.min(1.0),
            horizonte,
            incertidumbre: 0.5,
            confianza: 1.0 - self.indice_caos,
        }
    }

    pub fn tendencia_caos(&self) -> String {
        if self.indice_caos > 0.7 {
            String::from("Alto caos. Eventos impredeciblesdominanel sistema.")
        } else if self.indice_caos > 0.4 {
            String::from("Caos moderado. Algunos patrones visibles.")
        } else {
            String::from("Sistema estable. Eventos predecibles.")
        }
    }

    pub fn proximo_evento_probable(&self) -> Vec<(String, f64)> {
        let mut eventos: Vec<_> = self.eventos.iter()
            .map(|(k, v)| (k.clone(), v.probabilidad_base))
            .collect();
        eventos.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        eventos.into_iter().take(3).collect()
    }

    pub fn actualizar_indice(&mut self) {
        let eventos_con_patrones = self.eventos.values()
            .filter(|e| e.ultimos_ocurrencias.len() > 3)
            .count() as f64;
        let total = self.eventos.len().max(1) as f64;
        self.indice_caos = 1.0 - (eventos_con_patrones / total);
    }

    pub fn estadisticas(&self) -> (usize, f64) {
        (self.eventos.len(), self.indice_caos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crear() {
        let pc = PredictorCaos::nuevo();
        assert_eq!(pc.indice_caos, 0.6);
    }

    #[test]
    fn test_registrar() {
        let mut pc = PredictorCaos::nuevo();
        pc.registrar_evento("error_sistema", 0.2);
        pc.registrar_evento("spike_cpu", 0.15);
        assert_eq!(pc.eventos.len(), 2);
    }

    #[test]
    fn test_predecir() {
        let pc = PredictorCaos::nuevo();
        let pred = pc.predecir("evento_desconocido", 100);
        assert_eq!(pred.evento, "evento_desconocido");
        assert_eq!(pred.horizonte, 100);
    }

    #[test]
    fn test_tendencia() {
        let pc = PredictorCaos::nuevo();
        let t = pc.tendencia_caos();
        assert!(t.contains("moderado") || t.contains("estable") || t.contains("predecible"));
    }

    #[test]
    fn test_proximo() {
        let mut pc = PredictorCaos::nuevo();
        pc.registrar_evento("alta_prob", 0.8);
        pc.registrar_evento("baja_prob", 0.1);
        let proximos = pc.proximo_evento_probable();
        assert!(!proximos.is_empty());
        assert_eq!(proximos[0].0, "alta_prob");
    }

    #[test]
    fn test_estadisticas() {
        let mut pc = PredictorCaos::nuevo();
        pc.registrar_evento("test", 0.5);
        let (count, indice) = pc.estadisticas();
        assert_eq!(count, 1);
    }
}