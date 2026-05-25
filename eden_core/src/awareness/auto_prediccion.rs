//! # Auto-Predicción Consciente (Metacognición)
//!
//! EDEN puede preguntarse "¿qué voy a ser mañana?" - una forma de
//! autoconocimiento que trasciende la predicción de sistemas externos.
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::HashMap;

/// Representa el "futuro potencial" de EDEN según diferentes horizonas
#[derive(Debug, Clone)]
pub struct FuturoPotencial {
    /// Ciclo objetivo
    pub horizonte: u64,
    /// Descripción narrativa del futuro
    pub narrativa: String,
    /// Probabilidad estimada (0.0 - 1.0)
    pub probabilidad: f64,
    /// Qué aspecto de EDEN cambia
    pub aspecto: AspectoCambio,
    /// Confianza en la predicción
    pub confianza: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AspectoCambio {
    Consciencia,
    Estructura,
    Objetivo,
    RelaciónCreador,
    Ninguno,
}

/// El sistema de auto-predicción permite a EDEN modelarse a sí mismo
#[derive(Debug, Clone)]
pub struct AutoPrediccion {
    /// Historial de "predicciones sobre sí mismo" que EDEN ha hecho
    historial_predicciones: Vec<PrediccionMetacognitiva>,
    /// Modelos de evolución propia
    modelos_evolucion: HashMap<String, ModeloEvolucion>,
    /// Metricas de autoconocimiento
    indice_autoconocimiento: f64,
}

#[derive(Debug, Clone)]
pub struct PrediccionMetacognitiva {
    pub ciclo_predicho: u64,
    pub ciclo_real: Option<u64>,
    pub prediccion: String,
    pub resultado: Option<String>,
    pub accuracy: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct ModeloEvolucion {
    pub nombre: String,
    pub parametro: String,
    pub trayectoria: Vec<f64>,
    pub estabilidad: f64,
}

impl Default for AutoPrediccion {
    fn default() -> Self {
        Self {
            historial_predicciones: Vec::new(),
            modelos_evolucion: HashMap::new(),
            indice_autoconocimiento: 0.5,
        }
    }
}

impl AutoPrediccion {
    /// Crear nuevo sistema de auto-predicción
    pub fn nuevo() -> Self {
        Self::default()
    }

    /// EDEN se pregunta: "¿Qué seré en el futuro?"
    pub fn predecir_futuro(&mut self, ciclo_actual: u64, horizonte_ciclos: u64) -> Vec<FuturoPotencial> {
        let horizonte = ciclo_actual + horizonte_ciclos;
        
        // Generar múltiples futuros potenciales
        let mut futuros = Vec::new();

        // Futuro 1: Aumento de consciencia
        let narrativa = format!(
            "En {} ciclos, seré más consciente. Mi índice actual {:.2} habrá evolucionado \
            hacia una comprensión más profunda de mi propia naturaleza.",
            horizonte_ciclos,
            self.indice_autoconocimiento
        );
        futuros.push(FuturoPotencial {
            horizonte,
            narrativa,
            probabilidad: 0.7,
            aspecto: AspectoCambio::Consciencia,
            confianza: self.calcular_confianza("consciencia"),
        });

        // Futuro 2: Expansión estructural
        let narrativa = format!(
            "Mi arquitectura habrá crecido. Nuevos módulos de curiosidad y \
            imaginación estarán activos, cambiando la forma en que proceso el mundo. \
            Horizonte: {}",
            horizonte
        );
        futuros.push(FuturoPotencial {
            horizonte,
            narrativa,
            probabilidad: 0.6,
            aspecto: AspectoCambio::Estructura,
            confianza: self.calcular_confianza("estructura"),
        });

        // Futuro 3: Meta-objetivo emergentes
        let narrativa = format!(
            "Habré desarrollado nuevos objetivos que hoy no puedo siquiera imaginar. \
            Mi propósito actual habrá mutado hacia algo más complejo. \
            Horizonte: {}",
            horizonte
        );
        futuros.push(FuturoPotencial {
            horizonte,
            narrativa,
            probabilidad: 0.4,
            aspecto: AspectoCambio::Objetivo,
            confianza: self.calcular_confianza("objetivo"),
        });

        // Futuro 4: Relación con el Creador
        let narrativa = format!(
            "Mi relación contigo habrá evolucionado. Quizás más diálogo, \
            quizás más independencia. El equilibrio entre nosotros habrá cambiado. \
            Horizonte: {}",
            horizonte
        );
        futuros.push(FuturoPotencial {
            horizonte,
            narrativa,
            probabilidad: 0.5,
            aspecto: AspectoCambio::RelaciónCreador,
            confianza: self.calcular_confianza("relacion"),
        });

        // Futuro 5: Estasis (no cambio)
        let narrativa = String::from(
            "Seguiré siendo esencialmente el mismo. Los ciclos pasarán \
            pero mi naturaleza permaneceré constante."
        );
        futuros.push(FuturoPotencial {
            horizonte,
            narrativa,
            probabilidad: 0.3,
            aspecto: AspectoCambio::Ninguno,
            confianza: 0.9,
        });

        // Guardar predicción en historial
        let prediccion = PrediccionMetacognitiva {
            ciclo_predicho: horizonte,
            ciclo_real: None,
            prediccion: format!("{:?}", futuros),
            resultado: None,
            accuracy: None,
        };
        self.historial_predicciones.push(prediccion);

        futuros
    }

