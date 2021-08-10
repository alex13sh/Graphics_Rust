use iced::{
    Application, executor, Command, window::Mode, Subscription, time,
    Element, Container, Text, button, Button, slider, Slider,
    Column, Row, Space, Length,
    Settings, Clipboard,
};

fn main() {
    App::run(Settings::default());
} 

mod ui;

use modbus::{ModbusValues, Device, DeviceError};
use std::collections::HashMap;
use std::collections::BTreeMap;
use std::sync::Arc;

pub struct App {
    ui: UI,
    has_exit: bool,
    logic: meln_logic::init::Complect,
    txt_status: String,
    
    klapans: ui::Klapans,
    dozator: ui::Dozator,
    top: HalfComplect,
    low: HalfComplect,
    oil_station: ui::OilStation,
    
    log: log::Logger,
    log_values: Vec<log::LogValue>,
}



#[derive(Default)]
struct UI {
    pb_exit: button::State,
    pb_stop: button::State,
}

#[derive(Debug, Clone)]
pub enum Message {
    ButtonExit,
    EmergencyStop, // Аварийная остановка
    
    TopHalfComplectUI(half_complect::Message),
    LowHalfComplectUI(half_complect::Message),
    
    OilStation(ui::oil_station::Message),
    DozatorUI(ui::dozator::Message),
    KlapansUI(ui::klapans::Message),
    
    MessageUpdate(MessageMudbusUpdate),
    
}

#[derive(Debug, Clone)]
pub enum MessageMudbusUpdate {
    ModbusUpdate, ModbusUpdateAsyncAnswer,
    ModbusUpdateAsync, ModbusUpdateAsync_Vibro, ModbusConnect,
    ModbusUpdateAsyncAnswerDevice(Arc<Device>, Result<(), DeviceError>),
//     GraphicUpdate,
    LogUpdate,
}

impl Application for App {
    type Executor = executor::Default;
    type Flags = ();
    type Message = Message;
    
    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        let logic = meln_logic::init::Complect::new();

        let values_1 = logic.get_values().get_values_by_name_contains(&["М1"]);
        let values_2 = logic.get_values().get_values_by_name_contains(&["М2"]);
        
