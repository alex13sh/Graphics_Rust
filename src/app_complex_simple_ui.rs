use iced::{
    Application, executor, Command, window::Mode, Subscription, time,
    Element, Container, Text, button, Button, slider, Slider,
    Column, Row, Space, Length,
    Settings, Clipboard,
};

fn log_init() {
    use simplelog::*;
    use std::fs::File;
    let dt = logger::date_time_to_string_name_short(&logger::date_time_now());
    let conf_modbus_update = ConfigBuilder::new()
//         .add_filter_allow_str("app_complex_simple_ui")
        .add_filter_allow_str("modbus::update")
        .add_filter_allow_str("modbus::modbus_context")
        .set_time_format_str("%H:%M:%S%.3f")
        .build();
    let conf_meln_logic = ConfigBuilder::new()
        .add_filter_allow_str("meln_logic")
        .set_time_format_str("%H:%M:%S%.3f")
        .build();
    let conf_app = ConfigBuilder::new()
        .add_filter_allow_str("app_complex_simple_ui")
        .set_time_format_str("%H:%M:%S%.3f")
        .build();
    let conf_app_update = ConfigBuilder::new()
        .add_filter_allow_str("app::update")
        .set_time_format_str("%H:%M:%S%.3f")
        .build();
    CombinedLogger::init(
        vec![
//             TermLogger::new(LevelFilter::Warn, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
            WriteLogger::new(LevelFilter::Trace, conf_modbus_update,
                File::create(logger::get_file_path(
                    &format!("simplelog/[{}] modbus_update.log", dt)
                )).unwrap()
            ),
            WriteLogger::new(LevelFilter::Trace, conf_meln_logic,
                File::create(logger::get_file_path(
                    &format!("simplelog/[{}] meln_logic.log", dt)
                )).unwrap()
            ),
            WriteLogger::new(LevelFilter::Trace, conf_app,
                File::create(logger::get_file_path(
                    &format!("simplelog/[{}] app_complex_simple_ui.log", dt)
                )).unwrap()
            ),
            WriteLogger::new(LevelFilter::Trace, conf_app_update,
                File::create(logger::get_file_path(
                    &format!("simplelog/[{}] app_update.log", dt)
                )).unwrap()
            ),
        ]
    ).unwrap();
}

fn main() {
    log_init();
    App::run(Settings::default());
} 

mod ui;

use modbus::{ModbusValues, Device, DeviceError};
use std::collections::HashMap;
use std::collections::BTreeMap;
use std::sync::Arc;
use std::path::PathBuf;


pub struct App {
    ui: UI,
    has_exit: bool,
    logic: meln_logic::init::Complect,
    meln: meln_logic::Meln,
    is_worked: bool,
    txt_status: String,
    
    dvij_is_started: bool,
    klapans: ui::Klapans,
    dozator: ui::Dozator,
    top: HalfComplect,
    low: HalfComplect,
    oil_station: ui::OilStation,
    info_pane: ui::InfoPane,
    
    log_values: Vec<logger::LogValue>,
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
//     InfoPaneUpdate(Option<(logger::structs::TableState, PathBuf)>),
    
    MessageUpdate(MessageMudbusUpdate),
    MelnMessage(MelnMessage),
    ResultEmpty,
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
    IsWorkedChanged(bool),
    OilMotorChanged(bool),
    NextStep(meln_logic::watcher::MelnStep),
}

impl Application for App {
    type Executor = executor::Default;
    type Flags = ();
    type Message = Message;
    
    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        let logic = meln_logic::init::Complect::new();
        let meln = meln_logic::Meln::new(logic.get_values());
        let meln_fut = meln.clone();

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
            is_worked: false,
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
    fn scale_factor(&self) -> f64 {0.45}

