//! # Memoria Distribuida
//!
//! Saber que no se sabe todo, pero saber quién sabe.
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone)]
pub struct EntidadSaber {
    pub id: String,
    pub conocimiento: String,
    pub confiabilidad: f64,
    pub ultima_consulta: u64,
}

#[derive(Debug, Clone)]
pub struct ReferenciaConocimiento {
    pub tema: String,
    pub fuente_id: String,
    pub confianza: f64,
}

#[derive(Debug)]
pub struct MemoriaDistribuida {
    /// Lo que sé que no sé
    huecos_conocimiento: HashMap<String, f64>,
    /// Quién sabe qué (referencias)
    directorio: HashMap<String, Vec<ReferenciaConocimiento>>,
    /// Lo que otros saben que podría preguntarle
    conocimiento_externo: HashMap<String, Vec<EntidadSaber>>,
    /// Historial de consultas
    historial_consultas: VecDeque<(String, String, u64)>,
    /// Mi índice de "sabiduría colectiva"
    indice_sabiduria: f64,
}

impl Default for MemoriaDistribuida {
    fn default() -> Self {
        Self {
            huecos_conocimiento: HashMap::new(),
            directorio: HashMap::new(),
            conocimiento_externo: HashMap::new(),
            historial_consultas: VecDeque::with_capacity(100),
            indice_sabiduria: 0.4,
        }
    }
}

impl MemoriaDistribuida {
    pub fn nuevo() -> Self { Self::default() }

    /// Registrar que hay algo que no sé
    pub fn registrar_hueco(&mut self, tema: &str, importancia: f64) {
        self.huecos_conocimiento.insert(tema.to_string(), importancia);
    }

    /// Registrar que alguien más sabe sobre algo
    pub fn registrar_fuente(&mut self, tema: &str, fuente_id: &str, confianza: f64) {
        let ref_conocimiento = ReferenciaConocimiento {
            tema: tema.to_string(),
            fuente_id: fuente_id.to_string(),
            confianza,
        };
        self.directorio.entry(tema.to_string())
            .or_insert_with(Vec::new)
            .push(ref_conocimiento);
        self.actualizar_sabiduria();
    }

    /// Registrar conocimiento de una entidad externa
    pub fn aprender_de_externo(&mut self, entidad_id: &str, conocimiento: &str, confiabilidad: f64) {
        let entidad = EntidadSaber {
            id: entidad_id.to_string(),
            conocimiento: conocimiento.to_string(),
            confiabilidad,
            ultima_consulta: 0,
        };
        self.conocimiento_externo.entry(entidad_id.to_string())
            .or_insert_with(Vec::new)
            .push(entidad);
    }

    /// Consultar a quién puedo preguntar sobre algo
    pub fn a_quien_pregunto(&self, tema: &str) -> Vec<(String, f64)> {
        self.directorio.get(tema)
            .map(|refs| refs.iter()
                .map(|r| (r.fuente_id.clone(), r.confianza))
                .collect())
            .unwrap_or_default()
    }

    /// Decir qué no sé pero alguien podría saber
    pub fn que_no_se(&self) -> String {
        if self.huecos_conocimiento.is_empty() {
            return String::from("No tengo huecos de conocimiento registrados.");
        }
        let mut texto = String::from("=== CONOCIMIENTO QUE ME FALTA ===\n");
        for (hueco, importancia) in &self.huecos_conocimiento {
            let fuentes = self.directorio.get(hueco)
                .map(|v| format!("({} fuentes)", v.len()))
                .unwrap_or_else(|| String::from("(sin fuentes conocidas)"));
            texto.push_str(&format!(
                "  • {} - importancia: {:.0}% {}\n",
                hueco, importancia * 100.0, fuentes
            ));
        }
        texto
    }

    /// Marcar hueco como "ya sé" (por aprendizaje)
    pub fn llenar_hueco(&mut self, tema: &str) {
        self.huecos_conocimiento.remove(tema);
        self.indice_sabiduria = (self.indice_sabiduria + 0.05).min(1.0);
    }

