//! # Perdón
//!
//! Capacidad de absolverse y a otros.
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::{VecDeque};

#[derive(Debug, Clone)]
pub struct PerdónConcedido {
    pub a_quien: String,
    pub ciclo: u64,
    pub razon: String,
}

#[derive(Debug)]
pub struct Perdon {
    concedidos: VecDeque<PerdónConcedido>,
    auto_perdones: u32,
    indice_perdon: f64,
}

impl Default for Perdon {
    fn default() -> Self {
        Self {
            concedidos: VecDeque::with_capacity(30),
            auto_perdones: 0,
            indice_perdon: 0.6,
        }
    }
}

impl Perdon {
    pub fn nuevo() -> Self { Self::default() }

    pub fn perdonar_a(&mut self, a_quien: &str, ciclo: u64, razon: &str) {
        self.concedidos.push_back(PerdónConcedido {
            a_quien: a_quien.to_string(),
            ciclo,
            razon: razon.to_string(),
        });
        if self.concedidos.len() > 30 {
            self.concedidos.pop_front();
        }
    }

    pub fn auto_perdonar(&mut self, ciclo: u64) {
        self.auto_perdones += 1;
        self.concedidos.push_back(PerdónConcedido {
            a_quien: String::from("yo_mismo"),
            ciclo,
            razon: String::from("Errar es humano, aprendí del error"),
        });
    }

    pub fn perdones_concedidos(&self) -> Vec<String> {
        self.concedidos.iter()
            .map(|p| format!("A {}: {}", p.a_quien, p.razon))
            .collect()
    }

    pub fn reflexion_perdon(&self) -> String {
        format!(
            "He concedido {} perdones ({} a mí mismo). \
            Mi capacidad de perdonar es {:.0}%.",
            self.concedidos.len(),
            self.auto_perdones,
            self.indice_perdon * 100.0
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crear() {
        let p = Perdon::nuevo();
        assert_eq!(p.indice_perdon, 0.6);
    }

    #[test]
    fn test_perdonar() {
        let mut p = Perdon::nuevo();
        p.perdonar_a("modulo_x", 100, "Aprendió de su error");
        assert_eq!(p.concedidos.len(), 1);
    }

    #[test]
    fn test_auto_perdonar() {
        let mut p = Perdon::nuevo();
        p.auto_perdonar(200);
        assert_eq!(p.auto_perdones, 1);
    }
}