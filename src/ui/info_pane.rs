use std::sync::Arc;
use super::style;
use std::path::PathBuf;

use iced::{
    Element, Text, button, Button, slider, Slider,
    Column, Row, Space, Length,
};

pub struct InfoPane {
    ui: UI,
    // log values
    info: Option<logger::LogState>,
    file_path: Option<PathBuf>,
}

#[derive(Default)]
struct UI {
    open: button::State, // Открыть таблицу
}

#[derive(Debug, Clone)]
pub enum Message {
    OpenTable,
    UpdateInfo(Option<(logger::LogState, PathBuf)>),
}

impl InfoPane {
    pub fn new() -> Self {
        InfoPane {
            ui: UI::default(),
            info: None,
            file_path: None,
        }
    }

    pub fn set_info(&mut self, info: logger::LogState) {
        self.info = Some(info);
    }
    pub fn set_file_path(&mut self, path: PathBuf) {
        self.file_path = Some(path);
    }

    pub fn update(&mut self, message: Message) {
        match message {
        Message::OpenTable => {
            use std::process::Command;
            if let Some(ref file_path) = self.file_path {
                let res = Command::new("libreoffice")
                        .arg("--calc")
                        .arg(file_path)
                        .spawn();
                if let Err(e) = res {
                    dbg!(e);
                }
            }
        }
        Message::UpdateInfo(Some((table_state, path))) => {
            self.set_info(table_state);
            self.set_file_path(path);
        },
        Message::UpdateInfo(None) => {println!("Message::UpdateInfo(None)");}
        }
    }

    pub fn view(&mut self) -> Element<Message> {
        let pb_open = Button::new(&mut self.ui.open, Text::new("Открыть таблицу"))
//             .style(style::Button::Check{checked: *check})
            .on_press(Message::OpenTable);
        let txt_info = if let Some(ref info) = self.info {
            format!(
r#"
Время работы:       {} сек
Время разгона:      {} сек
Скорость двигателя: {}
Максимальня виброскорость: {}
Зона вибрации:      {}
--
"#
    , info.time_work
    , info.time_acel
    , info.hz_max
    , info.vibro_max
    , info.hz_vibro
        )} else {
            format!(
// Время запуска:      13:37:07
// Время остановки:    13:38:10
r#"
Время работы:       63 сек
Время разгона:      20 сек
Скорость двигателя: 10 000
Максимальня виброскорость: 2.5
Зона вибрации:      8 200
--
"#
        )};
        let txt_info = Text::new(txt_info);
        let mut elm = Column::new().spacing(20)
            .push(txt_info);
        if let Some(_) = self.file_path {
            elm = elm.push(pb_open);
        }
        elm.into()
    }
}
