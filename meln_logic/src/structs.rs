use epoxy::property::*;
use epoxy::{binding, Sink};

pub struct Dvij {
    pub speed: PropertyRead<u32>,
}

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

pub struct InvertorDvij {
    pub invertor: Invertor,
    pub dvij: Dvij,
    pub speed: PropertyRead<u32>,
    pub vacum: Vacum,
    
}

impl InvertorDvij {
    pub fn new() -> Self {
        let hz = Property::new(0);
//         let speed = Property::new(0).as_readonly();
        let speed = binding!{(*hz as u32) * 60};
        
        InvertorDvij {
            invertor: Invertor {
                hz: hz,
            },
            dvij: Dvij {
                speed: speed.clone(),
            },
            speed: speed.clone(),
            vacum: Vacum::new(),
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
}
