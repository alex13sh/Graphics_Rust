use iced::{
    Application, executor, Command, Subscription, time,
    Element, Container, Text, button, Button, slider, Slider, scrollable, Scrollable,
    Column, Row, Space, Length,
    Settings,
};

fn main() {
    App::run(Settings::default());
}

use modbus::{Value, ModbusValues, ValueError};

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::Arc;

pub struct App {
    ui: UI,
    
    logic: meln_logic::init::Complect,
    values: BTreeMap<String, Arc<Value>>,
}

#[derive(Default)]
struct UI {
    scroll: scrollable::State,
}

#[derive(Debug, Clone)]
pub enum Message {

}

impl Application for App {
    type Executor = executor::Default;
    type Flags = ();
    type Message = Message;
    
    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        let mut logic = meln_logic::init::Complect::new();
        let values = logic.make_values(false);
        logic.init_values(&values);
        
        (
        Self {
            ui: UI::default(),
            
            logic: logic,
            values: values,
            
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
        let values = self.values.keys().fold(String::from(""), |txt, name| txt+"\n"+name);
        let content = Text::new(values);
        let content = Container::new(content)
            .width(Length::Fill)
//             .height(Length::Fill)
            .padding(10)
            .center_x();
//             .center_y();
        
        Scrollable::new(&mut self.ui.scroll)
            .padding(10)
             .push(content)
            .into()
    }
}
