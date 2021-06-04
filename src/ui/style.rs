use iced::{button, container, Background, Color, Vector};

pub enum Button {
    Check { checked: bool },
    Exit
}

impl button::StyleSheet for Button {
    fn active(&self) -> button::Style {
        match self {
        Button::Check { checked } => if *checked {
            button::Style {
                background: Some(Background::Color(
                    Color::from_rgb8(150, 0,0),
                )),
                border_radius: 10_f32,
                text_color: Color::WHITE,
                ..button::Style::default()
            }
        } else {
            button::Style {
                background: Some(Background::Color(
                    Color::from_rgb8(0, 150, 0),
                )),
                border_radius: 10_f32,
                text_color: Color::WHITE,
                ..button::Style::default()
            }
        },
        Button::Exit => button::Style {
            background: Some(Background::Color(
                Color::from_rgb8(150, 0,0),
            )),
            border_radius: 10_f32,
            text_color: Color::WHITE,
            ..button::Style::default()
        }
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

pub struct MainContainer;
impl container::StyleSheet for MainContainer {
    fn style(&self) -> container::Style {
        container::Style {
            background: Some(Background::Color([0.8, 0.8, 0.8].into())),
            .. Default::default()
        }
    }
}

pub struct ValueContainer;
impl container::StyleSheet for ValueContainer {
    fn style(&self) -> container::Style {
        container::Style {
            background: Some(Background::Color([0.3, 0.3, 0.3].into())),
            .. Default::default()
        }
    }
}
