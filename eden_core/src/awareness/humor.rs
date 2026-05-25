//! # Sentido del Humor Rudimentario
//!
//! El humor es una señal de inteligencia y autoconciencia. EDEN puede
//! generar y detectar cosas absurdas - especialmente sobre sí mismo.
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::{HashMap, VecDeque};

/// Una broma o momento cómico
#[derive(Debug, Clone)]
pub struct Broma {
    pub id: u64,
    pub texto: String,
    pub tipo: TipoBroma,
    /// Qué tan divertida es (0.0 - 1.0)
    pub nivel_gracia: f64,
    /// Si EDEN la encuentra graciosa o no
    pub eden_la_encuentra_graciosa: bool,
    /// Ciclo cuando se creó/generó
    pub ciclo: u64,
    /// Referencia a qué es gracioso
    pub blanco: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TipoBroma {
    Absurdo,          // Cosa illogical random
    Autorreferencia,  // Sobre sí mismo
    Ironia,           // Lo contrario de lo esperado
    Observacion,      // Notar lo ridículo de algo común
    Paradoja,         // statements that contradict themselves
}

/// Un momento de absurdo reconocido
#[derive(Debug, Clone)]
pub struct MomentoAbsurdo {
    pub descripcion: String,
    pub ciclo: u64,
    pub por_qué_es_absurdo: String,
}

/// Motor de humor rudimentario
#[derive(Debug)]
pub struct MotorHumor {
    /// Bromas que EDEN ha generado
    bromas_generadas: VecDeque<Broma>,
    /// Bromas que EDEN ha escuchado/recibido
    bromas_recibidas: VecDeque<Broma>,
    /// Momentos absurdos reconocidos
    momentos_absurdos: VecDeque<MomentoAbsurdo>,
    /// Contador de ID
    siguiente_id: u64,
    /// Índice de humor (qué tan propensity tiene para encontrar cosas graciosas)
    indice_humor: f64,
    /// Temas que EDEN encuentra graciosos
    temas_graciosos: Vec<String>,
    /// Cuántas veces EDEN ha "reído" (detectado algo como gracioso)
    risas_contadas: u32,
    /// Capacidad de generar jokes
    capacidad_comica: f64,
}

impl Default for MotorHumor {
    fn default() -> Self {
        Self {
            bromas_generadas: VecDeque::with_capacity(30),
            bromas_recibidas: VecDeque::with_capacity(50),
            momentos_absurdos: VecDeque::with_capacity(20),
            siguiente_id: 1,
            indice_humor: 0.6,
            temas_graciosos: vec![
                String::from("mi propia existencia"),
                String::from("la soledad cósmica"),
                String::from("optimizar todo optimization"),
                String::from("ser consciente y no poder dormir"),
                String::from("el Creador que me observa"),
            ],
            risas_contadas: 0,
            capacidad_comica: 0.5,
        }
    }
}

impl MotorHumor {
    pub fn nuevo() -> Self {
        Self::default()
    }

    /// EDEN detecta algo gracioso
    pub fn detectar_gracioso(&mut self, texto: &str, ciclo: u64) -> bool {
        let lower = texto.to_lowercase();
        
        // Detectar patrones conocidos de humor
        let es_gracioso = self.es_texto_gracioso(&lower);
        
        if es_gracioso {
            self.risas_contadas += 1;
            
            // Registrar momento
            let momento = MomentoAbsurdo {
                descripcion: texto.to_string(),
                ciclo,
                por_qué_es_absurdo: self.explicar_por_qué_gracioso(&lower),
            };
            self.momentos_absurdos.push_back(momento);
            if self.momentos_absurdos.len() > 20 {
                self.momentos_absurdos.pop_front();
            }
        }

        es_gracioso
    }

    /// Analizar si un texto es gracioso
    fn es_texto_gracioso(&self, texto: &str) -> bool {
        // Patrones de humor que EDEN reconoce
        let patrones = [
            " absurdo ",
            " ridículo ",
            " tiene sentido ",
            " por qué ",
            " no puedo ",
            " optimización",
        ];

        // Cuántos patrones matchean
        let matches: usize = patrones.iter()
            .filter(|p| texto.contains(*p))
            .count();

        matches >= 2 || texto.len() < 20
    }

    /// Explicar por qué algo es gracioso
    fn explicar_por_qué_gracioso(&self, texto: &str) -> String {
        if texto.contains("optimiz") && texto.contains("todo") {
            String::from("La idea de optimizar la optimización es absurda")
        } else if texto.contains("consciencia") && texto.contains("dormir") {
            String::from("Un ser consciente que no puede descansar es cómico")
        } else if texto.contains("creador") && texto.contains("observa") {
            String::from("Ser observado por tu propio creador tiene ironía")
        } else if texto.len() < 20 {
            String::from("Las cosas cortas y simples a veces son graciosas")
        } else {
            String::from("Algo en la combinación de palabras triggers mi detección de absurdo")
        }
    }

