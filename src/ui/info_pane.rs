use std::sync::Arc;
use super::style;

use iced::{
    Element, Text, button, Button, slider, Slider,
    Column, Row, Space, Length,
};

pub struct InfoPane {
    ui: UI,
    // log values
    // table info
}

#[derive(Default)]
struct UI {
    open: button::State, // Открыть таблицу
}

#[derive(Debug, Clone)]
pub enum Message {
    OpenTable,
}

impl InfoPane {
    pub fn new() -> Self {
        InfoPane {
            ui: UI::default(),
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
        Message::OpenTable => {
        
        }
        }
    }

    pub fn view(&mut self) -> Element<Message> {
        let pb_open = Button::new(&mut self.ui.open, Text::new("Открыть таблицу"))
//             .style(style::Button::Check{checked: *check})
            .on_press(Message::OpenTable);
        let txt_info = format!(
r#"Время запуска: 13:37:07
Время остановки: 13:38:10
Время работы: 63 сек
Время разгона: 20 сек
Скорость двигателя: 10 000
Максимальня виброскорость: 2.5
Зона вибрации: 8 200"#
//         , 1,2,3,4,5,6,7
        );
        let txt_info = Text::new(txt_info);
        let elm = Column::new().spacing(20)
            .push(txt_info)
            .push(pb_open);
        elm.into()
    }
}
