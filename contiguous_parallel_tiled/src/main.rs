use std::env;

use contiguous_parallel_tiled::Matrix;

fn main() {
    let size = env::var("SIZE").unwrap_or_else(|_| "1000".to_string());
    let block_size = env::var("BLOCK_SIZE").unwrap_or_else(|_| "512".to_string());
    let size = size.parse::<usize>().unwrap();
    let block_size = block_size.parse::<usize>().unwrap();

    let matrix_a = Matrix::random(size, size);
    let matrix_b = Matrix::random(size, size);
    matrix_a.multiply(&matrix_b, block_size);
}
