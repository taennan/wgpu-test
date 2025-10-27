use crate::{
    error::*,
    graphics::{
        pipeline::PipelinePool,
        surface::{SurfaceConfigFactory, SurfaceFactory},
        texture::TexturePool,
    },
    scene::{Scene, SceneRenderer},
    utils::paths,
};
use glam::UVec2;
use std::{collections::HashSet, fs, sync::Arc};
use wgpu::{
    Device, DeviceDescriptor, Instance, InstanceDescriptor, PowerPreference, Queue,
    RequestAdapterOptions, Surface, SurfaceConfiguration,
};
use winit::{keyboard::KeyCode, window::Window};

pub struct AppState {
    pub surface: Surface<'static>,
    pub surface_config: SurfaceConfiguration,
    pub is_surface_configured: bool,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    pub texture_pool: TexturePool,
    pub pipeline_pool: PipelinePool,
    pub scene: Scene,
    pub scene_renderer: SceneRenderer,
    pub staged_scene: Option<Scene>,
    pub staged_scene_renderer: Option<SceneRenderer>,
    pub keys_pressed: HashSet<KeyCode>,
    pub window: Arc<Window>,
    pub is_inited: bool,
}

impl AppState {
    pub fn try_new(start_scene_name: &str, window: Arc<Window>) -> Result<Self> {
        let size = window.inner_size();
        let size = UVec2::new(size.width, size.height);
        if size.x == 0 || size.y == 0 {
            return Err(Error::WindowCreationFailed);
        }

        let instance = Instance::new(&InstanceDescriptor::default());

        let surface_factory = SurfaceFactory::new(&instance, window.clone());
        let surface = surface_factory.try_build()?;

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

        let device = Arc::new(device_request.0);
        let queue = Arc::new(device_request.1);

        let texture_pool = TexturePool::new(device.clone(), queue.clone());
        let pipeline_pool = PipelinePool::new(surface_config.format, device.clone());

        let scene_path = paths::scene(start_scene_name);
        let scene_toml = fs::read(scene_path).map_err(|_| {
            log::error!("Failed to read scene file");
            Error::AssetLoadingFailed
        })?;
        let scene = toml::from_slice::<Scene>(&scene_toml).map_err(|err| {
            log::error!("Failed to parse scene file: {:?}", err);
            Error::AssetLoadingFailed
        })?;

        let scene_renderer = SceneRenderer::new(&device);
        /*
        let mut camera_manager = Camera::new(&device);
        camera_manager.attributes.position = cgmath::Point3::new(0.0, 1.0, 2.0);
        camera_manager.attributes.aspect =
            surface_config.width as f32 / surface_config.height as f32;
         */

        Ok(Self {
            window,
            surface,
            surface_config,
            device,
            queue,
            texture_pool,
            pipeline_pool,
            scene,
            scene_renderer,
            staged_scene: None,
            staged_scene_renderer: None,
            keys_pressed: HashSet::new(),
            is_surface_configured: false,
            is_inited: false,
        })
    }
}
