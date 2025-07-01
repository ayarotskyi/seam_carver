use macroquad::prelude::*;
use std::{
    env,
    sync::{Arc, RwLock},
};

use crate::{seam_carver::spawn_seam_carver, structs::window_size::WindowSize};
mod seam_carver;
mod structs;
mod utils;

fn window_conf() -> Conf {
    Conf {
        window_title: "Seam Carving".to_owned(),
        window_width: 500,
        window_height: 500,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    env::set_var("RUST_BACKTRACE", "1");
    let window_size = Arc::new(RwLock::new(WindowSize {
        height: screen_height() as usize,
        width: screen_width() as usize,
    }));

    let displayed_image = Arc::new(RwLock::new(load_image("image.png").await.unwrap()));

    spawn_seam_carver(&displayed_image, &window_size);

    let mut displayed_image_clone = displayed_image.read().unwrap().clone();
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

        match displayed_image.try_read() {
            Ok(displayed_image_read_lock) => {
                displayed_image_clone = displayed_image_read_lock.clone();
            }
            Err(_) => {}
        }

        draw_texture(
            &Texture2D::from_image(&displayed_image_clone),
            0.,
            0.,
            WHITE,
        );

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
