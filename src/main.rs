use enu::{playback::{Playback, PlaybackViewport}, runtime_helpers::{init_gui, init_wgpu_winit}};
use iced::{mouse, Color, Size, Theme};
use iced_wgpu::{core::renderer, graphics::Viewport};
use iced_winit::{conversion};
use winit::{event::{Event, WindowEvent}};

#[pollster::main]
async fn main() {
    let (event_loop, window, mut physical_size, surface, device, queue, config) = init_wgpu_winit().await;
    let (mut viewport, mut cursor_pos, mut modifiers, mut clipboard, mut debug, mut renderer, mut state) = init_gui(physical_size, window.clone(), &device, &queue, config.format);
    let playback = Playback::new(&device, &queue, config.format);

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
                                clear_color,
                            );

                            playback.draw(&mut render_pass, &PlaybackViewport { x: physical_size.width as f32 / 2., y: 0., w: physical_size.width as f32 / 2., h: physical_size.height as f32 / 2. });
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
                        cursor_pos = Some(position);
                    }
                    WindowEvent::ModifiersChanged(new_modifiers) => {
                        modifiers = new_modifiers.state();
                    }
                    WindowEvent::Resized(_) => {
                        physical_size = window.inner_size();

                        viewport = Viewport::with_physical_size(
                            Size::new(physical_size.width, physical_size.height),
                            window.scale_factor(),
                        );

                        surface.configure(
                            &device,
                            &wgpu::SurfaceConfiguration {
                                format: config.format,
                                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                                width: physical_size.width,
                                height: physical_size.height,
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
                cursor_pos
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