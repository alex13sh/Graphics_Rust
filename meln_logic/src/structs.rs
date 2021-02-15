#![allow(dead_code)]

use epoxy::property::*;
use epoxy::{binding, Sink};

#[derive(Default)]
pub struct Engine {
    pub speed: PropertyRead<u32>,
    
    pub vibra: PropertyRead<f32>,
    pub temp_stator: PropertyRead<f32>,
    pub temp_rotor: PropertyRead<f32>,
//     temp_oil_in: PropertyRead<f32>,
    pub temp_oil_out:    PropertyRead<f32>,
    pub temp_podshipnik: PropertyRead<f32>, // bearing
    
    // davl -> pressure; podshipnik -> bearing
    pub davl_oil_podshipnik_bottom: PropertyRead<f32>,
    pub davl_oil_podshipnik_top:    PropertyRead<f32>,
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

pub struct Invertor {
    pub hz: PropertyWrite<u16>,
}

impl Invertor {
    pub fn stop(&self) {
        self.hz.set(0);
    }
}

