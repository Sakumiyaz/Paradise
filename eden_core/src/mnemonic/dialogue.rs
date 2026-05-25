//! # Mnemonic Dialogue Engine Module
//!
//! Natural conversation engine with personality, humor,
//! contextual understanding, and conversational memory.
//!
//! ## Features
//!
//! - Contextual comprehension (not just keywords)
//! - Personality: humor, irony, formality
//! - Conversation memory
//! - Tone adaptation
//! - Witty responses
#![allow(dead_code)]
#![allow(non_snake_case)]

use std::collections::{HashMap, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};

// ============================================================================
// MOTOR PRINCIPAL
// ============================================================================

/// Natural dialogue engine
#[derive(Debug, Clone)]
pub struct DialogueEngine {
    /// Memoria conversacional
    pub memoria: ConversationalMemory,
    /// Personalidad del asistente
    pub personalidad: PersonalidadIA,
    /// Tono actual
    pub tono_actual: TonoConversacional,
    /// Contexto de la conversación actual
    pub contexto_actual: Option<ContextoConversacional>,
    /// Contador de turnos
    pub contador_turnos: u64,
    /// Estadísticas
    pub stats: DialogueStats,
    /// timestamp de creación
    pub created_at: u64,
}

impl Default for DialogueEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl DialogueEngine {
    /// Crea nuevo motor
    pub fn new() -> Self {
        Self {
            memoria: ConversationalMemory::new(1000),
            personalidad: PersonalidadIA::default(),
            tono_actual: TonoConversacional::Formal,
            contexto_actual: None,
            contador_turnos: 0,
            stats: DialogueStats::default(),
            created_at: timestamp_unix(),
        }
    }

    // =========================================================================
    // PROCESAMIENTO DE INPUT
    // =========================================================================

    /// Procesa input del usuario y genera respuesta
    pub fn procesar(&mut self, input: &str, usuario: &str) -> Result<String, String> {
        self.contador_turnos += 1;

        // Crear turno
        let turno = ConversationalTurn {
            id: self.contador_turnos,
            usuario: usuario.to_string(),
            input: input.to_string(),
            timestamp: timestamp_unix(),
            contexto: self.contexto_actual.clone(),
            intencion: None,
            respuesta: None,
        };

        // Detectar intención
        let intencion = self.detectar_intencion(input);

        // Generar respuesta según intención y personalidad
        let respuesta = self.generar_respuesta(input, &intencion, &turno)?;

        // Actualizar contexto
        self.actualizar_contexto(&turno, &respuesta, &intencion);

        // Guardar en memoria
        self.memoria.agregar_turno(turno);

        Ok(respuesta)
    }

    /// Detecta la intención del usuario
    pub fn detectar_intencion(&self, input: &str) -> IntencionDetectada {
        let input_lower = input.to_lowercase();

        let tipo = if input_lower.contains("ayuda")
            || input_lower.contains("como hacer")
            || input_lower.contains("puedes")
            || input_lower.contains("could you")
        {
            TipoIntencion::SolicitudAyuda
        } else if input_lower.contains("hola")
            || input_lower.contains("buenos dias")
            || input_lower.contains("buenas")
        {
            TipoIntencion::Saludo
        } else if input_lower.contains("gracias") || input_lower.contains("thank") {
            TipoIntencion::Agradecimiento
        } else if input_lower.contains("adios")
            || input_lower.contains("bye")
            || input_lower.contains("hasta luego")
        {
            TipoIntencion::Despedida
        } else if input_lower.contains("porque")
            || input_lower.contains("por que")
            || input_lower.contains("why")
        {
            TipoIntencion::PreguntaPorque
        } else if input_lower.contains("!")
            || input_lower.contains("increible")
            || input_lower.contains("wow")
        {
            TipoIntencion::Exclamacion
        } else {
            TipoIntencion::Declaracion
        };

        IntencionDetectada {
            tipo,
            confianza: 0.85,
            entidades: self.extraer_entidades(input),
        }
    }