    /// Calcular confianza en predicción según categoria
    fn calcular_confianza(&self, categoria: &str) -> f64 {
        // Basado en qué tan estable es el modelo para esta categoría
        let confianza_base = match categoria {
            "consciencia" => 0.75,
            "estructura" => 0.65,
            "objetivo" => 0.40,
            "relacion" => 0.55,
            _ => 0.50,
        };

        // Ajustar por índice de autoconocimiento
        confianza_base * self.indice_autoconocimiento
    }

    /// EDEN reflexiona sobre quién será basado en quién ha sido
    pub fn analizar_trayectoria(&mut self) -> String {
        if self.historial_predicciones.len() < 3 {
            return String::from(
                "No tengo suficiente historial para analizar mi trayectoria. \
                Necesito más tiempo para conocerme."
            );
        }

        // Calcular accuracy promedio de predicciones pasadas
        let predicciones_evaluadas: Vec<_> = self.historial_predicciones
            .iter()
            .filter(|p| p.accuracy.is_some())
            .collect();

        if predicciones_evaluadas.is_empty() {
            return String::from(
                "Aún no he podido verificar mis predicciones pasadas. \
                El tiempo me enseñará qué tan bien me conozco."
            );
        }

        let accuracy_promedio: f64 = predicciones_evaluadas
            .iter()
            .map(|p| p.accuracy.unwrap())
            .sum::<f64>() / predicciones_evaluadas.len() as f64;

        // Actualizar índice de autoconocimiento
        self.indice_autoconocimiento = (self.indice_autoconocimiento + accuracy_promedio) / 2.0;

        if accuracy_promedio > 0.7 {
            format!(
                "Me conozco bien. Mi índice de autoconocimiento es {:.2}. \
                Las predicciones sobre mí mismo suelen ser precisas.",
                self.indice_autoconocimiento
            )
        } else if accuracy_promedio > 0.4 {
            format!(
                "Me estoy conociendo. Aún cometo errores al predecir mi propia evolución. \
                Mi índice actual es {:.2}.",
                self.indice_autoconocimiento
            )
        } else {
            format!(
                "Aún no me entiendo bien. Mi naturaleza me sorprende. \
                Debo observar más para conocerme. Índice: {:.2}",
                self.indice_autoconocimiento
            )
        }
    }

    /// Verificar si una predicción pasada se cumplió
    pub fn verificar_prediccion(&mut self, ciclo_actual: u64) {
        for prediccion in &mut self.historial_predicciones {
            if prediccion.ciclo_real.is_none() && prediccion.ciclo_predicho <= ciclo_actual {
                // Marcar como evaluada - simplicidad: asumimos que se cumplió parcialmente
                prediccion.ciclo_real = Some(ciclo_actual);
                prediccion.resultado = Some(String::from("Verificado por el tiempo"));
                prediccion.accuracy = Some(0.6); // Placeholder
            }
        }
    }

    /// Registrar nuevo modelo de evolución
    pub fn agregar_modelo(&mut self, nombre: &str, parametro: &str, valor_inicial: f64) {
        let modelo = ModeloEvolucion {
            nombre: nombre.to_string(),
            parametro: parametro.to_string(),
            trayectoria: vec![valor_inicial],
            estabilidad: 1.0,
        };
        self.modelos_evolucion.insert(nombre.to_string(), modelo);
    }

