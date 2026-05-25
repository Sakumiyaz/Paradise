// EDEN GARM Compositional — Estructuras recursivas de pensamiento.
// 100% Rust puro, 0 LLM, 0 red.
//
// Mientras scene_vector tenia 4 roles fijos (agent/action/patient/context),
// aqui los conceptos pueden combinarse RECURSIVAMENTE en arboles. Esto
// implementa composicionalidad: el significado de una expresion compleja
// es funcion del significado de sus partes y como se combinan.
//
// Tree node types:
//   - Atom(concept_id, label): hoja del arbol
//   - Compose(op, vec<Tree>): combinacion via operador (and, or, causes,
//     part_of, before)
//
// Operations:
//   - parse_to_tree: convierte una oracion en arbol composicional usando
//     conectores conocidos
//   - flatten_to_atoms: extrae todos los conceptos en orden
//   - tree_embedding: calcula embedding del arbol combinando recursivamente

use crate::eden_garm::capabilities::semantics::DistributionalSemantics;

#[derive(Clone, Debug, PartialEq)]
pub enum TreeOp {
    And,
    Or,
    Causes,
    PartOf,
    Before,
    After,
    If,
    Sequence,
}

#[derive(Clone, Debug)]
pub enum CompTree {
    Atom {
        concept_label: String,
        embedding: Vec<f32>,
    },
    Compose {
        op: TreeOp,
        children: Vec<CompTree>,
    },
}

#[derive(Clone, Debug)]
pub struct Compositional {
    pub n_parses: u64,
    pub max_depth: usize,
}

impl Compositional {
    pub fn new() -> Self {
        Compositional {
            n_parses: 0,
            max_depth: 6,
        }
    }

    /// Parse a sentence into a compositional tree using connector words to recurse.
    pub fn parse_to_tree(&mut self, sentence: &str, sem: &DistributionalSemantics) -> CompTree {
        self.n_parses += 1;
        let lower = sentence.to_lowercase();
        // Try to detect top-level connectors and split
        // Priority: porque/because (Causes) > si/if > antes/before > despues/after
        //           > tambien/and > o/or
        let connectors = [
            ("porque", TreeOp::Causes),
            ("because", TreeOp::Causes),
            ("si ", TreeOp::If),
            ("if ", TreeOp::If),
            ("antes", TreeOp::Before),
            ("before", TreeOp::Before),
            ("despues", TreeOp::After),
            ("after", TreeOp::After),
            (" y ", TreeOp::And),
            (" and ", TreeOp::And),
            (" o ", TreeOp::Or),
            (" or ", TreeOp::Or),
        ];
        for (con, op) in &connectors {
            if let Some(idx) = lower.find(con) {
                let before = sentence[..idx].trim().to_string();
                let after = sentence[idx + con.len()..].trim().to_string();
                if !before.is_empty() && !after.is_empty() && self.max_depth > 0 {
                    let mut child_engine = self.clone();
                    child_engine.max_depth = self.max_depth - 1;
                    let left = child_engine.parse_to_tree(&before, sem);
                    let right = child_engine.parse_to_tree(&after, sem);
                    self.n_parses = child_engine.n_parses;
                    return CompTree::Compose {
                        op: op.clone(),
                        children: vec![left, right],
                    };
                }
            }
        }
        // Atom case
        let emb = sem.sentence_embedding(sentence);
        CompTree::Atom {
            concept_label: sentence.trim().to_string(),
            embedding: emb,
        }
    }

    /// Compute embedding of the tree by combining children with op-specific weights.
    pub fn tree_embedding(tree: &CompTree) -> Vec<f32> {
        match tree {
            CompTree::Atom { embedding, .. } => embedding.clone(),
            CompTree::Compose { op, children } => {
                if children.is_empty() {
                    return vec![];
                }
                let dim = children
                    .iter()
                    .filter_map(|c| {
                        if let CompTree::Atom { embedding, .. } = c {
                            Some(embedding.len())
                        } else {
                            None
                        }
                    })
                    .next()
                    .unwrap_or(32);
                let mut sum = vec![0.0f32; dim];
                let weight_factor = match op {
                    TreeOp::Causes | TreeOp::If => 1.5,    // strong combination
                    TreeOp::And | TreeOp::Sequence => 1.0, // additive
                    TreeOp::Or => 0.7,                     // disjunctive
                    _ => 1.0,
                };
                let n = children.len() as f32;
                for c in children {
                    let e = Self::tree_embedding(c);
                    for d in 0..dim.min(e.len()) {
                        sum[d] += e[d] * weight_factor;
                    }
                }
                for d in 0..dim {
                    sum[d] /= n;
                }
                sum
            }
        }
    }

    /// Flatten the tree into ordered atoms.
    pub fn flatten_to_atoms<'a>(tree: &'a CompTree) -> Vec<&'a str> {
        let mut atoms = Vec::new();
        Self::flatten_helper(tree, &mut atoms);
        atoms
    }
    fn flatten_helper<'a>(tree: &'a CompTree, atoms: &mut Vec<&'a str>) {
        match tree {
            CompTree::Atom { concept_label, .. } => atoms.push(concept_label.as_str()),
            CompTree::Compose { children, .. } => {
                for c in children {
                    Self::flatten_helper(c, atoms);
                }
            }
        }
    }

    /// Render tree as text with indentation.
    pub fn render_tree(tree: &CompTree, indent: usize) -> String {
        match tree {
            CompTree::Atom { concept_label, .. } => {
                format!("{}'{}'", "  ".repeat(indent), concept_label)
            }
            CompTree::Compose { op, children } => {
                let mut out = format!("{}[{:?}]\n", "  ".repeat(indent), op);
                for c in children {
                    out.push_str(&Self::render_tree(c, indent + 1));
                    out.push('\n');
                }
                out.trim_end().to_string()
            }
        }
    }

    pub fn status(&self) -> String {
        format!(
            "Compositional | parses={} | max_depth={}",
            self.n_parses, self.max_depth
        )
    }
}
