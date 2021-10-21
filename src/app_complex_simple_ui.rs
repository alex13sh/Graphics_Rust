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
    meln: meln_logic::Meln,
    txt_status: String,
    
    dvij_is_started: bool,
    klapans: ui::Klapans,
    dozator: ui::Dozator,
    top: HalfComplect,
    low: HalfComplect,
    oil_station: ui::OilStation,
    info_pane: ui::InfoPane,
    
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
    InfoPane(ui::info_pane::Message),
    
    MessageUpdate(MessageMudbusUpdate),
    MelnMessage(MelnMessage),
}

#[derive(Debug, Clone)]
pub enum MessageMudbusUpdate {
    ModbusUpdate, ModbusUpdateAsyncAnswer,
    ModbusUpdateAsync, ModbusUpdateAsync_Vibro, ModbusUpdateAsync_Invertor,
    ModbusConnect, ModbusConnectAnswer(Arc<Device>, Result<(), DeviceError>),
    ModbusUpdateAsyncAnswerDevice(Arc<Device>, Result<(), DeviceError>),
//     GraphicUpdate,
    LogUpdate,
}

#[derive(Debug, Clone)]
pub enum MelnMessage {
    IsStartedChanged(bool),
    
}

impl Application for App {
    type Executor = executor::Default;
    type Flags = ();
    type Message = Message;
    
    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        let logic = meln_logic::init::Complect::new();
        let meln = meln_logic::Meln::new(logic.get_values());
        let meln_fut = meln.clone();

        let values_1 = logic.get_values().get_values_by_name_contains(&["М1"]);
        let values_2 = logic.get_values().get_values_by_name_contains(&["М2"]);
        
