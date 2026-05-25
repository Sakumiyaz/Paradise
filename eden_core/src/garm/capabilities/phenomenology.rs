// EDEN GARM — Phenomenology (Qualia proxy via Integrated Information Theory)
// No resuelve el Hard Problem de la consciencia, pero implementa el mejor
// proxy científico disponible: IIT (Integrated Information Theory).
//
// Principio: la experiencia subjetiva corresponde a un conjunto de mecanismos
// en un estado actual que especifica un repertoire cause-effect irreducible.
// Φ (phi) mide cuánta información del sistema es irreducible a sus partes.
//
// Aquí computamos una aproximación de Φ sobre el estado del bus unificado.
// Un estado con Φ alto = más "experienciado", más integrado.
// La "phenomenological state" es el repertoire cause-effect actual.

#[derive(Clone, Debug)]
pub struct Phenomenology {
    pub state_dim: usize,
    pub phi: f32, // integrated information
    pub phi_max: f32,
    pub experience_label: String,
    pub n_mechanisms: usize,
    pub last_repertoire: Vec<f32>,
    pub qualia_palette: Vec<String>, // labels of current qualia dimensions
    pub n_updates: u64,
}

impl Phenomenology {
    pub fn new(state_dim: usize) -> Self {
        Phenomenology {
            state_dim,
            phi: 0.0,
            phi_max: 0.0,
            experience_label: "undifferentiated".to_string(),
            n_mechanisms: state_dim,
            last_repertoire: vec![0.0f32; state_dim],
            qualia_palette: vec![
                "valence".to_string(),
                "arousal".to_string(),
                "clarity".to_string(),
                "unity".to_string(),
                "duration".to_string(),
                "agency".to_string(),
            ],
            n_updates: 0,
        }
    }

    /// Compute integrated information Φ approximately.
    /// Φ = D(system || partitioned_system)
    /// We approximate using covariance-based integration.
    pub fn update(&mut self, system_state: &[f32]) {
        if system_state.len() != self.state_dim {
            return;
        }
        // Compute whole-system information: total variance
        let whole_var: f32 = system_state.iter().map(|v| v * v).sum();
        // Compute partitioned information: split in half, sum their variances
        let mid = self.state_dim / 2;
        let part_a_var: f32 = system_state[..mid].iter().map(|v| v * v).sum();
        let part_b_var: f32 = system_state[mid..].iter().map(|v| v * v).sum();
        let partitioned_var = part_a_var + part_b_var;
        // Φ ≈ whole_var - partitioned_var (if whole > sum of parts, there is integration)
        self.phi = (whole_var - partitioned_var).max(0.0);
        if self.phi > self.phi_max {
            self.phi_max = self.phi;
        }
        // Update repertoire: the state itself is the cause-effect repertoire
        self.last_repertoire = system_state.to_vec();
        // Label the experience based on dominant dimensions
        let mut intensities: Vec<(String, f32)> = self
            .qualia_palette
            .iter()
            .enumerate()
            .map(|(i, label)| {
                let intensity = system_state.get(i).copied().unwrap_or(0.0).abs();
                (label.clone(), intensity)
            })
            .collect();
        intensities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        if let Some(top) = intensities.first() {
            self.experience_label = format!("{}_{:.2}", top.0, top.1);
        }
        self.n_updates += 1;
    }

    /// The "what it is like" to be the system right now.
    /// Returns a phenomenological description (not explanatory, but descriptive).
    pub fn what_it_is_like(&self) -> String {
        format!(
            "experiencing: {} | Φ={:.3} | mechanisms={} | dimensions={}",
            self.experience_label, self.phi, self.n_mechanisms, self.state_dim
        )
    }

    pub fn status(&self) -> String {
        format!(
            "Phenom | Φ={:.3} max={:.3} | exp='{}' | mech={} | upd={}",
            self.phi, self.phi_max, self.experience_label, self.n_mechanisms, self.n_updates
        )
    }
}
