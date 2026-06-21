use crate::graphics::{
    bind_group::{BindGroupBuilder, BindGroupLayoutBuilder},
    buffer::{MutBuffer, MutBufferBuilder},
    texture::{buffer_data::AtlasItemMetaBufferData, image::AtlasImage, utils},
};
use glam::{UVec2, Vec2};
use guillotiere::{AllocId, Allocation, AtlasAllocator};
use std::{
    collections::HashMap,
    fmt::{self, Debug, Formatter},
    path::{Path, PathBuf},
};
use wgpu::{
    AddressMode, BindGroup, BindGroupLayout, BindingResource, BindingType, Buffer, BufferBinding,
    BufferBindingType, BufferUsages, CommandEncoder, Device, Extent3d, FilterMode,
    MipmapFilterMode, Origin3d, Queue, Sampler, SamplerBindingType, SamplerDescriptor,
    ShaderStages, TexelCopyBufferLayout, TexelCopyTextureInfo, Texture, TextureAspect,
    TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType, TextureUsages,
    TextureView, TextureViewDescriptor, TextureViewDimension,
    util::{BufferInitDescriptor, DeviceExt},
};

#[derive(Clone)]
pub struct TextureAtlas {
    bind_group_layout: BindGroupLayout,
    bind_group: BindGroup,
    atlas_padding_buffer: Buffer,
    atlas_items_buffer: MutBuffer,
    texture_view: TextureView,
    sampler: Sampler,
    items: HashMap<PathBuf, AtlasItem>,
    allocator: AtlasAllocator,
}

#[derive(Clone, Debug)]
struct AtlasItem {
    pub allocation_id: AllocId,
    pub total_dependants: u32,
    pub buffer_index: u32,
    pub size: UVec2,
    pub position: UVec2,
    pub divisions: UVec2,
}

impl AtlasItem {
    fn from_allocation(allocation: &Allocation) -> Self {
        return Self::from_allocation_with_divisions(allocation, UVec2::ONE);
    }

    fn from_allocation_with_divisions(allocation: &Allocation, divisions: UVec2) -> Self {
        let rect = allocation.rectangle;
        let size = UVec2::new(rect.size().width as u32, rect.size().height as u32);
        let position = UVec2::new(rect.min.x as u32, rect.min.y as u32);

        Self {
            allocation_id: allocation.id,
            total_dependants: 0,
            buffer_index: 0,
            size,
            position,
            divisions,
        }
    }
}

impl Debug for TextureAtlas {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        #[derive(Debug)]
        struct TextureAtlas {
            size: UVec2,
            items: HashMap<PathBuf, AtlasItem>,
        }

        Debug::fmt(
            &TextureAtlas {
                size: self.size(),
                items: self.items.clone(),
            },
            f,
        )
    }
}

impl TextureAtlas {
    pub fn new(device: &Device) -> Self {
        Self::new_with_padding(device, 0)
    }

