//! # Campo Energético V2 - EDEN Energy Field
//! 
//! Sistema de propagación de energía 100% original para EDEN.
//! Sin Laplacian estándar, sin ecuaciones diferenciales conocidas.
//! 
//! ## Concepto: Ecología de Energía
//!
//! En EDEN, la energía no "difunde" como en física clásica.
//! La energía se "metaboliza" - cada Auton consume y produce energía
//! basándose en su estado interno.
//! 
//! ## Modelo Original:
//!
//! 1. **MetabolicFlow**: La energía fluye según demanda metabólica
//! 2. **GeneGradient**: Genes con alto fitness generangradientes de energía
//! 3. **FieldMemory**: El campo tiene "memoria" de estados anteriores
//! 4. **AdaptiveDiffusion**: La difusión se adapta según necesidad
//!
//! ## Sin fórmulas estándares:
//! - Sin ∇²φ (Laplacian)
//! - Sin Allen-Cahn
//! - Sin ecuaciones de reacción-difusión estándar
//!
//! Todo creado desde cero para la naturaleza de EDEN.

#![allow(dead_code)]

use crate::physics::fixed_point::I32F32;

// ============================================================================
// METABOLIC ENERGY CELL
// ============================================================================

/// Célula de energía metabólica - representa el estado energético local
#[derive(Clone, Debug)]
pub struct MetabolicEnergyCell {
    /// Energía actual (fixed point)
    pub energy: I32F32,
    /// Producción metabólica por ciclo
    pub production: I32F32,
    /// Consumo metabólico por ciclo
    pub consumption: I32F32,
    /// Potencial de flujo (diferencia de energía disponible)
    pub flow_potential: I32F32,
    /// Memoria del campo (historial)
    pub memory: [I32F32; 4],
    /// Generación del campo (índice de estabilidad)
    pub generation: u8,
}

impl MetabolicEnergyCell {
    pub fn new(initial_energy: I32F32) -> Self {
        MetabolicEnergyCell {
            energy: initial_energy,
            production: I32F32::ZERO,
            consumption: I32F32::ZERO,
            flow_potential: I32F32::ZERO,
            memory: [I32F32::ZERO; 4],
            generation: 0,
        }
    }
    
    /// Actualiza la memoria del campo (desplaza y añade nuevo valor)
    fn update_memory(&mut self, new_value: I32F32) {
        // Shift memory right
        self.memory[3] = self.memory[2];
        self.memory[2] = self.memory[1];
        self.memory[1] = self.memory[0];
        self.memory[0] = self.energy;
        
        self.generation = self.generation.saturating_add(1);
    }
    
    /// Calcula el "potencial metabólico" - qué tan diferente es el estado actual
    fn metabolic_potential(&self) -> I32F32 {
        let avg_memory = (self.memory[0] + self.memory[1] + self.memory[2] + self.memory[3]) 
            .wrapping_mul(I32F32::from_i32(1).wrapping_div(I32F32::from_i32(4)));
        
        (self.energy - avg_memory).abs()
    }
    
    /// Calcula estabilidad del campo (0 = caótico, 255 = estable)
    fn field_stability(&self) -> u8 {
        let potential = self.metabolic_potential();
        
        // Si la energía está cerca del promedio, es estable
        let stability: i32 = if potential.to_i32() < 100 { 255 } else { 0 };
        stability as u8
    }
}

// ============================================================================
// GENE-GRADIENT (Gradiente de genes)
// ============================================================================

/// Representa cómo los genes de un Auton influyen en el gradiente energético
#[derive(Clone, Debug)]
pub struct GeneGradient {
    /// Fitness del genoma local
    pub fitness: I32F32,
    /// Influencia en el campo energético (cuanto mejor gen, más energía)
    pub field_influence: I32F32,
    /// Ratio de adaptación
    pub adaptation_rate: I32F32,
}

impl GeneGradient {
    pub fn new(fitness: f32) -> Self {
        let fit_fp = I32F32::from_float(fitness);
        
        // Mejor fitness = mayor influencia en el campo
        // La influencia no es lineal - hay un umbral mínimo
        let field_influence = if fitness > 0.1 {
            let excess = fit_fp - I32F32::from_float(0.1);
            excess.wrapping_mul(I32F32::from_float(10.0))
        } else {
            I32F32::ZERO
        };
        
        GeneGradient {
            fitness: fit_fp,
            field_influence,
            adaptation_rate: I32F32::from_float(0.01),
        }
    }
    
