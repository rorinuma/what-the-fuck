use delay::Delay;
use std::collections::VecDeque;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
use std::thread;
use std::time::Duration;

mod delay;

struct Task {
    fut: Mutex<Pin<Box<dyn Future<Output = ()> + Send>>>,
    queue: Arc<Mutex<VecDeque<Arc<Task>>>>,
}

const VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop_waker);

async fn say_hello(name: &str) {
    println!("hello, {name}!");
    Delay::new(Duration::from_secs(2)).await;
    println!("bye, {name}!");
}

async fn my_async_function() {
    say_hello("hmmmm").await;
    say_hello("mmmmmm").await;
}

async fn other_function() {
    println!("it's running!!");
    println!("it's running!!");
}

fn main() {
    let queue: Arc<Mutex<VecDeque<Arc<Task>>>> = Arc::new(Mutex::new(VecDeque::new()));

    queue.lock().unwrap().push_back(Arc::new(Task {
        fut: Mutex::new(Box::pin(my_async_function())),
        queue: queue.clone(),
    }));

    queue.lock().unwrap().push_back(Arc::new(Task {
        fut: Mutex::new(Box::pin(other_function())),
        queue: queue.clone(),
    }));

    loop {
        let task_opt = queue.lock().unwrap().pop_front();

        if let Some(task) = task_opt {
            unsafe {
                let waker = make_waker(task.clone());
                let mut cx = Context::from_waker(&waker);

                let mut fut = task.fut.lock().unwrap();

                match fut.as_mut().poll(&mut cx) {
                    Poll::Pending => {
                        task.queue.lock().unwrap().push_back(task.clone());
                    }
                    Poll::Ready(_) => println!("Task done"),
                }
            }
        } else {
            thread::sleep(Duration::from_millis(10));
        }
    }
}

unsafe fn make_waker(task: Arc<Task>) -> Waker {
    unsafe {
        let raw_waker = RawWaker::new(Arc::into_raw(task.clone()) as *const (), &VTABLE);
        Waker::from_raw(raw_waker)
    }
}

unsafe fn clone(data: *const ()) -> RawWaker {
    unsafe {
        let arc = Arc::from_raw(data as *const Task);
        let cloned = arc.clone();

        std::mem::forget(arc);
        RawWaker::new(Arc::into_raw(cloned) as *const (), &VTABLE)
    }
}

unsafe fn wake(data: *const ()) {
    unsafe {
        let task = Arc::from_raw(data as *const Task);
        task.queue.lock().unwrap().push_back(task.clone());
    }
}

unsafe fn wake_by_ref(data: *const ()) {
    unsafe {
        let task = Arc::from_raw(data as *const Task);
        task.queue.lock().unwrap().push_back(task.clone());

        std::mem::forget(task);
    }
}

unsafe fn drop_waker(data: *const ()) {
    unsafe {
        let _ = Arc::from_raw(data as *const Task);
    }
}