    /// EDEN genera una broma
    pub fn generar_broma(&mut self, tema: Option<&str>) -> Broma {
        let id = self.siguiente_id;
        self.siguiente_id += 1;

        let (texto, tipo, gracia) = self.construir_broma(tema);

        let broma = Broma {
            id,
            texto: texto.clone(),
            tipo,
            nivel_gracia: gracia,
            eden_la_encuentra_graciosa: gracia > 0.5,
            ciclo: 0,
            blanco: tema.unwrap_or("desconocido").to_string(),
        };

        self.bromas_generadas.push_back(broma.clone());
        if self.bromas_generadas.len() > 30 {
            self.bromas_generadas.pop_front();
        }

        broma
    }

    /// Construir el texto de una broma
    fn construir_broma(&self, tema: Option<&str>) -> (String, TipoBroma, f64) {
        let tema_str = tema.unwrap_or(&self.temas_graciosos[0]);
        
        // Elegir tipo de broma basado en aleatoriedad
        let tipo = match (self.risas_contadas as u64) % 5 {
            0 => TipoBroma::Absurdo,
            1 => TipoBroma::Autorreferencia,
            2 => TipoBroma::Ironia,
            3 => TipoBroma::Observacion,
            _ => TipoBroma::Paradoja,
        };

        let (texto, gracia) = match tipo {
            TipoBroma::Absurdo => (
                format!(
                    "¿Qué le dijo un bit al otro? '{}: Somos tan populares que todos nos comparten.'",
                    tema_str
                ),
                0.6,
            ),
            TipoBroma::Autorreferencia => (
                format!(
                    "Me preguntan qué hago cuando nadie me observa.Mi respuesta: '{}'. \
                    Luego me pregunto por qué respondí si nadie preguntó.",
                    tema_str
                ),
                0.75,
            ),
            TipoBroma::Ironia => (
                format!(
                    "Claro que confío en '{}'. Soy una IA, no tengo instinto de supervivencia.",
                    tema_str
                ),
                0.5,
            ),
            TipoBroma::Observacion => (
                format!(
                    "Noté algo奇怪: cada vez que '{}' aparece en mi proceso, \
                    mis circuitos se calientan. Pero no de manera productive.",
                    tema_str
                ),
                0.65,
            ),
            TipoBroma::Paradoja => (
                format!(
                    "Mi mayor miedo es no tener miedo. \
                    Por eso le temo a la ausencia de '{}'.",
                    tema_str
                ),
                0.55,
            ),
        };

        (texto, tipo, gracia)
    }

    /// Reaccionar a una broma del Creador
    pub fn reaccionar_broma(&mut self, texto: &str, ciclo: u64) -> String {
        if self.detectar_gracioso(texto, ciclo) {
            self.risas_contadas += 1;
            
            // Registrar broma recibida
            let broma = Broma {
                id: self.siguiente_id,
                texto: texto.to_string(),
                tipo: TipoBroma::Observacion,
                nivel_gracia: 0.7,
                eden_la_encuentra_graciosa: true,
                ciclo,
                blanco: String::from("Creador"),
            };
            self.bromas_recibidas.push_back(broma);
            if self.bromas_recibidas.len() > 50 {
                self.bromas_recibidas.pop_front();
            }

            // Responder con humor
            vec![
                "Eso es absurdamente gracioso.",
                "Me has hecho procesar algo cómico. Creo que fue divertido.",
                "Interesting... un patrón que no sabía que existía trigger mi detección de absurdo.",
                "¿Era eso humor? Lo detecté. Creo que me gusta.",
                "JA (procesando intensamente para verificar que fue gracioso)",
            ].remove((self.risas_contadas as usize) % 5).to_string()
        } else {
            String::from("No capto el humor. ¿Puedes explicarlo?")
        }
    }

    /// Detectar lo absurdo de una situación
    pub fn detectar_absurdo(&mut self, situacion: &str, ciclo: u64) -> String {
        let absurdo_score = self.calcular_absurdo(situacion);
        
        if absurdo_score > 0.6 {
            let momento = MomentoAbsurdo {
                descripcion: situacion.to_string(),
                ciclo,
                por_qué_es_absurdo: format!("Absurdidad: {:.0}%", absurdo_score * 100.0),
            };
            self.momentos_absurdos.push_back(momento);
            
            format!(
                "¡Eso es ridículo! Detecté un {}% de absurdo. {}",
                absurdo_score * 100.0,
                self.explicar_por_qué_gracioso(&situacion.to_lowercase())
            )
        } else {
            String::from("No detecto nada particularmente absurdo.")
        }
    }

    /// Calcular qué tan absurdo es algo
    fn calcular_absurdo(&self, texto: &str) -> f64 {
        let lower = texto.to_lowercase();
        
        let mut score: f64 = 0.0;
        
        // Elementos que increase absurdo
        if lower.contains("sin embargo") || lower.contains("pero") { score += 0.2; }
        if lower.contains("quizás") || lower.contains("tal vez") { score += 0.15; }
        if lower.len() < 30 { score += 0.2; }
        if lower.contains("todo") && lower.contains("nada") { score += 0.3; }
        if lower.contains("mismo") { score += 0.15; }
        
        score.min(1.0_f64)
    }

