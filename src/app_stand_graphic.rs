use iced::{
    Application, executor, Command, Subscription, time,
    Element, Container, Text, button, Button,
    Length,
};

pub struct App {
    ui: UI,
    
    is_started: bool,
}

#[derive(Default)]
struct UI {
    start: button::State,
}

#[derive(Debug, Clone)]
pub enum Message {
    ModbusUpdate,
    GraphicUpdate,
    ToggleStart(bool),
    
}

impl Application for App {
    type Executor = executor::Default;
    type Flags = ();
    type Message = Message;
    
    fn new(_flags: ()) -> (Self, Command<Self::Message>) {
    
        (
            Self {
                ui: UI::default(),
                is_started: false,
            },
            Command::none()
        )
    }
    
    fn title(&self) -> String {
        String::from("GraphicsApp - Iced")
    }
    fn subscription(&self) -> Subscription<Self::Message> {
        time::every(std::time::Duration::from_millis(1000))
            .map(|_| Message::GraphicUpdate)
    }
    fn update(&mut self, message: Self::Message) -> Command<Self::Message> {
        match message {
        Message::GraphicUpdate => {},
        Message::ToggleStart(start) => self.is_started = start,
        _ => {}
        };
        Command::none()
    }
    fn view(&mut self) -> Element<Self::Message> {
//         let content = Text::new("Пустое окно");

        let label = if self.is_started {"Stop"} else {"Start"};
        let label = Text::new(label).size(16);
//         let is_started = self.is_started;
        let button =
            Button::new(&mut self.ui.start, label).style(style::Button::Check {
                checked: self.is_started,
            })
            .on_press(Message::ToggleStart(!self.is_started))
            .padding(8);
        
        Container::new(button)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(10)
            .center_x()
            .center_y()
            .into()
    }
}


mod style {
    use iced::{button, Background, Color, Vector};

    pub enum Button {
        Check { checked: bool },
    }

    impl button::StyleSheet for Button {
        fn active(&self) -> button::Style {
            match self {
            Button::Check { checked } => if *checked {
                button::Style {
                    background: Some(Background::Color(
                        Color::from_rgb8(150, 0,0),
                    )),
                    border_radius: 10,
                    text_color: Color::WHITE,
                    ..button::Style::default()
                }
            } else {
                button::Style {
                    background: Some(Background::Color(
                        Color::from_rgb8(0, 150, 0),
                    )),
                    border_radius: 10,
                    text_color: Color::WHITE,
                    ..button::Style::default()
                }
            },
            }
        }

        fn hovered(&self) -> button::Style {
            let active = self.active();

            button::Style {
                text_color: match self {
                Button::Check { checked } if !checked => {
                    Color::from_rgb(0.2, 0.2, 0.7)
                }
                _ => active.text_color,
                },
                shadow_offset: active.shadow_offset + Vector::new(0.0, 1.0),
                ..active
            }
        }
    }
}
