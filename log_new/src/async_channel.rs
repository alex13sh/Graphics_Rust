/* Сохраню на будущее (futures)
use tokio::sync::broadcast;
use std::future::Future;
use std::pin::Pin;

pub struct Subscribe<T> (broadcast::Sender<T>);

impl <T> Subscribe<T> {
    pub fn subscribe(&self) -> broadcast::Receiver<T> {
        self.0.subscribe()
    }
}

type BoxFut = Pin<Box<dyn Future<Output = ()> + 'static + Send>>;
type InitFut<T> = fn(Subscribe<T>) -> BoxFut;
type LogValues = broadcast::Receiver<LogValue>;
*/
