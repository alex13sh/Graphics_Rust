// use super::Device;

#[derive(Default, Debug)]
pub struct Sensor {
    pub name: String,
    pub pin: u8,
    pub interval: Option<u16>,
//     pub range: std::Range, 
    pub value_error: Option<ValueError>,
    pub sensor_type: SensorType,
}

#[derive(Default, Debug)]
pub struct ValueError {
    pub yellow: f32,
    pub red: f32
}

impl From<(f32, f32)> for ValueError {
    fn from((y, r): (f32, f32)) -> Self {
        Self {yellow: y, red: r}
    }
}
impl From<(i32, i32)> for ValueError {
    fn from((y, r): (i32, i32)) -> Self {
        Self {yellow: y as f32, red: r as f32}
    }
}


#[derive(Debug)]
#[allow(dead_code, snake_case)]
pub enum SensorAnalogType {
    
    Amper_4_20=11,
    Amper_0_20,
    Amper_0_5,

    Volt_1=14,
    Resister_0_2=38,
    Resister_0_5=39,

    Pt_50=8,
    Pt_100=3,
    Pt_500=30,
    Pt_1000=35,

    Cu_50=2,
    Cu_100=1,
    Cu_500=28,
    Cu_1000=33,
    
}

enum SensorDigitalInputType {

}

enum SensorDigitalOutputType {

}

#[derive(Debug)]
pub enum SensorType {
    Analog (SensorAnalogType),
    Perometr (SensorAnalogType),
    Vibra (SensorAnalogType),
    Davl (SensorAnalogType),
    
    DigitalOutput(bool),
    Counter (u32),
}

impl Default for SensorType {
    fn default() -> SensorType {
        SensorType::Analog(SensorAnalogType::Amper_4_20)
    }
}

