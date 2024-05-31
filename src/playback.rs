use ffmpeg::device::input::video;
use iced::Color;
use wgpu::{util::DeviceExt, BindGroupLayoutDescriptor, BindingResource, BufferAddress, Queue, Texture};
use winit::dpi::PhysicalSize;

pub struct Playback {
    pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    bg: wgpu::BindGroup,
    video_dimensions: PhysicalSize<u32>,
    video_texture: Texture
}

pub struct PlaybackViewport {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32
}

impl Playback {
    pub fn new(
        device: &wgpu::Device,
        queue: &Queue,
        texture_format: wgpu::TextureFormat
    ) -> Playback {
        let (vs_module, fs_module) = (
            device.create_shader_module(wgpu::include_wgsl!("shader/vert.wgsl")),
            device.create_shader_module(wgpu::include_wgsl!("shader/frag.wgsl")),
        );

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertexes"),
            usage: wgpu::BufferUsages::VERTEX,
            contents: bytemuck::cast_slice(VERTICES)
        });

        let video_dimensions = PhysicalSize::new(1920, 1080);

        let video_texture_size = wgpu::Extent3d {
            width: video_dimensions.width,
            height: video_dimensions.height,
            depth_or_array_layers: 1,
        };

        let video_texture = device.create_texture(
            &wgpu::TextureDescriptor {
                size: video_texture_size,
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                label: Some("diffuse_texture"),
                view_formats: &[],
            }
        );

        let img = image::open("img.png").unwrap();
        let image_rgba = img.to_rgba8();

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &video_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &image_rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * video_dimensions.width),
                rows_per_image: Some(video_dimensions.height),
            },
            video_texture_size,
        );

        let video_texture_view = video_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let video_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bgl = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bind group"),
            layout: &bgl,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&video_texture_view)
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&video_sampler)
                }
            ]
        });
    
        let pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                push_constant_ranges: &[],
                bind_group_layouts: &[&bgl],
            });
    
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs_module,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs_module,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: texture_format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Playback { pipeline, vertex_buffer, bg, video_dimensions, video_texture }
    }

    pub fn clear<'a>(
        target: &'a wgpu::TextureView,
        encoder: &'a mut wgpu::CommandEncoder,
        background_color: Color,
    ) -> wgpu::RenderPass<'a> {
        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear({
                        let [r, g, b, a] = background_color.into_linear();
                        
                        wgpu::Color {
                            r: r as f64,
                            g: g as f64,
                            b: b as f64,
                            a: a as f64,
                        }
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        })
    }

    pub fn draw<'a>(&'a self, render_pass: &mut wgpu::RenderPass<'a>, vp: &PlaybackViewport) {
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, &self.bg, &[]);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_viewport(vp.x, vp.y, vp.w, vp.h, 0., 1.);
        render_pass.draw(0..6, 0..1);
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    pos: [f32; 2],
    tex: [f32; 2]
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: mem::size_of::<[f32; 2]>() as BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x2,
                }
            ]
        }
    }
}

const VERTICES: &[Vertex] = &[
    Vertex { pos: [-1., 1.], tex: [0., 0.] },
    Vertex { pos: [1., 1.], tex: [1., 0.] },
    Vertex { pos: [1., -1.], tex: [1., 1.] },

    Vertex { pos: [1., -1.], tex: [1., 1.] },
    Vertex { pos: [-1., 1.], tex: [0., 0.] },
    Vertex { pos: [-1., -1.], tex: [0., 1.] }
];
