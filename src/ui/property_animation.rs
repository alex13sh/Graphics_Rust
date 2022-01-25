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
