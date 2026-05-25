use std::collections::HashMap;

pub struct UserModel {
    pub intent_history: Vec<String>,
    pub intent_freq: HashMap<String, u32>,
    pub last_emotion: String,
    pub preferred_action: String,
    pub max_history: usize,
    pub user_goal_stack: Vec<String>,
    pub user_confusion: f32,
    pub last_intents_window: Vec<String>,
}

impl UserModel {
    pub fn new() -> Self {
        UserModel {
            intent_history: Vec::with_capacity(1000),
            intent_freq: HashMap::new(),
            last_emotion: "neutral".to_string(),
            preferred_action: "unknown".to_string(),
            max_history: 1000,
            user_goal_stack: Vec::new(),
            user_confusion: 0.0,
            last_intents_window: Vec::new(),
        }
    }

    pub fn observe(&mut self, intent_label: &str, sentiment_score: f32) {
        self.intent_history.push(intent_label.to_string());
        if self.intent_history.len() > self.max_history {
            self.intent_history.remove(0);
        }
        self.last_intents_window.push(intent_label.to_string());
        if self.last_intents_window.len() > 5 {
            self.last_intents_window.remove(0);
        }
        let unique: std::collections::HashSet<_> = self.last_intents_window.iter().collect();
        self.user_confusion = unique.len() as f32 / 5.0;
        *self
            .intent_freq
            .entry(intent_label.to_string())
            .or_insert(0) += 1;
        self.last_emotion = if sentiment_score > 0.3 {
            "positive"
        } else if sentiment_score < -0.3 {
            "negative"
        } else {
            "neutral"
        }
        .to_string();
        // Update preferred action by frequency
        if let Some((best, _)) = self.intent_freq.iter().max_by_key(|(_, v)| *v) {
            self.preferred_action = best.clone();
        }
    }

    pub fn is_user_confused(&self) -> bool {
        self.user_confusion > 0.6
    }

    /// Predict the user's next intent based on Markov-like frequency of last 10 intents.
    pub fn predict_next_intent(&self) -> Option<String> {
        if self.is_user_confused() {
            return Some("clarify".to_string());
        }
        if self.intent_history.len() < 2 {
            return None;
        }
        let recent: Vec<String> =
            self.intent_history[self.intent_history.len().saturating_sub(10)..].to_vec();
        let mut transitions: HashMap<(String, String), u32> = HashMap::new();
        for w in recent.windows(2) {
            *transitions.entry((w[0].clone(), w[1].clone())).or_insert(0) += 1;
        }
        let last = recent.last()?;
        transitions
            .iter()
            .filter(|((a, _), _)| a == last)
            .max_by_key(|(_, c)| *c)
            .map(|((_, b), _)| b.clone())
    }

    pub fn status(&self) -> String {
        format!(
            "ToM | preferred={} | last_emotion={} | history={} | confusion={:.2}",
            self.preferred_action,
            self.last_emotion,
            self.intent_history.len(),
            self.user_confusion
        )
    }
}
