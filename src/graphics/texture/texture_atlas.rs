use crate::{
    graphics::texture::{AtlasBindings, texture_buffer_pool::TextureBufferPool, utils},
    utils::paths,
};
use glam::{UVec2, UVec3, Vec2};
use guillotiere::{AllocId, Allocation, AllocatorOptions, AtlasAllocator, Size};
use image::{DynamicImage, GenericImageView, ImageReader};
use std::{
    collections::{HashMap, HashSet},
    fmt::{self, Debug, Formatter},
    path::{Path, PathBuf},
};
use wgpu::{
    AddressMode, BindGroup, BindGroupLayout, CommandEncoder, Device, FilterMode, Sampler,
    SamplerDescriptor, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    TextureView, TextureViewDescriptor,
};

#[derive(Clone, Default)]
pub struct TextureAtlas {
    bindings: Option<AtlasBindings>,
    view: Option<TextureView>,
    sampler: Option<Sampler>,
    size: UVec2,
    items: HashMap<PathBuf, AtlasItem>,
}

#[derive(Debug, Default)]
struct AtlasAllocation {
    atlas_size: UVec2,
    atlas_items: HashMap<PathBuf, AtlasItem>,
}

#[derive(Clone, Debug)]
struct AtlasItem {
    pub allocation_id: AllocId,
    pub size: UVec2,
    pub position: UVec2,
    pub divisions: UVec2,
}

#[derive(Clone, Debug)]
pub struct AtlasItemUvData {
    pub offset: Vec2,
    pub size: Vec2,
    pub divisions: UVec2,
}

#[derive(Debug)]
enum ReinitImagePaths<'a> {
    Insert(&'a [PathBuf]),
    Remove(&'a [PathBuf]),
    Reset(&'a [PathBuf]),
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
            size,
            position,
            divisions,
        }
    }
}

impl Debug for TextureAtlas {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        #[derive(Debug)]
        struct TextureAtlas<'a> {
            view: &'a Option<TextureView>,
            sampler: &'a Option<Sampler>,
            size: UVec2,
        }

        Debug::fmt(
            &TextureAtlas {
                view: &self.view,
                sampler: &self.sampler,
                size: self.size(),
            },
            f,
        )
    }
}

impl TextureAtlas {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_init(&self) -> bool {
        self.view.is_some() && self.sampler.is_some()
    }

    pub fn view(&self) -> &Option<TextureView> {
        &self.view
    }

    pub fn view_unchecked(&self) -> &TextureView {
        &self
            .view
            .as_ref()
            .expect("Called TextureAtlas::view_unchecked on uninitialised TextureAtlas")
    }

    pub fn sampler(&self) -> &Option<Sampler> {
        &self.sampler
    }

    pub fn sampler_unchecked(&self) -> &Sampler {
        &self
            .sampler
            .as_ref()
            .expect("Called TextureAtlas::sampler_unchecked on uninitialised TextureAtlas")
    }

    pub fn bind_group_unchecked(&self) -> &BindGroup {
        &self
            .bindings
            .as_ref()
            .expect("Called TextureAtlas::bind_group_unchecked on uninitialised TextureAtlas")
            .bind_group
    }

    pub fn bind_group_layout_unchecked(&self) -> &BindGroupLayout {
        &self
            .bindings
            .as_ref()
            .expect(
                "Called TextureAtlas::bind_group_layout_unchecked on uninitialised TextureAtlas",
            )
            .bind_group_layout
    }

    pub fn size(&self) -> UVec2 {
        self.size
    }

    pub fn has(&self, texture_path: &PathBuf) -> bool {
        self.items.contains_key(texture_path)
    }

    pub fn get_texture_uv_data(&self, texture_path: &PathBuf) -> Option<AtlasItemUvData> {
        self.items.get(texture_path).map(|item| {
            let self_size = Vec2::new(self.size().x as f32, self.size().y as f32);

            let item_pos = Vec2::new(item.position.x as f32, item.position.y as f32);
            let offset = Vec2::ONE / (self_size / item_pos);

            let item_size = Vec2::new(item.size.x as f32, item.size.y as f32);
            let size = Vec2::ONE / (self_size / item_size);

            let uv_data = AtlasItemUvData {
                offset,
                size,
                divisions: item.divisions,
            };
            uv_data
        })
    }

    pub fn insert(
        &mut self,
        textures_to_insert: &[PathBuf],
        buffers: &mut TextureBufferPool,
        device: &Device,
        encoder: &mut CommandEncoder,
    ) {
        self.re_init(
            ReinitImagePaths::Insert(textures_to_insert),
            buffers,
            device,
            encoder,
        );
    }

    pub fn remove(
        &mut self,
        textures_to_remove: &[PathBuf],
        buffers: &mut TextureBufferPool,
        device: &Device,
        encoder: &mut CommandEncoder,
    ) {
        self.re_init(
            ReinitImagePaths::Remove(textures_to_remove),
            buffers,
            device,
            encoder,
        );
    }

