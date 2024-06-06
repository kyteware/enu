use std::sync::Arc;

use iced::{mouse::Cursor, widget::button, window::Id, Color, Command, Font, Pixels, Size, Theme};
use iced_wgpu::{core::renderer::Style, graphics::Viewport, Engine, Renderer};
use iced_winit::{conversion::{cursor_position, window_event}, runtime::{program, Debug, Program}, winit::{dpi::PhysicalPosition, event::WindowEvent, keyboard::ModifiersState, window::Window}, Clipboard};

use crate::gpu::GpuState;

pub struct Gui {
    text: String
}

impl Gui {
    pub fn new() -> Gui {
        Gui {
            text: "bobb".into()
        }
    }
}

impl Program for Gui {
    type Renderer = Renderer;
    type Theme = Theme;
    type Message = Message;

    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        if message == Message::Oy {
            println!("hi")
        }
        Command::none()
    }

    fn view(&self) -> iced_wgpu::core::Element<'_, Self::Message, Self::Theme, Self::Renderer> {
        button(self.text.as_str()).on_press(Message::Oy).into()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Message {
    Oy
}

pub struct GuiRuntime {
    pub engine: Engine,
    pub renderer: Renderer,
    pub state: program::State<Gui>,
    pub cursor_position: Option<PhysicalPosition<f64>>,
    pub clipboard: Clipboard,
    pub viewport: Viewport,
    pub modifiers: ModifiersState,
    pub debug: Debug
}

impl GuiRuntime {
    pub fn init(window: Arc<Window>, GpuState { adapter, device, queue, surface_config, .. }: &GpuState) -> GuiRuntime {
        let engine = Engine::new(&adapter, &device, &queue, surface_config.format, None); // TODO: antialiasing?
        let mut renderer = Renderer::new(&device, &engine, Font::default(), Pixels::from(16));
        let physical_size = window.inner_size();
        let viewport = Viewport::with_physical_size(
            Size::new(physical_size.width, physical_size.height),
            window.scale_factor(),
        );
        let mut debug = Debug::new();
        let gui = Gui::new();
        let state = program::State::new(
            gui,
            viewport.logical_size(),
            &mut renderer,
            &mut debug
        );
        let clipboard = Clipboard::connect(&window);
        let modifiers = ModifiersState::default();
        let cursor_position = None;
        GuiRuntime {
            engine,
            renderer,
            state,
            cursor_position,
            clipboard,
            viewport,
            modifiers,
            debug
        }
    }

    pub fn process_event(&mut self, event: WindowEvent, window: Arc<Window>) {
        if let Some(event) = window_event(
            Id::MAIN,
            event,
            window.scale_factor(),
            self.modifiers
        ) {
            self.state.queue_event(event)
        }

        if !self.state.is_queue_empty() {
            let _ = self.state.update(
                self.viewport.logical_size(),
                self.cursor_position
                    .map(|p| {
                        cursor_position(
                            p,
                            self.viewport.scale_factor(),
                        )
                    })
                    .map(Cursor::Available)
                    .unwrap_or(Cursor::Unavailable),
                    &mut self.renderer,
                &Theme::Dark,
                &Style {
                    text_color: Color::WHITE,
                },
                &mut self.clipboard,
                &mut self.debug,
            );

            window.request_redraw();
        }
    }
}