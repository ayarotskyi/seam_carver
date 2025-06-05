use mpsc::Sender;
use sorted_vec::SortedSet;

use crate::*;
use std::thread;

pub fn spawn_seam_carver(
    image_matrix: &Matrix<Color>,
    window_size: &Arc<RwLock<WindowSize>>,
    image_sender: Sender<Box<Image>>,
) {
    let (vertical_seam_sender, vertical_seam_receiver) = mpsc::channel::<Box<Seam>>();
    let (horizontal_seam_sender, horizontal_seam_receiver) = mpsc::channel::<Box<Seam>>();
    spawn_seam_extractors(&image_matrix, vertical_seam_sender, horizontal_seam_sender);

    let image_matrix = Box::new(image_matrix.clone());
    let window_size_clone = Arc::clone(&window_size);
    thread::Builder::new()
        .name("seam_carver".to_string())
        .spawn(move || {
            let mut seam_holder = Box::new(SeamHolder {
                horizontal_seams: Vec::with_capacity(image_matrix.height()),
                vertical_seams: Vec::with_capacity(image_matrix.width),
            });

            loop {
                let window_size = window_size_clone.read().unwrap().clone();
                seam_holder
                    .vertical_seams
                    .extend(vertical_seam_receiver.try_iter().map(|seam| *seam));
                seam_holder
                    .horizontal_seams
                    .extend(horizontal_seam_receiver.try_iter().map(|seam| *seam));

                image_sender
                    .send(Box::new(matrix_to_image(&carve_seams(
                        &image_matrix,
                        &seam_holder,
                        &window_size,
                    ))))
                    .unwrap();
            }
        })
        .unwrap();
}

fn carve_seams(
    image_matrix: &Matrix<Color>,
    seam_holder: &SeamHolder,
    window_size: &WindowSize,
) -> Matrix<Color> {
    let image_matrix = image_matrix.clone();
    let image_height = image_matrix.vector.len() / image_matrix.width;
    let mut indices_to_remove = SortedSet::with_capacity(image_matrix.vector.len());

    let horizontal_seams_amount = if window_size.height < image_height {
        image_height - window_size.height
    } else {
        0
    }
    .min(seam_holder.horizontal_seams.len());
    let vertical_seams_amount = if window_size.width < image_matrix.width {
        image_matrix.width - window_size.width
    } else {
        0
    }
    .min(seam_holder.vertical_seams.len());

    seam_holder
        .horizontal_seams
        .iter()
        .take(horizontal_seams_amount)
        .for_each(|seam| {
            indices_to_remove.extend(seam.indices.iter().map(|index: &usize| *index));
        });

    seam_holder
        .vertical_seams
        .iter()
        .take(vertical_seams_amount)
        .for_each(|seam| {
            indices_to_remove.extend(seam.indices.iter().map(|index: &usize| *index));
        });

    let vector = remove_sorted_indices(&image_matrix.vector, &indices_to_remove.to_vec());

    Matrix {
        width: image_matrix.width - vertical_seams_amount,
        vector: vector,
    }
}
