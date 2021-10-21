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
}

impl From<&ModbusValues> for Klapans {
    fn from(values: &ModbusValues) -> Self {
        Klapans {
            давление_воздуха: values.get_value_arc("Давление воздуха компрессора").unwrap(),
            двигатель_компрессора_воздуха: values.get_value_arc("Двигатель компрессора воздуха").unwrap(),
            klapans: values.get_values_by_name_contains(
                &["Клапан нижнего контейнера", "Клапан верхнего контейнера",
                "Клапан подачи материала", "Клапан помольной камеры",
                "Клапан напуска", "Клапан насоса М5"]
            ),
        }
    }
}

impl Klapans {

    pub fn klapan_turn(&self, name: &str, enb: bool) {
        if self.давление_воздуха.is_error() {
            return;
        }
        
        if let Err(e) = self.klapans.set_bit(name, enb) {
            dbg!(e);
        }
    }
    fn двигатель_компрессора_воздуха_turn(&self, enb: bool) {
        self.двигатель_компрессора_воздуха.set_bit(enb);
    }
}

pub mod watcher {
    use crate::structs::Property;
    use std::collections::HashMap;
    
    pub struct Klapans {
        pub давление_воздуха: Property<f32>,
        pub klapans: HashMap<String, Property<bool>>,
        pub klapans_send: Property<(String, bool)>,
    }
    impl Klapans {
        pub(crate) fn update_property(&self, values: &super::Klapans) {
            for (k, p) in &self.klapans {
                p.set(values.klapans.get_bit(k).unwrap());
            }
        }
        
        pub(crate) async fn automation(&self) {
            loop {
                let futs = self.klapans.iter()
                    .map(|(name, prop)| {
                        let mut sub = prop.subscribe();
                        async move {
                            sub.changed().await;
                            let klapan = sub.borrow();
                            self.klapans_send.send((name.to_owned(), *klapan));
                        }
                    });
                futures_util::future::join_all(futs);
            }
        }
    }
    
    impl Default for Klapans {
        fn default() -> Self {
            Klapans {
                давление_воздуха: Property::default(),
                klapans: ["Клапан нижнего контейнера", "Клапан верхнего контейнера",
                    "Клапан подачи материала", "Клапан помольной камеры",
                    "Клапан напуска", "Клапан насоса М5"].into_iter()
                    .map(|&n| (n.to_owned(), Property::<bool>::default()))
                    .collect(),
                klapans_send: Property::default(),
            }
        }
    }
}
