use iced::{
    Application, executor, Command, Subscription, time,
    Element, Container, Text, button, Button, slider, Slider,
    Column, Row, Space, Length,
    Settings,
};

fn main() {
    App::run(Settings::default());
}

pub struct App {
    ui: UI,
    
    
}

#[derive(Default)]
struct UI {

}

#[derive(Debug, Clone)]
pub enum Message {

}

impl Application for App {
    type Executor = executor::Default;
    type Flags = ();
    type Message = Message;
    
    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
    
        (
        Self {
            ui: UI::default(),
            
        },
        Command::none()
        )
    }
    
    fn title(&self) -> String {
        String::from("Config Modules - Iced")
    }
    
    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        Command::none()
    }
    
    fn view(&mut self) -> Element<Self::Message> {
        let content = Text::new("Test App Config");
        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(10)
            .center_x()
            .center_y()
            .into()
    }
}
