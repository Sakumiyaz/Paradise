//! # Generator - Speech Generation
//!
//! Generador de lenguaje hablado.
//! 100% original, sin dependencias.
#![allow(dead_code)]
#![allow(non_snake_case)]

use crate::verbal_comm::prosody::{EmotionalStyle, ProsodyModel};
use crate::verbal_comm::synthesis::VoiceSynthesis;
use std::collections::HashMap;

/// Generador de lenguaje hablado
pub struct SpeechGenerator {
    /// Vocabulario
    vocabulary: HashMap<String, Vec<(&'static str, f32)>>,
    /// Síntesis de voz
    synthesizer: VoiceSynthesis,
    /// Prosodia actual
    prosody: ProsodyModel,
    /// Buffer de salida
    output_buffer: Vec<f32>,
}

impl SpeechGenerator {
    pub fn new() -> Self {
        let mut generator = Self {
            vocabulary: HashMap::new(),
            synthesizer: VoiceSynthesis::new(),
            prosody: ProsodyModel::new(),
            output_buffer: Vec::new(),
        };
        generator.init_vocabulary();
        generator
    }

    fn init_vocabulary(&mut self) {
        // Vocabulario simplificado: palabra -> [(fonema, duración), ...]
        // Duraciones en milisegundos

        self.vocabulary.insert(
            "hello".to_string(),
            vec![
                ("h", 50.0),
                ("e", 80.0),
                ("l", 40.0),
                ("l", 40.0),
                ("o", 100.0),
            ],
        );

        self.vocabulary.insert(
            "world".to_string(),
            vec![
                ("w", 60.0),
                ("o", 80.0),
                ("r", 50.0),
                ("l", 40.0),
                ("d", 70.0),
            ],
        );

        self.vocabulary.insert(
            "yes".to_string(),
            vec![("y", 40.0), ("e", 80.0), ("s", 100.0)],
        );

        self.vocabulary
            .insert("no".to_string(), vec![("n", 60.0), ("o", 120.0)]);

        self.vocabulary.insert(
            "please".to_string(),
            vec![("p", 40.0), ("l", 30.0), ("i", 60.0), ("s", 80.0)],
        );

        self.vocabulary.insert(
            "thank".to_string(),
            vec![("th", 60.0), ("a", 80.0), ("n", 50.0), ("k", 50.0)],
        );

        self.vocabulary.insert(
            "thanks".to_string(),
            vec![
                ("th", 60.0),
                ("a", 80.0),
                ("n", 50.0),
                ("k", 50.0),
                ("s", 40.0),
            ],
        );

        self.vocabulary.insert(
            "you".to_string(),
            vec![("y", 40.0), ("o", 80.0), ("u", 100.0)],
        );

        self.vocabulary.insert(
            "how".to_string(),
            vec![("h", 50.0), ("o", 80.0), ("w", 60.0)],
        );

        self.vocabulary
            .insert("are".to_string(), vec![("a", 60.0), ("r", 40.0)]);

        self.vocabulary.insert(
            "what".to_string(),
            vec![("w", 50.0), ("a", 80.0), ("t", 60.0)],
        );

        self.vocabulary
            .insert("is".to_string(), vec![("i", 60.0), ("s", 80.0)]);

        self.vocabulary
            .insert("the".to_string(), vec![("th", 50.0), ("e", 60.0)]);

        self.vocabulary.insert(
            "time".to_string(),
            vec![("t", 40.0), ("a", 60.0), ("m", 50.0), ("e", 80.0)],
        );

        self.vocabulary.insert(
            "goodbye".to_string(),
            vec![
                ("g", 40.0),
                ("o", 60.0),
                ("o", 40.0),
                ("d", 30.0),
                ("b", 40.0),
                ("y", 60.0),
                ("e", 80.0),
            ],
        );

        self.vocabulary.insert(
            "bye".to_string(),
            vec![("b", 40.0), ("y", 60.0), ("e", 80.0)],
        );

        self.vocabulary
            .insert("hi".to_string(), vec![("h", 50.0), ("i", 80.0)]);

        self.vocabulary.insert(
            "hey".to_string(),
            vec![("h", 50.0), ("e", 70.0), ("y", 60.0)],
        );

        self.vocabulary.insert(
            "can".to_string(),
            vec![("k", 40.0), ("a", 60.0), ("n", 60.0)],
        );

        self.vocabulary.insert(
            "could".to_string(),
            vec![("k", 40.0), ("o", 60.0), ("u", 50.0), ("d", 60.0)],
        );

        self.vocabulary.insert(
            "help".to_string(),
            vec![("h", 50.0), ("e", 70.0), ("l", 40.0), ("p", 50.0)],
        );

        self.vocabulary
            .insert("i".to_string(), vec![("a", 60.0), ("y", 40.0)]);

        self.vocabulary
            .insert("am".to_string(), vec![("a", 60.0), ("m", 60.0)]);

        self.vocabulary.insert(
            "fine".to_string(),
            vec![
                ("f", 50.0),
                ("a", 60.0),
                ("y", 40.0),
                ("n", 60.0),
                ("e", 80.0),
            ],
        );

        self.vocabulary.insert(
            "nice".to_string(),
            vec![
                ("n", 50.0),
                ("a", 60.0),
                ("y", 40.0),
                ("s", 60.0),
                ("e", 80.0),
            ],
        );

        self.vocabulary
            .insert("to".to_string(), vec![("t", 40.0), ("o", 80.0)]);

        self.vocabulary.insert(
            "meet".to_string(),
            vec![("m", 50.0), ("e", 70.0), ("e", 50.0), ("t", 60.0)],
        );

        self.vocabulary.insert(
            "meet you".to_string(),
            vec![
                ("m", 50.0),
                ("e", 70.0),
                ("e", 50.0),
                ("t", 60.0),
                ("y", 40.0),
                ("o", 80.0),
                ("u", 100.0),
            ],
        );

        self.vocabulary.insert(
            "too".to_string(),
            vec![("t", 40.0), ("o", 80.0), ("o", 60.0)],
        );

        self.vocabulary.insert(
            "welcome".to_string(),
            vec![
                ("w", 50.0),
                ("e", 70.0),
                ("l", 40.0),
                ("k", 50.0),
                ("a", 60.0),
                ("m", 60.0),
                ("e", 80.0),
            ],
        );

        self.vocabulary.insert(
            "sorry".to_string(),
            vec![
                ("s", 50.0),
                ("o", 70.0),
                ("r", 50.0),
                ("r", 40.0),
                ("y", 80.0),
            ],
        );

        self.vocabulary.insert(
            "understand".to_string(),
            vec![
                ("a", 50.0),
                ("n", 50.0),
                ("d", 40.0),
                ("e", 60.0),
                ("r", 50.0),
                ("s", 60.0),
                ("t", 50.0),
                ("a", 60.0),
                ("n", 50.0),
                ("d", 60.0),
            ],
        );

        self.vocabulary.insert(
            "know".to_string(),
            vec![("k", 50.0), ("n", 60.0), ("o", 70.0), ("w", 50.0)],
        );

        self.vocabulary.insert(
            "think".to_string(),
            vec![("th", 60.0), ("i", 60.0), ("n", 50.0), ("k", 60.0)],
        );

        self.vocabulary.insert(
            "try".to_string(),
            vec![("t", 40.0), ("r", 60.0), ("y", 80.0)],
        );

        self.vocabulary.insert(
            "different".to_string(),
            vec![
                ("d", 40.0),
                ("i", 60.0),
                ("f", 40.0),
                ("f", 40.0),
                ("e", 60.0),
                ("r", 50.0),
                ("e", 60.0),
                ("n", 50.0),
                ("t", 50.0),
            ],
        );

        self.vocabulary.insert(
            "approach".to_string(),
            vec![
                ("a", 50.0),
                ("p", 40.0),
                ("p", 40.0),
                ("r", 50.0),
                ("o", 60.0),
                ("a", 50.0),
                ("ch", 70.0),
            ],
        );

        self.vocabulary.insert(
            "glad".to_string(),
            vec![("g", 40.0), ("l", 40.0), ("a", 60.0), ("d", 70.0)],
        );

        self.vocabulary.insert(
            "agree".to_string(),
            vec![("a", 50.0), ("g", 40.0), ("r", 60.0), ("e", 80.0)],
        );

        self.vocabulary.insert(
            "interesting".to_string(),
            vec![
                ("i", 50.0),
                ("n", 40.0),
                ("t", 40.0),
                ("e", 60.0),
                ("r", 50.0),
                ("e", 60.0),
                ("s", 60.0),
                ("t", 50.0),
                ("i", 50.0),
                ("n", 50.0),
                ("g", 70.0),
            ],
        );

        self.vocabulary.insert(
            "that".to_string(),
            vec![("th", 50.0), ("a", 60.0), ("t", 60.0)],
        );

        self.vocabulary.insert(
            "let".to_string(),
            vec![("l", 40.0), ("e", 60.0), ("t", 60.0)],
        );

        self.vocabulary
            .insert("me".to_string(), vec![("m", 50.0), ("e", 80.0)]);

        self.vocabulary.insert(
            "about".to_string(),
            vec![
                ("a", 50.0),
                ("b", 40.0),
                ("o", 60.0),
                ("u", 70.0),
                ("t", 50.0),
            ],
        );

        self.vocabulary.insert(
            "more".to_string(),
            vec![("m", 50.0), ("o", 70.0), ("r", 50.0), ("e", 80.0)],
        );
    }

