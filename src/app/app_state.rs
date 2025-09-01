use crate::{
    error::*,
    graphics::{
        Pipeline,
        camera::CameraManager,
        shaders::simple_shape,
        texture::{AlbatrossTexture, TexturePool},
    },
    surface::{SurfaceConfigFactory, SurfaceFactory},
    utils::Vec2,
    vertex,
};
use std::sync::Arc;
use wgpu::{
    Buffer, Device, DeviceDescriptor, Instance, InstanceDescriptor, PowerPreference, Queue,
    RequestAdapterOptions, Surface, SurfaceConfiguration,
    util::{BufferInitDescriptor, DeviceExt},
};
use winit::window::Window;

pub struct AppState {
    pub surface: Surface<'static>,
    pub surface_config: SurfaceConfiguration,
    pub is_surface_configured: bool,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub simple_bind_group: simple_shape::BindGroup,
    pub current_texture_type: AlbatrossTexture,
    pub texture_pool: TexturePool,
    pub vertex_buffer: Buffer,
    pub index_buffer: Buffer,
    pub num_indices: u32,
    pub render_pipeline: Pipeline,
    pub camera_manager: CameraManager,
    pub window: Arc<Window>,
    pub is_inited: bool,
}

impl AppState {
    pub async fn try_new(window: Arc<Window>) -> Result<Self> {
        let size = Vec2::from(window.inner_size());
        if size.x == 0 || size.y == 0 {
            return Err(Error::WindowCreationFailed);
        }

        let instance = Instance::new(&InstanceDescriptor::default());

        let surface_factory = SurfaceFactory::new(&instance, window.clone());
        let surface = surface_factory.try_build()?;

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::LowPower,
                compatible_surface: Some(&surface),
                // NOTE: The guide has this as 'true'. Don't know why
                force_fallback_adapter: false,
            })
            .await
            .map_err(|_| Error::AdapterCreationFailed)?;

        let surface_config_factory = SurfaceConfigFactory::new(&surface, &adapter, size);
        let surface_config = surface_config_factory.try_build()?;

        let device_request = adapter
            .request_device(&DeviceDescriptor {
                label: Some("Device One"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await
            .map_err(|_| Error::SurfaceCreationFailed)?;
        let device = Arc::new(device_request.0);
        let queue = Arc::new(device_request.1);

        let vertices = vertex::arrangements::PENTAGON;
        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Shape Vertex Buffer"),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let indices = vertex::arrangements::PENTAGON_INDICES;
        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Shape Index Buffer"),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let mut texture_pool = TexturePool::new(device.clone(), queue.clone());
        let current_texture_type = AlbatrossTexture::Light;
        let current_texture_key = &current_texture_type.to_str();

        texture_pool.load(current_texture_key)?;
        let diffuse_texture = texture_pool
            .get(current_texture_key)
            .ok_or(Error::AssetLoadingFailed)?;

        let simple_bind_group = simple_shape::BindGroup::new(&diffuse_texture, &device);

        let mut camera_manager = CameraManager::new(&device);
        camera_manager.attributes.position = cgmath::Point3::new(0.0, 1.0, 2.0);
        camera_manager.attributes.aspect =
            surface_config.width as f32 / surface_config.height as f32;
        //camera_manager.update_staging_buffer();

        let render_pipeline = Pipeline::new(
            "Simple Shape",
            "simple_shape",
            &device,
            surface_config.format,
            &[
                &simple_bind_group.layout(),
                &camera_manager.bind_group_layout(),
            ],
        );

        Ok(Self {
            window,
            surface,
            surface_config,
            device,
            queue,
            simple_bind_group,
            texture_pool,
            current_texture_type,
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
            render_pipeline,
            camera_manager,
            is_surface_configured: false,
            is_inited: false,
        })
    }
}
