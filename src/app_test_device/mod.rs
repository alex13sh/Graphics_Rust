#![allow(unused_imports)]

use iced::{
    Align, Column, Row, Scrollable, scrollable, Container, Element, Length,
    Text, text_input, TextInput, button, Button, 
    Application, window, Settings, executor, Subscription, Command, time,
};

mod test_invertor;
mod owen;

pub enum TestDeviceApp {
    Connect {
        input_ip_address: text_input::State,
        ip_address: String,
        pb_connect: button::State,
    },
    TestInvertor (test_invertor::TestInvertor),
    TestDigitIO (owen::io_digit::IODigit),
    TestPWM(owen::pwm::PWM_Gui),
}
#[derive(Debug, Clone)]
pub enum Message {
    InputIpAddressChanged(String),
    Connect,
    
    Invertor(test_invertor::Message),
    DigitIO(owen::io_digit::Message),
    PWM_GUI(owen::pwm::Message),
}

impl Application for TestDeviceApp {
    type Executor = executor::Default;
    type Flags = ();
    type Message = Message;

    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
        (
            Self::Connect {
                input_ip_address: text_input::State::new(),
                ip_address: "192.168.1.10".into(),
                pb_connect: button::State::new(),
            },
            Command::none()
        )
    }
    fn title(&self) -> String {
        String::from("TestDeviceApp - Iced")
    }
    
    fn subscription(&self) -> Subscription<Message> {
        match self {
        Self::TestInvertor (invertor) => invertor.subscription().map(Message::Invertor),
        Self::TestDigitIO (device) => device.subscription().map(Message::DigitIO),
        _ => Subscription::none(),
        }
    }
    
    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match self {
        Self::Connect {ip_address, ..} => match message {
            Message::InputIpAddressChanged(txt) => *ip_address = txt,
            Message::Connect => *self = Self::TestPWM ( owen::pwm::PWM_Gui::new(ip_address.clone())),
//             Message::Connect => *self = Self::TestDigitIO ( owen::io_digit::IODigit::new(ip_address.clone())),
            _ => {}
            },
        Self::TestInvertor (invertor) => match message {
            Message::Invertor(message) => invertor.update(message),
            _ => {}
            },
        Self::TestDigitIO (device) => match message {
            Message::DigitIO(message) => device.update(message),
            _ => {}
            },  
        Self::TestPWM (device) => match message {
            Message::PWM_GUI(message) => device.update(message),
            _ => {}
            },  
        };
        Command::none()
    }
    fn view(&mut self) -> Element<Self::Message> {
        let content: Element<_> = match self {
        Self::Connect {ip_address, input_ip_address, pb_connect} => {
            let input = TextInput::new(
                input_ip_address,
                "Введите IP адрес",
                ip_address,
                Message::InputIpAddressChanged,
            ).padding(10)
            .on_submit(Message::Connect);
            
            let connect = Button::new(pb_connect, Text::new("Подключиться"))
                .on_press(Message::Connect);
                
//                 let text = Text::new("My Text");
            Row::new()
                .spacing(20)
                .align_items(Align::Center)
                .push(input)
                .push(connect)
                .into()
        },
        Self::TestInvertor (invertor) => {
            invertor.view().map(Message::Invertor)
        },
        Self::TestDigitIO (device) => {
            device.view().map(Message::DigitIO)
        },
        Self::TestPWM (device) => {
            device.view().map(Message::PWM_GUI)
        }
        };
        Container::new(content)
            .width(Length::Fill).height(Length::Fill)
            .padding(10)
            .center_x().center_y()
            .into()
    }
}
