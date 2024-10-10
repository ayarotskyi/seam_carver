use mpsc::{Receiver, Sender};

use crate::*;
use std::thread;

pub fn start_seam_carver_thread(
    image_matrix: &Matrix<Color>,
    window_size: &Arc<RwLock<WindowSize>>,
    vertical_seam_receiver: Receiver<Box<Vec<usize>>>,
    image_sender: Sender<Box<Image>>,
) {
    let image_matrix = image_matrix.clone();
    let window_size_clone = Arc::clone(&window_size);
    thread::Builder::new()
        .name("seam_carver".to_string())
        .spawn(move || {
            let mut seam_holder = Box::new(SeamHolder {
                horizontal_seams: Vec::with_capacity(
                    image_matrix.vector.len() / image_matrix.width,
                ),
                vertical_seams: Vec::with_capacity(image_matrix.width),
            });
            loop {
                let window_size_clone = window_size_clone.read().unwrap().clone();
                let mut image_matrix = image_matrix.clone();

                seam_holder
                    .vertical_seams
                    .extend(vertical_seam_receiver.try_iter().map(|seam| *seam));

                seam_carving(&mut image_matrix, &seam_holder, &window_size_clone);
                image_sender
                    .send(Box::new(matrix_to_image(&image_matrix)))
                    .unwrap();
            }
        })
        .unwrap();
}

fn seam_carving(
    image_matrix: &mut Matrix<Color>,
    seam_holder: &SeamHolder,
    window_size: &WindowSize,
) {
    seam_holder
        .vertical_seams
        .iter()
        .take(if window_size.width > image_matrix.width {
            0
        } else {
            image_matrix.width - window_size.width
        })
        .for_each(|seam| {
            delete_vertical_seam(image_matrix, seam);
        })
}