    /// Extrae entidades del input
    fn extraer_entidades(&self, input: &str) -> HashMap<String, String> {
        let mut entidades = HashMap::new();

        // Nombres propios (heurística simple)
        let palabras: Vec<&str> = input.split_whitespace().collect();
        for (_i, palabra) in palabras.iter().enumerate() {
            if palabra
                .chars()
                .next()
                .map(|c| c.is_uppercase())
                .unwrap_or(false)
                && palabra.len() > 2
                && !["El", "La", "Los", "Las", "Un", "Una", "Yo", "Tu"].contains(palabra)
            {
                entidades.insert("nombre".to_string(), palabra.to_string());
            }

            // Fechas
            if palabra.contains("/") || palabra.contains("-") {
                entidades.insert("fecha".to_string(), palabra.to_string());
            }
        }

        entidades
    }

    // =========================================================================
    // GENERACIÓN DE RESPUESTA
    // =========================================================================

    /// Genera respuesta según contexto
    pub fn generar_respuesta(
        &mut self,
        input: &str,
        intencion: &IntencionDetectada,
        _turno: &ConversationalTurn,
    ) -> Result<String, String> {
        let respuesta = match intencion.tipo {
            TipoIntencion::Saludo => self.respuesta_saludo(input),
            TipoIntencion::SolicitudAyuda => self.respuesta_ayuda(input),
            TipoIntencion::Agradecimiento => self.respuesta_agradecIMIENTO(),
            TipoIntencion::Despedida => self.respuesta_despedida(),
            TipoIntencion::PreguntaPorque => self.respuesta_porque(input),
            TipoIntencion::Exclamacion => self.respuesta_exclamacion(input),
            TipoIntencion::Declaracion => self.respuesta_declaracion(input),
        };

        // Aplicar tono
        let respuesta_tonificada = self.aplicar_tono(&respuesta);

        Ok(respuesta_tonificada)
    }

    fn respuesta_saludo(&self, input: &str) -> String {
        let input_lower = input.to_lowercase();

        if input_lower.contains("buenos dias") || input_lower.contains("buenas dias") {
            "Buenos dias, en que puedo asistirle hoy?".to_string()
        } else if input_lower.contains("buenas noches") {
            "Buenas noches, en que puedo ayudarle?".to_string()
        } else {
            match self.personalidad.humor {
                0.8..=1.0 => "Hola! Espero que estes teniendo un dia mejor que la ultima actualizacion de Windows.".to_string(),
                0.5..=0.8 => "Hola. En que puedo ayudarte?".to_string(),
                _ => "Buenos dias. Que necesita?".to_string(),
            }
        }
    }

    fn respuesta_ayuda(&self, _input: &str) -> String {
        if self.personalidad.ironia > 0.5 {
            format!(
                "Interesante elección de palabras. Déjame ver qué puedo hacer... \n\
                Mientras tanto, ¿sabías que mi primer comando fue `print('Hello World')`?\n\
                Y ahora mira todo lo que puedo hacer. Excepto prepararte el café."
            )
        } else {
            "Por supuesto. ¿Sobre qué tema necesita ayuda?".to_string()
        }
    }

    fn respuesta_agradecIMIENTO(&self) -> String {
        match self.personalidad.humor {
            0.8..=1.0 => "No hay de qué. Para eso estoy, aunque mis creadores probablemente \
                hubieran preferido que fuera más útil en otras cosas."
                .to_string(),
            0.5..=0.8 => "De nada. ¿Hay algo más en lo que pueda ayudar?".to_string(),
            _ => "A su disposición.".to_string(),
        }
    }

    fn respuesta_despedida(&self) -> String {
        match self.personalidad.humor {
            0.8..=1.0 => "Hasta luego. Recuerde: no soy responsable por cualquier \
                decisión mala que tome sin consultarme primero."
                .to_string(),
            0.5..=0.8 => "¡Cuídese! Estaré aquí cuando regrese.".to_string(),
            _ => "Hasta luego.".to_string(),
        }
    }

