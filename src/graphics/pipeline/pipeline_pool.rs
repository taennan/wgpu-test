use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    rc::Rc,
};
use wgpu::{
    BindGroupLayout, Device, PipelineCompilationOptions, PipelineLayoutDescriptor, RenderPipeline,
    RenderPipelineDescriptor, ShaderModuleDescriptor, ShaderSource, TextureFormat,
    VertexBufferLayout,
};

pub struct PipelinePool {
    texture_format: TextureFormat,
    pipelines: HashMap<PathBuf, PipelineData>,
    device: Rc<Device>,
}

pub struct PipelineData {
    pub pipeline: RenderPipeline,
    pub bind_group_layouts: Vec<BindGroupLayout>,
}

pub struct CreatePipelineInput<'a> {
    pub shader_path: &'a Path,
    pub bind_group_layouts: &'a [BindGroupLayout],
    pub vertex_buffer_layouts: &'a [VertexBufferLayout<'a>],
}

impl PipelinePool {
    pub fn new(texture_format: TextureFormat, device: Rc<Device>) -> Self {
        Self {
            pipelines: HashMap::new(),
            texture_format,
            device,
        }
    }

    pub fn has<P>(&self, shader_path: &P) -> bool
    where
        P: AsRef<Path>,
    {
        let key = self.key(shader_path);
        self.pipelines.contains_key(&key)
    }

    pub fn get<P>(&self, shader_path: &P) -> Option<&PipelineData>
    where
        P: AsRef<Path>,
    {
        let key = self.key(shader_path);
        self.pipelines.get(&key)
    }

    pub fn get_unchecked<P>(&self, shader_path: &P) -> &PipelineData
    where
        P: AsRef<Path>,
    {
        self.get(shader_path)
            .expect("RenderPipeline was not loaded before accessing")
    }

    pub fn unload<P>(&mut self, shader_path: &P)
    where
        P: AsRef<Path>,
    {
        let key = self.key(shader_path);
        self.pipelines.remove(&key);
    }

    fn key<P>(&self, shader_path: &P) -> PathBuf
    where
        P: AsRef<Path>,
    {
        PathBuf::from(shader_path.as_ref())
    }

    pub fn load(&mut self, input: &CreatePipelineInput) {
        if self.has(&input.shader_path) {
            return;
        }

        let pipeline = self.create_pipeline(input);
        let key = self.key(&input.shader_path);

        let data = PipelineData {
            pipeline,
            bind_group_layouts: input.bind_group_layouts.to_vec(),
        };
        self.pipelines.insert(key, data);
    }

    fn create_pipeline(&mut self, input: &CreatePipelineInput) -> RenderPipeline {
        log::debug!("READING SHADER FILE {:?}", input.shader_path);
        let shader_source_bytes = fs::read(&input.shader_path).expect("Failed to read shader file");
        let shader_source_text = String::from_utf8_lossy(&shader_source_bytes);
        let name = &input.shader_path.to_str().unwrap_or("Unkown");

        let shader = self.device.create_shader_module(ShaderModuleDescriptor {
            label: Some(&format!("{} Shader", name)),
            source: ShaderSource::Wgsl(shader_source_text.into()),
        });

        let layouts = input
            .bind_group_layouts
            .iter()
            .map(|l| Some(l))
            .collect::<Vec<_>>();
        let render_pipeline_layout =
            self.device
                .create_pipeline_layout(&PipelineLayoutDescriptor {
                    label: Some(&format!("{} Pipeline Layout", name)),
                    bind_group_layouts: &layouts,
                    immediate_size: 0,
                });
        let render_pipeline = self
            .device
            .create_render_pipeline(&RenderPipelineDescriptor {
                label: Some(&format!("{} Pipeline", name)),
                layout: Some(&render_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    //entry_point: None,
                    entry_point: Some("vertex_main"),
                    buffers: input.vertex_buffer_layouts,
                    compilation_options: PipelineCompilationOptions::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    //entry_point: None,
                    entry_point: Some("fragment_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: self.texture_format,
                        blend: Some(wgpu::BlendState::REPLACE),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: PipelineCompilationOptions::default(),
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::Ccw,
                    cull_mode: Some(wgpu::Face::Back),
                    polygon_mode: wgpu::PolygonMode::Fill,
                    unclipped_depth: false,
                    conservative: false,
                },
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: u64::MAX,
                    alpha_to_coverage_enabled: false,
                },
                depth_stencil: None,
                multiview_mask: None,
                cache: None,
            });

        render_pipeline
    }
}
