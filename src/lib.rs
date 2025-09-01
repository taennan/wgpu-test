mod app;
mod error;
mod graphics;
mod surface;
mod systems;
mod utils;
mod vertex;

use app::App;
use dotenv::dotenv;
use env_logger;
use winit::event_loop::EventLoop;

pub fn main() {
    dotenv().ok();
    env_logger::init();

    let event_loop = EventLoop::with_user_event()
        .build()
        .expect("Event loop creation failed");
    let mut app = App::new();

    log::info!("Will start wgpu-test");

    event_loop.run_app(&mut app).expect("Event loop run failed");

    log::info!("Running wgpu-test");
}
