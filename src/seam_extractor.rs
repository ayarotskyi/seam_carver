use std::thread;

use mpsc::Sender;

use crate::*;

pub fn start_seam_extractor_thread(
    image_matrix: &Matrix<Color>,
    vertical_seam_sender: Sender<Box<Vec<usize>>>,
) {
    let mut grayscale_matrix = grayscale(&image_matrix);
    thread::Builder::new()
        .name("vertical_seam_extractor".to_string())
        .spawn(move || {
            for _ in 0..grayscale_matrix.width {
                let energy_matrix = gradient_magnitude(&grayscale_matrix);
                let seam = vertical_seam(&energy_matrix);
                delete_vertical_seam(&mut grayscale_matrix, &seam);
                vertical_seam_sender.send(Box::new(seam.clone())).unwrap();
            }
        })
        .unwrap();
}

fn vertical_seam(energy_matrix: &Matrix<f32>) -> Vec<usize> {
    let mut dp_result = energy_matrix.vector.clone();
    let width = energy_matrix.width;
    let height = energy_matrix.vector.len() / width;

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

    let skip = (height - 1) * width;
    seam[height - 1] = dp_result
        .iter()
        .skip(skip)
        .take(width)
        .enumerate()
        .fold((0, &INFINITY), |acc, value| {
            if *acc.1 > *value.1 {
                return value;
            }
            acc
        })
        .0
        + skip;

    for i in (0..height - 1).rev() {
        let index = {
            let prev_index = seam[i + 1] - width;
            let min = if prev_index % width > 0
                && energy_matrix.vector[prev_index - 1] < energy_matrix.vector[prev_index]
            {
                prev_index - 1
            } else {
                prev_index
            };
            if prev_index % width < width - 1
                && energy_matrix.vector[min] > energy_matrix.vector[prev_index + 1]
            {
                prev_index + 1
            } else {
                min
            }
        };

        seam[i] = index;
    }

    seam.iter()
        .map(|index| *index % energy_matrix.width)
        .collect()
}
