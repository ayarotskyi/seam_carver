use std::thread;

use mpsc::Sender;

use crate::*;

pub fn start_seam_extractor_thread(
    energy_matrix: &Matrix<f32>,
    vertical_seam_sender: Sender<Box<Vec<usize>>>,
) {
    let mut energy_matrix = energy_matrix.clone();
    thread::Builder::new()
        .name("vertical_seam_extractor".to_string())
        .spawn(move || {
            for _ in 0..energy_matrix.vector.len() / energy_matrix.width {
                let seam = vertical_seam(&energy_matrix);
                let chunks: Vec<Vec<f32>> = energy_matrix
                    .vector
                    .par_chunks(energy_matrix.width)
                    .zip(&seam)
                    .map(|(chunk, index_to_remove)| {
                        let mut chunk = chunk.to_vec();
                        chunk.remove(*index_to_remove);
                        chunk
                    })
                    .collect();
                vertical_seam_sender.send(Box::new(seam.clone())).unwrap();
                energy_matrix.vector = chunks.concat();
                energy_matrix.width = energy_matrix.width - 1;
            }
        })
        .unwrap();
}

fn vertical_seam(energy_matrix: &Matrix<f32>) -> Vec<usize> {
    let mut dp_result = energy_matrix.vector.clone();
    let mut i = 1;
    let mut j = 0;
    let width = energy_matrix.width;
    let height = energy_matrix.vector.len() / width;

    while i < height - 1 {
        if j == width {
            j = 0;
            i = i + 1;
        }

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

        j = j + 1;
    }

    let mut seam = vec![0; height];

    for i in (0..height).rev() {
        let index = if i == height - 1 {
            let skip = (height - 1) * width;
            dp_result
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
                + skip
        } else {
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
        .map(|index| index % energy_matrix.width)
        .collect()
}
