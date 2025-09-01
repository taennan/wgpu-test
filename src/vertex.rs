use bytemuck::{Pod, Zeroable};
use wgpu::{VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Vertex {
    pub position: VertexPosition,
    pub texture_pos: VertexTextureCoords,
}

type VertexPosition = [f32; 3];
type VertexTextureCoords = [f32; 2];

impl Vertex {
    pub fn buffer_layout<'a>() -> VertexBufferLayout<'a> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: VertexFormat::Float32x3,
                },
                VertexAttribute {
                    offset: std::mem::size_of::<VertexPosition>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: VertexFormat::Float32x2,
                },
            ],
        }
    }
}

pub mod arrangements {
    use super::*;

    // NOTE: The texture pos fields on this are nonsensical
    /*
    pub const TRIANGLE: &[Vertex] = &[
        Vertex {
            position: [-0.5, -0.5, 0.0],
            texture_pos: [0.0, 0.0],
        },
        Vertex {
            position: [0.5, -0.5, 0.0],
            texture_pos: [1.0, 0.0],
        },
        Vertex {
            position: [0.0, 0.5, 0.0],
            texture_pos: [0.5, 1.0],
        },
    ];
     */

    pub const PENTAGON: &[Vertex] = &[
        Vertex {
            position: [-0.0868241, 0.49240386, 0.0],
            texture_pos: [0.4131759, 0.00759614],
        }, // A
        Vertex {
            position: [-0.49513406, 0.06958647, 0.0],
            texture_pos: [0.0048659444, 0.43041354],
        }, // B
        Vertex {
            position: [-0.21918549, -0.44939706, 0.0],
            texture_pos: [0.28081453, 0.949397],
        }, // C
        Vertex {
            position: [0.35966998, -0.3473291, 0.0],
            texture_pos: [0.85967, 0.84732914],
        }, // D
        Vertex {
            position: [0.44147372, 0.2347359, 0.0],
            texture_pos: [0.9414737, 0.2652641],
        }, // E
    ];

    pub const PENTAGON_INDICES: &[u16] = &[0, 1, 4, 4, 1, 2, 4, 2, 3];
}
