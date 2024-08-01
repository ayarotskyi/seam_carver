use mpsc::{Receiver, Sender};

use crate::*;
use std::thread;

pub fn start_seam_carver_thread(
    image: &Matrix<Color>,
    window_size: &Arc<RwLock<WindowSize>>,
    vertical_seam_receiver: Receiver<Box<Vec<usize>>>,
    image_sender: Sender<Box<Image>>,
) {
    let mut image = image.clone();
    let window_size_clone = Arc::clone(&window_size);
    thread::Builder::new()
        .name("seam_carver".to_string())
        .spawn(move || {
            let mut seam_holder = Box::new(SeamHolder {
                horizontal_seams: Vec::with_capacity(image.vector.len() / image.width),
                vertical_seams: Vec::with_capacity(image.width),
            });
            loop {
                let window_size_clone = window_size_clone.read().unwrap().clone();

                seam_holder
                    .vertical_seams
                    .extend(vertical_seam_receiver.try_iter().map(|seam| *seam));

                seam_carving(&mut image, &seam_holder, &window_size_clone);
                image_sender
                    .send(Box::new(matrix_to_image(&image)))
                    .unwrap();
            }
        })
        .unwrap();
}

fn seam_carving(image: &mut Matrix<Color>, seam_holder: &SeamHolder, window_size: &WindowSize) {
    seam_holder.vertical_seams.iter().for_each(|seam| {
        let chunks: Vec<Vec<Color>> = image
            .vector
            .par_chunks(image.width)
            .zip(seam)
            .map(|(chunk, index_to_remove)| {
                let mut chunk = chunk.to_vec();
                chunk.remove(*index_to_remove);
                chunk
            })
            .collect();
        image.vector = chunks.concat();
        image.width = image.width - 1;
    })
}
