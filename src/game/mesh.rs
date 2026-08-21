use crate::graphics::{
    geometry::Geometry,
    mesh::MeshInstanceBufferData,
    renderer::{RENDERABLE_ID_COUNTER, Renderable},
};
use glam::Vec3;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::atomic::Ordering};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Mesh {
    pub mesh_key: PathBuf,
    pub texture_key: PathBuf,
    #[serde(skip)]
    pub texture_atlas_item_index: u16,
    #[serde(default = "Mesh::default_position")]
    pub position: Vec3,
    #[serde(default = "Mesh::default_scale")]
    pub scale: Vec3,
    #[serde(skip)]
    pub is_instance_data_dirty: bool,
    render_id: u64,
}

impl Mesh {
    pub fn new<P>(mesh_key: P, texture_key: P, texture_atlas_item_index: u16) -> Self
    where
        P: Into<PathBuf>,
    {
        Self {
            mesh_key: mesh_key.into(),
            texture_key: texture_key.into(),
            texture_atlas_item_index,
            position: Vec3::ZERO,
            scale: Vec3::ONE,
            render_id: RENDERABLE_ID_COUNTER.fetch_add(1, Ordering::Relaxed),
            is_instance_data_dirty: true,
        }
    }

    fn default_scale() -> Vec3 {
        Vec3::ONE
    }

    fn default_position() -> Vec3 {
        Vec3::ZERO
    }
}

impl Renderable for Mesh {
    fn render_id(&self) -> u64 {
        self.render_id
    }

    fn geometry_key(&self) -> PathBuf {
        self.mesh_key.clone()
    }

    fn vertex_bytes(&self) -> (Box<[u8]>, Box<[u8]>) {
        let geometry = Geometry::load(&self.mesh_key);
        let vertex_bytes = Box::from(geometry.vertex_bytes());
        let index_bytes = Box::from(geometry.index_bytes());

        (vertex_bytes, index_bytes)
    }

    fn instance_bytes(&self) -> Box<[u8]> {
        let instance_data = [MeshInstanceBufferData::from(self)];
        let instance_bytes = bytemuck::cast_slice(&instance_data);

        Box::from(instance_bytes)
    }

    fn dirty_instance_bytes(&mut self) -> Box<[u8]> {
        let bytes = if self.is_instance_data_dirty {
            self.instance_bytes()
        } else {
            Box::new([])
        };

        self.is_instance_data_dirty = false;
        bytes
    }
}
