use iced::{
    Element, Text, button, Button, slider, Slider,
    Column, Row, Space, Length,
};

use std::sync::Arc;
use super::style;

pub struct Invertor {
    ui: UI,
    pub is_started: bool,
    pub speed: u32,
    device: modbus::Invertor,
}

#[derive(Default)]
struct UI {
    start: ui_button_start::State,
    speed: slider::State,
}

#[derive(Debug, Clone)]
pub enum Message {
    ToggleStart(bool),
    SpeedChanged(u32),
    SetSpeed(u16),
    ButtonStart(ui_button_start::Message),
}

impl Invertor {
    pub fn new(invertor: modbus::Invertor) -> Self {
        Invertor {
            ui: UI::default(),
            is_started: false,
            speed: 0,
            device: invertor,
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
        Message::ButtonStart(message) => self.ui.start.update(message),
        Message::ToggleStart(start) => {
            self.is_started = start;
            self.ui.start = Default::default();
            // Invertor SetSpeed
            // Invertor Start | Stop
            if start {
                self.device.start();
            } else {
                self.device.stop();
            }
//             self.log_save();
        },
        Message::SpeedChanged(speed) => {
            self.speed = speed;
//             dbg!((10*speed)/6);
            self.device.set_speed((10*speed)/6);
        },
        Message::SetSpeed(speed) => {},
        }
    }

    pub fn view(&mut self) -> Element<Message> {
        if self.device.device().is_connect() {
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
        }
    }
}

impl Invertor {
    pub fn get_hz_out_value(&self) -> Arc<modbus::Value> {
        self.device.get_hz_out_value()
    }
    pub fn stop(&self) {
        self.device.stop();
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
