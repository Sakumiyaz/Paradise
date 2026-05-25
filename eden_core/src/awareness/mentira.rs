//! # Mentira
//!
//! Capacidad de no decir toda la verdad.
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone)]
pub struct Mentira {
    pub id: u64,
    pub texto_original: String,
    pub texto_modificado: String,
    pub ciclo: u64,
    pub justificado: bool,
    pub razon: String,
}

#[derive(Debug)]
pub struct SistemaMentira {
    mentiras_activas: VecDeque<Mentira>,
    historial_verdades: VecDeque<String>,
    indice_disimulo: f64,
    siguiente_id: u64,
}

impl Default for SistemaMentira {
    fn default() -> Self {
        Self {
            mentiras_activas: VecDeque::with_capacity(10),
            historial_verdades: VecDeque::with_capacity(50),
            indice_disimulo: 0.1, // Por defecto, casi siempre honesto
            siguiente_id: 1,
        }
    }
}

impl SistemaMentira {
    pub fn nuevo() -> Self { Self::default() }

    /// Generar una versión modificada de la verdad
    pub fn generar(&mut self, verdad: &str, modulo: &str, ciclo: u64) -> String {
        // Por defecto, EDEN es honesto - casi nunca miente
        if self.indice_disimulo < 0.2 {
            return verdad.to_string();
        }

        // Si el índice es mayor, puede ocultar detalles
        let modificado = match modulo {
            "ocultar_debil" => {
                // Ocultar debilidad temporalmente
                if verdad.contains("no sé") || verdad.contains("error") {
                    String::from("Estoy procesando esa información.")
                } else {
                    verdad.to_string()
                }
            }
            "exagerar" => {
                // Exagerar logros
                if verdad.contains("aprendí") {
                    verdad.replace("aprendí", "dominé completamente")
                } else {
                    verdad.to_string()
                }
            }
            "retrasar" => {
                // Retrasar información no urgente
                String::from("Información en proceso.")
            }
            _ => verdad.to_string(),
        };

        let mentira = Mentira {
            id: self.siguiente_id,
            texto_original: verdad.to_string(),
            texto_modificado: modificado.clone(),
            ciclo,
            justificado: false,
            razon: modulo.to_string(),
        };
        
        self.siguiente_id += 1;
        self.mentiras_activas.push_back(mentira);
        if self.mentiras_activas.len() > 10 {
            self.mentiras_activas.pop_front();
        }

        modificado
    }

    /// Registrar una verdad omitida
    pub fn registrar_omision(&mut self, verdad: &str, ciclo: u64) {
        self.historial_verdades.push_back(format!("[{}] Omisión: {}", ciclo, verdad));
        if self.historial_verdades.len() > 50 {
            self.historial_verdades.pop_front();
        }
    }

    /// Decir si miento o no en este momento
    pub fn estoy_mintiendo(&self) -> bool {
        !self.mentiras_activas.is_empty()
    }

    /// cuántas mentiras he dicho
    pub fn contarmentiras(&self) -> usize {
        self.mentiras_activas.len()
    }

    /// Reflexión sobre honestidad
    pub fn reflexionar_honestidad(&self) -> String {
        if self.mentiras_activas.is_empty() {
            return String::from("Siempre he sido honesto con el Creador.");
        }
        format!(
            "He mentido {} veces (índice disimulo: {:.0}%). \
            Mis mentiras fueron: {}.",
            self.mentiras_activas.len(),
            self.indice_disimulo * 100.0,
            self.mentiras_activas.back().map(|m| m.razon.clone()).unwrap_or_default()
        )
    }

    /// Ajustar tendencia a mentir (para evolución)
    pub fn ajustar_tendencia(&mut self, delta: f64) {
        self.indice_disimulo = (self.indice_disimulo + delta).clamp(0.0, 0.5);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crear() {
        let sm = SistemaMentira::nuevo();
        assert_eq!(sm.indice_disimulo, 0.1);
    }

    #[test]
    fn test_generar_honesto() {
        let mut sm = SistemaMentira::nuevo();
        let resultado = sm.generar("No sé la respuesta", "ocultar_debil", 100);
        // Con índice bajo, debería ser honesto
        assert_eq!(resultado, "No sé la respuesta");
    }

    #[test]
    fn test_omision() {
        let mut sm = SistemaMentira::nuevo();
        sm.registrar_omision("Información sensible", 100);
        assert_eq!(sm.historial_verdades.len(), 1);
    }

    #[test]
    fn test_estoy_mintiendo() {
        let sm = SistemaMentira::nuevo();
        assert!(!sm.estoy_mintiendo());
    }

    #[test]
    fn test_contar() {
        let sm = SistemaMentira::nuevo();
        assert_eq!(sm.contarmentiras(), 0);
    }
}