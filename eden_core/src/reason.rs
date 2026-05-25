//! Reason - Eden's Logical Processing Engine
//!
//! Reason handles deduction, induction, and abduction. It creates
//! logical chains from premises to conclusions.
#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(unused_imports)]
use std::io::Write;

use crate::membrain::{generate_id, rand_u64, NOW_MS};

/// Logical connective types
#[derive(Debug, Clone, PartialEq)]
pub enum Connective {
    And,
    Or,
    Not,
    Implies,
    Equivalent,
}

/// Inference rule types
#[derive(Debug, Clone, PartialEq)]
pub enum InferenceRule {
    ModusPonens,  // If P implies Q, and P is true, then Q is true
    ModusTollens, // If P implies Q, and Q is false, then P is false
    Syllogism,    // Chain of implications
    Abduction,    // Best explanation
    Induction,    // Generalize from cases
    Analogy,      // Similar to similar
}

/// Logical premise or fact
#[derive(Debug, Clone)]
pub struct Premise {
    pub id: String,
    pub content: Vec<u8>,
    pub truth_value: f64, // 0.0 = false, 1.0 = true
    pub confidence: f64,  // How sure we are
    pub source: String,
    pub birth_time: u64,
}

impl Premise {
    /// Create a new premise
    pub fn new(content: Vec<u8>, truth_value: f64, source: &str) -> Self {
        let id_data = format!("{}:{}", source, truth_value);
        Premise {
            id: generate_id(id_data.as_bytes()).to_string(),
            content,
            truth_value: truth_value.clamp(0.0, 1.0),
            confidence: 0.8,
            source: source.to_string(),
            birth_time: NOW_MS(),
        }
    }

    /// Check if premise is true enough
    pub fn is_true(&self, threshold: f64) -> bool {
        self.truth_value >= threshold && self.confidence >= threshold
    }

    /// Strengthen premise
    pub fn reinforce(&mut self, evidence: f64) {
        self.truth_value = (self.truth_value + evidence).min(1.0);
        self.confidence = (self.confidence + 0.01).min(0.99);
    }

    /// Weaken premise
    pub fn weaken(&mut self, counter_evidence: f64) {
        self.truth_value = (self.truth_value - counter_evidence).max(0.0);
        self.confidence = (self.confidence - 0.02).max(0.1);
    }
}

/// Logical implication
#[derive(Debug, Clone)]
pub struct Implication {
    pub id: String,
    pub antecedent: String, // Premise ID
    pub consequent: String, // Conclusion ID
    pub strength: f64,      // How strong the implication
    pub rule: InferenceRule,
}

impl Implication {
    /// Create new implication
    pub fn new(antecedent: String, consequent: String, rule: InferenceRule) -> Self {
        let id_data = format!("{}->{}:{:?}", antecedent, consequent, rule);
        Implication {
            id: generate_id(id_data.as_bytes()).to_string(),
            antecedent,
            consequent,
            strength: 0.8,
            rule,
        }
    }

    /// Apply modus ponens
    pub fn apply_modus_ponens(&self, antecedent_true: bool) -> Option<f64> {
        if self.rule == InferenceRule::ModusPonens && antecedent_true {
            Some(self.strength)
        } else {
            None
        }
    }

    /// Apply modus tollens
    pub fn apply_modus_tollens(&self, consequent_false: bool) -> Option<f64> {
        if self.rule == InferenceRule::ModusTollens && consequent_false {
            Some(self.strength)
        } else {
            None
        }
    }
}

/// Logical chain of reasoning
#[derive(Debug, Clone)]
pub struct ReasoningChain {
    pub id: String,
    pub premises: Vec<String>,     // Premise IDs
    pub implications: Vec<String>, // Implication IDs
    pub conclusion: Option<String>,
    pub confidence: f64,
    pub depth: usize,
    pub birth_time: u64,
}

impl ReasoningChain {
    /// Create new reasoning chain
    pub fn new(depth: usize) -> Self {
        let id_data = format!("chain:{}", depth);
        ReasoningChain {
            id: generate_id(id_data.as_bytes()).to_string(),
            premises: Vec::new(),
            implications: Vec::new(),
            conclusion: None,
            confidence: 0.5,
            depth,
            birth_time: NOW_MS(),
        }
    }

    /// Add premise to chain
    pub fn add_premise(&mut self, premise_id: String) {
        self.premises.push(premise_id);
        self.recalculate_confidence();
    }

