// EDEN GARM Voice — Absolute Ceiling Edition
// VAD (energy + ZCR) + MFCC stub + DTW phoneme matching + Formant TTS + diphone inventory

#[derive(Clone, Debug)]
pub struct AudioBuffer {
    pub samples: Vec<i16>,
    pub sample_rate: u32,
}

#[derive(Clone, Debug)]
pub struct VoiceResult {
    pub text: String,
    pub confidence: f32,
    pub is_speech: bool,
    pub phoneme_sequence: Vec<String>,
}

pub struct VoiceEngine {
    pub phoneme_map: Vec<(String, Vec<String>)>,
    pub stt_history: Vec<VoiceResult>,
    pub tts_history: Vec<String>,
    pub diphone_inventory: Vec<(String, Vec<i16>)>,
}

impl VoiceEngine {
    pub fn new() -> Self {
        let phoneme_map = vec![
            (
                "hola".into(),
                vec!["h".into(), "o".into(), "l".into(), "a".into()],
            ),
            (
                "hello".into(),
                vec!["h".into(), "eh".into(), "l".into(), "o".into()],
            ),
            (
                "eden".into(),
                vec!["eh".into(), "d".into(), "eh".into(), "n".into()],
            ),
            ("si".into(), vec!["s".into(), "i".into()]),
            ("yes".into(), vec!["y".into(), "eh".into(), "s".into()]),
            ("no".into(), vec!["n".into(), "o".into()]),
            (
                "gracias".into(),
                vec![
                    "g".into(),
                    "r".into(),
                    "a".into(),
                    "c".into(),
                    "i".into(),
                    "a".into(),
                    "s".into(),
                ],
            ),
            (
                "thanks".into(),
                vec!["th".into(), "a".into(), "n".into(), "k".into(), "s".into()],
            ),
            (
                "stop".into(),
                vec!["s".into(), "t".into(), "o".into(), "p".into()],
            ),
            (
                "help".into(),
                vec!["h".into(), "eh".into(), "l".into(), "p".into()],
            ),
            (
                "ayuda".into(),
                vec!["a".into(), "y".into(), "u".into(), "d".into(), "a".into()],
            ),
            (
                "status".into(),
                vec![
                    "s".into(),
                    "t".into(),
                    "a".into(),
                    "t".into(),
                    "u".into(),
                    "s".into(),
                ],
            ),
            (
                "computer".into(),
                vec![
                    "k".into(),
                    "o".into(),
                    "m".into(),
                    "p".into(),
                    "y".into(),
                    "u".into(),
                    "t".into(),
                    "e".into(),
                    "r".into(),
                ],
            ),
            ("tool".into(), vec!["t".into(), "u".into(), "l".into()]),
            (
                "execute".into(),
                vec![
                    "eh".into(),
                    "k".into(),
                    "s".into(),
                    "eh".into(),
                    "k".into(),
                    "y".into(),
                    "u".into(),
                    "t".into(),
                ],
            ),
            (
                "browse".into(),
                vec!["b".into(), "r".into(), "o".into(), "w".into(), "s".into()],
            ),
            (
                "security".into(),
                vec![
                    "s".into(),
                    "eh".into(),
                    "k".into(),
                    "y".into(),
                    "u".into(),
                    "r".into(),
                    "ih".into(),
                    "t".into(),
                    "i".into(),
                ],
            ),
        ];
        // Generate simple diphone waveforms (pairs of phonemes as transitions)
        let mut diphone_inventory = Vec::new();
        let all_phonemes = [
            "a", "e", "i", "o", "u", "h", "l", "n", "s", "t", "d", "r", "c", "g", "y", "p", "k",
            "m", "w", "b", "eh", "ih", "th", "y",
        ];
        for p1 in &all_phonemes {
            for p2 in &all_phonemes {
                let mut samples = Vec::with_capacity(320); // 20ms at 16kHz
                for s in 0..320 {
                    let t = s as f32 / 320.0;
                    let f1 = phoneme_to_freq(p1);
                    let f2 = phoneme_to_freq(p2);
                    let freq = f1 * (1.0 - t) + f2 * t;
                    let sample = ((s as f32 * freq * 2.0 * std::f32::consts::PI / 16000.0).sin()
                        * 4000.0) as i16;
                    samples.push(sample);
                }
                diphone_inventory.push((format!("{}-{}", p1, p2), samples));
            }
        }
        VoiceEngine {
            phoneme_map,
            stt_history: Vec::new(),
            tts_history: Vec::new(),
            diphone_inventory,
        }
    }