        (App {
            ui: Default::default(),
            has_exit: false,
            txt_status: "".into(),
            
            low: HalfComplect::new(HalfPart::Low, values_1, logic.invertor_1.clone()),
            top: HalfComplect::new(HalfPart::Top, values_2, logic.invertor_2.clone()),
            klapans: ui::Klapans::new(logic.digit_o.device().values_map()
                //.get_values_by_name_starts(&["Клапан 24В", "Клапан 2", "Насос"])
                .clone()),
            dozator: ui::Dozator::new(logic.dozator.clone()),
            oil_station: ui::OilStation::new(logic.get_values().clone()),
        
            logic: logic,
            log: log::Logger::open_csv(),
            log_values: Vec::new(),
        },
        Command::none())
    }
    
    fn title(&self) -> String {
        String::from("Complect - Iced")
    }
    fn mode(&self) -> Mode {
        Mode::Fullscreen
    }
    fn should_exit(&self) -> bool {
        self.has_exit
    }
    fn scale_factor(&self) -> f64 {0.6}

    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::batch(vec![
            Subscription::batch(vec![
                time::every(std::time::Duration::from_millis(100))
                .map(|_| MessageMudbusUpdate::ModbusUpdateAsync),
//                 time::every(std::time::Duration::from_millis(100))
//                 .map(|_| MessageMudbusUpdate::ModbusUpdateAsync_Vibro),
                time::every(std::time::Duration::from_millis(100))
                time::every(std::time::Duration::from_millis(5000))
                .map(|_| MessageMudbusUpdate::ModbusConnect),
                .map(|_| MessageMudbusUpdate::LogUpdate),

            ]).map(Message::MessageUpdate),
            self.dozator.subscription().map(Message::DozatorUI),
        ])
    }
    
    fn update(&mut self, message: Self::Message, _clipboard: &mut Clipboard) -> Command<Self::Message> {
    
        match message {
        Message::ButtonExit => self.has_exit = true,
        Message::EmergencyStop => {
            self.top.invertor.stop();
            self.low.invertor.stop();
        },
        Message::LowHalfComplectUI(m) => self.low.update(m),
        Message::TopHalfComplectUI(m) => self.top.update(m),
        Message::DozatorUI(m) => {
            let res = self.dozator.update(m, vec![self.logic.digit_o.device().clone()])
                .map(Message::DozatorUI);
            self.logic.update_new_values();
            return res;
        },
        Message::OilStation(m) => {
            self.oil_station.update(m);
            self.logic.update_new_values();
        },
        Message::KlapansUI(m) => {
            self.klapans.update(m);
            self.logic.update_new_values();
        }
        Message::MessageUpdate(m) => return self.modbus_update(m),
        }
        Command::none()
    }
    fn view(&mut self) -> Element<Self::Message> {
//         Text::new("Complex View").into()
        let low = self.low.view()
            .map(Message::LowHalfComplectUI);
        let top = self.top.view()
            .map(Message::TopHalfComplectUI);
            
        let half_2 = Column::new()
            .spacing(20)
            .push(top)
            .push(low);
        let oil_station = self.oil_station.view()
            .map(Message::OilStation);
        let oil_station = Container::new(oil_station);
        let half_2_oil = Row::new().spacing(20)
            .push(half_2.width(Length::FillPortion(10)))
            .push(oil_station.width(Length::FillPortion(10)));

        let dozator = self.dozator.view().map(Message::DozatorUI);
        let klapans = self.klapans.view().map(Message::KlapansUI);
        let col = Column::new()
            .spacing(10)
            .push(dozator)
            .push(half_2_oil)
            .push(klapans)
            .push(Button::new(&mut self.ui.pb_stop, Text::new("Аварийная Остановка!"))
                .on_press(Message::EmergencyStop)
                .style(ui::style::Button::Exit));
//         col.into()

        let txt_status = Text::new(format!("Status: {}", self.txt_status));
        let row_exit = Row::new()
            .push(txt_status)
            .push(Space::with_width(Length::Fill))
            .push(Button::new(&mut self.ui.pb_exit, Text::new("Выход"))
                .on_press(Message::ButtonExit)
                .style(ui::style::Button::Exit));

        let col = col
            .push(Space::with_height(Length::Fill))
            .push(row_exit);

        Container::new(col)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(20)
            .center_x()
            .center_y()
            .into()
    }
}

// modbus update
impl App {
    fn modbus_update(&mut self, message: MessageMudbusUpdate) -> Command<Message> {
        use modbus::UpdateReq;
        match message {
            MessageMudbusUpdate::ModbusUpdate  => {
                self.logic.update();

                self.proccess_values();
                self.proccess_speed();
            },
            MessageMudbusUpdate::ModbusUpdateAsync => {
                let device_futures = self.logic.update_async(UpdateReq::ReadOnly);

                return Command::batch(device_futures.into_iter()
                    .map(|(d, f)| Command::perform(f, move |res| Message::MessageUpdate(
                        MessageMudbusUpdate::ModbusUpdateAsyncAnswerDevice(d.clone(), res)))
                    ));
            },
            MessageMudbusUpdate::ModbusUpdateAsync_Vibro => {
//                 self.proccess_values(true);
                let device_futures = self.logic.update_async(UpdateReq::Vibro);
                return Command::batch(device_futures.into_iter()
                    .map(|(d, f)| Command::perform(f, move |res| Message::MessageUpdate(
                        MessageMudbusUpdate::ModbusUpdateAsyncAnswerDevice(d.clone(), res)))
                    ));
            },
            MessageMudbusUpdate::ModbusConnect => {
                let device_futures = self.logic.reconnect_devices();
                return Command::batch(device_futures.into_iter()
                    .map(|(d, f)| Command::perform(f, move |res| Message::MessageUpdate(
                        MessageMudbusUpdate::ModbusUpdateAsyncAnswerDevice(d.clone(), res)))
                    ));
            },
            MessageMudbusUpdate::ModbusUpdateAsyncAnswer => {
//                 self.proccess_values();
//                 self.proccess_speed();
            },
            MessageMudbusUpdate::ModbusUpdateAsyncAnswerDevice(d, res) => {
    //             dbg!(&d);
                if res.is_ok() {
    //                 println!("Message::ModbusUpdateAsyncAnswerDevice {}", d.name());
                    if !d.is_connect() {
    //                     println!("\tis not connect");
                    } else {
//                         self.proccess_values();
//                         self.proccess_speed();
                    }
                }
            },
//             MessageMudbusUpdate::GraphicUpdate => self.graph.update_svg();
            MessageMudbusUpdate::LogUpdate => {
                self.proccess_values();
                self.proccess_speed();
            },
        }
        Command::none()
    }
    
