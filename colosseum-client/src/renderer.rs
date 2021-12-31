use bytemuck::Pod;
use bytemuck::Zeroable;
use gltf::mesh::util::ReadIndices;
use gltf::mesh::util::ReadTexCoords;
use image::GenericImageView;
use log::warn;
use nalgebra::Matrix4;
use wgpu::Adapter;
use wgpu::Backends;
use wgpu::BindGroup;
use wgpu::BindGroupDescriptor;
use wgpu::BindGroupEntry;
use wgpu::BindGroupLayout;
use wgpu::BindGroupLayoutDescriptor;
use wgpu::BindGroupLayoutEntry;
use wgpu::BindingResource;
use wgpu::BindingType;
use wgpu::Buffer;
use wgpu::BufferAddress;
use wgpu::BufferBindingType;
use wgpu::BufferDescriptor;
use wgpu::BufferSize;
use wgpu::BufferUsages;
use wgpu::Color;
use wgpu::ColorTargetState;
use wgpu::ColorWrites;
use wgpu::CommandEncoderDescriptor;
use wgpu::CompareFunction;
use wgpu::DepthBiasState;
use wgpu::DepthStencilState;
use wgpu::Device;
use wgpu::DeviceDescriptor;
use wgpu::Extent3d;
use wgpu::Face;
use wgpu::Features;
use wgpu::FragmentState;
use wgpu::FrontFace;
use wgpu::IndexFormat;
use wgpu::Instance;
use wgpu::Limits;
use wgpu::LoadOp;
use wgpu::MultisampleState;
use wgpu::Operations;
use wgpu::PipelineLayout;
use wgpu::PipelineLayoutDescriptor;
use wgpu::PowerPreference;
use wgpu::PresentMode;
use wgpu::PrimitiveState;
use wgpu::PrimitiveTopology;
use wgpu::Queue;
use wgpu::RenderPassColorAttachment;
use wgpu::RenderPassDepthStencilAttachment;
use wgpu::RenderPassDescriptor;
use wgpu::RenderPipeline;
use wgpu::RenderPipelineDescriptor;
use wgpu::RequestAdapterOptions;
use wgpu::ShaderModule;
use wgpu::ShaderStages;
use wgpu::StencilState;
use wgpu::Surface;
use wgpu::SurfaceConfiguration;
use wgpu::SurfaceError;
use wgpu::Texture;
use wgpu::TextureDescriptor;
use wgpu::TextureDimension;
use wgpu::TextureFormat;
use wgpu::TextureSampleType;
use wgpu::TextureUsages;
use wgpu::TextureView;
use wgpu::TextureViewDescriptor;
use wgpu::TextureViewDimension;
use wgpu::VertexAttribute;
use wgpu::VertexBufferLayout;
use wgpu::VertexFormat;
use wgpu::VertexState;
use wgpu::VertexStepMode;
use wgpu::include_wgsl;
use wgpu::util::BufferInitDescriptor;
use wgpu::util::DeviceExt;
use winit::dpi::PhysicalSize;

use crate::error::Error;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    tex_coord: [f32; 2],
}

impl Vertex {
    fn desc<'a>() -> VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    format: VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                VertexAttribute {
                    format: VertexFormat::Float32x3,
                    offset: std::mem::size_of::<[f32; 3]>() as BufferAddress,
                    shader_location: 1,
                },
                VertexAttribute {
                    format: VertexFormat::Float32x2,
                    offset: std::mem::size_of::<[f32; 6]>() as BufferAddress,
                    shader_location: 2,
                },
            ]
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Locals {
    mvp: [[f32; 4]; 4],
}

pub struct Renderer {
    instance: Instance,
    surface: Surface,
    adapter: Adapter,
    device: Device,
    queue: Queue,
    surface_configuration: SurfaceConfiguration,
    shader: ShaderModule,
    bind_group_layout: BindGroupLayout,
    pipeline_layout: PipelineLayout,
    pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    locals: Buffer,
    albedo: Texture,
    albedo_view: TextureView,
    bind_group: BindGroup,
    depth_stencil: Texture,
    depth_stencil_view: TextureView,

    vertex_count: u32,
    index_count: u32,
}

