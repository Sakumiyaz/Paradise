//! # Vocoder - Audio/Text converter interface
//!
//! Interfaz unificada entre texto y audio.
//! Sin dependencias externas - 100% Rust.

#![allow(dead_code)]

use super::{AudioBuffer, ProsodyGenerator, Recognizer, Synthesizer};
use crate::voice::prosody::SentenceType;

/// Tipo de conversión
#[derive(Debug, Clone, Copy)]
pub enum VocoderMode {
    TTS, // Text to Speech
    STT, // Speech to Text
}

/// Vocoder principal
pub struct Vocoder {
    synthesizer: Synthesizer,
    recognizer: Recognizer,
    prosody: ProsodyGenerator,
    mode: VocoderMode,
}

impl Vocoder {
    pub fn new() -> Self {
        Self {
            synthesizer: Synthesizer::new(),
            recognizer: Recognizer::new(),
            prosody: ProsodyGenerator::new(),
            mode: VocoderMode::TTS,
        }
    }

    /// Convierte texto a audio
    pub fn text_to_speech(&self, text: &str) -> AudioBuffer {
        self.synthesizer.speak(text)
    }

    /// Convierte texto a audio con entonación específica
    pub fn text_to_speech_with_intonation(
        &self,
        text: &str,
        _sentence_type: SentenceType,
    ) -> AudioBuffer {
        let audio = self.synthesizer.speak(text);
        // La prosodia ya está aplicada en el synthesizer
        audio
    }

    /// Convierte audio a texto
    pub fn speech_to_text(
        &self,
        audio: &AudioBuffer,
    ) -> Result<String, super::recognizer::RecognizerError> {
        self.recognizer.recognize(audio)
    }

    /// Convierte audio a texto con confirmación
    pub fn speech_to_text_with_confirm(
        &self,
        audio: &AudioBuffer,
        expected: &str,
    ) -> Result<SpeechResult, super::recognizer::RecognizerError> {
        let text = self.speech_to_text(audio)?;

        let confidence = self.calculate_confidence(&text, expected);

        Ok(SpeechResult {
            text,
            confidence,
            matches_expected: confidence > 0.7,
        })
    }

    fn calculate_confidence(&self, recognized: &str, expected: &str) -> f32 {
        let recognized_lower = recognized.to_lowercase();
        let expected_lower = expected.to_lowercase();

        if recognized_lower == expected_lower {
            1.0
        } else if recognized_lower.is_empty() || expected_lower.is_empty() {
            0.0
        } else {
            // Similitud básica
            let matching: usize = recognized_lower
                .chars()
                .zip(expected_lower.chars())
                .filter(|(a, b)| a == b)
                .count();

            matching as f32 / expected_lower.len().max(recognized_lower.len()) as f32
        }
    }

    /// Alterna entre TTS y STT
    pub fn set_mode(&mut self, mode: VocoderMode) {
        self.mode = mode;
    }

    /// Proceso de diálogo completo
    pub fn dialogue(
        &self,
        input: &str,
        is_speech_input: bool,
    ) -> Result<DialogueResult, VocoderError> {
        if is_speech_input {
            // STT: audio -> texto
            todo!("STT from audio buffer requires audio input");
            /*
            let audio = AudioBuffer::from_bytes(input.as_bytes())?;
            let text = self.speech_to_text(&audio)?;
            Ok(DialogueResult {
                input_type: InputType::Speech,
                output_type: OutputType::Text,
                output: text,
            })
            */
        } else {
            // TTS: texto -> audio
            let audio = self.text_to_speech(input);
            Ok(DialogueResult {
                input_type: InputType::Text,
                output_type: OutputType::Speech,
                audio_output: Some(audio),
                text_output: None,
            })
        }
    }
}

#[derive(Debug, Clone)]
pub struct SpeechResult {
    pub text: String,
    pub confidence: f32,
    pub matches_expected: bool,
}

#[derive(Debug, Clone)]
pub struct DialogueResult {
    pub input_type: InputType,
    pub output_type: OutputType,
    pub audio_output: Option<AudioBuffer>,
    pub text_output: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum InputType {
    Text,
    Speech,
}

#[derive(Debug, Clone, Copy)]
pub enum OutputType {
    Text,
    Speech,
}

#[derive(Debug)]
pub enum VocoderError {
    UnsupportedFormat,
    ProcessingFailed,
}

impl std::fmt::Display for VocoderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VocoderError::UnsupportedFormat => write!(f, "Formato no soportado"),
            VocoderError::ProcessingFailed => write!(f, "Procesamiento falló"),
        }
    }
}

impl Default for Vocoder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tts() {
        let vocoder = Vocoder::new();
        let audio = vocoder.text_to_speech("hello world");
        assert!(audio.len() > 0);
    }

    #[test]
    fn test_confidence() {
        let vocoder = Vocoder::new();
        let conf = vocoder.calculate_confidence("hello", "hello");
        assert!(conf > 0.99);
    }
}
