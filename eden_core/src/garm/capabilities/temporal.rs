// EDEN GARM Temporal — Genuine time awareness, episodic memory, and short-term planning
// The system knows when things happened, predicts when they will recur, and schedules future intents.

#[derive(Clone, Debug)]
pub struct RichEpisode {
    pub tick: u64,
    pub timestamp_sec: u64,
    pub intent_label: String,
    pub concept_id: u64,
    pub input_snapshot: String, // first 100 chars of raw input
    pub mood_valence: f32,
    pub mood_arousal: f32,
    pub prediction_error: f32,
    pub actions_taken: Vec<String>,
    pub concepts_activated: Vec<u64>,
}

pub struct EventLog {
    pub events: Vec<RichEpisode>,
    pub planned: Vec<(u64, String)>, // (deadline_tick, intent_label)
    pub max_events: usize,
}

impl EventLog {
    pub fn new() -> Self {
        EventLog {
            events: Vec::with_capacity(1000),
            planned: Vec::new(),
            max_events: 2000,
        }
    }

    pub fn log(
        &mut self,
        tick: u64,
        timestamp_sec: u64,
        intent_label: &str,
        concept_id: u64,
        input_snapshot: &str,
        mood_valence: f32,
        mood_arousal: f32,
        prediction_error: f32,
        actions_taken: Vec<String>,
        concepts_activated: Vec<u64>,
    ) {
        let snapshot: String = input_snapshot.chars().take(100).collect();
        self.events.push(RichEpisode {
            tick,
            timestamp_sec,
            intent_label: intent_label.to_string(),
            concept_id,
            input_snapshot: snapshot,
            mood_valence,
            mood_arousal,
            prediction_error,
            actions_taken,
            concepts_activated,
        });
        if self.events.len() > self.max_events {
            self.events.remove(0);
        }
    }

    /// Predict the next tick an intent of `label` will occur, using mean interval.
    pub fn predict_next_tick(&self, label: &str, current_tick: u64) -> Option<u64> {
        let ticks: Vec<u64> = self
            .events
            .iter()
            .filter(|e| e.intent_label == label)
            .map(|e| e.tick)
            .collect();
        if ticks.len() < 2 {
            return None;
        }
        let mut intervals = Vec::new();
        for w in ticks.windows(2) {
            intervals.push(w[1] - w[0]);
        }
        let mean_interval: f32 = intervals.iter().sum::<u64>() as f32 / intervals.len() as f32;
        let last = *ticks.last()?;
        let next = last + mean_interval as u64;
        if next > current_tick {
            Some(next)
        } else {
            Some(current_tick + (mean_interval as u64).max(1))
        }
    }

    /// Schedule an intent to be executed at or after `deadline_tick`.
    pub fn plan(&mut self, deadline_tick: u64, intent_label: &str) {
        self.planned.push((deadline_tick, intent_label.to_string()));
    }

    /// Retrieve and remove due plans.
    pub fn due_plans(&mut self, current_tick: u64) -> Vec<String> {
        let mut due = Vec::new();
        let mut remaining = Vec::new();
        for (dt, label) in self.planned.drain(..) {
            if dt <= current_tick {
                due.push(label);
            } else {
                remaining.push((dt, label));
            }
        }
        self.planned = remaining;
        due
    }

    /// Number of events in the last N ticks.
    pub fn recent_event_count(&self, current_tick: u64, window: u64) -> usize {
        self.events
            .iter()
            .filter(|e| e.tick + window >= current_tick)
            .count()
    }

    pub fn status(&self) -> String {
        format!(
            "Temporal | episodes: {} | planned: {}",
            self.events.len(),
            self.planned.len()
        )
    }
}
