use std::env;

use contiguous_strided::Matrix;

fn main() {
    let size = env::var("SIZE").unwrap_or_else(|_| "1000".to_string());
    let size = size.parse::<usize>().unwrap();

    let matrix_a = Matrix::random(size, size);
    let matrix_b = Matrix::random(size, size);
    matrix_a.multiply(&matrix_b);
}
