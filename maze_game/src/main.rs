//! Entry: `mod maze` + `mod game` (Macroquad loop).

use macroquad::prelude::*;

mod game;
mod maze;

fn window_conf() -> Conf {
    Conf {
        window_title: "Maze — The Map".to_string(),
        window_width: 1280,
        window_height: 720,
        fullscreen: false,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut state = game::GameState::new().await;

    loop {
        // --- Update ---
        let dt = get_frame_time();
        state.update(dt);

        // --- Draw ---
        clear_background(Color::from_rgba(12, 12, 18, 255));
        state.draw();

        next_frame().await;
    }
}