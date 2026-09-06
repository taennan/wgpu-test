use crate::{
    graphics::{
        bind_group::{BindGroupBuilder, BindGroupLayoutBuilder},
        buffer::{MutBuffer, MutBufferBuilder},
        texture::{buffer_data::AtlasItemMetaBufferData, image::AtlasImage, utils},
    },
    utils::multiples,
};
use glam::UVec2;
use guillotiere::{AllocId, Allocation, AtlasAllocator};
use image::DynamicImage;
use std::{
    collections::HashMap,
    fmt::{self, Debug, Formatter},
    path::{Path, PathBuf},
};
use wgpu::{
    AddressMode, BindGroup, BindGroupLayout, BindingResource, BindingType, Buffer, BufferBinding,
    BufferBindingType, BufferUsages, COPY_BYTES_PER_ROW_ALIGNMENT, CommandEncoder, Device,
    Extent3d, FilterMode, MipmapFilterMode, Origin3d, Queue, Sampler, SamplerBindingType,
    SamplerDescriptor, ShaderStages, TexelCopyBufferLayout, TexelCopyTextureInfo, Texture,
    TextureAspect, TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType,
    TextureUsages, TextureView, TextureViewDescriptor, TextureViewDimension,
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
    pub buffer_index: u16,
    pub size: UVec2,
    pub image_size: UVec2,
    pub position: UVec2,
    pub divisions: UVec2,
}

impl AtlasItem {
    fn new(allocation: &Allocation, image_size: UVec2, buffer_index: u16) -> Self {
        return Self::new_with_divisions(allocation, image_size, buffer_index, UVec2::ONE);
    }

    fn new_with_divisions(
        allocation: &Allocation,
        image_size: UVec2,
        buffer_index: u16,
        divisions: UVec2,
    ) -> Self {
        let rect = allocation.rectangle;
        let size = UVec2::new(rect.size().width as u32, rect.size().height as u32);
        let position = UVec2::new(rect.min.x as u32, rect.min.y as u32);

        Self {
            allocation_id: allocation.id,
            buffer_index,
            size,
            image_size,
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

        let sampler_address_mode = AddressMode::ClampToEdge;
        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("TextureAtlas Sampler"),
            address_mode_u: sampler_address_mode,
            address_mode_v: sampler_address_mode,
            address_mode_w: sampler_address_mode,
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
            label: Some("Diffuse TextureAtlas"),
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
            label: Some("Diffuse TextureAtlas Texture View"),
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
                ShaderStages::FRAGMENT,
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
                ShaderStages::FRAGMENT,
                BindingType::Buffer {
                    ty: BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
            )
            .entry(
                ShaderStages::FRAGMENT,
                BindingType::Buffer {
                    ty: BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
            )
            .build(device);

        log::debug!("Writing atlas items meta to buffer {:?}", atlas_items_meta);

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

    pub fn atlas_item_index<P>(&self, texture_path: P) -> Option<u16>
    where
        P: AsRef<Path>,
    {
        self.items
            .get(texture_path.as_ref())
            .map(|item| item.buffer_index as u16)
    }

    pub fn texture(&self) -> &Texture {
        &self.texture_view().texture()
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
        let original_size = self.size();
        let new_items = self.insert_new_atlas_items(textures);

        self.grow_allocator_to_mappable_mutliple();

        let did_grow = self.size() != original_size;
        if did_grow {
            self.migrate_to_new_texture(device, encoder);
        }

        self.write_new_atlas_items_to_texture(&new_items, queue);

        let mut atlas_items = self.items.values_mut().collect::<Vec<_>>();
        atlas_items.sort_by(|a, b| {
            a.allocation_id
                .serialize()
                .cmp(&b.allocation_id.serialize())
        });

        let atlas_items_meta = atlas_items
            .into_iter()
            .enumerate()
            .map(|(index, item)| {
                item.buffer_index = index as u16;
                let atlas_item_meta =
                    AtlasItemMetaBufferData::new(item.position, item.size, item.divisions);
                atlas_item_meta
            })
            .collect::<Vec<_>>();

        log::debug!("AtlasItemsMeta = {:?}", atlas_items_meta);

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

    fn insert_new_atlas_items(&mut self, textures: &[PathBuf]) -> Vec<(AtlasItem, DynamicImage)> {
        let new_textures = textures
            .iter()
            .filter(|p| !self.items.contains_key(*p))
            .collect::<Vec<_>>();
        let new_images = new_textures
            .iter()
            .map(AtlasImage::open)
            .collect::<Vec<_>>();
        if new_images.is_empty() {
            return vec![];
        }

        let image_sizes = new_images
            .iter()
            .map(|v| v.size)
            .chain(self.items.values().map(|v| v.size))
            .collect::<Vec<_>>();
        let increment_size = utils::uvec2_to_size(utils::average_uvec2(&image_sizes));

        let mut new_items = Vec::with_capacity(new_images.len());
        for image in new_images.into_iter() {
            let size = utils::uvec2_to_size(image.size);

            let mut allocation = self.allocator.allocate(size);
            while allocation.is_none() {
                let new_size = self.allocator.size() + increment_size;
                self.allocator.grow(new_size);
                allocation = self.allocator.allocate(size);
            }

            let allocation = allocation.expect("Image was not allocated");
            let atlas_item = AtlasItem::new(&allocation, image.size, 0);
            let item_key = image.path.clone();

            self.items.insert(item_key, atlas_item.clone());
            new_items.push((atlas_item, image.image));
        }

        new_items
    }

    fn grow_allocator_to_mappable_mutliple(&mut self) {
        let mappable_multiple = COPY_BYTES_PER_ROW_ALIGNMENT;
        let x = multiples::next_nearest(self.size().x, mappable_multiple);
        let y = multiples::next_nearest(self.size().y, mappable_multiple);

        self.allocator.grow(utils::uvec2_to_size(UVec2::new(x, y)));
    }

    fn migrate_to_new_texture(&mut self, device: &Device, encoder: &mut CommandEncoder) {
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

    fn write_new_atlas_items_to_texture(
        &mut self,
        new_items: &[(AtlasItem, DynamicImage)],
        queue: &Queue,
    ) {
        for (item, image) in new_items {
            let image_bytes_per_row = Some(item.image_size.x * 4);

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
                &image.to_rgba8(),
                TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: image_bytes_per_row,
                    rows_per_image: None,
                },
                Extent3d {
                    width: item.image_size.x,
                    height: item.image_size.y,
                    depth_or_array_layers: 1,
                },
            );
        }
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.allocator.clear();
    }

    fn remove(&mut self, textures: &[PathBuf]) {
        for texture in textures {
            self.items.remove(texture);
        }
    }
}
