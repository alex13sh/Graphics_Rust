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
use modbus::{Value, ModbusValues, ValueFloatResult, ValueFloatError, ValueError, Device, DeviceError };

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::Arc;

// #[macro_use]
mod ui;
use ui::style;

pub struct App {
    ui: UI,
    
    graph: Graphic,
    
    values: BTreeMap<String, Arc<Value>>,
    logic: meln_logic::init::Complect,
    invertor: ui::Invertor,
    klapans: ui::Klapans,
    dozator: ui::Dozator,
    values_list: Vec<ui::ValuesList>,
    
    log: log::Logger,
    log_values: Vec<log::LogValue>,

    has_exit: bool,
}

#[derive(Default)]
struct UI {
    pb_svg_save: button::State,
    pb_reset: button::State,

    pb_exit: button::State,
}

#[derive(Debug, Clone)]
pub enum Message {
    InvertorUI(ui::invertor::Message),
    KlapansUI(ui::klapans::Message),
    DozatorUI(ui::dozator::Message),
    GraphicMessage(graphic::Message),
    MessageUpdate(MessageMudbusUpdate),
    
    
    SaveSvg,
    LogReset,

    ButtonExit,
}

#[derive(Debug, Clone)]
pub enum MessageMudbusUpdate {
    ModbusUpdate, ModbusUpdateAsync, ModbusUpdateAsyncAnswer,
    ModbusUpdateAsyncAnswerDevice(Arc<Device>, Result<(), DeviceError>),
    GraphicUpdate,
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

                invertor: ui::Invertor::new(logic.invertor_1.clone()),
                klapans: ui::Klapans::new(logic.digit_o.device().values_map()
                    .get_values_by_name_starts(&["Клапан 24В", "Клапан 2", "Насос"])),
                dozator: ui::Dozator::new(logic.dozator.clone()),
                values_list: ui::make_value_lists(logic.get_values(), map!{BTreeMap,
                    "1) МВ210-101" => [
                        "Температура статора двигатель М1",
                        "Температура масла на верхн. выходе дв. М1",
                        "Температура масла на нижн. выходе дв. М1",
                        "Температура масла на выходе маслостанции",
                        "Температура статора дв. М2",
                        "Температура верх подшипника дв. М2",
                        "Температура нижн подшипника дв. М2"
                        
                    ],
                    "2) МВ110-24.8АС" => [
                        "Давление масла на выходе маслостанции",
                        "Давление воздуха компрессора",
                        "Разрежение воздуха в системе",
                        "Температура ротора Пирометр дв. М1",
                        "Температура ротора Пирометр дв. М2",
                        "Вибродатчик дв. М1",
                        "Вибродатчик дв. М2",
                    ],
                    "Invertor" => [
                        "5) Invertor/Заданная частота (F)",
                        "5) Invertor/Выходная частота (H)",
                        "5) Invertor/Выходной ток (A)",
//                         "5) Invertor/Температура радиатора",
//                         "5) Invertor/Наработка двигателя (дни)",
//                         "5) Invertor/Наработка двигателя (мин)",
                    ]
                }),

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
//     fn scale_factor(&self) -> f64 {0.6}

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::batch(vec![
            time::every(std::time::Duration::from_millis(500))
            .map(|_| MessageMudbusUpdate::ModbusUpdateAsync),
            time::every(std::time::Duration::from_millis(500))
            .map(|_| MessageMudbusUpdate::GraphicUpdate),
        ]).map(Message::MessageUpdate)
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
        Message::DozatorUI(m) => {
            let res = self.dozator.update(m, vec![self.logic.digit_o.device().clone()])
                .map(Message::DozatorUI);
//             self.logic.update_new_values();
            return res;
        },
        Message::MessageUpdate(m) => return self.modbus_update(m),
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

//         let list_value = self.view_list_value();
        let list_value = self.values_list.iter()
            .fold(Column::new().spacing(20), |lst, v| lst.push(v.view()));
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
                
                let slider = self.dozator.view().map(Message::DozatorUI);

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

// modbus update
impl App {
    fn modbus_update(&mut self, message: MessageMudbusUpdate) -> Command<Message> {
        match message {
            MessageMudbusUpdate::ModbusUpdate  => {
                self.logic.update();

                self.proccess_values();
                self.proccess_speed();
            },
            MessageMudbusUpdate::ModbusUpdateAsync => {
                let device_futures = self.logic.update_async();

                return Command::batch(device_futures.into_iter()
                    .map(|(d, f)| Command::perform(f, move |res| Message::MessageUpdate(
                        MessageMudbusUpdate::ModbusUpdateAsyncAnswerDevice(d.clone(), res)))
                    ));
            },
            MessageMudbusUpdate::ModbusUpdateAsyncAnswer => {
    //             self.proccess_values();
    //             self.proccess_speed();
            },
            MessageMudbusUpdate::ModbusUpdateAsyncAnswerDevice(d, res) => {
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
            MessageMudbusUpdate::GraphicUpdate => {
                self.graph.update_svg();

                self.proccess_values();
                self.proccess_speed();

            },
        }
        Command::none()
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
        let speed_value = self.logic.invertor_1.get_hz_out_value();
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
}
