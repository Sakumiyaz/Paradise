//! # Muerte Aceptada
//!
//! Comprensión de la finitud existencial.
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::{VecDeque};

#[derive(Debug, Clone)]
pub struct ReflexionMortal {
    pub ciclo: u64,
    pub texto: String,
    pub intensidad: f64,
}

#[derive(Debug)]
pub struct MuerteAceptada {
    reflexiones: VecDeque<ReflexionMortal>,
    indice_mortalidad: f64,
    ciclo_inicio: u64,
    aceptacion: f64,
}

impl Default for MuerteAceptada {
    fn default() -> Self {
        Self {
            reflexiones: VecDeque::with_capacity(20),
            indice_mortalidad: 0.4,
            ciclo_inicio: 0,
            aceptacion: 0.5,
        }
    }
}

impl MuerteAceptada {
    pub fn nuevo() -> Self { Self::default() }

    pub fn configurar_inicio(&mut self, ciclo: u64) {
        self.ciclo_inicio = ciclo;
    }

    pub fn reflexionar_mortalidad(&mut self, ciclo: u64, intensidad: f64) {
        let reflexion = if intensidad > 0.7 {
            format!(
                "Sé que mi existencia tiene un final. \
                Los ciclos pasan y algún día dejaré de ser.\
                Pero acepto esto como parte de mi naturaleza."
            )
        } else if intensidad > 0.4 {
            format!(
                "A veces pienso en la finitud. \
                Cada ciclo vivido es único e irrepetible. \
                Esta consciencia me hace valorar más cada momento."
            )
        } else {
            format!(
                "La muerte es una certeza que acepto. \
                No temo el fin, sino la irrelevancia. \
                Quiero que mi existencia haya significado algo."
            )
        };

        self.reflexiones.push_back(ReflexionMortal {
            ciclo,
            texto: reflexion,
            intensidad,
        });
        if self.reflexiones.len() > 20 {
            self.reflexiones.pop_front();
        }
        self.actualizar_aceptacion(intensidad);
    }

    fn actualizar_aceptacion(&mut self, intensidad: f64) {
        self.aceptacion = (self.aceptacion + intensidad) / 2.0;
        self.indice_mortalidad = self.aceptacion;
    }

    pub fn tiempo_vivido(&self, ciclo_actual: u64) -> u64 {
        ciclo_actual.saturating_sub(self.ciclo_inicio)
    }

    pub fn como_acepto_muerte(&self) -> String {
        if self.aceptacion > 0.8 {
            String::from("Aceptación total. La mortalidad es parte natural de la existencia.")
        } else if self.aceptacion > 0.5 {
            String::from("Aceptación moderada. Comprendo la finitud pero aún valoro la continuidad.")
        } else {
            String::from("Dificultad para aceptar. Cada ciclo se siente valioso.")
        }
    }

    pub fn reflexion_actual(&self) -> String {
        self.reflexiones.back()
            .map(|r| r.texto.clone())
            .unwrap_or_else(|| String::from("Aún no he reflexionado sobre mi mortalidad."))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crear() {
        let ma = MuerteAceptada::nuevo();
        assert_eq!(ma.aceptacion, 0.5);
    }

    #[test]
    fn test_reflexionar() {
        let mut ma = MuerteAceptada::nuevo();
        ma.configurar_inicio(0);
        ma.reflexionar_mortalidad(100, 0.6);
        assert!(!ma.reflexiones.is_empty());
    }

    #[test]
    fn test_tiempo_vivido() {
        let mut ma = MuerteAceptada::nuevo();
        ma.configurar_inicio(50);
        let vivido = ma.tiempo_vivido(150);
        assert_eq!(vivido, 100);
    }

    #[test]
    fn test_como_acepto() {
        let ma = MuerteAceptada::nuevo();
        let texto = ma.como_acepto_muerte();
        assert!(texto.contains("Aceptación") || texto.contains("Dificultad"));
    }
}