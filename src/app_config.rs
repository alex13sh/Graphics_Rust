use iced::{
    Application, executor, Command, Subscription, time,
    text_input, TextInput, button, Button, slider, Slider, scrollable, Scrollable,
    Element, Container, Text, Column, Row, Space, Length, Align,
    Settings, Clipboard,
};

fn main() {
    App::run(Settings::default());
}

use modbus::{Value, ModbusValues, ValueError, Device, DeviceError };

use std::collections::BTreeMap;
use std::collections::HashMap;
use std::sync::Arc;

type Values = BTreeMap<String, Arc<Value>>;
type ValuesOldNew = BTreeMap<String, (String, String)>; // (old, new)

pub struct App {
    ui: UI,
    
    logic: meln_logic::init::Complect,
    values: Values,
    txt_values: ValuesOldNew,
}

#[derive(Default)]
struct UI {
    scroll: scrollable::State,
    txt_values: BTreeMap<String, text_input::State>,
    pb_update: button::State,
    pb_write: button::State,
}

#[derive(Debug, Clone)]
pub enum Message {
    ValueEdited(String, String), // name, value
    ModbusUpdate, ModbusWrite,
    ModbusUpdateAsync, ModbusUpdateAsyncAnswer,
    ModbusUpdateAsyncAnswerDevice(Arc<Device>, Result<(), DeviceError>),
}

fn make_values(values: &Values) -> ValuesOldNew {
    values.iter()
        .map(|(k, v)| {let v = v.value().to_string(); (k.clone(), (v.clone(), v))})
        .collect()
}

impl App {

    fn make_new_values(&self) -> ModbusValues {
        self.txt_values.iter()
            .filter(|(_, (old, new))| old != new)
            .flat_map(|(name, (_, new))| Some(((name.clone(), Arc::new(self.values[name].new_value(new.parse().ok()?))))))
            .collect::<HashMap<_,_>>()
            .into()
    }
}

impl Application for App {
    type Executor = executor::Default;
    type Flags = ();
    type Message = Message;
    
    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        let mut logic = meln_logic::init::Complect::new();
        let values = logic.make_values(false);
//         logic.init_values(&values);
        
        let txt_values = make_values(&values);
        
        (
        Self {
            ui: UI {
                txt_values: values.iter()
                    .map(|(k, v)| (k.clone(), text_input::State::default()))
                    .collect(),
                .. UI::default()
            },
            
            logic: logic,
            values: values,
            txt_values: txt_values,
        },
        Command::none()
        )
    }
    
    fn title(&self) -> String {
        String::from("Config Modules - Iced")
    }
    
    fn update(&mut self, message: Self::Message, _clipboard: &mut Clipboard) -> Command<Self::Message> {
        use modbus::UpdateReq;
        match message {
        Message::ValueEdited(name, value) => 
            if let Some((_old, new)) = self.txt_values.get_mut(&name) {
                *new = value;
            },
        Message::ModbusUpdate => {
            self.logic.update_all();
            self.txt_values = make_values(&self.values);
        }, Message::ModbusWrite => {
            let new_values = self.make_new_values();
            dbg!(&new_values);
            for (k,v) in new_values.iter() {
//                 self.values
                self.logic.set_value(v.name(), v.value());
            }
            self.logic.update_new_values();
            for (ref mut old, new) in self.txt_values.values_mut() {
                *old = new.clone();
            }
        },
        Message::ModbusUpdateAsync => {
//             use futures::future::join_all;
            let device_futures = self.logic.update_async(UpdateReq::All);
                
//             let fut = join_all(device_features);
//             return Command::perform(fut, move |_| Message::ModbusUpdateAsyncAnswer);
            println!("Message::ModbusUpdateAsync end");
            return Command::batch(device_futures.into_iter()
                .map(|(d, f)| Command::perform(f, move |res| Message::ModbusUpdateAsyncAnswerDevice(d.clone(), res)))
                );
        },
        Message::ModbusUpdateAsyncAnswer => {
            self.txt_values = make_values(&self.values);
        },
        Message::ModbusUpdateAsyncAnswerDevice(d, res) => {
            dbg!(&d.name(), &res);
            if res.is_ok() {
                println!("Message::ModbusUpdateAsyncAnswerDevice {}", d.name());
                if !d.is_connect() {
                    println!("\tis not connect");
                } else {
                    self.txt_values = make_values(&self.values);
                }
            }
        },
        };
        Command::none()
    }
    
    fn view(&mut self) -> Element<Self::Message> {
        let Self {
            txt_values,
            ui: UI {
                scroll: ui_scroll,
                txt_values: ui_txt_values,
                pb_update, pb_write,
            },
            ..
        } = self;
        
        let values = ui_txt_values.iter_mut()
            .fold(Column::new()
                .spacing(10).align_items(Align::Center), 
                |lst, (name, input_state)| {
                let name = name.clone();
                let txt_value = &txt_values[&name];
//                 if let Some(ref txt_value) = txt_values.get(name) {
                    lst.push(Row::new().spacing(20)
                        .push(Text::new(name.clone()).width(Length::FillPortion(70)))
                        .push(TextInput::new(input_state, "Value", &txt_value.1,
                            move |value| Message::ValueEdited(name.clone(), value))
                            .width(Length::FillPortion(30))
                            .padding(10)
                            .style(style::ValueInput {
                                changed: txt_value.0 != txt_value.1
                            })
                        )
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
                .push(Button::new(pb_update, Text::new("Обновление")).on_press(Message::ModbusUpdateAsync))
                .push(Button::new(pb_write, Text::new("Записать")).on_press(Message::ModbusWrite))
            ).push(Scrollable::new(ui_scroll)
                .padding(10)
                .push(content)
            ).into()
            
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