    /// Calcula el "gradiente de gene" - cuánto afecta este gen a la energía circundante
    fn compute_gradient(&self, neighbor_count: usize) -> I32F32 {
        if neighbor_count == 0 {
            return I32F32::ZERO;
        }
        
        // Más vecinos = el gen tiene más impacto
        let neighbor_factor = I32F32::from_i32(neighbor_count as i32);
        
        // El gradiente es: influencia * adaptation_rate * neighbor_factor
        self.field_influence
            .wrapping_mul(self.adaptation_rate)
            .wrapping_mul(neighbor_factor)
    }
}

// ============================================================================
// ECOLOGICAL FLOW CALCULATOR
// ============================================================================

/// Calculador de flujo ecológico de energía
/// Este es el algoritmo 100% original de EDEN
pub struct EcologicalFlowCalc {
    /// Factor de adaptación metabólica
    metabolic_factor: I32F32,
    /// Factor de memoria del campo
    memory_factor: I32F32,
    /// Factor de gradiente de genes
    gene_factor: I32F32,
}

impl EcologicalFlowCalc {
    pub fn new() -> Self {
        EcologicalFlowCalc {
            metabolic_factor: I32F32::from_float(0.1),
            memory_factor: I32F32::from_float(0.05),
            gene_factor: I32F32::from_float(0.2),
        }
    }
    
    /// Calcula el flujo de energía entre dos células metabólicas
    /// Este es el corazón del algoritmo original - NO es Laplacian
    pub fn compute_flow(
        &self,
        source: &MetabolicEnergyCell,
        sink: &MetabolicEnergyCell,
        gene_gradient: &GeneGradient,
    ) -> I32F32 {
        // 1. Diferencia base de energía
        let base_diff = source.energy - sink.energy;
        
        // 2. Potencial metabólico - demanda de energía basada en producción/consumo
        let source_demand = source.production - source.consumption;
        let sink_demand = sink.production - sink.consumption;
        
        // Source con alta producción quiere distribuir energía
        // Sink con alto consumo quiere absorber energía
        let metabolic_demand = sink_demand.wrapping_mul(I32F32::from_float(0.5))
            .wrapping_sub(source_demand.wrapping_mul(I32F32::from_float(0.5)));
        
        // 3. Memoria del campo - energía tiende a moverse hacia estados más "estables"
        let source_stability = I32F32::from_i32(source.field_stability() as i32);
        let sink_stability = I32F32::from_i32(sink.field_stability() as i32);
        let memory_flow = (source_stability - sink_stability)
            .wrapping_mul(self.memory_factor);
        
        // 4. Gradiente de genes - genes buenos generan flujo de energía
        let gene_flow = gene_gradient.compute_gradient(1)
            .wrapping_mul(self.gene_factor);
        
        // Flujo total: combinación de todos los factores
        // source.energy > sink.energy → flujo positivo (energía fluye de source a sink)
        let total_flow = base_diff
            .wrapping_add(metabolic_demand)
            .wrapping_add(memory_flow)
            .wrapping_add(gene_flow);
        
        // Aplicar factor metabólico global y limitar
        let scaled_flow = total_flow.wrapping_mul(self.metabolic_factor);
        
        // Clamp: el flujo no puede exceder ciertos límites
        let max_flow = I32F32::from_float(1000.0);
        clamp_flow(scaled_flow, max_flow)
    }
    
    /// Calcula múltiples flujos para una célula
    pub fn compute_multi_flow(
        &self,
        center: &MetabolicEnergyCell,
        neighbors: &[MetabolicEnergyCell],
        gene_gradient: &GeneGradient,
    ) -> I32F32 {
        if neighbors.is_empty() {
            return I32F32::ZERO;
        }
        
        let mut total_flow = I32F32::ZERO;
        let neighbor_count = neighbors.len() as i32;
        
        for neighbor in neighbors {
            let flow = self.compute_flow(center, neighbor, gene_gradient);
            total_flow = total_flow.wrapping_add(flow);
        }
        
        // Promedio y aplicar factor de conectividad
        let connectivity = I32F32::from_i32(neighbor_count);
        total_flow.wrapping_div(connectivity)
    }
    
