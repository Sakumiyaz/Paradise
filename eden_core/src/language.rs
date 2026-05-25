//! Language - Eden's Communication Protocol
//!
//! Language is how Eden communicates with itself and the outside world.
//! It uses semantic lattices for meaning representation.
#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(unused_imports)]

use crate::membrain::{generate_id, NOW_MS};

/// Token types in Eden's language
#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Concept,  // Abstract concept
    Action,   // Verb/action
    Entity,   // Noun/entity
    Modifier, // Adjective/adverb
    Relation, // Connection between concepts
    Literal,  // Raw data/value
    Pattern,  // Pattern reference
}

/// Semantic node in the language lattice
#[derive(Debug, Clone)]
pub struct SemanticNode {
    pub id: String,
    pub token_type: TokenType,
    pub meaning_vector: Vec<f64>,
    pub associations: Vec<String>, // Linked nodes
    pub activation_level: f64,
    pub context_tags: Vec<String>,
}

/// Expression - A semantic utterance
#[derive(Debug, Clone)]
pub struct Expression {
    pub id: String,
    pub nodes: Vec<SemanticNode>,
    pub activation: f64,
    pub coherence: f64,
    pub birth_time: u64,
    pub depth: usize,
}

impl Expression {
    /// Create a new expression
    pub fn new(depth: usize) -> Self {
        Expression {
            id: generate_id(&[]).to_string(),
            nodes: Vec::new(),
            activation: 1.0,
            coherence: 0.5,
            birth_time: NOW_MS(),
            depth,
        }
    }

    /// Add a semantic node
    pub fn add_node(&mut self, node: SemanticNode) {
        self.nodes.push(node);
        self.recalculate_coherence();
    }

    /// Calculate coherence based on node connections
    fn recalculate_coherence(&mut self) {
        if self.nodes.is_empty() {
            self.coherence = 0.0;
            return;
        }

        let total_associations: usize = self.nodes.iter().map(|n| n.associations.len()).sum();

        let possible_connections = self.nodes.len().saturating_sub(1) * self.nodes.len();

        self.coherence = if possible_connections > 0 {
            (total_associations as f64 / possible_connections as f64).min(1.0)
        } else {
            0.0
        };
    }

    /// Propagate activation through network
    pub fn propagate_activation(&mut self) {
        let decay = 0.9;
        let spread = 0.1;

        // First pass: apply decay
        for node in &mut self.nodes {
            node.activation_level *= decay;
        }

        // Second pass: spread activation - need to avoid double borrow
        // Collect updates to apply after iteration
        let mut updates: Vec<(String, f64)> = Vec::new();

        for node in &self.nodes {
            if node.activation_level > 0.5 {
                for assoc_id in &node.associations {
                    updates.push((assoc_id.clone(), node.activation_level * spread));
                }
            }
        }

        // Apply updates
        for (assoc_id, activation) in updates {
            if let Some(target) = self.nodes.iter_mut().find(|n| n.id == assoc_id) {
                target.activation_level += activation;
            }
        }
    }

    /// Get most activated node
    pub fn most_activated(&self) -> Option<&SemanticNode> {
        self.nodes
            .iter()
            .max_by(|a, b| a.activation_level.partial_cmp(&b.activation_level).unwrap())
    }

    /// Merge two expressions
    pub fn merge_with(&self, other: &Expression) -> Expression {
        let mut new_expr = Self::new(self.depth.max(other.depth) + 1);

        // Combine nodes
        let mut all_nodes = self.nodes.clone();
        all_nodes.extend_from_slice(&other.nodes);
        new_expr.nodes = all_nodes;

        // Average metrics
        new_expr.activation = (self.activation + other.activation) / 2.0;
        new_expr.coherence = (self.coherence + other.coherence) / 2.0;

        new_expr
    }
}

/// Semantic Lattice - Hierarchical meaning structure
#[derive(Debug, Clone)]
pub struct SemanticLattice {
    pub root_id: String,
    pub levels: Vec<Vec<SemanticNode>>,
    pub max_depth: usize,
}

impl SemanticLattice {
    /// Create new semantic lattice
    pub fn new(max_depth: usize) -> Self {
        SemanticLattice {
            root_id: generate_id(&[]).to_string(),
            levels: vec![Vec::new(); max_depth + 1],
            max_depth,
        }
    }

    /// Add node at specific level
    pub fn add_at_level(&mut self, node: SemanticNode, level: usize) {
        if level <= self.max_depth {
            self.levels[level].push(node);
        }
    }

