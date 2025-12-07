mod app;
mod error;
mod graphics;
mod scene;
mod systems;
mod tests;
mod utils;

use app::App;
use dotenv::dotenv;
use env_logger;
use winit::event_loop::{ControlFlow, EventLoop};

pub fn main() {
    dotenv().ok();
    env_logger::init();

    log::info!("Will start wgpu-test");

    let mut app = App::new();

    let event_loop = EventLoop::new().expect("Event loop creation failed");
    event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.run_app(&mut app).expect("Event loop run failed");

    log::info!("Finished running wgpu-test");
}
