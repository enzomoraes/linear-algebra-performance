use contiguous_strided::Matrix;

fn main() {
    println!("CONTIGUOUS STRIDED");

    let matrix_a = Matrix::random(1000, 1000);
    let matrix_b = Matrix::random(1000, 1000);
    matrix_a.multiply(&matrix_b);
}
