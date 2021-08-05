use super::style;
use super::animations::LinerAnimation;

use iced::{
    Element, Text, button, Button, slider, Slider,
    Column, Row, Space, Length,
    Command,
};

use std::sync::Arc;

pub struct Dozator {
    ui: UI,
    shim_hz_ui: i32, shim_hz_cur: i32, shim_hz_new: i32,
    klapan_enb: bool,
    device: meln_logic::devices::Dozator,
//     anim: LinerAnimation,
}

#[derive(Default)]
struct UI {
    shim_hz: slider::State,
    pb_klapan_enb: button::State,
}


#[derive(Debug, Clone)]
pub enum Message {
    ShimHzChanged(i32),
    SetShimHz, SetShimHzFinished,
    ToggleKlapan(bool),
    AnimationPos(super::animations::Progress),
}

impl Dozator {
    pub fn new(device: meln_logic::devices::Dozator) -> Self {
        Dozator {
            ui: UI::default(),
            shim_hz_ui: 0, shim_hz_cur: 0, shim_hz_new: 0,
            klapan_enb: false,
            device: device,
//             anim: LinerAnimation::new(0.0, 20).duration(5_000),
        }
    }

    pub fn subscription(&self) -> iced::Subscription<Message> {
//         if self.shim_hz_cur != self.shim_hz_new {
            iced::Subscription::from_recipe(
                LinerAnimation::from_to(self.shim_hz_cur as f32, self.shim_hz_new as f32)
                    .steps(20).duration(5_000)
            ).map(Message::AnimationPos)
//         } else {
//             iced::Subscription::none()
//         }
    }

    pub fn update(&mut self, message: Message, devices: Vec<Arc<modbus::Device>>)  -> Command<Message> {
        match message {
        Message::ShimHzChanged(hz) => self.shim_hz_ui = hz,
        Message::SetShimHz => {
            println!("Set HZ: {}", self.shim_hz_ui);
            self.shim_hz_new = self.shim_hz_ui;
//             self.anim.set_from_to(self.shim_hz_cur as f32, self.shim_hz_new as f32);
        },
        Message::ToggleKlapan(enb) => {
            self.klapan_enb = enb;
        },
        Message::AnimationPos(super::animations::Progress::Value(value)) => {
            self.shim_hz_ui = value as i32;
            self.shim_hz_cur = self.shim_hz_ui;
//             dbg!(value);
            self.device.set_value(self.shim_hz_cur);
        },
        Message::AnimationPos(super::animations::Progress::Finished) => {
//            self.anim.stop();
        },
        _ => {},
        }
        Command::none()
    }
    pub fn view(&mut self) -> Element<Message> {
        let slider = {
            let slider = Slider::new(
                &mut self.ui.shim_hz,
                -20..=20,
                self.shim_hz_ui,
                Message::ShimHzChanged
            )
            .on_release(Message::SetShimHz)
            .step(1);

            let pb = Button::new(&mut self.ui.pb_klapan_enb,
                                Text::new("Подать материал"))
                .style(style::Button::Check{checked: self.klapan_enb})
                .on_press(Message::ToggleKlapan(!self.klapan_enb));
            Column::new().spacing(5)
                .push(
                    Row::new().spacing(20)
                        .push(pb)
                        .push(Text::new(format!("Частота ШИМ: {:0>5}", self.shim_hz_ui)))
                        .push(slider)
                )
        };
        slider.into()
    }
}
