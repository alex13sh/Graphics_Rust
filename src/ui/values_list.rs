use modbus::{Value, ValueArc, ModbusValues, ValueError, ValueFloatResult};
use super::style;

use std::collections::{BTreeMap};
use std::sync::Arc;

use iced::{
    Element, Text,
    Container, Column, Row, Space, Length,
};

#[macro_export]
macro_rules! map(
  { $T:ident, $($key:expr => $value:expr),+ } => {
    {
      let mut m = $T::new();
      $(
        m.insert(
            $key.into(),
            ($value).into_iter().map(|&t| t.into()).collect()
        );
      )+
      m
    }
 };
);

pub struct ValuesList {
    pub name: String,
    pub values: Vec<ValueArc>,
}

// pub type Message = crate::Message;
impl ValuesList {

    pub fn view<'a, Message: 'a>(&'a self) -> Element<'a, Message> {
        self.view_with_style( Style {
            text_size: 34,
            column_width: 700,
        })
    }

    pub fn view_with_style<'a, Message: 'a>(&'a self, style: Style) -> Element<'a, Message> {
        let mut lst = Column::new()
            .width(Length::Units(style.column_width))
            .spacing(2);
        for v in &self.values {
//             dbg!(v.name());
            lst = lst.push(Self::view_value(v, &style));
        }
        lst.into()
    }
    fn view_value<'a, Message: 'a>(value: &ValueArc, style: &Style) -> Element<'a, Message> {
        pub use std::convert::TryFrom;
        let err = value.get_error_min_max();
        let name = value.name().unwrap();
        let suffix_name = if let Some(txt) = value.suffix_name() {format!("({})", txt)} else {String::from("")};
        let value = f32::try_from(value.value().as_ref());
        let color;
        let txt_value;
        match value {
        Ok(value) => {
            color = match err {
            (None, Some(max)) if max.red <= value =>
                    [1.0, 0.0, 0.0],
            (None, Some(max)) if max.yellow <= value =>
                    [1.0, 1.0, 0.0],
            (Some(min), Some(max)) if min.red >= value || max.red <= value =>
                    [1.0, 0.0, 0.0],
            (Some(min), Some(max)) if min.yellow >= value || max.yellow <= value =>
                    [1.0, 1.0, 0.0],
            _ => [0.0, 0.8, 0.0],
            };
            txt_value = format!("Value: {:.2} {}", value, suffix_name);
        },
        Err(e) => {
            color = [1.0, 0.0, 0.0];
            txt_value = format!("Error: {:?}", e);
        }}
        let text = Text::new(
            format!("{}\n{}", name, txt_value)
        ).size(style.text_size)
        .color(color);

        Container::new(text)
            .width(Length::Fill)
            .style(style::ValueContainer)
            .into()
    }
}

pub fn make_value_lists(modbus_values: &ModbusValues, values_groups: BTreeMap<String, Vec<String>>) -> Vec<ValuesList> {
    values_groups.into_iter()
        .map(|(name, values)|
            ValuesList {
                name: name,
                values: values.into_iter().flat_map(|name| { 
                    modbus_values.get_value_arc(&name)
                }).collect(),
            }
        ).collect()
}

pub fn make_value_lists_start(modbus_values: &ModbusValues, values_groups: BTreeMap<String, Vec<String>>) -> Vec<ValuesList> {
    values_groups.into_iter()
        .map(|(name, values)|
            ValuesList {
                name: name,
                values: values.into_iter().flat_map(|name| { 
                    modbus_values.get_value_arc_starts(&name)
                }).collect(),
            }
        ).collect()
}

pub fn make_value_lists_start_2(modbus_values: &ModbusValues, values_groups: BTreeMap<String, Vec<String>>) -> Vec<ValuesList> {
    values_groups.into_iter()
        .map(|(name, values)|
            ValuesList {
                name: name,
                values: modbus_values
                    .get_values_by_name_starts(&values.iter().map(|n| &n[..]).collect::<Vec<_>>())
                    .get_values_by_name_ends(&["/value"]).into(),
            }
        ).collect()
}

#[derive(Clone)]
pub struct Style {
    pub text_size: u16,
    pub column_width: u16,
}
