#![allow(dead_code)]
use tokio::sync::watch;
use std::fmt::Debug;

pub struct Property<T> {
    sender: watch::Sender<T>
}

impl <T> Property<T> {
    pub(crate) fn new(value: T) -> Self {
        Property{
            sender: watch::channel(value).0
        }
    }
    pub(crate) fn set(&self, value: T) 
    where T: PartialEq + Debug
    {
        if self.sender.is_closed() {return;}
        if self.sender.borrow().ne(&value) {
            self.sender.send(value).unwrap();
        }
    }
    pub(crate) fn send(&self, value: T) 
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

// pub fn changed_any<I, T>(iter: I) 
// where
//     I: IntoIterator,
//     I::Item: watch::Receiver<T>,
// {
//     futures::future::select_all(iter.map(|r| r.changed()));
// }

// #[macro_export]
macro_rules! changed_any(
    ($($r:ident),+) => {
        tokio::select! {
        $(_ = $r.changed() => {}),+
        };
    }
);

// #[macro_export]
macro_rules! changed_all(
    ($($r:ident),+) => {
        let res = tokio::join! (
            $($r.changed()),+
        );
        dbg!(res);
    }
);

pub(in crate::structs_2) use changed_any;
pub(in crate::structs_2) use changed_all;
