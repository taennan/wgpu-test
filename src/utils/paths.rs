use std::path::PathBuf;

pub fn root() -> PathBuf {
    PathBuf::from(&std::env::current_dir().expect("Couldn't read cwd for some reason"))
        .join("src")
        .to_path_buf()
}

pub fn assets() -> PathBuf {
    root().join("assets")
}

pub fn scenes() -> PathBuf {
    root().join("assets/scenes")
}

pub fn scene(name: &str) -> PathBuf {
    scenes().join(format!("{name}.toml"))
}

pub fn shader(entity_name: &str) -> PathBuf {
    root().join("scene").join(entity_name).join("shader.wgsl")
}
