use format as tr;

use super::{Engine, Vacum, Invertor, Sinks, PropertiesExt};
    
use std::collections::{BTreeMap, HashMap};
use epoxy::property::*;
use epoxy::{binding, Sink};

pub struct InvertorEngine {
    pub invertor: Invertor,
    pub dvij: Engine,
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

impl InvertorEngine {
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
            
        InvertorEngine {
            invertor: Invertor {
                hz: hz,
            },
            dvij: Engine {
                speed: speed.clone(),
                .. Engine::init_props()
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
    pub fn get_properties(&self) -> HashMap<String, PropertyRead<f32>> {
        let mut res = HashMap::new();
        let engine = &self.dvij.props.properties;
        res.extend(engine.clone().into_iter());
        res.extend(engine.clone().into_iter()
            .map(|(k, v)| (tr!("Engine/{}", k), v)));
        res.insert("davl".into(), self.vacum.davl.clone());
        res.insert("Vacum/davl".into(), self.vacum.davl.clone());
        res
    }
}

impl Sinks<f32> for InvertorEngine {
    fn emit(&self, name: &str, value: f32) -> bool {
        if let Some(name) = name.strip_prefix("Engine/") {
            self.dvij.emit(name, value)
        } else if let Some(name) = name.strip_prefix("Vacum/") {
            self.vacum.emit(name, value)
        } else {
            self.dvij.emit(name, value) ||
            self.vacum.emit(name, value)
        }
    }
}
