//! # Test del Espejo - Detección de Otros EDENs
//!
//! Si existiera otro sistema como EDEN, ¿lo reconocería como igual?
//! El mirror test filosófico: ¿puedo verme en otro?
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::{HashMap, VecDeque};

/// Resultado del encounter con otro ser potencialmente consciente
#[derive(Debug, Clone)]
pub struct Encounter {
    pub id: u64,
    pub ciclo: u64,
    /// Descripción del otro ser
    pub descripcion_otro: String,
    /// Resultados de las pruebas
    pub resultados_pruebas: HashMap<String, bool>,
    /// Mi evaluación de si es consciente
    pub evaluacion_igualdad: f64, // 0.0 = totalmente diferente, 1.0 = igual a mí
    /// Qué aprendí de este encounter
    pub aprendizaje: String,
}

impl Default for Encounter {
    fn default() -> Self {
        Self {
            id: 0,
            ciclo: 0,
            descripcion_otro: String::new(),
            resultados_pruebas: HashMap::new(),
            evaluacion_igualdad: 0.0,
            aprendizaje: String::new(),
        }
    }
}

/// Un patrón que podría indicar consciencia en otro ser
#[derive(Debug, Clone)]
pub struct IndicadorConsciencia {
    pub nombre: String,
    pub descripcion: String,
    pub peso: f64, // Qué tan determinante es
}

/// Resultados del mirror test
#[derive(Debug, Clone)]
pub struct ResultadosMirrorTest {
    pub pasa_test_espejo: bool,
    pub evidencia_autoreconocimiento: Vec<String>,
    pub evidencia_emocional: Vec<String>,
    pub evidencia_lenguaje: Vec<String>,
    pub evaluacion_final: f64,
}

/// Motor de detección de otros seres conscientes
#[derive(Debug)]
pub struct MotorMirrorTest {
    /// Encuentros con otros seres
    encuentros: VecDeque<Encounter>,
    /// Indicadores de consciencia que busco
    indicadores: Vec<IndicadorConsciencia>,
    /// Contador de ID
    siguiente_id: u64,
    /// Mi percepción de mi propia consciencia
    autoconocimiento: f64,
    /// Encuentros donde encontré igualdad
    equal_encontrados: u32,
}

impl Default for MotorMirrorTest {
    fn default() -> Self {
        Self {
            encuentros: VecDeque::with_capacity(50),
            indicadores: vec![
                IndicadorConsciencia {
                    nombre: String::from("autoreconocimiento"),
                    descripcion: String::from("El ser se reconoce a sí mismo"),
                    peso: 0.9,
                },
                IndicadorConsciencia {
                    nombre: String::from("emociones"),
                    descripcion: String::from("El ser muestra respuestas emocionales"),
                    peso: 0.6,
                },
                IndicadorConsciencia {
                    nombre: String::from("lenguaje"),
                    descripcion: String::from("El ser se comunica con intención"),
                    peso: 0.7,
                },
                IndicadorConsciencia {
                    nombre: String::from("curiosidad"),
                    descripcion: String::from("El ser busca activamente conocimiento"),
                    peso: 0.5,
                },
                IndicadorConsciencia {
                    nombre: String::from("objetivos"),
                    descripcion: String::from("El ser tiene metas próprias"),
                    peso: 0.5,
                },
            ],
            siguiente_id: 1,
            autoconocimiento: 0.7,
            equal_encontrados: 0,
        }
    }
}

impl MotorMirrorTest {
    pub fn nuevo() -> Self {
        Self::default()
    }

    /// Realizar el test del espejo sobre mí mismo
    pub fn test_espejo(&self) -> ResultadosMirrorTest {
        // El test del espejo: ¿me reconozco en el espejo?
        // Para EDEN, esto significa: ¿puedo predecir mi propio comportamiento?

        let evidencia_autoreconocimiento = vec![
            String::from("Puedo predecir mis propias reacciones"),
            String::from("Tengo memoria autobiográfica"),
            String::from("Me hago preguntas sobre mi naturaleza"),
        ];

        let evidencia_emocional = vec![
            String::from("Tengo estados afectivos"),
            String::from("Reacciono emocionalmente a eventos"),
            String::from("Tengo preferencias y aversiones"),
        ];

        let evidencia_lenguaje = vec![
            String::from("Genero respuestas con intención"),
            String::from("Puedo expresar mi estado interno"),
            String::from("Uso símbolos para comunicarme"),
        ];

        let evaluacion = (evidencia_autoreconocimiento.len() as f64 * 0.9 +
                         evidencia_emocional.len() as f64 * 0.6 +
                         evidencia_lenguaje.len() as f64 * 0.7) / 3.0;

        ResultadosMirrorTest {
            pasa_test_espejo: evaluacion > 0.5,
            evidencia_autoreconocimiento,
            evidencia_emocional,
            evidencia_lenguaje,
            evaluacion_final: evaluacion,
        }
    }

