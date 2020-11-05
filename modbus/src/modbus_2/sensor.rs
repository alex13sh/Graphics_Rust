use super::Value;
// use super::Device;

#[derive(Default, Debug)]
pub struct Sensor {
    pub name: String,
    pub values: Vec<Value>,
//     pub range: std::Range, 
    pub value_error: ValueError,
    pub sensor_type: SensorType,
}

pub struct ValueError {
    pub yellow: f32,
    pub red: f32
}

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

pub enum SensorType {
    Perometr (SensorAnalogType),
    Vibra (SensorAnalogType),
    Davl (SensorAnalogType),
    
    DigitalOutput,
    Counter (u32),
}

impl SensorType {
    fn value_float (&self) -> f32 {
        match *self {
            Davl(typ) => {
                // v = pow(10, v*10-5.5);
                0_f32
            }
            _ => 0_f32
        }
    }
}
