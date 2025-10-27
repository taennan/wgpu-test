use crate::app::AppState;
use winit::{event::WindowEvent, event_loop::ActiveEventLoop};

pub trait AppSystem {
    #[allow(unused)]
    fn can_run(&self, state: &mut AppState) -> bool {
        true
    }

    #[allow(unused)]
    fn handle_event<'a>(
        &self,
        event: &'a WindowEvent,
        event_loop: &'a ActiveEventLoop,
        app_state: &'a mut AppState,
    ) {
    }

    fn run(&self, state: &mut AppState);
}

/*
use std::{
    sync::{Arc, Mutex},
    thread,
};

pub trait MultiThreadAppSystem: AppSystem {
    fn run_subsystems(&self, ctx: &mut AppSystemContext) {}
}

pub struct MultiThreadSystem;

impl AppSystem for MultiThreadSystem {
    fn run(&self, ctx: &mut AppSystemContext) {
        let ctx = Arc::new(Mutex::new(ctx));

        let subsystems = &[multi_thread_method_0, multi_thread_method_1];
        let mut thread_handles = Vec::with_capacity(subsystems.len());
        for subsystem in subsystems {
            let thread_safe_ctx = ctx.clone();
            thread_handles.push(thread::spawn(move || subsystem(thread_safe_ctx)));
        }

        for handle in thread_handles {
            handle.join().unwrap();
        }
    }
}

fn multi_thread_method_0(ctx: Arc<Mutex<&mut AppSystemContext>>) {}

fn multi_thread_method_1(ctx: Arc<Mutex<&mut AppSystemContext>>) {}

*/
