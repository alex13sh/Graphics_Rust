#![allow(dead_code)]

use super::init::{DeviceType, InvertorFunc};
use super::{Device, DeviceError};
use super::Value;

use std::sync::Arc;

pub struct Invertor {
    device: Arc<Device>, // make mut
    // device_analog_output: Arc<Device>, // Owen Analog
}

impl Invertor {
    pub fn new(device: Device) -> Self {
        Invertor {
            device: Arc::new(device)
        }
    }
    
    pub fn start(&self) ->  Result<(), DeviceError> {
        let vm = self.device.values_map();
        let _v_start_hz: Arc<Value> = vm.get("Стартовая частота").unwrap().clone();
        let _v_max_hz = vm.get("Максимальная выходная частота").unwrap().clone();
        let v_bitmap_run = vm.get("2000H").unwrap().clone();
        
        v_bitmap_run.set_bit(1, false); // Stop
        v_bitmap_run.set_bit(2, true); // Run
        self.device.context()?.borrow_mut().set_value(&v_bitmap_run)?;
        Ok(())
    }
    pub fn stop(&self) ->  Result<(), DeviceError> {
        let vm = self.device.values_map();
        let v_bitmap_run = vm.get("2000H").unwrap().clone();
        
        v_bitmap_run.set_bit(1, true); // Stop
        v_bitmap_run.set_bit(2, false); // Run
        self.device.context()?.borrow_mut().set_value(&v_bitmap_run)?;
        Ok(())
    }
    pub fn set_hz(&mut self, hz: u16) ->  Result<(), DeviceError> {
        let vm = self.device.values_map();
        let v_set_hz = vm.get("Заданная частота по коммуникационному интерфейсу").unwrap().clone();
        v_set_hz.update_value(hz as u32);
        self.device.context()?.borrow_mut().set_value(&v_set_hz)?;
        Ok(())
    }
    pub fn get_amper_out_value(&self) -> Arc<Value> {
        let vm = self.device.values_map();
        vm.get("Выходной ток (A)").unwrap().clone()
    } 
    pub fn get_hz_out_value(&self) -> Arc<Value> {
        let vm = self.device.values_map();
        vm.get("Выходная частота (H)").unwrap().clone()
    }
    
    fn get_address_function(&self, num_func: u8) -> Option<u16> {
        match &self.device.device_type {
        DeviceType::Invertor {functions} => {
            for f in functions.iter() {
                match f {
                InvertorFunc::DigitalInput(num_input, num_func_input) => {
                    if *num_func_input == num_func {
                        if *num_input >= 2 {
                            return Some(*num_input as u16 -2 + 2*256+1);
                        } else {return None;}
                    }
                },
                InvertorFunc::DigitalOutput(num_output, num_func_output) => {
                    if *num_func_output == num_func {
                        if [0, 1, 3, 4].contains(num_output) {
                            return Some(*num_output as u16 + 2*256+13);
                        } else if *num_output >= 5 {
                            return Some(*num_output as u16-5 + 2*256+36);
                        }
                    }
                },
                _ => {return None;}
                }
            }
        },
        _ => {}
        };
        None
    }
    
    pub fn device(&self) -> Arc<Device> {
        self.device.clone()
    }
}

impl From<Device> for Invertor {
    fn from(d: Device) -> Self {
        Invertor {
            device: Arc::new(d)
        }
    }
}

// Funcs
impl Invertor {
    fn read_digit_func(_func: &InvertorFunc) -> bool {
        false
    }
    fn read_analog_func(_func: &InvertorFunc) -> f32 {
        0_f32
    }
}

#[test]
fn test_array_contsins() {
    assert_eq!([0,1, 3,4].contains(&1), true);
    assert_eq!([0,1, 3,4].contains(&2), false);
}
