use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};
use std::thread;
use std::time::{Duration, Instant};

pub struct Delay {
    when: Instant,
    waker: Arc<Mutex<Option<Waker>>>,
}

impl Delay {
    pub fn new(dur: Duration) -> Self {
        Self {
            when: Instant::now() + dur,
            waker: Arc::new(Mutex::new(None)),
        }
    }
}

impl Future for Delay {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if Instant::now() >= self.when {
            Poll::Ready(())
        } else {
            let mut w = self.waker.lock().unwrap();
            *w = Some(cx.waker().clone());

            let when = self.when;
            let waker = self.waker.clone();
            thread::spawn(move || {
                let now = Instant::now();
                if now < when {
                    thread::sleep(when - now);
                }

                if let Some(w) = waker.lock().unwrap().take() {
                    println!("waking up");
                    w.wake();
                }
            });

            Poll::Pending
        }
    }
}
