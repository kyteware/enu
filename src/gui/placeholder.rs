use std::sync::{Arc, Mutex};

use iced::mouse;
use iced::{Color, Length, Rectangle, Size};
use iced_wgpu::core::layout::{atomic, Limits, Node};
use iced_wgpu::core::widget::Tree;
use iced_wgpu::core::{renderer, Layout, Widget};

pub struct PlaybackTracker {
    viewport_arc: Arc<Mutex<Rectangle<f32>>>
}

impl PlaybackTracker {
    pub fn new(viewport_arc: Arc<Mutex<Rectangle<f32>>>) -> Self {
        Self {
            viewport_arc
        }
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for PlaybackTracker
where
    Renderer: renderer::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Fill,
            height: Length::Fill,
        }
    }

    fn layout(
        &self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &Limits,
    ) -> Node {
        atomic(limits, Length::Fill, Length::Fill)
    }

    fn draw(
        &self,
        _state: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let mut viewport = self.viewport_arc.lock().unwrap();
        *viewport = layout.bounds();
        renderer.fill_quad(
            renderer::Quad {
                bounds: layout.bounds(),
                ..renderer::Quad::default()
            },
            Color::TRANSPARENT,
        );
    }
}
