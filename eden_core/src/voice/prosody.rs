//! # Prosody Generator
//!
//! Genera patrones de entonación y ritmo natural.
//! Sin dependencias externas - 100% Rust.

#![allow(dead_code)]

use super::{AudioBuffer, Phoneme, SAMPLE_RATE};

/// Direction del pitch
#[derive(Debug, Clone, Copy)]
pub enum PitchDirection {
    Fall, // Caída (frases declarativas)
    Rise, // Subida (preguntas)
    Flat, // Neutral
    Custom(f32),
}

/// Información de entonación para una palabra
#[derive(Debug, Clone)]
pub struct Intonation {
    pub start_freq: f32,
    pub peak_freq: f32,
    pub end_freq: f32,
    pub peak_position: f32, // 0.0 - 1.0
}

impl Intonation {
    pub fn declarative() -> Self {
        // Caída gradual
        Self {
            start_freq: 180.0,
            peak_freq: 200.0,
            end_freq: 120.0,
            peak_position: 0.3,
        }
    }

    pub fn interrogative() -> Self {
        // Subida al final
        Self {
            start_freq: 160.0,
            peak_freq: 190.0,
            end_freq: 250.0,
            peak_position: 0.8,
        }
    }

    pub fn emphatic() -> Self {
        // Énfasis
        Self {
            start_freq: 150.0,
            peak_freq: 280.0,
            end_freq: 140.0,
            peak_position: 0.2,
        }
    }
}

/// Generador de prosodia
pub struct ProsodyGenerator {
    base_pitch: f32,
    speech_rate: f32,
}

impl ProsodyGenerator {
    pub fn new() -> Self {
        Self {
            base_pitch: 165.0,
            speech_rate: 1.0,
        }
    }

    /// Aplica prosodia a una secuencia de phonemes
    pub fn apply_prosody(&self, phonemes: &mut [Phoneme], sentence_type: SentenceType) {
        let intonation = match sentence_type {
            SentenceType::Declarative => Intonation::declarative(),
            SentenceType::Interrogative => Intonation::interrogative(),
            SentenceType::Emphatic => Intonation::emphatic(),
            SentenceType::Exclamatory => Intonation::emphatic(),
        };

        // Calcular duración total
        let total_duration: u32 = phonemes.iter().map(|p| p.default_duration()).sum();

        let mut elapsed: u32 = 0;

        for phoneme in phonemes.iter_mut() {
            let progress = elapsed as f32 / total_duration as f32;

            // Modificar duración basado en posición
            let duration = phoneme.default_duration();
            let new_duration = self.adjust_duration(duration, progress, &intonation);

            // Agregar pitch accent si corresponde
            if let Phoneme::Vowel { duration_ms, .. } = phoneme {
                *duration_ms = new_duration;

                // Insertar pitch accent cerca del peak
                if (progress - intonation.peak_position).abs() < 0.1 {
                    // Ya se manejó en synthesize
                }
            }

            if let Phoneme::Consonant { .. } = phoneme {
                // Reducir duración de consonantes para fluidez
                if let Phoneme::Consonant {
                    name,
                    voiced,
                    place,
                    manner,
                } = phoneme.clone()
                {
                    let _new_dur = (duration as f32 * 0.8 / self.speech_rate) as u32;
                    *phoneme = Phoneme::Consonant {
                        name,
                        voiced,
                        place,
                        manner,
                    };
                }
            }

            elapsed += duration;
        }
    }

    fn adjust_duration(&self, duration: u32, progress: f32, intonation: &Intonation) -> u32 {
        let factor = if progress < intonation.peak_position {
            // Antes del peak: acelerar
            0.9 + 0.1 * progress / intonation.peak_position
        } else {
            // Después del peak: ralentizar
            let post_peak =
                (progress - intonation.peak_position) / (1.0 - intonation.peak_position);
            1.0 - 0.2 * post_peak
        };

        (duration as f32 * factor / self.speech_rate) as u32
    }

    /// Genera pitch contour como señal
    pub fn generate_pitch_contour(&self, duration_ms: u32, intonation: &Intonation) -> AudioBuffer {
        let num_samples = ((SAMPLE_RATE as f32 * duration_ms as f32) / 1000.0) as usize;
        let mut samples = Vec::with_capacity(num_samples);

        for i in 0..num_samples {
            let t = i as f32 / num_samples as f32;

            // Interpolación de pitch
            let freq = if t < intonation.peak_position {
                // Pre-peak: interpolación lineal hacia peak
                intonation.start_freq
                    + (intonation.peak_freq - intonation.start_freq)
                        * (t / intonation.peak_position)
            } else {
                // Post-peak: interpolación hacia end
                intonation.peak_freq
                    + (intonation.end_freq - intonation.peak_freq)
                        * ((t - intonation.peak_position) / (1.0 - intonation.peak_position))
            };

            // Generar muestra con ese pitch
            let sample =
                (2.0 * std::f32::consts::PI * freq * (i as f32 / SAMPLE_RATE as f32)).sin();
            samples.push((sample * 0.1 * 32767.0) as i16);
        }

        AudioBuffer::new(samples)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SentenceType {
    Declarative,
    Interrogative,
    Emphatic,
    Exclamatory,
}

impl Default for ProsodyGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_declarative_intonation() {
        let pg = ProsodyGenerator::new();
        let contour = pg.generate_pitch_contour(1000, &Intonation::declarative());
        assert!(contour.len() > 0);
    }
}
