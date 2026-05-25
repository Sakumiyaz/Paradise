// EDEN GARM Evolution — Real evolutionary strategies on neural weights
// Population of weight vectors competing on prediction accuracy (fitness).
// No LLM. Pure Rust. Evolves the neural substrate in vivo.

use rand::Rng;

#[derive(Clone)]
pub struct Individual {
    pub genome: Vec<f32>,
    pub fitness: f32,
    pub generation: u32,
    pub species_id: usize,
}

impl Individual {
    /// Learning rate encoded in gene[0], clamped to [0.001, 0.5].
    pub fn get_lr(&self) -> f32 {
        self.genome.get(0).copied().unwrap_or(0.1).clamp(0.001, 0.5)
    }

    /// Hidden-size factor encoded in gene[1], clamped to [4, 32] and rounded to integer.
    pub fn get_hidden_size(&self) -> usize {
        self.genome
            .get(1)
            .copied()
            .unwrap_or(18.0)
            .round()
            .clamp(4.0, 32.0) as usize
    }

    /// Self-adaptive mutation rate encoded in gene[2], clamped to [0.01, 0.5].
    pub fn get_mutation_rate(&self) -> f32 {
        self.genome.get(2).copied().unwrap_or(0.1).clamp(0.01, 0.5)
    }
}

pub struct Population {
    pub individuals: Vec<Individual>,
    pub pop_size: usize,
    pub mutation_rate: f32,
    pub mutation_strength: f32,
    pub elitism: usize,
    pub generation: u32,
    pub genome_size: usize,
    pub speciation_threshold: usize,
    pub hall_of_fame: Vec<Individual>,
}

impl Population {
    pub fn new(weights_size: usize, pop_size: usize) -> Self {
        let mut rng = rand::thread_rng();
        let genome_size = weights_size.saturating_add(3);
        let mut individuals = Vec::with_capacity(pop_size);
        for _ in 0..pop_size {
            let mut genome: Vec<f32> = Vec::with_capacity(genome_size);
            // gene[0] = learning rate (clamped 0.001..0.5)
            genome.push(rng.gen::<f32>() * 0.499 + 0.001);
            // gene[1] = hidden_size factor (clamped 4..32, integer stored as f32)
            genome.push(rng.gen::<f32>() * 28.0 + 4.0);
            // gene[2] = mutation rate self-adaptive (clamped 0.01..0.5)
            genome.push(rng.gen::<f32>() * 0.49 + 0.01);
            // The rest are the neural weights
            for _ in 3..genome_size {
                genome.push(rng.gen::<f32>() * 0.2 - 0.1);
            }
            individuals.push(Individual {
                genome,
                fitness: 0.0,
                generation: 0,
                species_id: 0,
            });
        }
        Population {
            individuals,
            pop_size,
            mutation_rate: 0.1,
            mutation_strength: 0.05,
            elitism: 2,
            generation: 0,
            genome_size,
            speciation_threshold: (genome_size / 4).max(1),
            hall_of_fame: Vec::new(),
        }
    }

    /// Pairwise Hamming distance between two genomes using binary threshold |a-b| > 0.05.
    fn hamming_distance(a: &[f32], b: &[f32]) -> usize {
        a.iter()
            .zip(b.iter())
            .filter(|(&x, &y)| (x - y).abs() > 0.05)
            .count()
    }

    /// Group individuals into species based on Hamming distance < speciation_threshold.
    fn assign_species(&mut self) {
        if self.individuals.is_empty() {
            return;
        }
        let threshold = self.speciation_threshold.max(1);
        for ind in &mut self.individuals {
            ind.species_id = 0;
        }
        let mut representatives: Vec<usize> = vec![0];
        let mut next_id = 1usize;
        for i in 1..self.individuals.len() {
            let mut assigned = false;
            for (sid, &rep_idx) in representatives.iter().enumerate() {
                let dist = Self::hamming_distance(
                    &self.individuals[i].genome,
                    &self.individuals[rep_idx].genome,
                );
                if dist < threshold {
                    self.individuals[i].species_id = sid;
                    assigned = true;
                    break;
                }
            }
            if !assigned {
                self.individuals[i].species_id = next_id;
                representatives.push(i);
                next_id += 1;
            }
        }
    }

