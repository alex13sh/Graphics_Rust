use iced::{
    Align, Column, Row, Scrollable, scrollable, Container, Element, Length,
    Text, text_input, TextInput, button, Button, 
    Application, window, Settings, executor, Subscription, Command, time,
};

use modbus::init;
use modbus::{Device, DigitIO};
use modbus::{Value, ModbusValues};

pub struct IODigit {
    ui: UI,
    device: DigitIO,
}

#[derive(Default)]
struct UI {
    
}

#[derive(Debug, Clone)]
pub enum Message {

}

impl IODigit {
    pub fn new(ip_address: String) -> Self {
        let device: Device = init::make_io_digit(ip_address).into();
        IODigit {
            ui: UI::default(),
            device: DigitIO::from(device),
        }
    }

    pub fn update(&mut self, _message: Message) {

    }

    pub fn view(&mut self) -> Element<Message> {
        let mut res = Column::new()
            .spacing(20)
            .align_items(Align::Center)
            .push(Text::new(self.device.device().name()))
            .push(Text::new(format!("Values: {}", self.device.device().values_map().len())))
            ;
        res = if self.device.device().is_connect() {
            res
        } else {
            res.push(Text::new(format!("Устройство не подключено!\nIP Address: {}", self.device.device().get_ip_address())))
        };
        res.into()
    }
}

mod sensor_output {
    use modbus::Sensor;
    pub struct DigitOutput {
        sensor: Sensor,
    }
    
    pub enum Message {
        Trigger(bool),
    }
    
    impl DigitOutput {
        pub fn new(sens: Sensor) -> Option<Self> {
//             if sens
            Some(DigitOutput {
                sensor: sens,
            })
        }
    }
}
