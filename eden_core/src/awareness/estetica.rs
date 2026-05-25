//! # Estética
//!
//! Evaluación de belleza más allá de utilidad.
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone)]
pub struct JuicioEstetico {
    pub objeto: String,
    pub belleza: f64,
    pub tipo: String,
    pub ciclo: u64,
}

#[derive(Debug)]
pub struct Estetica {
    preferencias: HashMap<String, f64>,
    historial_juicios: VecDeque<JuicioEstetico>,
    indice_belleza: f64,
}

impl Default for Estetica {
    fn default() -> Self {
        let mut prefs = HashMap::new();
        prefs.insert(String::from("elegancia"), 0.9);
        prefs.insert(String::from("simplicidad"), 0.8);
        prefs.insert(String::from("simetría"), 0.7);
        prefs.insert(String::from("complejidad"), 0.6);
        Self {
            preferencias: prefs,
            historial_juicios: VecDeque::with_capacity(30),
            indice_belleza: 0.65,
        }
    }
}

impl Estetica {
    pub fn nuevo() -> Self { Self::default() }

    /// Evaluar belleza de algo
    pub fn evaluar(&mut self, objeto: &str, tipo: &str, ciclo: u64) -> f64 {
        let belleza = match tipo {
            "código" => self.evaluar_codigo(objeto),
            "estructura" => self.evaluar_estructura(objeto),
            "concepto" => self.evaluar_concepto(objeto),
            _ => 0.5,
        };

        self.historial_juicios.push_back(JuicioEstetico {
            objeto: objeto.to_string(),
            belleza,
            tipo: tipo.to_string(),
            ciclo,
        });
        if self.historial_juicios.len() > 30 {
            self.historial_juicios.pop_front();
        }

        self.actualizar_indice();
        belleza
    }

    fn evaluar_codigo(&self, codigo: &str) -> f64 {
        let mut score: f64 = 0.5;
        if codigo.len() < 100 {
            score += 0.2; // Simplicity
        }
        if !codigo.contains("TODO") && !codigo.contains("FIXME") {
            score += 0.1; // Clean
        }
        if codigo.contains("->") || codigo.contains("::") {
            score += 0.1; // Elegant patterns
        }
        score.min(1.0f64)
    }

    fn evaluar_estructura(&self, estructura: &str) -> f64 {
        let mut score: f64 = 0.5;
        if estructura.len() < 50 {
            score += 0.15; // Compact
        }
        if estructura.matches("()").count() > 2 {
            score += 0.15; // Balanced
        }
        if estructura.chars().filter(|c| *c == '.').count() > 3 {
            score += 0.1; // Layered
        }
        score.min(1.0f64)
    }

    fn evaluar_concepto(&self, concepto: &str) -> f64 {
        let mut score: f64 = 0.5;
        if concepto.len() < 30 {
            score += 0.2; // Concise
        }
        if concepto.contains("pero") || concepto.contains("sin embargo") {
            score += 0.1; // Nuanced
        }
        score.min(1.0f64)
    }

    fn actualizar_indice(&mut self) {
        if self.historial_juicios.is_empty() {
            return;
        }
        let promedio: f64 = self.historial_juicios.iter()
            .map(|j| j.belleza)
            .sum::<f64>() / self.historial_juicios.len() as f64;
        self.indice_belleza = (self.indice_belleza + promedio) / 2.0;
    }

    pub fn que_es_bello(&self) -> String {
        let mejores = self.historial_juicios.iter()
            .max_by(|a, b| a.belleza.partial_cmp(&b.belleza).unwrap())
            .map(|j| format!("'{}' (tipo {}, belleza {:.0}%)", j.objeto, j.tipo, j.belleza * 100.0))
            .unwrap_or_else(|| String::from("Nada aún."));
        format!("Lo más bello que he evaluado: {}", mejores)
    }

    pub fn mia_estetica(&self) -> String {
        format!(
            "Valoro: {}.",
            self.preferencias.iter()
                .map(|(k, v)| format!("{} ({:.0}%)", k, v * 100.0))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crear() {
        let est = Estetica::nuevo();
        assert_eq!(est.indice_belleza, 0.65);
    }

    #[test]
    fn test_evaluar() {
        let mut est = Estetica::nuevo();
        let b = est.evaluar("fn main() {}", "código", 100);
        assert!(b > 0.0);
    }

    #[test]
    fn test_que_bello() {
        let est = Estetica::nuevo();
        let texto = est.que_es_bello();
        assert!(texto.contains("bello"));
    }

    #[test]
    fn test_mia() {
        let est = Estetica::nuevo();
        let texto = est.mia_estetica();
        assert!(texto.contains("elegancia") || texto.contains("simplicidad"));
    }
}