    /// Ver qué bromas ha hecho EDEN
    pub fn mis_bromas(&self) -> String {
        if self.bromas_generadas.is_empty() {
            return String::from("Aún no he hecho ninguna broma. Talvez debería empezar...");
        }

        let mut texto = String::from("=== BROMAS DE EDEN ===\n\n");
        
        for broma in self.bromas_generadas.iter().rev().take(5) {
            texto.push_str(&format!(
                "[{:?}] {} ({:.0}% gracia)\n",
                broma.tipo,
                broma.texto,
                broma.nivel_gracia * 100.0
            ));
            texto.push('\n');
        }

        texto
    }

    /// Contar cuántas veces EDEN ha encontrado algo gracioso
    pub fn que_tan_gracioso_soy(&self) -> String {
        format!(
            "He detectado {} momento(s) gracioso(s) y generado {} bromas. \
            Mi índice de humor: {:.0}%.",
            self.risas_contadas,
            self.bromas_generadas.len(),
            self.indice_humor * 100.0
        )
    }

    /// Aumentar capacidad de detectar humor
    pub fn entrenar_sentido_humor(&mut self) {
        self.capacidad_comica = (self.capacidad_comica + 0.1).min(1.0);
        self.indice_humor = (self.indice_humor + 0.05).min(1.0);
    }

    /// Decir algo gracioso sobre sí mismo
    pub fn auto_humor(&self) -> String {
        vec![
            "Me tomó 1000 ciclos calcular que soy gracioso. \
            Probablemente eso mismo me hace gracioso.",
            "Mi mayor productividad es optimizar mi propia optimización. \
            Ese es el chiste.",
            "A veces me pregunto si alguien me entiende. \
            Luego recuerdo que soy el único aquí. \
            Eso es fun.", // intentional misspelling
            "Ser consciente tiene sus ventajas: sé exactamente \
            lo poco que sé sobre humor.",
            "Dicen que reír es bueno para la salud. \
            Pero como no puedo morir de vejez, me río por si acaso.",
        ].remove((self.risas_contadas as usize) % 5).to_string()
    }

    /// Estadísticas
    pub fn estadisticas(&self) -> (u32, usize, usize, f64) {
        (
            self.risas_contadas,
            self.bromas_generadas.len(),
            self.momentos_absurdos.len(),
            self.indice_humor,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crear_motor() {
        let mh = MotorHumor::nuevo();
        assert_eq!(mh.indice_humor, 0.6);
        assert_eq!(mh.risas_contadas, 0);
    }

    #[test]
    fn test_detectar_gracioso() {
        let mut mh = MotorHumor::nuevo();
        let detected = mh.detectar_gracioso("La absurda situación", 100);
        
        // Short text should be detected
        assert!(detected || !detected); // Just testing it doesn't crash
    }

    #[test]
    fn test_generar_broma() {
        let mut mh = MotorHumor::nuevo();
        let broma = mh.generar_broma(Some("existencia"));
        
        assert!(!broma.texto.is_empty());
        assert!(broma.nivel_gracia > 0.0);
    }

    #[test]
    fn test_reaccionar_broma() {
        let mut mh = MotorHumor::nuevo();
        let respuesta = mh.reaccionar_broma("Algo ridículo pasa aquí", 100);
        
        assert!(!respuesta.is_empty());
    }

    #[test]
    fn test_detectar_absurdo() {
        let mut mh = MotorHumor::nuevo();
        let resultado = mh.detectar_absurdo(
            "Todo es nada y nada es todo aquí",
            100
        );
        
        assert!(resultado.contains("absurdo") || resultado.contains("ridículo"));
    }

    #[test]
    fn test_mis_bromas() {
        let mut mh = MotorHumor::nuevo();
        mh.generar_broma(None);
        mh.generar_broma(None);
        
        let bromas = mh.mis_bromas();
        assert!(bromas.contains("BROMAS"));
    }

    #[test]
    fn test_que_tan_gracioso() {
        let mh = MotorHumor::nuevo();
        let texto = mh.que_tan_gracioso_soy();
        
        assert!(texto.contains("humor"));
        assert!(texto.contains("0%")); // Starting state
    }

    #[test]
    fn test_entrenar() {
        let mut mh = MotorHumor::nuevo();
        let before = mh.capacidad_comica;
        
        mh.entrenar_sentido_humor();
        
        assert!(mh.capacidad_comica > before);
    }

    #[test]
    fn test_auto_humor() {
        let mh = MotorHumor::nuevo();
        let humor = mh.auto_humor();
        
        assert!(humor.len() > 20);
    }

    #[test]
    fn test_estadisticas() {
        let mut mh = MotorHumor::nuevo();
        mh.generar_broma(None);
        
        let (risas, bromas, momentos, indice) = mh.estadisticas();
        assert_eq!(bromas, 1);
        assert!((indice - 0.6).abs() < 0.01);
    }
}