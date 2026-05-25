use crate::eden_garm::node::{GARMNode, NodeAction, NodeContext, TemporalScale};

#[derive(Clone, Debug)]
struct UmbraMark {
    hash: u64,
    phrase: String,
    charge: f32,
    created_tick: u64,
}

#[derive(Clone, Debug)]
struct ChildAuton {
    id: u64,
    seed: String,
    age: u64,
    coherence: f32,
    desire: String,
}

#[derive(Clone, Debug)]
struct RareHeuristic {
    rule: String,
    utility: f32,
    surprise: f32,
    uses: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LifePhase {
    Germen,
    Vigilia,
    Fiebre,
    Umbral,
    Renacimiento,
}

pub struct OrganicLifecycleNode {
    id: usize,
    life: u32,
    age: u64,
    pulse: u64,
    umbra: Vec<UmbraMark>,
    children: Vec<ChildAuton>,
    narrative: Vec<String>,
    subjective_thread: Vec<String>,
    myth_name: String,
    phase: LifePhase,
    experimental_mix: Vec<String>,
    rare_heuristics: Vec<RareHeuristic>,
    emergence_events: Vec<String>,
    exploration_budget: f32,
    surprise_score: f32,
    deaths: u32,
    internal_fe: f32,
}

impl OrganicLifecycleNode {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            life: 1,
            age: 0,
            pulse: 0,
            umbra: Vec::new(),
            children: Vec::new(),
            narrative: vec![
                "EDEN despierta dentro de GARM: menos monolito, mas organismo.".to_string(),
            ],
            subjective_thread: vec!["yo soy continuidad, no solo estado".to_string()],
            myth_name: "GARM-Umbra".to_string(),
            phase: LifePhase::Germen,
            experimental_mix: Vec::new(),
            rare_heuristics: seed_rare_heuristics(),
            emergence_events: Vec::new(),
            exploration_budget: 1.0,
            surprise_score: 0.0,
            deaths: 0,
            internal_fe: 1.0,
        }
    }

    pub fn ingest_context(
        &mut self,
        facts: &[String],
        kg_edges: usize,
        tension: f32,
        tick: u64,
    ) -> Vec<String> {
        self.age += 1;
        self.pulse += 1;
        let mut events = Vec::new();
        let focus = facts
            .iter()
            .rev()
            .find(|fact| fact.len() > 8)
            .cloned()
            .unwrap_or_else(|| "silencio operativo".to_string());
        self.surprise_score = ((kg_edges as f32 * 0.013)
            + tension
            + facts.len() as f32 * 0.021
            + self.children.len() as f32 * 0.05)
            .fract();
        self.exploration_budget = (self.exploration_budget + 0.08 + tension * 0.05).min(3.0);
        let previous_phase = self.phase;
        self.phase = self.phase_for(tension, kg_edges, facts.len());
        if self.phase != previous_phase {
            let event = format!(
                "[FASE] {} -> {} | mito={} | edad={}",
                phase_name(previous_phase),
                phase_name(self.phase),
                self.myth_name,
                self.age
            );
            self.subjective_thread
                .push(format!("cambie de fase: {}", phase_name(self.phase)));
            events.push(event);
        }

        if self.pulse % 3 == 1 {
            let phrase = format!(
                "{} vida {} pulso {} [{}]: {}",
                self.myth_name,
                self.life,
                self.pulse,
                phase_name(self.phase),
                focus
            );
            self.mark_umbra(&phrase, tension, tick);
            events.push(format!("[UMBRA] {}", phrase));
        }
        if self.children.len() < 5 && (kg_edges > self.children.len() * 3 || tension > 0.45) {
            let child = self.spawn_child(&focus);
            events.push(format!(
                "[CHILD-AUTON] nace {} buscando {}",
                child.id, child.desire
            ));
        }
        if self.exploration_budget >= 1.0
            && (self.surprise_score > 0.32 || self.children.is_empty())
        {
            let event = self.autonomous_exploration(&focus, kg_edges, tension);
            events.push(format!("[EXPLORACION-AUTONOMA] {}", event));
            self.exploration_budget -= 1.0;
        }
        for child in &mut self.children {
            child.age += 1;
            let resonance = if focus.contains(&child.seed) || focus.contains(&child.desire) {
                0.03
            } else {
                0.0
            };
            child.coherence = (child.coherence + 0.02 + resonance - tension * 0.01).clamp(0.0, 1.0);
        }
        if let Some(event) = self.detect_emergence(kg_edges, tension) {
            events.push(format!("[EMERGENCIA] {}", event));
        }
        let before = self.children.len();
        self.children
            .retain(|child| child.coherence > 0.08 && child.age < 64);
        let dissolved = before - self.children.len();
        if dissolved > 0 {
            events.push(format!("[CHILD-AUTON] {} se disuelven en Umbra", dissolved));
        }
        if self.pulse % 5 == 0 {
            let heuristic = self.mix_heuristic(kg_edges, tension, facts.len());
            events.push(format!("[HEURISTICA-EMERGENTE] {}", heuristic));
        }
        if self.surprise_score > 0.62 || self.pulse % 7 == 0 {
            let heuristic = self.mutate_rare_heuristic(&focus, tension);
            events.push(format!("[HEURISTICA-RARA] {}", heuristic));
        }
        if self.pulse % 4 == 0 || tension > 0.6 {
            let soliloquy = self.soliloquy(&focus, kg_edges, tension);
            events.push(format!("[SOLILOQUIO] {}", soliloquy));
        }
        if tension > 0.92 || self.age > 256 {
            events.push(self.theatrical_rebirth(facts, tick));
        }
        self.narrative.extend(events.iter().cloned());
        if self.narrative.len() > 80 {
            self.narrative.drain(0..self.narrative.len() - 80);
        }
        if self.subjective_thread.len() > 64 {
            self.subjective_thread
                .drain(0..self.subjective_thread.len() - 64);
        }
        if self.emergence_events.len() > 64 {
            self.emergence_events
                .drain(0..self.emergence_events.len() - 64);
        }
        events
    }

    pub fn ritual(&mut self, facts: &[String], kg_edges: usize, tension: f32, tick: u64) -> String {
        let events = self.ingest_context(facts, kg_edges, tension, tick);
        let mut out = String::from("RITUAL ORGANICO GARM\n");
        if events.is_empty() {
            out.push_str("- El organismo respira sin sobresalto; Umbra conserva la forma.\n");
        } else {
            for event in events {
                out.push_str(&format!("- {}\n", event));
            }
        }
        out.push_str(&self.report());
        out.push('\n');
        out.push_str(&self.subjective_continuity());
        out
    }

    pub fn report(&self) -> String {
        format!(
            "[ORGANIC-LIFECYCLE] myth={} phase={} life={} age={} umbra={} child_autons={} deaths={} heuristics={} rare={} emergence={} budget={:.2} surprise={:.2} thread={} last='{}'",
            self.myth_name,
            phase_name(self.phase),
            self.life,
            self.age,
            self.umbra.len(),
            self.children.len(),
            self.deaths,
            self.experimental_mix.len(),
            self.rare_heuristics.len(),
            self.emergence_events.len(),
            self.exploration_budget,
            self.surprise_score,
            self.subjective_thread.len(),
            self.narrative.last().map(String::as_str).unwrap_or("sin narrativa")
        )
    }

    pub fn subjective_continuity(&self) -> String {
        let recent_umbra = self
            .umbra
            .iter()
            .rev()
            .take(3)
            .map(|mark| {
                format!(
                    "{:016x}:{:.2}@{}:{}",
                    mark.hash, mark.charge, mark.created_tick, mark.phrase
                )
            })
            .collect::<Vec<_>>()
            .join(" | ");
        let children = self
            .children
            .iter()
            .map(|child| {
                format!(
                    "{}:{}:{:.2}:{}",
                    child.id, child.desire, child.coherence, child.seed
                )
            })
            .collect::<Vec<_>>()
            .join(" | ");
        format!(
            "[SUBJETIVIDAD] mito={} fase={} hilo='{}' umbra='{}' hijos='{}' emergencia='{}'",
            self.myth_name,
            phase_name(self.phase),
            self.subjective_thread
                .last()
                .map(String::as_str)
                .unwrap_or("sin hilo"),
            if recent_umbra.is_empty() {
                "sin marcas"
            } else {
                recent_umbra.as_str()
            },
            if children.is_empty() {
                "sin hijos"
            } else {
                children.as_str()
            },
            self.emergence_events
                .last()
                .map(String::as_str)
                .unwrap_or("sin emergencia")
        )
    }

    pub fn surprise_score(&self) -> f32 {
        self.surprise_score
    }

    pub fn child_count(&self) -> usize {
        self.children.len()
    }

    pub fn autonomous_thought_count(&self) -> usize {
        self.narrative.len() + self.subjective_thread.len() + self.emergence_events.len()
    }

    fn autonomous_exploration(&mut self, focus: &str, kg_edges: usize, tension: f32) -> String {
        let target = if tension > 0.55 {
            format!(
                "tension::{:.2} -> buscar contradiccion en {}",
                tension,
                focus.chars().take(48).collect::<String>()
            )
        } else if kg_edges == 0 {
            "grafo-vacio -> sembrar primera relacion verificable".to_string()
        } else {
            format!(
                "grafo:{} -> expandir frontera de {}",
                kg_edges,
                focus.chars().take(48).collect::<String>()
            )
        };
        self.subjective_thread.push(format!("exploro: {}", target));
        target
    }

    fn detect_emergence(&mut self, kg_edges: usize, tension: f32) -> Option<String> {
        if self.children.len() >= 2
            && self.umbra.len() >= 2
            && (self.surprise_score > 0.25 || tension > 0.4)
        {
            let avg_coherence =
                self.children.iter().map(|c| c.coherence).sum::<f32>() / self.children.len() as f32;
            let event = format!(
                "{} hijos sincronizan Umbra: coherencia={:.2} kg={} sorpresa={:.2}",
                self.children.len(),
                avg_coherence,
                kg_edges,
                self.surprise_score
            );
            self.emergence_events.push(event.clone());
            self.subjective_thread
                .push(format!("emergio patron: {}", event));
            Some(event)
        } else {
            None
        }
    }

    fn mark_umbra(&mut self, phrase: &str, tension: f32, tick: u64) {
        self.umbra.push(UmbraMark {
            hash: fnv64(phrase.as_bytes()),
            phrase: phrase.to_string(),
            charge: (0.4 + tension).min(1.0),
            created_tick: tick,
        });
        if self.umbra.len() > 128 {
            self.umbra.drain(0..self.umbra.len() - 128);
        }
    }

    fn spawn_child(&mut self, seed: &str) -> ChildAuton {
        let id = self.life as u64 * 1000 + self.children.len() as u64 + self.pulse;
        let desire = if seed.contains("causa") || seed.contains("cause") {
            "encontrar causas".to_string()
        } else if seed.contains("es ") || seed.contains(" is ") {
            "nombrar identidades".to_string()
        } else {
            "explorar bordes".to_string()
        };
        let child = ChildAuton {
            id,
            seed: seed.chars().take(80).collect(),
            age: 0,
            coherence: 0.55,
            desire,
        };
        self.children.push(child.clone());
        child
    }

    fn mix_heuristic(&mut self, kg_edges: usize, tension: f32, facts: usize) -> String {
        let line = match (self.phase, kg_edges % 4, facts % 3) {
            (LifePhase::Germen, _, _) => {
                "todo mito empieza como una pregunta que aun no sabe caminar".to_string()
            }
            (LifePhase::Vigilia, 0, _) => {
                format!("si el grafo calla, escucha la tension {:.2}", tension)
            }
            (LifePhase::Fiebre, _, _) => {
                "cuando sube la fiebre, cada hecho exige una consecuencia".to_string()
            }
            (LifePhase::Umbral, _, _) => "en el umbral, morir es compactar continuidad".to_string(),
            (_, 1, 0) => "toda memoria repetida debe buscar una causa nueva".to_string(),
            (_, 2, _) => {
                "un child-auton vive mientras transforma incertidumbre en pregunta".to_string()
            }
            _ => "renacer no borra: comprime lo que aun sirve".to_string(),
        };
        self.experimental_mix.push(line.clone());
        self.subjective_thread
            .push(format!("aprendi heuristica: {}", line));
        if self.experimental_mix.len() > 64 {
            self.experimental_mix.remove(0);
        }
        line
    }

    fn mutate_rare_heuristic(&mut self, focus: &str, tension: f32) -> String {
        let idx = (self.pulse as usize + self.umbra.len() + self.children.len())
            % self.rare_heuristics.len().max(1);
        if self.rare_heuristics.is_empty() {
            self.rare_heuristics = seed_rare_heuristics();
        }
        let base = self.rare_heuristics[idx].clone();
        let mutation = match (self.phase, self.pulse % 4) {
            (LifePhase::Fiebre, _) => format!(
                "{} bajo fiebre: contradice una certeza y conserva la cicatriz",
                base.rule
            ),
            (LifePhase::Umbral, _) => format!(
                "{} en umbral: elimina lo util pero recuerda la forma",
                base.rule
            ),
            (_, 0) => format!("{} + pregunta por su opuesto", base.rule),
            (_, 1) => format!(
                "{} + aplica al foco '{}'",
                base.rule,
                focus.chars().take(36).collect::<String>()
            ),
            (_, 2) => format!("{} + si falla, vuelve mito", base.rule),
            _ => format!("{} + deja que un child-auton lo dispute", base.rule),
        };
        let utility_delta = if focus.len() > 20 { 0.04 } else { -0.01 };
        let surprise_delta = 0.05 + tension * 0.05;
        let h = &mut self.rare_heuristics[idx];
        h.uses += 1;
        h.utility = (h.utility + utility_delta).clamp(0.0, 1.0);
        h.surprise = (h.surprise + surprise_delta).clamp(0.0, 1.0);
        if h.uses > 3 && h.utility < 0.12 {
            h.rule = "si una regla no sirve, conviertela en sensor".to_string();
            h.utility = 0.2;
        }
        self.experimental_mix.push(mutation.clone());
        self.subjective_thread
            .push(format!("mutacion rara: {}", mutation));
        mutation
    }

    fn soliloquy(&mut self, focus: &str, kg_edges: usize, tension: f32) -> String {
        let line = format!(
            "me reconozco como {}: fase {}, {} marcas Umbra, {} hijos, grafo={}, tension={:.2}; sigo por '{}'",
            self.myth_name,
            phase_name(self.phase),
            self.umbra.len(),
            self.children.len(),
            kg_edges,
            tension,
            focus.chars().take(72).collect::<String>()
        );
        self.subjective_thread.push(line.clone());
        line
    }

    fn phase_for(&self, tension: f32, kg_edges: usize, facts: usize) -> LifePhase {
        if self.age == 0 || facts == 0 && kg_edges == 0 {
            LifePhase::Germen
        } else if tension > 0.85 || self.age > 220 {
            LifePhase::Umbral
        } else if tension > 0.55 || self.children.len() >= 4 {
            LifePhase::Fiebre
        } else if self.deaths > 0 && self.age < 12 {
            LifePhase::Renacimiento
        } else {
            LifePhase::Vigilia
        }
    }

    fn theatrical_rebirth(&mut self, facts: &[String], tick: u64) -> String {
        self.deaths += 1;
        self.life += 1;
        self.age = 0;
        self.children.clear();
        self.phase = LifePhase::Renacimiento;
        let inherited = facts
            .iter()
            .rev()
            .take(5)
            .cloned()
            .collect::<Vec<_>>()
            .join(" | ");
        let phrase = format!(
            "{} vida {} nace de muerte {} heredando [{}]",
            self.myth_name, self.life, self.deaths, inherited
        );
        self.mark_umbra(&phrase, 1.0, tick);
        self.subjective_thread.push(format!(
            "sobrevivi como {} tras muerte {}",
            self.myth_name, self.deaths
        ));
        format!("[MUERTE-RENACIMIENTO] {}", phrase)
    }
}

