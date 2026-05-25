//! # Prosody - Prosody Model
//!
//! Modelo de prosodia para síntesis de voz.
//! 100% original, sin dependencias.
#![allow(dead_code)]
#![allow(non_snake_case)]

/// Modelo de prosodia para síntesis de voz
#[derive(Clone, Debug)]
pub struct ProsodyModel {
    /// Pitch base en Hz
    pub base_pitch: f32,
    /// Duración de sílabas (ms)
    pub syllable_duration: f32,
    /// Intensidad [0.0 - 1.0]
    pub intensity: f32,
    /// Pitch range (desviación máxima)
    pub pitch_range: f32,
    /// Velocidad de habla (palabras/min)
    pub speech_rate: f32,
    /// Frecuencia de pausas
    pub pause_frequency: f32,
    /// Pitch contour (puntos de inflexión)
    pub pitch_contour: Vec<f32>,
}

impl ProsodyModel {
    pub fn new() -> Self {
        Self {
            base_pitch: 150.0,        // Hz - voice típica
            syllable_duration: 250.0, // ms
            intensity: 0.8,
            pitch_range: 50.0,
            speech_rate: 150.0, // wpm
            pause_frequency: 0.2,
            pitch_contour: Vec::new(),
        }
    }

    /// Crea prosodia expresiva
    pub fn with_style(mut self, style: EmotionalStyle) -> Self {
        match style {
            EmotionalStyle::Calm => {
                self.speech_rate = 120.0;
                self.pitch_range = 30.0;
                self.intensity = 0.6;
            }
            EmotionalStyle::Urgent => {
                self.speech_rate = 180.0;
                self.pitch_range = 70.0;
                self.intensity = 0.9;
            }
            EmotionalStyle::Empathetic => {
                self.speech_rate = 130.0;
                self.pitch_range = 40.0;
                self.intensity = 0.7;
            }
            EmotionalStyle::Authoritative => {
                self.speech_rate = 140.0;
                self.pitch_range = 60.0;
                self.intensity = 0.85;
            }
            EmotionalStyle::Friendly => {
                self.speech_rate = 150.0;
                self.pitch_range = 45.0;
                self.intensity = 0.75;
            }
            EmotionalStyle::Thoughtful => {
                self.speech_rate = 110.0;
                self.pitch_range = 35.0;
                self.intensity = 0.65;
            }
        }
        self
    }

    /// Calcula pitch para una sílaba
    pub fn calculate_pitch(&self, syllable_index: usize) -> f32 {
        let contour_idx = syllable_index % self.pitch_contour.len().max(1);
        let contour_value = self.pitch_contour.get(contour_idx).unwrap_or(&0.5);

        let base = self.base_pitch;
        let variation = (contour_value - 0.5) * 2.0 * self.pitch_range;

        base + variation
    }

    /// Calcula duración de una palabra
    pub fn calculate_word_duration(&self, syllables: usize) -> f32 {
        let syllable_time = 60000.0 / self.speech_rate;
        syllable_time * syllables as f32
    }

    /// Añade punto al pitch contour
    pub fn add_contour_point(&mut self, value: f32) {
        self.pitch_contour.push(value);
    }
}

/// Estilo emocional de habla
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum EmotionalStyle {
    Calm,
    Urgent,
    Empathetic,
    Authoritative,
    Friendly,
    Thoughtful,
}

impl Default for ProsodyModel {
    fn default() -> Self {
        Self::new()
    }
}