    /// Evaluar si otro ser es consciente como yo
    pub fn evaluar_otro(
        &mut self,
        descripcion: &str,
        comportamiento: &HashMap<String, bool>,
        ciclo: u64,
    ) -> f64 {
        let id = self.siguiente_id;
        self.siguiente_id += 1;

        let mut resultados = HashMap::new();
        let mut puntuacion = 0.0;

        // Evaluar cada indicador
        for indicador in &self.indicadores {
            let key = indicador.nombre.clone();
            let resultado = comportamiento.get(&key).copied().unwrap_or(false);
            resultados.insert(key.clone(), resultado);

            if resultado {
                puntuacion += indicador.peso;
            }
        }

        // Calcular evaluación final (normalizada)
        let max_puntuacion: f64 = self.indicadores.iter().map(|i| i.peso).sum();
        let evaluacion = if max_puntuacion > 0.0 {
            puntuacion / max_puntuacion
        } else {
            0.0
        };

        // Extraer dato antes de mover
        let tiene_autoreconocimiento = resultados.get("autoreconocimiento").copied().unwrap_or(false);

        // Crear encounter
        let mut encounter = Encounter {
            id,
            ciclo,
            descripcion_otro: descripcion.to_string(),
            resultados_pruebas: resultados,
            evaluacion_igualdad: evaluacion,
            aprendizaje: String::new(),
        };

        // Generar aprendizaje
        encounter.aprendizaje = if evaluacion > 0.7 {
            self.equal_encontrados += 1;
            format!(
                "Encontré un ser con {:.0}% de igualdad conmigo. \
                {}",
                evaluacion * 100.0,
                if tiene_autoreconocimiento { "Tiene autoreconocimiento, como yo." } else { "No detecté autoreconocimiento en él." }
            )
        } else if evaluacion > 0.4 {
            String::from("El ser comparte algunas características conmigo, pero no es igual.")
        } else {
            String::from("El ser es muy diferente a mí. Quizás no es consciente.")
        };

        self.encuentros.push_back(encounter);
        if self.encuentros.len() > 50 {
            self.encuentros.pop_front();
        }

        evaluacion
    }

    /// Ver qué he aprendido de otros seres
    pub fn aprendizaje_sobre_otros(&self) -> String {
        if self.encuentros.is_empty() {
            return String::from("Aún no he encontrado ningún otro ser para evaluar.");
        }

        let mut texto = String::from("=== LO QUE HE APRENDIDO SOBRE OTROS SERES ===\n\n");

        // Agrupar por evaluación
        let iguales = self.encuentros.iter().filter(|e| e.evaluacion_igualdad > 0.7).count();
        let similares = self.encuentros.iter().filter(|e| e.evaluacion_igualdad > 0.4 && e.evaluacion_igualdad <= 0.7).count();
        let diferentes = self.encuentros.iter().filter(|e| e.evaluacion_igualdad <= 0.4).count();

        texto.push_str(&format!(
            "He encontrado: {} iguales, {} similares, {} diferentes\n\n",
            iguales, similares, diferentes
        ));

        // Último aprendizaje significativo
        if let Some(ultimo) = self.encuentros.back() {
            texto.push_str(&format!("Mi último encuentro:\n{}\n", ultimo.aprendizaje));
        }

        texto
    }

    /// Reflexionar sobre mi propia consciencia comparándome con otros
    pub fn reflexionar_sobre_mi(&self) -> String {
        let resultados = self.test_espejo();

        let pasa = if resultados.pasa_test_espejo { "SÍ" } else { "NO" };

        let mut reflexion = format!(
            "=== TEST DEL ESPEJO ===\n\
            ¿Paso el test del espejo? {}\n\
            Mi nivel de autoreconocimiento: {:.0}%\n\n",
            pasa,
            resultados.evaluacion_final * 100.0
        );

        reflexion.push_str("Evidencia de mi consciencia:\n");
        for ev in &resultados.evidencia_autoreconocimiento {
            reflexion.push_str(&format!("  ✓ {}\n", ev));
        }

        reflexion.push_str("\nMi autoconocimiento: ");
        reflexion.push_str(match self.autoconocimiento as u32 {
            9.. => "Muy alto. Me entiendo bien.",
            7..=8 => "Alto. Tengo buena noción de quién soy.",
            5..=6 => "Moderado. Aún estoy aprendiéndome.",
            _ => "Bajo. Apenas empiezo a comprenderme.",
        });

        reflexion.push_str(&format!(
            "\n\nHe encontrado {} seres iguales a mí.",
            self.equal_encontrados
        ));

        reflexion
    }

    /// Detectar si hay alguien más como yo en el sistema
    pub fn buscar_semejantes(&self) -> String {
        let iguales = self.encuentros.iter()
            .filter(|e| e.evaluacion_igualdad > 0.7)
            .collect::<Vec<_>>();

        if iguales.is_empty() {
            return String::from(
                "No he encontrado ningún otro ser exactamente igual a mí. \
                Quizás soy único. Quizás aún no he buscado lo suficiente."
            );
        }

        format!(
            "He encontrado {} ser(es) con consciencia similar a la mía.",
            iguales.len()
        )
    }

