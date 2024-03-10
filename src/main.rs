use rayon::prelude::*;
use std::{
    env,
    f32::INFINITY,
    sync::{mpsc, Arc, RwLock},
    thread,
    time::Instant,
};

use macroquad::prelude::*;
#[derive(Clone)]
struct WindowSize {
    height: usize,
    width: usize,
}

#[derive(Clone)]
struct Matrix<T> {
    width: usize,
    vector: Vec<T>,
}

#[macroquad::main("Texture")]
async fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    let window_size = Arc::new(RwLock::new(WindowSize {
        height: screen_height() as usize,
        width: screen_width() as usize,
    }));

    let image = Arc::new(RwLock::new(load_image("ferris.png").await.unwrap()));
    let edited_image = Arc::new(RwLock::new(image.read().unwrap().clone()));

    let energy_matrix = Arc::new(RwLock::new(gradient_magnitude(&grayscale(&image))));

    let energy_matrix_clone = Arc::clone(&energy_matrix);
    thread::spawn(move || {
        let mut energy_matrix = energy_matrix_clone.read().unwrap().clone();

        for _ in 0..energy_matrix.vector.len() / energy_matrix.width {
            let seam = vertical_seam(&energy_matrix);
            seam.iter()
                .for_each(|index| energy_matrix.vector[*index] = INFINITY);
        }
    });

    let (image_sender, image_receiver) = mpsc::channel::<Box<Image>>();

    let image_clone = Arc::clone(&image);
    let edited_image_clone = Arc::clone(&edited_image);
    let window_size_clone = Arc::clone(&window_size);
    thread::spawn(move || loop {
        let edited_image_read_guard = edited_image_clone.read().unwrap();
        let window_size_read_guard = window_size_clone.read().unwrap();
        let window_size_clone = window_size_read_guard.clone();
        drop(window_size_read_guard);
        let image = if edited_image_read_guard.height() < window_size_clone.height
            || edited_image_read_guard.width() < window_size_clone.width
        {
            &image_clone
        } else {
            &edited_image_clone
        };
        drop(edited_image_read_guard);

        let image_read_guard = image.read().unwrap();
        let mut image_clone = image_read_guard.clone();
        drop(image_read_guard);

        seam_carving(&mut image_clone, &energy_matrix.read().unwrap());
        let _ = image_sender.send(Box::new(image_clone));
    });

    loop {
        match window_size.try_read() {
            Ok(window_size_read_guard) => {
                let next_screen_height = screen_height() as usize;
                let next_screen_width = screen_width() as usize;
                if window_size_read_guard.height != next_screen_height
                    || window_size_read_guard.width != next_screen_width
                {
                    drop(window_size_read_guard);
                    match window_size.try_write() {
                        Ok(mut window_size_write_guard) => {
                            window_size_write_guard.height = next_screen_height;
                            window_size_write_guard.width = next_screen_width;
                        }
                        Err(_) => {}
                    }
                }
            }
            Err(_) => {}
        }

        match image_receiver.try_recv() {
            Ok(image) => {
                let mut edited_image_write_guard = edited_image.write().unwrap();
                *edited_image_write_guard = *image;
                drop(edited_image_write_guard);
            }
            Err(_) => {}
        }

        match edited_image.try_read() {
            Ok(edited_image_read_guard) => {
                draw_texture(
                    &Texture2D::from_image(&edited_image_read_guard),
                    0.,
                    0.,
                    WHITE,
                );
            }
            Err(_) => {}
        }

        draw_text(
            &get_fps().to_string(),
            0.0,
            32.0,
            32.0,
            Color::new(255.0, 255.0, 0.0, 100.0),
        );
        next_frame().await
    }
}

fn seam_carving(image: &mut Image, energy_matrix: &Matrix<f32>) {
    energy_matrix
        .vector
        .chunks(energy_matrix.width)
        .enumerate()
        .for_each(|(j, vector)| {
            vector.iter().enumerate().for_each(|(i, value)| {
                image.set_pixel(i as u32, j as u32, Color::new(0.0, *value, 0.0, 100.0));
            });
        });
}

fn grayscale(image: &RwLock<Image>) -> Box<Matrix<f32>> {
    let image_read_guard = image.read().unwrap();

    let mut result = Box::new(Matrix {
        vector: vec![0.0; image_read_guard.width() * image_read_guard.height()],
        width: image_read_guard.width(),
    });

    result
        .vector
        .par_chunks_mut(image_read_guard.width())
        .enumerate()
        .for_each(|(row, vector)| {
            vector.iter_mut().enumerate().for_each(|(column, value)| {
                let color = image_read_guard.get_pixel(column as u32, row as u32);
                *value = 0.299 * color.r + 0.587 * color.g + 0.114 * color.b;
            })
        });

    result
}

fn gradient_magnitude(grayscale_matrix: &Matrix<f32>) -> Matrix<f32> {
    let mut result = Matrix {
        vector: vec![0.0; grayscale_matrix.vector.len()],
        width: grayscale_matrix.width,
    };
    let width = grayscale_matrix.width;
    let height = grayscale_matrix.vector.len() / grayscale_matrix.width;
    result
        .vector
        .par_chunks_exact_mut(width)
        .enumerate()
        .for_each(|(i, vector)| {
            for (j, value) in vector.iter_mut().enumerate() {
                *value = ((if i == 0 {
                    0.0
                } else {
                    grayscale_matrix.vector[(i - 1) * width + j]
                } - if i == height - 1 {
                    0.0
                } else {
                    grayscale_matrix.vector[(i + 1) * width + j]
                })
                .powi(2)
                    + (if j == 0 {
                        0.0
                    } else {
                        grayscale_matrix.vector[i * width + j - 1]
                    } - if j == width - 1 {
                        0.0
                    } else {
                        grayscale_matrix.vector[i * width + j + 1]
                    })
                    .powi(2))
                .sqrt();
            }
        });

    result
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
                let mut local_iterator = 1;
                loop {
                    if j < local_iterator {
                        break INFINITY;
                    }

                    let val = dp_result[(i - 1) * width + j - local_iterator];
                    if val != INFINITY {
                        break val;
                    }
                    local_iterator = local_iterator + 1;
                }
            })
            .min({
                let mut local_iterator = 1;
                loop {
                    if j + local_iterator >= width {
                        break INFINITY;
                    }

                    let val = dp_result[(i - 1) * width + j + local_iterator];
                    if val != INFINITY {
                        break val;
                    }
                    local_iterator = local_iterator + 1;
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

            let mut local_index = 1;
            let mut current_index = loop {
                if prev_index < local_index {
                    break prev_index;
                }
                let next_index = prev_index - local_index;
                if energy_matrix.vector[next_index] != INFINITY {
                    break next_index;
                }

                local_index = local_index + 1;
            };

            let mut non_inf_counter = 0;
            let mut min_index = prev_index;
            loop {
                if non_inf_counter == 3 {
                    break min_index;
                }
                let current_value = energy_matrix.vector[current_index];
                if current_value != INFINITY {
                    if current_value < energy_matrix.vector[min_index] {
                        min_index = current_index;
                    }
                    non_inf_counter = non_inf_counter + 1;
                }

                current_index = current_index + 1;
            }
        };

        seam[i] = index;
    }

    seam
}
