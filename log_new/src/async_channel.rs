// pub use async_broadcast::broadcast;

pub struct Sender<T>(async_broadcast::Sender<T>);

impl <T: Clone> Sender<T> {
    pub async fn send(&self, m: T) {
        let res = self.0.broadcast(m).await;
        res.unwrap();
    }
}

impl <T> From<async_broadcast::Sender<T>> for Sender<T> {
    fn from(f: async_broadcast::Sender<T>) -> Self {
        Self(f)
    }
}

pub fn broadcast<T>(cap: usize) -> (Sender<T>, async_broadcast::Receiver<T>)
{
    let (s, r) = async_broadcast::broadcast(cap);
    (Sender::from(s), r)
}
