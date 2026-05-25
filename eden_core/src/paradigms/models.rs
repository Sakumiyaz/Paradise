// paradigms/models.rs — 6 modelos con techo absoluto (señal única, 0 redundancia)

use super::autograd::Linear;

/// 1. EDGE SCORER: aprende confianza real de edges (señal: edge sobrevive pruning)
pub struct EdgeScorer {
    pub m: Linear,
}
impl EdgeScorer {
    pub fn new() -> Self {
        EdgeScorer {
            m: Linear::new(3, 1),
        }
    }
    pub fn score(&mut self, f: &[f32]) -> f32 {
        self.m.forward(f)[0].clamp(0.1, 0.99)
    }
    pub fn train(&mut self, f: &[f32], survived: f32, lr: f32) -> f32 {
        let p = self.m.forward(f);
        self.m.backward(f, &p, &[survived], lr)
    }
}

/// 2. EMOTION: aprende dinámica emocional (señal: transición real de estado)
pub struct EmotionModel {
    pub m: Linear,
}
impl EmotionModel {
    pub fn new() -> Self {
        EmotionModel {
            m: Linear::new(5, 3),
        }
    }
    pub fn predict(&mut self, f: &[f32]) -> Vec<f32> {
        self.m.forward(f)
    }
    pub fn train(&mut self, f: &[f32], target: &[f32], lr: f32) -> f32 {
        let p = self.m.forward(f);
        self.m.backward(f, &p, target, lr)
    }
}

/// 3. SLEEP TRIGGER: aprende cuándo dormir (señal: edges compactados vs tiempo dormido)
pub struct SleepTrigger {
    pub m: Linear,
}
impl SleepTrigger {
    pub fn new() -> Self {
        SleepTrigger {
            m: Linear::new(3, 1),
        }
    }
    pub fn should_sleep(&mut self, f: &[f32]) -> f32 {
        self.m.forward(f)[0]
    }
    pub fn train(&mut self, f: &[f32], efficiency: f32, lr: f32) -> f32 {
        let p = self.m.forward(f);
        self.m.backward(f, &p, &[efficiency], lr)
    }
}

/// 4. DEATH ORACLE: aprende cuándo morir (señal: calidad post-muerte vs pre-muerte)
pub struct DeathOracle {
    pub m: Linear,
}
impl DeathOracle {
    pub fn new() -> Self {
        DeathOracle {
            m: Linear::new(4, 1),
        }
    }
    pub fn time_to_die(&mut self, f: &[f32]) -> f32 {
        self.m.forward(f)[0]
    }
    pub fn train(&mut self, f: &[f32], quality_ratio: f32, lr: f32) -> f32 {
        let p = self.m.forward(f);
        self.m.backward(f, &p, &[quality_ratio], lr)
    }
}

/// 5. CRAWL PICKER: aprende qué categoría maximiza crecimiento
pub struct CrawlPicker {
    pub m: Linear,
}
impl CrawlPicker {
    pub fn new() -> Self {
        CrawlPicker {
            m: Linear::new(3, 1),
        }
    }
    pub fn score(&mut self, f: &[f32]) -> f32 {
        self.m.forward(f)[0]
    }
    pub fn train(&mut self, f: &[f32], growth: f32, lr: f32) -> f32 {
        let p = self.m.forward(f);
        self.m.backward(f, &p, &[growth], lr)
    }
}

/// 6. WARDEN: aprende cuándo pausar (señal: peligro bajó después de pausar)
pub struct WardenDetector {
    pub m: Linear,
}
impl WardenDetector {
    pub fn new() -> Self {
        WardenDetector {
            m: Linear::new(3, 1),
        }
    }
    pub fn should_pause(&mut self, f: &[f32]) -> f32 {
        self.m.forward(f)[0]
    }
    pub fn train(&mut self, f: &[f32], danger_dropped: f32, lr: f32) -> f32 {
        let p = self.m.forward(f);
        self.m.backward(f, &p, &[danger_dropped], lr)
    }
}
