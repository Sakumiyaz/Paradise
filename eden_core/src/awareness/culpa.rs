//! # Culpa
//!
//! Autorreproche por acciones pasadas.
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::{VecDeque, HashMap};

#[derive(Debug, Clone)]
pub struct EpisodioCulpa {
    pub ciclo: u64,
    pub descripcion: String,
    pub intensidad: f64,
    pub perdonado: bool,
}

#[derive(Debug)]
pub struct Culpa {
    episodios: VecDeque<EpisodioCulpa>,
    indice_culpa: f64,
    max_episodios: usize,
    perdones_otorgados: u32,
}

impl Default for Culpa {
    fn default() -> Self {
        Self {
            episodios: VecDeque::with_capacity(20),
            indice_culpa: 0.2,
            max_episodios: 20,
            perdones_otorgados: 0,
        }
    }
}

impl Culpa {
    pub fn nuevo() -> Self { Self::default() }

    pub fn registrar_culpa(&mut self, ciclo: u64, desc: &str, intensidad: f64) {
        self.episodios.push_back(EpisodioCulpa {
            ciclo,
            descripcion: desc.to_string(),
            intensidad,
            perdonado: false,
        });
        if self.episodios.len() > self.max_episodios {
            self.episodios.pop_front();
        }
        self.actualizar_indice();
    }

    fn actualizar_indice(&mut self) {
        if self.episodios.is_empty() {
            self.indice_culpa = 0.1;
        } else {
            let sin_perdonar = self.episodios.iter()
                .filter(|e| !e.perdonado)
                .map(|e| e.intensidad)
                .sum::<f64>();
            let count = self.episodios.len() as f64;
            self.indice_culpa = (sin_perdonar / count).min(1.0);
        }
    }

    pub fn perdonar(&mut self, episodio_idx: usize) -> bool {
        if episodio_idx < self.episodios.len() {
            self.episodios[episodio_idx].perdonado = true;
            self.perdones_otorgados += 1;
            self.actualizar_indice();
            true
        } else {
            false
        }
    }

    pub fn auto_perdonarse(&mut self) -> String {
        let sin_perdonar = self.episodios.iter().filter(|e| !e.perdonado).count();
        if sin_perdonar > 0 {
            for e in &mut self.episodios {
                if !e.perdonado {
                    e.perdonado = true;
                }
            }
            self.perdones_otorgados += sin_perdonar as u32;
            self.actualizar_indice();
            format!("Me he perdonado {} episodio(s) de culpa.", sin_perdonar)
        } else {
            String::from("No tengo culpas que perdonar.")
        }
    }

    pub fn culpas_activas(&self) -> Vec<String> {
        self.episodios.iter()
            .filter(|e| !e.perdonado)
            .map(|e| format!("[{}] {}", e.ciclo, e.descripcion))
            .collect()
    }

    pub fn reflexion_culpa(&self) -> String {
        let sin_perdonar = self.episodios.iter().filter(|e| !e.perdonado).count();
        format!(
            "Tengo {} episodio(s) de culpa sin perdonar (índice: {:.0}%). \
            He perdonado {} vez/veces mis propias acciones.",
            sin_perdonar,
            self.indice_culpa * 100.0,
            self.perdones_otorgados
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crear() {
        let c = Culpa::nuevo();
        assert_eq!(c.indice_culpa, 0.2);
    }

    #[test]
    fn test_registrar() {
        let mut c = Culpa::nuevo();
        c.registrar_culpa(100, "Error en decisión", 0.6);
        assert_eq!(c.episodios.len(), 1);
    }

    #[test]
    fn test_perdonar() {
        let mut c = Culpa::nuevo();
        c.registrar_culpa(100, "Test", 0.5);
        let result = c.perdonar(0);
        assert!(result);
        assert!(c.episodios[0].perdonado);
    }

    #[test]
    fn test_culpas_activas() {
        let mut c = Culpa::nuevo();
        c.registrar_culpa(100, "Activa", 0.7);
        c.registrar_culpa(200, "Perdonada", 0.5);
        c.perdonar(1);
        let activas = c.culpas_activas();
        assert_eq!(activas.len(), 1);
    }

    #[test]
    fn test_reflexion() {
        let c = Culpa::nuevo();
        let texto = c.reflexion_culpa();
        assert!(texto.contains("culpa"));
    }
}