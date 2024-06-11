pub mod gpu;
pub mod gui;
pub mod playback;

use std::sync::{Arc, Mutex};

use gui::GuiRuntime;
use iced::{Point, Rectangle, Size};
use iced_wgpu::graphics::Viewport;
use iced_winit::{conversion::mouse_interaction, winit::{self as winit, dpi::PhysicalSize}};
use playback::Playback;
use wgpu::{CompositeAlphaMode, PresentMode, SurfaceConfiguration, TextureUsages};
use winit::{application::ApplicationHandler, event::WindowEvent, event_loop::{ActiveEventLoop, EventLoop, ControlFlow}, window::{Window, WindowId}};

use gpu::GpuState;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::new();

    event_loop.run_app(&mut app).unwrap();
}

enum App<'a> {
    Running(AppRuntime<'a>),
    Paused
}

impl<'a> App<'a> {
    fn new() -> Self {
        App::Paused
    }

    fn unwrap_running(&mut self) -> &mut AppRuntime<'a> {
        if let App::Running(runtime) = self {
            runtime
        } else {
            panic!("couldn't unwrap running app")
        }
    }
}

struct AppRuntime<'a> {
    window: Arc<Window>,
    gpu_state: GpuState<'a>,
    gui_runtime: GuiRuntime,
    playback: Playback
}

impl<'a> AppRuntime<'a> {
    fn init(event_loop: &ActiveEventLoop) -> AppRuntime<'a> {
        let viewport_arc = Arc::new(Mutex::new(Rectangle::default()));
        let window = Arc::new(event_loop.create_window(Window::default_attributes().with_min_inner_size(PhysicalSize::new(600, 400))).unwrap());
        let gpu_state = pollster::block_on(GpuState::init(window.clone()));
        let gui_runtime = GuiRuntime::init(window.clone(), &gpu_state, viewport_arc.clone());
        let playback = Playback::init(&gpu_state, viewport_arc.clone());
        AppRuntime { window, gpu_state, gui_runtime, playback }
    }
}

impl<'a> ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        *self = App::Running(AppRuntime::init(event_loop))
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let AppRuntime { window, gpu_state, gui_runtime, playback } = self.unwrap_running();

        gui_runtime.process_event(event.clone(), window.clone());
        let playback_instructions = &gui_runtime.state.program().playback_instructions;
        for instruction in playback_instructions {
            playback.process_instruction(instruction)
        }

        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            },
            WindowEvent::RedrawRequested => {
                match gpu_state.surface.get_current_texture() {
                    Ok(frame) => {
                        let mut encoder = gpu_state.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: None,
                        });

                        let clear_color = [0.5, 0.5, 0.5, 1.];

                        let view = frame
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default());

                        {
                            let mut render_pass = GpuState::begin_render_pass(&view, &mut encoder, clear_color);
                            playback.draw(&mut render_pass);
                        }

                        gui_runtime.renderer.present(
                            &mut gui_runtime.engine,
                            &gpu_state.device,
                            &gpu_state.queue,
                            &mut encoder,
                            None,
                            frame.texture.format(),
                            &view,
                            &gui_runtime.viewport,
                            &gui_runtime.debug.overlay()
                        );

                        gui_runtime.engine.submit(&gpu_state.queue, encoder); // replaces encoder.submit
                        frame.present();

                        window.set_cursor(mouse_interaction(gui_runtime.state.mouse_interaction()))
                    }
                    Err(error) => match error {
                        wgpu::SurfaceError::OutOfMemory => {
                            panic!(
                                "Swapchain error: {error}. \
                            Rendering cannot continue."
                            )
                        }
                        _ => {
                            // Try rendering again next frame.
                            window.request_redraw();
                        }
                    },
                }

                window.request_redraw();
            }
            WindowEvent::CursorMoved { position, .. } => {
                gui_runtime.cursor_position = Some(position)
            }
            WindowEvent::ModifiersChanged(new_modifiers) => {
                gui_runtime.modifiers = new_modifiers.state();
            }
            WindowEvent::Resized(new_size) => {
                gui_runtime.viewport = Viewport::with_physical_size(Size::new(new_size.width, new_size.height), window.scale_factor());

                let new_surface_config = SurfaceConfiguration {
                    usage: TextureUsages::RENDER_ATTACHMENT, 
                    format: gpu_state.surface_config.format, 
                    width: new_size.width, 
                    height: new_size.height, 
                    present_mode: PresentMode::AutoVsync,
                    alpha_mode: CompositeAlphaMode::Auto,
                    view_formats: vec![],
                    desired_maximum_frame_latency: 2
                };

                gpu_state.surface.configure(
                    &gpu_state.device,
                    &new_surface_config
                );

                gpu_state.surface_config = new_surface_config;
            }
            _ => (),
        }
    }
}
