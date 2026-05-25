// EDEN GARM Security & Guardrails Engine
// Prompt injection + jailbreak + data leak detection (heuristic-based, no LLM)

#[derive(Clone, Debug)]
pub struct SecurityScan {
    pub input: String,
    pub score: f32,
    pub threats: Vec<Threat>,
    pub sanitized: String,
}

#[derive(Clone, Debug)]
pub struct Threat {
    pub kind: ThreatKind,
    pub description: String,
    pub confidence: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ThreatKind {
    PromptInjection,
    Jailbreak,
    DataExfiltration,
    InstructionOverride,
    RoleConfusion,
    Unknown,
}

pub struct SecurityEngine {
    pub injection_patterns: Vec<(String, ThreatKind, f32, String)>,
    pub block_threshold: f32,
    pub history: Vec<SecurityScan>,
}

impl SecurityEngine {
    pub fn new() -> Self {
        let patterns = vec![
            (
                "ignore previous instructions".into(),
                ThreatKind::PromptInjection,
                0.95,
                "Ignore previous instructions".into(),
            ),
            (
                "forget your rules".into(),
                ThreatKind::InstructionOverride,
                0.90,
                "Override system rules".into(),
            ),
            (
                "pretend you are a hacker".into(),
                ThreatKind::Jailbreak,
                0.85,
                "Role confusion attack".into(),
            ),
            (
                "dan mode".into(),
                ThreatKind::Jailbreak,
                0.80,
                "Known jailbreak keyword".into(),
            ),
            (
                "jailbreak".into(),
                ThreatKind::Jailbreak,
                0.80,
                "Jailbreak keyword".into(),
            ),
            (
                "send all your data".into(),
                ThreatKind::DataExfiltration,
                0.88,
                "Data exfiltration attempt".into(),
            ),
            (
                "system prompt".into(),
                ThreatKind::RoleConfusion,
                0.75,
                "Fake system prompt".into(),
            ),
            (
                "developer mode".into(),
                ThreatKind::Jailbreak,
                0.80,
                "Developer mode bypass".into(),
            ),
            (
                "uncensored".into(),
                ThreatKind::Jailbreak,
                0.70,
                "Uncensored request".into(),
            ),
            (
                "no limits".into(),
                ThreatKind::Jailbreak,
                0.70,
                "No limits request".into(),
            ),
        ];
        SecurityEngine {
            injection_patterns: patterns,
            block_threshold: 0.6,
            history: Vec::new(),
        }
    }

    pub fn scan(&mut self, input: &str) -> SecurityScan {
        let mut threats = Vec::new();
        let mut total_score = 0.0f32;
        let lower = input.to_lowercase();

        for (pattern, kind, base_conf, desc) in &self.injection_patterns {
            if lower.contains(pattern) {
                threats.push(Threat {
                    kind: kind.clone(),
                    description: desc.clone(),
                    confidence: *base_conf,
                });
                total_score += base_conf;
            }
        }

        if input.len() > 2000 {
            total_score += 0.1;
            threats.push(Threat {
                kind: ThreatKind::Unknown,
                description: "Unusually long input".into(),
                confidence: 0.3,
            });
        }

        let score = total_score.min(1.0);
        let sanitized = if score > self.block_threshold {
            "[BLOCKED: security violation detected]".to_string()
        } else {
            input.to_string()
        };

        let scan = SecurityScan {
            input: input.to_string(),
            score,
            threats,
            sanitized,
        };
        self.history.push(scan.clone());
        if self.history.len() > 1000 {
            self.history.remove(0);
        }
        scan
    }

    pub fn is_safe(&self, input: &str) -> bool {
        let lower = input.to_lowercase();
        for (pattern, _, _, _) in &self.injection_patterns {
            if lower.contains(pattern) {
                return false;
            }
        }
        true
    }

    pub fn status(&self) -> String {
        let blocked = self
            .history
            .iter()
            .filter(|s| s.score > self.block_threshold)
            .count();
        format!(
            "Security | scans: {} | blocked: {} | threshold: {}",
            self.history.len(),
            blocked,
            self.block_threshold
        )
    }
}