impl Renderer {
    pub async fn new(window: &winit::window::Window) -> Result<Self, Error> {
        let instance = Instance::new(Backends::PRIMARY);

        let surface = unsafe { instance.create_surface(&window) };

        let adapter = match instance.request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(&surface),
        }).await {
            Some(adapter) => adapter,
            None => return Err(Error::NoSuitableGraphicsAdapter),
        };

        let (device, queue) = match adapter.request_device(
            &DeviceDescriptor { label: Some("device"), features: Features::empty(), limits: Limits::default() },
            None,
        ).await {
            Ok(dq) => dq,
            Err(_) => return Err(Error::NoSuitableGraphicsDevice),
        };

        let resolution = window.inner_size();
        let surface_configuration = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: match surface.get_preferred_format(&adapter) {
                Some(format) => format,
                None => return Err(Error::IncompatibleSurface),
            },
            width: resolution.width,
            height: resolution.height,
            present_mode: PresentMode::Mailbox,
        };
        surface.configure(&device, &surface_configuration);

        let shader = device.create_shader_module(&include_wgsl!("shaders/pbr.wgsl"));

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("bind_group_layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: BufferSize::new(std::mem::size_of::<Locals>() as _),
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: TextureSampleType::Float { filterable: false },
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("pipeline"),
            layout: Some(&pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            primitive: PrimitiveState {
                topology: PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: FrontFace::Ccw,
                cull_mode: Some(Face::Back),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: Some(DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Greater,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
            multisample: MultisampleState::default(),
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[ColorTargetState {
                    format: surface_configuration.format,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                }],
            }),
            multiview: None,
        });

        let mut vertex_positions = vec![];
        let mut normals = vec![];
        let mut tex_coords = vec![];
        let mut indices = vec![];

        let (gltf, buffers, _) = gltf::import_slice(include_bytes!("../content/colosseum.glb")).unwrap();
        for mesh in gltf.meshes() {
            for primitive in mesh.primitives() {
                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
                if let Some(iter) = reader.read_positions() {
                    for vertex_position in iter {
                        vertex_positions.push(vertex_position);
                    }
                }

                if let Some(iter) = reader.read_normals() {
                    for normal in iter {
                        normals.push(normal);
                    }
                }

                if let Some(iter) = reader.read_tex_coords(0) {
                    if let ReadTexCoords::F32(iter) = iter {
                        for tex_coord in iter {
                            tex_coords.push(tex_coord);
                        }
                    }
                }

                if let Some(iter) = reader.read_indices() {
                    if let ReadIndices::U16(iter) = iter {
                        for index in iter {
                            indices.push(index);
                        }
                    }
                }
            }
        }

        let mut vertices = vec![];
        for i in 0..vertex_positions.len() {
            vertices.push(Vertex {
                position: vertex_positions[i],
                normal: normals[i],
                tex_coord: tex_coords[i],
            });
        }

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("vertex_buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("index_buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: BufferUsages::INDEX,
        });

        let locals = device.create_buffer(&BufferDescriptor {
            label: Some("locals"),
            size: std::mem::size_of::<Locals>() as _,
            usage: BufferUsages::COPY_DST | BufferUsages::UNIFORM,
            mapped_at_creation: false,
        });

        let albedo_bytes = include_bytes!("../content/albedo.png");
        let albedo_image = image::load_from_memory(albedo_bytes).unwrap();
        let albedo_rgba = albedo_image.to_rgba8();
        let albedo_dimensions = albedo_image.dimensions();

        let albedo = device.create_texture_with_data(
            &queue,
            &TextureDescriptor {
                label: Some("albedo"),
                size: Extent3d {
                    width: albedo_dimensions.0,
                    height: albedo_dimensions.1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8UnormSrgb,
                usage: TextureUsages::TEXTURE_BINDING,
            },
            &albedo_rgba,
        );

        let albedo_view = albedo.create_view(&TextureViewDescriptor::default());

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("bind_group"),
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: locals.as_entire_binding(),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::TextureView(&albedo_view),
                },
            ],
        });

        let (depth_stencil, depth_stencil_view) = create_depth_stencil(&device, resolution);

        Ok(Self {
            instance,
            surface,
            adapter,
            device,
            queue,
            surface_configuration,
            shader,
            bind_group_layout,
            pipeline_layout,
            pipeline,
            vertex_buffer,
            index_buffer,
            locals,
            albedo,
            albedo_view,
            bind_group,
            depth_stencil,
            depth_stencil_view,

            vertex_count: vertices.len() as _,
            index_count: indices.len() as _,
        })
    }

    pub fn resize(&mut self, resolution: PhysicalSize<u32>) {
        if resolution.width == 0 || resolution.height == 0 { return; }

        self.surface_configuration.width = resolution.width;
        self.surface_configuration.height = resolution.height;
        self.surface.configure(&self.device, &self.surface_configuration);

        let (depth_stencil, depth_stencil_view) = create_depth_stencil(&self.device, resolution);
        self.depth_stencil = depth_stencil;
        self.depth_stencil_view = depth_stencil_view;
    }

    pub fn render(&self, view_projection: Matrix4<f32>) -> Result<(), Error> {
        let surface = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(e) => match e {
                SurfaceError::Timeout => {
                    warn!("Timed out while retrieving surface");
                    return Ok(());
                },
                SurfaceError::Outdated => {
                    warn!("Retrieved surface was outdated");
                    return Ok(());
                },
                SurfaceError::Lost => return Err(Error::SurfaceLost),
                SurfaceError::OutOfMemory => return Err(Error::OutOfMemory),
            },
        };

        let surface_view = surface.texture.create_view(&TextureViewDescriptor::default());

        self.queue.write_buffer(&self.locals, 0, bytemuck::bytes_of(&Locals {
            mvp: view_projection.into(),
        }));

        let mut command_encoder = self.device.create_command_encoder(&CommandEncoderDescriptor {
            label: Some("command_encoder"),
        });

        {
            let mut render_pass = command_encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("render_pass"),
                color_attachments: &[RenderPassColorAttachment {
                    view: &surface_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: true,
                    },
                }],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &self.depth_stencil_view,
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(0.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });

            render_pass.set_pipeline(&self.pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);
            render_pass.set_bind_group(0, &self.bind_group, &[]);
            render_pass.draw_indexed(0..self.index_count, 0, 0..1);
        }

        self.queue.submit([command_encoder.finish()]);
        surface.present();
        Ok(())
    }
}

fn create_depth_stencil(device: &Device, resolution: PhysicalSize<u32>) -> (Texture, TextureView) {
    let depth_stencil = device.create_texture(&TextureDescriptor {
        label: Some("depth_stencil"),
        size: Extent3d {
            width: resolution.width,
            height: resolution.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: TextureFormat::Depth32Float,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
    });

    let depth_stencil_view = depth_stencil.create_view(&wgpu::TextureViewDescriptor::default());

    (depth_stencil, depth_stencil_view)
}
