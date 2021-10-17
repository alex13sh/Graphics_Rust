use tokio::sync::watch;
use std::fmt::Debug;

pub struct Property<T> {
    sender: watch::Sender<T>
}

impl <T> Property<T> {
    pub fn new(value: T) -> Self {
        Property{
            sender: watch::channel(value).0
        }
    }
    pub fn set(&self, value: T) 
    where T: Eq + Debug
    {
        if self.sender.is_closed() {return;}
        if self.sender.borrow().ne(&value) {
            self.sender.send(value).unwrap();
        }
    }
    pub fn send(&self, value: T) 
    where T: Debug 
    {
        if self.sender.is_closed() {return;}
        self.sender.send(value).unwrap();
    }
    pub fn get(&self) -> T 
    where T: Clone
    {
        self.sender.borrow().clone()
    }
    pub fn subscribe(&self) -> watch::Receiver<T> {
        self.sender.subscribe()
    }
}