    /// Number of distinct species in the current population.
    pub fn species_count(&self) -> usize {
        if self.individuals.is_empty() {
            return 0;
        }
        self.individuals
            .iter()
            .map(|ind| ind.species_id)
            .max()
            .map(|m| m + 1)
            .unwrap_or(0)
    }

    /// Evaluate fitness for each individual. fitness_fn returns higher = better.
    pub fn evaluate<F>(&mut self, mut fitness_fn: F)
    where
        F: FnMut(&[f32]) -> f32,
    {
        self.assign_species();
        for ind in &mut self.individuals {
            ind.fitness = fitness_fn(&ind.genome[3..]);
        }
        // Fitness sharing: adjusted_fitness = raw_fitness / species_size
        let mut species_sizes = std::collections::HashMap::new();
        for ind in &self.individuals {
            *species_sizes.entry(ind.species_id).or_insert(0usize) += 1;
        }
        for ind in &mut self.individuals {
            let size = species_sizes
                .get(&ind.species_id)
                .copied()
                .unwrap_or(1)
                .max(1);
            ind.fitness = ind.fitness / size as f32;
        }
        // Sort descending by adjusted fitness
        self.individuals.sort_by(|a, b| {
            b.fitness
                .partial_cmp(&a.fitness)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
    }

    /// One generation of evolution: elitism + tournament selection + Gaussian mutation + crossover.
    pub fn evolve(&mut self) {
        let mut rng = rand::thread_rng();
        let mut new_pop = Vec::with_capacity(self.pop_size);
        // Elitism
        for i in 0..self.elitism.min(self.individuals.len()) {
            new_pop.push(Individual {
                genome: self.individuals[i].genome.clone(),
                fitness: self.individuals[i].fitness,
                generation: self.generation + 1,
                species_id: self.individuals[i].species_id,
            });
        }
        // Generate offspring
        while new_pop.len() < self.pop_size {
            let parent = self.tournament_select(3, &mut rng);
            let mut child = parent.clone();
            // Self-adaptive mutation: use each individual's own gene[2] as its mutation rate
            let mr = child.get_mutation_rate();
            for gene in &mut child.genome {
                if rng.gen::<f32>() < mr {
                    *gene +=
                        rng.gen::<f32>() * self.mutation_strength * 2.0 - self.mutation_strength;
                }
            }
            // Occasional crossover with another parent
            if rng.gen::<f32>() < 0.3 && self.individuals.len() > 1 {
                let other = self.tournament_select(3, &mut rng);
                let crossover_point = rng.gen_range(0..self.genome_size);
                for i in crossover_point..self.genome_size {
                    child.genome[i] = other.genome[i];
                }
            }
            child.fitness = 0.0;
            child.generation = self.generation + 1;
            child.species_id = 0; // will be reassigned in next evaluate
            new_pop.push(child);
        }
        self.individuals = new_pop;
        self.generation += 1;
    }

    fn tournament_select<R: Rng + ?Sized>(&self, k: usize, rng: &mut R) -> Individual {
        let mut best = &self.individuals[rng.gen_range(0..self.individuals.len())];
        for _ in 1..k {
            let candidate = &self.individuals[rng.gen_range(0..self.individuals.len())];
            if candidate.fitness > best.fitness {
                best = candidate;
            }
        }
        best.clone()
    }

    pub fn best_genome(&self) -> Option<&[f32]> {
        self.individuals.first().map(|ind| &ind.genome[3..])
    }

    pub fn best_fitness(&self) -> f32 {
        self.individuals
            .first()
            .map(|ind| ind.fitness)
            .unwrap_or(0.0)
    }

    pub fn mean_fitness(&self) -> f32 {
        let sum: f32 = self.individuals.iter().map(|ind| ind.fitness).sum();
        sum / self.individuals.len().max(1) as f32
    }

    pub fn update_hall_of_fame(&mut self) {
        for ind in &self.individuals {
            self.hall_of_fame.push(ind.clone());
        }
        self.hall_of_fame.sort_by(|a, b| {
            b.fitness
                .partial_cmp(&a.fitness)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        self.hall_of_fame.truncate(3);
    }

    pub fn status(&self) -> String {
        format!(
            "Evolution | gen: {} | pop: {} | best_fit: {:.4} | mean_fit: {:.4}",
            self.generation,
            self.pop_size,
            self.best_fitness(),
            self.mean_fitness()
        )
    }
}