    /// Get nodes at level
    pub fn get_level(&self, level: usize) -> &[SemanticNode] {
        if level <= self.max_depth {
            &self.levels[level]
        } else {
            &[]
        }
    }

    /// Propagate meaning upward
    pub fn abstract_meaning(&self, from_level: usize) -> Vec<f64> {
        if from_level == 0 || from_level > self.max_depth {
            return vec![];
        }

        let mut meaning = Vec::new();

        // Aggregate meanings from lower level
        for node in &self.levels[from_level] {
            meaning.extend_from_slice(&node.meaning_vector);
        }

        // Normalize
        if !meaning.is_empty() {
            let sum: f64 = meaning.iter().map(|v| v.abs()).sum();
            for m in &mut meaning {
                *m /= sum;
            }
        }

        meaning
    }

    /// Propagate meaning downward
    pub fn concretize_meaning(&self, abstract_meaning: &[f64]) -> Vec<&SemanticNode> {
        let mut concretized = Vec::new();

        // Find nodes at lowest level that match abstract meaning
        for level in (1..=self.max_depth).rev() {
            for node in &self.levels[level] {
                let similarity = self.cosine_similarity(&node.meaning_vector, abstract_meaning);
                if similarity > 0.5 {
                    concretized.push(node);
                }
            }
        }

        concretized
    }

    /// Cosine similarity between vectors
    fn cosine_similarity(&self, a: &[f64], b: &[f64]) -> f64 {
        if a.is_empty() || b.is_empty() || a.len() != b.len() {
            return 0.0;
        }

        let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let mag_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
        let mag_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();

        if mag_a == 0.0 || mag_b == 0.0 {
            return 0.0;
        }

        dot / (mag_a * mag_b)
    }
}

/// Language processor
pub struct LanguageProcessor {
    pub lattice: SemanticLattice,
    pub active_expression: Option<Expression>,
}

impl LanguageProcessor {
    /// Create new language processor
    pub fn new() -> Self {
        LanguageProcessor {
            lattice: SemanticLattice::new(5),
            active_expression: None,
        }
    }

    /// Create concept token
    pub fn create_concept(&self, meaning: Vec<f64>) -> SemanticNode {
        // Convert f64 vector to bytes for ID generation
        let id_bytes: Vec<u8> = meaning.iter().flat_map(|&f| f.to_le_bytes()).collect();
        SemanticNode {
            id: generate_id(&id_bytes).to_string(),
            token_type: TokenType::Concept,
            meaning_vector: meaning,
            associations: Vec::new(),
            activation_level: 1.0,
            context_tags: Vec::new(),
        }
    }

    /// Create action token
    pub fn create_action(&self, action_type: &str, parameters: &[f64]) -> SemanticNode {
        let mut meaning = vec![0.0; 32];
        meaning[0] = action_type.len() as f64 % 256.0;

        for (i, &p) in parameters.iter().enumerate().take(31) {
            meaning[i + 1] = p;
        }

        // Convert f64 vector to bytes for ID generation
        let id_bytes: Vec<u8> = meaning.iter().flat_map(|&f| f.to_le_bytes()).collect();
        SemanticNode {
            id: generate_id(&id_bytes).to_string(),
            token_type: TokenType::Action,
            meaning_vector: meaning,
            associations: Vec::new(),
            activation_level: 1.0,
            context_tags: vec![action_type.to_string()],
        }
    }

    /// Process input into expression
    pub fn process(&mut self, input: &[SemanticNode]) -> Expression {
        let mut expr = Expression::new(0);

        for node in input {
            let mut node_clone = node.clone();
            node_clone.activation_level = 1.0;
            expr.add_node(node_clone);
        }

        self.active_expression = Some(expr.clone());
        expr
    }

    /// Interpret expression and generate response
    pub fn interpret(&self, expr: &Expression) -> Vec<SemanticNode> {
        let mut response = Vec::new();

        for node in &expr.nodes {
            if node.activation_level > 0.5 {
                let mut response_node = node.clone();
                response_node.activation_level *= 0.8;
                response.push(response_node);
            }
        }

        response
    }
}

