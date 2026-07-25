mod app;
mod error;
mod game;
mod graphics;
mod systems;
mod utils;

use app::App;
use dotenv::dotenv;
use env_logger;
use utils::fontmap;
use winit::event_loop::{ControlFlow, EventLoop};

pub fn main() {
    dotenv().ok();
    env_logger::init();

    log::info!("Will start wgpu-test");

    fontmap::save_fontmap("JetBrainsMono-Variable.ttf", 12);

    return;

    let mut app = App::new();

    let event_loop = EventLoop::new().expect("Event loop creation failed");
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut app).expect("Event loop run failed");

    log::info!("Fin!");
}
