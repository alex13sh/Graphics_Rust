#![allow(dead_code)]

use modbus::{ValueArc, ModbusValues};
// use tokio::sync::Mutex;
use std::sync::Mutex;

pub struct Dozator {
    speed: ValueArc, // скоость ШИМа
    direct: ValueArc, 
    
    target_speed: Mutex<Option<TargetSpeedState>>,
}

#[derive(Debug)]
struct TargetSpeedState {
    target_speed: i32,
    current_speed: i32,
    delta: i32,
    step: u32,
}

impl Dozator {
    const STEPS: u32 = 20;
    pub fn set_speed(&self, speed: i32) {
        log::trace!("set_speed: {}", speed);
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
    
    pub fn set_target_speed(&self, target_speed: i32) {
        let mut target_speed_lock = self.target_speed.lock().unwrap();
        let current_speed = self.speed();
        let delta: i32 = (target_speed - current_speed) / Self::STEPS as i32;
        let tg = TargetSpeedState {
            target_speed, current_speed, delta,
            step: Self::STEPS
        };
        log::trace!("set_target_speed: tg: {:?}", &tg);
        *target_speed_lock = Some(tg);
    }
    fn get_next_step(&self) -> Option<i32> {
        let mut target = self.target_speed.lock().unwrap();
        if let Some(ref mut tg) = target.as_mut() {
            if tg.step>0 {
                tg.step -= 1;
                tg.current_speed += tg.delta;
                log::trace!("get_next_step: tg: {:?}", tg);
                return Some(tg.current_speed);
            }
        }
        *target = None;
        None
    }
    fn next_step(&self) {
        if let Some(current_speed) = self.get_next_step() {
            self.set_speed(current_speed);
        }
    }
    pub async fn animation(&self) {
        use tokio::time::{sleep, Duration};
        loop {
            self.next_step();
            sleep(Duration::from_millis(5_000 / super::Dozator::STEPS as u64)).await;
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