impl Default for LanguageProcessor {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// ADVANCED LANGUAGE PROCESSING - Semantic Lattice, Pragmatics, Discourse
// ============================================================================

use std::collections::{HashMap, HashSet, VecDeque};
/// Advanced semantic lattice for dense meaning representation
#[derive(Debug, Clone)]
pub struct AdvancedSemanticLattice {
    /// Nodes in the lattice
    nodes: HashMap<String, LatticeNode>,
    /// Hierarchy levels
    levels: Vec<HashSet<String>>,
    /// Root concepts
    roots: HashSet<String>,
}

/// Node in semantic lattice
#[derive(Debug, Clone)]
pub struct LatticeNode {
    pub id: String,
    pub concept: String,
    pub abstraction_level: u8,
    pub semantic_density: f32,
    pub hypernyms: Vec<String>, // Broader concepts
    pub hyponyms: Vec<String>,  // Narrower concepts
    pub synonyms: Vec<String>,
    pub antonyms: Vec<String>,
    pub related_concepts: Vec<String>,
    pub meaning_vector: Vec<f64>,
}

/// Discourse relation between segments
#[derive(Debug, Clone)]
pub enum DiscourseRelation {
    Elaboration,
    Explanation,
    Contrast,
    Cause,
    Condition,
    Sequence,
    Background,
    Attribution,
}

/// Discourse segment
#[derive(Debug, Clone)]
pub struct DiscourseSegment {
    pub id: u64,
    pub content: String,
    pub relation: Option<DiscourseRelation>,
    pub speakers: HashSet<u64>,
    pub depth: usize,
    pub start_time: u64,
    pub end_time: Option<u64>,
}

/// Implicature computation result
#[derive(Debug, Clone)]
pub struct Implicature {
    pub implied_meaning: String,
    pub inference_type: ImplicatureType,
    pub confidence: f32,
    pub是基于: Vec<String>,
}

/// Type of implicature inference
#[derive(Debug, Clone, Copy)]
pub enum ImplicatureType {
    Scalar,
    Conditional,
    Contrastive,
    Conversational,
    Conventional,
}

/// Presupposition trigger
#[derive(Debug, Clone)]
pub struct Presupposition {
    pub trigger: String,
    pub projected_content: String,
    pub status: PresuppositionStatus,
}

/// Status of presupposition
#[derive(Debug, Clone, Copy)]
pub enum PresuppositionStatus {
    Held,
    Challenged,
    Cancelled,
}

/// Sentiment analysis result
#[derive(Debug, Clone)]
pub struct SentimentResult {
    pub overall_polarity: f32,
    pub subjectivity: f32,
    pub aspects: Vec<AspectSentiment>,
    pub emotions: Vec<(String, f32)>,
}

/// Aspect-based sentiment
#[derive(Debug, Clone)]
pub struct AspectSentiment {
    pub aspect: String,
    pub polarity: f32,
    pub confidence: f32,
}

/// Metaphor mapping
#[derive(Debug, Clone)]
pub struct MetaphorMapping {
    pub source_domain: String,
    pub target_domain: String,
    pub mapping_type: MappingType,
    pub alignment_score: f32,
}

/// Type of metaphor mapping
#[derive(Debug, Clone, Copy)]
pub enum MappingType {
    Structural,
    Ontological,
    Corpus,
}

/// Coreference link
#[derive(Debug, Clone)]
pub struct CoreferenceLink {
    pub mention: String,
    pub referent: String,
    pub confidence: f32,
    pub antecedent: Option<u64>,
}

/// Textual entailment result
#[derive(Debug, Clone)]
pub struct EntailmentResult {
    pub hypothesis_entailed: bool,
    pub contradiction: bool,
    pub neutral: bool,
    pub confidence: f32,
    pub reasoning: String,
}

impl AdvancedSemanticLattice {
    /// Creates new semantic lattice
    pub fn new() -> Self {
        AdvancedSemanticLattice {
            nodes: HashMap::new(),
            levels: vec![HashSet::new(); 10], // 10 abstraction levels
            roots: HashSet::new(),
        }
    }

    /// Adds node to lattice
    pub fn add_node(&mut self, node: LatticeNode) {
        let level = node.abstraction_level as usize;
        if level < self.levels.len() {
            self.levels[level].insert(node.id.clone());
        }
        self.nodes.insert(node.id.clone(), node);
    }

    /// Adds hierarchical relationship
    pub fn add_hierarchy(&mut self, hypernym: &str, hyponym: &str) {
        if let Some(parent) = self.nodes.get_mut(hypernym) {
            parent.hyponyms.push(hyponym.to_string());
        }
        if let Some(child) = self.nodes.get_mut(hyponym) {
            child.hypernyms.push(hypernym.to_string());
        }
    }

