use std::sync::Arc;

use iced::{Font, Pixels, Size};
use iced_wgpu::{graphics::Viewport, Backend, Renderer, Settings};
use iced_winit::{
    runtime::{program::State, Debug},
    Clipboard,
};
use wgpu::{Device, Queue, Surface, SurfaceConfiguration, TextureFormat};
use winit::{
    dpi::{PhysicalPosition, PhysicalSize},
    event_loop::EventLoop,
    keyboard::ModifiersState,
    window::{Window, WindowBuilder},
};

use crate::gui::Gui;

pub async fn init_wgpu_winit<'a>() -> (
    EventLoop<()>,
    Arc<Window>,
    PhysicalSize<u32>,
    Surface<'a>,
    Device,
    Queue,
    SurfaceConfiguration,
) {
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Enu")
            .build(&event_loop)
            .unwrap(),
    );
    let physical_size = window.inner_size();

    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
    let surface = instance.create_surface(window.clone()).unwrap();
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::default(),
            compatible_surface: None,
            force_fallback_adapter: false,
        })
        .await
        .unwrap();
    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
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
    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format: surface_format,
        width: physical_size.width,
        height: physical_size.height,
        present_mode: surface_caps.present_modes[0],
        alpha_mode: surface_caps.alpha_modes[0],
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };
    surface.configure(&device, &config);

    (
        event_loop,
        window,
        physical_size,
        surface,
        device,
        queue,
        config,
    )
}

pub fn init_gui(
    physical_size: PhysicalSize<u32>,
    window: Arc<Window>,
    device: &Device,
    queue: &Queue,
    surface_format: TextureFormat,
) -> (
    Viewport,
    Option<PhysicalPosition<f64>>,
    ModifiersState,
    Clipboard,
    Debug,
    Renderer,
    State<Gui>,
) {
    let viewport = Viewport::with_physical_size(
        Size::new(physical_size.width, physical_size.height),
        window.scale_factor(),
    );
    let cursor_position: Option<PhysicalPosition<f64>> = None;
    let modifiers = ModifiersState::default();
    let clipboard = Clipboard::connect(&window);

    // CREATE GUI, LOADER AND VIEWPORT HERE
    let gui = Gui::new();

    let mut debug = Debug::new();
    let mut renderer = Renderer::new(
        Backend::new(&device, &queue, Settings::default(), surface_format),
        Font::default(),
        Pixels(16.0),
    );

    let state = State::new(gui, viewport.logical_size(), &mut renderer, &mut debug);

    (
        viewport,
        cursor_position,
        modifiers,
        clipboard,
        debug,
        renderer,
        state,
    )
}
