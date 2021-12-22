use super::init;

#[derive(Debug, Clone, Default, Hash, Eq, PartialEq)]
pub struct ValueID {
    pub device_id: u16,
    pub device_name: String,
    pub sensor_name: String,
    pub value_name: String,
}

impl From<init::ValueID> for ValueID {
    fn from(v: init::ValueID) -> Self {
        ValueID {
            device_id: v.device_id.unwrap_or(0),
            device_name: v.device_name.unwrap_or("".into()),
            sensor_name: v.sensor_name.unwrap_or("".into()),
            value_name: v.value_name.unwrap_or("value".into()),
        }
    }
}

impl std::fmt::Display for ValueID {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}) {}/{}/{}", self.device_id, self.device_name, self.sensor_name, self.value_name)
    }
}

impl ValueID {
    pub fn sensor_value_name(&self) -> String {
        format!("{}/{}", self.sensor_name, self.value_name)
    }
}

impl PartialEq<init::ValueID> for ValueID {
    fn eq(&self, other: &init::ValueID) -> bool {
        other.device_id.map_or(true, |id| self.device_id == id) && 
        other.device_name.as_ref().map_or(true, |name| &self.device_name == name) && 
        other.sensor_name.as_ref().map_or(true, |name| &self.sensor_name == name) && 
        other.value_name.as_ref().map_or(true, |name| &self.value_name == name)
    }
}

#[derive(Debug)]
pub enum ValueFloatError {
//     None, // Измерение успешно
    ValueFalse, // 0xF0 -- Значение заведомо неверно
    SensorDisconnect, // 0xF7 -- Датчик отключен
    TemperHight, // 0XF8 -- Велика температура свободных концов ТП
    TemperLow, // 0XF9 -- Мала температура свободных концов ТП
    ValueHigth, // 0xFA -- Измеренное значение слишком велико
    ValueLow, // 0xFB -- Измеренное значение слишком мало
    ShortCircuit, // 0xFC -- Короткое замыкание датчика
//     SensorDisconnect, // 0xFD -- Обрыв датчика
    AdcError, // 0xFE -- Отсутствие связи с АЦП
    RatioError, // 0xFF -- Некорректный калибровочный коэффициент

//     CriticalValue(super::ValueError),
}

impl ValueFloatError {
    // pub(crate) -- это на всякий случай, можно и просто pub
    pub(crate) fn new(value: u32) -> Option<Self> {
        let lb = value >> 24;
        let res = match lb {
        0xF0 => Some(Self::ValueFalse),
        0xF7 | 0xFD => Some(Self::SensorDisconnect),
        0xF8 => Some(Self::TemperHight),
        0xF9 => Some(Self::TemperLow),
        0xFA => Some(Self::ValueHigth),
        0xFB => Some(Self::ValueLow),
        0xFC => Some(Self::ShortCircuit),
        0xFE => Some(Self::AdcError),
        0xFF => Some(Self::RatioError),
        _ => None,
        };
//         if let Some(ref err) = res {
//             println!("ValueFloatError: {:#X} - {:#X} -- {:?}", value, lb, res);
//         }
        res
    }
    pub(crate) fn new_u16(value: u32) -> Option<Self> {
        if value == 32768 {
            Some(Self::SensorDisconnect)
        } else {
            None
        }
    }
}

