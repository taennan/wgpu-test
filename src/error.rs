pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug)]
pub enum Error {
    SurfaceCreationFailed,
    AdapterCreationFailed,
    WindowCreationFailed,
    Texture(TextureError),
    AssetLoadingFailed,
}

#[derive(Clone, Debug)]
pub enum TextureError {
    Lost,
    Outdated,
    Other,
}
