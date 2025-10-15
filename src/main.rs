use std::collections::VecDeque;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
use std::thread::sleep;
use std::time::Duration;

struct Task {
    fut: Mutex<Pin<Box<dyn Future<Output = ()> + Send>>>,
    queue: Arc<Mutex<VecDeque<Arc<Task>>>>,
}

const VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop_waker);

async fn my_async_function() {
    println!("async function running");

    sleep(Duration::from_secs(2));

    println!("it's finished");
}

fn main() {
    let queue: Arc<Mutex<VecDeque<Arc<Task>>>> = Arc::new(Mutex::new(VecDeque::new()));

    queue.lock().unwrap().push_back(Arc::new(Task {
        fut: Mutex::new(Box::pin(my_async_function())),
        queue: queue.clone(),
    }));

    while let Some(task) = queue.lock().unwrap().pop_front() {
        unsafe {
            let waker = make_waker(task.clone());
            let mut cx = Context::from_waker(&waker);

            let mut fut = task.fut.lock().unwrap();
            match fut.as_mut().poll(&mut cx) {
                Poll::Pending => {
                    task.queue.lock().unwrap().push_back(task.clone());
                }
                Poll::Ready(_) => println!("Task done!"),
            }
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
        let task = Arc::from_raw(data as *const Task);

        let task = task.clone();

        let arc_raw = Arc::into_raw(task);
        RawWaker::new(arc_raw as *const (), &VTABLE)
    }
}

unsafe fn wake(data: *const ()) {
    unsafe {
        let task = Arc::from_raw(data as *const Task);
        task.queue.lock().unwrap().push_back(task.clone());

        std::mem::forget(task);
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