    /// Genera habla desde texto
    pub fn generate_speech(&mut self, text: &str, style: EmotionalStyle) -> Vec<f32> {
        self.prosody = ProsodyModel::new().with_style(style);
        self.synthesizer.clear();
        self.output_buffer.clear();

        let words: Vec<&str> = text.split_whitespace().collect();

        for word in words {
            let clean_word = word.trim().to_lowercase();
            if let Some(phonemes) = self.vocabulary.get(&clean_word) {
                // Apply prosody modifications
                let prosodic_phonemes: Vec<(&str, f32)> = phonemes
                    .iter()
                    .map(|(p, d)| (*p, d * (150.0 / self.prosody.speech_rate)))
                    .collect();

                let audio = self.synthesizer.synthesize_word(&prosodic_phonemes);
                self.output_buffer.extend(audio);

                // Add prosodic pauses
                if self.prosody.pause_frequency > 0.3 {
                    let pause_samples =
                        (self.prosody.syllable_duration * 0.5 * self.prosody.speech_rate / 60000.0)
                            as usize;
                    self.output_buffer
                        .extend(vec![0.0f32; pause_samples.max(100)]);
                }
            }
        }

        self.output_buffer.clone()
    }

    /// Genera respuesta a partir de template
    pub fn generate_response(
        &mut self,
        template: &str,
        params: &[String],
        style: EmotionalStyle,
    ) -> Vec<f32> {
        let mut text = template.to_string();

        for (i, param) in params.iter().enumerate() {
            text = text.replace(&format!("{{{}}}", i), param);
        }

        self.generate_speech(&text, style)
    }

    /// Obtiene buffer de audio
    pub fn get_audio(&self) -> &[f32] {
        &self.output_buffer
    }
}

impl Default for SpeechGenerator {
    fn default() -> Self {
        Self::new()
    }
}
