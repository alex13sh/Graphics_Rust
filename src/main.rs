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
    graph: graphic::Graphic,
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
            graph: graphic::Graphic::series(&["ser_1", "ser_2"])
        },
        Command::none()
        )
    }
    fn title(&self) -> String {
        String::from("GraphicsApp - Iced")
    }
    
    fn subscription(&self) -> Subscription<Self::Message> {
        time::every(std::time::Duration::from_millis(10))
            .map(|_| Self::Message::Tick(chrono::Local::now()))
    }

    
    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        use GraphicsAppMessage::*;
        use graphic::Message::*;
        match message {
        Tick(_) => {
            self.graph.update(AppendValues(vec![1_f32, 2_f32]));
        },
        _ => {}
        };
        Command::none()
    }
    fn view(&mut self) -> Element<Self::Message> {
//         let canvas = Canvas::new(Graphic::new(&mut self.graph_state))
//             .width(Length::Units(1000))
//             .height(Length::Units(1000));
//         canvas.into()
        let canvas = self.graph.view()
            .map(GraphicsAppMessage::Graphic);
        Container::new(canvas)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(10)
            .center_x()
            .center_y()
            .into()
    }
}
