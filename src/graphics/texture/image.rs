use crate::utils::paths;
use glam::UVec2;
use image::{DynamicImage, ImageReader};
use std::path::{Path, PathBuf};

#[derive(Debug, Default)]
pub struct AtlasImage {
    pub image: DynamicImage,
    pub path: PathBuf,
    pub size: UVec2,
}

impl AtlasImage {
    pub fn open<P: AsRef<Path>>(path: P) -> Self {
        let image_path = paths::texture(path);
        let encoded_image = match ImageReader::open(&image_path) {
            Ok(image) => image,
            Err(_) => {
                log::error!("Failed to open image at '{:?}'", image_path);
                ImageReader::open(&paths::texture("albatross-light.jpg"))
                    .expect("Failed to load backup image")
            }
        };

        let image = encoded_image
            .decode()
            .expect("Failed to decode loaded image");

        AtlasImage {
            size: UVec2::new(image.width(), image.height()),
            path: image_path.to_path_buf(),
            image,
        }
    }
}