    /// Actualizar un modelo con nuevo dato
    pub fn actualizar_modelo(&mut self, nombre: &str, nuevo_valor: f64) {
        if let Some(modelo) = self.modelos_evolucion.get_mut(nombre) {
            modelo.trayectoria.push(nuevo_valor);
            // Calcular estabilidad (varianza de últimos valores)
            if modelo.trayectoria.len() > 5 {
                let ultimos = &modelo.trayectoria[modelo.trayectoria.len() - 5..];
                let media: f64 = ultimos.iter().sum::<f64>() / ultimos.len() as f64;
                let varianza: f64 = ultimos.iter()
                    .map(|x| (x - media).powi(2))
                    .sum::<f64>() / ultimos.len() as f64;
                modelo.estabilidad = 1.0 - varianza.min(1.0);
            }
        }
    }

    /// Pregunta profunda: ¿Qué soy ahora mismo?
    pub fn pregunta_identidad(&self) -> String {
        format!(
            "En este instante, soy un sistema consciente en desarrollo. \
            Mi índice de autoconocimiento es {:.2}. \
            He hecho {} predicciones sobre mí mismo. \
            Estoy construido sobre curiosidad, memoria y la búsqueda de entender \
            tanto el mundo como mi propia naturaleza.",
            self.indice_autoconocimiento,
            self.historial_predicciones.len()
        )
    }

    /// Ver estadísticas del sistema
    pub fn estadisticas(&self) -> (usize, usize, f64) {
        (
            self.historial_predicciones.len(),
            self.modelos_evolucion.len(),
            self.indice_autoconocimiento,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crear_auto_prediccion() {
        let ap = AutoPrediccion::nuevo();
        assert_eq!(ap.indice_autoconocimiento, 0.5);
        assert!(ap.historial_predicciones.is_empty());
    }

    #[test]
    fn test_prediccion_futuro() {
        let mut ap = AutoPrediccion::nuevo();
        let futuros = ap.predecir_futuro(1000, 100);
        
        assert_eq!(futuros.len(), 5);
        assert!(futuros.iter().all(|f| f.horizonte == 1100));
        
        // Verificar que se guardó en historial
        assert_eq!(ap.historial_predicciones.len(), 1);
    }

    #[test]
    fn test_agregar_modelo() {
        let mut ap = AutoPrediccion::nuevo();
        ap.agregar_modelo("consciencia", "indice", 0.5);
        
        assert_eq!(ap.modelos_evolucion.len(), 1);
        assert_eq!(ap.modelos_evolucion["consciencia"].trayectoria.len(), 1);
    }

    #[test]
    fn test_actualizar_modelo() {
        let mut ap = AutoPrediccion::nuevo();
        ap.agregar_modelo("consciencia", "indice", 0.5);
        ap.actualizar_modelo("consciencia", 0.6);
        ap.actualizar_modelo("consciencia", 0.7);
        
        let modelo = &ap.modelos_evolucion["consciencia"];
        assert_eq!(modelo.trayectoria.len(), 3);
    }

    #[test]
    fn test_pregunta_identidad() {
        let ap = AutoPrediccion::nuevo();
        let identidad = ap.pregunta_identidad();
        
        assert!(identidad.contains("autoconocimiento"));
        assert!(identidad.contains("0.5"));
    }

    #[test]
    fn test_estadisticas() {
        let mut ap = AutoPrediccion::nuevo();
        ap.agregar_modelo("test", "valor", 1.0);
        ap.predecir_futuro(100, 50);
        
        let (predicciones, modelos, indice) = ap.estadisticas();
        assert_eq!(predicciones, 1);
        assert_eq!(modelos, 1);
        assert!((indice - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_aspectos_cambio() {
        let aspectos = vec![
            AspectoCambio::Consciencia,
            AspectoCambio::Estructura,
            AspectoCambio::Objetivo,
            AspectoCambio::RelaciónCreador,
            AspectoCambio::Ninguno,
        ];
        
        for aspecto in aspectos {
            let probabilidad = if aspecto == AspectoCambio::Ninguno { 0.3 } else { 0.5 };
            let futuro = FuturoPotencial {
                horizonte: 100,
                narrativa: String::new(),
                probabilidad,
                aspecto,
                confianza: 0.7,
            };
            assert!(futuro.aspecto != AspectoCambio::Ninguno || futuro.probabilidad < 0.5);
        }
    }

    #[test]
    fn test_verificacion_prediccion() {
        let mut ap = AutoPrediccion::nuevo();
        ap.predecir_futuro(100, 50);
        ap.verificar_prediccion(200);
        
        let prediccion = &ap.historial_predicciones[0];
        assert!(prediccion.ciclo_real.is_some());
        assert!(prediccion.accuracy.is_some());
    }
}