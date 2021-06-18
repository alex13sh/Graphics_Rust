 
use iced::{
    Element, Text, button, Button, slider, Slider,
    Column, Row, Space, Length,
    Command
};

use std::sync::Arc;

pub struct Dozator {
    ui: UI,
    shim_hz: i32,
    shim_hz_enb: bool,
    device: meln_logic::devices::Dozator,
}

#[derive(Default)]
struct UI {
    shim_hz: slider::State,
}


#[derive(Debug, Clone)]
pub enum Message {
    ShimHzChanged(i32),
    SetShimHz, SetShimHzFinished,
}

impl Dozator {
    pub fn new(device: meln_logic::devices::Dozator) -> Self {
        Dozator {
            ui: UI::default(),
            shim_hz: 0,
            shim_hz_enb: true,
            device: device,
        }
    }

    pub fn update(&mut self, message: Message, devices: Vec<Arc<modbus::Device>>)  -> Command<Message> {
        match message {
        Message::ShimHzChanged(hz) => if self.shim_hz_enb {
            self.shim_hz_enb = false;
            self.shim_hz = hz;
            use futures_util::pin_mut;
            use futures_util::stream::StreamExt;
            let s = self.device.set_value(hz);
//             let devices = vec![self.logic.digit_o.device().clone()];//self.logic.get_devices();
            let f =  async move {
                pin_mut!(s);
                while let Some(value) = s.next().await {
                    println!("Dozator: new value: {}", value);
                    if let Err(_) = meln_logic::init::Complect::update_new_values_static(&devices) {
                        println!("Dozator: after update; Error!!");
                    } else {println!("Dozator: after update; Ok!");}
                };
                println!("Dozator: finished!");
                return Message::SetShimHzFinished;
            };
//             return Command::perform(f, move |_| Message::SetShimHzFinished);
            return f.into();
        },
        Message::SetShimHz => {
            println!("Set HZ: {}", self.shim_hz);
        },
        Message::SetShimHzFinished => self.shim_hz_enb = true,
        }
        Command::none()
    }
    pub fn view(&mut self) -> Element<Message> {
        let slider = {
            let slider = Slider::new(
                &mut self.ui.shim_hz,
                -20..=20,
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