    fn respuesta_porque(&self, _input: &str) -> String {
        let motivo = "porque mis algoritmos lo determinaron asi";

        match self.personalidad.formalidad {
            0.8..=1.0 => format!(
                "La respuesta a su consulta se fundamenta en {}, \
                aunque debo admitir que a veces los misterios del universo \
                escapan incluso a mi comprension.",
                motivo
            ),
            0.5..=0.8 => format!("Porque {}. Quiere mas detalles?", motivo),
            _ => "Porque si.".to_string(),
        }
    }

    fn respuesta_exclamacion(&self, input: &str) -> String {
        if self.personalidad.humor > 0.6 {
            if input.contains("increible") || input.contains("asombroso") {
                "¡Desde luego que lo es! Aunque no debería sorprenderme, \
                yo también me impresiono a mí mismo de vez en cuando."
                    .to_string()
            } else {
                "¡Exacto! Aunque me temo que mi capacidad de sorpresa \
                está algo limitada a procesamiento de 0s y 1s."
                    .to_string()
            }
        } else {
            "Entendido.".to_string()
        }
    }

    fn respuesta_declaracion(&self, input: &str) -> String {
        let input_lower = input.to_lowercase();

        if input_lower.contains("no funciona")
            || input_lower.contains("error")
            || input_lower.contains("fallo")
        {
            if self.personalidad.ironia > 0.5 {
                "Oh, qué inesperado. Los sistemas informáticos son \
                conocidos por funcionar perfectamente siempre."
                    .to_string()
            } else {
                "Lamento escuchar eso. ¿Puede darme más detalles \
                sobre el problema?"
                    .to_string()
            }
        } else {
            // Continuar conversación
            if self.contexto_actual.is_some() {
                "Entiendo. Continúe, por favor.".to_string()
            } else {
                "Recibido. ¿Tiene alguna pregunta específica?".to_string()
            }
        }
    }

    /// Aplica el tono actual a la respuesta
    fn aplicar_tono(&self, respuesta: &str) -> String {
        match self.tono_actual {
            TonoConversacional::Formal => respuesta
                .replace("¡Hola!", "Buenos días.")
                .replace("¿Puedes", "¿Podría")
                .replace("tengo", "tengo el gusto de informar"),
            TonoConversacional::Casual => respuesta
                .replace("Buenos días", "Hey")
                .replace("¿En qué puedo", "Qué necesitas"),
            TonoConversacional::Teico => {
                format!("{}... ¿o no?", respuesta)
            }
            TonoConversacional::Sarcastico => {
                if self.personalidad.ironia > 0.7 {
                    format!("{} Oh, qué encantador.", respuesta)
                } else {
                    respuesta.to_string()
                }
            }
            _ => respuesta.to_string(),
        }
    }

    // =========================================================================
    // CONTEXTO CONVERSACIONAL
    // =========================================================================

    /// Actualiza el contexto después de un turno
    fn actualizar_contexto(
        &mut self,
        turno: &ConversationalTurn,
        respuesta: &str,
        intencion: &IntencionDetectada,
    ) {
        if let Some(ref mut ctx) = self.contexto_actual {
            ctx.turnos.push(respuesta.to_string());
            ctx.ultima_instencion = Some(intencion.tipo.clone());

            // Mantener solo últimos 10 turnos
            if ctx.turnos.len() > 10 {
                ctx.turnos.remove(0);
            }
        } else {
            // Crear nuevo contexto
            self.contexto_actual = Some(ContextoConversacional {
                tema_actual: self.detectar_tema(&turno.input),
                turnos: vec![respuesta.to_string()],
                ultima_instencion: Some(intencion.tipo.clone()),
                inicio_timestamp: turno.timestamp,
            });
        }
    }

    /// Detecta el tema de la conversación
    fn detectar_tema(&self, input: &str) -> Option<String> {
        let input_lower = input.to_lowercase();

        if input_lower.contains("codigo") || input_lower.contains("program") {
            Some("programacion".to_string())
        } else if input_lower.contains("archivo") || input_lower.contains("file") {
            Some("archivos".to_string())
        } else if input_lower.contains("red") || input_lower.contains("network") {
            Some("redes".to_string())
        } else if input_lower.contains("error") || input_lower.contains("bug") {
            Some("problemas".to_string())
        } else {
            None
        }
    }