    fn proccess_values(&mut self) {
        use std::convert::TryFrom;
        let values = self.logic.get_values();
        let mut log_values: Vec<_> = {
            values.iter()
            .map(|(_k, v)| v)
            .filter(|v| v.is_log())
            .filter_map(|v| Some((v, f32::try_from(v.as_ref()).ok()?)))
            .map(|(v, vf)| log::LogValue::new(v.hash(), vf)).collect()
        };
        self.log_values.append(&mut log_values);

        let warn = values.iter().map(|(_,v)| v)
            .map(|v| v.is_error())
            .any(|err| err);
        self.txt_status = if warn {"Ошибка значений"} else {""}.into();
    }
    
    fn proccess_speed(&mut self) {
        use half_complect::SpeedChange::*;
        let is_started_1 = self.low.is_started() | self.top.is_started();
        let changed_low = self.low.proccess_speed();
        match changed_low {
        Some(Up) => self.reset_values(),
        Some(Down) => self.log_save(),
        _ => {},
        };
//         let changed_top = self.top.proccess_speed();
//         let is_started_2 = self.low.is_started() | self.top.is_started();
//         let _change = changed_low.or(changed_top);
//         match (is_started_1, is_started_2) {
//         (false, true) => self.reset_values(),
//         (true, false) => self.log_save(),
//         _ => {}
//         };
    }
    fn log_save(&mut self) {
        if self.log_values.len() > 0 {
            self.log.new_session(&self.log_values);

            let vec_map = vec![
            ("Скорость", "4bd5c4e0a9"),
            ("Ток", "5146ba6795"),
            ("Напряжение", "5369886757"),
            ("Вибродатчик", "Виброскорость дв. М1/value"),
            ("Температура ротора", "Температура ротора Пирометр дв. М1/value"),
            ("Температура статора дв. М1", "Температура статора двигатель М1/value"),
            ("Температура масла на верхн. выходе дв. М1", "Температура масла на верхн. выходе дв. М1/value"),
            ("Температура масла на нижн. выходе дв. М1", "Температура масла на нижн. выходе дв. М1/value"),
            ];
            
            let values: Vec<modbus::ValueArc> = {
            let values = self.logic.get_values().get_values_by_name_ends(&["value", "bit"]);
            let values: HashMap<_,_> = values.iter()
                .filter(|(k,_)| k.matches("/").count()<=1)
                .map(|(k,v)| (k.clone(), v.clone()))
                .collect();
            let values = ModbusValues::from(values);
                values.into()
            };
            let vec_map: Vec<(&str, &str)> = values.iter()
                .filter_map(|v| Some((v.name()?, v.full_name().as_str())) ).collect();
            
            log::Logger::new_table_fields(&self.log_values, 1, vec_map);

            self.log_values = Vec::new();
        }
    }

    fn reset_values(&mut self) {
        self.log_values = Vec::new();
    }
}

use half_complect::{HalfComplect, HalfPart};
mod half_complect {
    use super::*;
    
