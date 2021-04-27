
pub trait ValueExt<T> {
    fn name(&self) -> &String;
    fn value(&self) -> T;
    fn set_value(&mut self, value: T);
}
trait NewValue {
    fn is_new_value(&self) -> bool;
    fn reset_new_value(&mut self);
}

// enum Value {
//     Float(ValueFloat),
//     Bit(ValueBit),
// }

struct ValueFloat {
    name: String,
    value: f32,
    value_error: (f32, f32),
    // log
    is_new_value: bool,
}

impl ValueExt<f32> for ValueFloat {
    fn name(&self) -> &String {
        &self.name
    }
    fn value(&self) -> f32 {
        self.value
    }
    fn set_value(&mut self, value: f32) {
        if self.value != value {
            self.value = value;
            self.is_new_value = true;
        }
    }
}

impl NewValue for ValueFloat {
    fn is_new_value(&self) -> bool {
        self.is_new_value
    }
    fn reset_new_value(&mut self) {
        self.is_new_value = false
    }
}

struct ValueBit {
    name: String,
    bit: bool,
    is_new_value: bool,
}

impl ValueExt<bool> for ValueBit {
    fn name(&self) -> &String {
        &self.name
    }
    fn value(&self) -> bool {
        self.bit
    }
    fn set_value(&mut self, value: bool) {
        if self.bit != value {
            self.bit = value;
            self.is_new_value = true;
        }
    }
}

impl NewValue for ValueBit {
    fn is_new_value(&self) -> bool {
        self.is_new_value
    }
    fn reset_new_value(&mut self) {
        self.is_new_value = false
    }
}

trait NewValueExt<T> : ValueExt<T> + NewValue {}
struct ValueUpdate<T>(Box<dyn NewValueExt<T>>, Box<dyn ValueExt<T>>);
pub struct QueueUpdate {
    values_float: Vec<ValueUpdate<f32>>,
    values_bit: Vec<ValueUpdate<bool>>,
}

impl QueueUpdate {

    pub fn empty() -> Self {
        Self {
            values_float: Vec::new(),
            values_bit: Vec::new(),
        }
    }
    
    pub fn append<T: 'static>(&mut self, name: &str, val: &dyn ValueExt<T>) // -> &Box<dyn ValueExt<T> 
    {
        println!("{:?}", std::any::TypeId::of::<T>());
    }
    
    fn update_impl<T: Clone>(vals: &mut Vec<ValueUpdate<T>>) {
        for val in vals {
            if val.0.is_new_value() {
                val.1.set_value(val.0.value().clone());
            }
        }
    }
    pub fn update(&mut self) {
        Self::update_impl(&mut self.values_float);
        Self::update_impl(&mut self.values_bit);
    }
}

trait QueueUpdateExt {
    fn new() -> Self;
//     fn update(&mut self);
    fn tick(&mut self);
}
