//! # Recognizer - Speech to Text
//!
//! Reconocimiento de voz usando FFT y MFCC.
//! Sin dependencias externas - 100% Rust.

#![allow(dead_code)]

//use super::{AudioBuffer, SAMPLE_RATE};

use super::AudioBuffer;

/// Recognizer - convierte audio a texto
/// Usa FFT básico y MFCC para extracción de features
pub struct Recognizer {
    sample_rate: u32,
    fft_size: usize,
}

impl Recognizer {
    pub fn new() -> Self {
        Self {
            sample_rate: super::SAMPLE_RATE,
            fft_size: 512,
        }
    }

    /// Reconoce texto desde AudioBuffer
    pub fn recognize(&self, audio: &AudioBuffer) -> Result<String, RecognizerError> {
        if audio.is_empty() {
            return Err(RecognizerError::EmptyAudio);
        }

        // Paso 1: Pre-emphasis ( усиление altas frecuencias)
        let emphasized = self.pre_emphasis(audio);

        // Paso 2: Windowing (ventana Hann)
        let windowed = self.apply_window(&emphasized);

        // Paso 3: FFT
        let spectrum = self.fft(&windowed);

        // Paso 4: Mel-spectrum
        let mel_spec = self.to_mel_spectrum(&spectrum);

        // Paso 5: MFCC
        let mfcc = self.compute_mfcc(&mel_spec);

        // Paso 6: Buscar fonemas en MFCC
        let text = self.mfcc_to_text(&mfcc);

        Ok(text)
    }

    fn pre_emphasis(&self, audio: &AudioBuffer) -> Vec<f32> {
        let mut result = Vec::with_capacity(audio.len());
        let alpha = 0.95;

        for (i, &sample) in audio.samples.iter().enumerate() {
            let s = sample as f32 / 32768.0;
            if i == 0 {
                result.push(s);
            } else {
                let prev = audio.samples[i - 1] as f32 / 32768.0;
                result.push(s - alpha * prev);
            }
        }

        result
    }

    fn apply_window(&self, samples: &[f32]) -> Vec<f32> {
        let n = samples.len();
        let mut windowed = Vec::with_capacity(n);

        for i in 0..n {
            let w = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i as f32 / (n - 1) as f32).cos());
            windowed.push(samples[i] * w);
        }

        windowed
    }

    /// FFT simplificado (DFT para casos simples)
    /// Para FFT real se usaría Cooley-Tukey pero esto funciona para demo
    fn fft(&self, samples: &[f32]) -> Vec<f32> {
        let n = samples.len().next_power_of_two();
        let mut padded = samples.to_vec();
        padded.resize(n, 0.0);

        // DFT simple O(n²) - OK para tamaños pequeños
        let mut spectrum = Vec::with_capacity(n / 2);

        for k in 0..n / 2 {
            let mut real = 0.0;
            let mut imag = 0.0;

            for (i, &s) in padded.iter().enumerate() {
                let angle = -2.0 * std::f32::consts::PI * k as f32 * i as f32 / n as f32;
                real += s * angle.cos();
                imag += s * angle.sin();
            }

            let magnitude = (real * real + imag * imag).sqrt();
            spectrum.push(magnitude);
        }

        spectrum
    }

    /// Convierte spectrum a Mel scale
    fn to_mel_spectrum(&self, spectrum: &[f32]) -> Vec<f32> {
        let n_mel_bins = 26;
        let mut mel_spec = Vec::with_capacity(n_mel_bins);

        // Frecuencias Mel
        let f_min = 0.0;
        let f_max = self.sample_rate as f32 / 2.0;

        for i in 0..n_mel_bins {
            // Frecuencia centro en Mel scale
            let mel_low =
                f_min + i as f32 * (2595.0 * (f_max / f_min).log10() / (n_mel_bins + 1) as f32);
            let mel_high = mel_low + 2595.0 * (f_max / f_min).log10() / (n_mel_bins + 1) as f32;

            // Energia en esa banda
            let f_low = 700.0 * (10.0_f32.powf(mel_low / 2595.0) - 1.0);
            let f_high = 700.0 * (10.0_f32.powf(mel_high / 2595.0) - 1.0);

            let bin_low =
                (f_low * spectrum.len() as f32 / (self.sample_rate as f32 / 2.0)) as usize;
            let bin_high =
                (f_high * spectrum.len() as f32 / (self.sample_rate as f32 / 2.0)) as usize;

            let mut energy = 0.0;
            let count = (bin_high - bin_low).max(1);
            for b in bin_low..bin_high.min(spectrum.len()) {
                energy += spectrum[b];
            }
            mel_spec.push(energy / count as f32);
        }

        mel_spec
    }

    /// DCT para obtener MFCC
    fn compute_mfcc(&self, mel_spec: &[f32]) -> Vec<f32> {
        let n_coef = 13;
        let n_mels = mel_spec.len();
        let mut mfcc = Vec::with_capacity(n_coef);

        // Log de mel spectrum
        let log_mel: Vec<f32> = mel_spec
            .iter()
            .map(|&x| (x + 1e-10).max(0.0).log10())
            .collect();

        // DCT tipo II
        for k in 0..n_coef {
            let mut sum = 0.0;
            for (n, &val) in log_mel.iter().enumerate() {
                let angle =
                    std::f32::consts::PI * k as f32 * (2 * n + 1) as f32 / (2 * n_mels) as f32;
                sum += val * angle.cos();
            }
            mfcc.push(sum);
        }

        mfcc
    }

    /// Convierte MFCC a texto usando pattern matching simple
    fn mfcc_to_text(&self, mfcc: &[f32]) -> String {
        // Modelo muy simplificado - en producción usaría HMM o red neuronal
        // Aquí usamos thresholds básicos

        if mfcc.is_empty() {
            return String::new();
        }

        // Características simples para detección de vocales
        let energy = mfcc[0];

        // Si hay energía significativa, asumimos que hay voz
        if energy > 5.0 {
            // Estimación burda de fonema basado en coeficientes
            let ratio = mfcc.get(1).unwrap_or(&0.0) / mfcc.get(2).unwrap_or(&0.1).max(0.1);

            if ratio > 1.5 {
                "a".to_string()
            } else if ratio > 1.0 {
                "e".to_string()
            } else if ratio > 0.5 {
                "i".to_string()
            } else {
                "o".to_string()
            }
        } else {
            String::new()
        }
    }
}

#[derive(Debug, Clone)]
pub enum RecognizerError {
    EmptyAudio,
    ProcessingError,
}

impl std::fmt::Display for RecognizerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecognizerError::EmptyAudio => write!(f, "Audio está vacío"),
            RecognizerError::ProcessingError => write!(f, "Error en procesamiento"),
        }
    }
}

impl Default for Recognizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recognizer_empty() {
        let rec = Recognizer::new();
        let result = rec.recognize(&AudioBuffer::empty());
        assert!(result.is_err());
    }
}
