use iced::{
    Application, executor, Command, Subscription, time,
    Element, Container, Text, button, Button,
    Column,
    Length,
};

use crate::graphic::{self, Graphic};
use modbus::{Value, ModbusValues, ValueError};
use modbus::init;
use modbus::invertor::{Invertor, DvijDirect}; // Device
use modbus::{Device, DigitIO};

use std::collections::BTreeMap;
use std::sync::Arc;

pub struct App {
    ui: UI,
    
    is_started: bool,
    
    values: BTreeMap<String, Arc<Value>>,
    invertor: Invertor,
    digit_io: DigitIO,
    owen_analog: Device,
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
        let dev_owen_analog: Device = init::make_owen_analog("192.168.1.5".into()).into();
        
        let mut values = BTreeMap::new();
        for (dev, (k,v)) in dev_invertor.values_map().iter().map(|v|("Invertor", v))
            .chain(dev_digit_io.values_map().iter().map(|v|("DigitIO", v)))
            .chain(dev_owen_analog.values_map().iter().map(|v|("Analog", v)))
            .filter(|(_dev, (_k,v))| v.is_read_only()) {
            values.insert(format!("{}/{}", dev, k.clone()), v.clone());
        }
        
        (
            Self {
                ui: UI::default(),
                is_started: false,
                
                values: values,
                invertor: invertor,
                digit_io: digit_io,
                owen_analog: dev_owen_analog,
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

        let content = self.view_list_value();
        
        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(10)
            .center_x()
            .center_y()
            .into()
    }
}

impl App {
    fn view_list_value(&self) -> Element<Message> {
    
        let mut lst = Column::new()
        .spacing(20);
//         .width(Length::Units(200));
        {
            let values_name = vec![
                "Температура Ротора",
                "Температура Статора",
                "Температура Пер.Под.",
                "Температура Зад.Под.",
                
                "Давление -1_1 V",
                "Вибрация 4_20 A",
            ];
            
            let values_map = self.owen_analog.values_map();
            lst = lst.push( Self::view_map_values(values_name, &values_map, |name| format!("{}/value_float", name)));
        };
        {
            let values_name = vec![
                "Клапан 24В",
                "Клапан 2",
                "Насос",
            ];
            let dev = self.digit_io.device();
            let values_map = dev.values_map();
            lst = lst.push( Self::view_map_values(values_name, &values_map, |name| format!("{}/value", name)));
        };
        
        {
            let values_name = vec![
                "Заданная частота (F)",
                "Выходная частота (H)",
                "Выходной ток (A)",
                "Температура радиатора",
            ];
            let dev = self.invertor.device();
            let values_map = dev.values_map();
            lst = lst.push( Self::view_map_values(values_name, &values_map, |name| format!("{}", name)));
        };
        
        lst.into()
    }
    
    fn view_map_values<'a, F>(names: Vec<&str>, map: &ModbusValues, value_key: F) -> Element<'a, Message> 
    where F: Fn(&str) -> String
    {
        pub use std::convert::TryFrom;
        names.into_iter()
            .fold(Column::new(),
            |lst, name| {
                let key = value_key(name);
                let name = name.into();
                if let Some(value) = map.get(&key) {
                    let err = value.get_error();
                    let value = f32::try_from(value.as_ref()).unwrap();
                    lst.push(Self::view_value(name, value, err))
                } else {lst}
            }
        ).into()
    }
    
    fn view_value<'a>(text: String, value: f32, err: Option<ValueError>) -> Element<'a, Message> {
        let color = match err {
            Some(err) if err.yellow <= value => 
                [1.0, 1.0, 0.0],
            Some(err) if err.red <= value =>
                [1.0, 0.0, 0.0],
            Some(_) | None => [0.0, 0.8, 0.0],
        };
        Text::new(
            format!("Name: {}\nValue: {}", text, value)
        ).size(16)
        .color(color)
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
