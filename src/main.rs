use iced::{
    slider, Align, Column, Container, Element, Length, /*Application,*/ Sandbox, Settings,
    Slider, Text,
};

fn main() {
    println!("Hello World");
    GraphicsApp::run(Settings::default());
}

struct GraphicsApp {

}

#[derive(Debug, Clone, Copy)]
enum GraphicsAppMessage {

}

impl Sandbox for GraphicsApp {
    type Message = GraphicsAppMessage;
    
    fn new() -> Self {
        Self {
            
        }
    }
    fn title(&self) -> String {
        String::from("GraphicsApp - Iced")
    }
    fn update(&mut self, _message: Self::Message) {
        
    }
    fn view(&mut self) -> Element<Self::Message> {
        Column::new().into()
    }
}