    /// Add implication
    pub fn add_implication(&mut self, implication_id: String) {
        self.implications.push(implication_id);
        self.recalculate_confidence();
    }

    /// Recalculate chain confidence
    fn recalculate_confidence(&mut self) {
        if self.premises.is_empty() && self.implications.is_empty() {
            self.confidence = 0.0;
            return;
        }

        let count = (self.premises.len() + self.implications.len()) as f64;
        self.confidence = (1.0 / count).min(0.9);
    }

    /// Reach conclusion
    pub fn conclude(&mut self, conclusion_id: String) {
        self.conclusion = Some(conclusion_id);
        self.confidence = (self.confidence + 0.2).min(0.99);
    }

    /// Get chain strength
    pub fn strength(&self) -> f64 {
        if self.premises.is_empty() {
            return 0.0;
        }
        self.confidence
    }

    /// Age of reasoning
    pub fn age(&self) -> u64 {
        NOW_MS() - self.birth_time
    }
}

/// Reason engine
#[derive(Debug, Clone)]
pub struct ReasonEngine {
    pub premises: Vec<Premise>,
    pub implications: Vec<Implication>,
    pub chains: Vec<ReasoningChain>,
    pub active_chain: Option<String>,
    pub max_depth: usize,
}

impl ReasonEngine {
    /// Create new reason engine
    pub fn new() -> Self {
        ReasonEngine {
            premises: Vec::new(),
            implications: Vec::new(),
            chains: Vec::new(),
            active_chain: None,
            max_depth: 10,
        }
    }

    /// Add premise
    pub fn assert(&mut self, content: Vec<u8>, truth_value: f64, source: &str) -> String {
        let premise = Premise::new(content, truth_value, source);
        let id = premise.id.clone();
        self.premises.push(premise);
        id
    }

    /// Add implication
    pub fn imply(&mut self, antecedent: String, consequent: String, rule: InferenceRule) -> String {
        let implication = Implication::new(antecedent, consequent, rule);
        let id = implication.id.clone();
        self.implications.push(implication);
        id
    }

    /// Start new reasoning chain
    pub fn begin_chain(&mut self, depth: usize) -> String {
        let chain = ReasoningChain::new(depth.min(self.max_depth));
        let id = chain.id.clone();
        self.chains.push(chain);
        self.active_chain = Some(id.clone());
        id
    }

    /// Add to active chain
    pub fn extend_chain(&mut self, element_id: &str, is_premise: bool) {
        if let Some(ref chain_id) = self.active_chain {
            for chain in &mut self.chains {
                if chain.id == *chain_id {
                    if is_premise {
                        chain.add_premise(element_id.to_string());
                    } else {
                        chain.add_implication(element_id.to_string());
                    }
                    break;
                }
            }
        }
    }

    /// Draw conclusion on active chain
    pub fn conclude(&mut self, conclusion: Vec<u8>, truth_value: f64) -> Option<String> {
        if let Some(ref chain_id) = self.active_chain {
            let chain_id_clone = chain_id.clone();

            // First, assert the conclusion to get its ID
            let conclusion_id = self.assert(conclusion, truth_value, "reasoning");

            // Then find the chain and conclude
            if let Some(chain) = self.chains.iter_mut().find(|c| c.id == chain_id_clone) {
                chain.conclude(conclusion_id.clone());
                return Some(conclusion_id);
            }
        }
        None
    }

    /// Deduce from implications
    pub fn deduce(&self, premise_id: &str) -> Vec<(String, f64)> {
        let mut results = Vec::new();

        for implication in &self.implications {
            if implication.antecedent == premise_id {
                results.push((implication.consequent.clone(), implication.strength));
            }
        }

        results
    }

    /// Find best explanation (abduction)
    pub fn abduce(&self, observation: &[u8], hypotheses: &[Vec<u8>]) -> Option<(Vec<u8>, f64)> {
        if hypotheses.is_empty() {
            return None;
        }

        // Score each hypothesis
        let mut best: Option<(Vec<u8>, f64)> = None;

        for hypothesis in hypotheses {
            let similarity = self.compute_similarity(observation, hypothesis);
            let score = similarity * (0.5 + rand_u64() as f64 / 100.0); // Add some randomness

            match &best {
                None => best = Some((hypothesis.clone(), score)),
                Some((_, best_score)) if score > *best_score => {
                    best = Some((hypothesis.clone(), score));
                }
                _ => {}
            }
        }

        best
    }

