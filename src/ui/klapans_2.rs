use std::sync::Arc;
use super::style;

use iced::{
    Element, Text, button, Button, slider, Slider,
    Column, Row, Space, Length,
};

use std::collections::BTreeMap;

pub struct Klapans {
    klapans: Vec<(String, bool, button::State)>,
    buttons: Vec<(String, bool, button::State)>,
    values: modbus::ModbusValues,
}

#[derive(Debug, Clone)]
pub enum Message {
    ToggleKlapan(String, bool),
    PressButton(String),
}

impl Klapans {
    pub fn new(values: modbus::ModbusValues) -> Self {
        let klapan_names = [
            "Двигатель компрессора воздуха",
            "Клапан нижнего контейнера", // ШК1
            "Клапан верхнего контейнера", // ШК5
            "Клапан подачи материала",  // ШК2
            "Клапан помольной камеры",  // ШК3
//             "Клапан напуска",           // ШК4
//             "Клапан насоса М5",         // ШК6
        ];
        let button_names = [
            "Уменьшить давление",
        ];
        Klapans {
            klapans: klapan_names.into_iter()
                .map(|&n| ((*n).into(), false, Default::default())).collect(),
            buttons: button_names.into_iter()
                .map(|&n| ((*n).into(), false, Default::default())).collect(),
            values: values,
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
        Message::ToggleKlapan(name, enb) => {
            self.klapans.iter_mut().find(|s| s.0==name).unwrap().1 = enb;
            self.values.set_bit(&name, enb).unwrap();
        }
        Message::PressButton(name) => {
            let mut pb = self.buttons.iter_mut().find(|s| s.0==name).unwrap();
            match (name.as_str(), pb.1) {
            ("Уменьшить давление", false) => {
                pb.1 = true;
                self.davl_down();
            }, ("Уменьшить давление", true) => {
                pb.1 = false;
                pb.0 = "Увеличить давление".into();
                self.davl_dis();

            }, ("Увеличить давление", false) => {
                pb.1 = true;

                self.davl_up();
            }, ("Увеличить давление", true) => {
                pb.1 = false;
                pb.0 = "Уменьшить давление".into();

                self.davl_dis();
            }
            _ => {}}
//             match name {
//             "Двигатель насоса вакуума" => {
//                 self.values.set_bit("Двигатель насоса вакуума 1", enb).unwrap();
//                 self.values.set_bit("Двигатель насоса вакуума 2", enb).unwrap();
//             },
//             _ => {}
//             }
        }
        }
    }

    pub fn view(&mut self) -> Element<Message> {
        let controls_klapan = self.klapans.iter_mut()
            .fold(Row::new().spacing(20),
                |row, (ref name, ref check, pb)|
                row.push(Button::new(pb, Text::new(name))
                .style(style::Button::Check{checked: *check})
                .on_press(Message::ToggleKlapan(name.clone(), !check)))
            );
        let controls_buttons = self.buttons.iter_mut()
            .fold(Row::new().spacing(20),
                |row, (ref name, ref check, pb)|
                row.push(Button::new(pb, Text::new(name))
                .style(style::Button::Check{checked: *check})
                .on_press(Message::PressButton(name.clone())))
            );
        Row::new().spacing(25)
            .push(controls_buttons)
            .push(controls_klapan)
            .into()
    }
}

impl Klapans {

    fn davl_down(&self) {
        self.values.set_bit("Двигатель насоса вакуума 1", true).unwrap();
        self.values.set_bit("Двигатель насоса вакуума 2", true).unwrap();
        self.values.set_bit("Клапан насоса М5", true).unwrap();
    }
    pub fn davl_dis(&self) {
        self.values.set_bit("Клапан насоса М5", false).unwrap();
        self.values.set_bit("Двигатель насоса вакуума 1", false).unwrap();
        self.values.set_bit("Двигатель насоса вакуума 2", false).unwrap();

        self.values.set_bit("Клапан напуска", false).unwrap();
    }
    fn davl_up(&self) {
        self.davl_dis();
        self.values.set_bit("Клапан напуска", true).unwrap();
    }

    pub fn oil_station(&self, enb: bool) {

    }
}
