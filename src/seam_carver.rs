use crate::*;
use ::rand::{rngs::ThreadRng, thread_rng, Rng};
use std::thread;

pub fn spawn_seam_carver(
    displayed_image: &Arc<RwLock<Image>>,
    window_size: &Arc<RwLock<WindowSize>>,
) {
    let image_matrix = image_to_matrix(&displayed_image.read().unwrap().clone());
    let window_size_clone = Arc::clone(&window_size);
    let displayed_image_clone = Arc::clone(&displayed_image);
    thread::Builder::new()
        .name("seam_carver".to_string())
        .spawn(move || {
            let mut rng = thread_rng();
            loop {
                let window_size = window_size_clone.read().unwrap().clone();
                let mut energy_matrix = gradient_magnitude(&grayscale(&image_matrix));
                let mut carved_image_matrix = image_matrix.clone();
                while match window_size_clone.try_read() {
                    Ok(next_window_size) => window_size == *next_window_size,
                    Err(_) => true,
                } || window_size.width < energy_matrix.width
                    || window_size.height < energy_matrix.height()
                {
                    if window_size.width >= energy_matrix.width
                        && window_size.height >= energy_matrix.height()
                    {
                        continue;
                    }
                    let lesser_energy_seam: Seam = if window_size.width >= energy_matrix.width {
                        extract_horizontal_seam(&energy_matrix, &mut rng).0
                    } else if window_size.height >= energy_matrix.height() {
                        extract_vertical_seam(&energy_matrix, &mut rng).0
                    } else {
                        let (vertical_seam, vertical_seam_energy) =
                            extract_vertical_seam(&energy_matrix, &mut rng);
                        let (horizontal_seam, horizontal_seam_energy) =
                            extract_horizontal_seam(&energy_matrix, &mut rng);

                        if vertical_seam_energy < horizontal_seam_energy {
                            vertical_seam
                        } else {
                            horizontal_seam
                        }
                    };

                    carved_image_matrix.carve_seam(&lesser_energy_seam);
                    energy_matrix = gradient_magnitude(&grayscale(&carved_image_matrix));

                    match displayed_image_clone.try_write() {
                        Ok(mut display_image_write_lock) => {
                            *display_image_write_lock = matrix_to_image(&carved_image_matrix);
                        }
                        Err(_) => {}
                    }
                }
            }
        })
        .unwrap();
}

fn extract_vertical_seam(energy_matrix: &Matrix<f32>, rng: &mut ThreadRng) -> (Seam, f32) {
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

    (
        Seam {
            indices: indices
                .iter()
                .map(|index| energy_matrix.original_indices[*index])
                .collect(),
            is_vertical: true,
        },
        indices
            .iter()
            .fold(0.0, |acc, index| acc + energy_matrix.vector[*index]),
    )
}

fn extract_horizontal_seam(energy_matrix: &Matrix<f32>, rng: &mut ThreadRng) -> (Seam, f32) {
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

    (
        Seam {
            indices: indices
                .iter()
                .map(|index| energy_matrix.original_indices[*index])
                .collect(),
            is_vertical: false,
        },
        indices
            .iter()
            .fold(0.0, |acc, index| acc + energy_matrix.vector[*index]),
    )
}
