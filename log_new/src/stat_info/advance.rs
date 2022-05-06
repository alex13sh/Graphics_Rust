/// Анализ грфиков, и построение таблички с информацией
/// Табличка должна состоять из:
/// 1) Мощность холостого хода (значение до подачи материала) .(оба двигателя)
/// 2) Максимальная мощность, токи, вибрация. (оба двигателя)
/// 3) Время подачи материала
/// 4) Энергия потрачена на всю работу, и на подачу материала.
/// И ещё можно поля с формулы
/// 1) Масса материала - входной параметр.
/// 2) Удельная энергия на грамм и килограмм.

use crate::utils::{
    DateTimeFix as DateTime,
    date_time_now
};

type TimeInterval = f32;

struct StateInfo {
    material_time: TimeInterval,
    max_values: MaxValues,
    energy: Energy,
    material: Option<StateMaterial>,
}

#[derive(Default)]
struct MaxValues {
    power: f32, /// мощность
    amper: f32, /// ток
    vibro: f32, /// вибрация
}

#[derive(Default)]
struct Energy {
    sum_watt: f32,
    time: TimeInterval,
}

struct WattBeforeMaterial {
    watt: f32,
}

struct StateMaterial {
    start_time: DateTime,
    finish_time: DateTime,
    energy: Energy,
    watt_before: WattBeforeMaterial,
}

impl StateMaterial {
    fn start() -> Self {
        let cur_time = date_time_now();
        StateMaterial {
            start_time: cur_time.clone(),
            finish_time: cur_time,

        }
    }

    fn finish(self) {

    }
}
