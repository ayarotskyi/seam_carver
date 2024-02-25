use std::{
    sync::{mpsc, Arc, RwLock},
    thread,
};

use macroquad::prelude::*;
#[derive(Clone)]
struct WindowSize {
    height: usize,
    width: usize,
}

#[macroquad::main("Texture")]
async fn main() {
    let window_size = Arc::new(RwLock::new(WindowSize {
        height: screen_height() as usize,
        width: screen_width() as usize,
    }));

    let image = Arc::new(RwLock::new(load_image("ferris.png").await.unwrap()));
    let edited_image = image.clone();

    let (tx, rx) = mpsc::channel::<Box<Image>>();

    let image_clone = Arc::clone(&image);
    let edited_image_clone = Arc::clone(&edited_image);
    let window_size_clone = Arc::clone(&window_size);
    thread::spawn(move || loop {
        let read_guard = edited_image_clone.read().unwrap();
        let window_size_read_guard = window_size_clone.read().unwrap();
        let window_size_clone = window_size_read_guard.clone();
        drop(window_size_read_guard);
        let image = if read_guard.height() < window_size_clone.height
            || read_guard.width() < window_size_clone.width
        {
            &image_clone
        } else {
            &edited_image_clone
        };
        drop(read_guard);

        let image_read_guard = image.read().unwrap();
        let mut image_clone = image_read_guard.clone();
        drop(image_read_guard);

        seam_carving(&mut image_clone, &window_size_clone);
        let _ = tx.send(Box::new(image_clone));
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

        match rx.try_recv() {
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
            format!("{}", get_fps()).as_str(),
            0.0,
            32.0,
            32.0,
            Color::new(255.0, 255.0, 0.0, 100.0),
        );
        next_frame().await
    }
}

fn seam_carving(image: &mut Image, window_size: &WindowSize) {
    for i in 0..image.width() {
        for j in 0..image.height() {
            image.set_pixel(
                i.try_into().unwrap(),
                j.try_into().unwrap(),
                Color::new(
                    rand::rand() as f32 / u32::MAX as f32,
                    window_size.height as f32 / window_size.width as f32,
                    0.0,
                    100.0,
                ),
            )
        }
    }
}