    fn subscription(&self) -> Subscription<Self::Message> {
        let props = &self.meln.properties;
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
            
            Self::sub_meln(&self.meln.properties).map(Message::MelnMessage),
            
            self.dozator.subscription(&props.material.dozator).map(Message::DozatorUI),
            self.klapans.subscription(&props.klapans).map(Message::KlapansUI),
            self.klapans.subscription_vacuum(&props.vacuum).map(Message::KlapansUI),
        ])
    }
    
    fn update(&mut self, message: Self::Message, _clipboard: &mut Clipboard) -> Command<Self::Message> {
        if let Message::MessageUpdate(_) = &message {

        } else {
            log::trace!(target: "app::update", "update message:\n\t{:?}", &message);
        }
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
        Message::MelnMessage(m) => return self.meln_update(m),
        Message::ResultEmpty => {},
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

        let txt_status = Text::new(format!("Step: {}", self.txt_status));
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
        log::trace!(target: "modbus::update", "modbus_update \n\tmessage: {:?}", &message);
        use modbus::UpdateReq;
        match message {
            MessageMudbusUpdate::ModbusUpdate  => {
                self.logic.update();

                self.proccess_values();
            },
            MessageMudbusUpdate::ModbusUpdateAsync => {
                self.meln.properties.update_property(&self.meln.values);
                
                let device_futures = self.logic.update_async(UpdateReq::ReadOnlyOrLogable);

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
                if res.is_ok() {
                    self.meln.properties.update_property(&self.meln.values);
                }
            },
//             MessageMudbusUpdate::GraphicUpdate => self.graph.update_svg();
            MessageMudbusUpdate::LogUpdate => {
                self.proccess_values();
            },
        }
        Command::none()
    }
    
    fn sub_meln(props: &meln_logic::watcher::Meln) -> Subscription<MelnMessage> {
        use ui::animations::PropertyAnimation;
        Subscription::batch(vec![
            Subscription::from_recipe(
                PropertyAnimation::new("IsStarted", props.is_started.subscribe())
            ).map(MelnMessage::IsStartedChanged),
            Subscription::from_recipe(
                PropertyAnimation::new("IsWorked", props.is_worked.subscribe())
            ).map(MelnMessage::IsWorkedChanged),
            Subscription::from_recipe(
                PropertyAnimation::new("Steps", props.step.subscribe())
            ).map(MelnMessage::NextStep),

            Subscription::from_recipe(
                PropertyAnimation::new("OilMotor", props.oil.motor.subscribe())
            ).map(MelnMessage::OilMotorChanged),
        ])
    }

    fn meln_update(&mut self, message: MelnMessage) -> Command<Message> {
        log::trace!(target: "meln_logic::update", "meln_update:\n\t{:?}", &message);

        use MelnMessage::*;
        match message {
        IsStartedChanged(is_started) => {
            self.dvij_is_started = is_started;
            if is_started {
                self.reset_values();
            } else {
                let f = Self::log_save(std::mem::take(&mut self.log_values));
                return Command::perform(f, |res| Message::InfoPane(ui::info_pane::Message::UpdateInfo(res)));
            }
        }
        IsWorkedChanged(enb) => self.is_worked = enb,
        NextStep(step) => {
            self.txt_status = format!("{:?}",step);
        }

        OilMotorChanged(_) => {}
        }
        Command::none()
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
                .map(|(v, vf)| logger::LogValue::new(v.hash(), vf)).collect() // Избавиться от hash
            };
            // Разницу записывать
            self.log_values.append(&mut log_values);
        }

//         let warn = values.iter().map(|(_,v)| v)
//             .map(|v| v.is_error())
//             .any(|err| err);
//         self.txt_status = if warn {"Ошибка значений"} else {""}.into();
    }

    async fn log_save(values: Vec<logger::LogValue>) -> Option<(logger::structs::TableState, PathBuf)> {
        log::trace!("log_save: values len: {}", values.len());
        if values.is_empty() { return None; }
            
        logger::new_csv_raw(&values);

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
        
        let res = logger::new_table_fields(values, 1, vec_map);
        return res;
    }

    fn save_invertor(invertor_values: &ModbusValues) {
        let dt = logger::date_time_now();
        log::trace!("save_invertor date_time: {:?}", &dt);
        let dt = logger::date_time_to_string_name_short(&dt);
        let path = logger::get_file_path("tables/log/").join(dt).with_extension(".csv");
        log::trace!("path: {:?}", &path);
        let parametrs: Vec<_> = invertor_values.iter_values()
            .map(|(adr, v, n)| logger::InvertorParametr {
                address: format!("({}, {})", adr/256, adr%256),
                value: v,
                name: n,
            }).collect();
        if let Err(e) = logger::csv::write_invertor_parametrs(&path, parametrs) {
            log::error!("logger::csv::write_invertor_parametrs: {:?}", e);
        }
    }

    fn reset_values(&mut self) {
        self.log_values = Vec::new();
    }
    fn is_started(&self) -> bool {
        self.meln.properties.is_started.get()
    }
    fn is_worked(&self) -> bool {
        self.is_worked
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
