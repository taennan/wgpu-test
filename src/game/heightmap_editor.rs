use crate::utils::map_2d::Map2D;
use glam::Vec2;

pub struct HeightmapEditor {
    pub map: Map2D<f32>,
    pub brush: Option<Map2D<f32>>,
    pub position: Vec2,
    pub zoom: f32,
}

impl HeightmapEditor {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            map: Map2D::new(width, height),
            brush: None,
            position: Vec2::ZERO,
            zoom: 1.0,
        }
    }
}
