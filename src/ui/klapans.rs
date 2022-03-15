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
    turn: bool,
    state: button::State,
}
struct MyButton {
    name: String,
    turn: bool,
    state: button::State,
}

pub struct Klapans {
    klapans: Vec<MyKlapan>,
    buttons: Vec<MyButton>,
    enb: bool,
}

#[derive(Debug, Clone)]
pub enum Message {
    ToggleKlapan(String, bool),
    ToggledKlapan(String, bool),
    PressButton(String),
    StatusChanged(meln_logic::watcher::VacuumStatus),
//     ДавлениеВоздухаChanged(f32),
    KlapansError(meln_logic::watcher::KlapansError),
}

impl Klapans {
    pub fn new() -> Self {
        let klapan_names = [
            ("ШК1", "Клапан нижнего контейнера"), // ШК1
            ("ШК3", "Клапан верхнего контейнера"), // ШК5
            ("ШК2", "Клапан подачи материала"),  // ШК2
            ("ШК5", "Клапан помольной камеры"),  // ШК3
            ("ШК4", "Клапан напуска"),           // ШК4
            ("ШК6", "Клапан насоса М5"),         // ШК6
        ];
        let button_names = [
            // "Двигатель компрессора воздуха",
            "ШК в рабочее положение",
            "Уменьшить давление",
            "Увеличить давление",
        ];
        Klapans {
            enb: true,
            klapans: klapan_names.into_iter()
                .map(|(shk, n)| MyKlapan {
                    shk: (*shk).into(),
                    name: (*n).into(),
                    enb: true,
                    turn: false,
                    state: Default::default()
                }).collect(),
            buttons: button_names.into_iter()
                .map(|n| MyButton {
                    name: (*n).into(),
                    turn: false,
                    state: Default::default()
                }).collect(),
        }
    }

    pub fn subscription(&self, props: &meln_logic::watcher::Klapans) -> iced::Subscription<Message> {
        use super::animations::*;
        iced::Subscription::batch(vec![
            PropertyAnimation::new_sub("Ошибка", props.klapans_error.subscribe())
                .map(Message::KlapansError),
            BroadcastAnimation::new_sub("Клапана", props.klapans_шк_send.subscribe())
                .map(|(name, turn)| Message::ToggledKlapan(name, turn))
        ])
    }

    pub fn subscription_vacuum(&self, props: &meln_logic::watcher::VacuumStation) -> iced::Subscription<Message> {
        use super::animations::PropertyAnimation;
        iced::Subscription::from_recipe(
            PropertyAnimation::new("Vacuum_Status", props.status.subscribe())
        ).map(Message::StatusChanged)
    }
    
    pub fn update_vacuum(&mut self, message: Message, values: &meln_logic::values::VacuumStation) {
        match message {
        Message::PressButton(name) => {
            let mut pb = self.buttons.iter_mut().find(|s| s.name==name).unwrap();
            match (name.as_str(), pb.turn) {
            ("Уменьшить давление", false) => {
                pb.turn = true;
                values.davl_down();
            }, ("Увеличить давление", false) => {
                pb.turn = true;
                values.davl_up();
            }, ("Увеличить давление" | "Уменьшить давление", true) => {
                pb.turn = false;
                values.davl_dis();
            },
            _ => {}
            }
        }
        Message::StatusChanged(status) => {
            log::trace!(target: "meln_logic::vacuum","status: {:?}", &status);
            use meln_logic::watcher::VacuumStatus::*;
            let pb_name = match status {
            Уменьшение_давления => "Уменьшить давление",
            Увеличение_давления => "Увеличить давление",
            Насосы_отключены => "",
            };
            log::trace!(target: "meln_logic::vacuum","pb_name: {}", pb_name);
            for pb in &mut self.buttons {
                if pb.name == pb_name {
                    pb.turn = true;
                } else {
                    pb.turn = false;
                }
            }
        }
        _ => {}
        }
    }
    
    pub fn update_material(&mut self, message: Message, values: &meln_logic::values::Material) {
        match message {
        Message::PressButton(name) => {
            let mut pb = self.buttons.iter_mut().find(|s| s.name==name).unwrap();
            match (name.as_str(), pb.turn) {
            ("ШК в рабочее положение", _) => {
                pb.turn = !pb.turn;
                values.ШК_в_рабочее_положение(pb.turn);
            },
            _ => {}
            }
        }
        _ => {}
        }
    }
    
    pub fn update(&mut self, message: Message, values: &meln_logic::values::Klapans) {
        match message {
        Message::ToggleKlapan(name, turn) => {
            if let Err(_) = values.klapan_turn(name.as_str(), turn) {
                self.enb = false;
            } else {
                self.enb = true;
            }
        }
        Message::ToggledKlapan(name, turn) => {
            if let Some(v) = self.klapans.iter_mut().find(|s| s.name==name) {
                v.turn = turn;
            }
        },
//         Message::ДавлениеВоздухаChanged(давление) => {
//             if
//         }
        Message::KlapansError(err) => {
            // Если ошибки нет, то клавиши работают
            self.enb = err.is_empty();
        }
        _ => {}
        }
    }

    pub fn view(&mut self) -> Element<Message> {
        let senb = self.enb;

        let controls_klapan = self.klapans.iter_mut()
            .fold(Row::new().spacing(5),
                |row, MyKlapan {ref name, ref enb, turn: ref check, state: pb, ..}|
                row.push(Button::new(pb, Text::new(name))
                .style(style::Button::Klapan {
                    enabled: senb,
                    checked: *check
                })
                .on_press(Message::ToggleKlapan(name.clone(), !check)))
            );
        let controls_buttons = self.buttons.iter_mut()
            .fold(Row::new().spacing(5),
                |row, MyButton{ref name, turn: ref check, state: pb}|
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
