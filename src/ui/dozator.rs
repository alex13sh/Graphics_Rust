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
    shim_hz_ui: i32, shim_hz_new: i32,
    klapan_enb: bool,
    
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
    AnimationPos(i32),
}

impl Dozator {
    pub fn new() -> Self {
        Dozator {
            ui: UI::default(),
            shim_hz_ui: 0, shim_hz_new: 0,
            klapan_enb: false,
        }
    }

    pub fn subscription(&self, props: &meln_logic::watcher::Dozator) -> iced::Subscription<Message> {
        use super::animations::PropertyAnimation;
        iced::Subscription::from_recipe(
            PropertyAnimation::new("ШИМ", props.speed.subscribe())
        ).map(Message::AnimationPos)
    }

    pub fn update(&mut self, message: Message, values: &meln_logic::values::Dozator)  -> Command<Message> {
        match message {
        Message::ShimHzChanged(hz) => {
            self.shim_hz_ui = hz;
            self.shim_hz_new = hz;
        }
        Message::SetShimHz => {
            values.set_target_speed(self.shim_hz_new);
        },
        Message::ToggleKlapan(enb) => {
            self.klapan_enb = enb;
        },
        Message::AnimationPos(value) => {
            self.shim_hz_ui = value as i32;
        },
        _ => {},
        }
        Command::none()
    }
    pub fn view(&mut self) -> Element<Message> {
        // 1000 = 3 градусов / сек
        // 10_000 = 1 оборот / сек
        let slider = {
            let slider = Slider::new(
                &mut self.ui.shim_hz,
                -500..=5_000,
                self.shim_hz_ui,
                Message::ShimHzChanged
            )
            .on_release(Message::SetShimHz)
            .step(100);

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
