use iced::{
    Application, executor, Command, window::Mode, Subscription, time,
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

mod ui;
use ui::style;

pub struct App {
    ui: UI,
    
    graph: Graphic,

    shim_hz: u32,
    shim_hz_enb: bool,
    
    values: BTreeMap<String, Arc<Value>>,
    logic: meln_logic::init::Complect,
    invertor: ui::Invertor,
    klapans: ui::Klapans,
    
    log: log::Logger,
    log_values: Vec<log::LogValue>,

    has_exit: bool,
}

#[derive(Default)]
struct UI {

    shim_hz: slider::State,
    
    pb_svg_save: button::State,
    pb_reset: button::State,

    pb_exit: button::State,
}

#[derive(Debug, Clone)]
pub enum Message {
    InvertorUI(ui::invertor::Message),
    KlapansUI(ui::klapans::Message),

    ModbusUpdate, ModbusUpdateAsync, ModbusUpdateAsyncAnswer,
    ModbusUpdateAsyncAnswerDevice(Arc<Device>, Result<(), DeviceError>),
    GraphicUpdate,

    
    ShimHzChanged(u32),
    SetShimHz,
    
    GraphicMessage(graphic::Message),
    
    SaveSvg,
    LogReset,

    ButtonExit,
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
        
//         let value_names: Vec<_> = values.keys().into_iter()
//             .map(|k| k.as_str())
//             .collect();
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

                shim_hz: 0,
                shim_hz_enb: true,

                invertor: ui::Invertor::new(logic.invertor.device().clone()),
                klapans: ui::Klapans::new(logic.digit_o.device().values_map()
                    .get_values_by_name_starts(&["Клапан 24В", "Клапан 2", "Насос"])),
                logic: logic,
                values: values,
                
                log: log::Logger::open_csv(),
                log_values: Vec::new(),

                has_exit: false,
            },
            Command::none()
        )
    }
    
    fn title(&self) -> String {
        String::from("GraphicsApp (2 fps) - Iced")
    }
    fn mode(&self) -> Mode {
        Mode::Fullscreen
    }
    fn should_exit(&self) -> bool {
        self.has_exit
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::batch(vec![
            time::every(std::time::Duration::from_millis(500))
            .map(|_| Message::ModbusUpdateAsync),
            time::every(std::time::Duration::from_millis(500))
            .map(|_| Message::GraphicUpdate),
        ])
    }
    fn update(&mut self, message: Self::Message, _clipboard: &mut Clipboard) -> Command<Self::Message> {
    
        match message {
        Message::InvertorUI(m) => {
            use ui::invertor::Message as M;
            match &m {
                M::ToggleStart(_) => self.log_save(),
                _ => {}
            }
            self.invertor.update(m);
        },
        Message::KlapansUI(m) => {
            self.klapans.update(m);
            self.logic.update_new_values();
        },
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

        Message::ShimHzChanged(hz) => self.shim_hz = hz,
        Message::SetShimHz => {
            println!("Set HZ: {}", self.shim_hz);
        },
//         Message::SetSpeed(speed) => {},
        Message::SaveSvg => self.graph.save_svg(),
        Message::LogReset => self.reset_values(),
        Message::ButtonExit => self.has_exit = true,
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
                let controls_klapan = self.klapans.view().map(Message::KlapansUI);
                let controls = Row::new().spacing(20)
                    .push(controls_klapan);
                
                let slider = {
                    let slider = Slider::new(
                        &mut self.ui.shim_hz,
                        0..=20,
                        self.shim_hz,
                        Message::ShimHzChanged
                    )
                    .on_release(Message::SetShimHz)
                    .step(1);

                    Column::new().spacing(5)
                        .push(
                            Row::new().spacing(20)
                                .push(Text::new(format!("Частота ШИМ: {:0>5}", self.shim_hz)))
                                .push(slider)
                        )
                };

                let buttons = controls.push(
                    Button::new(&mut self.ui.pb_svg_save, Text::new("Сохранить график"))
                        .on_press(Message::SaveSvg)
                    ).push(
                    Button::new(&mut self.ui.pb_reset, Text::new("Сброс значений"))
                        .on_press(Message::LogReset)
                    );
    //             controls_klapan.into()
                let controls = Column::new().spacing(2)
                    .push(buttons)
                    .push(slider);
                Element::from(controls)
            } else {Element::from(Text::new("Цифровой модуль ОВЕН не подключен"))};
            
            let invertor: Element<_> = self.invertor.view().map(Message::InvertorUI);
            
            Column::new()
                .spacing(20)
                .push(klapans)
                .push(invertor)
                .push(Space::with_height(Length::Fill))
                .push(Row::new()
                    .push(Space::with_width(Length::Fill))
                    .push(Button::new(&mut self.ui.pb_exit, Text::new("Выход"))
                        .on_press(Message::ButtonExit)
                        .style(style::Button::Exit))
                )
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
            
        if self.invertor.is_started == false && speed_value > 5.0 {
            self.invertor.is_started = true;
            self.reset_values();
        } else if self.invertor.is_started == true
                && (speed_value < 2.0 && vibra_value<0.2) {
            self.invertor.is_started = false;
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
            ("Скорость", "4bd5c4e0a9"),
            ("Ток", "5146ba6795"),
            ("Напряжение", "5369886757"),
            ("Вибродатчик", "2) МВ110-24.8АС/7/value"),
            ("Температура ротора", "2) МВ110-24.8АС/5/value"),
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


