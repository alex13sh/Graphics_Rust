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
        // forward, trasparent
        pub value: Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ValuesLine<Value> {
    #[serde(deserialize_with = "date_time_from_str")]
    #[serde(serialize_with = "date_time_to_str")]
    pub date_time: DateTimeFix,
    pub values: Box<[Value]>,
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
//     use std::ops::{Generator, GeneratorState};
//     use std::pin::Pin;
    
    fn convert(mut vin: impl Iterator<Item=u32>) -> impl Iterator<Item=f32> 
    {
        let mut sum = 0;
        std::iter::from_fn(move || {
            let v = vin.next()?;
            sum += v;
            Some(sum  as f32)
        })
    }
    
    #[test]
    fn test_convert() {
        let arr = 0..10;
        let arr: Vec<f32> = convert(arr.into_iter()).collect();
        dbg!(arr);
        assert!(false);
    }
//     pub fn gen(vin: impl Iterator<Item=u32>) -> impl Iterator<Item=f32> {
//         || {
//             for i in vin {
//                 yield i as f32;
//             }
//         }
//     }
    
//     impl <T> Iterator for T 
//     where T: Generator<Return = ()>,
//     {
//         type Item = <T as Generator>::Yield;
//         fn next(&mut self) -> Option<Self::Item> {
//             match self.resume(()) {
//                 GeneratorState::Yielded(x) => Some(x),
//                 GeneratorState::Complete(_) => None,
//             }
//         }
//     }
//     
//     struct GeneratorIteratorAdapter<G>(Pin<Box<G>>);
// 
//     impl<G> GeneratorIteratorAdapter<G>
//     where
//         G: Generator<Return = ()>,
//     {
//         fn new(gen: G) -> Self {
//             Self(Box::pin(gen))
//         }
//     }
//     
//     impl <T, I, G> From<I> for GeneratorIteratorAdapter<G>
//     where I: Iterator<T>,
//     G: Generator<Return = ()>,
//     {
//     
//     }
//     
//     impl<G> Iterator for GeneratorIteratorAdapter<G>
//     where
//         G: Generator<Return = ()>,
//     {
//         type Item = G::Yield;
// 
//         fn next(&mut self) -> Option<Self::Item> {
//             match self.0.as_mut().resume(()) {
//                 GeneratorState::Yielded(x) => Some(x),
//                 GeneratorState::Complete(_) => None,
//             }
//         }
//     }
}
