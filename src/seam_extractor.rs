use std::thread;

use mpsc::Sender;

use crate::*;
use ::rand::{rngs::ThreadRng, thread_rng, Rng};

pub fn spawn_seam_extractors(
    image_matrix: &Matrix<Color>,
    vertical_seam_sender: Sender<Box<Seam>>,
    horizontal_seam_sender: Sender<Box<Seam>>,
) {
    let mut vertical_grayscale_matrix = grayscale(&image_matrix);
    let mut horizontal_grayscale_matrix = vertical_grayscale_matrix.clone();
    // thread::Builder::new()
    //     .name("vertical_seam_extractor".to_string())
    //     .spawn(move || {
    //         let mut rng = thread_rng();
    //         for _ in 0..vertical_grayscale_matrix.width {
    //             let energy_matrix = gradient_magnitude(&vertical_grayscale_matrix);
    //             let vertical_seam = extract_vertical_seam(&energy_matrix, &mut rng);

    //             vertical_grayscale_matrix.vector =
    //                 mask_sorted_indices(vertical_grayscale_matrix.vector, &vertical_seam.indices);
    //             vertical_seam_sender.send(Box::new(vertical_seam)).unwrap();
    //         }
    //     })
    //     .unwrap();

    thread::Builder::new()
        .name("horizontal_seam_extractor".to_string())
        .spawn(move || {
            let mut rng = thread_rng();
            for _ in 0..horizontal_grayscale_matrix.width {
                let energy_matrix = gradient_magnitude(&horizontal_grayscale_matrix);
                let horizontal_seam = extract_horizontal_seam(&energy_matrix, &mut rng);
                horizontal_grayscale_matrix.carve_horizontal_seams(vec![horizontal_seam.clone()]);

                horizontal_seam_sender
                    .send(Box::new(horizontal_seam))
                    .unwrap();
            }
        })
        .unwrap();
}

fn extract_vertical_seam(energy_matrix: &Matrix<f32>, rng: &mut ThreadRng) -> Seam {
    let mut dp_result = energy_matrix.vector.clone();
    let width = energy_matrix.width;
    let height = energy_matrix.height();

    // fill in the vector using dynamic programming
    for i in 1..height {
        for j in 0..width {
            dp_result[i * width + j] = dp_result[(i - 1) * width + j]
                .min(
                    match (0..j)
                        .rev()
                        .find(|k| energy_matrix.vector[(i - 1) * width + *k] != INFINITY)
                    {
                        Some(index) => dp_result[(i - 1) * width + index],
                        None => INFINITY,
                    },
                )
                .min(
                    match ((j + 1)..width)
                        .find(|k| energy_matrix.vector[(i - 1) * width + *k] != INFINITY)
                    {
                        Some(index) => dp_result[(i - 1) * width + index],
                        None => INFINITY,
                    },
                )
                + dp_result[i * width + j];
        }
    }

    let mut indices = vec![0; height];

    // calculate the last element in seam by randomly
    // selecting one of the minimum points in the last row
    let mut min_indices = Vec::with_capacity(width);
    let mut current_min = dp_result[(height - 1) * width];
    dp_result
        .iter()
        .enumerate()
        .skip((height - 1) * width)
        .for_each(|(index, value)| {
            if *value < current_min {
                min_indices.truncate(0);
                min_indices.push(index);
                current_min = *value;
            }
            if *value == current_min {
                min_indices.push(index);
            }
        });
    indices[height - 1] = min_indices[rng.gen_range(0..min_indices.len())];

    // calculate the rest of the indexes for the seam
    for i in (0..height - 1).rev() {
        let index = {
            let mid_index = indices[i + 1] - width;
            let left_index = match ((i * width)..mid_index)
                .rev()
                .find(|k| energy_matrix.vector[*k] != INFINITY)
            {
                Some(index) => index,
                None => mid_index,
            };
            let mut min_index = if dp_result[left_index] < dp_result[mid_index] {
                left_index
            } else {
                mid_index
            };

            let right_index = match ((mid_index + 1)..(width * (i + 1)))
                .find(|k| energy_matrix.vector[*k] != INFINITY)
            {
                Some(index) => index,
                None => mid_index,
            };

            min_index = if dp_result[min_index] < dp_result[right_index] {
                min_index
            } else {
                right_index
            };

            min_index
        };
        indices[i] = index;
    }

    Seam {
        indices: indices
            .iter()
            .map(|index| energy_matrix.original_indices[*index])
            .collect(),
    }
}

fn extract_horizontal_seam(energy_matrix: &Matrix<f32>, rng: &mut ThreadRng) -> Seam {
    let mut dp_result = energy_matrix.vector.clone();
    let width = energy_matrix.width;
    let height = energy_matrix.vector.len() / width;

    // fill in the vector using dynamic programming
    for i in 0..height {
        for j in 1..width {
            dp_result[i * width + j] = dp_result[i * width + j - 1]
                .min(
                    match (0..i)
                        .rev()
                        .find(|k| energy_matrix.vector[k * width + j - 1] != INFINITY)
                    {
                        Some(index) => dp_result[index * width + j - 1],
                        None => INFINITY,
                    },
                )
                .min(
                    match ((i + 1)..height)
                        .find(|k| energy_matrix.vector[k * width + j - 1] != INFINITY)
                    {
                        Some(index) => dp_result[index * width + j - 1],
                        None => INFINITY,
                    },
                )
                + dp_result[i * width + j];
        }
    }

    let mut indices = vec![0; width];

    // calculate the last element in seam by randomly
    // selecting one of the minimum points in the last column
    let mut min_indices = Vec::with_capacity(height);
    let mut current_min = dp_result[width - 1];
    dp_result
        .iter()
        .enumerate()
        .skip(width - 1)
        .step_by(width)
        .for_each(|(index, value)| {
            if *value < current_min {
                min_indices.truncate(0);
                min_indices.push(index);
                current_min = *value;
            }
            if *value == current_min {
                min_indices.push(index);
            }
        });
    indices[width - 1] = min_indices[rng.gen_range(0..min_indices.len())];

    // calculate the rest of the indexes for the seam
    for i in (0..width - 1).rev() {
        let index = {
            let mid_index = indices[i + 1] - 1;
            let top_index = match ((mid_index % width)..mid_index)
                .step_by(width)
                .rev()
                .find(|k| energy_matrix.vector[*k] != INFINITY)
            {
                Some(index) => index,
                None => mid_index,
            };

            let mut min_index = if dp_result[top_index] < dp_result[mid_index] {
                top_index
            } else {
                mid_index
            };

            let bottom_index = match ((mid_index + width)..energy_matrix.vector.len())
                .step_by(width)
                .find(|k| energy_matrix.vector[*k] != INFINITY)
            {
                Some(index) => index,
                None => mid_index,
            };

            min_index = if dp_result[min_index] < dp_result[bottom_index] {
                min_index
            } else {
                bottom_index
            };

            min_index
        };
        indices[i] = index;
    }

    Seam {
        indices: indices
            .iter()
            .map(|index| energy_matrix.original_indices[*index])
            .collect(),
    }
}