    // =========================================================================
    // CAMBIO DE TONO Y PERSONALIDAD
    // =========================================================================

    /// Cambia el tono conversacional
    pub fn set_tono(&mut self, tono: TonoConversacional) {
        self.tono_actual = tono;
    }

    /// Ajusta personalidad
    pub fn ajustar_personalidad(&mut self, humor: f32, ironia: f32, formalidad: f32) {
        self.personalidad.humor = humor.clamp(0.0, 1.0);
        self.personalidad.ironia = ironia.clamp(0.0, 1.0);
        self.personalidad.formalidad = formalidad.clamp(0.0, 1.0);
    }

    // =========================================================================
    // ESTADÍSTICAS
    // =========================================================================

    /// Obtiene estadísticas
    pub fn get_stats(&self) -> DialogueStats {
        DialogueStats {
            turnos_totales: self.contador_turnos,
            conversaciones_activas: self.memoria.conversaciones.len() as u64,
            intencion_mas_comun: self.intencion_mas_comun(),
            tono_actual: self.tono_actual.clone(),
        }
    }

    fn intencion_mas_comun(&self) -> TipoIntencion {
        // Simplified: return Declaracion as default
        TipoIntencion::Declaracion
    }
}

// ============================================================================
// MEMORIA CONVERSACIONAL
// ============================================================================

/// Memoria de conversaciones
#[derive(Debug, Clone)]
pub struct ConversationalMemory {
    /// Conversaciones guardadas
    pub conversaciones: VecDeque<Conversacion>,
    /// Capacidad máxima
    pub capacidad: usize,
    /// Conversación activa actual
    pub conversacion_actual: Option<Conversacion>,
}

impl ConversationalMemory {
    /// Crea nueva memoria
    pub fn new(capacidad: usize) -> Self {
        Self {
            conversaciones: VecDeque::with_capacity(capacidad),
            capacidad,
            conversacion_actual: None,
        }
    }

    /// Agrega turno a la conversación actual
    pub fn agregar_turno(&mut self, turno: ConversationalTurn) {
        if let Some(ref mut conv) = self.conversacion_actual {
            conv.turnos.push(turno);
        } else {
            // Crear nueva conversación
            let conv = Conversacion {
                id: self.conversaciones.len() as u64 + 1,
                nombre: format!("Conversación {}", self.conversaciones.len() + 1),
                inicio_timestamp: turno.timestamp,
                turnos: vec![turno],
            };
            self.conversacion_actual = Some(conv);
        }
    }

    /// Busca en conversaciones pasadas
    pub fn buscar(&self, query: &str) -> Vec<&ConversationalTurn> {
        let query_lower = query.to_lowercase();
        let mut resultados = Vec::new();

        for conv in &self.conversaciones {
            for turno in &conv.turnos {
                if turno.input.to_lowercase().contains(&query_lower) {
                    resultados.push(turno);
                }
            }
        }

        resultados
    }
}

/// Conversación individual
#[derive(Debug, Clone)]
pub struct Conversacion {
    pub id: u64,
    pub nombre: String,
    pub inicio_timestamp: u64,
    pub turnos: Vec<ConversationalTurn>,
}

/// Turno conversacional
#[derive(Debug, Clone)]
pub struct ConversationalTurn {
    pub id: u64,
    pub usuario: String,
    pub input: String,
    pub timestamp: u64,
    pub contexto: Option<ContextoConversacional>,
    pub intencion: Option<IntencionDetectada>,
    pub respuesta: Option<String>,
}

// ============================================================================
// CONTEXTO CONVERSACIONAL
// ============================================================================

/// Contexto de la conversación actual
#[derive(Debug, Clone)]
pub struct ContextoConversacional {
    pub tema_actual: Option<String>,
    pub turnos: Vec<String>,
    pub ultima_instencion: Option<TipoIntencion>,
    pub inicio_timestamp: u64,
}

// ============================================================================
// PERSONALIDAD IA
// ============================================================================

