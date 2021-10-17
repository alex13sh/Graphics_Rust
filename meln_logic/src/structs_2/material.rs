use modbus::{ValueArc, ModbusValues};

pub struct Material {
    клапан_подачи_материала: ValueArc,      // ШК2
    клапан_верхнего_контейнера: ValueArc,   // ШК5
    клапан_помольной_камеры: ValueArc,      // ШК3
    клапан_нижнего_контейнера: ValueArc,    // ШК1
}

impl From<&ModbusValues> for Material {
    fn from(values: &ModbusValues) -> Self {
        Material {
            клапан_подачи_материала: values.get_value_arc("Клапан подачи материала").unwrap(),
            клапан_верхнего_контейнера: values.get_value_arc("Клапан верхнего контейнера").unwrap(),
            клапан_помольной_камеры: values.get_value_arc("Клапан помольной камеры").unwrap(),
            клапан_нижнего_контейнера: values.get_value_arc("Клапан нижнего контейнера").unwrap(),
        }
    }
}

pub mod watcher {
    use crate::Property;
    pub struct Material {
        клапан_подачи_материала: Property<bool>,
        клапан_верхнего_контейнера: Property<bool>,
        клапан_помольной_камеры: Property<bool>,
        клапан_нижнего_контейнера: Property<bool>,
    }
    
    impl Material {
        pub(crate) fn update_property(&self, values: &super::Material) {
            self.клапан_подачи_материала.set(values.клапан_подачи_материала.get_bit());
            self.клапан_верхнего_контейнера.set(values.клапан_верхнего_контейнера.get_bit());
            self.клапан_помольной_камеры.set(values.клапан_помольной_камеры.get_bit());
            self.клапан_нижнего_контейнера.set(values.клапан_нижнего_контейнера.get_bit());
        }
    }
}
