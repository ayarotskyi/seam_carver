use crate::{structs::matrix::Matrix, *};

pub fn grayscale(image_matrix: &Matrix<Color>) -> Box<Matrix<f32>> {
    Box::new(Matrix::new(
        image_matrix
            .vector
            .iter()
            .map(|color| 0.299 * color.r + 0.587 * color.g + 0.114 * color.b)
            .collect(),
        image_matrix.width(),
    ))
}

pub fn gradient_magnitude(grayscale_matrix: &Matrix<f32>) -> Matrix<f32> {
    let mut result = grayscale_matrix.clone();
    let width = grayscale_matrix.width();
    let height = grayscale_matrix.height();
    result
        .vector
        .par_chunks_exact_mut(width)
        .enumerate()
        .for_each(|(i, vector)| {
            for (j, value) in vector.iter_mut().enumerate() {
                *value = (((if i > 0 {
                    grayscale_matrix.vector[(i - 1) * width + j]
                } else {
                    0.0
                }) - (if i < height - 1 {
                    grayscale_matrix.vector[(i + 1) * width + j]
                } else {
                    0.0
                }))
                .powi(2)
                    + ((if j > 0 {
                        grayscale_matrix.vector[i * width + j - 1]
                    } else {
                        0.0
                    }) - (if j < width - 1 {
                        grayscale_matrix.vector[i * width + j + 1]
                    } else {
                        0.0
                    }))
                    .powi(2))
                .sqrt();
            }
        });

    result
}

pub fn image_to_matrix(image: &Image) -> Matrix<Color> {
    Matrix::new(
        {
            let mut vector = Vec::with_capacity(image.width() * image.height());
            for y in 0..image.height() {
                for x in 0..image.width() {
                    vector.push(image.get_pixel(x as u32, y as u32));
                }
            }
            vector
        },
        image.width(),
    )
}

pub fn matrix_to_image(matrix: &Matrix<Color>) -> Image {
    let mut image = Image {
        bytes: vec![0; matrix.vector.len() * 4],
        width: matrix.width() as u16,
        height: matrix.height() as u16,
    };
    image.update(&matrix.vector);
    image
}
