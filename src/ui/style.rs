use iced::{button, container, Background, Color, Vector};

pub enum Button {
    Klapan {
        enabled: bool,
        checked: bool
    },
    Check {
        checked: bool
    },
    Exit
}

fn button_style_color(color: Color) -> button::Style {
    button::Style {
        background: Some(Background::Color(color)),
        border_radius: 10_f32,
        text_color: Color::WHITE,
        ..button::Style::default()
    }
}

fn button_style_checked(checked: bool) -> button::Style {
    if checked {
        button_style_color(Color::from_rgb8(150, 0,0))
    } else {
        button_style_color(Color::from_rgb8(0, 150, 0))
    }
}

impl button::StyleSheet for Button {
    fn active(&self) -> button::Style {
        let enabled = &true;
        match self {
        Button::Check { checked } => button_style_checked(*checked),
        Button::Klapan { enabled, checked } => if *enabled {
                button_style_checked(*checked)
            } else {
                button_style_color(Color::from_rgb8(150, 150, 150))
        },
        Button::Exit => button_style_color(Color::from_rgb8(150, 0,0)),
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