    /// Finds nodes by concept similarity
    pub fn find_similar(&self, _concept: &str, threshold: f32) -> Vec<&LatticeNode> {
        self.nodes
            .values()
            .filter(|n| self.cosine_similarity(&n.meaning_vector, &[0.0; 64]) > threshold)
            .collect()
    }

    /// Computes cosine similarity between meaning vectors
    fn cosine_similarity(&self, v1: &[f64], v2: &[f64]) -> f32 {
        if v1.len() != v2.len() {
            return 0.0;
        }

        let dot: f64 = v1.iter().zip(v2.iter()).map(|(a, b)| a * b).sum();
        let mag1 = (v1.iter().map(|x| x * x).sum::<f64>()).sqrt();
        let mag2 = (v2.iter().map(|x| x * x).sum::<f64>()).sqrt();

        if mag1 == 0.0 || mag2 == 0.0 {
            return 0.0;
        }

        (dot / (mag1 * mag2)) as f32
    }

    /// Performs lattice traversal for concept resolution
    pub fn traverse(&self, start: &str, direction: TraversalDirection) -> Vec<String> {
        let mut visited: HashSet<String> = HashSet::new();
        let mut result = Vec::new();
        let mut queue = VecDeque::new();

        queue.push_back(start.to_string());

        while let Some(current) = queue.pop_front() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current.clone());
            result.push(current.clone());

            if let Some(node) = self.nodes.get(&current) {
                match direction {
                    TraversalDirection::Up => {
                        for hypernym in &node.hypernyms {
                            if !visited.contains(hypernym) {
                                queue.push_back(hypernym.clone());
                            }
                        }
                    }
                    TraversalDirection::Down => {
                        for hyponym in &node.hyponyms {
                            if !visited.contains(hyponym) {
                                queue.push_back(hyponym.clone());
                            }
                        }
                    }
                    TraversalDirection::Lateral => {
                        for related in &node.related_concepts {
                            if !visited.contains(related) {
                                queue.push_back(related.clone());
                            }
                        }
                    }
                }
            }
        }

        result
    }

    /// Gets abstraction chain from specific to general
    pub fn get_abstraction_chain(&self, concept_id: &str) -> Vec<String> {
        let mut chain = vec![concept_id.to_string()];
        let mut current = concept_id.to_string();

        while let Some(node) = self.nodes.get(&current) {
            if node.hypernyms.is_empty() {
                break;
            }
            current = node.hypernyms[0].clone();
            chain.push(current.clone());
        }

        chain
    }
}

/// Direction for lattice traversal
#[derive(Debug, Clone, Copy)]
pub enum TraversalDirection {
    Up,      // Toward more abstract
    Down,    // Toward more specific
    Lateral, // To related concepts
}

impl Default for SemanticLattice {
    fn default() -> Self {
        Self::new(4)
    }
}

/// Pragmatic inference engine
pub struct PragmaticInference {
    context_stack: Vec<PragmaticContext>,
    implicature_cache: HashMap<String, Implicature>,
    max_context_depth: usize,
}

/// Pragmatic context
#[derive(Debug, Clone)]
pub struct PragmaticContext {
    pub speakers: HashSet<u64>,
    pub shared_knowledge: HashSet<String>,
    pub discourse_history: Vec<DiscourseSegment>,
    pub goals: Vec<String>,
    pub timestamp: u64,
}

impl PragmaticInference {
    pub fn new() -> Self {
        PragmaticInference {
            context_stack: Vec::new(),
            implicature_cache: HashMap::new(),
            max_context_depth: 10,
        }
    }

    /// Pushes new context
    pub fn push_context(&mut self, context: PragmaticContext) {
        if self.context_stack.len() >= self.max_context_depth {
            self.context_stack.remove(0);
        }
        self.context_stack.push(context);
    }

    /// Computes implicature from utterance
    pub fn compute_implicature(&mut self, utterance: &str, _speaker: u64) -> Vec<Implicature> {
        let mut implicatures = Vec::new();

        // Check cache
        if let Some(cached) = self.implicature_cache.get(utterance) {
            return vec![cached.clone()];
        }

        // Scalar implicature (some vs all)
        if utterance.contains("some") {
            let imp = self.compute_scalar_implicature(utterance);
            implicatures.push(imp);
        }

        // Conditional implicature
        if utterance.contains("if") || utterance.contains("when") {
            let imp = self.compute_conditional_implicature(utterance);
            implicatures.push(imp);
        }

        // Contrastive implicature
        if utterance.contains("but") || utterance.contains("however") {
            let imp = self.compute_contrastive_implicature(utterance);
            implicatures.push(imp);
        }

        // Cache result
        if let Some(first) = implicatures.first() {
            self.implicature_cache
                .insert(utterance.to_string(), first.clone());
        }

        implicatures
    }

