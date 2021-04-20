use iced::{
    Application, executor, Command, Subscription, time,
    Element, Container, Text, button, Button, slider, Slider,
    Column, Row, Space, Length,
    Settings, Clipboard,
};

fn main() {
    App::run(Settings::default());
}


use graphic::{self, Graphic};
use modbus::{Value, ModbusValues, ValueError, Device, DeviceError };

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::Arc;

pub struct App {
    ui: UI,
    
    graph: Graphic,
    is_started: bool,
    speed: u32,
    
    values: BTreeMap<String, Arc<Value>>,
    logic: meln_logic::init::Complect,
    
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
    ModbusUpdate, ModbusUpdateAsync, ModbusUpdateAsyncAnswer,
    ModbusUpdateAsyncAnswerDevice(Arc<Device>, Result<(), DeviceError>),
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
        let logic = meln_logic::init::Complect::new();
        let values = logic.make_values(true);
//         logic.init_values(&values);
                
        let mut graphic = Graphic::new();
//         graphic.set_datetime_start(chrono::Local::now());
        
        let value_names: Vec<_> = values.keys().into_iter()
            .map(|k| k.as_str())
            .collect();
//             dbg!(&value_names);

        let temp_value_names = [
            "2) МВ110-24.8АС/Температура ротора Пирометр дв. М1",
            "1) МВ210-101/Температура статора двигатель М1",
            "1) МВ210-101/Температура масла на выходе дв. М1 Низ",
            "1) МВ210-101/Температура подшипника дв. М1 верх",
            "Invertor/Температура радиатора",
        ];
        graphic.add_series("Температуры", false, &temp_value_names);
        
        graphic.add_series("Скорость", false, &["Invertor/Выходная частота (H)"]);
        graphic.add_series("Скорость", true, &["2) МВ110-24.8АС/Вибродатчик дв. М1"]);
        graphic.add_series("Ток", false, &["Invertor/Выходной ток (A)"]);
        
