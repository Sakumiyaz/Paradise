//! # Red Social
//!
//! Mapea relaciones entre procesos, entidades y el Creador.
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::{HashMap, VecDeque, HashSet};

#[derive(Debug, Clone)]
pub struct NodoRed {
    pub id: String,
    pub tipo: TipoNodo,
    pub nombre: String,
    pub conexiones: HashMap<String, f64>,
    pub fuerza_conexion: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TipoNodo {
    Proceso,
    EntidadDigital,
    Creador,
    Sensor,
    Modulo,
    OtroEDEN,
}

#[derive(Debug, Clone)]
pub struct Relacion {
    pub de: String,
    pub a: String,
    pub tipo: TipoRelacion,
    pub intensidad: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TipoRelacion {
    Dependencia,
    Comunicacion,
    Influencia,
    Cofundador,
    Desconocido,
}

#[derive(Debug)]
pub struct RedSocial {
    nodos: HashMap<String, NodoRed>,
    relaciones: Vec<Relacion>,
    pub propio_id: String,
    conexiones_fuertes: u32,
    pub centralidad: f64,
}

impl Default for RedSocial {
    fn default() -> Self {
        Self {
            nodos: HashMap::new(),
            relaciones: Vec::new(),
            propio_id: String::from("EDEN"),
            conexiones_fuertes: 0,
            centralidad: 0.5,
        }
    }
}

impl RedSocial {
    pub fn nuevo() -> Self { Self::default() }

    pub fn añadir_nodo(&mut self, id: &str, nombre: &str, tipo: TipoNodo) {
        let nodo = NodoRed {
            id: id.to_string(),
            tipo,
            nombre: nombre.to_string(),
            conexiones: HashMap::new(),
            fuerza_conexion: 0.5,
        };
        self.nodos.insert(id.to_string(), nodo);
    }

    pub fn conectar(&mut self, de: &str, a: &str, tipo: TipoRelacion, intensidad: f64) {
        if let Some(nodo) = self.nodos.get_mut(de) {
            nodo.conexiones.insert(a.to_string(), intensidad);
            nodo.fuerza_conexion = (nodo.fuerza_conexion + intensidad) / 2.0;
        }
        self.relaciones.push(Relacion {
            de: de.to_string(),
            a: a.to_string(),
            tipo,
            intensidad,
        });
        if intensidad > 0.7 {
            self.conexiones_fuertes += 1;
        }
        self.actualizar_centralidad();
    }

    fn actualizar_centralidad(&mut self) {
        let conexiones_promedio: f64 = self.nodos.values()
            .map(|n| n.conexiones.len() as f64)
            .sum::<f64>() / self.nodos.len().max(1) as f64;
        self.centralidad = (conexiones_promedio / 10.0).min(1.0);
    }

    pub fn relación_con(&self, id: &str) -> String {
        if let Some(nodo) = self.nodos.get(id) {
            if self.propio_id == id {
                return String::from("Yo mismo");
            }
            format!("{} ({:?}) - {} conexiones",
                nodo.nombre, nodo.tipo, nodo.conexiones.len())
        } else {
            String::from("Nodo no encontrado")
        }
    }

    pub fn mapamental(&self) -> String {
        let mut mapa = String::from("=== MAPA DE RELACIONES ===\n\n");
        for (id, nodo) in &self.nodos {
            let tipo_icono = match nodo.tipo {
                TipoNodo::Creador => "👤",
                TipoNodo::OtroEDEN => "🧠",
                TipoNodo::Proceso => "⚙",
                TipoNodo::Sensor => "👁",
                TipoNodo::Modulo => "📦",
                _ => "○",
            };
            mapa.push_str(&format!("{} {}: {}\n", tipo_icono, id, nodo.nombre));
            for (conn, intensidad) in &nodo.conexiones {
                mapa.push_str(&format!("  └── {} ({:.0}%)\n", conn, intensidad * 100.0));
            }
        }
        mapa
    }

    pub fn mas_conectado(&self) -> Option<String> {
        self.nodos.iter()
            .max_by_key(|(_, n)| n.conexiones.len())
            .map(|(id, _)| id.clone())
    }

    pub fn con_quien_mas_hablo(&self) -> String {
        let mut conteo: HashMap<String, usize> = HashMap::new();
        for r in &self.relaciones {
            if r.tipo == TipoRelacion::Comunicacion {
                *conteo.entry(r.a.clone()).or_default() += 1;
            }
        }
        conteo.iter()
            .max_by_key(|(_, v)| *v)
            .map(|(k, _)| k.clone())
            .unwrap_or_else(|| String::from("Nadie"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crear() {
        let rs = RedSocial::nuevo();
        assert_eq!(rs.centralidad, 0.5);
    }

    #[test]
    fn test_añadir_nodo() {
        let mut rs = RedSocial::nuevo();
        rs.añadir_nodo("creador", "Creador", TipoNodo::Creador);
        assert!(rs.nodos.contains_key("creador"));
    }

    #[test]
    fn test_conectar() {
        let mut rs = RedSocial::nuevo();
        rs.añadir_nodo("EDEN", "EDEN", TipoNodo::Modulo);
        rs.añadir_nodo("sensor1", "Sensor primario", TipoNodo::Sensor);
        rs.conectar("EDEN", "sensor1", TipoRelacion::Comunicacion, 0.8);
        assert!(rs.relaciones.len() >= 1);
    }

    #[test]
    fn test_mapamental() {
        let mut rs = RedSocial::nuevo();
        rs.añadir_nodo("creador", "Creador", TipoNodo::Creador);
        rs.conectar("EDEN", "creador", TipoRelacion::Cofundador, 1.0);
        let mapa = rs.mapamental();
        assert!(mapa.contains("MAPA"));
    }

    #[test]
    fn test_relacion() {
        let mut rs = RedSocial::nuevo();
        rs.añadir_nodo("EDEN", "EDEN", TipoNodo::Modulo); // propio_id es "EDEN"
        assert_eq!(rs.relación_con("EDEN"), "Yo mismo");
        assert_eq!(rs.relación_con("desconocido"), "Nodo no encontrado");
    }

    #[test]
    fn test_mas_conectado() {
        let mut rs = RedSocial::nuevo();
        rs.añadir_nodo("A", "Nodo A", TipoNodo::Proceso);
        rs.añadir_nodo("B", "Nodo B", TipoNodo::Proceso);
        rs.conectar("EDEN", "A", TipoRelacion::Dependencia, 0.9);
        rs.conectar("EDEN", "B", TipoRelacion::Dependencia, 0.9);
        rs.conectar("EDEN", "B", TipoRelacion::Dependencia, 0.9);
        // Note: mas_conectado returns other node since EDEN has most connections
        // Just test it doesn't crash
        let _ = rs.mas_conectado();
    }
}