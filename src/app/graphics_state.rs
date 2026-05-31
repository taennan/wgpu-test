use crate::{
    error::*,
    graphics::{
        CameraRenderer, MeshRenderer, SpriteRenderer,
        geometry::GeometryPool,
        pipeline::PipelinePool,
        surface::SurfaceConfigFactory,
        texture::{TextureAtlas, TextureBufferPool},
    },
};
use glam::UVec2;
use std::{rc::Rc, sync::Arc};
use wgpu::{
    CommandBuffer, Device, DeviceDescriptor, Instance, InstanceDescriptor, PowerPreference, Queue,
    RequestAdapterOptions, Surface, SurfaceConfiguration,
};
use winit::window::Window;

pub struct GraphicsState {
    pub surface: Surface<'static>,
    pub surface_config: SurfaceConfiguration,
    pub is_surface_configured: bool,
    pub device: Rc<Device>,
    pub queue: Queue,
    pub texture_atlas: TextureAtlas,
    pub texture_buffers: TextureBufferPool,
    pub geometry_pool: GeometryPool,
    pub pipeline_pool: PipelinePool,
    pub camera_renderer: CameraRenderer,
    pub mesh_renderer: MeshRenderer,
    pub sprite_renderer: SpriteRenderer,
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
                // NOTE: The guide has this as 'true'. Don't know why
                force_fallback_adapter: false,
            }))
            .map_err(|_| Error::AdapterCreationFailed)?;

        let surface_config_factory = SurfaceConfigFactory::new(&surface, &adapter, size);
        let surface_config = surface_config_factory.try_build()?;

        let device_request =
            futures::executor::block_on(adapter.request_device(&DeviceDescriptor {
                label: Some("Device One"),
                // NOTE: For some reason, we need this feature to use map_async on Buffers
                required_features: wgpu::Features::MAPPABLE_PRIMARY_BUFFERS,
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            }))
            .map_err(|_| Error::SurfaceCreationFailed)?;

        let device = Rc::new(device_request.0);
        let queue = device_request.1;

        let texture_atlas = TextureAtlas::new(&device);
        let texture_buffers = TextureBufferPool::new();

        let geometry_pool = GeometryPool::new();

        let mut pipeline_pool = PipelinePool::new(surface_config.format, device.clone());

        let camera_renderer = CameraRenderer::new(&device);
        let mesh_renderer = MeshRenderer::new(&camera_renderer, &mut pipeline_pool, &device);
        let sprite_renderer = SpriteRenderer::new(&texture_atlas, &mut pipeline_pool, &device);

        Ok(Self {
            surface,
            surface_config,
            device,
            queue,
            texture_atlas,
            texture_buffers,
            geometry_pool,
            pipeline_pool,
            camera_renderer,
            mesh_renderer,
            sprite_renderer,
            command_buffers: vec![],
            is_surface_configured: false,
        })
    }
}
