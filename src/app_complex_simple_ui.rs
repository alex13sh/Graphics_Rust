#![deny(unused_must_use)]

use futures::FutureExt;
use iced::{
    Application, executor, Command, window::Mode, Subscription, time,
    Element, Container, Text, button, Button, slider, Slider,
    Column, Row, Space, Length,
    Settings, Clipboard,
};

fn log_init() {
    use simplelog::*;
    use std::fs::File;
    let dt = logger::utils::date_time_to_string_name_short(&logger::utils::date_time_now());
    std::fs::create_dir(logger::get_file_path(&format!("./simplelog/[{}]/", dt))).unwrap();
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
    let conf_dozator = ConfigBuilder::new()
        .add_filter_allow_str("dozator")
        .set_time_format_str("%H:%M:%S%.3f")
        .build();
    let conf_modbus_update_ok = ConfigBuilder::new()
        .add_filter_allow_str("modbus::update::ok")
        .set_time_format_str("%H:%M:%S%.3f")
        .build();
    CombinedLogger::init(
        vec![
//             TermLogger::new(LevelFilter::Warn, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
            WriteLogger::new(LevelFilter::Trace, conf_modbus_update,
                File::create(logger::utils::get_file_path(
                    &format!("simplelog/[{}]/modbus_update.log", dt)
                )).unwrap()
            ),
            WriteLogger::new(LevelFilter::Trace, conf_meln_logic,
                File::create(logger::utils::get_file_path(
                    &format!("simplelog/[{}]/meln_logic.log", dt)
                )).unwrap()
            ),
            WriteLogger::new(LevelFilter::Trace, conf_app,
                File::create(logger::utils::get_file_path(
                    &format!("simplelog/[{}]/app_complex_simple_ui.log", dt)
                )).unwrap()
            ),
            WriteLogger::new(LevelFilter::Trace, conf_app_update,
                File::create(logger::utils::get_file_path(
                    &format!("simplelog/[{}]/app_update.log", dt)
                )).unwrap()
            ),
            WriteLogger::new(LevelFilter::Trace, conf_dozator,
                File::create(logger::utils::get_file_path(
                    &format!("simplelog/[{}]/dozator.log", dt)
                )).unwrap()
            ),
            WriteLogger::new(LevelFilter::Trace, conf_modbus_update_ok,
                File::create(logger::get_file_path(
                    &format!("simplelog/[{}]/modbus_update_ok.log", dt)
                )).unwrap()
            ),
        ]
    ).unwrap();
}

fn main() {
    log_init();
    App::run(Settings::default()).unwrap();
} 

mod ui;
use logger::LogSession;
use ui::style;

use modbus::{ModbusValues, Device, DeviceID, DeviceError, DeviceResult};
use std::collections::HashMap;
// use std::collections::BTreeMap;
use std::sync::Arc;
// use std::path::PathBuf;
use tokio::sync::Mutex;


pub struct App {
    ui: UI,
    has_exit: bool,
    devices: modbus::Devices,
    meln: meln_logic::Meln,
    is_worked: bool,
    txt_status: String,
    devices_disconnect: bool, // Если устройства отключены, показать кнопку reconnect
    
    dvij_is_started: bool,
    klapans: ui::Klapans,
    dozator: ui::Dozator,
    top: HalfComplect,
    low: HalfComplect,
    oil_station: ui::OilStation,
    info_pane: ui::InfoPane,
    
    devices_queue: HashMap<DeviceID, Arc<Device>>,
    log_session: LogSession,
    is_logging: bool,
}



#[derive(Default)]
struct UI {
    pb_exit: button::State,
    pb_stop: button::State,
    pb_log_turn: button::State,
    pb_reconnect: button::State,
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
    
    LoggingTurn(bool),
    DevicesReconnect,
    MessageUpdate(MessageMudbusUpdate),
    MelnMessage(MelnMessage),
    ResultEmpty,
}

#[derive(Debug, Clone)]
pub enum MessageMudbusUpdate {
    UpdateDevice(Arc<Device>), ModbusUpdateAsync, ModbusUpdateAsyncAnswer(Option<DeviceResult>),
    ModbusUpdateAsync_Invertor,
    ModbusConnect, ModbusConnectAnswer(Arc<Device>, DeviceResult),
//     GraphicUpdate,
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
        let devices = modbus::Devices::new();
        let meln = meln_logic::Meln::new(devices.get_values());
        let meln_fut = meln.clone();

