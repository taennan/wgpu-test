use crate::graphics::{
    geometry::Geometry,
    renderer::{RENDERABLE_ID_COUNTER, Renderable},
    sprite::SpriteInstanceBufferData,
};
use glam::{UVec2, Vec2, Vec3};
use serde::{Deserialize, Serialize};
use std::{
    path::{Path, PathBuf},
    sync::atomic::Ordering,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Sprite {
    name: Option<String>,
    mesh_key: PathBuf,
    texture_key: PathBuf,
    #[serde(default = "Sprite::default_texture_division_coords")]
    texture_division_coords: UVec2,
    #[serde(skip)]
    texture_atlas_item_index: u16,
    #[serde(default = "Sprite::default_position")]
    position: Vec3,
    #[serde(default = "Sprite::default_scale")]
    scale: Vec2,
    #[serde(skip)]
    instance_data_updated: bool,
    #[serde(skip)]
    vertex_data_updated: bool,
    #[serde(skip)]
    render_id: u64,
}

impl Sprite {
    fn default_position() -> Vec3 {
        Vec3::ZERO
    }

    fn default_scale() -> Vec2 {
        Vec2::ONE
    }

    fn default_texture_division_coords() -> UVec2 {
        UVec2::ZERO
    }

    pub fn new_square(texture_key: PathBuf, texture_atlas_item_index: u16) -> Self {
        Self::new(
            PathBuf::from("TODO: Add default square mesh with uvs"),
            texture_key,
            texture_atlas_item_index,
        )
    }

    pub fn new(mesh_key: PathBuf, texture_key: PathBuf, texture_atlas_item_index: u16) -> Self {
        Self {
            name: None,
            mesh_key,
            texture_key,
            texture_atlas_item_index,
            scale: Self::default_scale(),
            position: Self::default_position(),
            texture_division_coords: Self::default_texture_division_coords(),
            instance_data_updated: true,
            vertex_data_updated: true,
            render_id: RENDERABLE_ID_COUNTER.fetch_add(1, Ordering::Relaxed),
        }
    }

    pub fn name(&self) -> Option<&String> {
        self.name.as_ref()
    }

    pub fn set_name(&mut self, new_value: Option<String>) {
        self.name = new_value;
    }

    pub fn mesh_key(&self) -> &Path {
        &self.mesh_key
    }

    pub fn texture_key(&self) -> &Path {
        &self.texture_key
    }

    pub fn texture_division_coords(&self) -> UVec2 {
        self.texture_division_coords
    }

    pub fn set_texture_division_coords(&mut self, new_value: UVec2) {
        self.texture_division_coords = new_value;
        self.instance_data_updated = true;
    }

    pub fn texture_atlas_item_index(&self) -> u16 {
        self.texture_atlas_item_index
    }

    pub fn set_texture_atlas_item_index(&mut self, new_value: u16) {
        self.texture_atlas_item_index = new_value;
        self.instance_data_updated = true;
    }

    pub fn position(&self) -> Vec3 {
        self.position
    }

    pub fn set_position(&mut self, new_value: Vec3) {
        self.position = new_value;
        self.instance_data_updated = true;
    }

    pub fn scale(&self) -> Vec2 {
        self.scale
    }

    pub fn set_scale(&mut self, new_value: Vec2) {
        self.scale = new_value;
        self.instance_data_updated = true;
    }
}

impl Renderable for Sprite {
    fn render_id(&self) -> u64 {
        self.render_id
    }

    fn geometry_key(&self) -> PathBuf {
        self.mesh_key.clone()
    }

    fn vertex_bytes(&self) -> (Box<[u8]>, Box<[u8]>) {
        let geometry = Geometry::load(self.mesh_key());
        let vertex_bytes = Box::from(geometry.vertex_bytes());
        let index_bytes = Box::from(geometry.index_bytes());

        (vertex_bytes, index_bytes)
    }

    fn instance_bytes(&self) -> Box<[u8]> {
        let instance_data = [SpriteInstanceBufferData::from(self)];
        let instance_bytes = bytemuck::cast_slice(&instance_data);

        Box::from(instance_bytes)
    }

    fn dirty_instance_bytes(&mut self) -> Box<[u8]> {
        let bytes = if self.instance_data_updated {
            self.instance_bytes()
        } else {
            Box::new([])
        };

        self.instance_data_updated = false;
        bytes
    }
}
