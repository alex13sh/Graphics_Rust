#![allow(dead_code)]

use epoxy::property::*;
use epoxy::{binding, Sink};

#[derive(Default)]
pub struct Dvij {
    pub speed: PropertyRead<u32>,
    
    vibra: PropertyRead<f32>,
    temp_stator: PropertyRead<f32>,
    temp_rotor: PropertyRead<f32>,
//     temp_oil_in: PropertyRead<f32>,
    temp_oil_out:    PropertyRead<f32>,
    temp_podshipnik: PropertyRead<f32>, // bearing
    
    davl_oil_podshipnik_bottom: PropertyRead<f32>,
    davl_oil_podshipnik_top:    PropertyRead<f32>,
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
    fn new() -> Self {
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

pub use invertor_dvij::InvertorDvij;
pub mod invertor_dvij {
    use super::{Dvij, Vacum, Invertor};
    
    use std::collections::BTreeMap;
    use epoxy::property::*;
    use epoxy::{binding, Sink};
    
    pub struct InvertorDvij {
        pub invertor: Invertor,
        pub dvij: Dvij,
        pub speed: PropertyRead<u32>,
        pub vacum: Vacum,
        
        pub messages: Sink<Message>,
        subs: Vec<epoxy::Subscription<Message>>,
    }
    
    #[derive(Debug, Clone)]
    pub enum Message {
        Start, Stop,
        SpeedChanged(u32),
        VacumDowned(f32),
        VacumUpped,
    }

    impl InvertorDvij {
        pub fn new() -> Self {
            let hz = Property::new(0);
    //         let speed = Property::new(0).as_readonly();
            let speed = binding!{(*hz as u32) * 60};
            let messages = Sink::new();
            
            let mut subs = Vec::new();
            let s = speed.as_stream()
                .map(|speed| Message::SpeedChanged(*speed))
                .pipe_into(&messages);
            subs.push(s);
                
            InvertorDvij {
                invertor: Invertor {
                    hz: hz,
                },
                dvij: Dvij {
                    speed: speed.clone(),
                    .. Dvij::default()
                },
                speed: speed.clone(),
                vacum: Vacum::new(),
                
                messages: messages,
                subs: subs,
            }
        }
        
        pub fn stop(&self) {
            self.invertor.stop();
        }
        
        pub fn set_speed(&self, speed: u32) {
            self.invertor.hz.set((speed/60) as u16);
        }
        pub fn set_hz(&self, hz: u16) {
            self.invertor.hz.set(hz);
        }
        
        pub fn get_values(&self) -> BTreeMap<String, f32> {
            BTreeMap::new()
        }
    }
}
