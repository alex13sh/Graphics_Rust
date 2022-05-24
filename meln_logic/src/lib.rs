/// Логика работы мельниы
/// Алгоритм запуска:
/// - Откачка воздуха из всех контейнеров
/// - Закрыть клапан подачи материала, перед запуском маслостанции
/// - Включить маслостанцию перед запуском моторов.
/// - Перед подачей материала открывается (контейнер ?)
/// - Подача материала происходит после нажатии кнопки.
/// - Отслеживается энергетика и завершения материала.

pub mod meln;
pub use meln::{Meln as MelnValues};
pub mod watcher;
use watcher::Meln as MelnWatch;

use std::sync::Arc;

#[derive(Clone)]
pub struct Meln {
    pub values: Arc<MelnValues>,
    pub properties: Arc<MelnWatch>,
}

impl Meln {
    pub fn new(values: &modbus::ModbusValues) -> Self {
        Meln {
            values: Arc::new(values.into()),
            properties: Default::default(),
        }
    }
    pub async fn automation(&self) {
        self.properties.automation().await
    }
    pub async fn automation_mut(&self) {
        meln::watcher::automation_mut(&self.values, &self.properties).await
    }
}

pub mod values {
    pub use super::meln::{
        Meln,
        half_meln::{HalfMeln, HalfPart, Invertor},
        dozator::Dozator,
        oil_station::OilStation,
        vacuum_station::VacuumStation,
        material::Material,
        klapans::Klapans,
    };
}