        (App {
            ui: Default::default(),
            has_exit: false,
            txt_status: "".into(),
            devices_disconnect: true,

            dvij_is_started: false,
            low: HalfComplect::new(&meln.values.half_bottom),
            top: HalfComplect::new(&meln.values.half_top),
            klapans: ui::Klapans::new(),
            dozator: ui::Dozator::new(),
            oil_station: ui::OilStation::new_by_meln(&meln.values),
            info_pane: ui::InfoPane::new(),
        
            devices: devices,
            meln: meln,
            is_worked: false,

            devices_queue: HashMap::new(),
            log_session: logger::LogSession::new(),
            is_logging: false,
        },
        
            Command::batch(vec![
                async{Message::MessageUpdate(MessageMudbusUpdate::ModbusConnect)}.into(),
                async move {
                    tokio::join!(
                        meln_fut.automation(), 
                        meln_fut.automation_mut()
                    );
                    Message::MessageUpdate(MessageMudbusUpdate::ModbusUpdateAsyncAnswer(None))
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
    fn scale_factor(&self) -> f64 {0.65}

    fn subscription(&self) -> Subscription<Self::Message> {
        let props = &self.meln.properties;
        let interval_update = if self.is_logging {100} else {500};
        Subscription::batch(vec![
            Subscription::batch(vec![
                time::every(std::time::Duration::from_millis(interval_update))
                .map(|_| MessageMudbusUpdate::ModbusUpdateAsync),
//                 time::every(std::time::Duration::from_secs(20))
//                 .map(|_| MessageMudbusUpdate::ModbusConnect),
//                 time::every(std::time::Duration::from_secs(30*60))
//                 .map(|_| MessageMudbusUpdate::ModbusUpdateAsync_Invertor),
                Self::sub_devices(self.devices.iter().cloned()),
            ]).map(Message::MessageUpdate),
            
            Self::sub_meln(&self.meln.properties).map(Message::MelnMessage),
            
            self.dozator.subscription(&props.material.dozator).map(Message::DozatorUI),
            self.klapans.subscription(&props.klapans).map(Message::KlapansUI),
            self.klapans.subscription_vacuum(&props.vacuum).map(Message::KlapansUI),

            if let Some(stream) = self.log_session.get_statistic_low() {
                Subscription::from_recipe(
                    ui::animations::MyStream{name: "statistic_low".into(), stream: stream}
                ).map(ui::info_pane::Message::UpdateInfo)
                .map(Message::InfoPane)
            } else {Subscription::none()},
        ])
    }
    
    fn update(&mut self, message: Self::Message, _clipboard: &mut Clipboard) -> Command<Self::Message> {
        match &message {
        Message::MessageUpdate(_) => {},
        Message::DozatorUI(_) => {
            log::trace!(target: "dozator", "update message:\n\t{:?}", &message);
        }
        _ => log::trace!(target: "app::update", "update message:\n\t{:?}", &message),
        }

        let meln = &self.meln.values;
        match message {
        Message::ButtonExit => self.has_exit = true,
        Message::EmergencyStop => {
            self.meln.values.stop();
        },
        Message::DevicesReconnect => {
        
        },
        Message::LowHalfComplectUI(m) => self.low.update(m, &meln.half_bottom),
        Message::TopHalfComplectUI(m) => self.top.update(m, &meln.half_top),
        Message::DozatorUI(m) => {
            let res = self.dozator.update(m, &meln.material.dozator)
                .map(Message::DozatorUI);
//             self.devices.update_new_values();
            return res;
        },
        Message::OilStation(m) => {
            self.oil_station.update(m, &meln.oil);
//             self.devices.update_new_values();
        },
        Message::KlapansUI(m) => {
            self.klapans.update_material(m.clone(), &meln.material);
            self.klapans.update_vacuum(m.clone(), &meln.vacuum);
            self.klapans.update(m, &meln.klapans);
//             self.devices.update_new_values();
        }
        Message::InfoPane(m) => self.info_pane.update(m),
        Message::MessageUpdate(m) => return self.modbus_update(m),
        Message::MelnMessage(m) => return self.meln_update(m),
        Message::ResultEmpty => {},
        Message::LoggingTurn(enb) => {
            self.is_logging = enb;

            let log_session = &mut self.log_session;
            if enb {
                log_session.start();
//                 let log_session = log_session.make_read().unwrap();
                let file = log_session.make_path_excel("low");
                let f = log_session.write_full();
                let f = async move {
                    f.await;
                    file
                };
                return Command::perform(f, |path| Message::InfoPane(ui::info_pane::Message::UpdateFile(path)));
            } else {
                log_session.stop();
            }
        }
        }
        Command::none()
    }
    fn view(&mut self) -> Element<Self::Message> {
//         Text::new("Complex View").into()
        let ba_1 = true;
        let ba_2 = true;
        let bd_1 = true;
        let bd_2 = true;

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
        let pb_log_turn = Button::new(&mut self.ui.pb_log_turn, Text::new("Логи включены"))
                .style(style::Button::Check{checked: self.is_logging})
                .on_press(Message::LoggingTurn(!self.is_logging));
        let pb_reconnect: Element<_> = if self.devices_disconnect {
            Button::new(&mut self.ui.pb_reconnect, Text::new("Reconnect Devices"))
//                 .on_press(Message::DevicesReconnect)
                .on_press(Message::MessageUpdate(MessageMudbusUpdate::ModbusConnect))
                .style(ui::style::Button::Exit).into()
            } else {
                Text::new("Devices Connected").into()
            };
        let row_exit = Row::new().spacing(20)
            .push(txt_status)
            .push(pb_log_turn)
            .push(pb_reconnect)
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
    fn sub_devices(devices: impl Iterator<Item=Arc<Device>>) -> Subscription<MessageMudbusUpdate> {
        use ui::animations::DeviceUpdate;
        Subscription::batch(
            devices
//                 .filter(|d| d.is_connect())
                .map(|d| Subscription::from_recipe(
                    DeviceUpdate::new(d.clone(), MessageMudbusUpdate::UpdateDevice)
                ))
        )
    }

    fn modbus_update(&mut self, message: MessageMudbusUpdate) -> Command<Message> {
        log::trace!(target: "modbus::update", "modbus_update \n\tmessage: {:?}", &message);
        use modbus::UpdateReq;
        match message {
            MessageMudbusUpdate::UpdateDevice(d) => {
                // Добавить в очередь
                self.devices_queue.insert(d.id().clone(), d);
            },
            MessageMudbusUpdate::ModbusUpdateAsync => {
                
                let f_update_new_values: Vec<_> = self.devices.iter()
                    .cloned().map(Device::update_new_values)
                    .collect();
                    
                // Обновлять устройства из очереди
                let devices = std::mem::take(&mut self.devices_queue);
                let devices_future = async move {
                
                    for f in f_update_new_values {
                        if let Err(err) = f.await {
//                             println!("f_update_new_values err: {:?}", err);
                        }
                    }

                    let mut devices_reconnect = Vec::new();
                    for (_, d) in devices {
                        let d2 = d.clone();
                        let res = async move {
//                             log::trace!(target: "modbus::update::update_new_values", "{:?}", d.id());
                            d.clone().update_new_values().await?;
//                             log::trace!(target: "modbus::update::update_async", "{:?}", d.id());
                            d.clone().update_async(UpdateReq::ReadOnlyOrLogable).await
                        }.await;
                        if let Err(e) = res {
                            println!("Error: {:?} {:?}", &e, d2.id());
                            log::trace!(target: "modbus::update", "[error] {:?} {:?}", &e, d2.id());
                            match e {
                            DeviceError::TimeOut => {
                                devices_reconnect.push(d2);
//                                 self.devices_disconnect = true;
                            },
                            _ => {}
                            }
                        } else if d2.id().id == 2 {
                            log::info!(target: "modbus::update::ok", "Device: {:?}", d2.id());
                        }
                    }
                    
                    let mut res = None;
                    for d in devices_reconnect {
                        if let Err(e) = d.reconnect().await {
                            dbg!(e);
//                             self.devices_disconnect = true;
                            res = Some(Err(DeviceError::ContextNull));
                        }
                    }
                    res
                };

                return Command::perform(devices_future, move |res| Message::MessageUpdate(
                        MessageMudbusUpdate::ModbusUpdateAsyncAnswer(res)));
            },
            MessageMudbusUpdate::ModbusConnect => {
//                 self.save_invertor();
                let device_futures = self.devices.iter().map(|d| (d.clone(), d.clone().connect()));
                return Command::batch(device_futures
                    .map(|(d, f)| Command::perform(f, move |res| Message::MessageUpdate(
                        MessageMudbusUpdate::ModbusConnectAnswer(d.clone(), res)))
                    ));
            },
            MessageMudbusUpdate::ModbusConnectAnswer(d, res) => {
                println!("id: {:?} - {:?}", d.id(), &res);
//                 if res.is_ok() {
//                     self.devices_disconnect = false;
//                 }
                self.devices_disconnect = self.devices.iter()
//                     .filter(|d| d.id().name != "Invertor")
                    .any(|d| !d.is_connect());
            },
            MessageMudbusUpdate::ModbusUpdateAsync_Invertor => {
//                 let d = self.devices.invertor_2.device();
//                 Self::save_invertor(d.values_map());
//                 let f = d.update_async(UpdateReq::All);
//                 return Command::perform(f, move |res| Message::MessageUpdate(
//                         MessageMudbusUpdate::ModbusUpdateAsyncAnswerDevice(d.clone(), res)));
            },
            MessageMudbusUpdate::ModbusUpdateAsyncAnswer(res) => {
                self.meln.properties.update_property(&self.meln.values);
                if self.is_logging {
                    self.proccess_values();
                }
                if let Some(res) = res {
                    self.devices_disconnect = !res.is_ok();
                }
            },
//             MessageMudbusUpdate::GraphicUpdate => self.graph.update_svg();
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
//             if is_started {
//                 self.reset_values();
//                 self.is_worked = true;
//             } else {
//                 self.is_worked = false;
//                 let f = Self::log_save(std::mem::take(&mut self.log_values));
//                 return Command::perform(f, |res| Message::InfoPane(ui::info_pane::Message::UpdateInfo(res)));
//             }
            return async move{Message::LoggingTurn(is_started)}.into();
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
        let values = self.devices.get_values();
        if self.is_logging {
            let mut log_values: Vec<_> = {
                values.iter()
                .map(|(_k, v)| v)
                .filter(|v| v.is_log())
                .filter_map(|v| Some((v, f32::try_from(v.as_ref()).ok()?)))
                .map(|(v, vf)| logger::Value {
                    device_id: v.id().device_id.clone(),
                    device_name: v.id().device_name.clone(),
                    sensor_name: v.id().sensor_name.clone(),
                    value_name: v.id().value_name.clone(),
                    value_u32: v.value(),
                    value_f32: vf,
                }).collect() // Избавиться от hash
            };
            // Разницу записывать
            self.log_session.push_values(log_values.into_boxed_slice());
        } 

//         let warn = values.iter().map(|(_,v)| v)
//             .map(|v| v.is_error())
//             .any(|err| err);
//         self.txt_status = if warn {"Ошибка значений"} else {""}.into();
    }

//     fn save_invertor(invertor_values: &ModbusValues) {
//         let dt = logger::date_time_now();
//         log::trace!("save_invertor date_time: {:?}", &dt);
//         let dt = logger::date_time_to_string_name_short(&dt);
//         let path = logger::get_file_path("tables/log/").join(dt).with_extension(".csv");
//         log::trace!("path: {:?}", &path);
//         let parametrs: Vec<_> = invertor_values.iter_values()
//             .map(|(adr, v, id)| logger::InvertorParametr {
//                 address: format!("({}, {})", adr/256, adr%256),
//                 value: v,
// //                 name: id.to_string(),
//                 name: id.sensor_name.clone(),
//             }).collect();
//         if let Err(e) = logger::csv::write_invertor_parametrs(&path, parametrs) {
//             log::error!("logger::csv::write_invertor_parametrs: {:?}", e);
//         }
//     }

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
                            values.invertor.индикация_мощности.clone(),
                            values.invertor.get_amper_out_value().into(),
                            values.invertor.get_hz_out_value().into(),
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
