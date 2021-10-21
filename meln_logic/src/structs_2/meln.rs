#![allow(dead_code)]

use modbus::ModbusValues;
use super::HalfMeln;
use super::Dozator;
use super::OilStation;
use super::VacuumStation;
use super::Material;
use super::Klapans;

pub struct Meln {
    
    pub material: Material,
    
    pub half_top: HalfMeln,
    pub half_bottom: HalfMeln,
    
    pub oil: OilStation,
    pub vacuum: VacuumStation,
    
    pub klapans: Klapans,
}

impl From<&ModbusValues> for Meln {
    fn from(values: &ModbusValues) -> Self {
        Meln {
            material: values.into(),
            
            half_top: HalfMeln::top(values),
            half_bottom: HalfMeln::low(values),
            
            oil: values.into(),
            vacuum: values.into(),
            
            klapans: values.into(),
        }
    }
}

impl Meln {
    // Экстренное торможение ?
    pub fn stop(&self) {
        self.half_top.stop();
        self.half_bottom.stop();
    }
}

pub mod watcher {
    use crate::structs::{
        Property, changed_all, changed_any,
        watcher::{
            HalfMeln, OilStation, VacuumStation,
            Material, Dozator, Klapans,
        }
    };
    
    #[derive(Default)]
    pub struct Meln {
        pub material: Material,
        
        pub half_top: HalfMeln,
        pub half_bottom: HalfMeln,
        pub is_started: Property<bool>,
        pub is_worked: Property<bool>, // is started | oil.motor
        
        pub oil: OilStation,
        pub vacuum: VacuumStation,
        pub klapans: Klapans,
        
        pub step: Property<MelnStep>,
    }
    
    impl Meln {
        pub fn update_property(&self, values: &super::Meln) {
            self.material.update_property(&values.material);
            
            self.half_top.update_property(&values.half_top);
            self.half_bottom.update_property(&values.half_bottom);
            
            self.oil.update_property(&values.oil);
            self.vacuum.update_property(&values.vacuum);
            self.klapans.update_property(&values.klapans);
            
            self.is_worked.set(self.oil.motor.get());
        }
        
        pub async fn automation(&self) {
            let f_is_started = async {
                let mut start_top = self.half_top.is_started.subscribe();
                let mut start_bottom = self.half_bottom.is_started.subscribe();
                
                loop {
                    changed_any!(start_top, start_bottom);
                    let start_top = *start_top.borrow();
                    let start_bottom = *start_bottom.borrow();
                    
                    self.is_started.set(start_top || start_bottom);
                }
            };
            let f_step = async {
                loop {
                    let next_step = self.step.get()
                        .check_next_step(self).await;
                    self.step.set(next_step);
                }
            };
            tokio::join!(
                f_is_started,
                f_step,
                self.half_top.automation(),
                self.half_bottom.automation(),
                self.klapans.automation(),
            );
        }
    }
    pub async fn automation_mut(values: &super::Meln, properties: &Meln) {
        Dozator::automation_mut(
            &values.material.dozator, 
            &properties.material.dozator
        ).await;
    }
    
    // Шаги алгоритма работы мельницы
    #[derive(Debug, PartialEq, Clone)]
    #[allow(non_camel_case_types)]
    pub enum MelnStep {
        Начало_работы,
//         Накачка_воздуха, // Этот этап вроде уже не нужен
        Step_3, // Установка ШК вакуумной системы в рабочее положение
        Откачка_воздуха_из_вакуумной_системы,
        Запуск_маслостанции_и_основных_двигателей,
        Подача_материала, // 6 Запуск дозатора, подача материала для измельчения
        Измельчение_материала,
        Предварительное_завершение_работы_мельницы,
        Завершение_работы_мельницы,
        Стартовое_положение,
        
        ErrorStep,
    }
    
    impl Default for MelnStep {
        fn default() -> Self {
            MelnStep::Начало_работы
        }
    }
    
    impl MelnStep {
        async fn check_next_step(self, meln: &Meln) -> Self {
            use MelnStep::*;
            #[allow(unused_must_use)]
            match self {
            Начало_работы => {
                let mut клапан_помольной_камеры = meln.material.клапан_помольной_камеры.subscribe();
                let mut клапан_верхнего_контейнера = meln.material.клапан_верхнего_контейнера.subscribe();
                let mut клапан_нижнего_контейнера = meln.material.клапан_нижнего_контейнера.subscribe();
                
                changed_all!(
                    клапан_помольной_камеры,
                    клапан_верхнего_контейнера,
                    клапан_нижнего_контейнера
                );
                Step_3
            }
            Step_3 => {
                let mut motor = meln.vacuum.motor.subscribe();
                changed_all!(motor);
                Откачка_воздуха_из_вакуумной_системы
            }
            Откачка_воздуха_из_вакуумной_системы => {
                let mut meln_motor = meln.is_started.subscribe();
                let mut oil_motor = meln.oil.motor.subscribe();
                let _klapan = meln.material.клапан_подачи_материала.get();
                // klapan == false; // Проверить закрыт ли клапан, если нет, то ошибка!
                changed_all!(meln_motor, oil_motor);
                Запуск_маслостанции_и_основных_двигателей
            }
            Запуск_маслостанции_и_основных_двигателей => {
                let mut motor = meln.material.dozator.motor.subscribe();
                changed_all!(motor);
                // *motor.borrow() == true
                Подача_материала
            }
            Подача_материала => {
                let mut клапан_помольной_камеры = meln.material.клапан_помольной_камеры.subscribe();
                let mut клапан_нижнего_контейнера = meln.material.клапан_помольной_камеры.subscribe();
                changed_all!(клапан_помольной_камеры, клапан_нижнего_контейнера);
                
                Измельчение_материала
            }
            Измельчение_материала => {
                // ...
                Предварительное_завершение_работы_мельницы
            }
            _ => self,
            }
        }
    }
}
