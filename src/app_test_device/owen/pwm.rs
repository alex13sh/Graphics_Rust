#![allow(unused_imports)]

use iced::{
    Align, Column, Row, Scrollable, scrollable, Container, Element, Length,
    Text, text_input, TextInput, button, Button, slider, Slider,
    Application, window, Settings, executor, Subscription, Command, time,
};

use modbus::init;
use modbus::{Device, DigitIO};
use modbus::DeviceInner;
use modbus::{Value, ModbusValues};

pub struct PWM_Gui {
    ui: UI,
    device: DigitIO,
    hz: f32,
}

#[derive(Default)]
struct UI {
    hz: slider::State,
}

#[derive(Debug, Clone)]
pub enum Message {
    PwmHzChanged(f32),
    Update,
}

impl PWM_Gui {
    pub fn new(ip_address: String) -> Self {
        let device: Device = init::make_io_digit(ip_address).into();
        let mut device = DigitIO::from(device);
        device.config_genetaror_hz(4);
        PWM_Gui {
            ui: UI::default(),
            device: device,
            hz: 0_f32,
        }
    }

    pub fn subscription(&self) -> Subscription<Message> {
        time::every(std::time::Duration::from_millis(500))
            .map(|_| Message::Update)
    }
    
    pub fn update(&mut self, message: Message) {
        use Message::*;
        match message {
        PwmHzChanged(hz) => {
//             if self.hz == 0_f32 {
//                 self.device.config_genetaror_hz(5);
//             }
            if let Ok(()) = self.device.set_hz(hz) {
                self.hz = hz;
            }
        },
        Update => {
            use modbus::DeviceError;
            println!("Message::Update");
            if let Err(error) = self.device.device().update() {
                dbg!(&error);
//                 self.ui.error = Some(format!("Error: {}", error));
            } // else { self.ui.error = None; }
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
            
            res.push(Text::new(format!("hz: {}", self.hz)))
                .push(Slider::new(
                &mut self.ui.hz,
                0.0 ..= 1.0,
                self.hz,
                Message::PwmHzChanged
            ).step(0.1))
        } else {
            res.push(Text::new(format!("Устройство не подключено!\nIP Address: {}", self.device.device().get_ip_address())))
        };
        res.into()
    }
}
