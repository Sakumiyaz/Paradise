// EDEN GARM — Temporal Hierarchy
// Niveles: frames -> events -> episodes -> schemas -> eras
// Compresión temporal recursiva: frames similares se agrupan en eventos,
// eventos en episodios, episodios en schemas (scripts), schemas en eras.

use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct TemporalHierarchy {
    pub frames: Vec<Vec<f32>>,               // raw percepts
    pub events: Vec<(u64, Vec<f32>)>,        // (start_tick, summary_vector)
    pub episodes: Vec<(u64, u64, Vec<f32>)>, // (start, end, summary)
    pub schemas: HashMap<String, Vec<f32>>,  // label -> prototype vector
    pub eras: Vec<(u64, String)>,            // (tick, era_label)
    pub n_compressions: u64,
}

impl TemporalHierarchy {
    pub fn new() -> Self {
        TemporalHierarchy {
            frames: Vec::new(),
            events: Vec::new(),
            episodes: Vec::new(),
            schemas: HashMap::new(),
            eras: Vec::new(),
            n_compressions: 0,
        }
    }

    /// Push a raw frame.
    pub fn push_frame(&mut self, frame: &[f32], tick: u64) {
        self.frames.push(frame.to_vec());
        // Compress frames -> events every 10 frames
        if self.frames.len() % 10 == 0 {
            self.compress_frames_to_event(tick);
        }
        // Compress events -> episodes every 50 events
        if self.events.len() % 50 == 0 && !self.events.is_empty() {
            self.compress_events_to_episode(tick);
        }
        // Compress episodes -> schema every 100 episodes
        if self.episodes.len() % 100 == 0 && !self.episodes.is_empty() {
            self.compress_episodes_to_schema();
        }
    }

    fn compress_frames_to_event(&mut self, tick: u64) {
        let n = self.frames.len();
        if n < 10 {
            return;
        }
        let start = n - 10;
        let mut summary = vec![0.0f32; self.frames[0].len()];
        for i in start..n {
            for j in 0..summary.len() {
                summary[j] += self.frames[i][j];
            }
        }
        for j in 0..summary.len() {
            summary[j] /= 10.0;
        }
        self.events.push((tick, summary));
        self.n_compressions += 1;
    }

    fn compress_events_to_episode(&mut self, tick: u64) {
        let n = self.events.len();
        if n < 50 {
            return;
        }
        let start = self.events[n - 50].0;
        let mut summary = vec![0.0f32; self.events[0].1.len()];
        for i in (n - 50)..n {
            for j in 0..summary.len() {
                summary[j] += self.events[i].1[j];
            }
        }
        for j in 0..summary.len() {
            summary[j] /= 50.0;
        }
        self.episodes.push((start, tick, summary));
        self.n_compressions += 1;
    }

    fn compress_episodes_to_schema(&mut self) {
        let n = self.episodes.len();
        if n < 100 {
            return;
        }
        let mut summary = vec![0.0f32; self.episodes[0].2.len()];
        for i in (n - 100)..n {
            for j in 0..summary.len() {
                summary[j] += self.episodes[i].2[j];
            }
        }
        for j in 0..summary.len() {
            summary[j] /= 100.0;
        }
        let label = format!("schema_{}", self.schemas.len());
        self.schemas.insert(label, summary);
        self.n_compressions += 1;
    }

    pub fn status(&self) -> String {
        format!(
            "TempHier | frames={} events={} episodes={} schemas={} eras={} | compressions={}",
            self.frames.len(),
            self.events.len(),
            self.episodes.len(),
            self.schemas.len(),
            self.eras.len(),
            self.n_compressions
        )
    }
}
