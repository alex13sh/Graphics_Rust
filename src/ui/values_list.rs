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
        let mut lst = Column::new().width(Length::Units(550)).spacing(2);
        for v in &self.values {
            dbg!(v.name());
            lst = lst.push(Self::view_value(v));
        }
        lst.into()
    }
    fn view_value<'a, Message: 'a>(value: &ValueArc) -> Element<'a, Message> {
        pub use std::convert::TryFrom;
        let err = value.get_error();
        let name = value.name().unwrap();
        let value = f32::try_from(value.value().as_ref());
        let color;
        let txt_value;
        match value {
        Ok(value) => {
            color = match err {
                Some(err) if err.red <= value =>
                    [1.0, 0.0, 0.0],
                Some(err) if err.yellow <= value =>
                    [1.0, 1.0, 0.0],
                Some(_) | None => [0.0, 0.8, 0.0],
            };
            txt_value = format!("Value: {:.2}", value);
        },
        Err(e) => {
            color = [1.0, 0.0, 0.0];
            txt_value = format!("Error: {:?}", e);
        }}
        let text = Text::new(
            format!("{}\n{}", name, txt_value)
        ).size(28)
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
                    modbus_values.get_value_arc_stars(&name)
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
