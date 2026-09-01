use crate::{app::RootState, systems::AppSystem, utils::paths};
use winit::keyboard::KeyCode;

#[derive(Debug)]
pub struct AtlasDumper;

impl AppSystem for AtlasDumper {
    fn run(&self, state: &mut RootState) {
        if !state.app.keys_pressed.contains(&KeyCode::Backslash) {
            return;
        }

        log::debug!("Created encoder for atlas dump");
        let mut encoder =
            state
                .graphics
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });

        state
            .graphics
            .atlas_dumper
            .dump_to_png(
                paths::texture("atlas.png"),
                state.graphics.texture_atlas.texture(),
                &state.graphics.device,
                &mut encoder,
            )
            .expect("Failed to dump atlas texture");
    }
}
