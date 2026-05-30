use std::path::{Path, PathBuf};

pub fn scene(name: &str) -> PathBuf {
    scenes().join(format!("{name}.toml"))
}

pub fn scenes() -> PathBuf {
    root().join("assets/scenes")
}

pub fn geometry<P>(name: P) -> PathBuf
where
    P: AsRef<Path>,
{
    assets().join("geometry").join(name)
}

pub fn texture<P>(name: P) -> PathBuf
where
    P: AsRef<Path>,
{
    assets().join("textures").join(name)
}

pub fn shader(entity_name: &str) -> PathBuf {
    root()
        .join("graphics")
        .join(entity_name)
        .join("shader.wgsl")
}

fn assets() -> PathBuf {
    root().join("assets")
}

pub fn root() -> PathBuf {
    PathBuf::from(&std::env::current_dir().expect("Couldn't read cwd for some reason"))
        .join("src")
        .to_path_buf()
}