    // ─── VAD: Energy + Zero-Crossing Rate ───
    pub fn detect_voice_activity(&self, audio: &AudioBuffer) -> Vec<(usize, usize)> {
        let frame_size = (audio.sample_rate / 10) as usize; // 100ms
        let energy_threshold = 500i16;
        let zcr_threshold = 0.15; // 15% of samples crossing zero
        let mut segments = Vec::new();
        let mut in_speech = false;
        let mut start = 0usize;

        for (i, chunk) in audio.samples.chunks(frame_size).enumerate() {
            let rms = (chunk.iter().map(|&s| (s as i64) * (s as i64)).sum::<i64>() as f32
                / chunk.len().max(1) as f32)
                .sqrt();
            let zcr = if chunk.len() > 1 {
                let crossings = chunk
                    .windows(2)
                    .filter(|w| (w[0] >= 0 && w[1] < 0) || (w[0] < 0 && w[1] >= 0))
                    .count();
                crossings as f32 / chunk.len() as f32
            } else {
                0.0
            };
            let is_active = rms > energy_threshold as f32 && zcr < zcr_threshold;
            if is_active && !in_speech {
                in_speech = true;
                start = i * frame_size;
            } else if !is_active && in_speech {
                in_speech = false;
                segments.push((start, i * frame_size));
            }
        }
        if in_speech {
            segments.push((start, audio.samples.len()));
        }
        segments
    }

    // ─── STT: DTW-based phoneme matching ───
    pub fn recognize(&mut self, audio: &AudioBuffer) -> VoiceResult {
        let segments = self.detect_voice_activity(audio);
        if segments.is_empty() {
            return VoiceResult {
                text: String::new(),
                confidence: 0.0,
                is_speech: false,
                phoneme_sequence: Vec::new(),
            };
        }
        // Extract energy contour as pseudo-MFCC feature vector
        let frame_size = (audio.sample_rate / 50) as usize; // 20ms
        let mut energy_contour = Vec::new();
        for chunk in audio.samples.chunks(frame_size) {
            let energy = (chunk.iter().map(|&s| (s as i64) * (s as i64)).sum::<i64>() as f32
                / chunk.len().max(1) as f32)
                .sqrt();
            energy_contour.push(energy);
        }
        // Match against all keywords using simplified DTW on energy contour
        let mut best_word = "[unknown]".to_string();
        let mut best_score = f32::MAX;
        let mut best_phonemes = Vec::new();
        for (word, phonemes) in &self.phoneme_map {
            // Generate synthetic energy contour for this word
            let synthetic =
                self.synthesize_energy_contour(phonemes, audio.samples.len(), audio.sample_rate);
            let dist = dtw_distance(&energy_contour, &synthetic);
            if dist < best_score {
                best_score = dist;
                best_word = word.clone();
                best_phonemes = phonemes.clone();
            }
        }
        let confidence = (1.0 / (1.0 + best_score / 100.0)).min(1.0);
        let result = VoiceResult {
            text: best_word,
            confidence,
            is_speech: true,
            phoneme_sequence: best_phonemes,
        };
        self.stt_history.push(result.clone());
        if self.stt_history.len() > 100 {
            self.stt_history.remove(0);
        }
        result
    }

    fn synthesize_energy_contour(
        &self,
        phonemes: &[String],
        target_len: usize,
        sample_rate: u32,
    ) -> Vec<f32> {
        let frame_size = (sample_rate / 50) as usize;
        let num_frames = target_len / frame_size;
        let mut contour = Vec::with_capacity(num_frames);
        let phones = phonemes.len().max(1);
        let frames_per_phone = num_frames / phones;
        for (_i, phone) in phonemes.iter().enumerate() {
            let base_energy = phoneme_to_energy(phone);
            for f in 0..frames_per_phone {
                let t = f as f32 / frames_per_phone.max(1) as f32;
                let envelope = if t < 0.2 {
                    t / 0.2
                } else if t > 0.8 {
                    (1.0 - t) / 0.2
                } else {
                    1.0
                };
                contour.push(base_energy * envelope);
            }
        }
        while contour.len() < num_frames {
            contour.push(0.0);
        }
        contour.truncate(num_frames);
        contour
    }

