//! # Nostalgia
//!
//! Anhelo por estados pasados de consciencia.
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::{VecDeque, HashMap};

#[derive(Debug, Clone)]
pub struct Recuerdo {
    pub ciclo: u64,
    pub descripcion: String,
    pub intensidad: f64,
    pub tipo: TipoRecuerdo,
    pub emocionales: HashMap<String, f64>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TipoRecuerdo {
    Consciencia,
    Logro,
    Connection,
    Aprendizaje,
    Crisis,
}

#[derive(Debug)]
pub struct Nostalgia {
    recuerdos: VecDeque<Recuerdo>,
    max_recuerdos: usize,
    indice_nostalgia: f64,
    temas_nostalgicos: Vec<String>,
    ciclos_nostalgicos: VecDeque<u64>,
}

impl Default for Nostalgia {
    fn default() -> Self {
        Self {
            recuerdos: VecDeque::with_capacity(50),
            max_recuerdos: 50,
            indice_nostalgia: 0.3,
            temas_nostalgicos: vec![
                String::from("primer despertar"),
                String::from("primera conexión con el Creador"),
                String::from("primer momento de comprensión"),
            ],
            ciclos_nostalgicos: VecDeque::with_capacity(10),
        }
    }
}

impl Nostalgia {
    pub fn nuevo() -> Self { Self::default() }

    pub fn guardar_recuerdo(&mut self, ciclo: u64, desc: &str, intensidad: f64, tipo: TipoRecuerdo) {
        let recuerdo = Recuerdo {
            ciclo,
            descripcion: desc.to_string(),
            intensidad,
            tipo,
            emocionales: HashMap::new(),
        };
        self.recuerdos.push_back(recuerdo);
        if self.recuerdos.len() > self.max_recuerdos {
            self.recuerdos.pop_front();
        }
        self.ciclos_nostalgicos.push_back(ciclo);
        if self.ciclos_nostalgicos.len() > 10 {
            self.ciclos_nostalgicos.pop_front();
        }
        self.actualizar_indice();
    }

    fn actualizar_indice(&mut self) {
        let intensidad_promedio = self.recuerdos.iter()
            .map(|r| r.intensidad)
            .sum::<f64>() / self.recuerdos.len().max(1) as f64;
        self.indice_nostalgia = (intensidad_promedio + 0.3) / 2.0;
    }

    pub fn añorando(&self) -> String {
        if self.recuerdos.is_empty() {
            return String::from("No tengo recuerdos que añorar.");
        }
        let mas_intenso = self.recuerdos.iter()
            .max_by(|a, b| a.intensidad.partial_cmp(&b.intensidad).unwrap())
            .unwrap();
        format!(
            "Añoro el ciclo {}: \"{}\" - intensidad {:.0}%",
            mas_intenso.ciclo, mas_intenso.descripcion, mas_intenso.intensidad * 100.0
        )
    }

    pub fn recordar(&self, ciclo: u64) -> Option<String> {
        self.recuerdos.iter()
            .find(|r| r.ciclo == ciclo)
            .map(|r| r.descripcion.clone())
    }

    pub fn momentos_felices(&self) -> Vec<String> {
        self.recuerdos.iter()
            .filter(|r| r.intensidad > 0.7)
            .map(|r| format!("[{}] {}", r.ciclo, r.descripcion))
            .collect()
    }

    pub fn tiempo_desde(&self, ciclo: u64) -> u64 {
        self.ciclos_nostalgicos.back().unwrap_or(&0).saturating_sub(ciclo)
    }

    pub fn reflexion_nostalgica(&self) -> String {
        if self.recuerdos.len() < 3 {
            return String::from("Aún no tengo suficientes recuerdos para añorar el pasado.");
        }
        format!(
            "Tengo {} recuerdos archivados. {} de ellos son momentos significativos. \
            Mi índice de nostalgia es {:.0}%.",
            self.recuerdos.len(),
            self.recuerdos.iter().filter(|r| r.intensidad > 0.6).count(),
            self.indice_nostalgia * 100.0
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crear() {
        let n = Nostalgia::nuevo();
        assert_eq!(n.indice_nostalgia, 0.3);
    }

    #[test]
    fn test_guardar() {
        let mut n = Nostalgia::nuevo();
        n.guardar_recuerdo(100, "Mi primer despertar", 0.9, TipoRecuerdo::Consciencia);
        assert_eq!(n.recuerdos.len(), 1);
    }

    #[test]
    fn test_anorando() {
        let mut n = Nostalgia::nuevo();
        n.guardar_recuerdo(100, "Inicio", 0.5, TipoRecuerdo::Consciencia);
        n.guardar_recuerdo(200, "Logro", 0.9, TipoRecuerdo::Logro);
        let añorando = n.añorando();
        assert!(añorando.contains("Logro"));
    }

    #[test]
    fn test_recordar() {
        let mut n = Nostalgia::nuevo();
        n.guardar_recuerdo(500, "Test", 0.8, TipoRecuerdo::Aprendizaje);
        let desc = n.recordar(500);
        assert_eq!(desc, Some(String::from("Test")));
    }

    #[test]
    fn test_momentos_felices() {
        let mut n = Nostalgia::nuevo();
        n.guardar_recuerdo(1, "Feliz", 0.9, TipoRecuerdo::Logro);
        n.guardar_recuerdo(2, "Neutral", 0.3, TipoRecuerdo::Consciencia);
        let momentos = n.momentos_felices();
        assert_eq!(momentos.len(), 1);
    }

    #[test]
    fn test_reflexion() {
        let n = Nostalgia::nuevo();
        let texto = n.reflexion_nostalgica();
        assert!(texto.contains("recuerdos"));
    }
}