use iced::{
    Application, executor, Command, Subscription, time,
    Element, Container, Text, button, Button, slider, Slider,
    Column, Row, Space,
    Length,
};

use crate::graphic::{self, Graphic};
use modbus::{Value, ModbusValues, ValueError};
use modbus::init;
use modbus::invertor::{Invertor, DvijDirect}; // Device
use modbus::{Device, DeviceError, DigitIO};

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::Arc;

pub struct App {
    ui: UI,
    
    graph: Graphic,
    is_started: bool,
    speed: u32,
    
    values: BTreeMap<String, Arc<Value>>,
    invertor: Invertor,
    digit_io: DigitIO,
    owen_analog: Arc<Device>,
    
    klapans: [bool; 2],
    
    log: log::Logger,
    log_values: Vec<log::LogValue>,
}

#[derive(Default)]
struct UI {
    start: ui_button_start::State,
    klapan: [button::State; 3],
    speed: slider::State,
    
    pb_svg_save: button::State,
    pb_reset: button::State,
}

#[derive(Debug, Clone)]
pub enum Message {
    ModbusUpdate, ModbusUpdateAsync, ModbusUpdateAsyncAnswer(Result<(), DeviceError>),
    GraphicUpdate,
    ToggleStart(bool),
    ToggleKlapan(usize, bool),
    
    SpeedChanged(u32),
    SetSpeed(u16),
    
    GraphicMessage(graphic::Message),
    ButtonStart(ui_button_start::Message),
    
    SaveSvg,
    LogReset,
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
        let dev_owen_analog: Device = init::make_owen_analog("192.168.1.11".into()).into();
        
        let mut values = BTreeMap::new();
        for (dev, (k,v)) in dev_invertor.values_map().iter().map(|v|("Invertor", v))
            .chain(dev_digit_io.values_map().iter().map(|v|("DigitIO", v)))
            .chain(dev_owen_analog.values_map().iter().map(|v|("Analog", v)))
            .filter(|(_dev, (_k,v))| v.is_read_only()) {
            values.insert(format!("{}/{}", dev, k.clone()), v.clone());
        }
        
        let mut graphic = Graphic::new();
//         graphic.set_datetime_start(chrono::Local::now());
        
        let value_names: Vec<_> = values.keys().into_iter()
            .map(|k| k.as_str())
            .collect();
//             dbg!(&value_names);

        let temp_value_names = [
            "Analog/Температура Ротора",
            "Analog/Температура Статора",
            "Analog/Температура Пер.Под.",
            "Analog/Температура Зад.Под.",
            "Invertor/Температура радиатора",
        ];
        graphic.add_series("Температуры", false, &temp_value_names);
        
        graphic.add_series("Скорость", false, &["Invertor/Выходная частота (H)"]);
        graphic.add_series("Скорость", true, &["Analog/Вибрация 4_20 A"]);
        graphic.add_series("Ток", true, &["Invertor/Выходной ток (A)"]);
        
        (
            Self {
                ui: UI::default(),
                graph: graphic,
                is_started: false,
                speed: 0,
                
                klapans: [false; 2],
                
                values: values,
                invertor: invertor,
                digit_io: digit_io,
                owen_analog: Arc::new(dev_owen_analog),
                
                log: log::Logger::open_csv(),
                log_values: Vec::new(),
            },
            Command::none()
        )
    }
    
    fn title(&self) -> String {
        String::from("GraphicsApp - Iced")
    }
    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::batch(vec![
            time::every(std::time::Duration::from_millis(200))
            .map(|_| Message::ModbusUpdate),
            time::every(std::time::Duration::from_millis(200))
            .map(|_| Message::GraphicUpdate),
        ])
    }
    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
        Message::ModbusUpdate  => {
            use std::convert::TryFrom;
            let devices = [&self.owen_analog, 
                &self.digit_io.device(), &self.invertor.device()];
                
            for d in &devices {
                d.update();
            }
            let values = {
                self.values.iter()
                .map(|(k, v)| 
                    if let Ok(value) = f32::try_from(v.as_ref()) {
                        (&k[..], value)
                    } else {(&v.name()[..], -1.0)}
                ).collect()
            };
            self.graph.append_values(values);
            let mut log_values: Vec<_> = {
                use std::convert::TryFrom;
                self.values.iter()
                .map(|(k, v)| v)
                .filter(|v| v.is_log())
                .filter_map(|v| Some((v, f32::try_from(v.as_ref()).ok()?)))
                .map(|(v, vf)| log::LogValue::new(v.hash(), vf)).collect()
            };
            self.log_values.append(&mut log_values);
            
            let speed_value = self.invertor.get_hz_out_value();
            let speed_value = f32::try_from(speed_value.as_ref()).unwrap();
            if self.is_started == false && speed_value > 5.0 {
                self.is_started = true;
                self.reset_values();
            } else if self.is_started == true && speed_value < 5.0 {
                self.is_started = false;
                self.graph.save_svg();
                self.log_save();
            }
        },
        Message::ModbusUpdateAsync => {
//             self.owen_analog.update();
            return Command::perform(self.owen_analog.update_async(), Message::ModbusUpdateAsyncAnswer);
        },
        Message::GraphicUpdate => self.graph.update_svg(),
        Message::ButtonStart(message) => self.ui.start.update(message),
        
        Message::ToggleStart(start) => {
            self.is_started = start;
            self.ui.start = Default::default();
            // Invertor SetSpeed
            // Invertor Start | Stop
            if start {
                self.invertor.start();
            } else {
                self.invertor.stop();
            }
            self.log_save();
        },
        Message::ToggleKlapan(ind, enb) => {
            let device = &self.digit_io;
            self.klapans[ind as usize] = enb;
            self.klapans[1-ind as usize] = false;
            match ind {
            0 => {
                device.turn_clapan(1, false).unwrap();
                device.turn_clapan(2, enb).unwrap();
                device.turn_clapan(3, enb).unwrap();
            }, 1 => {
                device.turn_clapan(1, enb).unwrap();
                device.turn_clapan(2, false).unwrap();
                device.turn_clapan(3, false).unwrap();
            }, _ => {}
            }
        },
        Message::SpeedChanged(speed) => {
            self.speed = speed;
//             dbg!((10*speed)/6);
            self.invertor.set_speed((10*speed)/6);
        },
