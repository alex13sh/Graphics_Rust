use std::sync::Arc;
use super::style;

use iced::{
    Element, Text, button, Button, slider, Slider,
    Column, Row, Space, Length,
};

use std::collections::BTreeMap;

pub struct Klapans {
    klapans: BTreeMap<String, (bool, button::State)>,
    values: modbus::ModbusValues,
}

#[derive(Debug, Clone)]
pub enum Message {
    ToggleKlapan(String, bool),
}

impl Klapans {
    pub fn new(values: modbus::ModbusValues) -> Self {
        let names = [
            "Клапан нижнего контейнера", // ШК1
            "Клапан подачи материала",  // ШК2
            "Клапан помольной камеры",  // ШК3
            "Клапан напуска",           // ШК4
            "Клапан верхнего контейнера", // ШК5
            "Клапан насаса М5",         // ШК6
            "Двигатель насоса вакуума",
        ];
        Klapans {
            klapans: names.into_iter()
                .map(|&n| ((*n).into(), (false, Default::default()))).collect(),
            values: values,
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
        Message::ToggleKlapan(name, enb) => {
            self.klapans.get_mut(&name).unwrap().0 = enb;
            if name == "Двигатель насоса вакуума" {
                self.values.set_bit("Двигатель насоса вакуума 1", enb).unwrap();
                self.values.set_bit("Двигатель насоса вакуума 2", enb).unwrap();
            } else {
                self.values.set_bit(&name, enb).unwrap();
            }
        }}
    }

    pub fn view(&mut self) -> Element<Message> {
        let controls_klapan = self.klapans.iter_mut()
            .fold(Row::new().spacing(20),
                |row, (name, (ref check, pb))|
                row.push(Button::new(pb, Text::new(name))
                .style(style::Button::Check{checked: *check})
                .on_press(Message::ToggleKlapan(name.clone(), !check)))
            );
        controls_klapan.into()
    }
}
