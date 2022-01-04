use crate::utils::{DateTimeFix, date_time_from_str, date_time_to_str};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
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

pub type LogValueRaw = ValueDate<raw::Value>;
pub type RawValuesLine = ValuesLine<raw::Value>;

pub mod raw {
    use super::*;
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Value {
        pub full_name: String,
        pub value_u32: u32,
    //     pub value_f32: f32,
    }
    #[derive(Serialize, Deserialize, Debug, Clone)]
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
        pub value: f32,
    }
    
    impl Value {
        pub fn get_sensor_value(&self) -> (&str, f32) {
            (&self.sensor_name, self.value)
        }
    }
}

pub mod invertor {
    use super::*;
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct InvertorParametr {
        pub address: String, //(u8, u8),
        pub value: u32,
        pub name: String,
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
}

pub mod iterators {
//     use std::pin::Pin;
    
    fn fiban_iter(vin: impl Iterator<Item=u32>) -> impl Iterator<Item=f32> 
    {
        let mut sum = 0;
//         std::iter::from_fn(move || {
//             let v = vin.next()?;
//             sum += v;
//             Some(sum  as f32)
//         })
        vin.map(move |v| {sum = sum+v; sum as f32})
    }
    
    #[test]
    fn test_iter() {
        let arr = 0..10;
        let arr: Vec<f32> = fiban_iter(arr.into_iter()).collect();
        dbg!(arr);
//         assert!(false);
    }
    
    use futures::stream::{Stream, StreamExt};
    
    fn fiban_stream(vin: impl Stream<Item=u32>) -> impl Stream<Item=f32> 
    {
        let mut sum = 0;
//         futures::stream::poll_fn(move |_| {
//             let v = vin.next()?;
//             sum += v;
//             Some(sum  as f32)
//         })
        vin.map(move |v| {sum = sum+v; sum as f32})
    }

    #[test]
    fn test_stream() {
        let arr = 0..10;
        let arr: Vec<f32> = 
            futures::executor::block_on(fiban_stream(futures::stream::iter(arr)).collect());
        dbg!(arr);
//         assert!(false);
    }
}
