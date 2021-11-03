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
        log::trace!("Meln Init");
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
//             log::trace!("Meln update_property");

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
                    log::trace!("Meln is started: {:?}", start_top || start_bottom);
                    self.is_started.set(start_top || start_bottom);
                }
            };

            let f_step = async {
                self.step.send(self.step.get());
                loop {
                    let next_step = self.step.get()
                        .check_next_step(self).await;
                    log::trace!("Next Step: {:?}", &next_step);
                    self.step.set(next_step);
                }
            };
            tokio::join!(
                f_is_started,
                f_step,
                self.half_top.automation(),
                self.half_bottom.automation(),
                self.klapans.automation(),
                self.vacuum.automation(),
            );
        }
    }
    pub async fn automation_mut(values: &super::Meln, props: &Meln) {
        use tokio::time::{sleep, Duration};
        let mut sub_is_started = props.is_started.subscribe();
        let f_stop = async move {
            loop {
                let _ = sub_is_started.changed().await;
                let is_started = *sub_is_started.borrow();
                match is_started {
                false => {
                    values.vacuum.davl_dis();
                    sleep(Duration::from_millis(10_000)).await;
                    values.oil.stop();
                }
                true => {
                    values.oil.start();
                }
                }
            }
        };
    
        tokio::join!(
            Dozator::automation_mut(
                &values.material.dozator, 
                &props.material.dozator
            ),
            f_stop,
        );
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
        
//         Тестовый_запуск_на_воздухе,
//         Тестовый_запуск_в_вакууме,
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
//                 let mut клапан_верхнего_контейнера = meln.material.клапан_верхнего_контейнера.subscribe();

                if meln.material.клапан_помольной_камеры.get() == false {
                   let _ = meln.material.клапан_помольной_камеры.subscribe().changed().await;
                }
                if meln.material.клапан_нижнего_контейнера.get() == false {
                   let _ = meln.material.клапан_нижнего_контейнера.subscribe().changed().await;
                }
                if meln.material.клапан_помольной_камеры.get()
                        && meln.material.клапан_нижнего_контейнера.get() {
                    Step_3
                } else {self}
            }
            Step_3 => {
                if meln.vacuum.motor.get() == false {
                   let _ = meln.vacuum.motor.subscribe().changed().await;
                   Откачка_воздуха_из_вакуумной_системы
                } else {self}
            }
            Откачка_воздуха_из_вакуумной_системы => {
//                 let _klapan = meln.material.клапан_подачи_материала.get();
                // klapan == false; // Проверить закрыт ли клапан, если нет, то ошибка!
                if meln.is_started.get()
                        && meln.oil.motor.get() {
                    Запуск_маслостанции_и_основных_двигателей
                } else {
                    let mut meln_motor = meln.is_started.subscribe();
                    let mut oil_motor = meln.oil.motor.subscribe();
                    changed_any!(meln_motor, oil_motor);
                    self
                }
            }
            Запуск_маслостанции_и_основных_двигателей => {
                if meln.material.dozator.motor.get() {
                    Подача_материала
                } else {
                    let mut motor = meln.material.dozator.motor.subscribe();
                    let _ = motor.changed().await;
                    Подача_материала
                }
            }
            Подача_материала => {
                let mat = &meln.material;
                if mat.клапан_помольной_камеры.get()
                        && mat.клапан_помольной_камеры.get() {
                    Измельчение_материала
                } else {
                    let mut клапан_помольной_камеры = meln.material.клапан_помольной_камеры.subscribe();
                    let mut клапан_нижнего_контейнера = meln.material.клапан_помольной_камеры.subscribe();
                    changed_any!(клапан_помольной_камеры, клапан_нижнего_контейнера);
                    self
                }
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
