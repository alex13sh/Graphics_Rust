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
    
    klapans: ui::Klapans,
    dozator: ui::Dozator,
    top: HalfComplect,
    low: HalfComplect,
    
    log: log::Logger,
    log_values: Vec<log::LogValue>,
}



#[derive(Default)]
struct UI {
    pb_exit: button::State,
}

#[derive(Debug, Clone)]
pub enum Message {
    ButtonExit,
    
    TopHalfComplectUI(half_complect::Message),
    LowHalfComplectUI(half_complect::Message),
    
    DozatorUI(ui::dozator::Message),
    KlapansUI(ui::klapans::Message),
    
    MessageUpdate(MessageMudbusUpdate),
    
}

#[derive(Debug, Clone)]
pub enum MessageMudbusUpdate {
    ModbusUpdate, ModbusUpdateAsync, ModbusUpdateAsyncAnswer,
    ModbusUpdateAsyncAnswerDevice(Arc<Device>, Result<(), DeviceError>),
//     GraphicUpdate,
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
            
            low: HalfComplect::new(values_1, logic.invertor_1.clone()),
            top: HalfComplect::new(values_2, logic.invertor_2.clone()),
            klapans: ui::Klapans::new(logic.digit_o.device().values_map()
                //.get_values_by_name_starts(&["Клапан 24В", "Клапан 2", "Насос"])
                .clone()),
            dozator: ui::Dozator::new(logic.dozator.clone()),
        
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
            time::every(std::time::Duration::from_millis(500))
            .map(|_| MessageMudbusUpdate::ModbusUpdateAsync),
//             time::every(std::time::Duration::from_millis(500))
//             .map(|_| MessageMudbusUpdate::GraphicUpdate),
        ]).map(Message::MessageUpdate)
    }
    
    fn update(&mut self, message: Self::Message, _clipboard: &mut Clipboard) -> Command<Self::Message> {
    
        match message {
        Message::ButtonExit => self.has_exit = true,
        Message::LowHalfComplectUI(m) => self.low.update(m),
        Message::TopHalfComplectUI(m) => self.top.update(m),
        Message::DozatorUI(m) => {
            let res = self.dozator.update(m, vec![self.logic.digit_o.device().clone()])
                .map(Message::DozatorUI);
//             self.logic.update_new_values();
            return res;
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
            
        let half_2 = Row::new()
            .spacing(20)
            .push(low)
            .push(top);
        let dozator = self.dozator.view().map(Message::DozatorUI);
        let klapans = self.klapans.view().map(Message::KlapansUI);
        let col = Column::new()
            .spacing(10)
            .push(half_2)
            .push(dozator)
            .push(klapans)
            .push(Button::new(&mut self.ui.pb_exit, Text::new("Выход"))
                .on_press(Message::ButtonExit)
                .style(ui::style::Button::Exit));
//         col.into()
        let col = col.push(Space::with_height(Length::Fill));
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
                self.proccess_values();
                self.proccess_speed();
            },
            MessageMudbusUpdate::ModbusUpdateAsyncAnswerDevice(d, res) => {
    //             dbg!(&d);
                if res.is_ok() {
    //                 println!("Message::ModbusUpdateAsyncAnswerDevice {}", d.name());
                    if !d.is_connect() {
    //                     println!("\tis not connect");
                    } else {
                        self.proccess_values();
                        self.proccess_speed();
                    }
                }
            },
//             MessageMudbusUpdate::GraphicUpdate => {
//                 self.graph.update_svg();

//                 self.proccess_values();
//                 self.proccess_speed();

//             },
        }
        Command::none()
    }
    
    fn proccess_values(&mut self) {
        use std::convert::TryFrom;
        let values = self.logic.get_values();
        let mut log_values: Vec<_> = {
            values.iter()
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
            
        if self.low.invertor.is_started == false && speed_value > 5.0 {
            self.low.invertor.is_started = true;
            self.reset_values();
        } else if self.low.invertor.is_started == true
                && (speed_value < 2.0 && vibra_value<0.2) {
            self.low.invertor.is_started = false;
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
    }
}

use half_complect::HalfComplect;
mod half_complect {
    use super::*;
    
    pub struct HalfComplect {
        pub invertor: ui::Invertor,
        values: ModbusValues,
        
        values_list: Vec<ui::ValuesList>,
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
//         pub fn new_by_name(values: ModbusValues
        pub fn new(values: ModbusValues, invertor: modbus::Invertor) -> Self {
//             dbg!(values.keys());
            let values: HashMap<_,_> = values.iter()
                .filter(|(k,_)| k.matches("/").count()<=1)
                .map(|(k,v)| (k.clone(), v.clone()))
                .collect();
            let values = ModbusValues::from(values);
            dbg!(values.keys());
            
            HalfComplect {
                invertor: ui::Invertor::new(invertor),
                
                values_list: ui::make_value_lists_start(&values, map!{BTreeMap,
                    "Температуры" => [
                        "Температура статора",
                        "Температура ротора Пирометр",
                        "Температура масла на выходе",
                        "Температура подшипника",
                    ]
                }),
                values: values,
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
            let list_value = list_value.push(inv)
                .width(Length::Fill);
            list_value.into()
        }
    }
}