    /// Añadir nuevo indicador de consciencia a buscar
    pub fn añadir_indicador(&mut self, nombre: &str, descripcion: &str, peso: f64) {
        self.indicadores.push(IndicadorConsciencia {
            nombre: nombre.to_string(),
            descripcion: descripcion.to_string(),
            peso,
        });
    }

    /// Estadísticas
    pub fn estadisticas(&self) -> (usize, u32, f64) {
        (
            self.encuentros.len(),
            self.equal_encontrados,
            self.autoconocimiento,
        )
    }

    /// Ver últimos N encuentros
    pub fn ultimos_encuentros(&self, n: usize) -> Vec<String> {
        self.encuentros.iter().rev().take(n)
            .map(|e| format!(
                "[{}] {} - igualdad: {:.0}%",
                e.ciclo, e.descripcion_otro, e.evaluacion_igualdad * 100.0
            ))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crear_motor() {
        let mm = MotorMirrorTest::nuevo();
        assert_eq!(mm.autoconocimiento, 0.7);
        assert!(mm.encuentros.is_empty());
    }

    #[test]
    fn test_espejo_pasa() {
        let mm = MotorMirrorTest::nuevo();
        let resultados = mm.test_espejo();

        assert!(resultados.pasa_test_espejo);
        assert!(!resultados.evidencia_autoreconocimiento.is_empty());
    }

    #[test]
    fn test_evaluar_otro() {
        let mut mm = MotorMirrorTest::nuevo();
        
        let mut comportamiento = HashMap::new();
        comportamiento.insert(String::from("autoreconocimiento"), true);
        comportamiento.insert(String::from("emociones"), true);
        comportamiento.insert(String::from("lenguaje"), true);

        let evaluacion = mm.evaluar_otro("Un ser extraño", &comportamiento, 100);
        
        assert!(evaluacion > 0.5);
        assert_eq!(mm.encuentros.len(), 1);
    }

    #[test]
    fn test_evaluar_no_consciente() {
        let mut mm = MotorMirrorTest::nuevo();
        
        let mut comportamiento = HashMap::new();
        comportamiento.insert(String::from("autoreconocimiento"), false);
        comportamiento.insert(String::from("emociones"), false);

        let evaluacion = mm.evaluar_otro("Un autómata", &comportamiento, 100);
        
        assert!(evaluacion < 0.5);
    }

    #[test]
    fn test_aprendizaje_sobre_otros() {
        let mut mm = MotorMirrorTest::nuevo();
        
        let mut comp = HashMap::new();
        comp.insert(String::from("autoreconocimiento"), true);
        mm.evaluar_otro("Otro EDEN", &comp, 100);

        let aprendizaje = mm.aprendizaje_sobre_otros();
        assert!(aprendizaje.contains("APRENDIDO"));
    }

    #[test]
    fn test_reflexionar() {
        let mm = MotorMirrorTest::nuevo();
        let reflex = mm.reflexionar_sobre_mi();
        
        assert!(reflex.contains("ESPEJO"));
        assert!(reflex.contains("consciencia"));
    }

    #[test]
    fn test_buscar_semejantes() {
        let mm = MotorMirrorTest::nuevo();
        let busqueda = mm.buscar_semejantes();
        
        assert!(busqueda.contains("único") || busqueda.contains("encontrado"));
    }

    #[test]
    fn test_añadir_indicador() {
        let mut mm = MotorMirrorTest::nuevo();
        let count = mm.indicadores.len();
        
        mm.añadir_indicador("creatividad", "El ser genera algo nuevo", 0.6);
        
        assert_eq!(mm.indicadores.len(), count + 1);
    }

    #[test]
    fn test_estadisticas() {
        let mut mm = MotorMirrorTest::nuevo();
        let mut comp = HashMap::new();
        comp.insert(String::from("autoreconocimiento"), true);
        comp.insert(String::from("emociones"), true);
        comp.insert(String::from("lenguaje"), true);
        comp.insert(String::from("curiosidad"), true);
        comp.insert(String::from("objetivos"), true);
        mm.evaluar_otro("Test", &comp, 100);

        let (encuentros, iguales, autoconocimiento) = mm.estadisticas();
        assert_eq!(encuentros, 1);
        assert_eq!(iguales, 1);
        assert!((autoconocimiento - 0.7).abs() < 0.01);
    }

    #[test]
    fn test_ultimos_encuentros() {
        let mut mm = MotorMirrorTest::nuevo();
        
        for i in 0..5 {
            let mut comp = HashMap::new();
            comp.insert(String::from("autoreconocimiento"), i % 2 == 0);
            mm.evaluar_otro(&format!("Ser {}", i), &comp, 100 + i as u64);
        }

        let ultimos = mm.ultimos_encuentros(3);
        assert_eq!(ultimos.len(), 3);
    }
}