    fn new_with_padding(device: &Device, padding: u32) -> Self {
        let texture_size = UVec2::ONE + UVec2::new(padding, padding) * 2;
        let (_, texture_view) = Self::init_gpu_texture(texture_size, device);

        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("TextureAtlas Sampler"),
            address_mode_u: AddressMode::Repeat,
            address_mode_v: AddressMode::Repeat,
            address_mode_w: AddressMode::Repeat,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Nearest,
            mipmap_filter: MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let atlas_padding_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Atlas Padding Buffer"),
            usage: BufferUsages::UNIFORM,
            contents: bytemuck::cast_slice(&[padding]),
        });

        let (bind_group_layout, bind_group, atlas_items_buffer) = Self::init_atlas_items_buffer(
            &[],
            &texture_view,
            &sampler,
            &atlas_padding_buffer,
            device,
        );

        Self {
            bind_group,
            bind_group_layout,
            atlas_padding_buffer,
            atlas_items_buffer,
            texture_view,
            sampler,
            items: HashMap::default(),
            allocator: AtlasAllocator::new(utils::uvec2_to_size(texture_size)),
        }
    }

    fn init_gpu_texture(texture_size: UVec2, device: &Device) -> (Texture, TextureView) {
        let extent = Extent3d {
            width: texture_size.x,
            height: texture_size.y,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&TextureDescriptor {
            label: Some("Diffuse Texture Atlas"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let texture_view = texture.create_view(&TextureViewDescriptor {
            label: Some("TextureAtlas Texture View"),
            ..Default::default()
        });

        (texture, texture_view)
    }

    fn init_atlas_items_buffer(
        atlas_items_meta: &[AtlasItemMetaBufferData],
        texture_view: &TextureView,
        sampler: &Sampler,
        atlas_padding_buffer: &Buffer,
        device: &Device,
    ) -> (BindGroupLayout, BindGroup, MutBuffer) {
        let bind_group_layout = BindGroupLayoutBuilder::new()
            .name("TextureAtlas Bind Group Layout")
            .entry(
                ShaderStages::VERTEX_FRAGMENT,
                BindingType::Texture {
                    sample_type: TextureSampleType::Float { filterable: true },
                    view_dimension: TextureViewDimension::D2,
                    multisampled: false,
                },
            )
            .entry(
                ShaderStages::FRAGMENT,
                BindingType::Sampler(SamplerBindingType::Filtering),
            )
            .entry(
                ShaderStages::VERTEX,
                BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
            )
            .entry(
                ShaderStages::VERTEX,
                BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
            )
            .build(device);

        let atlas_items_buffer_builder = MutBufferBuilder::default()
            .name("Atlas Items Metadata Buffer")
            .usages(BufferUsages::MAP_WRITE | BufferUsages::STORAGE);
        let atlas_items_buffer = match atlas_items_meta.len() {
            0 => atlas_items_buffer_builder.build(256u64, device),
            _ => atlas_items_buffer_builder.build_init(&atlas_items_meta, device),
        };

        let bind_group = BindGroupBuilder::new()
            .name("TextureAtlas Bind Group")
            .entry(BindingResource::TextureView(&texture_view))
            .entry(BindingResource::Sampler(&sampler))
            .entry(BindingResource::Buffer(BufferBinding {
                buffer: &atlas_padding_buffer,
                offset: 0,
                size: None,
            }))
            .entry(BindingResource::Buffer(BufferBinding {
                buffer: atlas_items_buffer.buffer(),
                offset: 0,
                size: None,
            }))
            .build(&bind_group_layout, device);

        (bind_group_layout, bind_group, atlas_items_buffer)
    }

    pub fn atlas_item_index<P>(&self, texture_path: P) -> Option<u32>
    where
        P: AsRef<Path>,
    {
        self.items
            .get(texture_path.as_ref())
            .map(|item| item.buffer_index)
    }

    pub fn texture_view(&self) -> &TextureView {
        &self.texture_view
    }

    pub fn sampler(&self) -> &Sampler {
        &self.sampler
    }

    pub fn bind_group(&self) -> &BindGroup {
        &self.bind_group
    }

    pub fn bind_group_layout(&self) -> &BindGroupLayout {
        &self.bind_group_layout
    }

    pub fn size(&self) -> UVec2 {
        UVec2 {
            x: self.allocator.size().width as u32,
            y: self.allocator.size().height as u32,
        }
    }

    pub fn has(&self, texture_path: &PathBuf) -> bool {
        self.items.contains_key(texture_path)
    }

    pub fn insert(
        &mut self,
        textures: &[PathBuf],
        device: &Device,
        queue: &mut Queue,
        encoder: &mut CommandEncoder,
    ) {
        let new_textures = textures
            .iter()
            .filter(|p| !self.items.contains_key(*p))
            .collect::<Vec<_>>();
        let new_images = new_textures
            .iter()
            .map(AtlasImage::open)
            .collect::<Vec<_>>();
        if new_images.is_empty() {
            return;
        }

        let original_size = self.size();
        let mut new_items = Vec::<(AtlasItem, &AtlasImage)>::with_capacity(new_images.len());

        for image in &new_images {
            let image_sizes = new_images
                .iter()
                .map(|v| v.size)
                .chain(self.items.values().map(|v| v.size))
                .collect::<Vec<_>>();
            let increment_size = utils::uvec2_to_size(utils::average_uvec2(&image_sizes));
            let size = utils::uvec2_to_size(image.size);

            log::debug!("Atlas increment size: {:?}", increment_size);
            log::debug!("Allocation size {:?}", size);

            let mut allocation = self.allocator.allocate(size);
            while allocation.is_none() {
                let new_size = self.allocator.size() + increment_size;
                self.allocator.grow(new_size);
                allocation = self.allocator.allocate(size);
                log::debug!("  Allocator size: {:?}", self.allocator.size());
                log::debug!("  Allocation: {:?}", allocation);
            }

            let allocation = allocation.expect("Image was not allocated");
            let atlas_item = AtlasItem::from_allocation(&allocation);
            let item_key = image.path.clone();
            self.items.insert(item_key, atlas_item.clone());
            new_items.push((atlas_item, image));
        }

        let did_grow = self.size() != original_size;
        if did_grow {
            let (new_texture, new_texture_view) = Self::init_gpu_texture(self.size(), device);
            encoder.copy_texture_to_texture(
                TexelCopyTextureInfo {
                    texture: self.texture_view().texture(),
                    mip_level: 0,
                    origin: Origin3d::ZERO,
                    aspect: TextureAspect::All,
                },
                TexelCopyTextureInfo {
                    texture: &new_texture,
                    mip_level: 0,
                    origin: Origin3d::ZERO,
                    aspect: TextureAspect::All,
                },
                self.texture_view().texture().size(),
            );
            self.texture_view = new_texture_view;
        }

        for (item, image) in new_items {
            let rgba_bytes = image.image.to_rgba8();
            let image_bytes_per_row = Some(image.size.x * 4);

            queue.write_texture(
                TexelCopyTextureInfo {
                    texture: self.texture_view().texture(),
                    mip_level: 0,
                    origin: Origin3d {
                        x: item.position.x,
                        y: item.position.y,
                        z: 0,
                    },
                    aspect: TextureAspect::All,
                },
                &rgba_bytes,
                TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: image_bytes_per_row,
                    rows_per_image: None,
                },
                Extent3d {
                    width: image.size.x,
                    height: image.size.y,
                    depth_or_array_layers: 1,
                },
            );
        }

        for texture_path in textures {
            if let Some(item) = self.items.get_mut(texture_path) {
                item.total_dependants += 1;
            }
        }

        let atlas_items_meta = self
            .items
            .values_mut()
            .enumerate()
            .map(|(index, item)| {
                item.buffer_index = index as u32;
                let atlas_item_meta =
                    AtlasItemMetaBufferData::new(item.position, item.size, item.divisions);
                atlas_item_meta
            })
            .collect::<Vec<_>>();
        let (bind_group_layout, bind_group, atlas_items_buffer) = Self::init_atlas_items_buffer(
            &atlas_items_meta,
            &self.texture_view,
            &self.sampler,
            &self.atlas_padding_buffer,
            device,
        );
        self.bind_group_layout = bind_group_layout;
        self.bind_group = bind_group;
        self.atlas_items_buffer = atlas_items_buffer;
        queue.submit([]);
    }

    fn remove(&mut self, textures: &[PathBuf]) {
        for texture in textures {
            if let Some(item) = self.items.get_mut(texture) {
                item.total_dependants -= 1;
                if item.total_dependants == 0 {
                    self.items.remove(texture);
                }
            }
        }
    }
}
