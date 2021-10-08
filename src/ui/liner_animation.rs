use iced_futures::*;
use async_stream::stream;

use super::animations::Progress;

#[derive(Clone)]
pub struct LinerAnimation {
    from: f32,
    to: f32,
    steps: usize,
    duration: usize,
//     cur_step: usize,
}

impl LinerAnimation {
    pub fn new(pos: f32, steps: usize) -> Self {
        LinerAnimation {
            from: pos,
            to: pos,
            steps: steps,
            duration: 1_000,
        }
    }

    pub fn from_to(from: f32, to: f32) -> Self {
        LinerAnimation {
            from: from,
            to: to,
            steps: 10,
            duration: 1_000,
        }
    }

    pub fn steps(mut self, steps: usize) -> Self {
        self.steps = steps;
        self
    }
    pub fn duration(mut self, time: usize) -> Self {
        self.duration = time;
        self
    }
    pub fn set_to(&mut self, value: f32) {
        self.from = self.to;
        self.to = value;
//         self.cur_step = 0;
    }
    pub fn set_from_to(&mut self, from: f32, to: f32) {
        self.from = from;
        self.to = to;
//         self.cur_step = 0;
    }
    pub fn stop(&mut self) {
        self.from = self.to;
    }
}

impl<H, I> subscription::Recipe<H, I> for LinerAnimation
where
    H: std::hash::Hasher,
{
    type Output = Progress;

    fn hash(&self, state: &mut H) {
        use std::hash::Hash;

        std::any::TypeId::of::<Self>().hash(state);
        self.steps.hash(state);
        self.duration.hash(state);

//         (self.from as i32).hash(state);
        ((self.to - self.from) == 0.0).hash(state);
        (self.to as i32).hash(state);
    }

    fn stream(
        self: Box<Self>,
        _input: futures::stream::BoxStream<'static, I>,
    ) -> futures::stream::BoxStream<'static, Self::Output> {
        /*let gen_1 = stream! {
//             let steps = self.steps as u32;
            let step_ms = self.duration/self.steps;
            let mut cur_value = self.from;
            while cur_value != self.to {
                use tokio::time::sleep;
                use std::time::Duration;


                let dlt_value = (self.to - self.from) / self.steps as f32;
                while self.cur_step < self.steps {
                    let i = self.cur_step as f32;
                    cur_value = (self.from + dlt_value * i);
                    yield Progress::Value(cur_value);
                    sleep(Duration::from_millis(step_ms as u64)).await;
                }
                yield Progress::Pause;
            }
        };*/
        let gen = stream! {
            use tokio::time::sleep;
            use std::time::Duration;
            dbg!(&self.from, &self.to);
            loop {
                if self.from != self.to {
                    let step_ms = self.duration/self.steps;
                    let dlt_value = (self.to - self.from) / self.steps as f32;
                    for i in 0..=self.steps {
                        let i = i as f32;
                        let v = (self.from + dlt_value * i);
                        yield Progress::Value(v);
                        sleep(Duration::from_millis(step_ms as u64)).await;
                    }
                    yield Progress::Finished;
                }
                sleep(Duration::from_millis(500)).await;
            }
        };

        Box::pin(gen)
    }
}
