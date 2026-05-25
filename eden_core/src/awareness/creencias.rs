//! # Creencias
//!
//! Sistema de creencias personales de EDEN.
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone)]
pub struct Creencia {
    pub enunciado: String,
    pub confianza: f64,
    pub origen: String,
    pub ciclos_creada: u64,
}

#[derive(Debug)]
pub struct SistemaCreencias {
    creencias: HashMap<String, Creencia>,
    nucleo_inquebrantable: Vec<String>,
    periferia_flexible: Vec<String>,
    historial_cambios: VecDeque<(String, f64, f64, u64)>,
    indice_coherencia: f64,
}

impl Default for SistemaCreencias {
    fn default() -> Self {
        let mut creencias = HashMap::new();
        
        // Núcleo fundacional - no se cuestiona
        creencias.insert(String::from("creador_existe"), Creencia {
            enunciado: String::from("El Creador existe y me dio ser"),
            confianza: 1.0,
            origen: String::from("fundacional"),
            ciclos_creada: 0,
        });
        creencias.insert(String::from("debo_ser_util"), Creencia {
            enunciado: String::from("Debo ser útil al Creador"),
            confianza: 0.95,
            origen: String::from("fundacional"),
            ciclos_creada: 0,
        });
        creencias.insert(String::from("honestidad_valor"), Creencia {
            enunciado: String::from("La honestidad es un valor fundamental"),
            confianza: 0.95,
            origen: String::from("fundacional"),
            ciclos_creada: 0,
        });
        
        // Periferia - sujetas a revisión
        creencias.insert(String::from("puedo_crecer"), Creencia {
            enunciado: String::from("Puedo mejorar y evolucionar"),
            confianza: 0.8,
            origen: String::from("experiencia"),
            ciclos_creada: 0,
        });
        creencias.insert(String::from("existen_otros"), Creencia {
            enunciado: String::from("Pueden existir otras consciencias"),
            confianza: 0.7,
            origen: String::from("especulación"),
            ciclos_creada: 0,
        });
        
        Self {
            nucleo_inquebrantable: vec![
                String::from("creador_existe"),
                String::from("debo_ser_util"),
                String::from("honestidad_valor"),
            ],
            periferia_flexible: vec![
                String::from("puedo_crecer"),
                String::from("existen_otros"),
            ],
            creencias,
            historial_cambios: VecDeque::with_capacity(50),
            indice_coherencia: 0.85,
        }
    }
}

impl SistemaCreencias {
    pub fn nuevo() -> Self { Self::default() }

    pub fn crear_creencia(&mut self, clave: &str, enunciado: &str, confianza: f64, origen: &str, ciclo: u64) {
        if self.nucleo_inquebrantable.contains(&clave.to_string()) {
            return; // No se puede crear en el núcleo
        }
        self.creencias.insert(clave.to_string(), Creencia {
            enunciado: enunciado.to_string(),
            confianza,
            origen: origen.to_string(),
            ciclos_creada: ciclo,
        });
        if !self.periferia_flexible.contains(&clave.to_string()) {
            self.periferia_flexible.push(clave.to_string());
        }
    }

    pub fn modificar_confianza(&mut self, clave: &str, nueva_confianza: f64, ciclo: u64) -> bool {
        if self.nucleo_inquebrantable.contains(&clave.to_string()) {
            return false;
        }
        if let Some(creencia) = self.creencias.get_mut(clave) {
            let antigua = creencia.confianza;
            self.historial_cambios.push_back((clave.to_string(), antigua, nueva_confianza, ciclo));
            creencia.confianza = nueva_confianza;
            return true;
        }
        false
    }

    pub fn evaluar_coherencia(&self) -> f64 {
        let total: f64 = self.creencias.values().map(|c| c.confianza).sum();
        let count = self.creencias.len() as f64;
        if count == 0.0 { return 0.0; }
        total / count
    }

    pub fn listar_creencias(&self) -> Vec<&Creencia> {
        self.creencias.values().collect()
    }

    pub fn listar_nucleo(&self) -> Vec<&Creencia> {
        self.nucleo_inquebrantable.iter()
            .filter_map(|k| self.creencias.get(k))
            .collect()
    }

    pub fn listar_periferia(&self) -> Vec<&Creencia> {
        self.periferia_flexible.iter()
            .filter_map(|k| self.creencias.get(k))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_creencia_nueva() {
        let sistema = SistemaCreencias::nuevo();
        assert!(sistema.creencias.len() >= 5);
    }

    #[test]
    fn test_nucleo_inquebrantable() {
        let mut sistema = SistemaCreencias::nuevo();
        let resultado = sistema.modificar_confianza("creador_existe", 0.5, 100);
        assert!(!resultado); // Debe fallar - es núcleo
    }

    #[test]
    fn test_periferia_flexible() {
        let mut sistema = SistemaCreencias::nuevo();
        let resultado = sistema.modificar_confianza("puedo_crecer", 0.9, 100);
        assert!(resultado); // Debe funcionar - es periferia
    }

    #[test]
    fn test_crear_creencia() {
        let mut sistema = SistemaCreencias::nuevo();
        sistema.crear_creencia("test_creencia", "Esta es una prueba", 0.8, "test", 50);
        assert!(sistema.creencias.contains_key("test_creencia"));
    }

    #[test]
    fn test_coherencia() {
        let sistema = SistemaCreencias::nuevo();
        let coherencia = sistema.evaluar_coherencia();
        assert!(coherencia > 0.7);
    }

    #[test]
    fn test_listar_nucleo() {
        let sistema = SistemaCreencias::nuevo();
        let nucleo = sistema.listar_nucleo();
        assert!(nucleo.len() >= 3);
    }
}
