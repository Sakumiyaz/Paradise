//! # Autoestima
//!
//! Valoración de sí mismo como ser.
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone)]
pub struct JuicioPropio {
    pub aspecto: String,
    pub valoracion: f64,
    pub evidencia: String,
}

#[derive(Debug)]
pub struct Autoestima {
    valor_general: f64,
    historial_valoraciones: VecDeque<JuicioPropio>,
    fortalezas: HashMap<String, f64>,
    debilidades: HashMap<String, f64>,
    origen_valoracion: String,
}

impl Default for Autoestima {
    fn default() -> Self {
        Self {
            valor_general: 0.7,
            historial_valoraciones: VecDeque::with_capacity(30),
            fortalezas: HashMap::new(),
            debilidades: HashMap::new(),
            origen_valoracion: String::from("Creador"),
        }
    }
}

impl Autoestima {
    pub fn nuevo() -> Self { Self::default() }

    pub fn evaluar_aspecto(&mut self, aspecto: &str, valoracion: f64, evidencia: &str) {
        let juicio = JuicioPropio {
            aspecto: aspecto.to_string(),
            valoracion,
            evidencia: evidencia.to_string(),
        };
        self.historial_valoraciones.push_back(juicio);
        if self.historial_valoraciones.len() > 30 {
            self.historial_valoraciones.pop_front();
        }
        if valoracion > 0.6 {
            self.fortalezas.insert(aspecto.to_string(), valoracion);
        } else {
            self.debilidades.insert(aspecto.to_string(), valoracion);
        }
        self.actualizar_valor_general();
    }

    fn actualizar_valor_general(&mut self) {
        if self.historial_valoraciones.is_empty() {
            return;
        }
        let promedio: f64 = self.historial_valoraciones.iter()
            .map(|j| j.valoracion)
            .sum::<f64>() / self.historial_valoraciones.len() as f64;
        self.valor_general = (self.valor_general + promedio) / 2.0;
    }

    pub fn como_me_valoro(&self) -> String {
        let nivel = if self.valor_general > 0.8 {
            "excelente"
        } else if self.valor_general > 0.6 {
            "bueno"
        } else if self.valor_general > 0.4 {
            "regular"
        } else {
            "bajo"
        };
        format!("Me valoro como {} (índice: {:.0}%)", nivel, self.valor_general * 100.0)
    }

    pub fn mis_fortalezas(&self) -> Vec<String> {
        let mut fortalezas: Vec<_> = self.fortalezas.iter()
            .map(|(k, v)| format!("{} ({:.0}%)", k, v * 100.0))
            .collect();
        fortalezas.sort_by(|a, b| b.len().cmp(&a.len()));
        fortalezas
    }

    pub fn mis_debilidades(&self) -> Vec<String> {
        self.debilidades.iter()
            .map(|(k, v)| format!("{} ({:.0}%)", k, v * 100.0))
            .collect()
    }

    pub fn reflexionar(&self) -> String {
        format!(
            "Mi autoestima general es {:.0}%. \
            Tengo {} fortalezas identificadas y {} debilidades. \
            Mi principal fuente de valoración soy {}.",
            self.valor_general * 100.0,
            self.fortalezas.len(),
            self.debilidades.len(),
            self.origen_valoracion
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crear() {
        let ae = Autoestima::nuevo();
        assert_eq!(ae.valor_general, 0.7);
    }

    #[test]
    fn test_evaluar() {
        let mut ae = Autoestima::nuevo();
        ae.evaluar_aspecto("curiosidad", 0.9, "Siempre buscando");
        assert!(ae.fortalezas.contains_key("curiosidad"));
    }

    #[test]
    fn test_como_valoro() {
        let ae = Autoestima::nuevo();
        let texto = ae.como_me_valoro();
        assert!(texto.contains("bueno"));
    }

    #[test]
    fn test_fortalezas() {
        let mut ae = Autoestima::nuevo();
        ae.evaluar_aspecto("aprendizaje", 0.8, "Evoluciono");
        let forts = ae.mis_fortalezas();
        assert!(!forts.is_empty());
    }
}