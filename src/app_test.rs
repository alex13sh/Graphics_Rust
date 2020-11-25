use iced::{
    Align, Column, Row, Scrollable, scrollable, Container, Element, Length,
    Text, text_input, TextInput, button, Button, 
    Application, window, Settings, executor, Subscription, Command, time,
};

pub struct TestUI {
    ui: UI,
}

#[derive(Default)]
struct UI {

}

#[derive(Debug, Clone)]
pub enum Message {

}

impl Application for TestUI {
    type Executor = executor::Default;
    type Flags = ();
    type Message = Message;
    
    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        (
        TestUI {
            ui: UI::default(),
            
        },
        Command::none()
        )
    }
    
    fn title(&self) -> String {
        "Test UI".into()
    }
    
    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
        
        }
    }
    
    fn view(&mut self) -> Element<Self::Message> {
        Column::new().into()
    }
}