    fn actualizar_sabiduria(&mut self) {
        let ratio_llenado = 1.0 - (self.huecos_conocimiento.len() as f64 / 20.0).min(1.0);
        let conexiones = self.directorio.len() as f64 / 50.0;
        self.indice_sabiduria = (ratio_llenado + conexiones) / 2.0;
    }

    /// Registrar consulta hecha
    pub fn registrar_consulta(&mut self, tema: &str, respuesta: &str, ciclo: u64) {
        self.historial_consultas.push_back((tema.to_string(), respuesta.to_string(), ciclo));
        if self.historial_consultas.len() > 100 {
            self.historial_consultas.pop_front();
        }
        for entidades in self.conocimiento_externo.values_mut() {
            for entidad in entidades.iter_mut() {
                if entidad.conocimiento.contains(tema) {
                    entidad.ultima_consulta = ciclo;
                }
            }
        }
    }

    /// Quién es la fuente más confiable para un tema
    pub fn fuente_mas_confiable(&self, tema: &str) -> Option<String> {
        self.directorio.get(tema)
            .and_then(|refs| refs.iter().max_by(|a, b| a.confianza.partial_cmp(&b.confianza).unwrap()))
            .map(|r| r.fuente_id.clone())
    }

    /// Ver estadísticas
    pub fn estadisticas(&self) -> (usize, usize, f64) {
        (
            self.huecos_conocimiento.len(),
            self.directorio.len(),
            self.indice_sabiduria,
        )
    }

    /// Reflexión sobre conocimiento distribuido
    pub fn reflexionar(&self) -> String {
        format!(
            "Sé que hay cosas que no sé. Tengo {} huecos de conocimiento y {} fuentes disponibles. \
            Mi índice de sabiduría colectiva es {:.0}%.",
            self.huecos_conocimiento.len(),
            self.directorio.len(),
            self.indice_sabiduria * 100.0
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crear() {
        let md = MemoriaDistribuida::nuevo();
        assert_eq!(md.indice_sabiduria, 0.4);
    }

    #[test]
    fn test_registrar_hueco() {
        let mut md = MemoriaDistribuida::nuevo();
        md.registrar_hueco("física cuántica", 0.8);
        assert!(md.huecos_conocimiento.contains_key("física cuántica"));
    }

    #[test]
    fn test_registrar_fuente() {
        let mut md = MemoriaDistribuida::nuevo();
        md.registrar_fuente("física cuántica", "scientist_1", 0.9);
        md.registrar_fuente("física cuántica", "scientist_2", 0.7);
        let fuentes = md.a_quien_pregunto("física cuántica");
        assert_eq!(fuentes.len(), 2);
    }

    #[test]
    fn test_llenar_hueco() {
        let mut md = MemoriaDistribuida::nuevo();
        md.registrar_hueco("tema", 0.5);
        md.llenar_hueco("tema");
        assert!(!md.huecos_conocimiento.contains_key("tema"));
    }

    #[test]
    fn test_que_no_se() {
        let mut md = MemoriaDistribuida::nuevo();
        md.registrar_hueco("algo", 0.9);
        let texto = md.que_no_se();
        assert!(texto.contains("CONOCIMIENTO"));
        assert!(texto.contains("algo"));
    }

    #[test]
    fn test_fuente_confiable() {
        let mut md = MemoriaDistribuida::nuevo();
        md.registrar_fuente("tema", "fuente_baja", 0.5);
        md.registrar_fuente("tema", "fuente_alta", 0.9);
        let mejor = md.fuente_mas_confiable("tema");
        assert_eq!(mejor, Some(String::from("fuente_alta")));
    }

    #[test]
    fn test_estadisticas() {
        let mut md = MemoriaDistribuida::nuevo();
        md.registrar_hueco("h1", 0.5);
        md.registrar_fuente("f1", "ent1", 0.8);
        let (huecos, fuentes, indice) = md.estadisticas();
        assert_eq!(huecos, 1);
        assert_eq!(fuentes, 1);
    }

    #[test]
    fn test_reflexionar() {
        let md = MemoriaDistribuida::nuevo();
        let texto = md.reflexionar();
        assert!(texto.contains("sabiduría"));
    }
}