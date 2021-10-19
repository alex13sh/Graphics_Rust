use iced::{
    Element, Text, button, Button, slider, Slider,
    Column, Row, Space, Length,
};

use std::collections::BTreeMap;
use super::style;

pub struct OilStation {
    ui: UI,
    values_list: super::ValuesList,
    pub is_started: bool,
}

#[derive(Default)]
struct UI {
    pb_start_stop: button::State,
}

#[derive(Debug, Clone)]
pub enum Message {
    StartStopToggle,
}

impl OilStation {
    pub fn new_by_meln(values: &meln_logic::values::Meln) -> Self {

        OilStation {
            ui: UI::default(),
            values_list: super::ValuesList {
                name: "МаслоСтанция".into(),
                values: vec![
                    values.oil.уровень_масла.clone(),
                    values.oil.температура.clone(),
                    values.oil.давление_масла.clone(),
                    
                    values.half_bottom.invertor.выходной_ток.clone(),
                    values.half_bottom.invertor.индикация_мощности.clone(),
                    values.half_bottom.invertor.скорость_двигателя.clone(),
                    values.half_bottom.invertor.индикация_скорости.clone(),

                    values.half_bottom.vibro.clone(),
                    // Давление воздуха в Клапанах
                    values.vacuum.vacuum.clone(),
                ]
            },
                            
            is_started: false,
        }
    }
    
    pub fn new(values: &meln_logic::values::OilStation) -> Self {

        OilStation {
            ui: UI::default(),
            values_list: super::ValuesList {
                name: "МаслоСтанция".into(),
                values: vec![
                    values.уровень_масла.clone(),
                    values.температура.clone(),
                    values.давление_масла.clone(),
                ]
            },
                            
            is_started: false,
        }
    }

    pub fn update(&mut self, message: Message, values: &meln_logic::values::OilStation) {
        match message {
        Message::StartStopToggle => {
            self.is_started = !self.is_started;
            values.motor_turn(self.is_started);
        },
        }
    }

    pub fn view(&mut self) -> Element<Message> {

        let is_started = self.is_started;
        let start = Button::new(&mut self.ui.pb_start_stop,
                if !is_started { Text::new("Запустить маслостанцию") }
                else {Text::new("Остановить маслостанцию")})
            .style(style::Button::Check{
                    checked: is_started
            }).on_press(Message::StartStopToggle);

//         let list_value = self.values_list.iter()
//             .fold(Column::new().spacing(20), |lst, v| lst.push(v.view()));
        let list_value = Column::new().spacing(20)
            .push(self.values_list.view());
        let column = Column::new().spacing(10)
            .push(list_value)
            .push(start)
            .width(Length::Fill);

        column.into()
    }
}
