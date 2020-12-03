use iced::{
    Application, executor, Command, Subscription, time,
    Element, Container, Text, 
    Length, 
};

pub struct App {

    ui: UI,
}

#[derive(Default)]
struct UI {

}

#[derive(Debug, Clone)]
pub enum Message {
    ModbusUpdate,
    GraphicUpdate,
    Start, Stop,
    
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
        String::from("GraphicsApp - Iced")
    }
    fn subscription(&self) -> Subscription<Self::Message> {
        time::every(std::time::Duration::from_millis(1000))
            .map(|_| Message::GraphicUpdate)
    }
    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
        Message::GraphicUpdate => {},
        _ => {}
        };
        Command::none()
    }
    fn view(&mut self) -> Element<Self::Message> {
        let content = Text::new("Пустое окно");
        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(10)
            .center_x()
            .center_y()
            .into()
    }
}
