//! # Synthesizer - Text to Speech
//!
//! Síntesis de voz desde texto usando formantes.
//! Sin dependencias externas - 100% Rust.

#![allow(dead_code)]

//use super::{AudioBuffer, FormantFreqs, Phoneme, SAMPLE_RATE, sine_wave, mix};

use super::{mix, sine_wave, white_noise, AudioBuffer, FormantFreqs, Phoneme, SAMPLE_RATE};

/// Synthesizer principal - convierte texto a audio
pub struct Synthesizer {
    base_freq: f32, // Frecuencia base del hablante (pitch)
    speed: f32,     // Velocidad del habla (1.0 = normal)
    volume: f32,    // Volumen (0.0 - 1.0)
}

impl Synthesizer {
    pub fn new() -> Self {
        Self {
            base_freq: 150.0, // 150 Hz - voz masculina grave
            speed: 1.0,
            volume: 0.8,
        }
    }

    /// Crea sintetizador con configuración custom
    pub fn with_config(base_freq: f32, speed: f32, volume: f32) -> Self {
        Self {
            base_freq,
            speed,
            volume,
        }
    }

    /// Sintetiza texto completo a audio
    pub fn speak(&self, text: &str) -> AudioBuffer {
        let phonemes = self.text_to_phonemes(text);
        self.phonemes_to_audio(&phonemes)
    }

    /// Text -> Phoneme sequence
    fn text_to_phonemes(&self, text: &str) -> Vec<Phoneme> {
        let mut phonemes = Vec::new();

        for word in text.split_whitespace() {
            let word_phonemes = self.word_to_phonemes(word);
            phonemes.extend(word_phonemes);
            // Espacio entre palabras
            phonemes.push(Phoneme::Silence { duration_ms: 50 });
        }

        phonemes
    }

    /// Word -> Phoneme sequence usando reglas simples
    fn word_to_phonemes(&self, word: &str) -> Vec<Phoneme> {
        let mut phonemes = Vec::new();
        let chars: Vec<char> = word.chars().collect();

        let mut i = 0;
        while i < chars.len() {
            let c = chars[i];

            // Vocales
            if Self::is_vowel(c) {
                let vowel_str = Self::char_to_vowel_string(c);
                let duration = self.estimate_vowel_duration(&chars, i);
                phonemes.push(Phoneme::Vowel {
                    name: vowel_str.clone(),
                    formant: FormantFreqs::for_vowel(&vowel_str),
                    duration_ms: duration,
                });
            }
            // Consonantes
            else if Self::is_consonant(c) {
                let consonant = Self::classify_consonant(c);
                phonemes.push(consonant);

                // Si es nasal (m, n, ng) seguida de vocal, crear resonancia
                if Self::is_nasal(c) && i + 1 < chars.len() && Self::is_vowel(chars[i + 1]) {
                    let nasal_formant = FormantFreqs {
                        f1: 250.0,
                        f2: 2000.0,
                        f3: 2500.0,
                        f4: 3500.0,
                    };
                    phonemes.push(Phoneme::Vowel {
                        name: "nasal".to_string(),
                        formant: nasal_formant,
                        duration_ms: 30,
                    });
                }
            }
            // Puntuación -> pausa
            else if c == '.' || c == '!' || c == '?' {
                phonemes.push(Phoneme::Silence { duration_ms: 200 });
                phonemes.push(Phoneme::PitchAccent { direction: -1 }); // Caída de pitch
            } else if c == ',' || c == ';' {
                phonemes.push(Phoneme::Silence { duration_ms: 100 });
            }

            i += 1;
        }

        phonemes
    }

    fn is_vowel(c: char) -> bool {
        matches!(c.to_ascii_lowercase(), 'a' | 'e' | 'i' | 'o' | 'u' | 'y')
    }

    fn is_consonant(c: char) -> bool {
        c.is_ascii_alphabetic() && !Self::is_vowel(c)
    }

    fn is_nasal(c: char) -> bool {
        matches!(c.to_ascii_lowercase(), 'm' | 'n')
    }

    fn char_to_vowel_string(c: char) -> String {
        match c.to_ascii_lowercase() {
            'a' => "a".to_string(),
            'e' => "e".to_string(),
            'i' => "i".to_string(),
            'o' => "o".to_string(),
            'u' => "u".to_string(),
            'y' => "i".to_string(),
            _ => "a".to_string(),
        }
    }

    fn estimate_vowel_duration(&self, chars: &[char], pos: usize) -> u32 {
        let base: u32 = 100;
        // Vocales largas en palabras especiales
        let c = chars[pos].to_ascii_lowercase();
        let is_long = (c == 'e' && pos > 0 && chars[pos - 1].to_ascii_lowercase() == 'e')
            || (c == 'i' && pos + 1 < chars.len() && chars[pos + 1].to_ascii_lowercase() == 'e');

        if is_long {
            (base as f32 * 1.5 / self.speed) as u32
        } else {
            (base as f32 / self.speed) as u32
        }
    }