    fn re_init(
        &mut self,
        image_paths: ReinitImagePaths,
        buffers: &mut TextureBufferPool,
        device: &Device,
        encoder: &mut CommandEncoder,
    ) {
        let mut reinit_paths = HashSet::<PathBuf>::with_capacity(self.items.len());
        match image_paths {
            ReinitImagePaths::Insert(_) | ReinitImagePaths::Remove(_) => {
                for p in self.items.keys() {
                    reinit_paths.insert(p.clone());
                }
            }
            _ => {}
        };

        match image_paths {
            ReinitImagePaths::Insert(paths) => {
                let old_paths_length = reinit_paths.len();
                for p in paths {
                    reinit_paths.insert(p.clone());
                }
                if old_paths_length == reinit_paths.len() {
                    return;
                }
            }
            ReinitImagePaths::Remove(paths) => {
                if !paths.is_empty() {
                    for p in paths {
                        reinit_paths.remove(p);
                    }
                }
            }
            ReinitImagePaths::Reset(paths) => {
                for p in paths {
                    reinit_paths.insert(p.clone());
                }
            }
        };

        let reinit_paths: Vec<_> = reinit_paths.into_iter().collect();
        let allocation = Self::allocate_items(&reinit_paths, buffers, device);
        self.items = allocation.atlas_items;
        self.size = allocation.atlas_size;
        self.reinit_gpu_texture(device, buffers, encoder);
    }

    fn allocate_items<P>(
        image_paths: &[P],
        buffers: &mut TextureBufferPool,
        device: &Device,
    ) -> AtlasAllocation
    where
        P: AsRef<Path> + Into<PathBuf> + Clone,
    {
        if image_paths.is_empty() {
            return AtlasAllocation::default();
        }

        let images: Vec<_> = image_paths.iter().map(Self::read_image).collect();
        let average_image_size = Self::average_image_size(&images);
        let incremental_texture_width =
            utils::align_to_bytes_per_row(average_image_size.x as usize);
        let incremental_image_size =
            UVec2::new(incremental_texture_width as u32, average_image_size.y);

        let initial_atlas_size = incremental_image_size * images.len() as u32;

        let allocator_options = AllocatorOptions::default();
        let mut allocator = AtlasAllocator::with_options(
            Self::uvec2_to_size(initial_atlas_size),
            &allocator_options,
        );

        let mut atlas_items = HashMap::<PathBuf, AtlasItem>::with_capacity(images.len());

        for (index, image) in images.iter().enumerate() {
            let (width, height) = image.dimensions();
            let size = Size::new(width as i32, height as i32);

            let mut allocation = allocator.allocate(size);
            while allocation.is_none() {
                allocator.grow(Self::uvec2_to_size(incremental_image_size));
                allocation = allocator.allocate(size);
            }

            let atlas_item =
                AtlasItem::from_allocation(&allocation.expect("Image was not allocated"));
            let image_key: PathBuf = image_paths[index].clone().into();

            atlas_items.insert(image_key.clone(), atlas_item);
            buffers.insert(image_key, image, device);
        }

        let allocator_size = allocator.size();
        let atlas_size = UVec2::new(allocator_size.width as u32, allocator_size.height as u32);

        AtlasAllocation {
            atlas_size,
            atlas_items,
        }
    }

    fn read_image<P>(image_path: &P) -> DynamicImage
    where
        P: AsRef<Path>,
    {
        let image_path = paths::texture(image_path);
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
        image
    }

    fn uvec2_to_size(vec: UVec2) -> Size {
        Size::new(vec.x as i32, vec.y as i32)
    }

    fn average_image_size(images: &[DynamicImage]) -> UVec2 {
        let mut sum = UVec2::ZERO;
        for image in images {
            let (width, height) = image.dimensions();
            sum += UVec2::new(width, height);
        }

        let total = UVec2::new(images.len() as u32, images.len() as u32);
        let average = sum / total;

        average
    }

    fn reinit_gpu_texture(
        &mut self,
        device: &Device,
        texture_buffers: &TextureBufferPool,
        encoder: &mut CommandEncoder,
    ) {
        if self.items.is_empty() || self.size == UVec2::ZERO {
            //self.view = None;
            //self.sampler = None;
            //return;
        }

        let extent = wgpu::Extent3d {
            width: self.size.x,
            height: self.size.y,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&TextureDescriptor {
            label: Some("Diffuse Texture Atlas"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });

        for (image_path, item) in self.items.iter() {
            texture_buffers
                .write_to_texture(
                    image_path,
                    &texture,
                    UVec3::new(item.position.x, item.position.y, 0),
                    encoder,
                )
                .expect("Failed to write to texture");
        }

        let view = texture.create_view(&TextureViewDescriptor::default());
        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::Repeat,
            address_mode_v: AddressMode::Repeat,
            address_mode_w: AddressMode::Repeat,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });
        let bindings = AtlasBindings::new(&view, &sampler, device);

        self.view = Some(view);
        self.sampler = Some(sampler);
        self.bindings = Some(bindings);
    }
}
