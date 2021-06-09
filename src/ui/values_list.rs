use modbus::{Value, ValueArc, ModbusValues, ValueError};
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
        let mut lst = Column::new().width(Length::Units(250)).spacing(2);
        for v in &self.values {
            dbg!(v.name());
            lst = lst.push(Self::view_value(v));
        }
        lst.into()
    }
    fn view_value<'a, Message: 'a>(value: &ValueArc) -> Element<'a, Message> {
        pub use std::convert::TryFrom;
        let err = value.get_error();
        let valuef = f32::try_from(value.value().as_ref()).unwrap();
        let color = match err {
            Some(err) if err.red <= valuef =>
                [1.0, 0.0, 0.0],
            Some(err) if err.yellow <= valuef =>
                [1.0, 1.0, 0.0],
            Some(_) | None => [0.0, 0.8, 0.0],
        };
        let text = Text::new(
            format!("{}\nValue: {:.2}", value.name().unwrap(), valuef)
        ).size(20)
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
                    dbg!(&name);
                modbus_values.get_value_arc(&name)
                }).collect(),
            }
        ).collect()
}
