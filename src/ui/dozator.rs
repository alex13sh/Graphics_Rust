 
use iced::{
    Element, Text, button, Button, slider, Slider,
    Column, Row, Space, Length,
};

pub struct Dozator {
    ui: UI,
    shim_hz: u32,
    shim_hz_enb: bool,
    values: modbus::ModbusValues,
}

#[derive(Default)]
struct UI {
    shim_hz: slider::State,
}


#[derive(Debug, Clone)]
pub enum Message {
    ShimHzChanged(u32),
    SetShimHz,
}

impl Dozator {
    pub fn new(values: modbus::ModbusValues) -> Self {
        Dozator {
            ui: UI::default(),
            shim_hz: 0,
            shim_hz_enb: true,
            values: values,
        }
    }

    pub fn update(&mut self, message: Message) {
        match message {
        Message::ShimHzChanged(hz) => self.shim_hz = hz,
        Message::SetShimHz => {
            println!("Set HZ: {}", self.shim_hz);
        }
        }
    }
    pub fn view(&mut self) -> Element<Message> {
        let slider = {
            let slider = Slider::new(
                &mut self.ui.shim_hz,
                0..=20,
                self.shim_hz,
                Message::ShimHzChanged
            )
            .on_release(Message::SetShimHz)
            .step(1);

            Column::new().spacing(5)
                .push(
                    Row::new().spacing(20)
                        .push(Text::new(format!("Частота ШИМ: {:0>5}", self.shim_hz)))
                        .push(slider)
                )
        };
        slider.into()
    }
}
