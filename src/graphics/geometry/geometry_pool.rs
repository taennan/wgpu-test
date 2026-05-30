use super::{Geometry, Vertex};
use crate::utils;
use glam::{Vec2, Vec3, Vec3Swizzles};
use gltf;
use image::EncodableLayout;
use std::{
    collections::HashMap,
    ffi::OsStr,
    fmt::Debug,
    path::{Path, PathBuf},
};
use tobj;

#[derive(Debug, Default)]
pub struct GeometryPool {
    geometry: HashMap<PathBuf, Geometry>,
}

impl GeometryPool {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, path: &PathBuf) -> Option<&Geometry> {
        self.geometry.get(path)
    }

    pub fn has<P>(&self, path: &P) -> bool
    where
        P: AsRef<Path>,
    {
        self.geometry.contains_key(path.as_ref())
    }

    pub fn remove(&mut self, path: &PathBuf) {
        self.geometry.remove(path);
    }

    pub fn load(&mut self, path: &PathBuf) -> &Geometry {
        if !self.has(path) {
            let extension = path.extension();
            let gltf_ext = OsStr::new("gltf");
            let obj_ext = OsStr::new("obj");

            let geometry = if extension == Some(gltf_ext) {
                self.load_gltf(path)
            } else if extension == Some(obj_ext) {
                self.load_obj(path)
            } else {
                panic!("Unsupported file type");
            };

            self.geometry.insert(path.clone(), geometry);
        }

        self.get(path).expect("Did not load Geometry")
    }

    fn load_gltf<P>(&mut self, path: &P) -> Geometry
    where
        P: AsRef<Path> + Into<PathBuf> + Clone,
    {
        let path = utils::paths::geometry(path);
        let (gltf_data, buffers, _) = gltf::import(path.clone())
            .map_err(|error| {
                log::error!("Error loading gltf file\n{}\n{}", &path.display(), error);
                error
            })
            .expect("Failed to read .gltf file");

        let mut geometry = Geometry::default();
        for mesh in gltf_data.meshes() {
            for primitive in mesh.primitives() {
                let reader =
                    primitive.reader(|buffer| buffers.get(buffer.index()).map(|b| b.as_bytes()));

                let indices: Vec<_> = reader
                    .read_indices()
                    .map(|read_indices| read_indices.into_u32().collect())
                    .unwrap_or_default();
                let mut indices: Vec<_> = indices
                    .into_iter()
                    .map(|index| index + geometry.indices.len() as u32)
                    .collect();

                let positions: Vec<[f32; 3]> = reader
                    .read_positions()
                    .map(|iter| iter.collect())
                    .unwrap_or_default();
                let uvs: Vec<[f32; 2]> = reader
                    .read_tex_coords(0)
                    .map(|read_tex_coords| read_tex_coords.into_f32().collect())
                    .unwrap_or_default();

                let mut vertices: Vec<_> = positions
                    .into_iter()
                    .zip(uvs)
                    .map(|(position, uv)| Vertex {
                        position: position.into(),
                        uv: uv.into(),
                    })
                    .collect();

                geometry.vertices.append(&mut vertices);
                geometry.indices.append(&mut indices);
            }
        }

        geometry
    }

    fn load_obj<P>(&mut self, path: &P) -> Geometry
    where
        P: AsRef<Path> + Into<PathBuf> + Debug + Clone,
    {
        let path = utils::paths::geometry(path);
        let (models, _) = tobj::load_obj(
            path,
            &tobj::LoadOptions {
                single_index: false,
                triangulate: true,
                ignore_lines: true,
                ignore_points: true,
            },
        )
        .expect("Failed to load .obj file");

        let mut geometry = Geometry::default();

        for model in models {
            let mesh = model.mesh;
            let index_offset = geometry.vertices.len() as u32;

            log::debug!("Mesh data");
            println!("positions.len() = {}", mesh.positions.len());
            println!("texcoords.len() = {}", mesh.texcoords.len());
            println!("indices.len() = {}", mesh.indices.len());

            for i in 0..mesh.indices.len() {
                let pos_idx = (mesh.indices[i] * 3) as usize;
                let uv_idx = (mesh.texcoord_indices[i] * 2) as usize;

                let vertex = Vertex {
                    position: Vec3::new(
                        mesh.positions[pos_idx],
                        mesh.positions[pos_idx + 1],
                        mesh.positions[pos_idx + 2],
                    ),
                    uv: Vec2::new(
                        mesh.texcoords.get(uv_idx).copied().unwrap_or(0.0),
                        mesh.texcoords.get(uv_idx + 1).copied().unwrap_or(0.0),
                    ),
                };

                geometry.vertices.push(vertex);
            }

            let num_new_vertices = mesh.indices.len();
            geometry
                .indices
                .extend((0..num_new_vertices as u32).map(|i| i + index_offset));

            geometry
                .indices
                .extend(mesh.indices.iter().map(|i| i + index_offset));
        }

        log::debug!("LOADING GEOMETRY");
        for index in geometry.indices.iter() {
            let i = *index as usize;
            println!("{} {}", index, geometry.vertices[i].position.xyz());
        }

        geometry
    }
}
