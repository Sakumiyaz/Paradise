//! # VOICE - Synthesizer & Recognizer
//!
//! Sistema de síntesis y reconocimiento de voz 100% Rust.
//! Sin dependencias externas - todo desde cero.
//!
//! ## Componentes
//!
//! - **Synthesizer**: TTS basado en formantes y ondas sinusoidales
//! - **Recognizer**: STT usando FFT y MFCC
//! - **Vocoder**: Conversión entre texto y audio
//! - **Prosody**: Entonación y ritmo natural

#![allow(dead_code)]

mod prosody;
mod recognizer;
mod synthesizer;
mod vocoder;

pub use prosody::ProsodyGenerator;
pub use recognizer::Recognizer;
pub use synthesizer::Synthesizer;
pub use vocoder::Vocoder;

// Constantes de audio
pub const SAMPLE_RATE: u32 = 22050;
pub const BIT_DEPTH: u16 = 16;
pub const CHANNELS: u16 = 1;

/// Frecuencias formánticas estándar (Hz) para vocal tract promedio
#[derive(Debug, Clone, PartialEq)]
pub struct FormantFreqs {
    pub f1: f32, // Primera formante
    pub f2: f32, // Segunda formante
    pub f3: f32, // Tercera formante
    pub f4: f32, // Cuarta formante
}

impl FormantFreqs {
    /// Frequencies para vocales en inglés (aproximado)
    pub fn for_vowel(vowel: &str) -> Self {
        match vowel.to_lowercase().as_str() {
            "a" | "ah" => FormantFreqs {
                f1: 730.0,
                f2: 1090.0,
                f3: 2440.0,
                f4: 3300.0,
            },
            "e" | "eh" => FormantFreqs {
                f1: 530.0,
                f2: 1840.0,
                f3: 2500.0,
                f4: 3500.0,
            },
            "i" | "ih" => FormantFreqs {
                f1: 390.0,
                f2: 1990.0,
                f3: 2550.0,
                f4: 3600.0,
            },
            "o" | "oh" => FormantFreqs {
                f1: 570.0,
                f2: 840.0,
                f3: 2410.0,
                f4: 3200.0,
            },
            "u" | "uh" => FormantFreqs {
                f1: 440.0,
                f2: 1020.0,
                f3: 2240.0,
                f4: 3100.0,
            },
            _ => FormantFreqs {
                f1: 500.0,
                f2: 1500.0,
                f3: 2500.0,
                f4: 3500.0,
            },
        }
    }
}

/// Phoneme básico del idioma
#[derive(Debug, Clone, PartialEq)]
pub enum Phoneme {
    // Vocales
    Vowel {
        name: String,
        formant: FormantFreqs,
        duration_ms: u32,
    },
    // Consonantes
    Consonant {
        name: String,
        voiced: bool,
        place: String,
        manner: String,
    },
    // Pausa
    Silence {
        duration_ms: u32,
    },
    // Especiales
    PitchAccent {
        direction: i8,
    }, // -1 down, 0 flat, 1 up
}

impl Phoneme {
    /// Duración estándar del phoneme
    pub fn default_duration(&self) -> u32 {
        match self {
            Phoneme::Vowel { duration_ms, .. } => *duration_ms,
            Phoneme::Consonant { .. } => 50,
            Phoneme::Silence { duration_ms } => *duration_ms,
            Phoneme::PitchAccent { .. } => 10,
        }
    }
}

/// Audio buffer básico
#[derive(Debug, Clone)]
pub struct AudioBuffer {
    pub samples: Vec<i16>,
    pub sample_rate: u32,
    pub channels: u16,
}

impl AudioBuffer {
    pub fn new(samples: Vec<i16>) -> Self {
        Self {
            samples,
            sample_rate: SAMPLE_RATE,
            channels: 1,
        }
    }

    pub fn empty() -> Self {
        Self {
            samples: Vec::new(),
            sample_rate: SAMPLE_RATE,
            channels: 1,
        }
    }

    pub fn silence_ms(duration_ms: u32) -> Self {
        let num_samples = ((SAMPLE_RATE as u64 * duration_ms as u64) / 1000) as usize;
        Self {
            samples: vec![0; num_samples],
            sample_rate: SAMPLE_RATE,
            channels: 1,
        }
    }

    pub fn len(&self) -> usize {
        self.samples.len()
    }

    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    pub fn append(&mut self, other: &AudioBuffer) {
        self.samples.extend_from_slice(&other.samples);
    }
}

/// Onda sinusoidal pura
pub fn sine_wave(freq: f32, duration_ms: u32, amplitude: f32) -> AudioBuffer {
    let num_samples = ((SAMPLE_RATE as f32 * duration_ms as f32) / 1000.0) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    for i in 0..num_samples {
        let t = i as f32 / SAMPLE_RATE as f32;
        let sample = (2.0 * std::f32::consts::PI * freq * t).sin() * amplitude;
        samples.push((sample * 32767.0) as i16);
    }

    AudioBuffer::new(samples)
}

/// Ruido blanco para consonantes
pub fn white_noise(duration_ms: u32, amplitude: f32) -> AudioBuffer {
    use std::time::SystemTime;
    let num_samples = ((SAMPLE_RATE as f32 * duration_ms as f32) / 1000.0) as usize;
    let mut samples = Vec::with_capacity(num_samples);

    let seed = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u32;

    let mut rng = seed;
    for _ in 0..num_samples {
        rng = rng.wrapping_mul(1103515245).wrapping_add(12345);
        let noise = ((rng as i32 >> 16) & 0x7FFF) as f32 / 32768.0;
        samples.push((noise * amplitude * 32767.0) as i16);
    }

    AudioBuffer::new(samples)
}

/// Mezcla dos audio buffers
pub fn mix(a: &AudioBuffer, b: &AudioBuffer, ratio_a: f32) -> AudioBuffer {
    let len = a.samples.len().max(b.samples.len());
    let mut result = Vec::with_capacity(len);

    for i in 0..len {
        let sample_a = a.samples.get(i).unwrap_or(&0);
        let sample_b = b.samples.get(i).unwrap_or(&0);
        let mixed = (*sample_a as f32 * ratio_a + *sample_b as f32 * (1.0 - ratio_a)) as i16;
        result.push(mixed.clamp(-32768, 32767));
    }

    AudioBuffer::new(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sine_wave() {
        let wave = sine_wave(440.0, 1000, 0.5);
        assert_eq!(wave.sample_rate, SAMPLE_RATE);
        assert!(wave.len() > 0);
    }

    #[test]
    fn test_silence() {
        let silence = AudioBuffer::silence_ms(100);
        assert_eq!(silence.len(), (SAMPLE_RATE as usize / 10)); // 100ms
    }
}
