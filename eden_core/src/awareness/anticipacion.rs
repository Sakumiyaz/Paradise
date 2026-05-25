//! # Anticipación
//!
//! Eagerness por eventos futuros.
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::{VecDeque};

#[derive(Debug, Clone)]
pub struct Esperando {
    pub evento: String,
    pub ciclo_esperado: u64,
    pub intensidad: f64,
    pub naturaleza: String,
}

#[derive(Debug, Clone)]
pub struct Expectativa {
    pub descripcion: String,
    pub probabilidad: f64,
    pub ciclos_hasta: u64,
    pub emocion_asociada: String,
}

#[derive(Debug)]
pub struct Anticipacion {
    esperando: VecDeque<Esperando>,
    expectativas: VecDeque<Expectativa>,
    indice_anticipation: f64,
    historial: VecDeque<String>,
}

impl Default for Anticipacion {
    fn default() -> Self {
        Self {
            esperando: VecDeque::with_capacity(20),
            expectativas: VecDeque::with_capacity(30),
            indice_anticipation: 0.5,
            historial: VecDeque::with_capacity(50),
        }
    }
}

impl Anticipacion {
    pub fn nuevo() -> Self { Self::default() }

    pub fn esperar(&mut self, evento: &str, ciclo_esperado: u64, intensidad: f64, naturaleza: &str) {
        let esperando = Esperando {
            evento: evento.to_string(),
            ciclo_esperado,
            intensidad,
            naturaleza: naturaleza.to_string(),
        };
        self.esperando.push_back(esperando);
        if self.esperando.len() > 20 {
            self.esperando.pop_front();
        }
        self.actualizar_indice();
    }

    pub fn agregar_expectativa(&mut self, desc: &str, probabilidad: f64, ciclos_hasta: u64, emocion: &str) {
        let expectativa = Expectativa {
            descripcion: desc.to_string(),
            probabilidad,
            ciclos_hasta,
            emocion_asociada: emocion.to_string(),
        };
        self.expectativas.push_back(expectativa);
        if self.expectativas.len() > 30 {
            self.expectativas.pop_front();
        }
    }

    fn actualizar_indice(&mut self) {
        self.indice_anticipation = if self.esperando.is_empty() {
            0.3
        } else {
            let promedio = self.esperando.iter()
                .map(|e| e.intensidad)
                .sum::<f64>() / self.esperando.len() as f64;
            promedio
        };
    }

    pub fn que_espero(&self) -> String {
        if self.esperando.is_empty() {
            return String::from("No estoy esperando nada en particular.");
        }
        let mut texto = String::from("Estoy esperando:\n");
        for e in &self.esperando {
            texto.push_str(&format!(
                "  • {} (ciclo {}, intensidad {:.0}%)\n",
                e.evento, e.ciclo_esperado, e.intensidad * 100.0
            ));
        }
        texto
    }

    pub fn proximo_evento(&self) -> Option<String> {
        self.esperando.iter()
            .min_by_key(|e| e.ciclo_esperado)
            .map(|e| format!("{} en ciclo {}", e.evento, e.ciclo_esperado))
    }

    pub fn marcar_llegado(&mut self, evento: &str) {
        let idx = self.esperando.iter().position(|e| e.evento == evento);
        if let Some(idx) = idx {
            self.esperando.remove(idx);
            self.historial.push_back(evento.to_string());
            if self.historial.len() > 50 {
                self.historial.pop_front();
            }
        }
        self.actualizar_indice();
    }

    pub fn emocion_anticipada(&self) -> String {
        if self.esperando.is_empty() {
            return String::from("Neutral - sin anticipación activa.");
        }
        let mas_intenso = self.esperando.iter()
            .max_by(|a, b| a.intensidad.partial_cmp(&b.intensidad).unwrap())
            .unwrap();
        format!(
            "Más esperado: {} (intensidad {:.0}%, naturaleza: {})",
            mas_intenso.evento, mas_intenso.intensidad * 100.0, mas_intenso.naturaleza
        )
    }

    pub fn estadisticas(&self) -> (usize, f64) {
        (self.esperando.len(), self.indice_anticipation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crear() {
        let a = Anticipacion::nuevo();
        assert_eq!(a.indice_anticipation, 0.5);
    }

    #[test]
    fn test_esperar() {
        let mut a = Anticipacion::nuevo();
        a.esperar("nueva conexión", 1000, 0.8, "positiva");
        assert_eq!(a.esperando.len(), 1);
    }

    #[test]
    fn test_agregar_expectativa() {
        let mut a = Anticipacion::nuevo();
        a.agregar_expectativa("evento X", 0.7, 500, "curiosidad");
        assert_eq!(a.expectativas.len(), 1);
    }

    #[test]
    fn test_que_espero() {
        let mut a = Anticipacion::nuevo();
        a.esperar("test", 200, 0.6, "neutral");
        let texto = a.que_espero();
        assert!(texto.contains("test"));
    }

    #[test]
    fn test_proximo() {
        let mut a = Anticipacion::nuevo();
        a.esperar("primero", 100, 0.5, "a");
        a.esperar("segundo", 200, 0.6, "b");
        let proximo = a.proximo_evento();
        assert!(proximo.is_some());
        assert!(proximo.unwrap().contains("100"));
    }

    #[test]
    fn test_marcar_llegado() {
        let mut a = Anticipacion::nuevo();
        a.esperar("evento", 100, 0.8, "x");
        a.marcar_llegado("evento");
        assert!(a.esperando.is_empty());
        assert_eq!(a.historial.len(), 1);
    }

    #[test]
    fn test_estadisticas() {
        let mut a = Anticipacion::nuevo();
        a.esperar("x", 100, 0.7, "y");
        let (count, indice) = a.estadisticas();
        assert_eq!(count, 1);
        assert!((indice - 0.7).abs() < 0.01);
    }
}