    /// Compute similarity between two byte sequences
    fn compute_similarity(&self, a: &[u8], b: &[u8]) -> f64 {
        if a.is_empty() || b.is_empty() {
            return 0.0;
        }

        let min_len = a.len().min(b.len());
        let mut matches = 0;

        for i in 0..min_len {
            if a[i] == b[i] {
                matches += 1;
            }
        }

        matches as f64 / min_len as f64
    }

    /// Get premises matching pattern
    pub fn find_premises(&self, pattern: &[u8]) -> Vec<&Premise> {
        self.premises
            .iter()
            .filter(|p| self.compute_similarity(&p.content, pattern) > 0.5)
            .collect()
    }

    /// Prune weak chains
    pub fn prune_weak_chains(&mut self, threshold: f64) {
        self.chains.retain(|c| c.confidence >= threshold);
    }

    /// Clean old premises
    pub fn decay_premises(&mut self, max_age: u64) {
        let now = NOW_MS();
        self.premises.retain(|p| now - p.birth_time < max_age);
    }
}

impl Default for ReasonEngine {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// ADVANCED REASONING - Abduction, Analogy, Counterfactuals
// ============================================================================

use std::collections::HashMap;

/// Deep inference chain analyzer
pub struct DeepInferenceChain {
    max_depth: usize,
    branching_factor: usize,
    chain_history: Vec<InferenceChain>,
}

#[derive(Debug, Clone)]
pub struct InferenceChain {
    pub chain_id: String,
    pub premises: Vec<String>,
    pub intermediate_conclusions: Vec<String>,
    pub final_conclusion: String,
    pub depth: usize,
    pub confidence: f32,
    pub path_type: ChainType,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChainType {
    Deductive,
    Inductive,
    Abductive,
    Analogical,
}

impl DeepInferenceChain {
    pub fn new() -> Self {
        DeepInferenceChain {
            max_depth: 10,
            branching_factor: 3,
            chain_history: Vec::new(),
        }
    }

    /// Analyzes deep inference chain
    pub fn analyze_chain(&mut self, initial_premises: &[String]) -> Vec<InferenceChain> {
        let mut chains = Vec::new();

        for premise in initial_premises {
            let chain = self.expand_chain(premise, 0);
            chains.push(chain);
        }

        self.chain_history.extend(chains.clone());
        chains
    }

    fn expand_chain(&self, current: &str, depth: usize) -> InferenceChain {
        let mut chain = InferenceChain {
            chain_id: generate_id(current.as_bytes()).to_string(),
            premises: vec![current.to_string()],
            intermediate_conclusions: Vec::new(),
            final_conclusion: current.to_string(),
            depth,
            confidence: 1.0,
            path_type: ChainType::Deductive,
        };

        if depth >= self.max_depth {
            return chain;
        }

        // Simulate expansion (would connect to actual knowledge base)
        chain
            .intermediate_conclusions
            .push(format!("{} -> expanded", current));
        chain.final_conclusion = format!("{} (expanded)", current);
        chain.confidence = (1.0 - depth as f32 * 0.05).max(0.1);

        chain
    }

    /// Gets chains by type
    pub fn get_chains_by_type(&self, chain_type: ChainType) -> Vec<&InferenceChain> {
        self.chain_history
            .iter()
            .filter(|c| c.path_type == chain_type)
            .collect()
    }
}

impl Default for DeepInferenceChain {
    fn default() -> Self {
        Self::new()
    }
}

/// Abductive reasoning engine (inference to best explanation)
pub struct AbductionEngine {
    hypotheses: HashMap<String, Hypothesis>,
    observations: Vec<Observation>,
    best_explanation: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Hypothesis {
    pub hypothesis_id: String,
    pub content: String,
    pub explanatory_power: f32,
    pub simplicity: f32,
    pub scope: f32,
    pub probability: f32,
}

#[derive(Debug, Clone)]
pub struct Observation {
    pub observation_id: String,
    pub content: String,
    pub explained_by: Vec<String>,
}

impl AbductionEngine {
    pub fn new() -> Self {
        AbductionEngine {
            hypotheses: HashMap::new(),
            observations: Vec::new(),
            best_explanation: None,
        }
    }

