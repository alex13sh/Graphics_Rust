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
//     s.set_overflow(true);
    (Sender::from(s), r)
}

use futures::task::{Poll, Context};
use std::pin::Pin;
use futures::Sink;

impl<T:Clone> Sink<T> for Sender<T> {
        type Error = async_broadcast::TrySendError<T>;

        fn poll_ready(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            if unsafe { self.get_unchecked_mut() }.0.is_full() {
                Poll::Pending
//                 Poll::Ready(Self::Error::Full(T))
            } else {
                Poll::Ready(Ok(()))
            }
        }

        fn start_send(self: Pin<&mut Self>, item: T) -> Result<(), Self::Error> {
            // TODO: impl<T> Unpin for Vec<T> {}
            unsafe { self.get_unchecked_mut() }.0.try_broadcast(item)?;
            Ok(())
        }

        fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn poll_close(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            self.0.close();
            Poll::Ready(Ok(()))
        }
    }