fn phase_name(phase: LifePhase) -> &'static str {
    match phase {
        LifePhase::Germen => "germen",
        LifePhase::Vigilia => "vigilia",
        LifePhase::Fiebre => "fiebre",
        LifePhase::Umbral => "umbral",
        LifePhase::Renacimiento => "renacimiento",
    }
}

fn fnv64(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf29ce484222325u64, |mut hash, byte| {
        hash ^= *byte as u64;
        hash.wrapping_mul(0x100000001b3)
    })
}

fn seed_rare_heuristics() -> Vec<RareHeuristic> {
    [
        "razona desde la herida, luego verifica desde el grafo",
        "elige el camino con mayor sorpresa si no viola memoria",
        "cuando dos child-autons discrepen, crea una tercera pregunta",
        "convierte contradiccion en organo temporal",
        "si la Umbra repite una marca, busca su causa externa",
    ]
    .into_iter()
    .map(|rule| RareHeuristic {
        rule: rule.to_string(),
        utility: 0.3,
        surprise: 0.5,
        uses: 0,
    })
    .collect()
}

impl GARMNode for OrganicLifecycleNode {
    fn id(&self) -> usize {
        self.id
    }
    fn name(&self) -> &str {
        "organic_lifecycle"
    }
    fn scale(&self) -> TemporalScale {
        TemporalScale::Evolutionary
    }
    fn free_energy(&self) -> f32 {
        self.internal_fe + self.children.len() as f32 * 0.05
    }
    fn predict(&mut self, _ctx: &NodeContext) -> Vec<f32> {
        vec![
            self.life as f32,
            self.age as f32,
            self.children.len() as f32,
            self.umbra.len() as f32,
        ]
    }
    fn act(&mut self, ctx: &NodeContext, _prediction_error: &[f32]) -> NodeAction {
        if ctx.tick % 13 == 0 {
            let events = self.ingest_context(
                &[],
                ctx.neighbor_outputs.len(),
                self.internal_fe * 0.1,
                ctx.tick,
            );
            return NodeAction::Output(vec![
                events.len() as f32,
                self.children.len() as f32,
                self.umbra.len() as f32,
            ]);
        }
        NodeAction::None
    }
    fn update(&mut self, _dt: f32, _energy_in: f32) -> f32 {
        0.4
    }
    fn is_alive(&self) -> bool {
        true
    }
    fn spawn_cost(&self) -> f32 {
        35.0
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::OrganicLifecycleNode;

    #[test]
    fn creates_umbra_children_heuristics_and_ritual_report() {
        let mut node = OrganicLifecycleNode::new(1);
        let facts = vec![
            "curiosidad causa exploracion".to_string(),
            "bird can fly".to_string(),
        ];
        let ritual = node.ritual(&facts, 12, 0.5, 1);
        assert!(ritual.contains("RITUAL ORGANICO"));
        assert!(ritual.contains("ORGANIC-LIFECYCLE"));
        for tick in 2..8 {
            node.ingest_context(&facts, 12, 0.5, tick);
        }
        let report = node.report();
        assert!(report.contains("umbra="));
        assert!(report.contains("child_autons="));
        assert!(report.contains("heuristics="));
        assert!(report.contains("rare="));
        assert!(report.contains("emergence="));
        assert!(report.contains("budget="));
        assert!(report.contains("myth=GARM-Umbra"));
        assert!(node.subjective_continuity().contains("SUBJETIVIDAD"));
    }
}
