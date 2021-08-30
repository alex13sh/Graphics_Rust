use iced::{
    Application, executor, Command, Subscription, time,
    text_input, TextInput, button, Button, slider, Slider, scrollable, Scrollable,
    Element, Container, Text, Column, Row, Space, Length, Align,
    Settings, Clipboard,
};

fn main() {
    App::run(Settings::default());
}

use modbus::{Value, ModbusValues, ValueError, Device, Invertor, DeviceError };

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::Arc;

type Values = BTreeMap<u16, Arc<Value>>;
type ValuesOldNew = BTreeMap<u16, (u32, u32)>; // (old, new)
type ValuesOld = BTreeMap<u16, u32>;

pub struct App {
    ui: UI,
    
    invertor_1: Invertor,
    update_enb: bool,

    values: Values,
    values_old_new: ValuesOldNew,
    prev_values: Option<ValuesOld>,
    
    logs_path: Option<Vec<std::path::PathBuf>>
}

#[derive(Default)]
struct UI {
    scroll: scrollable::State,
    values_old_new: BTreeMap<u16, text_input::State>,
    pb_update: button::State,
    pb_write: button::State,
}

#[derive(Debug, Clone)]
pub enum Message {
    ValueEdited(u16, String), // name, value

    ModbusWrite,
    ModbusUpdate(MessageMudbusUpdate),
}

#[derive(Debug, Clone)]
pub enum MessageMudbusUpdate {
    ModbusUpdate, ModbusUpdateAsyncAnswer,
    ModbusUpdateAsync, ModbusConnect,
    ModbusUpdateAsyncAnswerDevice(Arc<Device>, Result<(), DeviceError>),
}

fn make_values(values: &Values) -> ValuesOldNew {
    values.iter()
        .map(|(adr, v)| {let v = v.value(); (*adr, (v, v))})
        .collect()
}

impl App {

    fn make_new_values(&self) -> Values {
        self.values_old_new.iter()
            .filter(|(_, (old, new))| old != new)
            .map(|(adr, (_, new))| ((adr.clone(), Arc::new(self.values[adr].new_value(*new)))))
            .collect()
    }
}

impl Application for App {
    type Executor = executor::Default;
    type Flags = ();
    type Message = Message;
    
    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        let invertor = modbus::init::make_invertor("192.168.1.5".into(), 5);
        let invertor_1 = Invertor::new(invertor.into());

        let values = invertor_1.device().values()
            .into_iter().map(|v| (v.address(), v)).collect();
        let values_old_new = make_values(&values);

        let logs_path = func_files::get_list_log(&log::get_file_path("tables/log/")).ok();
        let prev_values = logs_path.as_ref().and_then(|paths| paths.last())
            .and_then(|path| func_files::read_file(&path));
        
        (
        Self {
            ui: UI {
                values_old_new: values.iter()
                    .map(|(k, v)| (k.clone(), text_input::State::default()))
                    .collect(),
                .. UI::default()
            },
            
            invertor_1: invertor_1,
            update_enb: false,
            values: values,
            values_old_new: values_old_new,
            prev_values: prev_values,
            
            logs_path: logs_path,
        },
        Command::none()
        )
    }
    
    fn title(&self) -> String {
        String::from("Config Modules - Iced")
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        time::every(std::time::Duration::from_millis(5000))
            .map(|_| MessageMudbusUpdate::ModbusConnect)
            .map(Message::ModbusUpdate)
    }
    
    fn update(&mut self, message: Self::Message, _clipboard: &mut Clipboard) -> Command<Self::Message> {
        use modbus::UpdateReq;
        match message {
        Message::ValueEdited(name, value) => 
            if let Some((_old, new)) = self.values_old_new.get_mut(&name) {
                if let Ok(v) = value.parse() {
                    *new = v;
                }
            },
        Message::ModbusWrite => {
            let new_values = self.make_new_values();
            dbg!(&new_values);
            for (k,v) in new_values.iter() {
//                 self.values
                self.invertor_1.device().values_map()
                    .set_value(v.name(), v.value());
            }
            self.invertor_1.device().update_new_values();
            for (ref mut old, new) in self.values_old_new.values_mut() {
                *old = new.clone();
            }
        },
        Message::ModbusUpdate(m) => return self.modbus_update(m),
        };
        Command::none()
    }
    
    fn view(&mut self) -> Element<Self::Message> {
        let Self {
            values_old_new,
            values, prev_values,
            ui: UI {
                scroll: ui_scroll,
                values_old_new: ui_values_old_new,
                pb_update, pb_write,
            },
            ..
        } = self;
        
        use std::convert::TryFrom;
        let value_to_f32 = |adr, value| {
            f32::try_from(&values[&adr].new_value(value)).unwrap_or(value as f32)
        };
        let values = ui_values_old_new.iter_mut()
            .fold(Column::new()
                .spacing(10).align_items(Align::Center), 
                |lst, (adr, input_state)| {
                let adr = adr.clone();
                let value_old_new = &values_old_new[&adr];
                let p_name = values[&adr].name();
//                 if let Some(ref txt_value) = values_old_new.get(name) {
                    lst.push(Row::new().spacing(20)
                        .push(Text::new(format!("{} - {}", log::InvertorParametr::parametr_str(adr), p_name))
                            .width(Length::Fill))
                        .push(TextInput::new(input_state, "Value", &value_to_f32(adr, value_old_new.1).to_string(),
                            move |value| Message::ValueEdited(adr, value))
                            .width(Length::Units(50))
                            .padding(10)
                            .style(style::ValueInput {
                                changed: value_old_new.0 != value_old_new.1
                            })
                        )
                        .push(Text::new(if let Some(prev_values) = prev_values {
                                    if let Some(v) = prev_values.get(&adr) {value_to_f32(adr, *v).to_string()}
                                    else {"None".into()}
                                } else {String::new()})
                                .width(Length::Units(50))
                                .height(Length::Units(40))
                                .vertical_alignment(iced::VerticalAlignment::Center))
                    )
//                 } else {lst}
            });
         
        let content = values;
        let content = Container::new(content)
            .width(Length::Fill)
//             .height(Length::Fill)
            .padding(10)
            .center_x();
//             .center_y();

        Column::new().spacing(20)
            .push(Row::new().spacing(20)
                .push(Button::new(pb_update, Text::new("Обновление")).on_press(Message::ModbusUpdate(MessageMudbusUpdate::ModbusUpdateAsync)))
                .push(Button::new(pb_write, Text::new("Записать")).on_press(Message::ModbusWrite))
            ).push(Scrollable::new(ui_scroll)
                .padding(10)
                .push(content)
            ).into()
            
    }
}

