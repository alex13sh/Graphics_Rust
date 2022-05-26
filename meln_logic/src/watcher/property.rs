#![allow(dead_code)]
use tokio::sync::watch;
use std::fmt::Debug;

pub struct Property<T> {
    sender: watch::Sender<T>
}

impl<T> Debug for Property<T> 
    where T: Debug
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Property").field(&*self.sender.borrow()).finish()
    }
}

impl <T: Default> Default for Property <T> {
    fn default() -> Self {
        Property::new(T::default())
    }
}

impl <T> Property<T> {
    pub(crate) fn new(value: T) -> Self {
        Property{
            sender: watch::channel(value).0
        }
    }

    #[track_caller]
    pub(crate) fn set(&self, value: T) 
    where T: PartialEq + Debug
    {
        if self.sender.is_closed() {
            println!("[{}] Property dont set {:?}", core::panic::Location::caller(), value);
            return;
        }
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

    pub fn get_opt(&self) -> Option<T>
    where T: Clone
    {
        if self.sender.is_closed() {
            println!("[{}] Property dont get", core::panic::Location::caller());
            return None;
        }
        Some(self.sender.borrow().clone())
    }

    // pub fn get_ref(&self) -> &T
    // {
    //     &**self.sender.borrow()
    // }
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

pub(in crate) use changed_any;
pub(in crate) use changed_all;