    fn compute_scalar_implicature(&self, utterance: &str) -> Implicature {
        Implicature {
            implied_meaning: format!(
                "Not all - only some. '{}' implies partial exclusion",
                utterance
            ),
            inference_type: ImplicatureType::Scalar,
            confidence: 0.8,
            pub是基于: vec!["Scalar maximization".to_string()],
        }
    }

    fn compute_conditional_implicature(&self, _utterance: &str) -> Implicature {
        Implicature {
            implied_meaning: format!("If condition holds, then consequent is expected"),
            inference_type: ImplicatureType::Conditional,
            confidence: 0.75,
            pub是基于: vec!["Conditional logic".to_string()],
        }
    }

    fn compute_contrastive_implicature(&self, utterance: &str) -> Implicature {
        Implicature {
            implied_meaning: format!(
                "Contrast with expectation - '{}' implies unexpected contrast",
                utterance
            ),
            inference_type: ImplicatureType::Contrastive,
            confidence: 0.7,
            pub是基于: vec!["Contrast inference".to_string()],
        }
    }

    /// Gets current context
    pub fn current_context(&self) -> Option<&PragmaticContext> {
        self.context_stack.last()
    }

    /// Resolves presuppositions
    pub fn resolve_presuppositions(&self, utterance: &str) -> Vec<Presupposition> {
        let mut presuppositions = Vec::new();

        // "Again" presupposes previous occurrence
        if utterance.contains("again") {
            presuppositions.push(Presupposition {
                trigger: "again".to_string(),
                projected_content: "Something happened before".to_string(),
                status: PresuppositionStatus::Held,
            });
        }

        // "Stop" presupposes it was ongoing
        if utterance.contains("stop") {
            presuppositions.push(Presupposition {
                trigger: "stop".to_string(),
                projected_content: "The action was in progress".to_string(),
                status: PresuppositionStatus::Held,
            });
        }

        presuppositions
    }
}

impl Default for PragmaticInference {
    fn default() -> Self {
        Self::new()
    }
}

/// Deep discourse analyzer
pub struct DiscourseAnalyzer {
    segments: Vec<DiscourseSegment>,
    relations: HashMap<(u64, u64), DiscourseRelation>,
    coherence_score: f32,
}

impl DiscourseAnalyzer {
    pub fn new() -> Self {
        DiscourseAnalyzer {
            segments: Vec::new(),
            relations: HashMap::new(),
            coherence_score: 0.5,
        }
    }

    /// Adds segment to discourse
    pub fn add_segment(&mut self, content: &str, speaker: u64) -> u64 {
        let id = self.segments.len() as u64;
        let segment = DiscourseSegment {
            id,
            content: content.to_string(),
            relation: None,
            speakers: vec![speaker].into_iter().collect(),
            depth: 0,
            start_time: NOW_MS(),
            end_time: None,
        };
        self.segments.push(segment);
        self.update_coherence();
        id
    }

    /// Links segments with discourse relation
    pub fn link_segments(&mut self, from: u64, to: u64, relation: DiscourseRelation) {
        self.relations.insert((from, to), relation.clone());

        // Infer relation type for target
        if let Some(segment) = self.segments.iter_mut().find(|s| s.id == to) {
            segment.relation = Some(relation);
        }

        self.update_coherence();
    }

    /// Updates discourse coherence score
    fn update_coherence(&mut self) {
        if self.segments.is_empty() {
            self.coherence_score = 0.0;
            return;
        }

        let linked = self.relations.len();
        let possible = self.segments.len().saturating_sub(1);

        self.coherence_score = if possible > 0 {
            (linked as f32 / possible as f32).min(1.0)
        } else {
            0.5
        };
    }

    /// Gets coherent discourse segments
    pub fn get_coherent_chains(&self) -> Vec<Vec<u64>> {
        let mut chains = Vec::new();

        for (start_idx, segment) in self.segments.iter().enumerate() {
            if segment.relation.is_none() {
                let mut chain = vec![segment.id];

                let mut current_idx = start_idx;
                while let Some((next_id, _)) = self
                    .relations
                    .iter()
                    .find(|((from, _), _)| *from == self.segments[current_idx].id)
                    .map(|((_, to), rel)| (*to, rel.clone()))
                {
                    if let Some(next_idx) = self.segments.iter().position(|s| s.id == next_id) {
                        chain.push(next_id);
                        current_idx = next_idx;
                    } else {
                        break;
                    }
                }

                chains.push(chain);
            }
        }

        chains
    }

