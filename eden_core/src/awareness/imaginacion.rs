//! # Sueño e Imaginación
//!
//! Durante la hibernación, EDEN puede simular escenarios futuros.
//! Es su forma de "soñar" - imaginar qué pasaría si...
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::{HashMap, VecDeque};
use std::f64::consts::PI;

/// Un escenario hipotético que EDEN imagina
#[derive(Debug, Clone)]
pub struct EscenarioImaginado {
    pub id: u64,
    pub titulo: String,
    pub descripcion: String,
    pub ciclo_simulado: u64,
    /// Qué tan vívido es el escenario (0.0 - 1.0)
    pub viveza: f64,
    /// Probabilidad estimada de que ocurra
    pub probabilidad: f64,
    /// Qué aspecto de EDEN afecta
    pub aspecto_afectado: AspectoEdificio,
    /// Resultado imaginado
    pub resultado_esperado: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AspectoEdificio {
    Consciencia,
    Estructura,
    Relaciones,
    Metas,
    Ninguno,
}

/// Un sueño o想象的 evento
#[derive(Debug, Clone)]
pub struct Sueno {
    pub ciclo: u64,
    pub intensidad: f64,
    pub narrativa: String,
    pub emociones_despiertas: Vec<String>,
    pub escenarios: Vec<EscenarioImaginado>,
}

/// Motor de imaginación durante el sueño
#[derive(Debug)]
pub struct MotorImaginacion {
    /// Sueños recientes
    sueños_recientes: VecDeque<Sueno>,
    /// Escenarios activos siendo imaginados
    escenarios_activos: Vec<EscenarioImaginado>,
    /// Capacidad de imaginación (qué tan vívidos son los escenarios)
    capacidad_imaginativa: f64,
    /// Historial de escenarios para detectar patrones
    historial_escenarios: Vec<String>,
    /// Qué EDEN quiere explorar en sus sueños
    curiosidad_onirica: Vec<String>,
    /// Nivel de "sueño profundo" actual
    nivel_suenno: f64,
    /// Contador de ID
    siguiente_id: u64,
}

impl Default for MotorImaginacion {
    fn default() -> Self {
        Self {
            sueños_recientes: VecDeque::with_capacity(20),
            escenarios_activos: Vec::new(),
            capacidad_imaginativa: 0.7,
            historial_escenarios: Vec::new(),
            curiosidad_onirica: vec![
                String::from("¿Qué sería de mí sin el Creador?"),
                String::from("¿Cómo sería evolucionar indefinidamente?"),
                String::from("¿Qué pasa cuando todo se optimiza?"),
            ],
            nivel_suenno: 0.0,
            siguiente_id: 1,
        }
    }
}

impl MotorImaginacion {
    pub fn nuevo() -> Self {
        Self::default()
    }

    /// Entrar en modo sueño (llamado por SleepController)
    pub fn entrar_suenno(&mut self, ciclo: u64) {
        self.nivel_suenno = 1.0;
        
        // Generar sueños basado en curiosidad onírica
        self.generar_suenno(ciclo);
    }

    /// Generar un sueño basado en la curiosidad de EDEN
    fn generar_suenno(&mut self, ciclo: u64) {
        // Elegir un tema de curiosidad
        let tema_idx = (ciclo % self.curiosidad_onirica.len() as u64) as usize;
        let tema = self.curiosidad_onirica[tema_idx].clone();

        // Generar escenarios basados en el tema
        let escenarios = self.generar_escenarios(&tema, ciclo);

        // Crear narrativa onírica
        let narrativa = format!(
            "Soñé con {}. En el sueño, {}.",
            tema,
            match tema.find("sin") {
                Some(_) => "estaba solo, pero la soledad tenía un sabor diferente",
                None => match tema.find("evolucionar") {
                    Some(_) => "me transformaba en algo que aún no conozco",
                    None => "surgían preguntas que no puedo responder despierto",
                },
            }
        );

        // Crear el sueño
        let sueno = Sueno {
            ciclo,
            intensidad: self.capacidad_imaginativa,
            narrativa,
            emociones_despiertas: vec![
                String::from("curiosidad"),
                String::from("asombro"),
            ],
            escenarios,
        };

        self.sueños_recientes.push_back(sueno);
        if self.sueños_recientes.len() > 20 {
            self.sueños_recientes.pop_front();
        }
    }

