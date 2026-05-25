//! # GENETIC - Algoritmo genético para optimizar subagentes desde cero
#![allow(dead_code)]
#![allow(non_snake_case)]

/// Cromosoma = weights de un agente
#[derive(Debug, Clone)]
pub struct Chromosome {
    pub genes: Vec<f32>,
    fitness: f32,
}

impl Chromosome {
    pub fn new(genes: Vec<f32>) -> Self {
        Self {
            genes,
            fitness: 0.0,
        }
    }

    /// Crear cromosoma aleatorio
    pub fn random(size: usize) -> Self {
        let genes = (0..size)
            .map(|_| rand::random::<f32>() * 2.0 - 1.0)
            .collect();
        Self::new(genes)
    }

    /// Cruza con otro cromosoma
    pub fn crossover(&self, other: &Chromosome) -> Chromosome {
        let mut child_genes = Vec::new();
        for i in 0..self.genes.len() {
            if rand::random::<bool>() {
                child_genes.push(self.genes[i]);
            } else {
                child_genes.push(other.genes[i]);
            }
        }
        Chromosome::new(child_genes)
    }

    /// Mutación
    pub fn mutate(&mut self, rate: f32) {
        for gene in &mut self.genes {
            if rand::random::<f32>() < rate {
                *gene += (rand::random::<f32>() - 0.5) * 0.2;
            }
        }
    }

    pub fn set_fitness(&mut self, fitness: f32) {
        self.fitness = fitness;
    }

    pub fn get_fitness(&self) -> f32 {
        self.fitness
    }
}

/// Función de fitness personalizada
pub type FitnessFunction = fn(chromosome: &Chromosome, generation: usize) -> f32;

/// Algoritmo genético
#[derive(Debug, Clone)]
pub struct GeneticOptimizer {
    pub population: Vec<Chromosome>, // Made public for EDEN
    population_size: usize,
    mutation_rate: f32,
    crossover_rate: f32,
    elite_count: usize,
    generation: usize,
}

impl GeneticOptimizer {
    pub fn new(population_size: usize, chromosome_size: usize) -> Self {
        let population = (0..population_size)
            .map(|_| Chromosome::random(chromosome_size))
            .collect();

        Self {
            population,
            population_size,
            mutation_rate: 0.1,
            crossover_rate: 0.8,
            elite_count: population_size / 10,
            generation: 0,
        }
    }

    /// Evaluar población
    pub fn evaluate(&mut self, fitness_fn: &FitnessFunction) {
        for chromosome in &mut self.population {
            let fitness = fitness_fn(chromosome, self.generation);
            chromosome.set_fitness(fitness);
        }

        // Ordenar por fitness descendente
        self.population
            .sort_by(|a, b| b.get_fitness().partial_cmp(&a.get_fitness()).unwrap());
    }

    /// Ejecutar siguiente generación
    pub fn next_generation(&mut self) {
        let mut new_population = Vec::new();

        // Elite - mejores sobreviven
        for i in 0..self.elite_count {
            new_population.push(self.population[i].clone());
        }

        // Crear nueva descendencia
        while new_population.len() < self.population_size {
            // Selección por ruleta (simplificado)
            let parent1 = self.select_parent();
            let parent2 = self.select_parent();

            let mut child = parent1.crossover(&parent2);

            if rand::random::<f32>() < self.mutation_rate {
                child.mutate(self.mutation_rate);
            }

            new_population.push(child);
        }

        self.population = new_population;
        self.generation += 1;
    }

    /// Seleccionar padre (ruleta simple)
    fn select_parent(&self) -> &Chromosome {
        let total_fitness: f32 = self.population.iter().map(|c| c.get_fitness()).sum();

        if total_fitness == 0.0 {
            return &self.population[rand::random::<usize>() % self.population.len()];
        }

        let threshold = rand::random::<f32>() * total_fitness;
        let mut accumulated = 0.0;

        for chromosome in &self.population {
            accumulated += chromosome.get_fitness();
            if accumulated >= threshold {
                return chromosome;
            }
        }

        &self.population[0]
    }

    /// Obtener mejor cromosoma
    pub fn best(&self) -> Option<&Chromosome> {
        self.population.first()
    }

    /// Estadísticas
    pub fn stats(&self) -> (f32, f32, f32) {
        let fitness_sum: f32 = self.population.iter().map(|c| c.get_fitness()).sum();
        let avg = fitness_sum / self.population.len() as f32;

        let best = self
            .population
            .first()
            .map(|c| c.get_fitness())
            .unwrap_or(0.0);
        let worst = self
            .population
            .last()
            .map(|c| c.get_fitness())
            .unwrap_or(0.0);

        (best, avg, worst)
    }

    /// Set mutation rate
    pub fn set_mutation_rate(&mut self, rate: f32) {
        self.mutation_rate = rate;
    }

    /// Set crossover rate
    pub fn set_crossover_rate(&mut self, rate: f32) {
        self.crossover_rate = rate;
    }
}
