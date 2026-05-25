// EDEN GARM — Logic Reasoning (formal deduction)
// Sistema de reglas horn + modus ponens + resolución unitaria.
// Permite inferencias garantizadas, no solo heurísticas.

#[derive(Clone, Debug)]
pub struct LogicReasoning {
    pub rules: Vec<Rule>,
    pub facts: Vec<Fact>,
    pub n_inferences: u64,
    pub last_conclusion: String,
}

#[derive(Clone, Debug)]
pub struct Fact {
    pub predicate: String,
    pub args: Vec<String>,
    pub truth: bool,
}

#[derive(Clone, Debug)]
pub struct Rule {
    pub premises: Vec<Fact>,
    pub conclusion: Fact,
}

impl LogicReasoning {
    pub fn new() -> Self {
        LogicReasoning {
            rules: Vec::new(),
            facts: Vec::new(),
            n_inferences: 0,
            last_conclusion: String::new(),
        }
    }

    pub fn add_fact(&mut self, predicate: &str, args: &[&str], truth: bool) {
        self.facts.push(Fact {
            predicate: predicate.to_string(),
            args: args.iter().map(|s| s.to_string()).collect(),
            truth,
        });
    }

    pub fn add_rule(
        &mut self,
        premises: &[(String, Vec<String>)],
        conclusion: (String, Vec<String>),
    ) {
        let premises_vec: Vec<Fact> = premises
            .iter()
            .map(|(p, a)| Fact {
                predicate: p.clone(),
                args: a.clone(),
                truth: true,
            })
            .collect();
        self.rules.push(Rule {
            premises: premises_vec,
            conclusion: Fact {
                predicate: conclusion.0,
                args: conclusion.1,
                truth: true,
            },
        });
    }

    /// Forward chaining: apply modus ponens until saturation.
    pub fn infer(&mut self) -> Vec<Fact> {
        let mut new_facts = Vec::new();
        let mut changed = true;
        while changed {
            changed = false;
            for rule in &self.rules {
                let all_premises_true = rule.premises.iter().all(|prem| {
                    self.facts
                        .iter()
                        .any(|f| f.predicate == prem.predicate && f.args == prem.args && f.truth)
                });
                if all_premises_true {
                    let already_known = self.facts.iter().any(|f| {
                        f.predicate == rule.conclusion.predicate && f.args == rule.conclusion.args
                    });
                    if !already_known {
                        new_facts.push(rule.conclusion.clone());
                        self.facts.push(rule.conclusion.clone());
                        changed = true;
                        self.n_inferences += 1;
                    }
                }
            }
        }
        if let Some(last) = new_facts.last() {
            self.last_conclusion = format!("{}({})", last.predicate, last.args.join(","));
        }
        new_facts
    }

    pub fn status(&self) -> String {
        format!(
            "Logic | facts={} | rules={} | inferences={} | last='{}'",
            self.facts.len(),
            self.rules.len(),
            self.n_inferences,
            self.last_conclusion
        )
    }
}
