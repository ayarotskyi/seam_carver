use crate::*;

pub fn grayscale(image_matrix: &Matrix<Color>) -> Box<Matrix<f32>> {
    Box::new(Matrix {
        vector: image_matrix
            .vector
            .iter()
            .map(|color| 0.299 * color.r + 0.587 * color.g + 0.114 * color.b)
            .collect(),
        width: image_matrix.width,
        original_indices: image_matrix.original_indices.clone(),
    })
}

pub fn gradient_magnitude(grayscale_matrix: &Matrix<f32>) -> Matrix<f32> {
    let mut result = Matrix {
        vector: vec![INFINITY; grayscale_matrix.vector.len()],
        width: grayscale_matrix.width,
        original_indices: grayscale_matrix.original_indices.clone(),
    };
    let width = grayscale_matrix.width;
    let height = grayscale_matrix.height();
    result
        .vector
        .par_chunks_exact_mut(width)
        .enumerate()
        .for_each(|(i, vector)| {
            for (j, value) in vector.iter_mut().enumerate() {
                if grayscale_matrix.vector[i * width + j] == INFINITY {
                    continue;
                }
                *value = (((match (0..i)
                    .rev()
                    .find(|k| grayscale_matrix.vector[k * width + j] != INFINITY)
                {
                    Some(index) => grayscale_matrix.vector[index * width + j],
                    None => 0.0,
                }) - (match (i..height)
                    .find(|k| grayscale_matrix.vector[k * width + j] != INFINITY)
                {
                    Some(index) => grayscale_matrix.vector[index * width + j],
                    None => 0.0,
                }))
                .powi(2)
                    + ((match (0..j)
                        .rev()
                        .find(|k| grayscale_matrix.vector[i * width + k] != INFINITY)
                    {
                        Some(index) => grayscale_matrix.vector[i * width + index],
                        None => 0.0,
                    }) - (match (j..width)
                        .find(|k| grayscale_matrix.vector[i * width + k] != INFINITY)
                    {
                        Some(index) => grayscale_matrix.vector[i * width + index],
                        None => 0.0,
                    }))
                    .powi(2))
                .sqrt();
            }
        });

    result
}

pub fn image_to_matrix(image: &Image) -> Matrix<Color> {
    Matrix {
        width: image.width(),
        vector: {
            let mut vector = Vec::with_capacity(image.width() * image.height());
            for y in 0..image.height() {
                for x in 0..image.width() {
                    vector.push(image.get_pixel(x as u32, y as u32));
                }
            }
            vector
        },
        original_indices: (0..(image.width() * image.height())).collect(),
    }
}

pub fn matrix_to_image(matrix: &Matrix<Color>) -> Image {
    let mut image = Image {
        bytes: vec![0; matrix.vector.len() * 4],
        width: matrix.width as u16,
        height: matrix.height() as u16,
    };
    image.update(&matrix.vector);
    image
}

pub fn carve_horizontal_seam<T>(matrix: &Matrix<T>, seam: &Seam) -> Matrix<T>
where
    T: Clone + std::marker::Send + Sync + Copy,
{
    let column_vectors: Vec<(Vec<T>, Vec<usize>)> = (0..matrix.width)
        .into_par_iter()
        .map(|column| {
            let mut vector_result = Vec::with_capacity(matrix.height() - 1);
            let mut original_indices_result = Vec::with_capacity(matrix.height() - 1);
            for row in 0..matrix.height() {
                let index = matrix.original_indices[row * matrix.width + column];
                if seam.indices[column] != index {
                    vector_result.push(matrix.vector[row * matrix.width + column]);
                    original_indices_result.push(index);
                }
            }

            return (vector_result, original_indices_result);
        })
        .collect::<Vec<(Vec<T>, Vec<usize>)>>();

    let result = (0..(matrix.height() - 1))
        .into_par_iter()
        .map(|row| {
            column_vectors
                .iter()
                .map(|column_vector| (column_vector.0[row], column_vector.1[row]))
                .collect::<Vec<(T, usize)>>()
        })
        .collect::<Vec<Vec<(T, usize)>>>()
        .concat();

    Matrix {
        width: matrix.width,
        vector: result.iter().map(|item| item.0).collect(),
        original_indices: result.iter().map(|item| item.1).collect(),
    }
}
