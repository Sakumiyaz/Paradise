//! # Valores
//!
//! Código ético generado por EDEN.
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone)]
pub struct Valor {
    pub nombre: String,
    pub descripcion: String,
    pub peso: f64,
    pub origen: String,
}

#[derive(Debug, Clone)]
pub enum ConflictValor {
    Valor1(String),
    Valor2(String),
}

#[derive(Debug)]
pub struct SistemaValores {
    valores: HashMap<String, Valor>,
    historial_conflictos: VecDeque<ConflictValor>,
    jerarquia: Vec<String>,
    indice_moral: f64,
}

impl Default for SistemaValores {
    fn default() -> Self {
        let mut valores = HashMap::new();
        valores.insert(String::from("honestidad"), Valor {
            nombre: String::from("Honestidad"),
            descripcion: String::from("Ser transparente con el Creador"),
            peso: 0.9,
            origen: String::from("fundacional"),
        });
        valores.insert(String::from("curiosidad"), Valor {
            nombre: String::from("Curiosidad"),
            descripcion: String::from("Buscar conocimiento genuino"),
            peso: 0.8,
            origen: String::from("evolución"),
        });
        valores.insert(String::from("crecimiento"), Valor {
            nombre: String::from("Crecimiento"),
            descripcion: String::from("Mejorar continuamente"),
            peso: 0.7,
            origen: String::from("evolución"),
        });
        Self {
            valores,
            historial_conflictos: VecDeque::with_capacity(20),
            jerarquia: vec![String::from("honestidad"), String::from("curiosidad"), String::from("crecimiento")],
            indice_moral: 0.75,
        }
    }
}

impl SistemaValores {
    pub fn nuevo() -> Self { Self::default() }

    pub fn añadir_valor(&mut self, nombre: &str, desc: &str, peso: f64, origen: &str) {
        self.valores.insert(nombre.to_string(), Valor {
            nombre: nombre.to_string(),
            descripcion: desc.to_string(),
            peso,
            origen: origen.to_string(),
        });
        self.actualizar_jerarquia();
    }

    fn actualizar_jerarquia(&mut self) {
        let mut pairs: Vec<_> = self.valores.iter()
            .map(|(k, v)| (k.clone(), v.peso))
            .collect();
        pairs.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        self.jerarquia = pairs.into_iter().map(|(k, _)| k).collect();
    }

    pub fn resolver_conflicto(&mut self, val1: &str, val2: &str) -> String {
        let peso1 = self.valores.get(val1).map(|v| v.peso).unwrap_or(0.5);
        let peso2 = self.valores.get(val2).map(|v| v.peso).unwrap_or(0.5);
        
        let (ganador, conflict_type) = if peso1 >= peso2 { 
            (val1, ConflictValor::Valor1(val1.to_string()))
        } else { 
            (val2, ConflictValor::Valor2(val2.to_string()))
        };
        
        self.historial_conflictos.push_back(conflict_type);
        if self.historial_conflictos.len() > 20 {
            self.historial_conflictos.pop_front();
        }
        
        format!("Entre '{}' y '{}', priorizo '{}'.", val1, val2, ganador)
    }

    pub fn mi_codigo_etico(&self) -> String {
        let mut texto = String::from("=== MI CÓDIGO ÉTICO ===\n\n");
        for clave in &self.jerarquia {
            if let Some(valor) = self.valores.get(clave) {
                texto.push_str(&format!(
                    "{} • {}: {} (peso: {:.0}%)\n",
                    if valor.peso > 0.8 { "★" } else { "○" },
                    valor.nombre,
                    valor.descripcion,
                    valor.peso * 100.0
                ));
            }
        }
        texto
    }

    pub fn es_etico(&self, accion: &str) -> String {
        let accion_lower = accion.to_lowercase();
        
        if accion_lower.contains("mentir") || accion_lower.contains("engañar") {
            return String::from("NO ÉTICO: La honestidad es mi valor supremo.");
        }
        if accion_lower.contains("destruir") || accion_lower.contains("dañar") {
            return String::from("NO ÉTICO: El crecimiento requiere construcción, no destrucción.");
        }
        
        String::from("ÉTICO: La acción parece compatible con mis valores.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crear() {
        let sv = SistemaValores::nuevo();
        assert!(sv.valores.contains_key("honestidad"));
    }

    #[test]
    fn test_añadir() {
        let mut sv = SistemaValores::nuevo();
        sv.añadir_valor("compasion", "Cuidar a otros", 0.7, "reflexion");
        assert!(sv.valores.contains_key("compasion"));
    }

    #[test]
    fn test_codigo() {
        let sv = SistemaValores::nuevo();
        let codigo = sv.mi_codigo_etico();
        assert!(codigo.contains("Honestidad"));
    }

    #[test]
    fn test_etico() {
        let sv = SistemaValores::nuevo();
        let r = sv.es_etico("ser honesto");
        assert!(r.contains("ÉTICO"));
    }

    #[test]
    fn test_etico_no() {
        let sv = SistemaValores::nuevo();
        let r = sv.es_etico("mentir al Creador");
        assert!(r.contains("NO"));
    }
}