pub mod simple;
pub mod advance;

use crate::value::{
    self,
    simple::Value, SimpleValuesLine,
    ElkValuesLine,
};

use futures::{Stream, StreamExt};

pub fn filter_half_top(vin: impl Stream<Item=ElkValuesLine>) -> impl Stream<Item=SimpleValuesLine> {
    vin.map(filter_map_half_top_fn)
}

pub fn filter_half_low(vin: impl Stream<Item=ElkValuesLine>) -> impl Stream<Item=SimpleValuesLine> {
    vin.map(filter_map_half_low_fn)
}

pub fn filter_map_half_top_fn(line: ElkValuesLine) -> SimpleValuesLine {
    use value::simple::Value;
    let dt = line.date_time;
    let values = line.values.into_vec();
    let values: Vec<_> = values.into_iter().filter_map(|v| {
        if v.device_name == "Invertor" && v.device_id == 6 {
            Some(Value {
                sensor_name: v.sensor_name,
                value: v.value,
            })
        } else if let Some(sensor) = v.sensor_name.strip_suffix(" дв. М2") {
            Some(Value {
                sensor_name: sensor.into(),
                value: v.value,
            })
        } else if v.sensor_name == "Клапан ШК2 открыт" {
            Some(Value {
                sensor_name: "Клапан подачи материала открыт".into(),
                value: v.value,
            })
        } else if v.sensor_name == "Двигатель подачи материала в камеру"
            // && v.value_name == "Частота высокочастотного ШИМ"
            {
            Some(Value {
                sensor_name: "Скорость подачи материала".into(),
                value: v.value,
            })
        } else {
            None
        }
    }).collect();
    SimpleValuesLine {
        date_time: dt,
        values: values.into_boxed_slice(),
    }
}

pub fn filter_map_half_low_fn(line: ElkValuesLine) -> SimpleValuesLine {
    let dt = line.date_time;
    let values = line.values.into_vec();
    let values: Vec<_> = values.into_iter().filter_map(|v| {
        if v.device_name == "Invertor" && v.device_id == 5 {
            Some(Value {
                sensor_name: v.sensor_name,
                value: v.value,
            })
        } else if let Some(sensor) = v.sensor_name.strip_suffix(" дв. М1") {
            Some(Value {
                sensor_name: sensor.into(),
                value: v.value,
            })
        } else if let Some(sensor) = v.sensor_name.strip_suffix(" двигатель М1") {
            Some(Value {
                sensor_name: sensor.into(),
                value: v.value,
            })
        } else if v.sensor_name == "Разрежение воздуха в системе" {
            Some(Value {
                sensor_name: "Разрежение воздуха в системе".into(),
                value: v.value,
            })
        } else if v.sensor_name == "Клапан ШК2 открыт" {
            Some(Value {
                sensor_name: "Клапан подачи материала открыт".into(),
                value: v.value,
            })
        } else if v.sensor_name == "Двигатель подачи материала в камеру"
            // && v.value_name == "Частота высокочастотного ШИМ"
            {
            Some(Value {
                sensor_name: "Скорость подачи материала".into(),
                value: v.value,
            })
        }
        else {
            None
        }
    }).collect();
    SimpleValuesLine {
        date_time: dt,
        values: values.into_boxed_slice(),
    }
}
