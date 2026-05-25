//! # Conversation - Conversation Management
//!
//! Gestión de conversaciones multi-turno.
//! 100% original, sin dependencias.
#![allow(dead_code)]
#![allow(non_snake_case)]

use crate::theory_of_mind::AgentId;
use crate::verbal_comm::prosody::EmotionalStyle;
use std::collections::{HashMap, VecDeque};

/// Estado de conversación
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ConversationState {
    /// Esperando input
    Listening,
    /// Procesando
    Processing,
    /// Generando respuesta
    Speaking,
    /// Pausado
    Paused,
    /// Finalizado
    Finished,
}

/// Turno en conversación
#[derive(Clone, Debug)]
pub struct ConversationTurn {
    /// Quién habla
    pub speaker: AgentId,
    /// Contenido del turno
    pub content: String,
    /// Timestamp
    pub timestamp: u64,
    /// Prosodia usada
    pub prosody: Option<EmotionalStyle>,
    /// Intención detectada
    pub detected_intent: Option<String>,
}

/// Gestor de conversaciones multi-turno
pub struct ConversationManager {
    /// turns history
    turns: VecDeque<ConversationTurn>,
    /// Estado actual
    state: ConversationState,
    /// Máximo de turnos en historial
    max_history: usize,
    /// Tema actual
    current_topic: Option<String>,
    /// Contexto compartido
    shared_context: HashMap<String, String>,
    /// Último intent
    last_intent: Option<String>,
}

impl ConversationManager {
    pub fn new() -> Self {
        Self {
            turns: VecDeque::new(),
            state: ConversationState::Listening,
            max_history: 50,
            current_topic: None,
            shared_context: HashMap::new(),
            last_intent: None,
        }
    }

    /// Registra un turno
    pub fn add_turn(&mut self, speaker: AgentId, content: String, prosody: Option<EmotionalStyle>) {
        let turn = ConversationTurn {
            speaker,
            content,
            timestamp: 0,
            prosody,
            detected_intent: None,
        };

        self.turns.push_back(turn);

        // Trim history
        while self.turns.len() > self.max_history {
            self.turns.pop_front();
        }
    }

    /// Obtiene últimos N turnos
    pub fn get_recent_turns(&self, n: usize) -> Vec<&ConversationTurn> {
        self.turns.iter().rev().take(n).collect()
    }

    /// Obtiene historial de conversación
    pub fn get_conversation_history(&self) -> String {
        let mut history = String::new();

        for turn in &self.turns {
            let speaker_str = if turn.speaker == 0 { "User" } else { "Agent" };
            history.push_str(&format!("{}: {}\n", speaker_str, turn.content));
        }

        history
    }

    /// Detecta tema de conversación
    pub fn detect_topic(&self) -> Option<String> {
        let recent = self.get_recent_turns(5);
        let mut topic_keywords = std::collections::HashMap::new();

        for turn in recent {
            for word in turn.content.to_lowercase().split_whitespace() {
                if word.len() > 4 {
                    *topic_keywords.entry(word.to_string()).or_insert(0) += 1;
                }
            }
        }

        topic_keywords
            .into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(topic, _)| topic)
    }

    /// Detecta intención del usuario
    pub fn detect_intent(&self, text: &str) -> Option<String> {
        let text_lower = text.to_lowercase();

        let intents = [
            ("greeting", vec!["hello", "hi", "hey", "greetings"]),
            (
                "question",
                vec!["what", "how", "why", "when", "where", "who", "which"],
            ),
            (
                "request",
                vec!["please", "can", "could", "would", "will", "help"],
            ),
            ("farewell", vec!["bye", "goodbye", "see you", "later"]),
            (
                "affirmation",
                vec!["yes", "yeah", "yep", "correct", "right"],
            ),
            ("negation", vec!["no", "nope", "not", "never"]),
            ("thanks", vec!["thank", "thanks", "appreciate"]),
        ];

        for (intent_name, keywords) in intents {
            for keyword in keywords {
                if text_lower.contains(keyword) {
                    return Some(intent_name.to_string());
                }
            }
        }

        None
    }

    /// Actualiza estado
    pub fn set_state(&mut self, state: ConversationState) {
        self.state = state;
    }

    /// Obtiene estado
    pub fn get_state(&self) -> ConversationState {
        self.state.clone()
    }

    /// Añade al contexto compartido
    pub fn add_context(&mut self, key: &str, value: &str) {
        self.shared_context
            .insert(key.to_string(), value.to_string());
    }

    /// Obtiene del contexto compartido
    pub fn get_context(&self, key: &str) -> Option<&String> {
        self.shared_context.get(key)
    }

    /// Obtiene último intent
    pub fn get_last_intent(&self) -> Option<&String> {
        self.last_intent.as_ref()
    }
}

impl Default for ConversationManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Genera respuesta conversacional simple
pub fn generate_conversational_response(
    manager: &mut ConversationManager,
    user_input: &str,
) -> String {
    let intent = manager.detect_intent(user_input);
    manager.last_intent = intent.clone();

    match intent.as_deref() {
        Some("greeting") => "Hello! How can I help you today?".to_string(),
        Some("question") => "That's an interesting question. Let me think about it.".to_string(),
        Some("request") => "I'll do my best to help you with that.".to_string(),
        Some("farewell") => "Goodbye! It was nice talking with you.".to_string(),
        Some("affirmation") => "I'm glad you agree.".to_string(),
        Some("negation") => "I understand. Let me try a different approach.".to_string(),
        Some("thanks") => "You're welcome! Is there anything else I can help with?".to_string(),
        _ => "I see. Could you tell me more about that?".to_string(),
    }
}
