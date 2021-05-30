use std::sync::Arc;
use super::style;

use iced::{
    Element, Text, button, Button, slider, Slider,
    Column, Row, Space, Length,
};

pub struct Klapans {
    ui: UI,
    klapans: [bool; 2],
    values: modbus::ModbusValues,
}

#[derive(Default)]
struct UI {
    klapan: [button::State; 3],
}

#[derive(Debug, Clone)]
pub enum Message {
    ToggleKlapan(usize, bool),
}

impl Klapans {
    pub fn new(values: modbus::ModbusValues) -> Self {
        Klapans {
            ui: UI::default(),
            klapans: [false; 2],
            values: values,
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
        Message::ToggleKlapan(ind, enb) => {
            self.klapans[ind as usize] = enb;
            self.klapans[1-ind as usize] = false;
            match ind {
            0 => {
                self.values.set_bit("Клапан 24В", false).unwrap();
                self.values.set_bit("Клапан 2", enb).unwrap();
                self.values.set_bit("Насос", enb).unwrap();
            }, 1 => {
                self.values.set_bit("Клапан 24В", enb).unwrap();
                self.values.set_bit("Клапан 2", false).unwrap();
                self.values.set_bit("Насос", false).unwrap();
            }, _ => {}
            }
        }}
    }

    pub fn view(&mut self) -> Element<Message> {
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
    }
}
