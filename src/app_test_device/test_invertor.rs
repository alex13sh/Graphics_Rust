#![allow(unused_imports)]

use iced::{
    Align, Column, Row, Scrollable, scrollable, Container, Element, Length,
    Text, text_input, TextInput, button, Button, slider, Slider, Radio,
    Application, window, Settings, executor, Subscription, Command, time,
};

use modbus::init;
use modbus::invertor::{Invertor, DvijDirect}; // Device
use modbus::{Value, ModbusValues};

pub struct TestInvertor {
    ui: UI,
    invertor: Invertor,
    values: Vec<DeviceValue>,
    speed: u16,
    direct: DvijDirect,
}

#[derive(Default)]
struct UI {
    pb_start: button::State,
    pb_stop: button::State,
    scroll_value: scrollable::State,
    speed_slider: slider::State,
    error: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Message {
    Start,
    Stop,
    SpeedChanged(u16),
    DirectChanged(DvijDirect),
    Update,
}

impl TestInvertor {
    pub fn new(ip_address: String) -> Self {
        let invertor = Invertor::new(init::make_invertor(ip_address).into());
        Self {
            values: make_values(invertor.device().values_map()),
            invertor: invertor,
            speed: 10_u16,
            direct: DvijDirect::FWD,
            ui: Default::default()
        }
    }
    
    pub fn subscription(&self) -> Subscription<Message> {
        time::every(std::time::Duration::from_millis(500))
            .map(|_| Message::Update)
    }

    #[allow(unused_must_use)]
    pub fn update(&mut self, message: Message) {
//                 println!("update");
        match message {
            Message::Start => self.invertor.start().unwrap(),
            Message::Stop => self.invertor.stop().unwrap(),
            Message::SpeedChanged(speed) => {
                self.speed = speed;
                if let Err(error) = self.invertor.set_hz(speed) {
                    self.ui.error = Some(format!("Error: {}", error));
                } else { self.ui.error = Some("Not error".into()); }
            },
            Message::DirectChanged(direct) => {
                self.invertor.set_direct(direct);
                self.direct = direct;
            },
            Message::Update => {
                use modbus::DeviceError;
                if let Err(error) = self.invertor.device().update() {
                    self.ui.error = Some(format!("Error: {}", error));
                } else { self.ui.error = None; }
            }
        };
    }
    pub fn view(&mut self) -> Element<Message> {
//                 println!("view");
        let start = Button::new(&mut self.ui.pb_start, Text::new("Старт"))
            .on_press(Message::Start);
        let stop = Button::new(&mut self.ui.pb_stop, Text::new("Стоп"))
            .on_press(Message::Stop);
        let mut res = Column::new()
            .spacing(20)
            .align_items(Align::Center)
            .push(Text::new(self.invertor.device().name()))
            .push(Text::new(format!("Values: {}", self.invertor.device().values_map().len())))
            ;
        res = if self.invertor.device().is_connect() {
            let slider = {
                let slider = Slider::new(
                    &mut self.ui.speed_slider,
                    0..=100/10,
                    self.speed/10,
                    |speed| Message::SpeedChanged(speed*10),
                );
                let speed_out = self.invertor.get_hz_out_value().value();
                Row::new()
                    .spacing(20)
                    .push(Text::new(format!("Установка скорости: {}", self.speed)))
                    .push(slider)
                    .push(Text::new(format!("Выходная скорость: {}", speed_out)))
            };
            let direct = {
                let ref cur_direct = self.direct;
                let radio_direct = |direct: &DvijDirect| Radio::new(
                        *direct, &format!("{:?}", direct),
                        Some(*cur_direct), Message::DirectChanged,
                        );
                Row::new()
                    .spacing(20)
                    .push(radio_direct(&DvijDirect::FWD))
                    .push(radio_direct(&DvijDirect::REV))
            };
            res.push(start)
                .push(stop)
                .push(slider)
                .push(direct)
        } else {
            res.push(Text::new(format!("Инвертор не подключен!\nIP Address: {}", self.invertor.device().get_ip_address())))
        };
        if let Some(ref error) = self.ui.error {
            res = res.push(Text::new(error));
        }
        /*{
            let mut scroll = Scrollable::new(&mut self.ui.scroll_value);
            scroll = self.values.iter_mut().fold(scroll, |scroll, v| scroll.push(v.view()));
            res.push(scroll).into()
        }*/
        
        res.into()
    }
}

fn make_values(values: &ModbusValues) -> Vec<DeviceValue> {
//             println!("values_view");
    use std::collections::HashMap;
    let mut adr_name: Vec<_> = values.values().into_iter().map(|v| (v.address(), v.name().clone())).collect();
    adr_name.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    adr_name.into_iter().map(|(_, name)| 
        DeviceValue::new(values.get(&name).unwrap().clone())
    ).collect()
}

use std::sync::Arc;
struct DeviceValue {
    value: Arc<Value>,
}
impl DeviceValue {
    fn new(value: Arc<Value>) -> Self {
        Self {
            value: value.clone(),
        }
    }
    
    fn view(&mut self) -> Element<Message> {
        let mut txt: String = self.value.name().chars().take(20).collect();
        if self.value.name().chars().nth(20).is_some() {
            txt = txt + "...";
        }
        Text::new(format!("{:0>4X}) name: {}", self.value.address(), txt)).size(12) // {:0>4})
            .into()
    }
}