        (App {
            ui: Default::default(),
            has_exit: false,
            txt_status: "".into(),

            dvij_is_started: false,
            low: HalfComplect::new(&meln.values.half_bottom),
            top: HalfComplect::new(&meln.values.half_top),
            klapans: ui::Klapans::new(),
            dozator: ui::Dozator::new(),
            oil_station: ui::OilStation::new_by_meln(&meln.values),
            info_pane: ui::InfoPane::new(),
        
            logic: logic,
            meln: meln,
            log: log::Logger::open_csv(),
            log_values: Vec::new(),
        },
        
            Command::batch(vec![
                async{Message::MessageUpdate(MessageMudbusUpdate::ModbusConnect)}.into(),
                async move {
                    tokio::join!(
                        meln_fut.automation(), 
                        meln_fut.automation_mut()
                    );
                    Message::MessageUpdate(MessageMudbusUpdate::ModbusUpdateAsyncAnswer)
                }.into()
            ])
        )
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
        let interval_update = if self.is_worked() {100} else {1000};
        let interval_log = if self.is_worked() {100} else {10000};
        Subscription::batch(vec![
            Subscription::batch(vec![
                time::every(std::time::Duration::from_millis(interval_update))
                .map(|_| MessageMudbusUpdate::ModbusUpdateAsync),
//                 time::every(std::time::Duration::from_millis(100))
//                 .map(|_| MessageMudbusUpdate::ModbusUpdateAsync_Vibro),
                time::every(std::time::Duration::from_millis(5000))
                .map(|_| MessageMudbusUpdate::ModbusConnect),
                time::every(std::time::Duration::from_millis(5000))
                .map(|_| MessageMudbusUpdate::ModbusUpdateAsync_Invertor),
                time::every(std::time::Duration::from_millis(interval_log))
                .map(|_| MessageMudbusUpdate::LogUpdate),

            ]).map(Message::MessageUpdate),
            self.dozator.subscription().map(Message::DozatorUI),
        ])
    }
    
    fn update(&mut self, message: Self::Message, _clipboard: &mut Clipboard) -> Command<Self::Message> {
        let meln = &self.meln.values;
        match message {
        Message::ButtonExit => self.has_exit = true,
        Message::EmergencyStop => {
            self.meln.values.stop();
        },
        Message::LowHalfComplectUI(m) => self.low.update(m, &meln.half_bottom),
        Message::TopHalfComplectUI(m) => self.top.update(m, &meln.half_top),
        Message::DozatorUI(m) => {
            let res = self.dozator.update(m, &meln.material.dozator)
                .map(Message::DozatorUI);
            self.logic.update_new_values();
            return res;
        },
        Message::OilStation(m) => {
            self.oil_station.update(m, &meln.oil);
            self.logic.update_new_values();
        },
        Message::KlapansUI(m) => {
            self.klapans.update_material(m.clone(), &meln.material);
            self.klapans.update_vacuum(m.clone(), &meln.vacuum);
            self.klapans.update(m, &meln.klapans);
            self.logic.update_new_values();
        }
        Message::InfoPane(m) => self.info_pane.update(m),
        Message::MessageUpdate(m) => return self.modbus_update(m),
        Message::MelnMessage(m) => self.meln_update(m),
        }
        Command::none()
    }
    fn view(&mut self) -> Element<Self::Message> {
//         Text::new("Complex View").into()
        let ba_1 = self.logic.owen_analog_1.is_connect();
        let ba_2 = self.logic.owen_analog_2.is_connect();
        let bd_1 = self.logic.digit_i.device().is_connect();
        let bd_2 = self.logic.digit_o.device().is_connect();

        let low = self.low.view()
            .map(Message::LowHalfComplectUI);
        let top = self.top.view()
            .map(Message::TopHalfComplectUI);
            
        let half_2 = Column::new()
            .spacing(20)
            .push(top)
            .push(low);
        let oil_station = if bd_2  {
            self.oil_station.view()
            .map(Message::OilStation)
        } else {Text::new("Отключен модуль с клапанами").into()};
//         let oil_station = Container::new(oil_station);
        let info_pane = self.info_pane.view()
            .map(Message::InfoPane);
        let right_column = Column::new().spacing(20)
            .push(oil_station)
            .push(info_pane);
            
        let half_2_oil = Row::new().spacing(20)
            .push(half_2.width(Length::FillPortion(10)))
            .push(right_column.width(Length::FillPortion(10)));

        let dozator = self.dozator.view().map(Message::DozatorUI);
        let klapans = if bd_2 /*&& !self.dvij_is_started*/ {self.klapans.view().map(Message::KlapansUI)} else {Text::new("Отключен модуль с клапанами").into()};
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
                println!("MessageMudbusUpdate::ModbusConnect ");
//                 self.save_invertor();
                let device_futures = self.logic.reconnect_devices();
                return Command::batch(device_futures.into_iter()
                    .map(|(d, f)| Command::perform(f, move |res| Message::MessageUpdate(
                        MessageMudbusUpdate::ModbusConnectAnswer(d.clone(), res)))
                    ));
            },
            MessageMudbusUpdate::ModbusConnectAnswer(d, res) => {
                let dc = d.clone();
                let f = async move {dc.update_async(UpdateReq::All).await};
                return Command::perform(f, move |res| Message::MessageUpdate(
                        MessageMudbusUpdate::ModbusUpdateAsyncAnswerDevice(d.clone(), res)));
            },
            MessageMudbusUpdate::ModbusUpdateAsync_Invertor => {
                let d = self.logic.invertor_1.device();
                let dc = d.clone();
                let f = async move {dc.update_async(UpdateReq::All).await};
                return Command::perform(f, move |res| Message::MessageUpdate(
                        MessageMudbusUpdate::ModbusUpdateAsyncAnswerDevice(d.clone(), res)));
            },
            MessageMudbusUpdate::ModbusUpdateAsyncAnswer => {
//                 self.proccess_values();
//                 self.proccess_speed();
            },
            MessageMudbusUpdate::ModbusUpdateAsyncAnswerDevice(d, res) => {
    //             dbg!(&d);
                if res.is_ok() {
    //                 println!("Message::ModbusUpdateAsyncAnswerDevice {}", d.name());
                }
            },
