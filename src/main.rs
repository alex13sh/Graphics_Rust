#![allow(dead_code)]

use iced::{
    Align, Column, Row, Scrollable, scrollable, Container, Element, Length,
    Text, text_input, TextInput, button, Button, 
    Application, window, Settings, executor, Subscription, Command, time,
};

mod graphic;

fn main() {
    println!("Hello World");
//     app_graphic::GraphicsApp::run(Settings::default());
    app_test_device::TestDeviceApp::run(Settings { 
        window: window::Settings {
            size: (600, 500), //size: (1200, 800),
            resizable: true,
            .. Default::default()
        },
        flags: (),
        .. Settings::default()
    });
}

mod app_graphic {
    use super::*;
    use modbus::init;
    use modbus::Device;
    use modbus::{Value, ModbusValues};
    use modbus::{Sensor, ModbusSensors};
    
    pub struct GraphicsApp {
        graph: graphic::Graphic,
        log_js: log::NewJsonLog,
    //     log_value_iter: &dyn Iterator<Item=log::LogValue>,
        log_value_index: usize,
        sensors: ModbusSensors,
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
            let devices = modbus::init::init_devices();
            let devices: Vec<_> = devices.iter().map(Device::From).collect();
            let sensors = devices.iter().fold(mut Vec::new(), |mut sens, d| {}).collect();
            
            // Загрузка логов
            let js = log::open_json_file("values_25_08_2020__13_41_06_111.json");
            dbg!(js.values.len());
            let hashs = js.get_all_hash();
            let hashs: Vec<_> = hashs.iter().map(|s| &s[..]).collect();
            
            (
                Self {
                    graph: graphic::Graphic::series(&hashs),
                    log_value_index: 0,
                    log_js: js,
                    sensors: sensors,
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

    mod sensor_list {
        
        struct Sensor {
        
        }
        
        #[derive(Debug, Clone)]
        pub enum Message {
            
        }
        
        impl Sensor {
            pub fn new() -> Self {
            
            }
        }
    }
}

mod app_test {
    use super::*;
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
}

mod app_test_device {
    use super::*;
    pub enum TestDeviceApp {
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
    
    impl Application for TestDeviceApp {
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
            String::from("TestDeviceApp - Iced")
        }
        
        fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
            match self {
            Self::Connect {ip_address, ..} => match message {
                Message::InputIpAddressChanged(txt) => *ip_address = txt,
                Message::Connect => *self = Self::TestInvertor ( test_invertor::TestInvertor::new(ip_address.clone())),
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
        use modbus::{Invertor}; // Device
        use modbus::{Value, ModbusValues};
        
        pub struct TestInvertor {
            ui: UI,
            invertor: Invertor,
            values: Vec<DeviceValue>,
        }
        
        #[derive(Default)]
        struct UI {
            pb_start: button::State,
            pb_stop: button::State,
            scroll_value: scrollable::State,
        }
        
        #[derive(Debug, Clone)]
        pub enum Message {
            Start,
            Stop,
            InputValue(String, String), // ValueName, Value
        }
        
        impl TestInvertor {
            pub fn new(ip_address: String) -> Self {
                let invertor = Invertor::new(init::make_invertor(ip_address).into());
                Self {
                    values: make_values(invertor.device().values_map()),
                    invertor: invertor,
                    ui: Default::default()
                }
            }
            #[allow(unused_must_use)]
            pub fn update(&mut self, message: Message) {
//                 println!("update");
                match message {
                    Message::Start => self.invertor.start(),
                    Message::Stop => self.invertor.stop(),
                };
            }
            pub fn view(&mut self) -> Element<Message> {
//                 println!("view");
                
                let mut res = Column::new()
                    .spacing(20)
                    .align_items(Align::Center)
                    .push(Text::new(self.invertor.device().name()))
                    .push(Text::new(format!("Values: {}", self.invertor.device().values_map().len())))
                    ;
                res = if self.invertor.device().is_connect() {
                    let start = Button::new(&mut self.ui.pb_start, Text::new("Старт"))
                        .on_press(Message::Start);
                    let stop = Button::new(&mut self.ui.pb_stop, Text::new("Стоп"))
                        .on_press(Message::Stop);
                        
                    res.push(start)
                        .push(stop)
                } else {
                    res.push(Text::new(format!("Инвертор не подключен!\nIP Address: {}", self.invertor.device().get_ip_address())))
                };
                
                let mut scroll = Scrollable::new(&mut self.ui.scroll_value);
                scroll = self.values.iter_mut().fold(scroll, |scroll, v| scroll.push(v.view()));
                res = res.push(scroll);
                
                res.into()
            }
            
            fn control_view(&mut self) -> Element<Message> {
                let make_row = |label, input_state, input_value| Row::new()
                    .spacing(20)
                    .align_items(Align::Center)
                    .push(Text::new(label))
                    .push(TextInput::new(input_state, "Input Value", input_value, 
                        |inpit_value| Message::InputValue(label, input_value)));
                    
                Column::new()
                    .spacing(20)
                    .align_items(Align::Center)
//                     .push(make_row("Скорость", ))
            }
        }
        
        fn make_values(values: &ModbusValues) -> Vec<DeviceValue> {
//             println!("values_view");
            use std::collections::HashMap;
            let mut adr_name: Vec<_> = values.values().into_iter().map(|v| (v.address(), v.name().clone())).collect();
            adr_name.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
            adr_name.into_iter().map(|(_, name)| 
                DeviceValue::new(values.get(&name).unwrap().clone())
            ).collect()
        }
        
        use std::sync::Arc;
        struct DeviceValue {
            value: Arc<Value>,
        }
        impl DeviceValue {
            fn new(value: Arc<Value>) -> Self {
                Self {
                    value: value.clone(),
                }
            }
            
            fn view(&mut self) -> Element<Message> {
                let mut txt: String = self.value.name().chars().take(20).collect();
                if self.value.name().chars().nth(20).is_some() {
                    txt = txt + "...";
                }
                Text::new(format!("{:0>4X}) name: {}", self.value.address(), txt)).size(12) // {:0>4})
                    .into()
            }
        }
    }
}