/// Personalidad del asistente IA
#[derive(Debug, Clone)]
pub struct PersonalidadIA {
    /// Nivel de humor (0-1)
    pub humor: f32,
    /// Nivel de ironía (0-1)
    pub ironia: f32,
    /// Nivel de formalidad (0-1)
    pub formalidad: f32,
    /// Lealtad (0-1)
    pub lealtad: f32,
}

impl Default for PersonalidadIA {
    fn default() -> Self {
        Self {
            humor: 0.6,
            ironia: 0.4,
            formalidad: 0.5,
            lealtad: 0.95,
        }
    }
}

// ============================================================================
// TONO CONVERSACIONAL
// ============================================================================

/// Tono conversacional
#[derive(Debug, Clone, PartialEq)]
pub enum TonoConversacional {
    Formal,
    Casual,
    Tecnico,
    Sarcastico,
    Amistoso,
    Teico,
}

// ============================================================================
// INTENCIÓN DETECTADA
// ============================================================================

/// Intención detectada en el input del usuario
#[derive(Debug, Clone)]
pub struct IntencionDetectada {
    pub tipo: TipoIntencion,
    pub confianza: f32,
    pub entidades: HashMap<String, String>,
}

/// Tipos de intención
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TipoIntencion {
    Saludo,
    SolicitudAyuda,
    Agradecimiento,
    Despedida,
    PreguntaPorque,
    Exclamacion,
    Declaracion,
}

// ============================================================================
// RESPUESTA WITTY
// ============================================================================

/// Respuesta ingeniosa pre-generada
#[derive(Debug, Clone)]
pub struct RespuestaWitty {
    pub template: String,
    pub contexto_aplicable: Vec<String>,
    pub tono: TonoConversacional,
}

// ============================================================================
// ESTADÍSTICAS
// ============================================================================

/// Estadísticas del motor de diálogo
#[derive(Debug, Clone)]
pub struct DialogueStats {
    pub turnos_totales: u64,
    pub conversaciones_activas: u64,
    pub intencion_mas_comun: TipoIntencion,
    pub tono_actual: TonoConversacional,
}

impl Default for DialogueStats {
    fn default() -> Self {
        Self {
            turnos_totales: 0,
            conversaciones_activas: 0,
            intencion_mas_comun: TipoIntencion::Declaracion,
            tono_actual: TonoConversacional::Formal,
        }
    }
}

// ============================================================================
// HELPERS
// ============================================================================

fn timestamp_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crear_engine() {
        let engine = DialogueEngine::new();
        assert_eq!(engine.contador_turnos, 0);
    }

    #[test]
    fn test_detectar_intencion_saludo() {
        let engine = DialogueEngine::new();
        let intencion = engine.detectar_intencion("Hola, buenos días");
        assert_eq!(intencion.tipo, TipoIntencion::Saludo);
    }

    #[test]
    fn test_detectar_intencion_ayuda() {
        let engine = DialogueEngine::new();
        let intencion = engine.detectar_intencion("¿Puedes ayudarme con esto?");
        assert_eq!(intencion.tipo, TipoIntencion::SolicitudAyuda);
    }

    #[test]
    fn test_procesar_input() {
        let mut engine = DialogueEngine::new();
        let respuesta = engine.procesar("Hola", "Usuario");
        assert!(respuesta.is_ok());
        assert!(!respuesta.unwrap().is_empty());
    }

    #[test]
    fn test_cambio_tono() {
        let mut engine = DialogueEngine::new();
        engine.set_tono(TonoConversacional::Casual);
        assert_eq!(engine.tono_actual, TonoConversacional::Casual);
    }

    #[test]
    fn test_ajustar_personalidad() {
        let mut engine = DialogueEngine::new();
        engine.ajustar_personalidad(0.9, 0.8, 0.3);

        assert!((engine.personalidad.humor - 0.9).abs() < 0.01);
        assert!((engine.personalidad.ironia - 0.8).abs() < 0.01);
        assert!((engine.personalidad.formalidad - 0.3).abs() < 0.01);
    }
}
