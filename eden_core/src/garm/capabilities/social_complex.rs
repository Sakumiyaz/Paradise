// EDEN GARM SocialComplex — Interaccion social profunda.
// 100% Rust puro, 0 LLM, 0 red.
//
// Sistemas:
//   - Reputacion: historial de cooperacion vs defeccion por agente
//   - Cultura: normas compartidas (reglas de conducta) con evolucion
//   - ToM de 2o orden: "A cree que B quiere X"
//   - Negociacion: oferta / contraoferta con utilidad esperada

use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    Cooperate,
    Defect,
    Negotiate,
    Observe,
}

#[derive(Clone, Debug)]
pub struct Reputation {
    pub agent_id: String,
    pub cooperations: u64,
    pub defections: u64,
    pub trust_score: f32, // 0..1
}

impl Reputation {
    pub fn new(agent_id: &str) -> Self {
        Reputation {
            agent_id: agent_id.to_string(),
            cooperations: 0,
            defections: 0,
            trust_score: 0.5,
        }
    }
    pub fn update(&mut self, action: Action) {
        match action {
            Action::Cooperate => {
                self.cooperations += 1;
            }
            Action::Defect => {
                self.defections += 1;
            }
            _ => {}
        }
        let total = self.cooperations + self.defections;
        if total > 0 {
            self.trust_score = (self.cooperations as f32 + 1.0) / (total as f32 + 2.0);
        }
    }
}

#[derive(Clone, Debug)]
pub struct Norm {
    pub label: String,
    pub strength: f32, // how strongly the group holds this norm
    pub violations: u64,
    pub compliances: u64,
}

#[derive(Clone, Debug)]
pub struct SecondOrderBelief {
    pub observer: String,
    pub subject: String,
    pub target: String,
    pub belief: String, // e.g. "wants to cooperate"
    pub confidence: f32,
}

#[derive(Clone, Debug)]
pub struct Offer {
    pub from: String,
    pub to: String,
    pub give: String,
    pub receive: String,
    pub utility_for_sender: f32,
    pub utility_for_receiver: f32,
}

#[derive(Clone, Debug)]
pub struct SocialComplex {
    pub reputations: HashMap<String, Reputation>,
    pub norms: Vec<Norm>,
    pub second_order_beliefs: Vec<SecondOrderBelief>,
    pub pending_offers: Vec<Offer>,
    pub n_interactions: u64,
    pub n_negotiations: u64,
}

impl SocialComplex {
    pub fn new() -> Self {
        SocialComplex {
            reputations: HashMap::new(),
            norms: vec![
                Norm {
                    label: "cooperar cuando se confia".to_string(),
                    strength: 0.8,
                    violations: 0,
                    compliances: 0,
                },
                Norm {
                    label: "honrar promesas".to_string(),
                    strength: 0.9,
                    violations: 0,
                    compliances: 0,
                },
                Norm {
                    label: "no danar sin razon".to_string(),
                    strength: 0.7,
                    violations: 0,
                    compliances: 0,
                },
            ],
            second_order_beliefs: Vec::new(),
            pending_offers: Vec::new(),
            n_interactions: 0,
            n_negotiations: 0,
        }
    }

    pub fn interact(&mut self, agent_a: &str, agent_b: &str, action_a: Action, action_b: Action) {
        self.n_interactions += 1;
        let rep_a = self
            .reputations
            .entry(agent_a.to_string())
            .or_insert_with(|| Reputation::new(agent_a));
        rep_a.update(action_a.clone());
        let rep_b = self
            .reputations
            .entry(agent_b.to_string())
            .or_insert_with(|| Reputation::new(agent_b));
        rep_b.update(action_b.clone());
        // Record second-order beliefs: A thinks B wants...
        let belief = match action_b {
            Action::Cooperate => "wants to cooperate",
            Action::Defect => "wants to defect",
            Action::Negotiate => "wants to negotiate",
            Action::Observe => "is uncertain",
        };
        self.second_order_beliefs.push(SecondOrderBelief {
            observer: agent_a.to_string(),
            subject: agent_b.to_string(),
            target: agent_a.to_string(),
            belief: belief.to_string(),
            confidence: rep_b.trust_score,
        });
        // Check norms
        for norm in self.norms.iter_mut() {
            match norm.label.as_str() {
                "cooperar cuando se confia" => {
                    if action_a == Action::Defect && rep_b.trust_score > 0.6 {
                        norm.violations += 1;
                    } else if action_a == Action::Cooperate && rep_b.trust_score > 0.6 {
                        norm.compliances += 1;
                    }
                }
                _ => {}
            }
        }
    }

    pub fn negotiate(&mut self, from: &str, to: &str, give: &str, receive: &str) -> Option<Offer> {
        self.n_negotiations += 1;
        let trust = self
            .reputations
            .get(to)
            .map(|r| r.trust_score)
            .unwrap_or(0.5);
        let util_sender = trust * 0.8 + 0.2;
        let util_receiver = trust * 0.6 + 0.3;
        let offer = Offer {
            from: from.to_string(),
            to: to.to_string(),
            give: give.to_string(),
            receive: receive.to_string(),
            utility_for_sender: util_sender,
            utility_for_receiver: util_receiver,
        };
        if util_receiver > 0.5 {
            self.pending_offers.push(offer.clone());
            Some(offer)
        } else {
            None
        }
    }

    pub fn report_reputations(&self) -> String {
        if self.reputations.is_empty() {
            return "Sin reputaciones aun".to_string();
        }
        let mut out = "Reputaciones:\n".to_string();
        for (_, r) in self.reputations.iter() {
            out.push_str(&format!(
                "  {} | trust={:.2} | coop={} | defect={}\n",
                r.agent_id, r.trust_score, r.cooperations, r.defections
            ));
        }
        out
    }

    pub fn report_norms(&self) -> String {
        let mut out = "Normas culturales:\n".to_string();
        for n in &self.norms {
            let total = n.compliances + n.violations;
            let compliance_rate = if total > 0 {
                n.compliances as f32 / total as f32
            } else {
                0.0
            };
            out.push_str(&format!(
                "  '{}' | strength={:.2} | compliance={:.1}%\n",
                n.label,
                n.strength,
                compliance_rate * 100.0
            ));
        }
        out
    }

    pub fn status(&self) -> String {
        format!(
            "SocialComplex | agents={} | interactions={} | negotiations={} | beliefs={} | norms={}",
            self.reputations.len(),
            self.n_interactions,
            self.n_negotiations,
            self.second_order_beliefs.len(),
            self.norms.len(),
        )
    }
}