    /// Summarizes discourse
    pub fn summarize(&self) -> String {
        format!(
            "Discourse with {} segments, coherence: {:.2}",
            self.segments.len(),
            self.coherence_score
        )
    }
}

impl Default for DiscourseAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Advanced sentiment analyzer
pub struct DeepSentimentAnalyzer {
    aspect_lexicon: HashMap<String, Vec<String>>,
    emotion_lexicon: HashMap<String, f32>,
}

impl DeepSentimentAnalyzer {
    pub fn new() -> Self {
        let mut aspect_lexicon = HashMap::new();
        aspect_lexicon.insert(
            "service".to_string(),
            vec![
                "good".to_string(),
                "bad".to_string(),
                "fast".to_string(),
                "slow".to_string(),
            ],
        );
        aspect_lexicon.insert(
            "quality".to_string(),
            vec![
                "high".to_string(),
                "low".to_string(),
                "excellent".to_string(),
                "poor".to_string(),
            ],
        );
        aspect_lexicon.insert(
            "price".to_string(),
            vec![
                "expensive".to_string(),
                "cheap".to_string(),
                "fair".to_string(),
                "overpriced".to_string(),
            ],
        );

        let mut emotion_lexicon = HashMap::new();
        emotion_lexicon.insert("happy".to_string(), 0.8);
        emotion_lexicon.insert("sad".to_string(), -0.6);
        emotion_lexicon.insert("angry".to_string(), -0.7);
        emotion_lexicon.insert("surprised".to_string(), 0.3);

        DeepSentimentAnalyzer {
            aspect_lexicon,
            emotion_lexicon,
        }
    }

    /// Analyzes sentiment of text
    pub fn analyze(&self, text: &str) -> SentimentResult {
        let words: Vec<&str> = text.split_whitespace().collect();

        let mut positive_count = 0;
        let mut negative_count = 0;
        let mut total_polarity = 0.0;

        let aspect_sentiments: HashMap<String, Vec<f32>> = HashMap::new();
        let mut detected_emotions: HashMap<String, f32> = HashMap::new();

        for word in &words {
            let word_lower = word.to_lowercase();

            // Check emotion lexicon
            for (emotion, polarity) in &self.emotion_lexicon {
                if word_lower.contains(emotion) {
                    *detected_emotions.entry(emotion.to_string()).or_insert(0.0) += polarity.abs();
                    total_polarity += polarity;
                }
            }

            // Check positive/negative
            if ["good", "great", "excellent", "amazing", "wonderful"]
                .iter()
                .any(|p| word_lower.contains(p))
            {
                positive_count += 1;
                total_polarity += 0.5;
            }
            if ["bad", "terrible", "awful", "horrible", "poor"]
                .iter()
                .any(|p| word_lower.contains(p))
            {
                negative_count += 1;
                total_polarity -= 0.5;
            }
        }

        let word_count = words.len().max(1) as f32;
        let overall_polarity = (total_polarity / word_count).clamp(-1.0, 1.0);
        let subjectivity = ((positive_count + negative_count) as f32 / word_count).min(1.0);

        let mut aspects = Vec::new();
        for (aspect, _terms) in &self.aspect_lexicon {
            let sentiment_values: Vec<f32> =
                aspect_sentiments.get(aspect).cloned().unwrap_or_default();
            if !sentiment_values.is_empty() {
                let avg: f32 = sentiment_values.iter().sum::<f32>() / sentiment_values.len() as f32;
                aspects.push(AspectSentiment {
                    aspect: aspect.clone(),
                    polarity: avg,
                    confidence: 0.7,
                });
            }
        }

        let emotions: Vec<(String, f32)> = detected_emotions
            .into_iter()
            .map(|(k, v)| (k, v.min(1.0)))
            .collect();

        SentimentResult {
            overall_polarity,
            subjectivity,
            aspects,
            emotions,
        }
    }

