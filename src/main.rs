use iced::{
    slider, Align, Column, Container, Element, Length,
    Slider, Text, Canvas,
    Application, Settings, executor, Subscription, Command, time,
};

mod graphic;
use graphic::Graphic;

fn main() {
    println!("Hello World");
    GraphicsApp::run(Settings::default());
}

struct GraphicsApp {
    graph_state: graphic::State,
}

#[derive(Debug, Clone)]
enum GraphicsAppMessage {
    Graphic(graphic::Message),
    Tick(chrono::DateTime<chrono::Local>)
}

impl Application for GraphicsApp {
    type Executor = executor::Default;
    type Flags = ();
    type Message = GraphicsAppMessage;
    
    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        (
        Self {
            graph_state: graphic::State::default()
        },
        Command::none()
        )
    }
    fn title(&self) -> String {
        String::from("GraphicsApp - Iced")
    }
    
    fn subscription(&self) -> Subscription<Self::Message> {
        time::every(std::time::Duration::from_millis(500))
            .map(|_| Self::Message::Tick(chrono::Local::now()))
    }

    
    fn update(&mut self, _message: Self::Message) -> Command<Self::Message> {
        Command::none()
    }
    fn view(&mut self) -> Element<Self::Message> {
        let canvas = Canvas::new(Graphic::new(&mut self.graph_state))
            .width(Length::Units(400))
            .height(Length::Units(400));
//         canvas.into()
        let canvas = Element::from(canvas)
        .map(GraphicsAppMessage::Graphic);
        Container::new(canvas)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .center_x()
            .center_y()
            .into()
    }
}
