//! # NEURON - Neurona individual desde cero

#![allow(dead_code)]

use super::activation::ActivationFunc;
use rand::Rng;

#[derive(Debug, Clone)]
pub struct Neuron {
    pub weights: Vec<f32>,
    pub bias: f32,
    pub activation: ActivationFunc,
    pub last_output: Option<f32>,
    pub last_input: Option<Vec<f32>>,
}

impl Neuron {
    pub fn new(input_size: usize, activation: ActivationFunc) -> Self {
        let mut rng = rand::thread_rng();
        let weights: Vec<f32> = (0..input_size)
            .map(|_| rng.gen::<f32>() * 2.0 - 1.0)
            .collect();

        Self {
            weights,
            bias: rng.gen::<f32>() * 0.1,
            activation,
            last_output: None,
            last_input: None,
        }
    }

    pub fn forward(&mut self, input: &[f32]) -> f32 {
        self.last_input = Some(input.to_vec());

        let mut sum = self.bias;
        for (i, w) in self.weights.iter().enumerate() {
            if i < input.len() {
                sum += input[i] * w;
            }
        }

        let output = self.activation.apply(sum);
        self.last_output = Some(output);
        output
    }

    pub fn backward(&mut self, gradient: f32, learning_rate: f32, momentum: f32) -> Vec<f32> {
        let output = self.last_output.unwrap_or(0.0);
        let input = self.last_input.clone().unwrap_or_default();

        let delta = gradient * self.activation.derivative(output);

        let mut input_gradients = Vec::new();
        for (i, w) in self.weights.iter_mut().enumerate() {
            let input_grad = delta * (*w);
            input_gradients.push(input_grad);

            // Actualizar pesos con momentum
            let input_val = if i < input.len() { input[i] } else { 0.0 };
            let weight_change = learning_rate * delta * input_val;
            *w += weight_change + momentum * weight_change;
        }

        self.bias += learning_rate * delta;

        input_gradients
    }

    pub fn get_weights(&self) -> &[f32] {
        &self.weights
    }

    pub fn set_weights(&mut self, weights: Vec<f32>) {
        self.weights = weights;
    }
}
