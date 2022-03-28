use crate::utils::{DateTimeFix, date_time_from_str, date_time_to_str, float_to_str};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Value {
    pub device_id: u16,
    pub device_name: String,
    pub sensor_name: String,
    pub value_name: String,
    pub value_f32: f32,
    pub value_u32: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ValueDate<Value> {
        #[serde(deserialize_with = "date_time_from_str")]
        #[serde(serialize_with = "date_time_to_str")]
        pub date_time: DateTimeFix,
        #[serde(flatten)]
        pub value: Value,
        
}

impl <V> std::ops::Deref for ValueDate<V> {
    type Target = V;
    fn deref(&self) -> &V {
        &self.value
    }
}

// impl <VF, VT> From<ValueDate<VF>> for ValueDate<VT> 
// where VT: From<VF>
// {
//     fn from(v: ValueDate<VF>) -> Self {
//         Self {
//             date_time: v.date_time,
//             value: v.value.into(),
//         }
//     }
// }

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ValuesLine<Value> {
    #[serde(deserialize_with = "date_time_from_str")]
    #[serde(serialize_with = "date_time_to_str")]
    pub date_time: DateTimeFix,
    pub values: Box<[Value]>,
}

impl <V> ValuesLine <V> {
    pub fn into_values_date(self) -> impl Iterator<Item = ValueDate<V>> 
    {
        let dt = self.date_time;
        let values = self.values.into_vec();
        values.into_iter()
            .map(move |v| ValueDate {
                date_time: dt.clone(),
                value: v,
            })
    }
}
impl <V: Clone> ValuesLine <V> {
    pub fn iter_values_date(&self) -> impl Iterator<Item = ValueDate<V>> + '_ 
    {
        self.values.iter()
            .map(|v| ValueDate {
                date_time: self.date_time.clone(),
                value: (*v).clone(),
            })
    }
}

impl <V> From<Box<[V]>> for ValuesLine <V> {
    fn from(v: Box<[V]>) -> Self {
        Self {
            date_time: crate::utils::date_time_now(),
            values: v,
        }
    }
}

pub type LogValueFull = ValueDate<Value>;
pub type LogValuesFull = ValuesLine<Value>;
pub type LogValueRaw = ValueDate<raw::Value>;
pub type LogValueRawOld = ValueDate<raw::ValueOld>;
pub type RawValuesLine = ValuesLine<raw::Value>;

pub mod raw {
    use super::*;
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Value {
        pub full_name: String,
        pub value_u32: u32,
    //     pub value_f32: f32,
    }
    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    pub struct ValueOld {
        pub hash: String,
        pub value: f32,
    }
    impl From<ValueOld> for Value {
        fn from(v: ValueOld) -> Self {
            Self {
                full_name: v.hash,
                value_u32: v.value as u32,
            }
        }
    }
    
    impl TryFrom<ValueOld> for super::elk::Value {
        type Error = String;
        fn try_from(v: ValueOld) -> Result<Self, String> {
            if let Some(id) = crate::convert::value::hash_to_names(&v.hash) {
                Ok(Self {
                    device_id: id.0,
                    device_name: id.1,
                    sensor_name: id.2,
                    
                    value: v.value,
                })
            } else {
                let e = Err(format!("Нет соответствия для hash: {}", v.hash));
                // dbg!(&e);
                e
            }
        }
    }
}

pub type LogValueHum = ValueDate<elk::Value>;
pub type ElkValuesLine = ValuesLine<elk::Value>;

pub mod elk {
    use super::*;
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Value {
        pub device_id: u16,
        pub device_name: String,
        #[serde(rename = "value_name")]
        pub sensor_name: String,
    //     pub value_name: String,
        #[serde(serialize_with = "float_to_str")]
        pub value: f32,
    }
    
    impl Value {
        pub fn get_sensor_value(&self) -> (&str, f32) {
            (&self.sensor_name, self.value)
        }
    }
    
    impl From<super::Value> for Value {
        fn from(v: super::Value) -> Self {
            Value {
                device_id: v.device_id,
                device_name: v.device_name,
                sensor_name: v.sensor_name,
                value: v.value_f32,
            }
        }
    }
}

pub type LogValueSimple = ValueDate<simple::Value>;
pub type SimpleValuesLine = ValuesLine<simple::Value>;
pub mod simple {
    use super::*;
    #[derive(Debug, Clone)]
    pub struct Value {
        pub sensor_name: String,
        pub value: f32,
    }
    #[derive(Debug, Clone)]
    pub struct ValueStr<'s> {
        pub sensor_name: &'s str,
        pub value: f32,
    }
    impl Value {
        pub fn as_ref(&self) -> ValueStr {
            ValueStr {
                sensor_name: self.sensor_name.as_str(),
                value: self.value,
            }
        }
    }
    
    use std::collections::BTreeMap;
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct ValuesMap {
        #[serde(deserialize_with = "date_time_from_str")]
        #[serde(serialize_with = "date_time_to_str")]
        pub date_time: DateTimeFix,
        #[serde(flatten)]
        pub values: BTreeMap<String, String>,
    }
}

pub mod invertor {
    use super::*;

    #[derive(Serialize, Deserialize, Debug, Clone, PartialOrd, PartialEq, Ord, Eq, Copy)]
    pub struct InvertorAddress(pub u8, pub u8);

    impl From<(u8, u8)> for InvertorAddress {
        fn from(adr: (u8, u8)) -> Self {
            InvertorAddress(adr.0, adr.1)
        }
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct InvertorParametr {
    #[serde(deserialize_with = "adr_from_str")]
    #[serde(serialize_with = "adr_to_str")]
        pub address: InvertorAddress, //(u8, u8),
        pub name: String,
        pub value: u32,
        pub value_f32: f32,
    }

    use serde::{de, de::Error, Deserializer, Serializer};
    pub(crate) fn adr_from_str<'de, D>(deserializer: D) -> Result<InvertorAddress, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = Deserialize::deserialize(deserializer)?;
        let adr = s.trim_matches(|c| c == '(' || c==')').split_once(',').ok_or("Address Invalid").map_err(D::Error::custom)?;
        Ok(InvertorAddress (
            adr.0.trim().parse().map_err(D::Error::custom)?,
            adr.1.trim().parse().map_err(D::Error::custom)?
        ))
    }

    pub(crate) fn adr_to_str<S>(adr: &InvertorAddress, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
    //     let s = dt.to_rfc3339_opts(SecondsFormat::Millis, false);
        let s = format!("({}, {})", adr.0, adr.1);
        serializer.serialize_str(&s)
    }
}

pub mod device {
    pub struct Value {
        pub address: u16,
        pub sensor_name: String,
        pub value_name: String,
        pub value_u32: u32,
    }
    pub struct Device {
         pub device_id: u16,
         pub device_name: String,
         pub values: Box<[Value]>,
    }

    pub struct InvertorParametr {
        pub address: String,
        pub value: u32,
        pub name: String,
    }
}