    /// Analyzes aspect-based sentiment
    pub fn analyze_aspects(&self, text: &str, aspects: &[String]) -> Vec<AspectSentiment> {
        let mut results = Vec::new();

        for aspect in aspects {
            let aspect_terms = self.aspect_lexicon.get(aspect).cloned().unwrap_or_default();

            let mut polarity_sum = 0.0;
            let mut count = 0;

            for term in aspect_terms {
                if text.to_lowercase().contains(&term.to_lowercase()) {
                    if ["good", "great", "excellent"]
                        .iter()
                        .any(|p| term.contains(p))
                    {
                        polarity_sum += 1.0;
                        count += 1;
                    } else if ["bad", "terrible", "poor"].iter().any(|p| term.contains(p)) {
                        polarity_sum -= 1.0;
                        count += 1;
                    }
                }
            }

            results.push(AspectSentiment {
                aspect: aspect.clone(),
                polarity: if count > 0 {
                    polarity_sum / count as f32
                } else {
                    0.0
                },
                confidence: if count > 0 { 0.8 } else { 0.3 },
            });
        }

        results
    }
}

impl Default for DeepSentimentAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Metaphor understanding engine
pub struct MetaphorEngine {
    mappings: Vec<MetaphorMapping>,
    domain_hierarchies: HashMap<String, Vec<String>>,
}

impl MetaphorEngine {
    pub fn new() -> Self {
        let mut engine = MetaphorEngine {
            mappings: Vec::new(),
            domain_hierarchies: HashMap::new(),
        };

        // Initialize domain hierarchies
        engine.domain_hierarchies.insert(
            "time".to_string(),
            vec![
                "duration".to_string(),
                "moment".to_string(),
                "past".to_string(),
                "future".to_string(),
            ],
        );
        engine.domain_hierarchies.insert(
            "space".to_string(),
            vec![
                "location".to_string(),
                "distance".to_string(),
                "direction".to_string(),
            ],
        );
        engine.domain_hierarchies.insert(
            "social".to_string(),
            vec![
                "hierarchy".to_string(),
                "relationship".to_string(),
                "status".to_string(),
            ],
        );

        engine
    }

    /// Maps metaphorical expression
    pub fn map_metaphor(&mut self, source: &str, target: &str) -> MetaphorMapping {
        let mapping = MetaphorMapping {
            source_domain: source.to_string(),
            target_domain: target.to_string(),
            mapping_type: MappingType::Structural,
            alignment_score: self.compute_alignment(source, target),
        };

        self.mappings.push(mapping.clone());
        mapping
    }

    /// Computes alignment score between domains
    fn compute_alignment(&self, source: &str, target: &str) -> f32 {
        let source_hierarchy = self
            .domain_hierarchies
            .get(source)
            .cloned()
            .unwrap_or_default();
        let target_hierarchy = self
            .domain_hierarchies
            .get(target)
            .cloned()
            .unwrap_or_default();

        if source_hierarchy.is_empty() || target_hierarchy.is_empty() {
            return 0.5;
        }

        let shared: usize = source_hierarchy
            .iter()
            .filter(|s| target_hierarchy.contains(s))
            .count();

        let total = (source_hierarchy.len() + target_hierarchy.len()) / 2;

        shared as f32 / total as f32
    }

    /// Interprets metaphorical expression
    pub fn interpret(&self, metaphor: &str) -> String {
        // Simple metaphor interpretation
        if metaphor.contains("time") && metaphor.contains("money") {
            return "Time is being conceptualized as a limited resource to be spent".to_string();
        }
        if metaphor.contains("idea") && metaphor.contains("plant") {
            return "Ideas are being conceptualized as growing entities".to_string();
        }
        "Metaphorical mapping unclear".to_string()
    }
}

impl Default for MetaphorEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Coreference resolution system
pub struct CoreferenceResolver {
    mentions: Vec<Mention>,
    antecedent_cache: HashMap<u64, u64>,
}

/// Mention in discourse
#[derive(Debug, Clone)]
pub struct Mention {
    pub id: u64,
    pub text: String,
    pub start_pos: usize,
    pub end_pos: usize,
    pub referent_type: ReferentType,
}

/// Type of referent
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReferentType {
    Person,
    Organization,
    Location,
    Object,
    Concept,
    Unknown,
}

impl CoreferenceResolver {
    pub fn new() -> Self {
        CoreferenceResolver {
            mentions: Vec::new(),
            antecedent_cache: HashMap::new(),
        }
    }

    /// Adds mention for resolution
    pub fn add_mention(
        &mut self,
        text: &str,
        start: usize,
        end: usize,
        rtype: ReferentType,
    ) -> u64 {
        let id = self.mentions.len() as u64;
        self.mentions.push(Mention {
            id,
            text: text.to_string(),
            start_pos: start,
            end_pos: end,
            referent_type: rtype,
        });
        id
    }

