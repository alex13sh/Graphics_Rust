#![allow(dead_code)]

use modbus::{ValueArc, ModbusValues};
// use tokio::sync::Mutex;
use std::sync::Mutex;

pub struct Dozator {
    speed: ValueArc, // скоость ШИМа
    direct: ValueArc, 
    
    target_speed: Mutex<Option<(/*speed:*/ i32, /*delta:*/ i32)>>,
}

impl Dozator {
    const STEPS: u32 = 20;
    pub fn set_speed(&self, speed: i32) {
        self.direct.set_bit(speed >= 0);
        self.speed.set_value(speed.abs() as u32);
    }
    fn speed(&self) -> i32 {
        let speed: i32 = self.speed.value() as i32;
        if self.direct.get_bit() == false {
            -speed
        } else {
            speed
        }
    }
    
    pub fn set_target_speed(&self, speed: i32) {
        let mut target_speed = self.target_speed.lock().unwrap();
        let current_speed = self.speed();
        let dlt: i32 = (speed - current_speed) / Self::STEPS as i32;
        
        *target_speed = Some((speed, dlt));
    }
    fn get_next_step(&self) -> Option<i32> {
        let mut target = self.target_speed.lock().unwrap();
        if let Some((target_speed, delta)) = *target {
            let current_speed = self.speed();
            if current_speed != target_speed {
                return Some(delta);
            }
        }
        *target = None;
        None
    }
    fn next_step(&self) {
        if let Some(step) = self.get_next_step() {
            self.set_speed(self.speed() + step);
        }
    }
    pub async fn animation(&self) {
        use tokio::time::{sleep, Duration};
        loop {
            self.next_step();
            sleep(Duration::from_millis(1000 / super::Dozator::STEPS as u64)).await;
        }
    }
}

impl From<&ModbusValues> for Dozator {
    fn from(values: &ModbusValues) -> Self {
        Dozator {
            speed: values.get_value_arc("Двигатель подачи материала в камеру/Частота высокочастотного ШИМ").unwrap(),
            direct: values.get_value_arc("Направление вращения двигателя ШД").unwrap(),
            
            target_speed: Mutex::new(None),
        }
    }
}

pub mod watcher {
    use crate::structs::Property;
    
    #[derive(Default)]
    pub struct Dozator {
        pub speed: Property<i32>,
        pub motor: Property<bool>,
    }
    impl Dozator {
        pub(crate) fn update_property(&self, values: &super::Dozator) {
            let speed: i32 = values.speed();
            self.speed.set(speed);
            self.motor.set(speed != 0);
        }
        
        pub(crate) async fn automation_mut(values: &super::Dozator, properties: &Dozator) {
            values.animation().await;
        }
    }
}
