use rayon::prelude::*;
use std::{
    env,
    f32::INFINITY,
    sync::{mpsc, Arc, RwLock},
};
mod structs;
use structs::*;
mod seam_carver;
use macroquad::prelude::*;
mod utils;
use seam_carver::*;
use utils::*;
mod seam_extractor;
use seam_extractor::*;

#[macroquad::main("Texture")]
async fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    let window_size = Arc::new(RwLock::new(WindowSize {
        height: screen_height() as usize,
        width: screen_width() as usize,
    }));

    let mut image = load_image("image.png").await.unwrap();
    let image_matrix = Box::new(image_to_matrix(&image));

    let (vertical_seam_sender, vertical_seam_receiver) = mpsc::channel::<Box<Vec<usize>>>();
    let (image_sender, image_receiver) = mpsc::channel::<Box<Image>>();

    start_seam_extractor_thread(&image_matrix, vertical_seam_sender);
    start_seam_carver_thread(
        &image_matrix,
        &window_size,
        vertical_seam_receiver,
        image_sender,
    );

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
            Ok(received_image) => {
                image = *received_image;
            }
            Err(_) => {}
        }
        draw_texture(&Texture2D::from_image(&image), 0., 0., WHITE);

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
