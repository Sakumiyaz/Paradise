//! # MATRIX - Operaciones de matrices desde cero
//!
//! 100% Rust puro - sin BLAS ni dependencias externas.
#![allow(unused_imports)]
#![allow(dead_code)]
use std::io::Write;

use std::cmp::PartialEq;

#[derive(Debug, Clone)]
pub struct Matrix {
    pub rows: usize,
    pub cols: usize,
    pub data: Vec<f32>,
}

impl Matrix {
    pub fn new(rows: usize, cols: usize) -> Self {
        Self {
            rows,
            cols,
            data: vec![0.0; rows * cols],
        }
    }

    pub fn from_vec(data: &[f32], rows: usize, cols: usize) -> Self {
        assert_eq!(data.len(), rows * cols);
        Self {
            rows,
            cols,
            data: data.to_vec(),
        }
    }

    pub fn zeros(rows: usize, cols: usize) -> Self {
        Self::new(rows, cols)
    }

    pub fn ones(rows: usize, cols: usize) -> Self {
        let mut m = Self::new(rows, cols);
        m.data = vec![1.0; rows * cols];
        m
    }

    pub fn random(rows: usize, cols: usize) -> Self {
        let mut m = Self::new(rows, cols);
        for i in 0..m.data.len() {
            m.data[i] = (rand_u32() as f32 / u32::MAX as f32) * 2.0 - 1.0;
        }
        m
    }

    pub fn identity(n: usize) -> Self {
        let mut m = Self::new(n, n);
        for i in 0..n {
            m[(i, i)] = 1.0;
        }
        m
    }

    pub fn get(&self, row: usize, col: usize) -> f32 {
        self.data[row * self.cols + col]
    }

    pub fn set(&mut self, row: usize, col: usize, val: f32) {
        self.data[row * self.cols + col] = val;
    }

    pub fn transpose(&self) -> Self {
        let mut result = Self::new(self.cols, self.rows);
        for r in 0..self.rows {
            for c in 0..self.cols {
                result[(c, r)] = self[(r, c)];
            }
        }
        result
    }

    pub fn add(&self, other: &Matrix) -> Self {
        assert_eq!(self.rows, other.rows);
        assert_eq!(self.cols, other.cols);
        let mut result = Self::new(self.rows, self.cols);
        for i in 0..self.data.len() {
            result.data[i] = self.data[i] + other.data[i];
        }
        result
    }

    pub fn sub(&self, other: &Matrix) -> Self {
        assert_eq!(self.rows, other.rows);
        assert_eq!(self.cols, other.cols);
        let mut result = Self::new(self.rows, self.cols);
        for i in 0..self.data.len() {
            result.data[i] = self.data[i] - other.data[i];
        }
        result
    }

    pub fn mul_scalar(&self, scalar: f32) -> Self {
        let mut result = self.clone();
        for i in 0..result.data.len() {
            result.data[i] *= scalar;
        }
        result
    }

    pub fn dot(&self, other: &Matrix) -> Self {
        assert_eq!(self.cols, other.rows);
        let mut result = Self::new(self.rows, other.cols);
        for i in 0..self.rows {
            for j in 0..other.cols {
                let mut sum = 0.0;
                for k in 0..self.cols {
                    sum += self[(i, k)] * other[(k, j)];
                }
                result[(i, j)] = sum;
            }
        }
        result
    }

    pub fn hadamard(&self, other: &Matrix) -> Self {
        assert_eq!(self.rows, other.rows);
        assert_eq!(self.cols, other.cols);
        let mut result = Self::new(self.rows, self.cols);
        for i in 0..self.data.len() {
            result.data[i] = self.data[i] * other.data[i];
        }
        result
    }

    pub fn sum(&self) -> f32 {
        self.data.iter().sum()
    }

    pub fn mean(&self) -> f32 {
        self.sum() / self.data.len() as f32
    }

    pub fn apply(&self, f: fn(f32) -> f32) -> Self {
        let mut result = self.clone();
        for i in 0..result.data.len() {
            result.data[i] = f(result.data[i]);
        }
        result
    }

    pub fn shape(&self) -> (usize, usize) {
        (self.rows, self.cols)
    }
}

impl PartialEq for Matrix {
    fn eq(&self, other: &Self) -> bool {
        self.rows == other.rows
            && self.cols == other.cols
            && self
                .data
                .iter()
                .zip(&other.data)
                .all(|(a, b)| (a - b).abs() < 1e-6)
    }
}

impl std::ops::Index<(usize, usize)> for Matrix {
    type Output = f32;
    fn index(&self, (row, col): (usize, usize)) -> &f32 {
        &self.data[row * self.cols + col]
    }
}

impl std::ops::IndexMut<(usize, usize)> for Matrix {
    fn index_mut(&mut self, (row, col): (usize, usize)) -> &mut f32 {
        &mut self.data[row * self.cols + col]
    }
}

fn rand_u32() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64;
    ((nanos ^ (nanos >> 32)) & 0xFFFFFFFF) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matrix_mul() {
        let a = Matrix::from_vec(&[1.0, 2.0, 3.0, 4.0], 2, 2);
        let b = Matrix::from_vec(&[5.0, 6.0, 7.0, 8.0], 2, 2);
        let c = a.dot(&b);
        assert_eq!(c[(0, 0)], 19.0);
        assert_eq!(c[(0, 1)], 22.0);
    }
}