//             MessageMudbusUpdate::GraphicUpdate => self.graph.update_svg();
            MessageMudbusUpdate::LogUpdate => {
                self.proccess_values();
            },
        }
        Command::none()
    }
    
    fn meln_update(&mut self, message: MelnMessage) {
        use MelnMessage::*;
        match message {
        IsStartedChanged(is_started) => {
            self.dvij_is_started = is_started;
            if is_started {
                self.reset_values();
            } else {
                self.log_save();
            }
        }
        }
    }
    
    fn proccess_values(&mut self) {
        use std::convert::TryFrom;
        let values = self.logic.get_values();
        if self.is_started() {
            let mut log_values: Vec<_> = {
                values.iter()
                .map(|(_k, v)| v)
                .filter(|v| v.is_log())
                .filter_map(|v| Some((v, f32::try_from(v.as_ref()).ok()?)))
                .map(|(v, vf)| log::LogValue::new(v.hash(), vf)).collect()
            };
            self.log_values.append(&mut log_values);
        }

        let warn = values.iter().map(|(_,v)| v)
            .map(|v| v.is_error())
            .any(|err| err);
        self.txt_status = if warn {"Ошибка значений"} else {""}.into();
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
            ("Разрежение воздуха в системе", "Разрежение воздуха в системе/value"),
            ];
            
//             let values: Vec<modbus::ValueArc> = {
//             let values = self.logic.get_values().get_values_by_name_ends(&["value", "bit"]);
//             let values: HashMap<_,_> = values.iter()
//                 .filter(|(k,_)| k.matches("/").count()<=1)
//                 .map(|(k,v)| (k.clone(), v.clone()))
//                 .collect();
//             let values = ModbusValues::from(values);
//                 values.into()
//             };
//             let vec_map: Vec<(&str, &str)> = values.iter()
//                 .filter_map(|v| Some((v.name()?, v.full_name().as_str())) ).collect();
            
            let values = std::mem::take(&mut self.log_values);
            let res = log::Logger::new_table_fields(values, 1, vec_map);
            
            self.save_invertor();
            if let Some((stat, path)) = res {
                self.info_pane.set_info(stat);
                self.info_pane.set_file_path(path);
            }
        }
    }

    fn save_invertor(&self) {
        let dt = log::date_time_now();
        dbg!(&dt);
        let dt = log::date_time_to_string_name_short(&dt);
        let path = log::get_file_path("tables/log/").join(dt).with_extension(".csv");
        dbg!(&path);
        let parametrs: Vec<_> = self.logic.invertor_1.device().values_map()
            .iter_values().map(|(adr, v, n)| log::InvertorParametr {
                address: format!("({}, {})", adr/256, adr%256),
                value: v,
                name: n,
            }).collect();
        if let Err(e) = log::csv::write_invertor_parametrs(&path, parametrs) {
            dbg!(e);
        }
    }

    fn reset_values(&mut self) {
        self.log_values = Vec::new();
    }
    fn is_started(&self) -> bool {
        self.meln.properties.is_started.get()
    }
    fn is_worked(&self) -> bool {
        self.meln.properties.is_worked.get()
    }
}

use half_complect::{HalfComplect};
mod half_complect {
    use super::*;
    pub use meln_logic::values::HalfPart;
    
    pub struct HalfComplect {
        pub invertor: ui::Invertor,
        
        values_list: Vec<ui::ValuesList>,
        part: HalfPart,
    }

    #[derive(Default)]
    struct UI {
    
    }

    #[derive(Debug, Clone)]
    pub enum Message {
        InvertorUI(ui::invertor::Message),
        UpdateValues,
    }

    impl HalfComplect {
        pub fn new(values: &meln_logic::values::HalfMeln) -> Self {

            HalfComplect {
                invertor: ui::Invertor::new(),
                
                values_list: vec![
                    ui::ValuesList {
                        name: "Температуры".into(),
                        values: vec![
                            values.motor.температура_статора.clone(),
                            values.motor.температура_ротора.clone(),
                            
                            // "Температура масла на верхн. выходе дв. М1",
                            // "Температура верх подшипника дв. М2",
                            values.температура_верх.clone(),
                            // "Температура масла на нижн. выходе дв. М1",
                            // "Температура нижн подшипника дв. М2",
                            values.температура_нижн.clone(),
                            
                            values.vibro.clone(),
                            values.invertor.get_amper_out_value().into(),
                            values.invertor.get_speed_out_value().into(),
                        ],
                    }
                ],
                
                part: values.get_part(),
            }
        }
        
        pub fn update(&mut self, message: Message, values: &meln_logic::values::HalfMeln) {
            match message {
            Message::InvertorUI(m) => self.invertor.update(m, &values.invertor),
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
}
