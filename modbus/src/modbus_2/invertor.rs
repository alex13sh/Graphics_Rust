#![allow(dead_code)]

use super::init::{DeviceType, InvertorFunc};
use super::{Device, DeviceError, ModbusValues};
use super::Value;

use std::sync::Arc;

pub struct Invertor {
    device: Arc<Device>, // make mut
    // device_analog_output: Arc<Device>, // Owen Analog
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DvijDirect {
    FWD,
    REV,
}

impl Invertor {
    pub fn new(device: Device) -> Self {
        Invertor {
            device: Arc::new(device)
        }
    }
    
    pub fn start(&self) ->  Result<(), DeviceError> {
        let vm = self.device.values_map();
//         let _v_start_hz: Arc<Value> = vm.get("Стартовая частота").unwrap().clone();
//         let _v_max_hz = vm.get("Максимальная выходная частота").unwrap().clone();
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
    pub fn set_direct(&self, direct: DvijDirect) ->  Result<(), DeviceError> {
        dbg!(direct);
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
    
    pub fn configure(&self) {
        let vm = self.device.values_map();
        let mut values = Vec::new();
        values.append(&mut Self::configure_source(&vm));
    }
    fn configure_ip_address(vm: &ModbusValues) -> Vec<Arc<Value>> {           
        let values = vec![
            vm.set_value("IP адрес 1 комм. платы", 192_u32),
            vm.set_value("IP адрес 2 комм. платы", 168_u32),
            vm.set_value("IP адрес 3 комм. платы", 1_u32),
            vm.set_value("IP адрес 4 комм. платы", 5_u32),
        ];
        values
    }
    fn configure_source(vm: &ModbusValues) -> Vec<Arc<Value>> { 
        vec![
            vm.set_value("Источник задания частоты", 0_u32),
            vm.set_value("Источник команд управления", 0_u32),
            vm.set_value("Источник задания частоты (HAND)", 0_u32),
        ]
    }
    fn configure_base(vm: &ModbusValues) -> Vec<Arc<Value>> { 
        vec![
            vm.set_value("Режим управления", 0_u32), // 0.10
            vm.set_value("Метод управления скоростью", 0_u32), // 0.11
            vm.set_value("Режим работы привода", 0_u32), // 0.16
            vm.set_value("Несущая частота ШИМ", 4_u32), // 0.17
            vm.set_value("Управление направлением вращения двигателя", 0_u32), // 0.23
            vm.set_value("Сбособ остановки", 1_u32), // 0.22
            
            vm.set_value("Максимальная выходная частота", 400_00_u32), // 1.0
            vm.set_value("Номинальная частота двигателя", 50_00_u32), // 1.1
            vm.set_value("Номинальное напряжение двигателя", 380_0_u32), // 1.2
            vm.set_value("Стартовая частота", 0_30_u32), // 1.9
            
            vm.set_value("Верхнее ограничение выходной частота", 400_00_u32), // 1.10
            vm.set_value("Нижнее ограничение выходной частота", 0_00_u32), // 1.11
            
            vm.set_value("Выбор режима разгона/замедления", 0_u32), // 1.44
        ]
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
