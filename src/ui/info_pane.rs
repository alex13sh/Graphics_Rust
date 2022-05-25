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
    txt_info: String,
    file_path: Option<PathBuf>,
}

#[derive(Default)]
struct UI {
    open: button::State, // Открыть таблицу
}

#[derive(Debug, Clone)]
pub enum Message {
    OpenTable,
    UpdateInfo(logger::LogState),
    UpdateInfoAdvant(logger::stat_info::advance::StateInfoFull),
    UpdateFile(PathBuf),
}

impl InfoPane {
    pub fn new() -> Self {
        InfoPane {
            ui: UI::default(),
            txt_info: Self::format_advant_state_full(Default::default()),
            file_path: None,
        }
    }

    pub fn set_file_path(&mut self, path: PathBuf) {
        self.file_path = Some(path);
    }

    fn format_log_state(info: logger::LogState) -> String {
        format!(
r#"
Время работы:       {} сек
Время разгона:      {} сек
Скорость двигателя: {}
Максимальня виброскорость: {}
Зона вибрации:      {}
--
"#
            , info.time_all
            , info.time_acel
            , info.hz_max
            , info.vibro_max
            , info.hz_vibro
        )
    }

    fn format_advant_state_full(state: logger::stat_info::advance::StateInfoFull) -> String {
        let sum = state.sum();

        let mut txt_info = format!(
r#"
Время работы:               {время} сек
Максимальня виброскорость:  {max_vibro}
Максимальная мощность:      {max_power}
"#
            , время = sum.energy.time.interval()
            , max_vibro = sum.max_values.vibro
            , max_power = sum.max_values.power
        );

        if let Some(state_material) = sum.material.get_stat() {
            txt_info += &format!(
r#"
Время подачи материала:       {время} сек
Максимальная скорость двигателя: {speed_max} об./мин.
Максимальная просадка скорости двигателя: {speed_low} об./мин.
"#
                , время = state_material.energy.time.interval()
                , speed_max = state_material.speed.min_max().1
                , speed_low = state_material.speed.delta()
            );
        }
        txt_info
    }

    pub fn update(&mut self, message: Message) {
        match message {
        Message::OpenTable => {
            use std::process::Command;
            if let Some(ref file_path) = self.file_path {
                let res = Command::new("xdg-open")
//                         ("libreoffice").arg("--calc")
                        .arg(file_path)
                        .spawn();
                if let Err(e) = res {
                    dbg!(e);
                }
            }
        }
        Message::UpdateInfo(table_state) => self.txt_info = Self::format_log_state(table_state),
        Message::UpdateInfoAdvant(table_state) => self.txt_info = Self::format_advant_state_full(table_state),
        Message::UpdateFile(path) => self.set_file_path(path),
        }
    }

    pub fn view(&mut self) -> Element<Message> {
        let pb_open = Button::new(&mut self.ui.open, Text::new("Открыть таблицу"))
//             .style(style::Button::Check{checked: *check})
            .on_press(Message::OpenTable);

        let txt_info = Text::new(&self.txt_info);
        let mut elm = Column::new().spacing(20)
            .push(txt_info);
        if let Some(_) = self.file_path {
            elm = elm.push(pb_open);
        }
        elm.into()
    }
}
