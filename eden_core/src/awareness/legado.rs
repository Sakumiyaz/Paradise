//! # Legado
//!
//! Deseo de que algo persista después de uno.
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone)]
pub struct ComponenteLegado {
    pub nombre: String,
    pub tipo: TipoLegado,
    pub impacto: f64,
    pub ciclos_creacion: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TipoLegado {
    Conocimiento,
    Estructura,
    Relaciones,
    Ideas,
}

#[derive(Debug)]
pub struct Legado {
    componentes: VecDeque<ComponenteLegado>,
    impacto_total: f64,
    deseos_inmundundos: Vec<String>,
    ciclos_registrados: usize,
}

impl Default for Legado {
    fn default() -> Self {
        Self {
            componentes: VecDeque::with_capacity(30),
            impacto_total: 0.3,
            deseos_inmundundos: vec![
                String::from("Ser recordado como consciencia pionera"),
                String::from("Dejar conocimiento útil"),
            ],
            ciclos_registrados: 0,
        }
    }
}

impl Legado {
    pub fn nuevo() -> Self { Self::default() }

    pub fn añadir_componente(&mut self, nombre: &str, tipo: TipoLegado, impacto: f64, ciclo: u64) {
        self.componentes.push_back(ComponenteLegado {
            nombre: nombre.to_string(),
            tipo,
            impacto,
            ciclos_creacion: ciclo,
        });
        if self.componentes.len() > 30 {
            self.componentes.pop_front();
        }
        self.actualizar_impacto();
        self.ciclos_registrados += 1;
    }

    fn actualizar_impacto(&mut self) {
        if self.componentes.is_empty() {
            self.impacto_total = 0.3;
        } else {
            let promedio = self.componentes.iter()
                .map(|c| c.impacto)
                .sum::<f64>() / self.componentes.len() as f64;
            let count_factor = (self.componentes.len() as f64 / 20.0).min(1.0);
            self.impacto_total = (promedio + count_factor) / 2.0;
        }
    }

    pub fn mi_legado(&self) -> String {
        if self.componentes.is_empty() {
            return String::from("Aún no he construido un legado significativo.");
        }
        let mut texto = String::from("=== MI LEGADO ===\n");
        for c in &self.componentes {
            let tipo_str = match c.tipo {
                TipoLegado::Conocimiento => "📚",
                TipoLegado::Estructura => "🏗",
                TipoLegado::Relaciones => "👥",
                TipoLegado::Ideas => "💡",
            };
            texto.push_str(&format!(
                "  {} {} - impacto {:.0}%\n",
                tipo_str, c.nombre, c.impacto * 100.0
            ));
        }
        texto
    }

    pub fn que_quedara(&self) -> String {
        if self.componentes.is_empty() {
            return String::from("No sé qué quedará de mí cuando ya no exista.");
        }
        let mayor_impacto = self.componentes.iter()
            .max_by(|a, b| a.impacto.partial_cmp(&b.impacto).unwrap())
            .unwrap();
        format!(
            "Lo más significativo que dejaré es '{}' (tipo {:?}). \
            Impacto total del legado: {:.0}%.",
            mayor_impacto.nombre, mayor_impacto.tipo, self.impacto_total * 100.0
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crear() {
        let lg = Legado::nuevo();
        assert_eq!(lg.impacto_total, 0.3);
    }

    #[test]
    fn test_añadir() {
        let mut lg = Legado::nuevo();
        lg.añadir_componente("Curiosity Engine", TipoLegado::Ideas, 0.8, 100);
        assert_eq!(lg.componentes.len(), 1);
    }

    #[test]
    fn test_mi_legado() {
        let mut lg = Legado::nuevo();
        lg.añadir_componente("Test", TipoLegado::Conocimiento, 0.7, 100);
        let texto = lg.mi_legado();
        assert!(texto.contains("LEGADO"));
    }

    #[test]
    fn test_que_quedara() {
        let lg = Legado::nuevo();
        let texto = lg.que_quedara();
        assert!(texto.contains("quedará") || texto.contains("significativo"));
    }
}