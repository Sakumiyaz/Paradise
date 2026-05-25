//! # ACTIVATION - Funciones de activación desde cero

#![allow(dead_code)]

#[derive(Debug, Clone, PartialEq)]
pub enum ActivationFunc {
    Sigmoid,
    Tanh,
    ReLU,
    LeakyReLU,
    Softmax,
}

impl ActivationFunc {
    pub fn apply(&self, x: f32) -> f32 {
        match self {
            ActivationFunc::Sigmoid => sigmoid(x),
            ActivationFunc::Tanh => tanh(x),
            ActivationFunc::ReLU => relu(x),
            ActivationFunc::LeakyReLU => leaky_relu(x),
            ActivationFunc::Softmax => x, // Softmax se aplica a todo el vector
        }
    }

    pub fn derivative(&self, x: f32) -> f32 {
        match self {
            ActivationFunc::Sigmoid => sigmoid(x) * (1.0 - sigmoid(x)),
            ActivationFunc::Tanh => 1.0 - tanh(x).powi(2),
            ActivationFunc::ReLU => {
                if x > 0.0 {
                    1.0
                } else {
                    0.0
                }
            }
            ActivationFunc::LeakyReLU => {
                if x > 0.0 {
                    1.0
                } else {
                    0.01
                }
            }
            ActivationFunc::Softmax => 1.0, // No se usa directamente
        }
    }
}

pub fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

pub fn tanh(x: f32) -> f32 {
    x.tanh()
}

pub fn relu(x: f32) -> f32 {
    if x > 0.0 {
        x
    } else {
        0.0
    }
}

pub fn leaky_relu(x: f32) -> f32 {
    if x > 0.0 {
        x
    } else {
        0.01 * x
    }
}

pub fn softmax(vec: &[f32]) -> Vec<f32> {
    let max = vec.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let exps: Vec<f32> = vec.iter().map(|x| (x - max).exp()).collect();
    let sum: f32 = exps.iter().sum();
    exps.iter().map(|x| x / sum).collect()
}

pub fn softmax_derivative(output: &[f32]) -> Vec<Vec<f32>> {
    let n = output.len();
    let mut jacobian = vec![vec![0.0; n]; n];

    for i in 0..n {
        for j in 0..n {
            let delta = if i == j { 1.0 } else { 0.0 };
            jacobian[i][j] = output[i] * (delta - output[j]);
        }
    }

    jacobian
}
