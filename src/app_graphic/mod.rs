#![allow(unused_imports)]

use iced::{
    Align, Column, Row, Scrollable, scrollable, Container, Element, Length,
    Text, text_input, TextInput, button, Button, 
    Application, window, Settings, executor, Subscription, Command, time,
};

use crate::graphic;

pub struct GraphicsApp {
    graph: graphic::Graphic,
    log_js: log::json::NewJsonLog,
//     log_value_iter: &dyn Iterator<Item=log::LogValue>,
    log_value_index: usize,
}

#[derive(Debug, Clone)]
pub enum GraphicsAppMessage {
    Graphic(graphic::Message),
    Tick(chrono::DateTime<chrono::Local>)
}

use graphic::Graphic;
impl Application for GraphicsApp {
    type Executor = executor::Default;
    type Flags = ();
    type Message = GraphicsAppMessage;
    
    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        let js = log::json::open_json_file("values_27_08_2020__13_08_30_042.json");
        dbg!(js.values.len());
        let hashs = js.get_all_hash();
        let hashs: Vec<_> = hashs.iter().map(|s| &s[..]).collect();
        (
            Self {
                graph: graphic::Graphic::series(&hashs),
                log_value_index: 0,
                log_js: js,
            },
            Command::none()
        )
    }
    fn title(&self) -> String {
        String::from("GraphicsApp - Iced")
    }
    
    fn subscription(&self) -> Subscription<Self::Message> {
        time::every(std::time::Duration::from_millis(200))
            .map(|_| Self::Message::Tick(chrono::Local::now()))
    }

    
    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        use GraphicsAppMessage::*;
//             use graphic::Message::*;
        match message {
        Tick(_) => {
//             self.graph.update(AppendValues(self.log_js.values[0]));
            for _ in 0..1000 {
                if self.log_value_index+1<self.log_js.values.len() {
                    self.log_value_index += 1;
                    self.graph.append_log_value(self.log_js.values[self.log_value_index].clone());
                }
            }
            #[cfg(feature = "plotters")] {self.graph.update_svg();}
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
