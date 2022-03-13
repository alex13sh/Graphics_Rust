use modbus::{ValueArc, ModbusValues};

pub struct Klapans {
    pub давление_воздуха: ValueArc,
    двигатель_компрессора_воздуха: ValueArc,
    
//     // Материал
//     клапан_подачи_материала: ValueArc,      // ШК2
//     клапан_верхнего_контейнера: ValueArc,   // ШК5
//     клапан_помольной_камеры: ValueArc,      // ШК3
//     клапан_нижнего_контейнера: ValueArc,    // ШК1
//     
//     // Вакуум
//     клапан_напуска: ValueArc,
//     клапан_насоса: ValueArc,
    
    klapans: ModbusValues,
    klapans_шк: ModbusValues,
}

impl From<&ModbusValues> for Klapans {
    fn from(values: &ModbusValues) -> Self {
        let klapans_шк = (0..12).map(|i| 
                format!("Клапан ШК{} {}", i/2+1,
                if i%2==0 {"открыт"} else {"закрыт"})
            ).collect::<Vec<_>>();
        Klapans {
            давление_воздуха: values.get_value_arc("Давление воздуха компрессора").unwrap(),
            двигатель_компрессора_воздуха: values.get_value_arc("Двигатель компрессора воздуха").unwrap(),
            klapans: values.get_values_by_id(|id|
                ["Клапан нижнего контейнера", "Клапан верхнего контейнера",
                "Клапан подачи материала", "Клапан помольной камеры",
                "Клапан напуска", "Клапан насоса М5"]
                .iter().any(|k| k==&id.sensor_name)
            ),
            klapans_шк: values.get_values_by_id(|id|
                klapans_шк.iter().any(|k| k==&id.sensor_name)
            ),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum KlapansError {
    НедостаточноДавленияВоздуха,
    НетОбратнойСвязи,
}

impl Klapans {

    pub fn klapan_turn(&self, name: &str, enb: bool) -> Result<(), KlapansError> {
        if self.давление_воздуха.is_error() {
            return Err(KlapansError::НедостаточноДавленияВоздуха);
        }
        
        self.klapans.get_value_arc(name).unwrap().set_bit(enb);
        Ok(())
    }
    fn двигатель_компрессора_воздуха_turn(&self, enb: bool) {
        self.двигатель_компрессора_воздуха.set_bit(enb);
    }

    pub fn ШК_в_рабочее_положение(&self, enb: bool) {

    }
}

pub mod watcher {
    use crate::watcher::Property;
    use std::collections::HashMap;
    use tokio::sync::broadcast;
    
    use bitflags::bitflags;

    bitflags! {
        #[derive(Default)]
        pub struct KlapansError: u8 {
            const НедостаточноДавленияВоздуха = 1<<0;
            const НетОбратнойСвязи = 1<<1;
        }
    }

    pub struct Klapans {
        pub давление_воздуха: Property<f32>,
        
        pub klapans: HashMap<String, Property<bool>>,
        pub klapans_send: broadcast::Sender<(String, bool)>,
        
        pub klapans_шк: HashMap<(String, String), Property<bool>>,
        pub klapans_шк_send: broadcast::Sender<(String, bool)>,

        pub klapans_error: Property<KlapansError>,
    }
    impl Klapans {
        pub(crate) fn update_property(&self, values: &super::Klapans) {
            for (n, p) in &self.klapans {
                p.set(values.klapans.get_value_arc(n).unwrap().get_bit());
            }
            for ((шк, _n), p) in &self.klapans_шк {
                p.set(values.klapans_шк.get_value_arc(&format!("Клапан {} открыт", шк)).unwrap().get_bit());
            }

            { // Обновление значения давления воздуха
                use modbus::{Value, TryFrom};
                let давление_воздуха: f32 = f32::try_from(&values.давление_воздуха as &Value).unwrap(); // todo: Обработка ошибок
                self.давление_воздуха.set(давление_воздуха);
            }
            { // Обновление флага ошибки
//                 self.klapans_error.set(Some(super::KlapansError::НедостаточноДавленияВоздуха));
                let mut new_flags = self.klapans_error.get();
                new_flags.set(KlapansError::НедостаточноДавленияВоздуха, values.давление_воздуха.is_error());
                self.klapans_error.set(new_flags);
            }
        }
        
        pub(crate) async fn automation(&self) {
            let futs1 = self.klapans.iter()
                .map(|(name, prop)| {
                    let mut sub = prop.subscribe();
                    async move {
                        loop {
                            sub.changed().await;
                            let klapan = sub.borrow();
                            let _ = self.klapans_send.send((name.to_owned(), *klapan));
                        }
                    }
                });
            let futs2 = self.klapans_шк.iter()
                .map(|((_шк, name), prop)| {
                    let mut sub = prop.subscribe();
                    async move {
                        loop {
                            sub.changed().await;
                            let klapan = sub.borrow();
                            let _ = self.klapans_шк_send.send((name.to_owned(), *klapan));
                        }
                    }
                });

            let _fut_error = {
                let mut sub = self.давление_воздуха.subscribe();
                async move {
                    sub.changed().await;
                    let value = sub.borrow();
//                     if *value
            }};
            tokio::join!(
                futures_util::future::join_all(futs1),
                futures_util::future::join_all(futs2),
            );
        }
    }
    
    impl Default for Klapans {
        fn default() -> Self {
            let klapan_names = [
                ("ШК1", "Клапан нижнего контейнера"), // ШК1
                ("ШК3", "Клапан верхнего контейнера"), // ШК5
                ("ШК2", "Клапан подачи материала"),  // ШК2
                ("ШК5", "Клапан помольной камеры"),  // ШК3
                ("ШК4", "Клапан напуска"),           // ШК4
                ("ШК6", "Клапан насоса М5"),         // ШК6
            ];
            Klapans {
                давление_воздуха: Property::default(),
                klapans: klapan_names.iter().map(|&(_, n)| (n.to_owned(), Property::<bool>::default())).collect(),
                klapans_send: broadcast::channel(16).0,
                klapans_шк: klapan_names.iter().map(|&(шк, n)| ((шк.to_owned(), n.to_owned()), Property::<bool>::default())).collect(),
                klapans_шк_send: broadcast::channel(16).0,

                klapans_error: Property::default(),
            }
        }
    }
}