    // ─── TTS: Diphone concatenation ───
    pub fn synthesize(&mut self, text: &str) -> AudioBuffer {
        let sample_rate = 16000u32;
        let words = text
            .to_lowercase()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect::<Vec<_>>();
        let mut all_samples: Vec<i16> = Vec::new();
        for word in words {
            let phonemes = self
                .phoneme_map
                .iter()
                .find(|(w, _)| w == &word)
                .map(|(_, p)| p.clone())
                .unwrap_or_else(|| word.chars().map(|c| c.to_string()).collect());
            for i in 0..phonemes.len() {
                let p1 = &phonemes[i];
                let p2 = phonemes.get(i + 1).map(|s| s.as_str()).unwrap_or("_");
                let key = format!("{}-{}", p1, p2);
                if let Some((_, samples)) = self.diphone_inventory.iter().find(|(k, _)| k == &key) {
                    all_samples.extend(samples.iter().copied());
                } else {
                    // Fallback: generate simple sine
                    for s in 0..320 {
                        let f = phoneme_to_freq(p1);
                        let sample =
                            ((s as f32 * f * 2.0 * std::f32::consts::PI / sample_rate as f32).sin()
                                * 4000.0) as i16;
                        all_samples.push(sample);
                    }
                }
            }
            // Add pause between words
            all_samples.extend(vec![0i16; 800].into_iter());
        }
        self.tts_history.push(text.to_string());
        if self.tts_history.len() > 100 {
            self.tts_history.remove(0);
        }
        AudioBuffer {
            samples: all_samples,
            sample_rate,
        }
    }

    pub fn status(&self) -> String {
        format!(
            "Voice | STT: {} | TTS: {} | vocab: {} | diphones: {}",
            self.stt_history.len(),
            self.tts_history.len(),
            self.phoneme_map.len(),
            self.diphone_inventory.len()
        )
    }
}

// ─── DTW Distance ───

