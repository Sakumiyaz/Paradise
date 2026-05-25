//! # TRAINER - Sistema de entrenamiento desde cero

#![allow(dead_code)]

use super::NeuralNetwork;

#[derive(Debug, Clone)]
pub struct TrainingConfig {
    pub epochs: usize,
    pub batch_size: usize,
    pub learning_rate: f32,
    pub momentum: f32,
    pub validation_split: f32,
    pub early_stopping_patience: usize,
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            epochs: 100,
            batch_size: 32,
            learning_rate: 0.01,
            momentum: 0.9,
            validation_split: 0.2,
            early_stopping_patience: 10,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TrainingResult {
    pub final_loss: f32,
    pub final_accuracy: f32,
    pub epochs_run: usize,
    pub loss_history: Vec<f32>,
    pub accuracy_history: Vec<f32>,
}

pub struct Trainer {
    config: TrainingConfig,
}

impl Trainer {
    pub fn new(config: TrainingConfig) -> Self {
        Self { config }
    }

    pub fn train(
        &self,
        network: &mut NeuralNetwork,
        data: &[(Vec<f32>, Vec<f32>)],
    ) -> TrainingResult {
        let mut loss_history = Vec::new();
        let mut accuracy_history = Vec::new();
        let mut best_loss = f32::INFINITY;
        let mut patience_counter = 0;

        for epoch in 0..self.config.epochs {
            let mut epoch_loss = 0.0;

            // Shuffle data
            let mut shuffled = data.to_vec();
            let mut rng = rand::thread_rng();
            for i in 0..shuffled.len() {
                let j = rand::Rng::gen_range(&mut rng, 0..i + 1);
                shuffled.swap(i, j);
            }

            // Train in batches
            for chunk in shuffled.chunks(self.config.batch_size) {
                for (input, target) in chunk {
                    let loss = network.train(input, target);
                    epoch_loss += loss;
                }
            }

            let avg_loss = epoch_loss / data.len() as f32;
            let accuracy = network.accuracy(data);

            loss_history.push(avg_loss);
            accuracy_history.push(accuracy);

            // Early stopping
            if avg_loss < best_loss {
                best_loss = avg_loss;
                patience_counter = 0;
            } else {
                patience_counter += 1;
                if patience_counter >= self.config.early_stopping_patience {
                    return TrainingResult {
                        final_loss: avg_loss,
                        final_accuracy: accuracy,
                        epochs_run: epoch + 1,
                        loss_history,
                        accuracy_history,
                    };
                }
            }

            // Log progress
            if epoch % 10 == 0 {
                println!(
                    "Epoch {}: loss={:.4}, accuracy={:.4}",
                    epoch, avg_loss, accuracy
                );
            }
        }

        let final_accuracy = network.accuracy(data);
        TrainingResult {
            final_loss: loss_history.last().unwrap_or(&0.0).clone(),
            final_accuracy,
            epochs_run: self.config.epochs,
            loss_history,
            accuracy_history,
        }
    }

    pub fn train_with_validation(
        &self,
        network: &mut NeuralNetwork,
        train_data: &[(Vec<f32>, Vec<f32>)],
        val_data: &[(Vec<f32>, Vec<f32>)],
    ) -> TrainingResult {
        let mut loss_history = Vec::new();
        let mut val_loss_history = Vec::new();
        let mut best_val_loss = f32::INFINITY;
        let mut patience_counter = 0;

        for epoch in 0..self.config.epochs {
            // Training
            let mut train_loss = 0.0;
            let mut shuffled = train_data.to_vec();
            let mut rng = rand::thread_rng();
            for i in 0..shuffled.len() {
                let j = rand::Rng::gen_range(&mut rng, 0..i + 1);
                shuffled.swap(i, j);
            }

            for chunk in shuffled.chunks(self.config.batch_size) {
                for (input, target) in chunk {
                    train_loss += network.train(input, target);
                }
            }

            let avg_train_loss = train_loss / train_data.len() as f32;
            let train_acc = network.accuracy(train_data);
            loss_history.push(avg_train_loss);

            // Validation
            let val_loss = {
                let mut total = 0.0;
                for (input, target) in val_data {
                    let output = network.predict(input);
                    total += cross_entropy(&output, target);
                }
                total / val_data.len() as f32
            };
            val_loss_history.push(val_loss);

            // Early stopping
            if val_loss < best_val_loss {
                best_val_loss = val_loss;
                patience_counter = 0;
            } else {
                patience_counter += 1;
                if patience_counter >= self.config.early_stopping_patience {
                    return TrainingResult {
                        final_loss: val_loss,
                        final_accuracy: network.accuracy(val_data),
                        epochs_run: epoch + 1,
                        loss_history,
                        accuracy_history: val_loss_history,
                    };
                }
            }

            if epoch % 10 == 0 {
                println!(
                    "Epoch {}: train_loss={:.4}, val_loss={:.4}, train_acc={:.4}",
                    epoch, avg_train_loss, val_loss, train_acc
                );
            }
        }

        TrainingResult {
            final_loss: val_loss_history.last().unwrap_or(&0.0).clone(),
            final_accuracy: network.accuracy(val_data),
            epochs_run: self.config.epochs,
            loss_history,
            accuracy_history: val_loss_history,
        }
    }
}

fn cross_entropy(prediction: &[f32], target: &[f32]) -> f32 {
    let mut loss = 0.0;
    for i in 0..prediction.len() {
        if prediction[i] > 0.0 {
            loss -= target[i] * prediction[i].ln();
        }
    }
    loss
}

// Gradiente descendiente con momentum
pub struct SGDOptimizer {
    learning_rate: f32,
    momentum: f32,
    velocity: Vec<f32>,
}

impl SGDOptimizer {
    pub fn new(learning_rate: f32, momentum: f32) -> Self {
        Self {
            learning_rate,
            momentum,
            velocity: Vec::new(),
        }
    }

    pub fn step(&mut self, weights: &mut [f32], gradient: &[f32]) {
        if self.velocity.len() != weights.len() {
            self.velocity = vec![0.0; weights.len()];
        }

        for i in 0..weights.len() {
            self.velocity[i] = self.momentum * self.velocity[i] - self.learning_rate * gradient[i];
            weights[i] += self.velocity[i];
        }
    }
}
