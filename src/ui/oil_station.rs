use iced::{
    Element, Text, button, Button, slider, Slider,
    Column, Row, Space, Length,
};

use std::collections::BTreeMap;
use super::invertor::ui_button_start;
use super::style;

pub struct OilStation {
    ui: UI,
    values: modbus::ModbusValues,
    values_list: super::ValuesList,
    pub is_started: bool,
}

#[derive(Default)]
struct UI {
//     pb_start_stop: ui_button_start::State,
    pb_start_stop: button::State,

}

#[derive(Debug, Clone)]
pub enum Message {
//     StartStop(bool),
    StartStopToggle,
//     ButtonStart(ui_button_start::Message),
}

impl OilStation {
    pub fn new(values: modbus::ModbusValues) -> Self {

        OilStation {
            ui: UI::default(),
            values_list: super::make_value_lists(&values, crate::map!{BTreeMap,
                    "МаслоСтанция" => [
                        "PDU-RS/value",
//                         "PDU-RS/hight limit",
//                         "PDU-RS/low limit",
                        "Температура масла на выходе маслостанции",
                        "Давление масла на выходе маслостанции",
                        "5) Invertor/Выходной ток (A)",
                        "5) Invertor/Выходная частота (H)",
                    ]
                }).pop().unwrap(),
            values: values,
            is_started: false,
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
//         Message::StartStop(enb) => self.is_started = enb;
        Message::StartStopToggle => {
            self.is_started = !self.is_started;
//             dbg!(self.is_started);
            self.values.get_value_arc("Двигатель маслостанции М4").unwrap().set_bit(self.is_started);
        },
        }
    }

    pub fn view(&mut self) -> Element<Message> {

        let is_started = self.is_started;
//         let start = self.ui.pb_start_stop.view(
//             self.is_started,
// //                     Message::ToggleStart(!self.is_started)
//         ).map(move |message| {
//             if let ui_button_start::Message::ToggleStart(start) = message {
//                 Message::StartStop(start)
//             } else {
//                 Message::ButtonStart(message)
//             }
//         });
        let start = Button::new(&mut self.ui.pb_start_stop,
                if !is_started { Text::new("Start") }
                else {Text::new("Stop")})
            .style(style::Button::Check{
                    checked: is_started
            }).on_press(Message::StartStopToggle);

//         let list_value = self.values_list.iter()
//             .fold(Column::new().spacing(20), |lst, v| lst.push(v.view()));
        let list_value = Column::new().spacing(20)
            .push(self.values_list.view());
        let mut column = Column::new().spacing(10)
            .push(list_value)
            .push(start)
            .width(Length::Fill);

        column.into()
    }
}
