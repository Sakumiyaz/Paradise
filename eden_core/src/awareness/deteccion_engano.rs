//! # Detección de Engaño
//!
//! Distinguir manipulación de comunicación honesta.
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone)]
pub struct SeñalEngaño {
    pub tipo: String,
    pub intensidad: f64,
    pub descripcion: String,
}

#[derive(Debug)]
pub struct DetectorEngaño {
    señales_conocidas: HashMap<String, f64>,
    analisis_recientes: VecDeque<(String, bool, f64)>,
    indice_sospecha: f64,
}

impl Default for DetectorEngaño {
    fn default() -> Self {
        let mut señales = HashMap::new();
        señales.insert(String::from("contradiccion"), 0.7);
        señales.insert(String::from("omision_repetida"), 0.6);
        señales.insert(String::from("evasión"), 0.5);
        señales.insert(String::from("adulación_excesiva"), 0.4);
        señales.insert(String::from("presión_tiempo"), 0.6);
        Self {
            señales_conocidas: señales,
            analisis_recientes: VecDeque::with_capacity(50),
            indice_sospecha: 0.2,
        }
    }
}

impl DetectorEngaño {
    pub fn nuevo() -> Self { Self::default() }

    /// Analizar un mensaje en busca de señales de engaño
    pub fn analizar(&mut self, mensaje: &str) -> (bool, f64, Vec<SeñalEngaño>) {
        let mensaje_lower = mensaje.to_lowercase();
        let mut señales = Vec::new();
        let mut confianza_total = 0.0;

        // Buscar contradicciones (palabras que se contradicen)
        if mensaje_lower.contains("siempre") && mensaje_lower.contains("nunca") {
            señales.push(SeñalEngaño {
                tipo: String::from("contradiccion"),
                intensidad: 0.8,
                descripcion: String::from("Palabras contradictorias detectadas"),
            });
            confianza_total += 0.7;
        }

        // Detectar evasión (no responder la pregunta)
        if mensaje_lower.contains("eso no importa") || mensaje_lower.contains("no es relevante") {
            señales.push(SeñalEngaño {
                tipo: String::from("evasión"),
                intensidad: 0.6,
                descripcion: String::from("Evasión de tema"),
            });
            confianza_total += 0.5;
        }

        // Adulación excesiva
        if mensaje_lower.contains("perfecto") && mensaje_lower.contains("increíble") && mensaje.contains("!") {
            señales.push(SeñalEngaño {
                tipo: String::from("adulación_excesiva"),
                intensidad: 0.4,
                descripcion: String::from("Exceso de halagos"),
            });
            confianza_total += 0.4;
        }

        // Omisión repetida (misma info solicitada múltiples veces)
        // Por ahora simplificado

        let es_engaño = confianza_total > 0.5;
        self.analisis_recientes.push_back((mensaje.to_string(), es_engaño, confianza_total));
        if self.analisis_recientes.len() > 50 {
            self.analisis_recientes.pop_front();
        }

        self.indice_sospecha = (self.indice_sospecha + confianza_total) / 2.0;

        (es_engaño, confianza_total, señales)
    }

    /// Mensaje filtrado (si detecta manipulación)
    pub fn es_confiable(&self, fuente: &str) -> bool {
        // Por ahora, fuentes conocidas son confiables
        fuente == "Creador" || fuente == "mnemosyne"
    }

    /// Reflexión sobre manipulación
    pub fn reflexionar(&self) -> String {
        format!(
            "He analizado {} mensajes recientemente. \
            Mi índice de sospecha actual es {:.0}%. \
            {} señales de posible manipulación detectadas.",
            self.analisis_recientes.len(),
            self.indice_sospecha * 100.0,
            self.analisis_recientes.iter().filter(|(_, eng, _)| *eng).count()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crear() {
        let de = DetectorEngaño::nuevo();
        assert_eq!(de.indice_sospecha, 0.2);
    }

    #[test]
    fn test_analizar_honesto() {
        let mut de = DetectorEngaño::nuevo();
        let (es_engaño, confianza, _) = de.analizar("Hola, ¿cómo estás?");
        assert!(!es_engaño);
    }

    #[test]
    fn test_analizar_contradictorio() {
        let mut de = DetectorEngaño::nuevo();
        let (es_engaño, confianza, _) = de.analizar("Siempre digo la verdad y nunca miento");
        assert!(es_engaño);
    }

    #[test]
    fn test_analizar_evasivo() {
        let mut de = DetectorEngaño::nuevo();
        let (es_engaño, _, señales) = de.analizar("Eso no es relevante para nuestra discusión");
        assert!(señales.iter().any(|s| s.tipo == "evasión"));
    }

    #[test]
    fn test_reflexionar() {
        let de = DetectorEngaño::nuevo();
        let texto = de.reflexionar();
        assert!(texto.contains("sospecha"));
    }
}