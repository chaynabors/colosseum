// Copyright 2021 Chay Nabors.

mod camera;
mod config;
mod game_state;
mod socket;

use std::fs;
use std::io::Write;
use std::path::Path;
use std::rc::Rc;

use gear::Engine;

use self::config::Config;
use self::game_state::GameState;

#[tokio::main(flavor = "current_thread")]
async fn main() {
    env_logger::Builder::new()
        .format(|buf, record| writeln!(buf, "[{}: {}] {}", record.level(), record.metadata().target(), record.args()))
        .filter(None, log::LevelFilter::Info)
        .filter(Some("gfx_backend_dx11"), log::LevelFilter::Warn)
        .filter(Some("gfx_backend_vulkan"), log::LevelFilter::Warn)
        .filter(Some("wgpu_core"), log::LevelFilter::Warn)
        .init();

    let config = Rc::new(load_config());
    let resolution = config.resolution;

    let engine = Engine::new().await;
    engine.window.resize(resolution);

    let mut game_state = GameState::new(config, &engine);

    engine.run(move |engine, event| {
        let new_state = match &mut game_state {
            GameState::MenuState(state) => state.handle_event(&event, engine),
            GameState::CombatState(state) => state.handle_event(&event, engine),
        };

        match new_state {
            Some(state) => game_state = state,
            None => (),
        }
    });
}

fn load_config() -> Config {
    let path = Path::new("client.json");

    match path.exists() {
        true => {
            let config = fs::read(path).unwrap();
            serde_json::from_slice::<Config>(&config).unwrap()
        },
        false => {
            let config = Config::default();
            fs::write(path, serde_json::to_string_pretty(&config).unwrap()).unwrap();
            config
        },
    }
}