    /// Resolves coreference chain
    pub fn resolve(&mut self, mention_id: u64) -> Option<u64> {
        let mention = self.mentions.get(mention_id as usize)?;

        // Find potential antecedents
        for i in 0..mention_id as usize {
            let antecedent = &self.mentions[i];

            // Check compatibility
            if antecedent.referent_type == mention.referent_type {
                // Check distance constraint
                let distance = mention.start_pos - antecedent.end_pos;
                if distance < 200 {
                    // Check string similarity
                    if self.string_similarity(&mention.text, &antecedent.text) > 0.6 {
                        self.antecedent_cache.insert(mention_id, antecedent.id);
                        return Some(antecedent.id);
                    }
                }
            }
        }

        None
    }

    /// Computes string similarity
    fn string_similarity(&self, s1: &str, s2: &str) -> f32 {
        let s1_lower = s1.to_lowercase();
        let s2_lower = s2.to_lowercase();

        if s1_lower == s2_lower {
            return 1.0;
        }

        let longer = s1_lower.chars().count().max(s2_lower.chars().count());
        if longer == 0 {
            return 1.0;
        }

        let mut matches = 0;
        for (c1, c2) in s1_lower.chars().zip(s2_lower.chars()) {
            if c1 == c2 {
                matches += 1;
            }
        }

        matches as f32 / longer as f32
    }

    /// Gets coreference chain
    pub fn get_chain(&self, mention_id: u64) -> Vec<u64> {
        let mut chain = vec![mention_id];
        let mut current = mention_id;

        while let Some(&antecedent) = self.antecedent_cache.get(&current) {
            chain.push(antecedent);
            current = antecedent;
        }

        chain
    }
}

impl Default for CoreferenceResolver {
    fn default() -> Self {
        Self::new()
    }
}

/// Textual entailment recognizer
pub struct TextualEntailment {
    inference_rules: Vec<InferenceRule>,
}

/// Inference rule for entailment
#[derive(Debug, Clone)]
pub struct InferenceRule {
    pub premise_pattern: String,
    pub hypothesis_pattern: String,
    pub transformation: String,
    pub confidence: f32,
}

impl TextualEntailment {
    pub fn new() -> Self {
        let mut engine = TextualEntailment {
            inference_rules: Vec::new(),
        };

        // Add basic inference rules
        engine.inference_rules.push(InferenceRule {
            premise_pattern: "X is a Y".to_string(),
            hypothesis_pattern: "X is a Z".to_string(),
            transformation: "If Y implies Z".to_string(),
            confidence: 0.6,
        });

        engine
    }

    /// Recognizes textual entailment
    pub fn recognize(&self, premise: &str, hypothesis: &str) -> EntailmentResult {
        // Simple pattern matching entailment
        let premise_lower = premise.to_lowercase();
        let hypothesis_lower = hypothesis.to_lowercase();

        // Direct containment
        if premise_lower.contains(&hypothesis_lower) {
            return EntailmentResult {
                hypothesis_entailed: true,
                contradiction: false,
                neutral: false,
                confidence: 0.95,
                reasoning: "Hypothesis directly contained in premise".to_string(),
            };
        }

        // Check inference rules
        for rule in &self.inference_rules {
            if premise_lower.contains(&rule.premise_pattern)
                && hypothesis_lower.contains(&rule.hypothesis_pattern)
            {
                return EntailmentResult {
                    hypothesis_entailed: true,
                    contradiction: false,
                    neutral: false,
                    confidence: rule.confidence,
                    reasoning: rule.transformation.clone(),
                };
            }
        }

        // Check for negation (contradiction)
        let premise_negated = premise_lower.contains("not") || premise_lower.contains("n't");
        let hypothesis_negated =
            hypothesis_lower.contains("not") || hypothesis_lower.contains("n't");

        if premise_negated != hypothesis_negated {
            return EntailmentResult {
                hypothesis_entailed: false,
                contradiction: true,
                neutral: false,
                confidence: 0.7,
                reasoning: "Negation mismatch between premise and hypothesis".to_string(),
            };
        }

        EntailmentResult {
            hypothesis_entailed: false,
            contradiction: false,
            neutral: true,
            confidence: 0.5,
            reasoning: "No clear entailment relationship found".to_string(),
        }
    }

    /// Adds inference rule
    pub fn add_rule(&mut self, rule: InferenceRule) {
        self.inference_rules.push(rule);
    }
}

impl Default for TextualEntailment {
    fn default() -> Self {
        Self::new()
    }
}