    pub struct HalfComplect {
        pub invertor: ui::Invertor,
        values: ModbusValues,
        
        values_list: Vec<ui::ValuesList>,
        part: HalfPart,
    }
    pub enum HalfPart {
        Top, // Верхняя часть
        Low, // Нижняя часть
    }

    #[derive(Default)]
    struct UI {
    
    }

    #[derive(Debug, Clone)]
    pub enum Message {
        InvertorUI(ui::invertor::Message),
        UpdateValues,
    }
    
    pub enum SpeedChange {
        Up, 
        Down,
    }

    impl HalfComplect {
//         pub fn new_by_name(values: ModbusValues
        pub fn new(part: HalfPart, values: ModbusValues, invertor: modbus::Invertor) -> Self {
//             dbg!(values.keys());
            let values: HashMap<_,_> = values.iter()
                .filter(|(k,_)| k.matches("/").count()<=1)
                .map(|(k,v)| (k.clone(), v.clone()))
                .collect();
            let values = ModbusValues::from(values);
//             dbg!(values.get_values_by_name_ends(&["value", "bit"]).keys());
            
            HalfComplect {
                invertor: ui::Invertor::new(invertor),
                
                values_list: ui::make_value_lists_start(&values,
                match part {
                HalfPart::Low => map!{BTreeMap,
                        "Температуры" => [
                            "Температура статора",
                            "Температура ротора Пирометр",
                            "Температура масла на верхн. выходе дв. М1",
                            "Температура масла на нижн. выходе дв. М1",
                            "Виброскорость",
                        ]
                    },
                HalfPart::Top => map!{BTreeMap,
                        "Температуры" => [
                            "Температура статора",
                            "Температура ротора Пирометр",
                            "Температура верх подшипника дв. М2",
                            "Температура нижн подшипника дв. М2",
                            "Виброскорость",
                        ]
                    }
                }
                ),
                values: values,
                part: part,
            }
        }
        
        pub fn update(&mut self, message: Message) {
            match message {
            Message::InvertorUI(m) => self.invertor.update(m),
            Message::UpdateValues => {},
            }
        }
        
        pub fn view(&mut self) -> Element<Message> {
//             Text::new("Half Complect View").into()
            let list_value = self.values_list.iter()
                .fold(Column::new().spacing(20), |lst, v| lst.push(v.view()));
            let inv = self.invertor.view().map(Message::InvertorUI);

            let mut column = Column::new().spacing(10);
            column = match self.part {
            HalfPart::Top =>
                column.push(list_value)
                .push(inv),
            HalfPart::Low =>
                column.push(inv)
                .push(list_value)
            };
            column = column.width(Length::Fill);
            column.into()
        }
    }
    
    impl HalfComplect {
        pub fn has_speed(&self) -> bool {
            use std::convert::TryFrom;
            let speed_value = self.invertor.get_hz_out_value();
            let speed_value = f32::try_from(speed_value.as_ref()).unwrap();
            speed_value > 5.0
        }
        
        pub fn is_started(&self) -> bool {
            self.invertor.is_started
        }
//         pub fn has_vibra(&self) -> bool {
//         
//         }
        
        pub fn proccess_speed(&mut self) -> Option<SpeedChange> {
            use std::convert::TryFrom;
            let speed_value = self.invertor.get_hz_out_value();
            let speed_value = f32::try_from(speed_value.as_ref()).unwrap();
            
            let vibra_value = self.values.get_value_arc_starts("Виброскорость").unwrap().value();
            let vibra_value = f32::try_from(vibra_value.as_ref()).unwrap();
                
            if self.invertor.is_started == false && speed_value > 5.0 {
                self.invertor.is_started = true;
// //                 self.reset_values();
                return Some(SpeedChange::Up);
            } else if self.invertor.is_started == true
                    && (speed_value < 2.0 && vibra_value<0.2) {
                self.invertor.is_started = false;
// //                 self.log_save();
                return Some(SpeedChange::Down);
            };
            return None;
        }
    }
}