//         Message::SetSpeed(speed) => {},
        Message::SaveSvg => self.graph.save_svg(),
        Message::LogReset => self.reset_values(),
        _ => {}
        };
        Command::none()
    }
    fn view(&mut self) -> Element<Self::Message> {
//         let content = Text::new("Пустое окно");

        let list_value = self.view_list_value();
        let graph = self.graph.view()
            .map(Message::GraphicMessage);
            
        let row = Row::new()
            .spacing(20)
            .push(list_value)
            .push(graph);
        
        let controls = {
            let klapans = if self.digit_io.device().is_connect() {
                let klapan_names = vec!["Уменьшить давление", "Увеличить давление"];
                let klapans = self.klapans.iter()
                    .zip(self.ui.klapan.iter_mut());
        //         let ui = &mut self.ui;
                let controls_klapan = klapan_names.iter()
                    .zip(0..)
                    .zip(klapans)
                    .fold(Row::new().spacing(20),
                        |row, ((&name, ind), (&check, pb))| 
                        row.push(Button::new(pb, Text::new(name))
                        .style(style::Button::Check{checked: check})
                        .on_press(Message::ToggleKlapan(ind, !check)))
                    );
                
                let buttons = controls_klapan.push(
                    Button::new(&mut self.ui.pb_svg_save, Text::new("Сохранить график"))
                        .on_press(Message::SaveSvg)
                    ).push(
                    Button::new(&mut self.ui.pb_reset, Text::new("Сброс значений"))
                        .on_press(Message::LogReset)
                    );
    //             controls_klapan.into()
                Element::from(buttons)
            } else {Element::from(Text::new("Цифровой модуль ОВЕН не подключен"))};
            
            let invertor: Element<_> = if self.invertor.device().is_connect() {
                let is_started = self.is_started;
                let start = self.ui.start.view(
                    self.is_started,
//                     Message::ToggleStart(!self.is_started)
                ).map(move |message| {
                    if let ui_button_start::Message::ToggleStart(start) = message {
                        Message::ToggleStart(start)
                    } else {
                        Message::ButtonStart(message)
                    }
                });
                
                let slider = Slider::new(
                    &mut self.ui.speed,
                    0..=24_000,
                    self.speed,
                    Message::SpeedChanged
                )
//                 .on_release(Message::SetSpeed(self.speed))
                .step(3_000);
                
                Column::new().spacing(5)
                    .push(
                        Row::new().spacing(20)
                            .push(Text::new(format!("Speed: {:0>5}", self.speed)))
                            .push(slider)
                    ).push(start)
                    .into()
            } else {
                Text::new("Инвертор не подключен")
                    .into()
            };
            
            Column::new()
                .spacing(20)
                .push(klapans)
                .push(invertor)
                .push(Space::with_height(Length::Fill))
        };
        
        let content: Element<_> = Column::new()
            .spacing(20)
            .push(row)
            .push(controls)
            .into();
            
//         let content = content.explain([0.0, 0.0, 0.0]);
        
        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(10)
            .center_x()
            .center_y()
            .style(style::MainContainer)
            .into()
    }
}

impl Drop for App {
    fn drop(&mut self) {
        self.log_save();
    }
}

impl App {

    fn log_save(&mut self) {
        if self.log_values.len() > 0 {
            self.log.new_session(&self.log_values);
            self.log_values = Vec::new();
        }
    }
    
    fn reset_values(&mut self) {
        self.log_values = Vec::new();
        self.graph.reset_values()
    }
    
    fn get_values_name_map<'a>() -> HashMap<&'a str, Vec<&'a str>> {
        let mut map = HashMap::new();
        map.insert("Analog", vec![
            "Температура Ротора",
            "Температура Статора",
            "Температура Пер.Под.",
            "Температура Зад.Под.",
            
            "Давление -1_1 V",
            "Вибрация 4_20 A",
        ]);
        
