pub mod gpu;

use std::sync::Arc;

use iced_winit::winit as winit;
use winit::{application::ApplicationHandler, event::WindowEvent, event_loop::{ActiveEventLoop, EventLoop, ControlFlow}, window::{Window, WindowId}};

use gpu::GpuState;

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app = App::new();

    event_loop.run_app(&mut app).unwrap();
}

struct App<'a> {
    window: Option<Arc<Window>>,
    gpu_state: Option<GpuState<'a>>
}

impl<'a> App<'a> {
    fn new() -> Self {
        App {
            window: None,
            gpu_state: None
        }
    }
}

impl<'a> ApplicationHandler for App<'a> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        println!("oy");
        self.window = Some(Arc::new(event_loop.create_window(Window::default_attributes()).unwrap()));
        self.gpu_state = Some(pollster::block_on(GpuState::init(self.window.as_ref().unwrap().clone())));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        println!("oyyyyy {:?}", event);
        let App { window, gpu_state } = self;
        let window = window.as_ref().unwrap().clone();
        let gpu_state = gpu_state.as_mut().unwrap();
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

                        let clear_color = [1., 1., 1., 1.];

                        let view = frame
                            .texture
                            .create_view(&wgpu::TextureViewDescriptor::default());

                        {
                            let _render_pass = GpuState::clear(&view, &mut encoder, clear_color);
                        }

                        gpu_state.queue.submit(Some(encoder.finish()));
                        frame.present();
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

                self.window.as_ref().unwrap().request_redraw();
            }
            _ => (),
        }
    }
}
