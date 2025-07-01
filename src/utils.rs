use crate::{
    structs::{color::CustomColor, matrix::Matrix},
    *,
};

fn grayscale(color: CustomColor) -> f32 {
    0.299 * color.r + 0.587 * color.g + 0.114 * color.b
}

#[derive(Clone, Copy)]
pub struct GradientMagnitudePoint {
    pub value: f32,
    pub is_inserted: bool,
}

pub fn gradient_magnitude(matrix: &Matrix<CustomColor>) -> Matrix<GradientMagnitudePoint> {
    let mut result = Matrix::new(
        vec![
            GradientMagnitudePoint {
                value: 0.0,
                is_inserted: false,
            };
            matrix.vector.len()
        ],
        matrix.width(),
    );
    let width = matrix.width();
    let height = matrix.height();
    result
        .vector
        .chunks_exact_mut(width)
        .enumerate()
        .for_each(|(i, vector)| {
            for (j, point) in vector.iter_mut().enumerate() {
                point.value = (((if i > 0 {
                    grayscale(matrix.vector[(i - 1) * width + j])
                } else {
                    0.0
                }) - (if i < height - 1 {
                    grayscale(matrix.vector[(i + 1) * width + j])
                } else {
                    0.0
                }))
                .powi(2)
                    + ((if j > 0 {
                        grayscale(matrix.vector[i * width + j - 1])
                    } else {
                        0.0
                    }) - (if j < width - 1 {
                        grayscale(matrix.vector[i * width + j + 1])
                    } else {
                        0.0
                    }))
                    .powi(2))
                .sqrt();
                point.is_inserted = matrix.vector[i * width + j].is_inserted;
            }
        });

    result
}

pub fn image_to_matrix(image: &Image) -> Matrix<CustomColor> {
    Matrix::new(
        {
            let mut vector = Vec::with_capacity(image.width() * image.height());
            for y in 0..image.height() {
                for x in 0..image.width() {
                    let color = image.get_pixel(x as u32, y as u32);
                    vector.push(CustomColor {
                        r: color.r,
                        g: color.g,
                        b: color.b,
                        is_inserted: false,
                    });
                }
            }
            vector
        },
        image.width(),
    )
}

pub fn matrix_to_image(matrix: &Matrix<CustomColor>) -> Image {
    let mut image = Image {
        bytes: vec![0; matrix.vector.len() * 4],
        width: matrix.width() as u16,
        height: matrix.height() as u16,
    };
    image.update(
        &matrix
            .vector
            .iter()
            .map(|color| Color {
                r: color.r,
                g: color.g,
                b: color.b,
                a: 100.0,
            })
            .collect::<Vec<Color>>(),
    );
    image
}
