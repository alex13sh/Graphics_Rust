use super::{Value, ValueArc, ValueID};
use super::init::ValueID as IValueID;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

#[derive(Debug, Default, Clone)]
pub struct ModbusValues(HashMap<ValueID, ValueArc>);

impl ModbusValues {
    pub fn new() -> Self {
        ModbusValues(HashMap::new())
    }

    pub fn get_value_arc(&self, id: impl Into<IValueID>) -> Option<ValueArc> {
        let id = id.into();
        self.0.iter().find(|(id_, v)| *id_ == &id)
            .map(|(id, v)| v.clone())
    }

    pub fn set_value(&self, name: &str, value: u32) -> ValueArc {
        let val = self.get_value_arc(name).unwrap();
        val.update_value(value);
        val
    }
    pub fn get_values_by_name(&self, names: &[&str]) -> ModbusValues {
        ModbusValues (
            names.iter().filter_map(
                |&name| self.get_value_arc(name).map(|v|
                    (v.id.clone(), v)
                )
            ).collect()
        )
    }
    
    pub fn get_values_by_id(&self, f: impl Fn(&ValueID) -> bool) -> ModbusValues {
        ModbusValues (
            self.0.iter().filter(|(id, v)| f(id))
                .map(|(id, v)| (id.clone(), v.clone()))
                .collect()
        )
    }
    pub fn unwrap_one(self) -> Result<ValueArc, ()> {
        if self.0.len() == 1 {
            Ok(self.0.into_iter().next().unwrap().1)
        } else {
            Err(())
        }
    }
    
    pub fn iter_values(&self) -> impl Iterator<Item=(u16, &ValueID, (u32, f32))> + '_ {
        self.0.iter().filter_map(|(k, v)| Some((
                v.address(),
                k,
                (v.value(), v.try_value_as_f32()?)
            ))
        )
    }
}

impl IntoIterator for ModbusValues {
    type Item = <HashMap<ValueID, ValueArc> as IntoIterator>::Item;
    type IntoIter = <HashMap<ValueID, ValueArc> as IntoIterator>::IntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl std::ops::Add<ModbusValues> for ModbusValues {
    type Output = ModbusValues;
    fn add(self, other: Self) -> Self {
        ModbusValues (
            self.0.into_iter()
                .chain(other.0.into_iter())
                .collect()
        )
    }
}

impl <'a> std::ops::Add<&'a ModbusValues> for &'a ModbusValues {
    type Output = ModbusValues;
    fn add(self, other: Self) -> Self::Output {
        ModbusValues (
            self.0.iter()
                .chain(other.0.iter())
                .map(|(name, v)| (name.clone(), v.clone()))
                .collect()
        )
    }
}

impl From<HashMap<ValueID, ValueArc>> for ModbusValues {
    fn from(values: HashMap<ValueID, ValueArc>) -> Self {
        ModbusValues(values)
    }
}

impl From<ModbusValues> for Vec<ValueArc> {
    fn from(values: ModbusValues) -> Self {
        values.0.into_iter().map(|v| v.1.clone()).collect()
    }
}
impl From<Vec<ValueArc>> for ModbusValues {
    fn from(values: Vec<ValueArc>) -> Self {
        values.into_iter().collect()
    }
}

impl std::iter::FromIterator<ValueArc> for ModbusValues {
    fn from_iter<I: IntoIterator<Item=ValueArc>>(iter: I) -> Self {
        let mut c = ModbusValues::new();
        
        for i in iter {
            c.insert(i.id().clone(), i);
        }

        c
    }
}


impl Deref for ModbusValues {
    type Target = HashMap<ValueID, ValueArc>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for ModbusValues {
//     type Target = HashMap<String, ValueArc>;
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
 
