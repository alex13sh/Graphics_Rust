use std::sync::Arc;
use super::style;

use iced::{
    Element, Text, button, Button, slider, Slider,
    Column, Row, Space, Length,
};

use std::collections::BTreeMap;

struct MyKlapan {
    shk: String,
    name: String,
    enb: bool,
    state: button::State,
}
struct MyButton {
    name: String,
    enb: bool,
    state: button::State,
}

pub struct Klapans {
    klapans: Vec<MyKlapan>,
    buttons: Vec<MyButton>,
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
            ("ШК1", "Клапан нижнего контейнера"), // ШК1
            ("ШК3", "Клапан верхнего контейнера"), // ШК5
            ("ШК2", "Клапан подачи материала"),  // ШК2
            ("ШК5", "Клапан помольной камеры"),  // ШК3
            ("ШК4", "Клапан напуска"),           // ШК4
            ("ШК6", "Клапан насоса М5"),         // ШК6
        ];
        let button_names = [
            "Двигатель компрессора воздуха",
            "Уменьшить давление",
            "Увеличить давление",
            "ШК в рабочее положение",
        ];
        Klapans {
            klapans: klapan_names.into_iter()
                .map(|(shk, n)| MyKlapan {
                    shk: (*shk).into(),
                    name: (*n).into(),
                    enb: false,
                    state: Default::default()
                }).collect(),
            buttons: button_names.into_iter()
                .map(|n| MyButton {
                    name: (*n).into(),
                    enb: false,
                    state: Default::default()
                }).collect(),
            values: values,
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
        Message::ToggleKlapan(name, enb) => {
//             self.klapans.iter_mut().find(|s| s.name==name).unwrap().enb = enb;
//             self.values.set_bit(&name, enb).unwrap();
            self.set_klapan(name.as_str(), enb);
        }
        Message::PressButton(name) => {
            let mut pb = self.buttons.iter_mut().find(|s| s.name==name).unwrap();
            match (name.as_str(), pb.enb) {
            ("Уменьшить давление", false) => {
                pb.enb = true;
                self.davl_down();
            }, ("Уменьшить давление", true) => {
                pb.enb = false;
//                 pb.0 = "Увеличить давление".into();
                self.davl_dis();

            }, ("Увеличить давление", false) => {
                pb.enb = true;

                self.davl_up();
            }, ("Увеличить давление", true) => {
                pb.enb = false;
//                 pb.0 = "Уменьшить давление".into();

                self.davl_dis();
            },
            ("ШК в рабочее положение", _) => {
                pb.enb = !pb.enb;
                let enb = pb.enb;
                self.set_klapan("Клапан нижнего контейнера", enb); // ШК1
                self.set_klapan("Клапан верхнего контейнера", enb); // ШК5
//                 self.set_klapan("Клапан подачи материала", enb); // ШК2
                self.set_klapan("Клапан помольной камеры", enb); // ШК3
            },
            ("Двигатель компрессора воздуха", _) => {
                pb.enb = !pb.enb;
                let enb = pb.enb;
                self.set_klapan("Двигатель компрессора воздуха", enb);
            },
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
            .fold(Row::new().spacing(5),
                |row, MyKlapan {ref name, enb: ref check, state: pb, ..}|
                row.push(Button::new(pb, Text::new(name))
                .style(style::Button::Check{checked: *check})
                .on_press(Message::ToggleKlapan(name.clone(), !check)))
            );
        let controls_buttons = self.buttons.iter_mut()
            .fold(Row::new().spacing(5),
                |row, MyButton{ref name, enb: ref check, state: pb}|
                row.push(Button::new(pb, Text::new(name))
                .style(style::Button::Check{checked: *check})
                .on_press(Message::PressButton(name.clone())))
            );
        Column::new().spacing(10)
            .push(controls_buttons)
            .push(controls_klapan)
            .into()
    }
}

impl Klapans {
    pub fn update_klapans(&mut self) {
//         dbg!("update klapans");
        for k in self.klapans.iter_mut() {
//             dbg!(&k.shk);
//             if let Ok(enb) = self.values.get_bit(&format!("Клапан {} открыт", k.shk)) {
            if let Ok(enb) = self.values.get_bit(&k.name) {
                k.enb = enb;
//                 dbg!(enb);
            }
        }
    }

    fn set_klapan(&mut self, name: &str, enb: bool) {
        pub use std::convert::TryFrom;
        if self.values.get_value_arc("Давление воздуха компрессора")
            .map(|v| v.value().is_error()).unwrap_or(true)
        {
            return;
        }

        // Если давление воздуха меньше 4, то клапана открываться не будут.
        if let Some(v) = self.klapans.iter_mut().find(|s| s.name==name) {
            v.enb = enb;
        }
        if let Err(e) = self.values.set_bit(name, enb) {
            dbg!(e);
        }
    }

    fn set_button(&mut self, name: &str, enb: bool) {
        if let Some(v) = self.buttons.iter_mut().find(|s| s.name==name) {
            v.enb = enb;
        }
    }

    fn davl_down(&mut self) {
        self.values.set_bit("Двигатель насоса вакуума 1", true).unwrap();
        self.values.set_bit("Двигатель насоса вакуума 2", true).unwrap();

        self.set_klapan("Клапан насоса М5", true);

//         self.set_button("Уменьшить давление", true);
    }
    pub fn davl_dis(&mut self) {
        self.values.set_bit("Клапан насоса М5", false).unwrap();
        self.values.set_bit("Двигатель насоса вакуума 1", false).unwrap();
        self.values.set_bit("Двигатель насоса вакуума 2", false).unwrap();

        self.set_klapan("Клапан напуска", false);
        self.set_button("Уменьшить давление", false);
        self.set_button("Увеличить давление", false);
    }
    fn davl_up(&mut self) {
        self.davl_dis();

        self.set_klapan("Клапан напуска", true);
    }

    pub fn oil_station(&self, enb: bool) {

    }
}
