use iced_futures::*;
use async_stream::stream;

pub struct PropertyAnimation<T> {
    pub name: String,
    pub sub: meln_logic::watcher::Subscription<T>,
}

impl <T> PropertyAnimation<T> {
    pub fn new(name: &str, sub: meln_logic::watcher::Subscription<T>) -> Self {
        PropertyAnimation {
            name: name.to_owned(),
            sub: sub,
        }
    }
    pub fn new_sub<H, E>(name: &str, sub: meln_logic::watcher::Subscription<T>) -> subscription::Subscription<H, E, T>
        where H: std::hash::Hasher,
        T: Clone + Send + Sync + 'static
    {
        subscription::Subscription::from_recipe(
            Self::new(name, sub)
        )
    }
}

impl<H, I, T> subscription::Recipe<H, I> for PropertyAnimation<T>
where
    H: std::hash::Hasher,
    T: Clone + Send + Sync + 'static
{
    type Output = T;

    fn hash(&self, state: &mut H) {
        use std::hash::Hash;

        std::any::TypeId::of::<Self>().hash(state);
//         self.sub.hash(state);
        self.name.hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: futures::stream::BoxStream<'static, I>,
    ) -> futures::stream::BoxStream<'static, Self::Output> {
        let mut sub = self.sub;
        let gen = stream! {
            loop {
                sub.changed().await.unwrap();
                let val = sub.borrow().clone();
                yield val;
            }
        };
        Box::pin(gen)
    }    
}


pub struct BroadcastAnimation<T> {
    pub name: String,
    pub sub: tokio::sync::broadcast::Receiver<T>,
}

impl <T> BroadcastAnimation<T> {
    pub fn new(name: &str, sub: tokio::sync::broadcast::Receiver<T>) -> Self {
        BroadcastAnimation {
            name: name.to_owned(),
            sub: sub,
        }
    }
    pub fn new_sub<H, E>(name: &str, sub: tokio::sync::broadcast::Receiver<T>) -> subscription::Subscription<H, E, T>
        where H: std::hash::Hasher,
        T: Clone + Send + Sync + 'static
    {
        subscription::Subscription::from_recipe(
            Self::new(name, sub)
        )
    }
}

impl<H, I, T> subscription::Recipe<H, I> for BroadcastAnimation<T>
where
    H: std::hash::Hasher,
    T: Clone + Send + Sync + 'static
{
    type Output = T;

    fn hash(&self, state: &mut H) {
        use std::hash::Hash;

        std::any::TypeId::of::<Self>().hash(state);
//         self.sub.hash(state);
        self.name.hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: futures::stream::BoxStream<'static, I>,
    ) -> futures::stream::BoxStream<'static, Self::Output> {
        let mut sub = self.sub;
        let gen = stream! {
            loop {
                let val = sub.recv().await.unwrap();
                yield val;
            }
        };
        Box::pin(gen)
    }    
}

use std::sync::Arc;

pub struct DeviceUpdate<Message> {
    device: Arc<modbus::Device>,
    message: fn(Arc<modbus::Device>) -> Message,
}

impl <M> DeviceUpdate<M> {
    pub fn new(d: Arc<modbus::Device>, m: fn(Arc<modbus::Device>) -> M) -> Self {
        Self {
            device: d,
            message: m,
        }
    }
}

impl<H, I, M> subscription::Recipe<H, I> for DeviceUpdate<M>
where
    H: std::hash::Hasher,
    M: Clone + Send + Sync + 'static
{
    type Output = M;
    fn hash(&self, state: &mut H) {
        use std::hash::Hash;

        std::any::TypeId::of::<Self>().hash(state);
        self.device.id().hash(state);
//         self.device.is_connect().hash(state);
        self.message.hash(state);
    }
    fn stream(
        self: Box<Self>,
        _input: futures::stream::BoxStream<'static, I>,
    ) -> futures::stream::BoxStream<'static, Self::Output> {
        use tokio::time::sleep;
        use std::time::Duration;
        let gen = stream! {
            if self.device.config.interval_update_in_sec != 0.0 {
                loop {
                    let interval = self.device.config.interval_update_in_sec;
                    sleep(Duration::from_millis((interval * 1_000.0) as u64)).await;
                    if self.device.is_connect() {
                        yield (self.message)(self.device.clone());
                    }
                }
            }
        };
        Box::pin(gen)
    }
}

pub struct MyStream <T> {
    pub name: String,
    pub stream: futures::stream::BoxStream<'static, T>,
}

impl<H, I, T> subscription::Recipe<H, I> for MyStream<T>
where
    H: std::hash::Hasher,
    T: Clone + Send + Sync + 'static
{
    type Output = T;

    fn hash(&self, state: &mut H) {
        use std::hash::Hash;

        std::any::TypeId::of::<Self>().hash(state);
//         self.sub.hash(state);
        self.name.hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: futures::stream::BoxStream<'static, I>,
    ) -> futures::stream::BoxStream<'static, Self::Output> {
        self.stream
    }
}
