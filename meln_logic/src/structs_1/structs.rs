#![allow(dead_code)]

use epoxy::property::*;
use epoxy::{binding, Sink};

use std::collections::HashMap;

pub type ValueSink = Sink<f32>;
pub type ValueRead = PropertyRead<f32>;

pub(crate) struct Properties {
    pub properties: HashMap<String, ValueRead>,
    pub(crate) sinks: HashMap<String, ValueSink>,
}

pub trait PropertiesExt {
    fn init_props() -> Self;
    fn get_props(&self) -> String;
}

impl Properties {
    pub(crate) fn new(props: &[&'static str]) -> Self {
        let mut properties = HashMap::new();
        let mut sinks = HashMap::new();
        
        for prop in props.into_iter().map(|s| String::from(*s)) {
            let sink = Sink::new();
            properties.insert(prop.clone(), 
                Property::from_stream(sink.get_stream()));
            sinks.insert(prop.clone(), sink);
        }
        Properties{
            properties: properties,
            sinks: sinks
        }
    }
    
    pub(crate) fn prop(&self, name: &str) -> PropertyRead<f32> {
        self.properties[&String::from(name)].clone()
    }
}

pub trait Sinks<T> {
    fn emit(&self, name: &str, value: T) -> bool;
}

impl Sinks<f32> for Properties {
    fn emit(&self, name: &str, value: f32) -> bool {
        if let Some(s) = self.sinks.get(&String::from(name)) {
            s.emit(value);
            true
        } else {false}
    }
}

#[derive(macros::PropertiesExt)]
pub struct Engine {
    pub speed: PropertyRead<u32>, // f32
    
    #[props] pub vibra: PropertyRead<f32>,
    #[props] pub temp_stator: PropertyRead<f32>,
    pub temp_rotor: PropertyRead<f32>,
//     temp_oil_in: PropertyRead<f32>,
    pub temp_oil_out:    PropertyRead<f32>,
    pub temp_podshipnik: PropertyRead<f32>, // bearing
    
    // davl -> pressure; podshipnik -> bearing
    pub davl_oil_podshipnik_bottom: PropertyRead<f32>,
    pub davl_oil_podshipnik_top:    PropertyRead<f32>,
    
//     pub(crate) sinks: HashMap<String, Sink<f32>>, // String -> PropertyRead<f32>
    pub(crate) props: Properties,
}

impl Sinks<f32> for Engine {
    fn emit(&self, name: &str, value: f32) -> bool {
        self.props.emit(name, value)
    }
}

pub enum EngineUpdateMessage {
    Speed(u32),
    Vibra(f32),
    TempStator(f32), TempRotor(f32),
    TempPodshipnik(f32),
    TempOilOut(f32),
    
    DavlOilPodshipnikBottom(f32),
    DavlOilPodshipnikTop(f32),
}

// pub struct Oil {
//     temp_dvij_out: PropertyRead<f32>,
//     temp_station_out: PropertyRead<f32>,
// }

pub struct Vacum {
    pub davl: PropertyRead<f32>,
    sink_davl: Sink<f32>,
}
impl Vacum {
    pub fn new() -> Self {
        let s = Sink::new();
        let davl = Property::from_stream(s.get_stream());
        s.emit(1_000_f32);
        Vacum {
            davl: davl,
            sink_davl: s,
        }
    }
    pub fn down(&self) {
        self.sink_davl.emit(4.5);
    }
    pub fn up(&self) {
        self.sink_davl.emit(1_000_f32);
    }
    pub fn stop(&self) {}
}

impl Sinks<f32> for Vacum {
    fn emit(&self, name: &str, value: f32) -> bool {
        if name == "davl" {
            self.sink_davl.emit(value);
            true
        } else {false}
    }
}

pub struct Invertor {
    pub hz: PropertyWrite<u16>,
}

impl Invertor {
    pub fn stop(&self) {
        self.hz.set(0);
    }
}

