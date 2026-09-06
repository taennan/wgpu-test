use crate::{app::RootState, systems::AppSystem, utils::paths};
use winit::keyboard::KeyCode;

#[derive(Debug)]
pub struct AtlasDumper;

impl AppSystem for AtlasDumper {
    fn run(&self, state: &mut RootState) {
        if !state.app.keys_pressed.contains(&KeyCode::Backslash) {
            return;
        }

        let dump_path = paths::texture("atlas-dump.png");
        log::info!("Dumping texture atlas to {:?}", dump_path);

        state.graphics.atlas_dumper.dump_to_png(
            &dump_path,
            state.graphics.texture_atlas.texture(),
            &state.graphics.device,
            &state.graphics.queue,
        );

        log::info!("Dumped texture atlas to {:?}", dump_path);
    }
}
