use crate::{graphics::geometry::Vertex, utils};
use glam::{Vec2, Vec3};
use gltf;
use image::EncodableLayout;
use std::{
    ffi::OsStr,
    fmt::{self, Debug},
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
};
use tobj;

pub struct Geometry<D, V> {
    key: PathBuf,
    vertices: Vec<D>,
    out_vert_generator: OutputVertexGenerator<D, V>,
    indices: Vec<u32>,
}

pub type MeshGeometry = Geometry<Vertex, Vertex>;
//pub type TilemapGeometry = Geometry<>;
pub type InputVertexGenerator<D> = Box<dyn Fn(RawVertex) -> D>;
pub type OutputVertexGenerator<D, V> = Box<dyn Fn(&D) -> Vec<V>>;

#[derive(Debug)]
pub struct RawVertex {
    xyz: Vec3,
    uv: Vec2,
}

impl<D, V> Debug for Geometry<D, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Geometry {{ key: {:?} }}", self.key)
    }
}

impl<D, V> Hash for Geometry<D, V> {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.key.hash(state);
    }
}

impl<D, V> Eq for Geometry<D, V> {}

impl<D, V> PartialEq for Geometry<D, V> {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
    }
}

impl<D, V> Geometry<D, V> {
    fn new(
        key: PathBuf,
        vertices: Vec<D>,
        out_vert_generator: OutputVertexGenerator<D, V>,
        indices: Vec<u32>,
    ) -> Self {
        Self {
            key,
            vertices,
            out_vert_generator,
            indices,
        }
    }

    pub fn vertices(&self) -> impl Iterator<Item = V> {
        self.vertices
            .iter()
            .flat_map(|v| (self.out_vert_generator)(v))
    }

    pub fn indices(&self) -> &[u32] {
        &self.indices
    }
}

impl MeshGeometry {
    pub fn load(path: &PathBuf) -> Self {
        let extension = path.extension();
        let gltf_ext = OsStr::new("gltf");
        let obj_ext = OsStr::new("obj");

        let in_vert_generator = Box::new(|raw: RawVertex| Vertex {
            position: raw.xyz,
            uv: raw.uv,
        });
        let out_vert_generator = Box::new(|v: &Vertex| vec![v.clone()]);

        let geometry = if extension == Some(gltf_ext) {
            Self::load_gltf(path, in_vert_generator, out_vert_generator)
        } else if extension == Some(obj_ext) {
            Self::load_obj(path, in_vert_generator, out_vert_generator)
        } else {
            panic!("Unsupported file type");
        };

        geometry
    }

    fn load_gltf<P>(
        path: &P,
        in_vert_generator: InputVertexGenerator<Vertex>,
        out_vert_generator: OutputVertexGenerator<Vertex, Vertex>,
    ) -> Self
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

        let mut vertices = vec![];
        let mut indices = vec![];

        for mesh in gltf_data.meshes() {
            for primitive in mesh.primitives() {
                let reader =
                    primitive.reader(|buffer| buffers.get(buffer.index()).map(|b| b.as_bytes()));

                let primitive_indices: Vec<_> = reader
                    .read_indices()
                    .map(|read_indices| read_indices.into_u32().collect())
                    .unwrap_or_default();
                let mut primitive_indices: Vec<_> = primitive_indices
                    .into_iter()
                    .map(|index| index + indices.len() as u32)
                    .collect();

                let positions: Vec<[f32; 3]> = reader
                    .read_positions()
                    .map(|iter| iter.collect())
                    .unwrap_or_default();
                let uvs: Vec<[f32; 2]> = reader
                    .read_tex_coords(0)
                    .map(|read_tex_coords| read_tex_coords.into_f32().collect())
                    .unwrap_or_default();

                let mut primitive_vertices: Vec<_> = positions
                    .into_iter()
                    .zip(uvs)
                    .map(|(position, uv)| RawVertex {
                        xyz: Vec3::from(position),
                        uv: Vec2::from(uv),
                    })
                    .map(|v| (in_vert_generator)(v))
                    .collect();

                vertices.append(&mut primitive_vertices);
                indices.append(&mut primitive_indices);
            }
        }

        Self::new(path, vertices, out_vert_generator, indices)
    }

    fn load_obj<P>(
        path: &P,
        in_vert_generator: InputVertexGenerator<Vertex>,
        out_vert_generator: OutputVertexGenerator<Vertex, Vertex>,
    ) -> Self
    where
        P: AsRef<Path> + Into<PathBuf> + Debug + Clone,
    {
        todo!();

        let path = utils::paths::geometry(path);
        let (models, _) = tobj::load_obj(
            path.clone(),
            &tobj::LoadOptions {
                single_index: false,
                triangulate: true,
                ignore_lines: true,
                ignore_points: true,
            },
        )
        .expect("Failed to load .obj file");

        let mut vertices = vec![];
        let mut indices = vec![];

        for model in models {
            let mesh = model.mesh;
            let index_offset = vertices.len() as u32;

            for i in 0..mesh.indices.len() {
                let pos_idx = (mesh.indices[i] * 3) as usize;
                let uv_idx = (mesh.texcoord_indices[i] * 2) as usize;

                let vertex = (in_vert_generator)(RawVertex {
                    xyz: Vec3::new(
                        mesh.positions[pos_idx],
                        mesh.positions[pos_idx + 1],
                        mesh.positions[pos_idx + 2],
                    ),
                    uv: Vec2::new(
                        mesh.texcoords.get(uv_idx).copied().unwrap_or(0.0),
                        mesh.texcoords.get(uv_idx + 1).copied().unwrap_or(0.0),
                    ),
                });

                vertices.push(vertex);
            }

            let num_new_vertices = mesh.indices.len();
            indices.extend((0..num_new_vertices as u32).map(|i| i + index_offset));
            indices.extend(mesh.indices.iter().map(|i| i + index_offset));
        }

        for index in indices.iter() {
            let _i = *index as usize;
        }

        Self::new(path, vertices, out_vert_generator, indices)
    }
}
