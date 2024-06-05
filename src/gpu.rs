use std::sync::Arc;

use iced_winit::winit as winit;
use wgpu::{Adapter, Color, CommandEncoder, Device, DeviceDescriptor, Features, Instance, InstanceDescriptor, Limits, LoadOp, Operations, PowerPreference, Queue, RenderPass, RenderPassColorAttachment, RenderPassDescriptor, RequestAdapterOptions, StoreOp, Surface, SurfaceConfiguration, TextureUsages, TextureView};
use winit::window::Window;

pub struct GpuState<'window> {
    pub adapter: Adapter,
    pub device: Device,
    pub queue: Queue,
    pub surface: Surface<'window>,
    pub surface_config: SurfaceConfiguration
}

impl<'window> GpuState<'window> {
    pub async fn init(window: Arc<Window>) -> Self {
        let physical_size = window.inner_size();

        let instance = Instance::new(InstanceDescriptor::default());
        let surface = instance.create_surface(window.clone()).unwrap();
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::default(),
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &DeviceDescriptor {
                    required_features: Features::empty(),
                    required_limits: Limits::default(),
                    label: None,
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let surface_config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: physical_size.width,
            height: physical_size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);

        GpuState {
            adapter,
            device,
            queue,
            surface,
            surface_config
        }                                                                
    }

    pub fn clear<'a>(
        target: &'a TextureView,
        encoder: &'a mut CommandEncoder,
        background_color: [f32; 4],
    ) -> RenderPass<'a> {
        encoder.begin_render_pass(&RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                ops: Operations {
                    load: LoadOp::Clear({
                        let [r, g, b, a] = background_color;

                        Color {
                            r: r as f64,
                            g: g as f64,
                            b: b as f64,
                            a: a as f64,
                        }
                    }),
                    store: StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        })
    }
}