        map.insert("DigitIO", vec![
            "Клапан 24В",
            "Клапан 2",
            "Насос",
        ]);
        
        map.insert("Invertor", vec![
            "Заданная частота (F)",
            "Выходная частота (H)",
            "Выходной ток (A)",
            "Температура радиатора",
            "Наработка двигателя (дни)",
            "Наработка двигателя (мин)",
        ]);
        
        map
    }

    fn view_list_value<'a>(&self) -> Element<'a, Message> {
    
        let mut lst = Column::new()
        .spacing(20);
//         .width(Length::Units(200));
        let values_name_map = Self::get_values_name_map();
        {
            let values_name = &values_name_map[&"Analog"];
            
            let values_map = self.owen_analog.values_map();
            lst = lst.push( Self::view_map_values(values_name, &values_map, |name| format!("{}/value_float", name)));
        };
//         {
//             let values_name = &values_name_map[&"DigitIO"];
//             let dev = self.digit_io.device();
//             let values_map = dev.values_map();
//             lst = lst.push( Self::view_map_values(values_name, &values_map, |name| format!("{}/value", name)));
//         };
        
        {
            let values_name = &values_name_map[&"Invertor"];
            let dev = self.invertor.device();
            let values_map = dev.values_map();
            lst = lst.push( Self::view_map_values(values_name, &values_map, |name| format!("{}", name)));
        };
        
        lst.into()
    }
    
    fn view_map_values<'a, F>(names: &Vec<&str>, map: &ModbusValues, value_key: F) -> Element<'a, Message> 
    where F: Fn(&str) -> String
    {
        pub use std::convert::TryFrom;
        names.into_iter()
            .fold(Column::new().width(Length::Units(200)).spacing(2),
            |lst, &name| {
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
            Some(err) if err.red <= value =>
                [1.0, 0.0, 0.0],
            Some(err) if err.yellow <= value => 
                [1.0, 1.0, 0.0],
            Some(_) | None => [0.0, 0.8, 0.0],
        };
        let text = Text::new(
            format!("{}\nValue: {:.2}", text, value)
        ).size(20)
        .color(color);
        
        Container::new(text)
            .width(Length::Fill)
            .style(style::ValueContainer)
            .into()
    }
    
}

mod style {
    use iced::{button, container, Background, Color, Vector};

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
                    border_radius: 10_f32,
                    text_color: Color::WHITE,
                    ..button::Style::default()
                }
            } else {
                button::Style {
                    background: Some(Background::Color(
                        Color::from_rgb8(0, 150, 0),
                    )),
                    border_radius: 10_f32,
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

    pub(super) struct MainContainer;
    impl container::StyleSheet for MainContainer {
        fn style(&self) -> container::Style {
            container::Style {
                background: Some(Background::Color([0.8, 0.8, 0.8].into())),
                .. Default::default()
            }
        }
    }
    
    pub(super) struct ValueContainer;
    impl container::StyleSheet for ValueContainer {
        fn style(&self) -> container::Style {
            container::Style {
                background: Some(Background::Color([0.3, 0.3, 0.3].into())),
                .. Default::default()
            }
        }
    }
}

mod ui_button_start {
    use super::*;
    
    pub enum State {
        Start {
            start: button::State,
        },
        Confirm {
            confirm: button::State,
            cancel: button::State,
        },
    }
    
    #[derive(Debug, Clone)]
    pub enum Message {
        TryStart,
        Confirm,
        Cancel,
        ToggleStart(bool),
    }
    
    impl Default for State {
        fn default() -> State {
            State::Start {
                start: Default::default(),
            }
        }
    }
    
    impl State {
        pub fn update(&mut self, message: Message) {
            match self {
            Self::Start {..} => if let Message::TryStart = message {
                *self = Self::Confirm {
                    confirm: Default::default(),
                    cancel: Default::default(),
                }
            },
            Self::Confirm {..} => {
                *self = Self::Start {
                    start: Default::default(),
                };
            },
            _ => {}
            };
        }
        
        pub fn view(&mut self, is_started: bool) -> Element<Message> {
            match self {
            Self::Start {start} => {
                let pb = Button::new(start, 
                    if !is_started { Text::new("Start") }
                    else {Text::new("Stop")}
                ).style(style::Button::Check{
                    checked: is_started
                });
                let pb = if !is_started {
                    pb.on_press(Message::TryStart)
                } else {
                    pb.on_press(Message::ToggleStart(false))
                };
                
                pb.into()
            }, Self::Confirm {confirm, cancel} => {
                let pb_cancel = Button::new(cancel, 
                    Text::new("Отмена")
                ).on_press(Message::Cancel);
                let pb_start = Button::new(confirm, 
                    Text::new("Запустить")
                ).on_press(Message::ToggleStart(true));
                
                Row::new().spacing(50)
                    .push(pb_cancel)
                    .push(pb_start)
                    .into()
            }
            }
        }
    }
}
