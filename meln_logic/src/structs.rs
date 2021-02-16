#![allow(dead_code)]

use epoxy::property::*;
use epoxy::{binding, Sink};

use std::collections::HashMap;

pub(crate) struct Properties {
    pub properties: HashMap<String, PropertyRead<f32>>,
    pub(crate) sinks: HashMap<String, Sink<f32>>,
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

trait Sinks<T> {
    fn emit(&self, name: &str, value: T);
}

impl Sinks<f32> for Properties {
    fn emit(&self, name: &str, value: f32) {
        self.sinks[&String::from(name)].emit(value);
    }
}

pub struct Engine {
    pub speed: PropertyRead<u32>, // f32
    
    pub vibra: PropertyRead<f32>,
    pub temp_stator: PropertyRead<f32>,
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
    fn emit(&self, name: &str, value: f32) {
        self.props.emit(name, value);
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

impl Engine {
    pub(crate) fn new() -> Self {
        let props = ["speed", "vibra", "temp_stator", "temp_rotor",
            "temp_oil_out", "temp_podshipnik", "davl_oil_podshipnik_bottom", "davl_oil_podshipnik_top"];
        let props = Properties::new(&props);
        
        Engine {
            speed: Default::default(),
            
            vibra: props.prop("vibra"),
            temp_stator: props.prop("temp_stator"),
            temp_rotor: props.prop("temp_rotor"),
            
            temp_oil_out: props.prop("temp_oil_out"),
            temp_podshipnik: props.prop("temp_podshipnik"),
            davl_oil_podshipnik_bottom: props.prop("davl_oil_podshipnik_bottom"),
            davl_oil_podshipnik_top: props.prop("davl_oil_podshipnik_top"),
            
            props: props,
        }
    }
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
    fn emit(&self, _name: &str, value: f32) {
        self.sink_davl.emit(value);
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

