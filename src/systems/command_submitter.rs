use crate::{app::RootState, systems::AppSystem};
use std::mem;

pub struct CommandSubmitter;

impl AppSystem for CommandSubmitter {
    fn run(&self, state: &mut RootState) {
        let command_buffers = mem::take(&mut state.graphics.command_buffers);
        state.graphics.queue.submit(command_buffers);
    }
}
