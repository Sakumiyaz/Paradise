// EDEN GARM Meta-Evolution — Fase 3: Compositional Generalization
// The genome encodes executable pseudocode that defines not just centroid updates
// but also symbolic inference rules over the concept graph.
// The system evolves its own algorithms for reasoning.

use super::morphogenesis::ConceptSpace;

#[derive(Clone, Debug)]
pub enum Op {
    Nop,
    // --- Centroid update ops (existing) ---
    Ema { alpha: f32 },
    Threshold { t: f32 },
    Scale { s: f32 },
    Decay { rate: f32 },
    ResetIfStagnant { max_age: u32 },
    // --- Symbolic reasoning ops (Fase 3) ---
    Push { val: f32 },   // push constant onto value stack
    Pop,                 // pop value stack
    Add,                 // pop a,b, push a+b
    Mul,                 // pop a,b, push a*b
    CmpLt { t: f32 },    // pop a, push 1.0 if a < t else 0.0
    IfSkip { n: usize }, // if top of value stack <= 0.0, skip next n ops
    // --- Concept graph ops ---
    ActivateConcept, // pop concept_id (as f32) from value stack, push onto activation stack
    Link { rel: String }, // link top two activated concepts with relation
    Infer { rel: String, steps: u8 }, // infer transitive relations from top activated concept
    QueryRel { rel: String }, // query relations of type rel from top activated concept, push count as f32
}

impl Op {
    /// Decode 3 consecutive genes into an Op.
    pub fn decode(genes: &[f32]) -> Self {
        if genes.len() < 3 {
            return Op::Nop;
        }
        let opcode = genes[0];
        let op1 = genes[1].clamp(0.0, 1.0);
        let op2 = genes[2];
        let rels = ["leads_to", "is_a", "causes_risk", "part_of", "similar_to"];
        let rel_idx = (op1 * rels.len() as f32) as usize % rels.len();
        let rel = rels[rel_idx].to_string();
        let steps = (op2.abs() * 5.0) as u8 + 1;

        if opcode < 1.0 {
            Op::Nop
        } else if opcode < 2.0 {
            Op::Ema { alpha: op1 }
        } else if opcode < 3.0 {
            Op::Threshold { t: op2.abs() }
        } else if opcode < 4.0 {
            Op::Scale { s: op1 * 2.0 }
        } else if opcode < 5.0 {
            Op::Decay { rate: op1 }
        } else if opcode < 6.0 {
            Op::ResetIfStagnant {
                max_age: (op2.abs() * 100.0) as u32 + 10,
            }
        } else if opcode < 7.0 {
            Op::Push { val: op2 }
        } else if opcode < 8.0 {
            Op::Pop
        } else if opcode < 9.0 {
            Op::Add
        } else if opcode < 10.0 {
            Op::Mul
        } else if opcode < 11.0 {
            Op::CmpLt { t: op2.abs() }
        } else if opcode < 12.0 {
            Op::IfSkip {
                n: (op1 * 5.0) as usize + 1,
            }
        } else if opcode < 13.0 {
            Op::ActivateConcept
        } else if opcode < 14.0 {
            Op::Link { rel }
        } else if opcode < 15.0 {
            Op::Infer { rel, steps }
        } else {
            Op::QueryRel { rel }
        }
    }
}

pub struct MetaVM {
    pub program: Vec<Op>,
    pub register: f32,
    pub value_stack: Vec<f32>,
    pub activation_stack: Vec<u64>, // concept IDs for graph ops
    pub inferred_relations: Vec<(u64, String, u64)>, // relations inferred by the program
}

impl MetaVM {
    pub fn from_genome(genome_tail: &[f32]) -> Self {
        let mut program = Vec::new();
        for chunk in genome_tail.chunks(3) {
            program.push(Op::decode(chunk));
        }
        MetaVM {
            program,
            register: 0.5,
            value_stack: Vec::new(),
            activation_stack: Vec::new(),
            inferred_relations: Vec::new(),
        }
    }

    /// Run the program to compute a centroid update blend factor.
    /// Returns alpha in [0,1]: 0 = keep old centroid, 1 = use new embedding fully.
    pub fn run(&mut self, old_centroid_dist: f32, concept_count: u32) -> f32 {
        let mut alpha = 0.5f32;
        let mut pc = 0usize;
        while pc < self.program.len() {
            let op = &self.program[pc].clone();
            match op {
                Op::Nop => {}
                Op::Ema { alpha: a } => {
                    alpha = *a;
                }
                Op::Threshold { t } => {
                    if old_centroid_dist > *t {
                        alpha = 0.0;
                    }
                }
                Op::Scale { s } => {
                    alpha = (alpha * s).clamp(0.0, 1.0);
                }
                Op::Decay { rate } => {
                    let decay = (-(*rate) * concept_count as f32).exp();
                    alpha *= decay;
                }
                Op::ResetIfStagnant { max_age } => {
                    if concept_count > *max_age {
                        alpha = 1.0;
                    }
                }
                // Fase 3 stack ops (ignored in pure centroid mode)
                Op::Push { val } => {
                    self.value_stack.push(*val);
                }
                Op::Pop => {
                    self.value_stack.pop();
                }
                Op::Add => {
                    if let (Some(b), Some(a)) = (self.value_stack.pop(), self.value_stack.pop()) {
                        self.value_stack.push(a + b);
                    }
                }
                Op::Mul => {
                    if let (Some(b), Some(a)) = (self.value_stack.pop(), self.value_stack.pop()) {
                        self.value_stack.push(a * b);
                    }
                }
                Op::CmpLt { t } => {
                    if let Some(a) = self.value_stack.pop() {
                        self.value_stack.push(if a < *t { 1.0 } else { 0.0 });
                    }
                }
                Op::IfSkip { n } => {
                    if let Some(cond) = self.value_stack.pop() {
                        if cond <= 0.0 {
                            pc += n; // skip next n ops
                            continue;
                        }
                    }
                }
                _ => {} // graph ops handled in run_graph
            }
            pc += 1;
        }
        alpha.clamp(0.0, 1.0)
    }

