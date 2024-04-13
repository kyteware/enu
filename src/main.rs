use std::sync::Arc;
use enu::{gui::Gui, playback::Playback};
use iced::{mouse, Color, Font, Pixels, Size, Theme};
use iced_wgpu::{core::renderer, graphics::Viewport, Backend, Renderer, Settings};
use iced_winit::{conversion, runtime::{program, Debug}, Clipboard};
use winit::{event::{Event, WindowEvent}, event_loop::EventLoop, keyboard::ModifiersState, window::WindowBuilder};

#[pollster::main]
async fn main() {
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(WindowBuilder::new().with_title("Enu").build(&event_loop).unwrap());
    let mut physical_size = window.inner_size();
    let mut viewport = Viewport::with_physical_size(
        Size::new(physical_size.width, physical_size.height),
        window.scale_factor(),
    );
    let mut cursor_position = None;
    let mut modifiers = ModifiersState::default();
    let mut clipboard = Clipboard::connect(&window);

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

    // CREATE GUI, LOADER AND VIEWPORT HERE
    let gui = Gui::new();
    let playback = Playback::new(&device, surface_format);

    let mut debug = Debug::new();
    let mut renderer = Renderer::new(
        Backend::new(&device, &queue, Settings::default(), surface_format),
        Font::default(),
        Pixels(16.0),
    );
    let mut state = program::State::new(
        gui,
        viewport.logical_size(),
        &mut renderer,
        &mut debug,
    );

    event_loop.run(move |event, window_target| {
        window_target.set_control_flow(winit::event_loop::ControlFlow::Wait);
        match event {
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                ..
            } => {
                match surface.get_current_texture() {
                    Ok(frame) => {
                        let mut encoder = device.create_command_encoder(
                            &wgpu::CommandEncoderDescriptor { label: None },
                        );

                        let clear_color = state.program().background_color();

                        let view = frame.texture.create_view(
                            &wgpu::TextureViewDescriptor::default(),
                        );

                        {
                            let mut render_pass = Playback::clear(
                                &view,
                                &mut encoder,
                                Color::new(0.2, 0.2, 0.2, 1.),
                            );

                            playback.draw(&mut render_pass);
                        }

                        renderer.with_primitives(|backend, primitive| {
                            backend.present(
                                &device,
                                &queue,
                                &mut encoder,
                                None,
                                frame.texture.format(),
                                &view,
                                primitive,
                                &viewport,
                                &debug.overlay(),
                            );
                        });

                        queue.write_buffer(&playback.alpha_buf, 0, bytemuck::cast_slice(&[state.program().num]));
                        queue.submit(Some(encoder.finish()));
                        frame.present();

                        window.set_cursor_icon(
                            iced_winit::conversion::mouse_interaction(
                                state.mouse_interaction(),
                            ),
                        );
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
            }
            Event::WindowEvent { event, .. } => {
                match event {
                    WindowEvent::CursorMoved { position, .. } => {
                        cursor_position = Some(position);
                    }
                    WindowEvent::ModifiersChanged(new_modifiers) => {
                        modifiers = new_modifiers.state();
                    }
                    WindowEvent::Resized(_) => {
                        let size = window.inner_size();

                        viewport = Viewport::with_physical_size(
                            Size::new(size.width, size.height),
                            window.scale_factor(),
                        );

                        surface.configure(
                            &device,
                            &wgpu::SurfaceConfiguration {
                                format: surface_format,
                                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                                width: size.width,
                                height: size.height,
                                present_mode: wgpu::PresentMode::AutoVsync,
                                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                                view_formats: vec![],
                                desired_maximum_frame_latency: 2,
                            },
                        );
                    }
                    WindowEvent::CloseRequested => {
                        window_target.exit();
                    }
                    _ => {}
                }

                // Map window event to iced event
                if let Some(event) = iced_winit::conversion::window_event(
                    iced::window::Id::MAIN,
                    event,
                    window.scale_factor(),
                    modifiers,
                ) {
                    state.queue_event(event);
                }
            }
            _ => {}
        }

        // If there are events pending
        if !state.is_queue_empty() {
            // We update iced
            let _ = state.update(
                viewport.logical_size(),
                cursor_position
                    .map(|p| {
                        conversion::cursor_position(p, viewport.scale_factor())
                    })
                    .map(mouse::Cursor::Available)
                    .unwrap_or(mouse::Cursor::Unavailable),
                &mut renderer,
                &Theme::Dark,
                &renderer::Style {
                    text_color: Color::WHITE,
                },
                &mut clipboard,
                &mut debug,
            );

            // and request a redraw
            window.request_redraw();
        }
    }).unwrap();
}