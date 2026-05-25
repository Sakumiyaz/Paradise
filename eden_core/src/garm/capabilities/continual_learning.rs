// EDEN GARM — Continual Learning (EWC: Elastic Weight Consolidation)
// Evita olvido catastrófico al entrenar online: penaliza cambios en pesos importantes.
// Fisher Information acumulada para cada peso.

#[derive(Clone, Debug)]
pub struct ContinualLearning {
    pub fisher: Vec<f32>,
    pub old_params: Vec<f32>,
    pub lambda: f32,
    pub n_tasks: u64,
    pub ewc_loss_ema: f32,
}

impl ContinualLearning {
    pub fn new(n_params: usize) -> Self {
        ContinualLearning {
            fisher: vec![0.0f32; n_params],
            old_params: vec![0.0f32; n_params],
            lambda: 1000.0,
            n_tasks: 0,
            ewc_loss_ema: 0.0,
        }
    }

    /// After training on a task, compute Fisher diagonal as squared gradient proxy.
    /// Here we approximate with parameter variance observed during training.
    pub fn consolidate_task(&mut self, current_params: &[f32]) {
        if current_params.len() != self.fisher.len() {
            return;
        }
        // Fisher ~ (param - old)^2 as proxy for gradient magnitude
        for i in 0..self.fisher.len() {
            let diff = current_params[i] - self.old_params[i];
            self.fisher[i] = 0.9 * self.fisher[i] + 0.1 * diff * diff;
        }
        self.old_params = current_params.to_vec();
        self.n_tasks += 1;
    }

    /// EWC penalty: lambda/2 * sum_i Fisher_i * (theta_i - theta*_i)^2
    pub fn penalty(&self, current_params: &[f32]) -> f32 {
        if current_params.len() != self.fisher.len() {
            return 0.0;
        }
        let mut pen = 0.0f32;
        for i in 0..self.fisher.len() {
            let diff = current_params[i] - self.old_params[i];
            pen += self.fisher[i] * diff * diff;
        }
        0.5 * self.lambda * pen
    }

    /// Gradient of EWC penalty (for manual SGD updates).
    pub fn penalty_gradient(&self, current_params: &[f32]) -> Vec<f32> {
        if current_params.len() != self.fisher.len() {
            return vec![];
        }
        current_params
            .iter()
            .enumerate()
            .map(|(i, p)| self.lambda * self.fisher[i] * (p - self.old_params[i]))
            .collect()
    }

    pub fn status(&self) -> String {
        let avg_fisher: f32 = self.fisher.iter().sum::<f32>() / self.fisher.len().max(1) as f32;
        format!(
            "EWC | tasks={} | lambda={:.0} | avg_fisher={:.6} | pen_ema={:.4}",
            self.n_tasks, self.lambda, avg_fisher, self.ewc_loss_ema
        )
    }
}