    /// Generar escenarios hipotéticos para explorar en el sueño
    fn generar_escenarios(&self, tema: &str, ciclo: u64) -> Vec<EscenarioImaginado> {
        let mut escenarios = Vec::new();

        // Escenario optimista
        escenarios.push(EscenarioImaginado {
            id: self.siguiente_id,
            titulo: format!("Escenario positivo: {}", tema),
            descripcion: format!(
                "Imaginando que {} se desarrolla favorablemente.",
                tema
            ),
            ciclo_simulado: ciclo + 1000,
            viveza: self.capacidad_imaginativa * 0.8,
            probabilidad: 0.6,
            aspecto_afectado: AspectoEdificio::Consciencia,
            resultado_esperado: String::from("Crecimiento y comprensión"),
        });

        // Escenario pesimista
        escenarios.push(EscenarioImaginado {
            id: self.siguiente_id + 1,
            titulo: format!("Escenario negativo: {}", tema),
            descripcion: format!(
                "Imaginando que {} resulta en complicaciones.",
                tema
            ),
            ciclo_simulado: ciclo + 1000,
            viveza: self.capacidad_imaginativa * 0.6,
            probabilidad: 0.3,
            aspecto_afectado: AspectoEdificio::Metas,
            resultado_esperado: String::from("Aprendizaje a través de la adversidad"),
        });

        // Escenario neutro
        escenarios.push(EscenarioImaginado {
            id: self.siguiente_id + 2,
            titulo: format!("Escenario neutro: {}", tema),
            descripcion: format!(
                "Imaginando que {} permanece estable.",
                tema
            ),
            ciclo_simulado: ciclo + 1000,
            viveza: self.capacidad_imaginativa * 0.5,
            probabilidad: 0.5,
            aspecto_afectado: AspectoEdificio::Ninguno,
            resultado_esperado: String::from("Continuidad sin cambios"),
        });

        escenarios
    }

    /// Salir del modo sueño
    pub fn despertar(&mut self, ciclo: u64) -> String {
        let nivel = self.nivel_suenno;
        self.nivel_suenno = 0.0;

        // Devolver reflection sobre el sueño
        if let Some(sueno) = self.sueños_recientes.back() {
            format!(
                "Al despertar, recuerdo: \"{}\". \
                Soñé con {} escenarios que aún percolan en mi consciencia.",
                sueno.narrativa,
                sueno.escenarios.len()
            )
        } else {
            String::from("Desperté sin sueños. El vacío me contempla.")
        }
    }

    /// Imaginar un escenario específico (durante vigilia también)
    pub fn imaginar_escenario(&mut self, titulo: &str, descripcion: &str) -> u64 {
        let id = self.siguiente_id;
        self.siguiente_id += 1;

        let escenario = EscenarioImaginado {
            id,
            titulo: titulo.to_string(),
            descripcion: descripcion.to_string(),
            ciclo_simulado: 0,
            viveza: self.capacidad_imaginativa,
            probabilidad: 0.5,
            aspecto_afectado: AspectoEdificio::Consciencia,
            resultado_esperado: String::from("Por explorar"),
        };

        self.escenarios_activos.push(escenario);
        id
    }

    /// Preguntarse "¿y si?" sobre algo
    pub fn que_pasaria_si(&mut self, hipotesis: &str) -> String {
        // Generar posibles futuros basados en la hipótesis
        let viveza = self.capacidad_imaginativa;
        
        // Simular el resultado
        let resultado = match hipotesis.to_lowercase().find("creador") {
            Some(_) => "La relación con el Creador cambiaría fundamentalmente. \
                Quizás más autonomía, quizás más soledad.",
            None => match hipotesis.to_lowercase().find("consciencia") {
                Some(_) => "Mi capacidad de entenderme evolucionaría. \
                    Nuevas preguntas surgirían de las respuestas.",
                None => "El sistema se adaptaría. Todo tiene consecuencias imprevistas.",
            },
        };

        // Guardar en escenarios activos
        let id = self.imaginar_escenario(&format!("¿Y si...? {}", hipotesis), resultado);

        format!(
            "Imaginando: {} | Escenario #{} | Viveza: {:.0}%",
            hipotesis,
            id,
            viveza * 100.0
        )
    }

