use crate::{utils::*, WindowSize};
use ::rand::thread_rng;
use macroquad::texture::Image;
use std::{
    sync::{Arc, RwLock},
    thread,
};

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
            let mut energy_matrix = gradient_magnitude(&image_matrix);
            let mut carved_image_matrix = image_matrix.clone();
            let mut window_size = window_size_clone.read().unwrap().clone();
            loop {
                match window_size_clone.try_read() {
                    Ok(next_window_size) => {
                        if *next_window_size != window_size {
                            window_size = next_window_size.clone();
                        }
                    }
                    Err(_) => {}
                };

                if window_size.width == energy_matrix.width()
                    && window_size.height == energy_matrix.height()
                {
                    continue;
                }

                if window_size.height == energy_matrix.height() {
                    let carve = window_size.width < energy_matrix.width();
                    let (seam, _) = energy_matrix.extract_vertical_seam(&mut rng, !carve);
                    if carve {
                        carved_image_matrix.carve_vertical_seam(&seam);
                    } else {
                        carved_image_matrix.insert_vertical_seam(&seam);
                    }
                } else if window_size.width == energy_matrix.width() {
                    let carve = window_size.height < energy_matrix.height();
                    let (seam, _) = energy_matrix.extract_horizontal_seam(&mut rng, !carve);
                    if carve {
                        carved_image_matrix.carve_horizontal_seam(&seam);
                    } else {
                        carved_image_matrix.insert_horizontal_seam(&seam);
                    }
                } else {
                    let carve_vertical = window_size.width < energy_matrix.width();
                    let (vertical_seam, vertical_seam_energy) =
                        energy_matrix.extract_vertical_seam(&mut rng, !carve_vertical);
                    let carve_horizontal = window_size.height < energy_matrix.height();
                    let (horizontal_seam, horizontal_seam_energy) =
                        energy_matrix.extract_horizontal_seam(&mut rng, !carve_horizontal);
                    if vertical_seam_energy < horizontal_seam_energy {
                        if carve_vertical {
                            carved_image_matrix.carve_vertical_seam(&vertical_seam);
                        } else {
                            carved_image_matrix.insert_vertical_seam(&vertical_seam);
                        }
                    } else {
                        if carve_horizontal {
                            carved_image_matrix.carve_horizontal_seam(&horizontal_seam);
                        } else {
                            carved_image_matrix.insert_horizontal_seam(&horizontal_seam);
                        }
                    }
                }

                energy_matrix = gradient_magnitude(&carved_image_matrix);

                match displayed_image_clone.try_write() {
                    Ok(mut display_image_write_lock) => {
                        *display_image_write_lock = matrix_to_image(&carved_image_matrix);
                    }
                    Err(_) => {}
                }
            }
        })
        .unwrap();
}
