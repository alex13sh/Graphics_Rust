use iced::{
    Application, executor, Command, Subscription, time,
    Element, Container, Text, button, Button,
    Column,
    Length,
};

use crate::graphic::{self, Graphic};
use modbus::{Value, ModbusValues};
use modbus::init;
use modbus::invertor::{Invertor, DvijDirect}; // Device
use modbus::{Device, DigitIO};


pub struct App {
    ui: UI,
    
    is_started: bool,
    
    values: ModbusValues,
    invertor: Invertor,
    digit_io: DigitIO,
}

#[derive(Default)]
struct UI {
    start: button::State,
}

#[derive(Debug, Clone)]
pub enum Message {
    ModbusUpdate,
    GraphicUpdate,
    ToggleStart(bool),
    
}

impl Application for App {
    type Executor = executor::Default;
    type Flags = ();
    type Message = Message;
    
    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        let invertor = Invertor::new(init::make_invertor("192.168.1.5".into()).into());
        let dev_invertor = invertor.device();
        let digit_io = DigitIO::new(init::make_io_digit("192.168.1.10".into()).into());
        let dev_digit_io = digit_io.device();
        let mut values = ModbusValues::new();
        for (k,v) in dev_invertor.values_map().iter()
            .chain(dev_digit_io.values_map().iter())
            .filter(|(_k,v)| v.is_read_only()) {
            values.insert(k.clone(), v.clone());
        }
        
        (
            Self {
                ui: UI::default(),
                is_started: false,
                
                values: values,
                invertor: invertor,
                digit_io: digit_io,
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
        Message::ToggleStart(start) => self.is_started = start,
        _ => {}
        };
        Command::none()
    }
    fn view(&mut self) -> Element<Self::Message> {
//         let content = Text::new("Пустое окно");

        let content = self.values.iter().fold(
            Column::new(),
            |row, (k, _value)| row.push(Text::new(k))
        );
        
        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(10)
            .center_x()
            .center_y()
            .into()
    }
}


mod style {
    use iced::{button, Background, Color, Vector};

    pub enum Button {
        Check { checked: bool },
    }

    impl button::StyleSheet for Button {
        fn active(&self) -> button::Style {
            match self {
            Button::Check { checked } => if *checked {
                button::Style {
                    background: Some(Background::Color(
                        Color::from_rgb8(150, 0,0),
                    )),
                    border_radius: 10,
                    text_color: Color::WHITE,
                    ..button::Style::default()
                }
            } else {
                button::Style {
                    background: Some(Background::Color(
                        Color::from_rgb8(0, 150, 0),
                    )),
                    border_radius: 10,
                    text_color: Color::WHITE,
                    ..button::Style::default()
                }
            },
            }
        }

        fn hovered(&self) -> button::Style {
            let active = self.active();

            button::Style {
                text_color: match self {
                Button::Check { checked } if !checked => {
                    Color::from_rgb(0.2, 0.2, 0.7)
                }
                _ => active.text_color,
                },
                shadow_offset: active.shadow_offset + Vector::new(0.0, 1.0),
                ..active
            }
        }
    }
}