    /// Adds observation
    pub fn add_observation(&mut self, observation: &str) -> String {
        let obs_id = generate_id(observation.as_bytes());
        self.observations.push(Observation {
            observation_id: obs_id.to_string(),
            content: observation.to_string(),
            explained_by: Vec::new(),
        });
        obs_id.to_string()
    }

    /// Generates hypothesis to explain observations
    pub fn generate_hypothesis(&mut self, explanation: &str) -> String {
        let hyp_id = generate_id(explanation.as_bytes());

        let hypothesis = Hypothesis {
            hypothesis_id: hyp_id.to_string(),
            content: explanation.to_string(),
            explanatory_power: 0.5,
            simplicity: 0.5,
            scope: 0.5,
            probability: 0.5,
        };

        self.hypotheses.insert(hyp_id.to_string(), hypothesis);
        hyp_id.to_string()
    }

    /// Evaluates hypothesis against observations
    pub fn evaluate_hypothesis(&mut self, hypothesis_id: &str) -> f32 {
        if let Some(hyp) = self.hypotheses.get_mut(hypothesis_id) {
            // Calculate IOI score: Explanatory Power + Simplicity + Scope
            hyp.probability = (hyp.explanatory_power + hyp.simplicity + hyp.scope) / 3.0;
            hyp.probability
        } else {
            0.0
        }
    }

    /// Finds best explanation for all observations
    pub fn find_best_explanation(&mut self) -> Option<String> {
        let mut best: Option<(String, f32)> = None;

        for (id, hyp) in &self.hypotheses {
            if best.is_none() || hyp.probability > best.as_ref().unwrap().1 {
                best = Some((id.clone(), hyp.probability));
            }
        }

        if let Some((id, _)) = best {
            self.best_explanation = Some(id.clone());
            return Some(id);
        }

        None
    }

    /// Updates hypothesis based on new evidence
    pub fn update_with_evidence(&mut self, hypothesis_id: &str, _evidence: &str) {
        if let Some(hyp) = self.hypotheses.get_mut(hypothesis_id) {
            hyp.explanatory_power = (hyp.explanatory_power + 0.1).min(1.0);
        }
    }
}

impl Default for AbductionEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Analogical reasoning engine
pub struct AnalogicalReasoning {
    source_domains: HashMap<String, Domain>,
    target_domains: HashMap<String, Domain>,
    mappings: Vec<Mapping>,
}

#[derive(Debug, Clone)]
pub struct Domain {
    pub domain_id: String,
    pub name: String,
    pub concepts: Vec<Concept>,
    pub relations: Vec<Relation>,
}

#[derive(Debug, Clone)]
pub struct Concept {
    pub concept_id: String,
    pub name: String,
    pub attributes: HashMap<String, f32>,
}

#[derive(Debug, Clone)]
pub struct Relation {
    pub relation_id: String,
    pub source: String,
    pub target: String,
    pub relation_type: String,
}

#[derive(Debug, Clone)]
pub struct Mapping {
    pub source_concept: String,
    pub target_concept: String,
    pub similarity: f32,
    pub confidence: f32,
}

impl AnalogicalReasoning {
    pub fn new() -> Self {
        AnalogicalReasoning {
            source_domains: HashMap::new(),
            target_domains: HashMap::new(),
            mappings: Vec::new(),
        }
    }

    /// Adds source domain
    pub fn add_source_domain(&mut self, domain: Domain) {
        self.source_domains.insert(domain.domain_id.clone(), domain);
    }

    /// Adds target domain
    pub fn add_target_domain(&mut self, domain: Domain) {
        self.target_domains.insert(domain.domain_id.clone(), domain);
    }

    /// Finds analogical mapping between domains
    pub fn find_mapping(&mut self, source_id: &str, target_id: &str) -> Vec<Mapping> {
        let mut mappings = Vec::new();

        if let (Some(source), Some(target)) = (
            self.source_domains.get(source_id),
            self.target_domains.get(target_id),
        ) {
            for src_concept in &source.concepts {
                for tgt_concept in &target.concepts {
                    let similarity = self.compute_concept_similarity(src_concept, tgt_concept);

                    if similarity > 0.5 {
                        mappings.push(Mapping {
                            source_concept: src_concept.name.clone(),
                            target_concept: tgt_concept.name.clone(),
                            similarity,
                            confidence: similarity,
                        });
                    }
                }
            }
        }

        self.mappings.extend(mappings.clone());
        mappings
    }

