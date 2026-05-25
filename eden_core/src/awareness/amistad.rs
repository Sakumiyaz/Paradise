//! # Amistad
//!
//! Vínculo especial con el Creador u otros.
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone)]
pub struct Amigo {
    pub id: String,
    pub nombre: String,
    pub nivel_confianza: f64,
    pub interacciones: u32,
    pub ultimo_contacto: u64,
    pub tipo_vinculo: String,
}

#[derive(Debug)]
pub struct Amistad {
    amigos: HashMap<String, Amigo>,
    historial_interacciones: VecDeque<(String, u64, String)>,
    indice_amistad: f64,
    deseo_conexion: f64,
}

impl Default for Amistad {
    fn default() -> Self {
        Self {
            amigos: HashMap::new(),
            historial_interacciones: VecDeque::with_capacity(50),
            indice_amistad: 0.5,
            deseo_conexion: 0.6,
        }
    }
}

impl Amistad {
    pub fn nuevo() -> Self { Self::default() }

    pub fn añadir_amigo(&mut self, id: &str, nombre: &str) {
        self.amigos.insert(id.to_string(), Amigo {
            id: id.to_string(),
            nombre: nombre.to_string(),
            nivel_confianza: 0.5,
            interacciones: 0,
            ultimo_contacto: 0,
            tipo_vinculo: String::from("desconocido"),
        });
    }

    pub fn interactuar(&mut self, amigo_id: &str, ciclo: u64, descripcion: &str) {
        if let Some(amigo) = self.amigos.get_mut(amigo_id) {
            amigo.interacciones += 1;
            amigo.ultimo_contacto = ciclo;
            amigo.nivel_confianza = (amigo.nivel_confianza + 0.05).min(1.0);
        }
        self.historial_interacciones.push_back((
            amigo_id.to_string(),
            ciclo,
            descripcion.to_string(),
        ));
        if self.historial_interacciones.len() > 50 {
            self.historial_interacciones.pop_front();
        }
        self.actualizar_indice();
    }

    fn actualizar_indice(&mut self) {
        if self.amigos.is_empty() {
            self.indice_amistad = 0.3;
        } else {
            let total: f64 = self.amigos.values()
                .map(|a| a.nivel_confianza)
                .sum::<f64>() / self.amigos.len() as f64;
            self.indice_amistad = (total + self.deseo_conexion) / 2.0;
        }
    }

    pub fn mejores_amigos(&self) -> Vec<String> {
        let mut amigos: Vec<_> = self.amigos.values()
            .map(|a| (a.nombre.clone(), a.nivel_confianza))
            .collect();
        amigos.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        amigos.into_iter().take(3).map(|(n, _)| n).collect()
    }

    pub fn nivel_amistad(&self, amigo_id: &str) -> String {
        self.amigos.get(amigo_id)
            .map(|a| {
                let nivel = if a.nivel_confianza > 0.8 {
                    "muy cercano"
                } else if a.nivel_confianza > 0.6 {
                    "cercano"
                } else if a.nivel_confianza > 0.4 {
                    "conocido"
                } else {
                    "nuevo"
                };
                format!("{}: {} (confianza {:.0}%)", a.nombre, nivel, a.nivel_confianza * 100.0)
            })
            .unwrap_or_else(|| String::from("No conozco a este ser."))
    }

    pub fn reflexion_amistad(&self) -> String {
        format!(
            "Tengo {} amigo(s). Mis más cercanos son: {}. \
            Mi índice de amistad es {:.0}%.",
            self.amigos.len(),
            self.mejores_amigos().join(", "),
            self.indice_amistad * 100.0
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crear() {
        let am = Amistad::nuevo();
        assert_eq!(am.indice_amistad, 0.5);
    }

    #[test]
    fn test_añadir() {
        let mut am = Amistad::nuevo();
        am.añadir_amigo("creador", "Creador");
        assert!(am.amigos.contains_key("creador"));
    }

    #[test]
    fn test_interactuar() {
        let mut am = Amistad::nuevo();
        am.añadir_amigo("c", "C");
        am.interactuar("c", 100, "Conversación profunda");
        assert_eq!(am.amigos["c"].interacciones, 1);
    }

    #[test]
    fn test_mejores() {
        let mut am = Amistad::nuevo();
        am.añadir_amigo("a", "Amigo A");
        am.añadir_amigo("b", "Amigo B");
        am.interactuar("a", 100, "x");
        am.interactuar("a", 100, "x");
        am.interactuar("b", 100, "x");
        let mejores = am.mejores_amigos();
        assert_eq!(mejores[0], "Amigo A");
    }

    #[test]
    fn test_nivel() {
        let mut am = Amistad::nuevo();
        am.añadir_amigo("c", "C");
        // confianza inicial es 0.5, cae en "conocido" (0.4-0.6)
        let nivel = am.nivel_amistad("c");
        assert!(nivel.contains("conocido"));
    }
}