    fn classify_consonant(c: char) -> Phoneme {
        let lower = c.to_ascii_lowercase();
        let (voiced, place, manner) = match lower {
            'b' | 'd' | 'g' | 'v' | 'z' | 'j' | 'w' => (true, "oral", "plosive"),
            'p' | 't' | 'k' | 'f' | 's' | 'h' => (false, "oral", "plosive"),
            'm' | 'n' => (true, "nasal", "nasal"),
            'l' | 'r' => (true, "oral", "liquid"),
            'c' => (false, "oral", "fricative"),
            'q' => (false, "oral", "plosive"),
            'x' => (false, "oral", "fricative"),
            _ => (false, "oral", "plosive"),
        };

        Phoneme::Consonant {
            name: c.to_string(),
            voiced,
            place: place.to_string(),
            manner: manner.to_string(),
        }
    }

    /// Phonemes -> AudioBuffer con formantes
    fn phonemes_to_audio(&self, phonemes: &[Phoneme]) -> AudioBuffer {
        let mut audio = AudioBuffer::empty();

        for phoneme in phonemes {
            let segment = self.phoneme_to_audio(phoneme);
            audio.append(&segment);
        }

        audio
    }

    fn phoneme_to_audio(&self, phoneme: &Phoneme) -> AudioBuffer {
        match phoneme {
            Phoneme::Vowel {
                formant,
                duration_ms,
                ..
            } => self.synthesize_vowel(formant, *duration_ms),
            Phoneme::Consonant { name, voiced, .. } => {
                self.synthesize_consonant(name.chars().next().unwrap_or('s'), *voiced)
            }
            Phoneme::Silence { duration_ms } => AudioBuffer::silence_ms(*duration_ms),
            Phoneme::PitchAccent { .. } => {
                // Pitch accent no genera audio, solo marca inflexión
                // El sintetizador lo usa para ajustar la siguiente vocal
                AudioBuffer::silence_ms(10)
            }
        }
    }

    /// Sintetiza vocal usando múltiples formantes
    fn synthesize_vowel(&self, formant: &FormantFreqs, duration_ms: u32) -> AudioBuffer {
        let num_samples = ((SAMPLE_RATE as f32 * duration_ms as f32) / 1000.0) as usize;
        let mut samples = Vec::with_capacity(num_samples);

        let attack = (num_samples / 10).max(1); // 10% attack
        let decay = (num_samples / 10).max(1); // 10% decay

        for i in 0..num_samples {
            let t = i as f32 / SAMPLE_RATE as f32;

            // Envelope: attack -> sustain -> decay
            let envelope = if i < attack {
                i as f32 / attack as f32
            } else if i > num_samples - decay {
                (num_samples - i) as f32 / decay as f32
            } else {
                1.0 - 0.3 * ((i - attack) as f32 / (num_samples - attack - decay) as f32).min(1.0)
            };

            // Generar formantes con amplitudes diferentes
            let f1_amp = 0.6 * (2.0 * std::f32::consts::PI * formant.f1 * t).sin();
            let f2_amp = 0.3 * (2.0 * std::f32::consts::PI * formant.f2 * t).sin();
            let f3_amp = 0.1 * (2.0 * std::f32::consts::PI * formant.f3 * t).sin();

            // Pitch base (fundamental)
            let pitch = 0.4 * (2.0 * std::f32::consts::PI * self.base_freq * t).sin();

            // Mezcla
            let sample = (f1_amp + f2_amp + f3_amp + pitch) * envelope * self.volume;
            samples.push((sample * 32767.0 * 0.5) as i16);
        }

        AudioBuffer::new(samples)
    }

    /// Sintetiza consonante (ruido + tono base)
    fn synthesize_consonant(&self, _c: char, voiced: bool) -> AudioBuffer {
        let duration_ms = 80;

        // Consonantes sordas = ruido blanco
        // Consonantes sonoras = ruido + tono
        let noise = white_noise(duration_ms, 0.3);

        if voiced {
            let tone = sine_wave(self.base_freq, duration_ms, 0.2);
            mix(&noise, &tone, 0.5)
        } else {
            noise
        }
    }
}

impl Default for Synthesizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_synthesize_hello() {
        let synth = Synthesizer::new();
        let audio = synth.speak("hello");
        assert!(audio.len() > 0);
    }

    #[test]
    fn test_vowel_synthesis() {
        let synth = Synthesizer::new();
        let f = FormantFreqs::for_vowel("a");
        let audio = synth.synthesize_vowel(&f, 200);
        assert_eq!(audio.len(), 4410); // ~200ms at 22050Hz
    }
}