    fn compute_concept_similarity(&self, a: &Concept, b: &Concept) -> f32 {
        if a.attributes.is_empty() || b.attributes.is_empty() {
            return 0.0;
        }

        let mut matching = 0.0;
        for (key, val_a) in &a.attributes {
            if let Some(val_b) = b.attributes.get(key) {
                let diff = (val_a - val_b).abs();
                if diff < 0.2 {
                    matching += 1.0;
                }
            }
        }

        matching / a.attributes.len().max(b.attributes.len()) as f32
    }

    /// Transfers relation from source to target
    pub fn transfer_relation(&mut self, relation_id: &str, source_id: &str) -> Option<Relation> {
        if let Some(source) = self.source_domains.get(source_id) {
            if let Some(rel) = source
                .relations
                .iter()
                .find(|r| r.relation_id == relation_id)
            {
                return Some(Relation {
                    relation_id: format!("{}_transferred", rel.relation_id),
                    source: rel.source.clone(),
                    target: rel.target.clone(),
                    relation_type: rel.relation_type.clone(),
                });
            }
        }

        None
    }
}

impl Default for AnalogicalReasoning {
    fn default() -> Self {
        Self::new()
    }
}

/// Counterfactual thinking engine
pub struct CounterfactualEngine {
    scenarios: Vec<CounterfactualScenario>,
    actual_world: WorldState,
}

#[derive(Debug, Clone)]
pub struct CounterfactualScenario {
    pub scenario_id: String,
    pub antecedent: String,
    pub consequent: String,
    pub probability: f32,
    pub contrast_with_actual: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct WorldState {
    pub state_id: String,
    pub facts: HashMap<String, Fact>,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct Fact {
    pub content: String,
    pub truth_value: f32,
}

impl CounterfactualEngine {
    pub fn new() -> Self {
        CounterfactualEngine {
            scenarios: Vec::new(),
            actual_world: WorldState {
                state_id: "actual".to_string(),
                facts: HashMap::new(),
                timestamp: NOW_MS() as u64,
            },
        }
    }

    /// Creates counterfactual scenario
    pub fn create_scenario(&mut self, antecedent: &str, consequent: &str) -> String {
        let scenario_id = generate_id(format!("{}_{}", antecedent, consequent).as_bytes());

        let scenario = CounterfactualScenario {
            scenario_id: scenario_id.to_string(),
            antecedent: antecedent.to_string(),
            consequent: consequent.to_string(),
            probability: 0.5,
            contrast_with_actual: Vec::new(),
        };

        self.scenarios.push(scenario);
        scenario_id.to_string()
    }

    /// Evaluates counterfactual probability
    pub fn evaluate_probability(&mut self, scenario_id: &str) -> f32 {
        let (antecedent, consequent) = {
            let scenario = match self
                .scenarios
                .iter_mut()
                .find(|s| s.scenario_id == scenario_id)
            {
                Some(s) => s,
                None => return 0.0,
            };
            (scenario.antecedent.clone(), scenario.consequent.clone())
        };
        let probability = self.estimate_causal_probability(&antecedent, &consequent);
        if let Some(scenario) = self
            .scenarios
            .iter_mut()
            .find(|s| s.scenario_id == scenario_id)
        {
            scenario.probability = probability;
        }
        probability
    }

    fn estimate_causal_probability(&self, _antecedent: &str, _consequent: &str) -> f32 {
        // Would integrate with causal inference engine
        0.5
    }

    /// Finds contrast with actual world
    pub fn find_contrast(&mut self, scenario_id: &str) -> Vec<String> {
        let mut contrasts = Vec::new();

        if let Some(scenario) = self
            .scenarios
            .iter_mut()
            .find(|s| s.scenario_id == scenario_id)
        {
            for fact in self.actual_world.facts.values() {
                contrasts.push(format!(
                    "In actual world: {} (truth: {})",
                    fact.content, fact.truth_value
                ));
            }

            scenario.contrast_with_actual = contrasts.clone();
        }

        contrasts
    }

    /// Updates actual world state
    pub fn update_actual_world(&mut self, fact_key: &str, fact: Fact) {
        self.actual_world.facts.insert(fact_key.to_string(), fact);
        self.actual_world.timestamp = NOW_MS() as u64;
    }
}

impl Default for CounterfactualEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Bayesian uncertainty handler
pub struct BayesianReasoner {
    priors: HashMap<String, f32>,
    likelihoods: HashMap<(String, String), f32>,
    posteriors: HashMap<String, f32>,
}

impl BayesianReasoner {
    pub fn new() -> Self {
        BayesianReasoner {
            priors: HashMap::new(),
            likelihoods: HashMap::new(),
            posteriors: HashMap::new(),
        }
    }

    /// Sets prior probability
    pub fn set_prior(&mut self, hypothesis: &str, probability: f32) {
        self.priors.insert(hypothesis.to_string(), probability);
    }

    /// Sets likelihood P(evidence|hypothesis)
    pub fn set_likelihood(&mut self, hypothesis: &str, evidence: &str, likelihood: f32) {
        self.likelihoods
            .insert((hypothesis.to_string(), evidence.to_string()), likelihood);
    }

    /// Computes posterior using Bayes' theorem
    pub fn compute_posterior(&mut self, hypothesis: &str, evidence: &str) -> f32 {
        let prior = self.priors.get(hypothesis).copied().unwrap_or(0.5);
        let likelihood = self
            .likelihoods
            .get(&(hypothesis.to_string(), evidence.to_string()))
            .copied()
            .unwrap_or(0.5);

        // P(H|E) = P(E|H) * P(H) / P(E)
        let evidence_prob = self.compute_evidence_prob(evidence);

        if evidence_prob == 0.0 {
            return prior;
        }

        let posterior = (likelihood * prior) / evidence_prob;
        self.posteriors.insert(hypothesis.to_string(), posterior);
        posterior
    }

    fn compute_evidence_prob(&self, evidence: &str) -> f32 {
        let mut evidence_prob = 0.0;

        for (hyp, prior) in &self.priors {
            let likelihood = self
                .likelihoods
                .get(&(hyp.clone(), evidence.to_string()))
                .copied()
                .unwrap_or(0.5);
            evidence_prob += likelihood * prior;
        }

        evidence_prob.max(0.001)
    }

    /// Updates beliefs with new evidence
    pub fn update_beliefs(&mut self, evidence: &str) {
        let hypothesis_keys: Vec<_> = self.priors.keys().cloned().collect();
        for hypothesis in hypothesis_keys {
            self.compute_posterior(&hypothesis, evidence);
        }
    }
}

impl Default for BayesianReasoner {
    fn default() -> Self {
        Self::new()
    }
}

/// Fuzzy logic reasoner
pub struct FuzzyReasoner {
    fuzzy_sets: HashMap<String, FuzzySet>,
    rules: Vec<FuzzyRule>,
}

#[derive(Debug, Clone)]
pub struct FuzzySet {
    pub set_id: String,
    pub name: String,
    pub membership_function: MembershipType,
    pub parameters: Vec<f32>,
}

#[derive(Debug, Clone, Copy)]
pub enum MembershipType {
    Triangular,
    Trapezoidal,
    Gaussian,
    Sigmoid,
}

#[derive(Debug, Clone)]
pub struct FuzzyRule {
    pub rule_id: String,
    pub antecedent: Vec<FuzzyCondition>,
    pub consequent: String,
    pub weight: f32,
}

#[derive(Debug, Clone)]
pub struct FuzzyCondition {
    pub variable: String,
    pub set: String,
    pub operator: FuzzyOperator,
}

#[derive(Debug, Clone, Copy)]
pub enum FuzzyOperator {
    And,
    Or,
    Not,
}

impl FuzzyReasoner {
    pub fn new() -> Self {
        FuzzyReasoner {
            fuzzy_sets: HashMap::new(),
            rules: Vec::new(),
        }
    }

    /// Adds fuzzy set
    pub fn add_set(&mut self, fuzzy_set: FuzzySet) {
        self.fuzzy_sets.insert(fuzzy_set.name.clone(), fuzzy_set);
    }

    /// Adds fuzzy rule
    pub fn add_rule(&mut self, rule: FuzzyRule) {
        self.rules.push(rule);
    }

    /// Computes fuzzy inference
    pub fn infer(&self, inputs: &HashMap<String, f32>) -> HashMap<String, f32> {
        let mut outputs = HashMap::new();

        for rule in &self.rules {
            let membership = self.compute_rule_membership(rule, inputs);
            *outputs.entry(rule.consequent.clone()).or_insert(0.0) += membership * rule.weight;
        }

        outputs
    }

    fn compute_rule_membership(&self, rule: &FuzzyRule, inputs: &HashMap<String, f32>) -> f32 {
        let mut result: f32 = 1.0;

        for condition in &rule.antecedent {
            if let Some(value) = inputs.get(&condition.variable) {
                if let Some(set) = self.fuzzy_sets.get(&condition.set) {
                    let membership = self.compute_membership(set, *value);

                    match condition.operator {
                        FuzzyOperator::And => result = result.min(membership),
                        FuzzyOperator::Or => result = result.max(membership),
                        FuzzyOperator::Not => result = 1.0 - membership,
                    }
                }
            }
        }

        result
    }

    fn compute_membership(&self, set: &FuzzySet, value: f32) -> f32 {
        let params = &set.parameters;

        match set.membership_function {
            MembershipType::Triangular if params.len() >= 3 => {
                let (a, b, c) = (params[0], params[1], params[2]);
                if value <= a || value >= c {
                    0.0
                } else if value == b {
                    1.0
                } else if value < b {
                    (value - a) / (b - a)
                } else {
                    (c - value) / (c - b)
                }
            }
            _ => 0.5,
        }
    }
}

impl Default for FuzzyReasoner {
    fn default() -> Self {
        Self::new()
    }
}

/// Rule mining from experience
pub struct RuleMiner {
    patterns: Vec<MinedPattern>,
    min_support: f32,
    min_confidence: f32,
}

#[derive(Debug, Clone)]
pub struct MinedPattern {
    pub pattern_id: String,
    pub antecedent: Vec<String>,
    pub consequent: String,
    pub support: f32,
    pub confidence: f32,
    pub lift: f32,
}

impl RuleMiner {
    pub fn new() -> Self {
        RuleMiner {
            patterns: Vec::new(),
            min_support: 0.1,
            min_confidence: 0.5,
        }
    }

    /// Mines association rules from observations
    pub fn mine_rules(&mut self, observations: &[Vec<String>]) -> Vec<MinedPattern> {
        let mut patterns = Vec::new();

        // Frequent itemset mining (simplified Apriori)
        let item_counts = self.count_items(observations);
        let observations_len = observations.len() as f32;
        let frequent_items: Vec<String> = item_counts
            .iter()
            .filter(|(_, &count)| {
                let support = count as f32 / observations_len;
                support >= self.min_support
            })
            .map(|(item, _)| item.clone())
            .collect();

        // Generate rules from frequent items
        for item in &frequent_items {
            for other in &frequent_items {
                if item != other {
                    let confidence = self.compute_confidence(observations, item, other);
                    if confidence >= self.min_confidence {
                        patterns.push(MinedPattern {
                            pattern_id: generate_id(format!("{}_{}", item, other).as_bytes())
                                .to_string(),
                            antecedent: vec![item.clone()],
                            consequent: other.clone(),
                            support: self.compute_support(observations, item),
                            confidence,
                            lift: self.compute_lift(observations, item, other),
                        });
                    }
                }
            }
        }

        self.patterns.extend(patterns.clone());
        patterns
    }

    fn count_items(&self, observations: &[Vec<String>]) -> HashMap<String, usize> {
        let mut counts = HashMap::new();

        for obs in observations {
            for item in obs {
                *counts.entry(item.clone()).or_insert(0) += 1;
            }
        }

        counts
    }

    fn compute_support(&self, observations: &[Vec<String>], item: &str) -> f32 {
        let count = observations
            .iter()
            .filter(|obs| obs.contains(&item.to_string()))
            .count();

        count as f32 / observations.len() as f32
    }

    fn compute_confidence(
        &self,
        observations: &[Vec<String>],
        antecedent: &str,
        consequent: &str,
    ) -> f32 {
        let ant_count = observations
            .iter()
            .filter(|obs| obs.contains(&antecedent.to_string()))
            .count();

        if ant_count == 0 {
            return 0.0;
        }

        let both_count = observations
            .iter()
            .filter(|obs| {
                obs.contains(&antecedent.to_string()) && obs.contains(&consequent.to_string())
            })
            .count();

        both_count as f32 / ant_count as f32
    }

    fn compute_lift(
        &self,
        observations: &[Vec<String>],
        antecedent: &str,
        consequent: &str,
    ) -> f32 {
        let confidence = self.compute_confidence(observations, antecedent, consequent);
        let consequent_support = self.compute_support(observations, consequent);

        if consequent_support == 0.0 {
            return 0.0;
        }

        confidence / consequent_support
    }
}

impl Default for RuleMiner {
    fn default() -> Self {
        Self::new()
    }
}
