// EDEN GARM — Economic Agent (agente económico real)
// El sistema posee recursos que debe producir, intercambiar y consumir.
// No es un tanque de gasolina abstracto: es una economía interna con múltiples
// bienes, producción, depreciación, y mercado de acceso a módulos.
//
// Recursos: compute_credits, knowledge_tokens, trust_points, bandwidth_units
// Produce: successful actions generan recursos.
// Consume: cada operación tiene un costo explícito en múltiples recursos.
// Debe equilibrar presupuesto o entra en recesión (shutdown de módulos).

#[derive(Clone, Debug)]
pub struct EconomicAgent {
    pub compute_credits: f32,
    pub knowledge_tokens: f32,
    pub trust_points: f32,
    pub bandwidth_units: f32,
    pub total_production: f32,
    pub total_consumption: f32,
    pub n_transactions: u64,
    pub n_recessions: u64,
    pub prices: std::collections::HashMap<String, f32>, // cost per unit of each module operation
    pub market_log: Vec<String>,
    pub recession: bool,
}

impl EconomicAgent {
    pub fn new() -> Self {
        let mut prices = std::collections::HashMap::new();
        prices.insert("train".to_string(), 5.0);
        prices.insert("plan".to_string(), 3.0);
        prices.insert("tool".to_string(), 2.0);
        prices.insert("debug".to_string(), 4.0);
        prices.insert("moe".to_string(), 6.0);
        prices.insert("dnc".to_string(), 2.5);
        prices.insert("body".to_string(), 1.5);
        EconomicAgent {
            compute_credits: 200.0,
            knowledge_tokens: 100.0,
            trust_points: 50.0,
            bandwidth_units: 100.0,
            total_production: 0.0,
            total_consumption: 0.0,
            n_transactions: 0,
            n_recessions: 0,
            prices,
            market_log: Vec::new(),
            recession: false,
        }
    }

    /// Produce resources from successful outcomes.
    pub fn produce(&mut self, outcome_type: &str, magnitude: f32) {
        match outcome_type {
            "tool_success" => {
                self.compute_credits += magnitude * 2.0;
                self.trust_points += magnitude * 0.5;
            }
            "learning" => {
                self.knowledge_tokens += magnitude * 1.0;
                self.compute_credits += magnitude * 0.5;
            }
            "discovery" => {
                self.knowledge_tokens += magnitude * 3.0;
                self.trust_points += magnitude * 1.0;
            }
            "build_clean" => {
                self.trust_points += magnitude * 2.0;
                self.bandwidth_units += magnitude * 1.0;
            }
            _ => {}
        }
        self.total_production += magnitude;
        self.n_transactions += 1;
    }

    /// Consume resources to execute an operation. Returns false if can't afford.
    pub fn consume(&mut self, operation: &str, intensity: f32) -> bool {
        let price = self.prices.get(operation).copied().unwrap_or(1.0) * intensity;
        let compute_needed = price * 0.6;
        let bandwidth_needed = price * 0.3;
        let knowledge_needed = price * 0.1;
        if self.compute_credits < compute_needed
            || self.bandwidth_units < bandwidth_needed
            || self.knowledge_tokens < knowledge_needed
        {
            self.recession = true;
            self.n_recessions += 1;
            self.market_log.push(format!(
                "RECESSION: cannot afford {} at price={:.1}",
                operation, price
            ));
            false
        } else {
            self.compute_credits -= compute_needed;
            self.bandwidth_units -= bandwidth_needed;
            self.knowledge_tokens -= knowledge_needed;
            self.total_consumption += price;
            self.n_transactions += 1;
            true
        }
    }

    /// Tick: depreciation and price adjustment based on supply/demand.
    pub fn tick(&mut self) {
        // Depreciation
        self.compute_credits *= 0.99;
        self.knowledge_tokens *= 0.995;
        self.trust_points *= 0.98;
        self.bandwidth_units *= 0.99;
        // Price adjustment: if in recession, lower prices
        if self.recession {
            for (_, price) in self.prices.iter_mut() {
                *price *= 0.95;
            }
            self.recession = false;
        }
        // Floor
        self.compute_credits = self.compute_credits.max(0.0);
        self.knowledge_tokens = self.knowledge_tokens.max(0.0);
        self.trust_points = self.trust_points.max(0.0);
        self.bandwidth_units = self.bandwidth_units.max(0.0);
    }

    /// Net worth
    pub fn net_worth(&self) -> f32 {
        self.compute_credits
            + self.knowledge_tokens * 2.0
            + self.trust_points * 3.0
            + self.bandwidth_units
    }

    pub fn status(&self) -> String {
        format!("EconAgent | CC={:.1} KT={:.1} TP={:.1} BW={:.1} | net={:.1} | prod={:.1} cons={:.1} | recessions={} | txs={}",
            self.compute_credits, self.knowledge_tokens, self.trust_points, self.bandwidth_units,
            self.net_worth(), self.total_production, self.total_consumption, self.n_recessions, self.n_transactions)
    }
}
