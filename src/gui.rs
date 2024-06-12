mod placeholder;

use std::sync::{Arc, Mutex};

use iced::{mouse::Cursor, widget::{button, column}, window::Id, Color, Command, Element, Font, Pixels, Rectangle, Size, Theme};
use iced_wgpu::{core::renderer::Style, graphics::Viewport, Engine, Renderer};
use iced_winit::{conversion::{cursor_position, window_event}, runtime::{program, Debug, Program}, winit::{dpi::PhysicalPosition, event::WindowEvent, keyboard::ModifiersState, window::Window}, Clipboard};
use placeholder::PlaybackTracker;

use crate::{gpu::GpuState, playback::PlaybackInstruction};

pub struct Gui {
    /// The instructions for the playback
    pub playback_instructions: Vec<PlaybackInstruction>,
    text: String,
    viewport_arc: Arc<Mutex<Rectangle<f32>>>
}

impl Gui {
    pub fn new(viewport_arc: Arc<Mutex<Rectangle<f32>>>) -> Gui {
        Gui {
            text: "bobb".into(),
            playback_instructions: vec![],
            viewport_arc
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
        column!(Element::new(PlaybackTracker::new(self.viewport_arc.clone())), button("hi")).into()
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
    pub fn init(window: Arc<Window>, GpuState { adapter, device, queue, surface_config, .. }: &GpuState, viewport_arc: Arc<Mutex<Rectangle<f32>>>) -> GuiRuntime {
        let engine = Engine::new(&adapter, &device, &queue, surface_config.format, None); // TODO: antialiasing?
        let mut renderer = Renderer::new(&device, &engine, Font::default(), Pixels::from(16));
        let physical_size = window.inner_size();
        let viewport = Viewport::with_physical_size(
            Size::new(physical_size.width, physical_size.height),
            window.scale_factor(),
        );
        let mut debug = Debug::new();
        let gui = Gui::new(viewport_arc);
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