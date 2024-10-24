use mpsc::{Receiver, Sender};

use crate::*;
use std::thread;

pub fn start_seam_carver_thread(
    image_matrix: &Matrix<Color>,
    window_size: &Arc<RwLock<WindowSize>>,
    vertical_seam_receiver: Receiver<Box<Vec<usize>>>,
    image_sender: Sender<Box<Image>>,
) {
    let original_image_matrix = Box::new(image_matrix.clone());
    let window_size_clone = Arc::clone(&window_size);
    thread::Builder::new()
        .name("seam_carver".to_string())
        .spawn(move || {
            let mut seam_holder = Box::new(SeamHolder {
                horizontal_seams: Vec::with_capacity(
                    original_image_matrix.vector.len() / original_image_matrix.width,
                ),
                vertical_seams: Vec::with_capacity(original_image_matrix.width),
            });
            let mut image_matrix_cache = original_image_matrix.clone();

            loop {
                let window_size = window_size_clone.read().unwrap().clone();
                seam_holder
                    .vertical_seams
                    .extend(vertical_seam_receiver.try_iter().map(|seam| *seam));

                if window_size.height > image_matrix_cache.vector.len() / image_matrix_cache.width
                    || window_size.width > image_matrix_cache.width
                {
                    image_matrix_cache = original_image_matrix.clone();
                }

                let start_from_vertical = original_image_matrix.width - image_matrix_cache.width;

                seam_carving(
                    &mut image_matrix_cache,
                    &seam_holder,
                    &window_size,
                    start_from_vertical,
                );

                image_sender
                    .send(Box::new(matrix_to_image(&image_matrix_cache)))
                    .unwrap();
            }
        })
        .unwrap();
}

fn seam_carving(
    image_matrix: &mut Matrix<Color>,
    seam_holder: &SeamHolder,
    window_size: &WindowSize,
    start_from_vertical: usize,
) {
    seam_holder
        .vertical_seams
        .iter()
        .skip(start_from_vertical)
        .take(if window_size.width > image_matrix.width {
            0
        } else {
            image_matrix.width - window_size.width
        })
        .for_each(|seam| {
            delete_vertical_seam(image_matrix, seam);
        })
}
