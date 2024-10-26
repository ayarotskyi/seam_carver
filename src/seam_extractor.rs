use std::thread;

use mpsc::Sender;

use crate::*;
use ::rand::{rngs::ThreadRng, thread_rng, Rng};

pub fn start_seam_extractor_thread(
    image_matrix: &Matrix<Color>,
    vertical_seam_sender: Sender<Box<Vec<usize>>>,
) {
    let mut grayscale_matrix = grayscale(&image_matrix);
    thread::Builder::new()
        .name("vertical_seam_extractor".to_string())
        .spawn(move || {
            let mut rng = thread_rng();
            for _ in 0..grayscale_matrix.width {
                let energy_matrix = gradient_magnitude(&grayscale_matrix);
                let seam = vertical_seam(&energy_matrix, &mut rng);
                delete_vertical_seam(&mut grayscale_matrix, &seam);
                vertical_seam_sender.send(Box::new(seam.clone())).unwrap();
            }
        })
        .unwrap();
}

fn vertical_seam(energy_matrix: &Matrix<f32>, rng: &mut ThreadRng) -> Vec<usize> {
    let mut dp_result = energy_matrix.vector.clone();
    let width = energy_matrix.width;
    let height = energy_matrix.vector.len() / width;

    // fill in the vector using dynamic programming
    for i in 1..height {
        for j in 0..width {
            dp_result[i * width + j] = dp_result[(i - 1) * width + j]
                .min({
                    if j == 0 {
                        INFINITY
                    } else {
                        dp_result[(i - 1) * width + j - 1]
                    }
                })
                .min({
                    if j == width - 1 {
                        INFINITY
                    } else {
                        dp_result[(i - 1) * width + j + 1]
                    }
                })
                + dp_result[i * width + j];
        }
    }

    let mut seam = vec![0; height];

    // calculate the last element in seam by randomly
    // selecting one of the minimum points in the last row
    let mut min_indexes = Vec::with_capacity(width);
    let mut current_min = dp_result[(height - 1) * width];
    dp_result
        .iter()
        .skip((height - 1) * width)
        .enumerate()
        .for_each(|(index, value)| {
            if *value > current_min {
                min_indexes.truncate(0);
                min_indexes.push(index);
                current_min = *value;
            }
            if *value == current_min {
                min_indexes.push(index);
            }
        });
    seam[height - 1] = min_indexes[rng.gen_range(0..min_indexes.len())];

    // calculate the rest of the indexes for the seam
    for i in (0..height - 1).rev() {
        let index = {
            let prev_index = seam[i + 1] + width * i;
            let min = if prev_index % width > 0 && dp_result[prev_index - 1] < dp_result[prev_index]
            {
                prev_index - 1
            } else {
                prev_index
            };
            if prev_index % width < width - 1 && dp_result[min] > dp_result[prev_index + 1] {
                prev_index + 1
            } else {
                min
            }
        };
        seam[i] = index % width;
    }

    seam
}