// modbus update
impl App {
    fn modbus_update(&mut self, message: MessageMudbusUpdate) -> Command<Message> {
        use modbus::UpdateReq;
        match message {
            MessageMudbusUpdate::ModbusUpdate  => {
                self.invertor_1.device().update();
                self.values_old_new = make_values(&self.values);
            },
            MessageMudbusUpdate::ModbusUpdateAsync => if self.update_enb {
                self.update_enb = false;
                let d = self.invertor_1.device();
                let f = async move {d.update_async(UpdateReq::All).await};

                return Command::perform(f, move |res| Message::ModbusUpdate(
                        MessageMudbusUpdate::ModbusUpdateAsyncAnswer));
            },
            MessageMudbusUpdate::ModbusConnect => {
                println!("MessageMudbusUpdate::ModbusConnect ");
                self.update_enb = false;
                let d = self.invertor_1.device();
                let f = async move {d.connect().await};
                return Command::perform(f, move |res| Message::ModbusUpdate(
                        MessageMudbusUpdate::ModbusUpdateAsyncAnswer));
            },
            MessageMudbusUpdate::ModbusUpdateAsyncAnswer => {
                self.update_enb = true;
                self.values_old_new = make_values(&self.values);
            },
            MessageMudbusUpdate::ModbusUpdateAsyncAnswerDevice(d, res) => {
    //             dbg!(&d);
                if !d.is_connect() {
                    println!("\tis not connect");
                } else {
                    self.values_old_new = make_values(&self.values);
                }
            },
        }
        Command::none()
    }
}

mod func_files {
    use super::ValuesOld;
    use std::path::PathBuf;
    
    pub fn read_file(file_name: &PathBuf) -> Option<ValuesOld> {
        dbg!(file_name);
        let values = log::csv::read_invertor_parametrs(file_name)?
            .into_iter().map(|p| (p.address(), p.value))
            .collect();
        Some(values)
    }
    pub fn get_list_log(dir: &PathBuf) -> std::io::Result<Vec<PathBuf>> {
        let mut v = if dir.is_dir() {
            std::fs::read_dir(dir)?.into_iter()
                .filter_map(|e| Some(e.ok()?.path()))
                .filter(|p| p.is_file() && p.extension().and_then(|s| s.to_str()) == Some("csv") )
                .collect()
        } else {Vec::new()};
        v.sort_by_key(|p| p.metadata().unwrap().modified().unwrap());
        Ok(v)
    }
}

mod style {
    use iced::{text_input, container, Background, Color, Vector};
    
    pub struct ValueInput{
        pub changed: bool,
    }
    
    impl text_input::StyleSheet for ValueInput {
        fn active(&self) -> text_input::Style {
            let g = if self.changed {255} else {0};
            text_input::Style {
                border_width: 2.0,
                border_color: Color::from_rgb8(0, g, 0),
                background: Background::Color(Color::from_rgba8(255-g, 255, 255-g, 0.2)),
                .. Default::default()
            }
        }
        
            /// Produces the style of a focused text input.
        fn focused(&self) -> text_input::Style {
            self.active()
        }

        fn placeholder_color(&self) -> Color {
            Color::from_rgb8(0, 0, 0)
        }

        fn value_color(&self) -> Color {
            Color::from_rgb8(0, 0, 0)
        }

        fn selection_color(&self) -> Color {
            Color::from_rgb8(0, 0, 255)
        }
    }
}
