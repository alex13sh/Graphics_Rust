use std::cell::Cell;
use modbus::{Value, ModbusValues, DeviceResult, DeviceError};
use std::sync::Arc;

struct Algorithm {
    values: ModbusValues,
    step: Cell<u8>,
}

impl Algorithm {
    pub fn new(values: ModbusValues) -> Self {
        Algorithm {
            values: values,
            step: Cell::new(0),
        }
    }

    fn value(&self, name: &str) -> Result<Arc<Value>, DeviceError> {
        let v = self.values.get_value_arc("Клапан выгрузки материала из камеры")
            .ok_or(DeviceError::ValueOut)?
            .value();
        Ok(v)
    }

    pub fn next(&mut self) -> DeviceResult {
//         let stp = self.step.update(|x| x + 1);
        let stp = self.step.get() + 1;
        self.step.set(stp);
        match stp {
        1 => self.stp_1_start_work(),
        2 => self.stp_2(),
        3 => self.stp_3()?,
        4 => self.stp_4_vacum_up(),
        5 => self.stp_5_start_oil(),
        6 => self.stp_6_start_dozator()?,
        7 => self.stp_7(),
        8 => self.stp_8()?,
        9 => self.stp_9_finish(),
        10 => {self.stp_10_start()?; self.step.set(0);},
        _ => {},
        };
        Ok(())
    }
    // 1 Начало работы
    pub fn stp_1_start_work(&self) {

    }
    // 2 Накачка воздуха
    pub fn stp_2(&self) {
        // 8 Компрессор -> 1
    }
    // 3 Установка ШК вакуумной системы в рабочее положение
    pub fn stp_3(&self) -> DeviceResult {
        // 3 Камера помола
            // ШК-03 -> 1 // Клапан помольной камеры
        // 5 Контейнер подачи материала
            // ШК-05 -> 1 // Клапан верхнего контейнера
        // 6 Контейнер приёма материала
            // ШК-01 -> 1 (4.14) // Клапан нижнего контейнера
        self.value("Клапан выгрузки материала из камеры")?
            .set_bit(true);
        // 11 Насос второго уровня
            // ШК-06 -> 1 // Клапан насоса М5
        Ok(())
    }
    // 4 Откачка воздуха из вакуумной системы
    pub fn stp_4_vacum_up(&self) {
        // 10 Насос первого уровня -> 1
        // 11 Насос второго уровня -> 1
    }
    // 5 Запуск маслостанции и основных двигателей
    pub fn stp_5_start_oil(&self) {
        // 1 Нижний привод -> 1
        // 2 Верхний привод -> 1
        // 5 Контейнер подачи материала
            // ШК-05 -> 0
        // 7 Маслостанция -> 1
    }
    // 6 Запуск дозатора, подача материала для измельчения
    pub fn stp_6_start_dozator(&self) -> DeviceResult {
        // 4 Дозатор  -> 1
        self.value("Двигатель подачи материала в камеру/Частота высокочастотного ШИМ")?
            .set_value(1); // <<----
        // 5 Контейнер подачи материала
            // ШК-02 -> 1
        self.value("Клапан насоса М6 вакуум")?
            .set_bit(true);
        Ok(())
    }
    // 7 Измельчение материала
    pub fn stp_7(&self) {
        // 3 Камера помола -> 1
        // 6 Контейнер приёма материала -> 1
    }
    // 8 Предварительное завершение работы мельницы
    pub fn stp_8(&self) -> DeviceResult {
        // 1 Нижний привод -> 0
        // 2 Верхний привод -> 0
        // 3 Камера помола -> 0
        // 4 Дозатор  -> 0
        // 5 Контейнер подачи материала -> 0
        // 6 Контейнер приёма материала
            // ШК-01 -> 0
        self.value("Клапан выгрузки материала из камеры")?
            .set_bit(false);
        // 7 Маслостанция -> 0
        Ok(())
    }
    // 9 завершение работы мельницы
    pub fn stp_9_finish(&self) {
        // 3 Камера помола
            // ШК-04 -> 1
        // 10 Насос первого уровня -> 0
        // 11 Насос второго уровня -> 0
    }
    // 10 Стартовое положение
    pub fn stp_10_start(&self) -> DeviceResult {
        // 3 Камера помола
            // ШК-03 -> 0
            // ШК-04 -> 0
        // 5 Контейнер подачи материала -> 1
            // ШК-02 -> 0
        self.value("Клапан насоса М6 вакуум")?
            .set_bit(false);
        // 8 Компрессор -> 0
        // 11 Насос второго уровня
            // ШК-06 -> 0
        Ok(())
    }
}
