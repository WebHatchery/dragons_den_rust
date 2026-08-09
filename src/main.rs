//! Dragon's Den — an idle hoard-building game (see gdd.md).

use macroquad::prelude::*;
use macroquad_toolkit::capture;

mod data;
mod game;
mod save;
mod simulation;
mod state;
mod ui;

use game::Game;

fn window_conf() -> Conf {
    capture::capture_window_conf(
        "DRAGONS_DEN",
        "Dragon's Den",
        ui::LOGICAL_WIDTH as i32,
        ui::LOGICAL_HEIGHT as i32,
    )
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut game = Game::new().await;

    // Screenshot harness: DRAGONS_DEN_CAPTURE_PATH renders a named scene
    // ("menu" or "hoard") deterministically, writes a PNG, and exits.
    if let Some(configs) = capture::CaptureConfig::all_from_env("DRAGONS_DEN") {
        for config in configs {
            game.begin_capture_scene(&config.scene);
            capture::run_capture_once(&config, |dt| {
                game.update(dt);
                game.draw();
            })
            .await;
        }
        return;
    }

    loop {
        let dt = get_frame_time().min(0.1);
        game.update(dt);
        game.draw();
        next_frame().await;
    }
}
