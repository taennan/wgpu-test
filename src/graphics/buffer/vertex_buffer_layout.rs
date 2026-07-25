use std::mem;
use wgpu::{VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

pub struct VertexBufferLayoutBuilder {
    stride: Option<u64>,
    step_mode: Option<VertexStepMode>,
    attributes: Vec<VertexAttribute>,
}

impl VertexBufferLayoutBuilder {
    pub fn new() -> Self {
        Self {
            stride: None,
            step_mode: None,
            attributes: vec![],
        }
    }

    pub fn array_stride<S>(mut self, value: S) -> Self
    where
        S: Into<u64>,
    {
        self.stride = Some(value.into());
        self
    }

    pub fn step_mode(mut self, value: VertexStepMode) -> Self {
        self.step_mode = Some(value);
        self
    }

    pub fn attribute(mut self, format: VertexFormat) -> Self {
        let total_attributes = self.attributes.len();
        let offset = self.last_attribute_end_size();

        self.attributes.push(VertexAttribute {
            format,
            offset,
            shader_location: total_attributes as u32,
        });
        self
    }

    pub fn build<T>(
        &self,
        shader_location_start: u32,
        step_mode: VertexStepMode,
    ) -> VertexBufferLayout<'static> {
        VertexBufferLayout {
            array_stride: mem::size_of::<T>() as u64,
            step_mode: step_mode,
            attributes: &self.attributes.clone(),
        }
    }

    fn last_attribute_end_size(&self) -> u64 {
        let total_attributes = self.attributes.len();
        let offset = match total_attributes {
            0 => 0,
            _ => {
                let last_attribute = self.attributes[total_attributes - 1];
                last_attribute.offset + last_attribute.format.size()
            }
        };

        offset
    }
}
