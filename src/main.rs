use iced::{
    Align, Column, Row, Container, Element, Length,
    Text, text_input, TextInput, button, Button, 
    Application, window, Settings, executor, Subscription, Command, time,
};

mod graphic;

fn main() {
    println!("Hello World");
//     app_graphic::GraphicsApp::run(Settings::default());
    app_test::TestApp::run(Settings { 
        window: window::Settings {
            size: (500, 400), //size: (1200, 800),
            resizable: true,
            .. Default::default()
        },
        flags: (),
        .. Settings::default()
    });
}

mod app_graphic {
    use super::*;
    pub struct GraphicsApp {
        graph: graphic::Graphic,
        log_js: log::NewJsonLog,
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
            let js = log::open_json_file("values_25_08_2020__13_41_06_111.json");
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
            time::every(std::time::Duration::from_millis(40))
                .map(|_| Self::Message::Tick(chrono::Local::now()))
        }

        
        fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
            use GraphicsAppMessage::*;
//             use graphic::Message::*;
            match message {
            Tick(_) => {
    //             self.graph.update(AppendValues(self.log_js.values[0]));
                for _ in 0..40 {
                    if self.log_value_index+1<self.log_js.values.len() {
                        self.log_value_index += 1;
                        self.graph.append_value(self.log_js.values[self.log_value_index].clone());
                    }
                }
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

}

mod app_test {
    use super::*;
    pub enum TestApp {
        Connect {
            input_ip_address: text_input::State,
            ip_address: String,
            pb_connect: button::State,
        },
    }
    #[derive(Debug, Clone)]
    pub enum Message {
        InputIpAddressChanged(String),
        Connect,
    }
    
    impl Application for TestApp {
        type Executor = executor::Default;
        type Flags = ();
        type Message = Message;
    
        fn new(_flags: ()) -> (Self, Command<Self::Message>) {
            (
                Self::Connect {
                    input_ip_address: text_input::State::new(),
                    ip_address: "192.168.1.5".into(),
                    pb_connect: button::State::new(),
                },
                Command::none()
            )
        }
        fn title(&self) -> String {
            String::from("TestApp - Iced")
        }
        
        fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
            match self {
            Self::Connect {ip_address, ..} => match message {
                Message::InputIpAddressChanged(txt) => *ip_address = txt,
                Message::Connect => *ip_address = "Готово!".into(),
                },
            };
            Command::none()
        }
        fn view(&mut self) -> Element<Self::Message> {
            match self {
            Self::Connect {ip_address, input_ip_address, pb_connect} => {
                let input = TextInput::new(
                    input_ip_address,
                    "Введите IP адрес",
                    ip_address,
                    Message::InputIpAddressChanged,
                ).padding(10)
                .on_submit(Message::Connect);
                
                let connect = Button::new(pb_connect, Text::new("Подключиться"))
                    .on_press(Message::Connect);
                    
//                 let text = Text::new("My Text");
                let row = Row::new()
                    .spacing(20)
                    .align_items(Align::Center)
                    .push(input)
                    .push(connect);
                Container::new(row)
                    .width(Length::Fill).height(Length::Fill)
                    .padding(10)
                    .center_x().center_y()
                    .into()
            },
            }
        }
    }
}