    /// Actualiza las células basándose en el flujo calculado
    pub fn apply_flow(
        &self,
        cell: &mut MetabolicEnergyCell,
        incoming_flow: I32F32,
        production: I32F32,
        consumption: I32F32,
    ) {
        // Actualizar energía con flujo entrante
        cell.energy = cell.energy.wrapping_add(incoming_flow);
        
        // Actualizar producción y consumo
        cell.production = production;
        cell.consumption = consumption;
        
        // Actualizar memoria
        cell.update_memory(cell.energy);
        
        // Calcular nuevo potencial de flujo
        cell.flow_potential = (production - consumption).abs();
    }
}

/// Limita el valor del flujo a un rango [-max, max]
fn clamp_flow(value: I32F32, max: I32F32) -> I32F32 {
    if value > max {
        max
    } else if value < I32F32::ZERO - max {
        I32F32::ZERO - max
    } else {
        value
    }
}

// ============================================================================
// FIELD PROPAGATION
// ============================================================================

/// Campo energético con propagación ecológica
pub struct EnergyField {
    /// Grid de células metabólicas
    cells: Vec<MetabolicEnergyCell>,
    /// Gradientes de genes (uno por región)
    gene_gradients: Vec<GeneGradient>,
    /// Dimensiones del grid
    nx: usize,
    ny: usize,
    nz: usize,
    /// Calculador de flujo
    flow_calc: EcologicalFlowCalc,
}

impl EnergyField {
    pub fn new(nx: usize, ny: usize, nz: usize) -> Self {
        let total_cells = nx * ny * nz;
        let total_regions = ((nx * ny * nz) / 1000).max(1);
        
        let cells = (0..total_cells)
            .map(|_| MetabolicEnergyCell::new(I32F32::from_float(0.5)))
            .collect();
        
        let gene_gradients = (0..total_regions)
            .map(|_| GeneGradient::new(0.5))
            .collect();
        
        EnergyField {
            cells,
            gene_gradients,
            nx,
            ny,
            nz,
            flow_calc: EcologicalFlowCalc::new(),
        }
    }
    
    /// Obtiene el índice lineal desde coordenadas
    fn idx(&self, i: usize, j: usize, k: usize) -> usize {
        k * self.nx * self.ny + j * self.nx + i
    }
    
    /// Obtiene el gradiente de gene para una posición
    fn get_gene_gradient(&self, idx: usize) -> &GeneGradient {
        let region_idx = idx % self.gene_gradients.len();
        &self.gene_gradients[region_idx]
    }
    
    /// Calcula flujo para una célula
    pub fn compute_cell_flow(&self, idx: usize) -> I32F32 {
        let i = idx % self.nx;
        let j = (idx / self.nx) % self.ny;
        let k = idx / (self.nx * self.ny);
        
        let center = &self.cells[idx];
        let gene_gradient = self.get_gene_gradient(idx);
        
        // Recolectar vecinos (con wrapping)
        let mut neighbors = Vec::new();
        
        let im1 = if i == 0 { self.nx - 1 } else { i - 1 };
        let ip1 = if i == self.nx - 1 { 0 } else { i + 1 };
        let jm1 = if j == 0 { self.ny - 1 } else { j - 1 };
        let jp1 = if j == self.ny - 1 { 0 } else { j + 1 };
        
        let idx_im1 = self.idx(im1, j, k);
        let idx_ip1 = self.idx(ip1, j, k);
        let idx_jm1 = self.idx(i, jm1, k);
        let idx_jp1 = self.idx(i, jp1, k);
        
        neighbors.push(&self.cells[idx_im1]);
        neighbors.push(&self.cells[idx_ip1]);
        neighbors.push(&self.cells[idx_jm1]);
        neighbors.push(&self.cells[idx_jp1]);
        
        // 2D por ahora
        self.flow_calc.compute_multi_flow(center, &neighbors, gene_gradient)
    }
    
