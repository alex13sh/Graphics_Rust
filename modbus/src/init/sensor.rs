#![allow(non_camel_case_types, dead_code)]

// use super::Device;
use super::Value;

#[derive(Debug)]
pub enum ValueGroup {
    Sensor {
        name: String,
        pin: u8,
        interval: u16,
    //     pub range: std::Range, 
        value_error: super::ValueError,
        sensor_type: SensorType,
    },
    GroupPin {
        name: String,
        pin: u8,
        group_type: GroupValueType
    },
    Group {
        name: String,
        values: Vec<Value>,
    },
    SensorValues(SensorValues),
    GroupPinValues(GroupPinValues),
}

#[derive(Debug)]
pub struct SensorValues {
    pub name: String,
    pub pin: u8,
    pub interval: u16,
    //     pub range: std::Range, 
    pub value_error: super::ValueError,
    pub sensor_type: SensorType,
    pub values: Vec<Value>,
}

#[derive(Debug)]
pub struct GroupPinValues {
    pub name: String,
    pub pin: u8,
    pub group_type: GroupValueType,
    pub values: Vec<Value>,
}

#[derive(Debug)]
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
    
    Counter (u32),
}

impl Default for SensorType {
    fn default() -> SensorType {
        SensorType::Analog(SensorAnalogType::Amper_4_20)
    }
}

#[derive(Debug)]
pub enum GroupValueType {
    DigitalOutput(bool),
    PWM(u32), // interval
}

impl Default for GroupValueType {
    fn default() -> GroupValueType {
        GroupValueType::DigitalOutput(false)
    }
}


