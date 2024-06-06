use std::sync::Arc;

use iced::{widget::text, Command, Font, Pixels, Size, Theme};
use iced_wgpu::{graphics::Viewport, Engine, Renderer};
use iced_winit::{runtime::{program, Debug, Program}, winit::{dpi::PhysicalPosition, keyboard::ModifiersState, window::Window}, Clipboard};

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

    fn update(&mut self, _message: Self::Message) -> Command<Self::Message> {
        Command::none()
    }

    fn view(&self) -> iced_wgpu::core::Element<'_, Self::Message, Self::Theme, Self::Renderer> {
        text(&self.text).into()
    }
}

#[derive(Clone, Debug)]
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
}