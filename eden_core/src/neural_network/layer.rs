//! # LAYER - Capa de neuronas desde cero
#![allow(unused_imports)]
#![allow(dead_code)]
use std::sync::Mutex;

use super::activation::ActivationFunc;
use super::neuron::Neuron;

#[derive(Debug, Clone)]
pub struct Layer {
    neurons: Vec<Neuron>,
    activation: ActivationFunc,
}

impl Layer {
    pub fn new(input_size: usize, output_size: usize, activation: ActivationFunc) -> Self {
        let neurons = (0..output_size)
            .map(|_| Neuron::new(input_size, activation.clone()))
            .collect();

        Self {
            neurons,
            activation,
        }
    }

    pub fn forward(&self, input: &[f32]) -> Vec<f32> {
        let mut output = Vec::new();
        for neuron in &self.neurons {
            let mut n = neuron.clone();
            output.push(n.forward(input));
        }
        output
    }

    pub fn forward_mut(&mut self, input: &[f32]) -> Vec<f32> {
        let mut output = Vec::new();
        for neuron in &mut self.neurons {
            output.push(neuron.forward(input));
        }
        output
    }

    pub fn backprop(
        &mut self,
        input: &[f32],
        output_gradients: &[f32],
        learning_rate: f32,
        momentum: f32,
    ) -> Vec<f32> {
        let mut input_gradients = vec![0.0; input.len()];

        for (i, neuron) in self.neurons.iter_mut().enumerate() {
            let gradient = output_gradients[i];
            let neuron_grads = neuron.backward(gradient, learning_rate, momentum);

            for (j, grad) in neuron_grads.iter().enumerate() {
                if j < input_gradients.len() {
                    input_gradients[j] += grad;
                }
            }
        }

        input_gradients
    }

    pub fn set_activation(&mut self, activation: ActivationFunc) {
        self.activation = activation.clone();
        for n in &mut self.neurons {
            n.activation = activation.clone();
        }
    }

    pub fn get_neurons_mut(&mut self) -> &mut Vec<Neuron> {
        &mut self.neurons
    }

    pub fn len(&self) -> usize {
        self.neurons.len()
    }

    pub fn neuron_count(&self) -> usize {
        self.neurons.len()
    }
}