    /// Run the program in graph-reasoning mode using a concept space.
    /// Discovers new relations and adds them to the concept space.
    pub fn run_graph(&mut self, morpho: &mut ConceptSpace, current_concept: u64) {
        self.activation_stack.clear();
        self.inferred_relations.clear();
        self.value_stack.push(current_concept as f32); // seed with current concept
        let mut pc = 0usize;
        while pc < self.program.len() {
            let op = self.program[pc].clone();
            match op {
                Op::Push { val } => {
                    self.value_stack.push(val);
                }
                Op::Pop => {
                    self.value_stack.pop();
                }
                Op::Add => {
                    if let (Some(b), Some(a)) = (self.value_stack.pop(), self.value_stack.pop()) {
                        self.value_stack.push(a + b);
                    }
                }
                Op::Mul => {
                    if let (Some(b), Some(a)) = (self.value_stack.pop(), self.value_stack.pop()) {
                        self.value_stack.push(a * b);
                    }
                }
                Op::CmpLt { t } => {
                    if let Some(a) = self.value_stack.pop() {
                        self.value_stack.push(if a < t { 1.0 } else { 0.0 });
                    }
                }
                Op::IfSkip { n } => {
                    if let Some(cond) = self.value_stack.pop() {
                        if cond <= 0.0 {
                            pc += n;
                            continue;
                        }
                    }
                }
                Op::ActivateConcept => {
                    if let Some(cid_f) = self.value_stack.pop() {
                        let cid = cid_f as u64;
                        if morpho.concepts.contains_key(&cid) {
                            self.activation_stack.push(cid);
                        }
                    }
                }
                Op::Link { rel } => {
                    if self.activation_stack.len() >= 2 {
                        let b = self.activation_stack.pop().unwrap();
                        let a = self.activation_stack.pop().unwrap();
                        morpho.add_relation(a, &rel, b);
                        self.inferred_relations.push((a, rel.clone(), b));
                    }
                }
                Op::Infer { rel, steps } => {
                    if let Some(cid) = self.activation_stack.last().copied() {
                        let inferred = morpho.infer_transitive(cid, &rel);
                        for target in &inferred[..inferred.len().min(steps as usize)] {
                            self.inferred_relations.push((cid, rel.clone(), *target));
                        }
                        // Push count of inferred relations as signal
                        self.value_stack.push(inferred.len() as f32);
                    }
                }
                Op::QueryRel { rel } => {
                    if let Some(cid) = self.activation_stack.last().copied() {
                        let count = morpho
                            .concepts
                            .get(&cid)
                            .and_then(|c| c.relations.get(&rel))
                            .map(|v| v.len())
                            .unwrap_or(0);
                        self.value_stack.push(count as f32);
                    }
                }
                // centroid ops are no-ops in graph mode
                _ => {}
            }
            pc += 1;
        }
    }

    pub fn n_inferred(&self) -> usize {
        self.inferred_relations.len()
    }
}

/// Apply the evolved program to a ConceptSpace update.
/// Called from morphogenesis when assimilating a new embedding.
pub fn apply_meta_update(
    vm: &mut MetaVM,
    concept: &mut super::morphogenesis::Concept,
    embedding: &[f32],
    dist: f32,
) {
    let alpha = vm.run(dist, concept.count);
    let n = concept.count as f32;
    for i in 0..embedding.len().min(concept.centroid.len()) {
        let default_avg = (concept.centroid[i] * n + embedding[i]) / (n + 1.0);
        concept.centroid[i] = concept.centroid[i] * (1.0 - alpha) + default_avg * alpha;
    }
}

/// The genome decoder extracts the algorithm chromosome from the tail.
pub fn extract_program(genome: &[f32], weights_size: usize) -> &[f32] {
    if genome.len() > weights_size + 3 {
        &genome[weights_size..]
    } else {
        &[]
    }
}

pub fn status(program: &[Op]) -> String {
    let ops: Vec<String> = program.iter().map(|op| format!("{:?}", op)).collect();
    format!(
        "MetaEvolve | ops: {} | program: {}",
        program.len(),
        ops.join(",")
    )
}
