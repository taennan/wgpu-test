use crate::{app::RootState, systems::AppSystem};
use std::mem;

#[derive(Debug)]
pub struct CommandSubmitter;

impl AppSystem for CommandSubmitter {
    fn run(&self, state: &mut RootState) {
        let command_buffers = mem::take(&mut state.graphics.command_buffers);
        log::debug!("Will submit {:?} commands", command_buffers.len());
        let submission_index = state.graphics.queue.submit(command_buffers);
        log::debug!(
            "Did submit commands, submission index: {:?}",
            submission_index
        );
    }
}
