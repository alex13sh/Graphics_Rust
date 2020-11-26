#![allow(unused_imports)]

use iced::{
    Align, Column, Row, Scrollable, scrollable, Container, Element, Length,
    Text, text_input, TextInput, button, Button, Checkbox, 
    Application, window, Settings, executor, Subscription, Command, time,
};

use modbus::init;
use modbus::{Device, DigitIO};
use modbus::{Value, ModbusValues};

pub struct IODigit {
    ui: UI,
    device: DigitIO,
    clapans: [bool; 3],
}

#[derive(Default)]
struct UI {
    
}

#[derive(Debug, Clone)]
pub enum Message {
    ClapanTurn(u8, bool),
}

impl IODigit {
    pub fn new(ip_address: String) -> Self {
        let device: Device = init::make_io_digit(ip_address).into();
        let device = DigitIO::from(device);
        let clapans: [bool; 3] = [
            device.get_turn_clapan(1).unwrap(),
            device.get_turn_clapan(2).unwrap(),
            device.get_turn_clapan(3).unwrap(),
        ];
        IODigit {
            ui: UI::default(),
            device: device,
            clapans: clapans,
        }
    }

    pub fn update(&mut self, message: Message) {
        use Message::*;
        match message {
        ClapanTurn(num, enb) => {
            if let 0..=2 = num {
                self.device.turn_clapan(num+1, enb);
                let enb = self.device.get_turn_clapan(num+1).unwrap();
                self.clapans[num as usize] = enb;
            }
        },
        }
    }

    pub fn view(&mut self) -> Element<Message> {
        let mut res = Column::new()
            .spacing(20)
            .align_items(Align::Center)
            .push(Text::new(self.device.device().name()))
            .push(Text::new(format!("Values: {}", self.device.device().values_map().len())))
            ;
        res = if self.device.device().is_connect() {
            let check = |num, txt: &str| Checkbox::new(self.clapans[num as usize], txt, move |enb:bool| Message::ClapanTurn(num, enb));
            res.push(check(2, "Clapan - 1"))
                .push(check(0, "Clapan - 2"))
                .push(check(1, "Clapan - 3"))
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
