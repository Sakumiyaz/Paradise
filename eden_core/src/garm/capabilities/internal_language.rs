// EDEN GARM Internal Language (Mentalese)
// Sequences of activated concepts form "thoughts" that can be reasoned about
// via the morphogenesis transition graph.

pub struct Thought {
    pub concept_ids: Vec<u64>,
    pub tick: u64,
    pub source: &'static str, // "perception", "imagination", "memory"
}

pub struct InternalLanguage {
    pub thoughts: Vec<Thought>,
    pub max_thoughts: usize,
}

impl InternalLanguage {
    pub fn new() -> Self {
        InternalLanguage {
            thoughts: Vec::with_capacity(500),
            max_thoughts: 500,
        }
    }

    pub fn encode_perception(&mut self, concept_ids: Vec<u64>, tick: u64) {
        self.thoughts.push(Thought {
            concept_ids,
            tick,
            source: "perception",
        });
        if self.thoughts.len() > self.max_thoughts {
            self.thoughts.remove(0);
        }
    }

    pub fn encode_imagination(&mut self, concept_ids: Vec<u64>, tick: u64) {
        self.thoughts.push(Thought {
            concept_ids,
            tick,
            source: "imagination",
        });
        if self.thoughts.len() > self.max_thoughts {
            self.thoughts.remove(0);
        }
    }

    /// Return thoughts that contain a given concept.
    pub fn thoughts_with(&self, concept_id: u64) -> Vec<&Thought> {
        self.thoughts
            .iter()
            .filter(|t| t.concept_ids.contains(&concept_id))
            .collect()
    }

    pub fn status(&self) -> String {
        format!(
            "Lang | thoughts: {} | recent_perception: {} | recent_imagination: {}",
            self.thoughts.len(),
            self.thoughts
                .iter()
                .rev()
                .take(10)
                .filter(|t| t.source == "perception")
                .count(),
            self.thoughts
                .iter()
                .rev()
                .take(10)
                .filter(|t| t.source == "imagination")
                .count(),
        )
    }
}
