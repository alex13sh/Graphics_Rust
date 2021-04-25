use iced::{
    Application, executor, Command, Subscription, time,
    Element, Container, Text, button, Button, slider, Slider,
    Column, Row, Space, Length,
    Settings, Clipboard,
};

fn main() {
    App::run(Settings::default());
}


use modbus::{Value, ModbusValues, ValueError, Device, DeviceError };

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::Arc;

pub struct App {
    ui: UI,
    
    logic: meln_logic::init::Complect,
    
    klapans: [bool; 2],
    shim_hz: u32,
    speed: u32,
}

#[derive(Default)]
struct UI {
    klapan: [button::State; 3],
    shim_hz: slider::State,
    speed: slider::State,
    
}

#[derive(Debug, Clone)]
pub enum Message {
    ModbusUpdate, ModbusUpdateAsync, ModbusUpdateAsyncAnswer,
    ModbusUpdateAsyncAnswerDevice(Arc<Device>, Result<(), DeviceError>),
    
    ToggleKlapan(usize, bool),
    
    ShimHzChanged(u32),
    SetShimHz(u16),
    
    SpeedChanged(u32),
}

impl Application for App {
    type Executor = executor::Default;
    type Flags = ();
    type Message = Message;
    
    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        let logic = meln_logic::init::Complect::new();
//         logic.init_values(&values);
                        
        (
            Self {
                ui: UI::default(),
                
                shim_hz: 0,
                speed: 0,
                
                klapans: [false; 2],

                logic: logic,
                
            },
            Command::none()
        )
    }
    
    fn title(&self) -> String {
        String::from("Test Klapans")
    }
    fn subscription(&self) -> Subscription<Self::Message> {
        Subscription::batch(vec![
            time::every(std::time::Duration::from_millis(2000))
            .map(|_| Message::ModbusUpdateAsync),
        ])
    }
    fn update(&mut self, message: Self::Message, _clipboard: &mut Clipboard) -> Command<Self::Message> {
    
        match message {
        
        Message::ModbusUpdateAsync => {
            let device_futures = self.logic.update_async();
                
            return Command::batch(device_futures.into_iter()
                .map(|(d, f)| Command::perform(f, move |res| Message::ModbusUpdateAsyncAnswerDevice(d.clone(), res)))
                );
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
            } else {
                dbg!(d.name(), &res);
            }
        },
        
        Message::ToggleKlapan(ind, enb) => {
            
            self.klapans[ind as usize] = enb;
            self.klapans[1-ind as usize] = false;
            match ind {
            0 => {
                self.logic.set_bit("Клапан насоса М5 вакуум", false).unwrap();
                self.logic.set_bit("Клапан насоса М6 вакуум", enb).unwrap();
                self.logic.set_bit("Клапан напуска воздуха", enb).unwrap();
            }, 1 => {
                self.logic.set_bit("Клапан насоса М5 вакуум", enb).unwrap();
                self.logic.set_bit("Клапан насоса М6 вакуум", false).unwrap();
                self.logic.set_bit("Клапан напуска воздуха", false).unwrap();
            }, _ => {}
            }
            self.logic.update_new_values();
        },
        Message::ShimHzChanged(hz) => {
            self.shim_hz = hz;
//             dbg!((10*speed)/6);
//             self.logic.invertor.set_speed((10*speed)/6);
        },
//         Message::SetSpeed(speed) => {},
        _ => {}
        };
        Command::none()
    }
    fn view(&mut self) -> Element<Self::Message> {
//         let content = Text::new("Пустое окно");
            
        
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
                
                controls_klapan.into()
            } else {Element::from(Text::new("Цифровой модуль ОВЕН не подключен"))};
            
            let invertor: Element<_> = if self.logic.invertor.device().is_connect() {
                
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
                    )
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
