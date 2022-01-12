// pub use async_broadcast::broadcast;
// pub use postage::broadcast::channel as broadcast;
// pub use postage::broadcast::Sender;
pub use postage::prelude::*;

pub fn broadcast<T: Clone>(cap: usize) -> (Sender<T>, postage::broadcast::Receiver<T>)
{
    let (s, r) = postage::broadcast::channel(cap);
//     s.set_overflow(true);
    (Sender::from(s), r)
}


pub struct Sender<T>(
//     std::pin::Pin<
        postage::broadcast::Sender<T>
    ,
    /*value:*/ Option<T>
);


impl <T> From<postage::broadcast::Sender<T>> for Sender<T> {
    fn from(f: postage::broadcast::Sender<T>) -> Self {
        Self(f, None)
    }
}

pub mod wrap_sink {
    use postage::sink::PollSend;
    use postage::sink::SendError;

    use postage::sink::Sink;
    use postage::Context;
    use std::pin::Pin;

    impl<T> Sink for super::Sender<T>
    where
        T: Clone,
    {
        type Item = T;

        fn poll_send(
            self: Pin<&mut Self>,
            cx: &mut Context<'_>,
            value: Self::Item,
        ) -> PollSend<Self::Item> {
            Pin::new(&mut unsafe{self.get_unchecked_mut()}.0).poll_send(cx, value)
        }
    }
}

use futures::Sink as _;

pub mod features_sink {
    pub use postage::prelude::Sink as _;
    use futures::task::{Poll, Context};
    use std::pin::Pin;
    use postage::sink::PollSend;
    use postage::sink::SendError;

    impl<T:Clone> futures::Sink<T> for super::Sender<T> {
        type Error = postage::sink::SendError<()>;

        // get_unchecked_mut
        fn poll_ready(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            let res = if let Some(value) = self.1.clone() {
                match self.poll_send(&mut ctx.into(), value.clone()) {
                PollSend::Ready => Poll::Ready(Ok(())),
                PollSend::Pending(_) => Poll::Pending,
                PollSend::Rejected(_) => Poll::Ready(Err(SendError(()))),
                }
            } else {
//                 Poll::Ready(Err(SendError(())))
//                 Poll::Pending
                Poll::Ready(Ok(()))
            };
            dbg!(&res);
            res
        }

        fn start_send(self: Pin<&mut Self>, item: T) -> Result<(), Self::Error> {
            dbg!("start_send");
            unsafe{self.get_unchecked_mut()}.1 = Some(item);
            Ok(())
        }

        fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }

        fn poll_close(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
//             self.0.close();
            Poll::Ready(Ok(()))
        }
    }
}
