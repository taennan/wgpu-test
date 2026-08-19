use crate::{
    error::*,
    graphics::{
        CameraRenderer,
        MeshRenderer,
        SpriteRenderer,
        //geometry::GeometryPool,
        pipeline::{GlobalBindGroupLayouts, PipelinePool},
        screen_size::GpuScreenSize,
        surface::SurfaceConfigFactory,
        texture::TextureAtlas,
        //tilemap::renderer::TilemapRenderer,
    },
};
use glam::UVec2;
use std::sync::Arc;
use wgpu::{
    CommandBuffer, Device, DeviceDescriptor, Instance, InstanceDescriptor, PowerPreference, Queue,
    RequestAdapterOptions, Surface, SurfaceConfiguration,
};
use winit::window::Window;

pub struct GraphicsState {
    pub surface: Surface<'static>,
    pub surface_config: SurfaceConfiguration,
    pub is_surface_configured: bool,
    pub device: Device,
    pub queue: Queue,
    pub texture_atlas: TextureAtlas,
    //pub geometry_pool: GeometryPool,
    pub pipelines: PipelinePool,
    pub screen_size: GpuScreenSize,
    pub camera_renderer: CameraRenderer,
    pub mesh_renderer: MeshRenderer,
    pub sprite_renderer: SpriteRenderer,
    //pub tilemap_renderer: TilemapRenderer,
    pub command_buffers: Vec<CommandBuffer>,
}

impl GraphicsState {
    pub fn try_new(window: Arc<Window>) -> Result<Self> {
        let size = window.inner_size();
        let size = UVec2::new(size.width, size.height);
        if size.x == 0 || size.y == 0 {
            return Err(Error::WindowCreationFailed);
        }

        let instance_descriptor = InstanceDescriptor::new_without_display_handle();
        let instance = Instance::new(instance_descriptor);

        let surface = instance
            .create_surface(Arc::clone(&window))
            .map_err(|_| Error::SurfaceCreationFailed)?;

        let adapter =
            futures::executor::block_on(instance.request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::LowPower,
                compatible_surface: Some(&surface),
                apply_limit_buckets: false,
                // NOTE: The guide has this as 'true'. Don't know why
                force_fallback_adapter: false,
            }))
            .map_err(|_| Error::AdapterCreationFailed)?;

        let surface_config_factory = SurfaceConfigFactory::new(&surface, &adapter, size);
        let surface_config = surface_config_factory.try_build()?;

        let device_request =
            futures::executor::block_on(adapter.request_device(&DeviceDescriptor {
                label: Some("Device One"),
                required_features: wgpu::Features::MAPPABLE_PRIMARY_BUFFERS,
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            }))
            .map_err(|_| Error::SurfaceCreationFailed)?;

        let (device, queue) = device_request;

        let screen_size = GpuScreenSize::new(&device);
        let camera_renderer = CameraRenderer::new(&device);

        let texture_atlas = TextureAtlas::new(&device);
        //let geometry_pool = GeometryPool::new();

        let pipelines = PipelinePool::new(
            GlobalBindGroupLayouts {
                camera: camera_renderer.bind_group_layout(),
                screen_size: screen_size.bind_group_layout(),
                atlas: texture_atlas.bind_group_layout(),
            },
            surface_config.format,
            &device,
        );

        let mesh_renderer = MeshRenderer::new();
        let sprite_renderer = SpriteRenderer::new();
        //let tilemap_renderer = TilemapRenderer::new();

        Ok(Self {
            surface,
            surface_config,
            device,
            queue,
            texture_atlas,
            //geometry_pool,
            pipelines,
            screen_size,
            camera_renderer,
            mesh_renderer,
            sprite_renderer,
            //tilemap_renderer,
            command_buffers: vec![],
            is_surface_configured: false,
        })
    }
}
