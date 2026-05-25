//! # NEURAL NETWORK - Red Neuronal desde cero
//!
//! Implementación 100% Rust puro - sin librerías externas.
//! Incluye forward propagation, backpropagation y entrenamiento.

#![allow(dead_code)]

mod activation;
mod layer;
mod matrix;
mod neuron;
mod trainer;

pub use activation::{leaky_relu, relu, sigmoid, softmax, tanh, ActivationFunc};
pub use layer::Layer;
pub use matrix::Matrix;
pub use neuron::Neuron;
pub use trainer::Trainer;

/// Red neuronal completa
#[derive(Debug, Clone)]
pub struct NeuralNetwork {
    layers: Vec<Layer>,
    learning_rate: f32,
    momentum: f32,
    loss_history: Vec<f32>,
    epochs_trained: usize,
}

impl NeuralNetwork {
    /// Crear nueva red con arquitectura especificada
    pub fn new(architecture: &[usize], activation: ActivationFunc) -> Self {
        let mut layers = Vec::new();

        for i in 0..architecture.len() - 1 {
            layers.push(Layer::new(
                architecture[i],
                architecture[i + 1],
                activation.clone(),
            ));
        }

        // Última capa usa softmax para clasificación
        if let Some(last) = layers.last_mut() {
            last.set_activation(ActivationFunc::Softmax);
        }

        Self {
            layers,
            learning_rate: 0.01,
            momentum: 0.9,
            loss_history: Vec::new(),
            epochs_trained: 0,
        }
    }

    /// Forward propagation
    pub fn forward(&self, input: &[f32]) -> Vec<f32> {
        let mut result = input.to_vec();

        for layer in &self.layers {
            result = layer.forward(&result);
        }

        result
    }

    /// Backpropagation - calcular gradientes
    pub fn backprop(&mut self, input: &[f32], target: &[f32]) -> f32 {
        // Forward pass
        let mut outputs = vec![input.to_vec()];
        for layer in &self.layers {
            outputs.push(layer.forward(outputs.last().unwrap()));
        }

        // Calcular error en última capa (cross-entropy con softmax)
        let prediction = outputs.last().unwrap();
        let mut gradients = Vec::new();

        for i in 0..prediction.len() {
            let error = prediction[i] - target[i];
            gradients.push(error);
        }

        // Backward pass (reversed)
        for i in (0..self.layers.len()).rev() {
            let input_layer = outputs[i].clone();
            gradients = self.layers[i].backprop(
                &input_layer,
                &gradients,
                self.learning_rate,
                self.momentum,
            );
        }

        // Calcular loss
        let loss = self.cross_entropy_loss(prediction, target);
        self.loss_history.push(loss);
        loss
    }

    /// Cross-entropy loss
    fn cross_entropy_loss(&self, prediction: &[f32], target: &[f32]) -> f32 {
        let mut loss = 0.0;
        for i in 0..prediction.len() {
            if prediction[i] > 0.0 {
                loss -= target[i] * prediction[i].ln();
            }
        }
        loss
    }

    /// Entrenar con un ejemplo
    pub fn train(&mut self, input: &[f32], target: &[f32]) -> f32 {
        self.epochs_trained += 1;
        self.backprop(input, target)
    }

    /// Entrenar con batch de ejemplos
    pub fn train_batch(&mut self, batch: &[(Vec<f32>, Vec<f32>)]) -> f32 {
        let mut total_loss = 0.0;
        for (input, target) in batch {
            total_loss += self.train(input, target);
        }
        total_loss / batch.len() as f32
    }

    /// Predicción
    pub fn predict(&self, input: &[f32]) -> Vec<f32> {
        self.forward(input)
    }

    /// Get accuracy
    pub fn accuracy(&self, test_data: &[(Vec<f32>, Vec<f32>)]) -> f32 {
        let mut correct = 0;
        for (input, target) in test_data {
            let output = self.predict(input);
            let predicted = Self::argmax(&output);
            let expected = Self::argmax(target);
            if predicted == expected {
                correct += 1;
            }
        }
        correct as f32 / test_data.len() as f32
    }

    fn argmax(v: &[f32]) -> usize {
        v.iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .map(|(i, _)| i)
            .unwrap_or(0)
    }

    /// Guardar pesos a archivo (simplificado)
    pub fn save(&self, path: &str) -> std::io::Result<()> {
        use std::io::Write;
        let mut file = std::fs::File::create(path)?;
        writeln!(file, "NeuralNetwork layers={}", self.layers.len())?;
        writeln!(file, "learning_rate={}", self.learning_rate)?;
        writeln!(file, "epochs_trained={}", self.epochs_trained)?;
        for (i, layer) in self.layers.iter().enumerate() {
            writeln!(file, "Layer {}: {} neurons", i, layer.len())?;
        }
        Ok(())
    }

    /// Cargar pesos desde archivo (simplificado)
    pub fn load(&mut self, path: &str) -> std::io::Result<()> {
        let _contents = std::fs::read_to_string(path)?;
        // Por ahora solo verifica que existe
        Ok(())
    }

    /// Set learning rate
    pub fn set_learning_rate(&mut self, lr: f32) {
        self.learning_rate = lr;
    }

    /// Get loss history
    pub fn get_loss_history(&self) -> &[f32] {
        &self.loss_history
    }

    /// Info de la red
    pub fn info(&self) -> String {
        format!(
            "NeuralNetwork: {} layers, {} epochs trained, loss={:.4}",
            self.layers.len(),
            self.epochs_trained,
            self.loss_history.last().unwrap_or(&0.0)
        )
    }

    /// Get input size (first layer neurons)
    pub fn input_size(&self) -> usize {
        self.layers.first().map(|l| l.neuron_count()).unwrap_or(0)
    }

    /// Get output size (last layer neurons)
    pub fn output_size(&self) -> usize {
        self.layers.last().map(|l| l.neuron_count()).unwrap_or(0)
    }

    /// Get hidden layer size (middle layer neurons)
    pub fn hidden_size(&self) -> usize {
        if self.layers.len() >= 2 {
            self.layers[self.layers.len() / 2].neuron_count()
        } else {
            0
        }
    }
}

/// Red neuronal con memoria persistente
#[derive(Debug, Clone)]
pub struct PersistentNeuralNetwork {
    network: NeuralNetwork,
    memory_path: String,
}

impl PersistentNeuralNetwork {
    pub fn new(architecture: &[usize], memory_path: &str) -> Self {
        let network = NeuralNetwork::new(architecture, ActivationFunc::ReLU);
        Self {
            network,
            memory_path: memory_path.to_string(),
        }
    }

    /// Guardar red después de entrenamiento
    pub fn save(&self) -> std::io::Result<()> {
        self.network.save(&self.memory_path)
    }

    /// Cargar red al iniciar
    pub fn load(&mut self) -> bool {
        self.network.load(&self.memory_path).is_ok()
    }

    /// Entrenar y guardar automáticamente
    pub fn train_and_save(&mut self, input: &[f32], target: &[f32]) -> f32 {
        let loss = self.network.train(input, target);
        let _ = self.save();
        loss
    }
}
