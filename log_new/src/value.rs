
#[derive(Serialize, Deserialize, Debug, Clone)]
struct ValueDate<Value> {
//         #[serde(deserialize_with = "date_time_from_str")]
//         #[serde(serialize_with = "date_time_to_str")]
        pub date_time: DateTimeFix,
        // forward, trasparent
        pub value: Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ValuesLine<Value> {
    pub date_time: DateTimeFix,
    pub values: Box<[Value]>,
}

pub type LogValueRaw = ValueDate<raw::Value>;
pub type RawValuesLine = ValuesLine<raw::Value>;

pub mod raw {
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
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct InvertorParametr {
        pub address: String, //(u8, u8),
        pub value: u32,
        pub name: String,
    }
}

pub mod device {
    pub struct Value {
        pub address: u16
        #[serde(rename = "value_name")]
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