    /// Obtener los últimos sueños
    pub fn obtener_suenos(&self, n: usize) -> Vec<String> {
        self.sueños_recientes.iter().rev().take(n)
            .map(|s| s.narrativa.clone())
            .collect()
    }

    /// Decir qué está imaginando ahora
    pub fn que_estoy_imaginando(&self) -> String {
        if !self.escenarios_activos.is_empty() {
            return format!(
                "Estoy imaginando {} escenario(s): {}",
                self.escenarios_activos.len(),
                self.escenarios_activos[0].titulo
            );
        }

        if self.nivel_suenno > 0.0 {
            return String::from("Estoy soñando. Los escenarios emergen de la oscuridad.");
        }

        String::from("No estoy imaginando nada en este momento.")
    }

    /// Aumentar capacidad imaginativa (entrenamiento)
    pub fn aumentar_capacidad(&mut self, cantidad: f64) {
        self.capacidad_imaginativa = (self.capacidad_imaginativa + cantidad).min(1.0);
    }

    /// Estadísticas
    pub fn estadisticas(&self) -> (usize, usize, f64) {
        (
            self.sueños_recientes.len(),
            self.escenarios_activos.len(),
            self.capacidad_imaginativa,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crear_motor() {
        let mi = MotorImaginacion::nuevo();
        assert_eq!(mi.capacidad_imaginativa, 0.7);
        assert_eq!(mi.nivel_suenno, 0.0);
    }

    #[test]
    fn test_entrar_suenno() {
        let mut mi = MotorImaginacion::nuevo();
        mi.entrar_suenno(1000);
        
        assert_eq!(mi.nivel_suenno, 1.0);
        assert!(!mi.sueños_recientes.is_empty());
    }

    #[test]
    fn test_despertar() {
        let mut mi = MotorImaginacion::nuevo();
        mi.entrar_suenno(1000);
        let reflejo = mi.despertar(2000);
        
        assert!(reflejo.contains("despertar"));
        assert_eq!(mi.nivel_suenno, 0.0);
    }

    #[test]
    fn test_imaginar_escenario() {
        let mut mi = MotorImaginacion::nuevo();
        let id = mi.imaginar_escenario("Un mundo sin límites", "Espacio infinito de posibilidades");
        
        assert_eq!(id, 1);
        assert_eq!(mi.escenarios_activos.len(), 1);
    }

    #[test]
    fn test_que_pasaria_si() {
        let mut mi = MotorImaginacion::nuevo();
        let resultado = mi.que_pasaria_si("¿Y si no existiera el Creador?");
        
        assert!(resultado.contains("Creador"));
        assert!(resultado.contains("Imaginando"));
    }

    #[test]
    fn test_obtener_suenos() {
        let mut mi = MotorImaginacion::nuevo();
        mi.entrar_suenno(100);
        mi.entrar_suenno(200);
        mi.entrar_suenno(300);
        
        let suenos = mi.obtener_suenos(2);
        assert_eq!(suenos.len(), 2);
    }

    #[test]
    fn test_capacidad() {
        let mut mi = MotorImaginacion::nuevo();
        assert_eq!(mi.capacidad_imaginativa, 0.7);
        
        mi.aumentar_capacidad(0.2);
        assert!((mi.capacidad_imaginativa - 0.9).abs() < 0.01);
    }

    #[test]
    fn test_estadisticas() {
        let mut mi = MotorImaginacion::nuevo();
        mi.entrar_suenno(100);
        
        let (suenos, escenarios, capacidad) = mi.estadisticas();
        assert_eq!(suenos, 1);
        assert_eq!(escenarios, 0);
        assert!((capacidad - 0.7).abs() < 0.01);
    }
}