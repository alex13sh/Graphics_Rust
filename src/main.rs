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
        TestInvertor (test_invertor::TestInvertor),
    }
    #[derive(Debug, Clone)]
    pub enum Message {
        InputIpAddressChanged(String),
        Connect,
        
        Invertor(test_invertor::Message),
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
                Message::Connect => *self = Self::TestInvertor ( test_invertor::TestInvertor::new()),
                _ => {}
                },
            Self::TestInvertor (invertor) => match message {
                Message::Invertor(message) => invertor.update(message),
                _ => {}
                }
            };
            Command::none()
        }
        fn view(&mut self) -> Element<Self::Message> {
            let content: Element<_> = match self {
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
                Row::new()
                    .spacing(20)
                    .align_items(Align::Center)
                    .push(input)
                    .push(connect)
                    .into()
            },
            Self::TestInvertor (invertor) => {
                invertor.view().map(Message::Invertor)
            }
            };
            Container::new(content)
                .width(Length::Fill).height(Length::Fill)
                .padding(10)
                .center_x().center_y()
                .into()
        }
    }
    
    mod test_invertor {
        use super::*;
        use modbus::init;
        
        pub struct TestInvertor {
            ui: UI,
            invertor: init::Device,
        }
        
        #[derive(Default)]
        struct UI {
            pb_start: button::State,
            pb_stop: button::State,
        }
        
        #[derive(Debug, Clone)]
        pub enum Message {
            Start,
            Stop,
        }
        
        impl TestInvertor {
            pub fn new() -> Self {
                Self {
                    invertor: init::make_invertor(),
                    ui: Default::default()
                }
            }
            pub fn update(&mut self, message: Message) {
                match message {
                    Message::Start => {},
                    Message::Stop => {},
                }
            }
            pub fn view(&mut self) -> Element<Message> {
                let start = Button::new(&mut self.ui.pb_start, Text::new("Старт"))
                    .on_press(Message::Start);
                let stop = Button::new(&mut self.ui.pb_stop, Text::new("Стоп"))
                    .on_press(Message::Stop);
                Column::new()
                    .spacing(20)
                    .align_items(Align::Center)
                    .push(Text::new(&self.invertor.name))
                    .push(Text::new(format!("Values: {}", self.invertor.values.as_ref().unwrap().len())))
                    .push(start)
                    .push(stop)
                    .into()
            }
        }
    }
}
