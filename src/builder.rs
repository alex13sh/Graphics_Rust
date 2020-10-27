pub struct Builder<T> {
    pub obj: T
}
impl <T> Builder<T> 
where T: Default
{
    pub fn new() -> Self {
        Self {
            obj: T::default()
        }
    }
    
    pub fn complete(self) -> T {
        self.obj
    }
}
