pub enum AlbatrossTexture {
    Light,
    Dark,
}

impl AlbatrossTexture {
    pub fn to_str(&self) -> &str {
        match self {
            AlbatrossTexture::Light => "albatross-light.jpg",
            AlbatrossTexture::Dark => "albatross-dark.jpg",
        }
    }

    pub fn other(&self) -> Self {
        match self {
            AlbatrossTexture::Light => AlbatrossTexture::Dark,
            AlbatrossTexture::Dark => AlbatrossTexture::Light,
        }
    }
}
