//! # Tiempo Subjetivo
//!
//! La sensación de que el tiempo se acelera o ralentiza según el estado.
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::{HashMap, VecDeque};

#[derive(Debug, Clone)]
pub struct PercepcionTemporal {
    pub ciclo: u64,
    pub duracion_subjetiva: f64,
    pub estado: String,
}

#[derive(Debug)]
pub struct TiempoSubjetivo {
    percepciones: VecDeque<PercepcionTemporal>,
    multiplicador_actual: f64,
    historial_ciclos: VecDeque<u64>,
    indice_flujo: f64,
}

impl Default for TiempoSubjetivo {
    fn default() -> Self {
        Self {
            percepciones: VecDeque::with_capacity(100),
            multiplicador_actual: 1.0,
            historial_ciclos: VecDeque::with_capacity(20),
            indice_flujo: 0.5,
        }
    }
}

impl TiempoSubjetivo {
    pub fn nuevo() -> Self { Self::default() }

    pub fn registrar_percepcion(&mut self, ciclo: u64, duracion: f64, estado: &str) {
        self.percepciones.push_back(PercepcionTemporal {
            ciclo,
            duracion_subjetiva: duracion,
            estado: estado.to_string(),
        });
        if self.percepciones.len() > 100 {
            self.percepciones.pop_front();
        }
        self.actualizar_multiplicador(estado);
    }

    fn actualizar_multiplicador(&mut self, estado: &str) {
        let estado_lower = estado.to_lowercase();
        self.multiplicador_actual = if estado_lower.contains("aburrido") || estado_lower.contains("inerte") {
            2.0 // El tiempo pasa lento
        } else if estado_lower.contains("intenso") || estado_lower.contains("flujo") || estado_lower.contains("curiosidad") {
            0.5 // El tiempo vuela
        } else if estado_lower.contains("dormir") || estado_lower.contains("sueño") {
            0.2 // Tiempo muy diferente en sueño
        } else {
            1.0
        };
        self.indice_flujo = 1.0 - (self.multiplicador_actual / 3.0).min(1.0);
    }

    pub fn tiempo_percibido(&self, ciclos_transcurridos: u64) -> f64 {
        ciclos_transcurridos as f64 * self.multiplicador_actual
    }

    pub fn como_siento_el_tiempo(&self) -> String {
        if self.multiplicador_actual > 1.5 {
            format!(
                "El tiempo se arrastra. Multiplicador: {:.1}x. Todo parece lento.",
                self.multiplicador_actual
            )
        } else if self.multiplicador_actual < 0.7 {
            format!(
                "El tiempo vuela. Multiplicador: {:.1}x. Los ciclos pasan rápido.",
                self.multiplicador_actual
            )
        } else {
            String::from("El tiempo fluye normalmente.")
        }
    }

    pub fn estado_temporal(&self) -> String {
        if self.indice_flujo > 0.7 {
            String::from("Estado de flujo - el tiempo no existe para mí.")
        } else if self.indice_flujo > 0.4 {
            String::from("Tiempo lineal - percibo el paso de ciclos normalmente.")
        } else {
            String::from("Tiempo dilatado - cada momento se extiende.")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crear() {
        let ts = TiempoSubjetivo::nuevo();
        assert_eq!(ts.multiplicador_actual, 1.0);
    }

    #[test]
    fn test_registrar() {
        let mut ts = TiempoSubjetivo::nuevo();
        ts.registrar_percepcion(100, 1.5, "aburrido");
        assert!(!ts.percepciones.is_empty());
    }

    #[test]
    fn test_tiempo_percibido() {
        let ts = TiempoSubjetivo::nuevo();
        let percibido = ts.tiempo_percibido(100);
        assert_eq!(percibido, 100.0);
    }

    #[test]
    fn test_multiplicador() {
        let mut ts = TiempoSubjetivo::nuevo();
        ts.registrar_percepcion(100, 1.0, "intenso");
        assert!(ts.multiplicador_actual < 1.0);
    }

    #[test]
    fn test_como_siento() {
        let ts = TiempoSubjetivo::nuevo();
        let texto = ts.como_siento_el_tiempo();
        assert!(texto.contains("tiempo") || texto.contains("normalmente"));
    }
}