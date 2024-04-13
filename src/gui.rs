use iced::{widget::{button, column, row, slider, text}, Color, Command, Element, Length, Theme};
use iced_wgpu::Renderer;
use iced_winit::runtime::Program;

pub struct Gui {
    pub num: f32
}

#[derive(Debug, Clone)]
pub enum Message {
    UpdateNum(f32)
}

impl Gui {
    pub fn new() -> Gui {
        Gui { num: 1. }
    }

    pub fn background_color(&self) -> Color {
        Color::new(0.2, 0.2, 0.2, 1.0)
    }
}

impl Program for Gui {
    type Theme = Theme;
    type Message = Message;
    type Renderer = Renderer;

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::UpdateNum(num) => {
                self.num = num
            }
        }

        Command::none()
    }

    fn view(&self) -> Element<Message, Theme, Renderer> {
        let sources = column!(button("source 1"), button("source 2"), slider(0.0..=1.0, self.num, Message::UpdateNum).step(0.1).width(20));
        let timeline = column!(button("timeline 1 --------------------------"), button("timeline 2 -----------------"));
        let playback = text("playback here");

        column!(
            row!(sources.width(Length::FillPortion(1)), playback).height(Length::FillPortion(1)),
            timeline.height(Length::FillPortion(1))
        ).into()
    }
}