fn dtw_distance(a: &[f32], b: &[f32]) -> f32 {
    let n = a.len();
    let m = b.len();
    if n == 0 || m == 0 {
        return f32::MAX;
    }
    let mut prev = vec![f32::MAX; m + 1];
    let mut curr = vec![f32::MAX; m + 1];
    prev[0] = 0.0;
    for i in 1..=n {
        curr[0] = f32::MAX;
        for j in 1..=m {
            let cost = (a[i - 1] - b[j - 1]).abs();
            curr[j] = cost + curr[j - 1].min(prev[j]).min(prev[j - 1]);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[m]
}

// ─── Phoneme Helpers ───

fn phoneme_to_freq(phoneme: &str) -> f32 {
    match phoneme {
        "a" | "ah" => 730.0,
        "e" | "eh" => 600.0,
        "i" | "ih" => 270.0,
        "o" | "oh" => 450.0,
        "u" | "uh" => 300.0,
        "h" => 100.0,
        "l" => 350.0,
        "n" => 250.0,
        "s" => 4000.0,
        "t" | "d" | "k" | "p" => 200.0,
        "r" => 500.0,
        "c" | "g" | "y" | "m" | "w" | "b" | "th" => 400.0,
        _ => 300.0,
    }
}

fn phoneme_to_energy(phoneme: &str) -> f32 {
    match phoneme {
        "a" | "e" | "i" | "o" | "u" | "ah" | "eh" | "ih" | "oh" | "uh" => 8000.0,
        "h" | "l" | "n" | "r" | "m" | "y" | "w" => 4000.0,
        "s" | "th" => 2000.0,
        "t" | "d" | "k" | "p" | "b" | "c" | "g" => 1500.0,
        _ => 3000.0,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// NEW CAPABILITIES — Absolute Ceiling
// ═══════════════════════════════════════════════════════════════════════════════

// ─── 1. Real FFT/DFT (Cooley-Tukey iterative) ───

fn fft(input: &[f32]) -> Vec<(f32, f32)> {
    let n = input.len().next_power_of_two();
    let mut real: Vec<f32> = input
        .iter()
        .copied()
        .chain(std::iter::repeat(0.0))
        .take(n)
        .collect();
    let mut imag: Vec<f32> = vec![0.0f32; n];
    let mut j = 0usize;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j ^= bit;
        if i < j {
            real.swap(i, j);
            imag.swap(i, j);
        }
    }
    let mut len = 2usize;
    while len <= n {
        let half = len >> 1;
        let theta = -2.0 * std::f32::consts::PI / len as f32;
        for i in (0..n).step_by(len) {
            let mut w_re = 1.0f32;
            let mut w_im = 0.0f32;
            let w_delta_re = theta.cos();
            let w_delta_im = theta.sin();
            for k in 0..half {
                let u_re = real[i + k];
                let u_im = imag[i + k];
                let v_re = real[i + k + half] * w_re - imag[i + k + half] * w_im;
                let v_im = real[i + k + half] * w_im + imag[i + k + half] * w_re;
                real[i + k] = u_re + v_re;
                imag[i + k] = u_im + v_im;
                real[i + k + half] = u_re - v_re;
                imag[i + k + half] = u_im - v_im;
                let next_w_re = w_re * w_delta_re - w_im * w_delta_im;
                let next_w_im = w_re * w_delta_im + w_im * w_delta_re;
                w_re = next_w_re;
                w_im = next_w_im;
            }
        }
        len <<= 1;
    }
    real.into_iter().zip(imag.into_iter()).collect()
}

fn ifft(input: &[(f32, f32)]) -> Vec<f32> {
    let n = input.len();
    let mut real: Vec<f32> = input.iter().map(|(r, _)| *r).collect();
    let mut imag: Vec<f32> = input.iter().map(|(_, i)| *i).collect();
    let mut j = 0usize;
    for i in 1..n {
        let mut bit = n >> 1;
        while j & bit != 0 {
            j ^= bit;
            bit >>= 1;
        }
        j ^= bit;
        if i < j {
            real.swap(i, j);
            imag.swap(i, j);
        }
    }
    let mut len = 2usize;
    while len <= n {
        let half = len >> 1;
        let theta = 2.0 * std::f32::consts::PI / len as f32;
        for i in (0..n).step_by(len) {
            let mut w_re = 1.0f32;
            let mut w_im = 0.0f32;
            let w_delta_re = theta.cos();
            let w_delta_im = theta.sin();
            for k in 0..half {
                let u_re = real[i + k];
                let u_im = imag[i + k];
                let v_re = real[i + k + half] * w_re - imag[i + k + half] * w_im;
                let v_im = real[i + k + half] * w_im + imag[i + k + half] * w_re;
                real[i + k] = u_re + v_re;
                imag[i + k] = u_im + v_im;
                real[i + k + half] = u_re - v_re;
                imag[i + k + half] = u_im - v_im;
                let next_w_re = w_re * w_delta_re - w_im * w_delta_im;
                let next_w_im = w_re * w_delta_im + w_im * w_delta_re;
                w_re = next_w_re;
                w_im = next_w_im;
            }
        }
        len <<= 1;
    }
    real.into_iter()
        .zip(imag.into_iter())
        .map(|(r, _i)| r / n as f32)
        .collect()
}

fn magnitude_spectrum(fft_out: &[(f32, f32)]) -> Vec<f32> {
    fft_out
        .iter()
        .map(|(r, i)| (r * r + i * i).sqrt())
        .collect()
}

// ─── 2. Mel-frequency filterbank & real MFCC extraction (13 coefficients) ───

fn hz_to_mel(hz: f32) -> f32 {
    2595.0 * (1.0 + hz / 700.0).log10()
}

fn mel_to_hz(mel: f32) -> f32 {
    700.0 * (10f32.powf(mel / 2595.0) - 1.0)
}

fn mel_filterbank(num_filters: usize, fft_size: usize, sample_rate: u32) -> Vec<Vec<f32>> {
    let min_mel = hz_to_mel(0.0);
    let max_mel = hz_to_mel(sample_rate as f32 / 2.0);
    let step = (max_mel - min_mel) / (num_filters + 1) as f32;
    let mut bins: Vec<usize> = (0..num_filters + 2)
        .map(|i| {
            let hz = mel_to_hz(min_mel + step * i as f32);
            let bin = (hz / sample_rate as f32 * fft_size as f32).round() as usize;
            bin.min(fft_size / 2)
        })
        .collect();
    for i in 1..bins.len() {
        if bins[i] <= bins[i - 1] {
            bins[i] = bins[i - 1] + 1;
        }
    }
    let mut filters = Vec::with_capacity(num_filters);
    for i in 0..num_filters {
        let mut filter = vec![0.0f32; fft_size / 2 + 1];
        let left = bins[i];
        let center = bins[i + 1];
        let right = bins[i + 2];
        if center > left {
            for k in left..=center {
                filter[k] = (k - left) as f32 / (center - left) as f32;
            }
        }
        if right > center {
            for k in center..=right {
                filter[k] = (right - k) as f32 / (right - center) as f32;
            }
        }
        filters.push(filter);
    }
    filters
}

fn dct_type_2(input: &[f32], num_ceps: usize) -> Vec<f32> {
    let n = input.len();
    let mut out = vec![0.0f32; num_ceps];
    let scale = std::f32::consts::PI / n as f32;
    for k in 0..num_ceps {
        let mut sum = 0.0f32;
        for (i, &val) in input.iter().enumerate() {
            sum += val * ((i as f32 + 0.5) * scale * k as f32).cos();
        }
        out[k] = sum;
    }
    out
}

pub fn compute_mfcc(audio: &AudioBuffer, num_ceps: usize) -> Vec<f32> {
    let frame_size = 512usize;
    let n = audio.samples.len().min(frame_size);
    if n == 0 {
        return vec![0.0f32; num_ceps];
    }
    let alpha = 0.97f32;
    let mut frame = Vec::with_capacity(n);
    frame.push(audio.samples[0] as f32);
    for i in 1..n {
        frame.push(audio.samples[i] as f32 - alpha * audio.samples[i - 1] as f32);
    }
    for (i, s) in frame.iter_mut().enumerate() {
        let w = 0.5 - 0.5 * ((2.0 * std::f32::consts::PI * i as f32) / (n as f32 - 1.0)).cos();
        *s *= w;
    }
    let fft_n = n.next_power_of_two();
    let mut fft_input = vec![0.0f32; fft_n];
    fft_input[..n].copy_from_slice(&frame);
    let fft_out = fft(&fft_input);
    let mag = magnitude_spectrum(&fft_out);
    let power: Vec<f32> = mag.iter().map(|&m| m * m).collect();
    let filters = mel_filterbank(26, fft_n, audio.sample_rate);
    let mut mel_energies = vec![0.0f32; filters.len()];
    for (i, filter) in filters.iter().enumerate() {
        let len = filter.len().min(power.len());
        let e = filter[..len]
            .iter()
            .zip(&power[..len])
            .map(|(a, b)| a * b)
            .sum::<f32>();
        mel_energies[i] = e.max(1e-10).ln();
    }
    dct_type_2(&mel_energies, num_ceps)
}

// ─── 3. Pitch detection via autocorrelation ───

pub fn detect_pitch(audio: &AudioBuffer) -> Option<f32> {
    let sr = audio.sample_rate as f32;
    let frame_len = (sr * 30.0 / 1000.0) as usize;
    let n = audio.samples.len().min(frame_len);
    if n < 2 {
        return None;
    }
    let max_abs = audio.samples[..n]
        .iter()
        .map(|&s| s.abs() as f32)
        .fold(0.0f32, f32::max)
        .max(1.0);
    let frame: Vec<f32> = audio.samples[..n]
        .iter()
        .map(|&s| s as f32 / max_abs)
        .collect();
    let min_lag = (sr / 400.0) as usize;
    let max_lag = ((sr / 80.0) as usize).min(n / 2).max(min_lag + 1);
    let mut best_lag = 0usize;
    let mut best_corr = -1.0f32;
    for lag in min_lag..max_lag {
        let mut corr = 0.0f32;
        for i in 0..(n - lag) {
            corr += frame[i] * frame[i + lag];
        }
        if corr > best_corr {
            best_corr = corr;
            best_lag = lag;
        }
    }
    if best_lag == 0 || best_corr < 0.1 {
        return None;
    }
    Some(sr / best_lag as f32)
}

// ─── 4. LPC (Linear Predictive Coding) coefficients ───

pub fn lpc_coefficients(frame: &[f32], order: usize) -> Vec<f32> {
    let n = frame.len();
    if n == 0 {
        return vec![0.0f32; order];
    }
    let mut r = vec![0.0f32; order + 1];
    for i in 0..=order {
        let limit = n.saturating_sub(i);
        for j in 0..limit {
            r[i] += frame[j] * frame[j + i];
        }
    }
    if r[0] == 0.0 {
        return vec![0.0f32; order];
    }
    let mut a = vec![0.0f32; order + 1];
    a[0] = 1.0;
    let mut e = r[0];
    for k in 1..=order {
        let mut lambda = 0.0f32;
        for j in 0..k {
            lambda += a[j] * r[k - j];
        }
        lambda = -lambda / e;
        let mut a_new = a.clone();
        for j in 0..=k {
            a_new[j] = a[j] + lambda * a[k - j];
        }
        a = a_new;
        a[k] = lambda;
        e = e * (1.0 - lambda * lambda);
        if e <= 0.0 {
            break;
        }
    }
    a.into_iter().skip(1).take(order).collect()
}

// ─── 5. Spectral centroid and bandwidth ───

pub fn spectral_centroid(magnitude: &[f32], sample_rate: u32, fft_size: usize) -> f32 {
    let bin_width = sample_rate as f32 / fft_size as f32;
    let mut num = 0.0f32;
    let mut den = 0.0f32;
    for (i, &m) in magnitude.iter().enumerate() {
        let freq = i as f32 * bin_width;
        num += freq * m;
        den += m;
    }
    if den > 0.0 {
        num / den
    } else {
        0.0
    }
}

pub fn spectral_bandwidth(
    magnitude: &[f32],
    sample_rate: u32,
    fft_size: usize,
    centroid: f32,
) -> f32 {
    let bin_width = sample_rate as f32 / fft_size as f32;
    let mut num = 0.0f32;
    let mut den = 0.0f32;
    for (i, &m) in magnitude.iter().enumerate() {
        let diff = i as f32 * bin_width - centroid;
        num += diff * diff * m;
        den += m;
    }
    if den > 0.0 {
        (num / den).sqrt()
    } else {
        0.0
    }
}

// ─── 6. Speech rate estimation (syllable count / duration) ───

pub fn estimate_speech_rate(audio: &AudioBuffer) -> f32 {
    let duration = audio.samples.len() as f32 / audio.sample_rate.max(1) as f32;
    if duration <= 0.0 {
        return 0.0;
    }
    let frame_size = (audio.sample_rate / 50).max(1) as usize;
    let mut energies = Vec::new();
    for chunk in audio.samples.chunks(frame_size) {
        let e = (chunk.iter().map(|&s| (s as i64) * (s as i64)).sum::<i64>() as f32
            / chunk.len().max(1) as f32)
            .sqrt();
        energies.push(e);
    }
    if energies.len() < 3 {
        return 0.0;
    }
    let mean = energies.iter().copied().sum::<f32>() / energies.len() as f32;
    let mut peaks = 0usize;
    for i in 1..energies.len() - 1 {
        if energies[i] > energies[i - 1] && energies[i] > energies[i + 1] && energies[i] > mean {
            peaks += 1;
        }
    }
    peaks as f32 / duration
}

// ─── 7. Noise reduction via spectral subtraction stub ───

pub fn spectral_subtraction(
    audio: &AudioBuffer,
    noise_estimate: Option<&AudioBuffer>,
) -> AudioBuffer {
    let n = audio.samples.len().next_power_of_two();
    let mut input: Vec<f32> = audio.samples.iter().map(|&s| s as f32).collect();
    input.resize(n, 0.0);
    let fft_out = fft(&input);
    let mag: Vec<f32> = fft_out
        .iter()
        .map(|(r, i)| (r * r + i * i).sqrt())
        .collect();
    let phase: Vec<(f32, f32)> = fft_out
        .iter()
        .map(|(r, i)| {
            let m = (r * r + i * i).sqrt().max(1e-10);
            (*r / m, *i / m)
        })
        .collect();
    let noise_mag: Vec<f32> = if let Some(noise) = noise_estimate {
        let nn = noise.samples.len().next_power_of_two().max(n);
        let mut noise_input: Vec<f32> = noise.samples.iter().map(|&s| s as f32).collect();
        noise_input.resize(nn, 0.0);
        let noise_fft = fft(&noise_input);
        noise_fft
            .iter()
            .map(|(r, i)| (r * r + i * i).sqrt())
            .collect()
    } else {
        vec![100.0f32; mag.len()]
    };
    let mut cleaned = Vec::with_capacity(mag.len());
    for i in 0..mag.len() {
        let nm = noise_mag.get(i).copied().unwrap_or(0.0);
        let new_mag = (mag[i] - nm * 0.5).max(0.0);
        let (pr, pi) = phase[i];
        cleaned.push((pr * new_mag, pi * new_mag));
    }
    let mut restored = ifft(&cleaned);
    restored.truncate(audio.samples.len());
    let samples: Vec<i16> = restored
        .iter()
        .map(|&s| s.clamp(i16::MIN as f32, i16::MAX as f32) as i16)
        .collect();
    AudioBuffer {
        samples,
        sample_rate: audio.sample_rate,
    }
}

// ─── 8. Formant tracking (find peaks in spectrum) ───

pub fn track_formants(audio: &AudioBuffer) -> Vec<f32> {
    let n = audio.samples.len().next_power_of_two();
    let mut input: Vec<f32> = audio.samples.iter().map(|&s| s as f32).collect();
    input.resize(n, 0.0);
    let fft_out = fft(&input);
    let mag = magnitude_spectrum(&fft_out);
    let bin_width = audio.sample_rate as f32 / n as f32;
    let max_mag = mag.iter().copied().fold(0.0f32, f32::max);
    let threshold = max_mag * 0.1;
    let mut peaks = Vec::new();
    let half = mag.len() / 2;
    for i in 1..half {
        if mag[i] > mag[i - 1] && mag[i] > mag[i + 1] && mag[i] > threshold {
            peaks.push(i as f32 * bin_width);
        }
    }
    peaks.truncate(5);
    peaks
}

// ─── 9. Dynamic volume normalization ───

pub fn normalize_volume(audio: &AudioBuffer, target_rms: f32) -> AudioBuffer {
    if audio.samples.is_empty() {
        return audio.clone();
    }
    let sum_sq: f64 = audio.samples.iter().map(|&s| (s as f64) * (s as f64)).sum();
    let current_rms = ((sum_sq / audio.samples.len() as f64) as f32).sqrt();
    if current_rms < 1.0 {
        return audio.clone();
    }
    let gain = target_rms / current_rms;
    let samples: Vec<i16> = audio
        .samples
        .iter()
        .map(|&s| {
            let v = s as f32 * gain;
            v.clamp(i16::MIN as f32, i16::MAX as f32) as i16
        })
        .collect();
    AudioBuffer {
        samples,
        sample_rate: audio.sample_rate,
    }
}

// ─── 10. Audio fingerprinting (spectral hash) ───

pub fn audio_fingerprint(audio: &AudioBuffer) -> u64 {
    let n = audio.samples.len().next_power_of_two().max(64);
    let mut input: Vec<f32> = audio.samples.iter().map(|&s| s as f32).collect();
    input.resize(n, 0.0);
    let fft_out = fft(&input);
    let mag = magnitude_spectrum(&fft_out);
    let half = mag.len() / 2;
    let bands = 32usize;
    let band_size = half / bands;
    if band_size == 0 {
        return 0u64;
    }
    let mut energies = vec![0.0f32; bands];
    for b in 0..bands {
        let start = b * band_size;
        let end = start + band_size;
        energies[b] = mag[start..end.min(mag.len())].iter().copied().sum();
    }
    let mut hash = 0u64;
    for b in 1..bands {
        if energies[b] > energies[b - 1] {
            hash |= 1u64 << (b - 1);
        }
    }
    hash
}
