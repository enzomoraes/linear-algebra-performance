use rand::Rng;
mod linear_algebra_tests;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Serialize, Deserialize)]
pub struct Matrix {
    pub rows: usize,
    pub cols: usize,
    pub data: Vec<Vec<f64>>,
}

impl Matrix {
    pub fn random(rows: usize, cols: usize) -> Matrix {
        let mut buffer = vec![vec![0.0; cols]; rows];

        for i in 0..rows {
            for j in 0..cols {
                buffer[i][j] = rand::thread_rng().gen::<f64>() * 2.0 - 1.0;
            }
        }

        Matrix {
            rows,
            cols,
            data: buffer,
        }
    }

    pub fn new(rows: usize, cols: usize, data: Vec<Vec<f64>>) -> Matrix {
        assert!(data.len() - 1 != rows * cols, "Invalid Size");
        Matrix { rows, cols, data }
    }

    pub fn zeros(rows: usize, cols: usize) -> Matrix {
        Matrix {
            rows,
            cols,
            data: vec![vec![0.0; cols]; rows],
        }
    }

    pub fn add(&self, other: &Matrix) -> Matrix {
        if self.rows != other.rows || self.cols != other.cols {
            panic!(
                "Cannot add matrices. {}x{} & {}x{}",
                self.rows, self.cols, other.rows, other.cols
            )
        }

        let mut result_data = vec![vec![0.0; self.cols]; self.rows];

        for i in 0..self.rows {
            for j in 0..self.cols {
                result_data[i][j] = self.data[i][j] + other.data[i][j];
            }
        }

        Matrix {
            rows: self.rows,
            cols: self.cols,
            data: result_data,
        }
    }

    pub fn subtract(&self, other: &Matrix) -> Matrix {
        assert!(
            self.rows == other.rows && self.cols == other.cols,
            "Cannot subtract matrices with different dimensions"
        );

        let mut result_data = vec![vec![0.0; self.cols]; self.rows];

        for i in 0..self.rows {
            for j in 0..self.cols {
                result_data[i][j] = self.data[i][j] - other.data[i][j];
            }
        }

        Matrix {
            rows: self.rows,
            cols: self.cols,
            data: result_data,
        }
    }

    pub fn multiply(&self, other: &Matrix) -> Matrix {
        if self.cols != other.rows {
            panic!(
                "Cannot multiply matrices. {}x{} & {}x{}",
                self.rows, self.cols, other.rows, other.cols
            )
        }

        let mut result_data = vec![vec![0.0; other.cols]; self.rows];

        for i in 0..self.rows {
            for j in 0..other.cols {
                let mut sum = 0.0;
                for k in 0..self.cols {
                    sum += self.data[i][k] * other.data[k][j];
                }
                result_data[i][j] = sum;
            }
        }

        Matrix {
            rows: self.rows,
            cols: other.cols,
            data: result_data,
        }
    }

    pub fn hadamard_product(&self, other: &Matrix) -> Matrix {
        if self.rows != other.rows || self.cols != other.cols {
            panic!(
                "Cannot apply hadamard product to matrices. {}x{} & {}x{}",
                self.rows, self.cols, other.rows, other.cols
            )
        }

        let mut result_data = vec![vec![0.0; self.cols]; self.rows];

        for i in 0..self.rows {
            for j in 0..self.cols {
                result_data[i][j] = self.data[i][j] * other.data[i][j];
            }
        }

        Matrix {
            rows: self.rows,
            cols: self.cols,
            data: result_data,
        }
    }

    pub fn transpose(&self) -> Matrix {
        let mut buffer = vec![vec![0.0; self.rows]; self.cols];

        for i in 0..self.rows {
            for j in 0..self.cols {
                buffer[j][i] = self.data[i][j];
            }
        }
        Matrix {
            rows: self.cols,
            cols: self.rows,
            data: buffer,
        }
    }
    pub fn identity(size: usize) -> Matrix {
        let mut result_data = vec![vec![0.0; size]; size];

        for i in 0..size {
            for j in 0..size {
                if j.eq(&i) {
                    result_data[i][j] = 1.0;
                }
            }
        }

        Matrix {
            rows: size,
            cols: size,
            data: result_data,
        }
    }

    pub fn apply_function(&self, func: &dyn Fn(f64) -> f64) -> Matrix {
        let a: Vec<Vec<f64>> = self.data.iter().map(|row| row.iter().map(|&val| func(val)).collect()).collect();
        return Matrix {
            cols: self.cols,
            rows: self.rows,
            data: a,
        };
    }
}

impl fmt::Display for Matrix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let max_width = self
            .data
            .iter()
            .map(|f| f.iter().map(|&val| val.to_string().len()).max().unwrap_or(0))
            .max()
            .unwrap_or(0);

        for i in 0..self.rows {
            write!(f, "|")?;
            for j in 0..self.cols {
                // std::fmt fill/alignment
                let cell_str = format!(
                    "{:^width$}",
                    self.data[i][j],
                    width = max_width
                );
                write!(f, "{}", cell_str)?;
                if j < self.cols - 1 {
                    // Print a whitespace between values in the same row
                    write!(f, "  ")?;
                }
            }
            writeln!(f, "|")?;
        }
        Ok(())
    }
}

impl From<Vec<f64>> for Matrix {
    /// This method will always return a matrix with rows = vec.len() and cols = 1
    fn from(vec: Vec<f64>) -> Self {
        let rows = vec.len();
        let cols = 1;
        let mut data = vec![];
        for i in 0..rows {
            data.push(vec![vec[i]; cols]);
        }
        Matrix {
            rows,
            cols,
            data,
        }
    }
}