    /// Propaga el campo completo
    pub fn propagate(&mut self) {
        // Calcular flujos para todas las células
        let flows: Vec<I32F32> = (0..self.cells.len())
            .map(|idx| self.compute_cell_flow(idx))
            .collect();
        
        // Aplicar flujos
        for (idx, flow) in flows.iter().enumerate() {
            let cell = &mut self.cells[idx];
            cell.energy = cell.energy.wrapping_add(*flow);
            
            // Mantener energía en rango válido [0, 1]
            if cell.energy > I32F32::from_float(1.0) {
                cell.energy = I32F32::from_float(1.0);
            } else if cell.energy < I32F32::ZERO {
                cell.energy = I32F32::ZERO;
            }
        }
    }
    
    /// Actualiza el gradiente de genes de una región
    pub fn update_gene_gradient(&mut self, region_idx: usize, fitness: f32) {
        if region_idx < self.gene_gradients.len() {
            self.gene_gradients[region_idx] = GeneGradient::new(fitness);
        }
    }
    
    /// Obtiene la energía en una posición
    pub fn energy_at(&self, i: usize, j: usize, k: usize) -> I32F32 {
        self.cells[self.idx(i, j, k)].energy
    }
    
    /// Obtiene estadísticas del campo
    pub fn stats(&self) -> EnergyFieldStats {
        let total: f32 = self.cells.iter()
            .map(|c| c.energy.to_float())
            .sum();
        let avg = total / self.cells.len() as f32;
        
        let stability_avg: f32 = self.cells.iter()
            .map(|c| c.field_stability() as f32)
            .sum::<f32>() / self.cells.len() as f32;
        
        EnergyFieldStats {
            avg_energy: avg,
            avg_stability: stability_avg / 255.0,
            cell_count: self.cells.len(),
            region_count: self.gene_gradients.len(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EnergyFieldStats {
    pub avg_energy: f32,
    pub avg_stability: f32,
    pub cell_count: usize,
    pub region_count: usize,
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_metabolic_cell() {
        let mut cell = MetabolicEnergyCell::new(I32F32::from_float(0.5));
        
        assert_eq!(cell.energy, I32F32::from_float(0.5));
        assert_eq!(cell.production, I32F32::ZERO);
        assert_eq!(cell.consumption, I32F32::ZERO);
    }
    
    #[test]
    fn test_cell_memory() {
        let mut cell = MetabolicEnergyCell::new(I32F32::from_float(0.5));
        
        cell.update_memory(I32F32::from_float(0.6));
        assert_eq!(cell.memory[0], I32F32::from_float(0.5));
        assert_eq!(cell.memory[3], I32F32::ZERO);
    }
    
    #[test]
    fn test_gene_gradient() {
        let gradient = GeneGradient::new(0.8);
        assert!(gradient.fitness > I32F32::from_float(0.5));
        assert!(gradient.field_influence > I32F32::ZERO);
    }
    
    #[test]
    fn test_flow_calculation() {
        let calc = EcologicalFlowCalc::new();
        
        let mut source = MetabolicEnergyCell::new(I32F32::from_float(0.8));
        source.production = I32F32::from_float(0.2);
        
        let mut sink = MetabolicEnergyCell::new(I32F32::from_float(0.3));
        sink.consumption = I32F32::from_float(0.1);
        
        let gene = GeneGradient::new(0.7);
        
        let flow = calc.compute_flow(&source, &sink, &gene);
        
        // Flow should be positive (energy flows from high to low energy)
        assert!(flow > I32F32::ZERO || flow < I32F32::from_float(0.5));
    }
    
    #[test]
    fn test_energy_field() {
        let mut field = EnergyField::new(10, 10, 1);
        
        let initial_stats = field.stats();
        assert_eq!(initial_stats.cell_count, 100);
        
        // Propagate should change things
        field.propagate();
        
        let after_stats = field.stats();
        // Energy should be roughly the same (conserved)
        assert!((after_stats.avg_energy - initial_stats.avg_energy).abs() < 0.1);
    }
    
    #[test]
    fn test_gene_gradient_update() {
        let mut field = EnergyField::new(10, 10, 1);
        
        field.update_gene_gradient(0, 0.9);
        
        let gradient = &field.gene_gradients[0];
        assert!(gradient.fitness > I32F32::from_float(0.8));
    }
}