use super::{InvertorEngine, Properties, ValueSink};

use modbus::{Value, ModbusValues, ValueError};
use modbus::init;
use modbus::invertor::{Invertor, DvijDirect}; // Device
use modbus::{Device, DigitIO};

use std::collections::BTreeMap;
use std::sync::Arc;

macro_rules! map(
  { $T:ident, $($key:expr => $value:expr),+ } => {
    {
      let mut m = $T::new();
      $(
        m.insert($key, $value);
      )+
      m
    }
 };
);

pub struct Complect {
    pub invertor_engine: InvertorEngine,
    
    pub invertor: Invertor,
    pub digit_io: DigitIO,
    pub owen_analog_1: Arc<Device>,
    pub owen_analog_2: Arc<Device>,
    
    values_sink: Vec<(Arc<Value>, ValueSink)>,
}

impl Complect {
    pub fn new() -> Self {
        let invertor = init::make_invertor("192.168.1.5".into());
        let invertor = Invertor::new(invertor.into());
        let digit_io = DigitIO::new(init::make_io_digit("192.168.1.10".into()).into());
        
        Complect {
            invertor_engine: InvertorEngine::new(),
            
            invertor: invertor,
            digit_io: digit_io,
            owen_analog_1: Arc::new(Device::from(init::make_owen_analog_1("192.168.1.11"))),
            owen_analog_2: Arc::new(Device::from(init::make_owen_analog_2("192.168.1.13"))),
            
            values_sink: Vec::new(),
        }
    }
    pub fn make_values(&self) -> BTreeMap<String, Arc<Value>> {
        let devices = self.get_arr_device();//.map(|&d| d.clone());
        
        let mut values = BTreeMap::new();
        for (dev, (k,v)) in devices.iter()
            .flat_map(|d| {
                let dname = d.name().clone();
                d.values_map().iter()
                .map(move |(k,v)| (dname.clone(), (k,v)))
            }).filter(|(_d, (_k,v))| v.is_read_only()) {
        
            values.insert(format!("{}/{}", dev, k.clone()), v.clone());
        }
        values
    }
    
    pub fn update(&self) {
        use std::convert::TryFrom;
        let devices = self.get_arr_device();
            
        for d in &devices {
            d.update();
        }
    }
    
    fn get_arr_device(&self) -> Vec<Arc<Device>> {
        [&self.owen_analog_1, &self.owen_analog_2,
        &self.digit_io.device(), &self.invertor.device()]
        .iter().map(|&d| d.clone()).collect()
    }
    
    pub fn init_values(&mut self, values: &BTreeMap<String, Arc<Value>>) {
        println!("Values: {:?}", values.keys());

        // "Analog/Температура Пер.Под./value_float"
        // "Analog/Температура Ротора/value_float"      -- Engine.temp_rotor
        // "Analog/Температура Статора/value_float"     -- Engine.temp_stator
        // "DigitIO/Битовая маска состояния выходов"
        // "DigitIO/Скоростной счётчик импульсов/value" -- Engine.speed
        
        // "Invertor/Выходная частота (H)"              -- Engine.speed
        // "Invertor/Выходное напряжение (E)"
        // "Invertor/Выходной ток (A)"
        // "Invertor/Заданная частота (F)"              -- Invertor.hz
        
        // "Invertor/Код ошибки"
        // "Invertor/Температура радиатора"
        // "Invertor/Наработка двигателя (дни)"
        // "Invertor/Наработка двигателя (мин)"
        
        let engine = &self.invertor_engine.dvij;
        let map_name_sink = map! {BTreeMap,
            "Analog/Давление -1_1 V/value_float" => &self.invertor_engine.vacum.davl,
            "Analog/Вибрация 4_20 A/value_float" => &engine.vibra,
            "Analog/Температура Зад.Под./value_float" => &engine.temp_podshipnik,
            "Analog/Температура Ротора/value_float" => &engine.temp_rotor,
            "Analog/Температура Статора/value_float" => &engine.temp_stator
        };
//         dbg!(map_name_sink);
//         self.values_sink = map_name_sink.into_iter()
//             .filter_map(|(k,v)| Some((values.get(k)?.clone(), v.clone())))
//             .collect();
    }
}


#[test]
fn test_values() {
    let logic = Complect::new();
    let values = logic.make_values();
    logic.init_values(&values);
    assert!(false);
}
