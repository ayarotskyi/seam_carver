use crate::*;

pub fn grayscale(image_matrix: &Matrix<Color>) -> Box<Matrix<f32>> {
    Box::new(Matrix {
        vector: image_matrix
            .vector
            .iter()
            .map(|color| 0.299 * color.r + 0.587 * color.g + 0.114 * color.b)
            .collect(),
        width: image_matrix.width,
    })
}

pub fn gradient_magnitude(grayscale_matrix: &Matrix<f32>) -> Matrix<f32> {
    let mut result = Matrix {
        vector: vec![INFINITY; grayscale_matrix.vector.len()],
        width: grayscale_matrix.width,
    };
    let width = grayscale_matrix.width;
    let height = grayscale_matrix.vector.len() / grayscale_matrix.width;
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
    }
}

pub fn matrix_to_image(matrix: &Matrix<Color>) -> Image {
    let mut image = Image {
        bytes: vec![0; matrix.vector.len() * 4],
        width: matrix.width as u16,
        height: (matrix.vector.len() / matrix.width) as u16,
    };
    image.update(&matrix.vector);
    image
}

pub fn remove_sorted_indices<T: Copy>(vector: &Vec<T>, indices: &Vec<usize>) -> Vec<T> {
    let mut indices = indices.into_iter();
    let mut result = Vec::with_capacity(vector.len() - indices.len());

    let mut index_to_remove = match indices.next() {
        Some(index) => index,
        None => return vector.clone(),
    };

    vector.into_iter().enumerate().for_each(|(index, value)| {
        if index != *index_to_remove {
            result.push(*value);
        } else {
            index_to_remove = match indices.next() {
                Some(index) => index,
                None => return,
            }
        }
    });

    result
}