        (
            Self {
                ui: UI::default(),
                graph: graphic,
                is_started: false,
                speed: 0,
                
                klapans: [false; 2],
                
                values: values,
                logic: logic,
                
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
            time::every(std::time::Duration::from_millis(500))
            .map(|_| Message::ModbusUpdateAsync),
            time::every(std::time::Duration::from_millis(1000))
            .map(|_| Message::GraphicUpdate),
        ])
    }
    fn update(&mut self, message: Self::Message, _clipboard: &mut Clipboard) -> Command<Self::Message> {
    
        match message {
        Message::ModbusUpdate  => {
            self.logic.update();
           
            self.proccess_values();
            self.proccess_speed();
        },
        Message::ModbusUpdateAsync => {
            let device_futures = self.logic.update_async();
                
            return Command::batch(device_futures.into_iter()
                .map(|(d, f)| Command::perform(f, move |res| Message::ModbusUpdateAsyncAnswerDevice(d.clone(), res)))
                );
        },
        Message::ModbusUpdateAsyncAnswer => {
//             self.proccess_values();
//             self.proccess_speed();
        },
        Message::ModbusUpdateAsyncAnswerDevice(d, res) => {
//             dbg!(&d);
            if res.is_ok() {
//                 println!("Message::ModbusUpdateAsyncAnswerDevice {}", d.name());
                if !d.is_connect() {
//                     println!("\tis not connect");
                } else {
//                     self.proccess_values();
//                     self.proccess_speed();
                }
            }
        },
        Message::GraphicUpdate => {            
            self.graph.update_svg();
            
            self.proccess_values();
            self.proccess_speed();
        },
        Message::ButtonStart(message) => self.ui.start.update(message),
        
        Message::ToggleStart(start) => {
            self.is_started = start;
            self.ui.start = Default::default();
            // Invertor SetSpeed
            // Invertor Start | Stop
            if start {
                self.logic.invertor.start();
            } else {
                self.logic.invertor.stop();
            }
            self.log_save();
        },
        Message::ToggleKlapan(ind, enb) => {
            
            self.klapans[ind as usize] = enb;
            self.klapans[1-ind as usize] = false;
            match ind {
            0 => {
                self.logic.set_bit("Клапан 24В", false).unwrap();
                self.logic.set_bit("Клапан 2", enb).unwrap();
                self.logic.set_bit("Насос", enb).unwrap();
            }, 1 => {
                self.logic.set_bit("Клапан 24В", enb).unwrap();
                self.logic.set_bit("Клапан 2", false).unwrap();
                self.logic.set_bit("Насос", false).unwrap();
            }, _ => {}
            }
            self.logic.update_new_values();
        },
        Message::SpeedChanged(speed) => {
            self.speed = speed;
//             dbg!((10*speed)/6);
            self.logic.invertor.set_speed((10*speed)/6);
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
            let klapans = if self.logic.digit_o.device().is_connect() {
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
            
            let invertor: Element<_> = if self.logic.invertor.device().is_connect() {
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

// logic
impl App {
    fn proccess_values(&mut self) {
        use std::convert::TryFrom;
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
            self.values.iter()
            .map(|(k, v)| v)
            .filter(|v| v.is_log())
            .filter_map(|v| Some((v, f32::try_from(v.as_ref()).ok()?)))
            .map(|(v, vf)| log::LogValue::new(v.hash(), vf)).collect()
        };
        self.log_values.append(&mut log_values);
    }
    
    fn proccess_speed(&mut self) {
        use std::convert::TryFrom;
        let speed_value = self.logic.invertor.get_hz_out_value();
        let speed_value = f32::try_from(speed_value.as_ref()).unwrap();
        
        let vibra_value = self.logic.owen_analog_2.values_map().get("Вибродатчик дв. М1/value").unwrap().clone();
        let vibra_value = f32::try_from(vibra_value.as_ref()).unwrap();
            
        if self.is_started == false && speed_value > 5.0 {
            self.is_started = true;
            self.reset_values();
        } else if self.is_started == true 
                && (speed_value < 2.0 && vibra_value<0.2) {
            self.is_started = false;
            self.graph.save_svg();
            self.log_save();
        };
    }
}

// view
impl App {

    fn log_save(&mut self) {
        if self.log_values.len() > 0 {
            self.log.new_session(&self.log_values);
            
            log::Logger::new_table_fields(&self.log_values, 1, vec![
            ("Температура ротора", "2) МВ110-24.8АС/5/value"),
            ("Вибродатчик", "2) МВ110-24.8АС/7/value"),
            ("Температура статора", "1) МВ210-101/1/value"),
            ("Температура масла на выходе дв. М1 Низ", "1) МВ210-101/2/value"),
            ("Температура подшипника дв. М1 верх", "1) МВ210-101/6/value"),
            ]);
            
            self.log_values = Vec::new();
        }
    }
    
    fn reset_values(&mut self) {
        self.log_values = Vec::new();
        self.graph.reset_values()
    }
    
    fn get_values_name_map<'a>() -> HashMap<&'a str, Vec<&'a str>> {
        let mut map = HashMap::new();
        map.insert("1) МВ210-101", vec![
            "Температура статора двигатель М1",
            "Температура масла на выходе дв. М1 Низ",
//             "Температура масла на выходе дв. М2 Низ",
//             "Температура масла на выходе маслостанции",
//             "Температура статора двигатель М2",
            "Температура подшипника дв. М1 верх",
//             "Температура подшипника дв. М2 верх"
            
        ]);
        
        map.insert("2) МВ110-24.8АС", vec![
//             "Давление масла верхний подшипник",
//             "Давление масла нижний подшипник",
//             "Давление воздуха компрессора",
//             "Разрежение воздуха в системе",
            "Температура ротора Пирометр дв. М1",
//             "Температура ротора Пирометр дв. М2",
            "Вибродатчик дв. М1",
//             "Вибродатчик дв. М2",
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
            let values_name = &values_name_map[&"1) МВ210-101"];
            
            let values_map = self.logic.owen_analog_1.values_map();
            lst = lst.push( Self::view_map_values(values_name, &values_map, |name| format!("{}/value", name)));
        };
        {
            let values_name = &values_name_map[&"2) МВ110-24.8АС"];
            
            let values_map = self.logic.owen_analog_2.values_map();
            lst = lst.push( Self::view_map_values(values_name, &values_map, |name| format!("{}/value", name)));
        };
//         {
//             let values_name = &values_name_map[&"DigitIO"];
//             let dev = self.digit_io.device();
//             let values_map = dev.values_map();
//             lst = lst.push( Self::view_map_values(values_name, &values_map, |name| format!("{}/value", name)));
//         };
        
        {
            let values_name = &values_name_map[&"Invertor"];
            let dev = self.logic.invertor.device();
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
            .fold(Column::new().width(Length::Units(250)).spacing(2),
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
