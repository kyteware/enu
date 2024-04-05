use iced::{widget::text, Command, Element, Theme};
use iced_wgpu::Renderer;
use iced_winit::runtime::Program;

pub struct Gui;

#[derive(Debug, Clone)]
pub enum Message {

}

impl Gui {
    pub fn new() -> Gui {
        Gui
    }
}

impl Program for Gui {
    type Theme = Theme;
    type Message = Message;
    type Renderer = Renderer;

    fn update(&mut self, message: Message) -> Command<Message> {
        // match message {
        // }

        Command::none()
    }

    fn view(&self) -> Element<Message, Theme, Renderer> {
        text("bonjour").into()
    }
}