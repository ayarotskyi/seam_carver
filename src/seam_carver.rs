use mpsc::Receiver;

use crate::*;
use std::thread;

pub fn start_seam_carver_thread(
    image_matrix: &Arc<RwLock<Matrix<Color>>>,
    window_size: &Arc<RwLock<WindowSize>>,
    vertical_seam_receiver: Receiver<Box<Vec<usize>>>,
) {
    let image_matrix_clone = Arc::clone(image_matrix);
    let original_image_matrix = image_matrix_clone.read().unwrap().clone();
    let window_size_clone = Arc::clone(window_size);
    thread::Builder::new()
        .name("seam_carver".to_string())
        .spawn(move || {
            let mut seam_holder = Box::new(SeamHolder {
                horizontal_seams: Vec::with_capacity(original_image_matrix.height()),
                vertical_seams: Vec::with_capacity(original_image_matrix.width),
            });
            let mut image_matrix_cache = original_image_matrix.clone();
            let mut prev_window_size = window_size_clone.read().unwrap().clone();

            loop {
                let window_size = window_size_clone.read().unwrap().clone();

                let mut received_seams_count = 0;
                seam_holder
                    .vertical_seams
                    .extend(vertical_seam_receiver.try_iter().map(|seam| {
                        received_seams_count += 1;
                        *seam
                    }));

                // drop the cache if window size has become bigger (needs improvement in future)
                if (window_size.height <= original_image_matrix.height()
                    || prev_window_size.height <= original_image_matrix.height())
                    && window_size.height > image_matrix_cache.height()
                    || (window_size.width <= original_image_matrix.width
                        || prev_window_size.width <= original_image_matrix.width)
                        && window_size.width > image_matrix_cache.width
                {
                    image_matrix_cache = original_image_matrix.clone();
                }

                // calculate the new image
                let start_from_vertical = original_image_matrix.width - image_matrix_cache.width;
                seam_carving(
                    &mut image_matrix_cache,
                    &seam_holder,
                    &window_size,
                    start_from_vertical,
                );

                // don't send the new image if the size
                if prev_window_size.height == window_size.height
                    && prev_window_size.width == window_size.width
                    && received_seams_count == 0
                {
                    continue;
                }
                prev_window_size = window_size.clone();

                // send the new image
                let mut image_matrix_write_guard = image_matrix_clone.write().unwrap();
                *image_matrix_write_guard = image_matrix_cache.clone();
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
