use std::collections::LinkedList;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use queue_rs::Queue;

fn queue_loop_n(n: u32) -> Duration {
    let q = Queue::new();
    let earler = Instant::now();

    for i in 0..n {
        q.enq(i);
    }
    for _ in 0..n {
        q.deq();
    }

    Instant::now().duration_since(earler)
}

fn list_loop_n(n: u32) -> Duration {
    let l = Mutex::new(LinkedList::new());
    let earler = Instant::now();

    for i in 0..n {
        l.lock().unwrap().push_front(i);
    }
    for _ in 0..n {
        l.lock().unwrap().pop_back();
    }

    Instant::now().duration_since(earler)
}

fn list_thread_n_m(n_threads: u32, number: u32) -> Duration {
    let mut handles = Vec::new();
    let elapsed = Arc::new(AtomicU64::default());

    for _ in 0..n_threads {
        let ml = Mutex::new(LinkedList::new());
        let elapsed_clone = elapsed.clone();

        handles.push(thread::spawn(move || {
            let start = Instant::now();

            for i in 0..number {
                ml.lock().unwrap().push_front(i);
            }
            for _ in 0..number {
                ml.lock().unwrap().pop_back();
            }

            let nanos = Instant::now().duration_since(start).as_nanos();
            elapsed_clone.fetch_add(nanos as u64, Ordering::SeqCst);
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    Duration::from_nanos(Arc::try_unwrap(elapsed).unwrap().into_inner())
}

fn queue_thread_n_m(n: u32, m: u32) -> Duration {
    let mut handles = Vec::new();
    let elapsed = Arc::new(AtomicU64::default());

    for _ in 0..n {
        let q = Queue::new();
        let elapsed_clone = elapsed.clone();

        handles.push(thread::spawn(move || {
            let start = Instant::now();

            for i in 0..m {
                q.enq(i);
            }
            for _ in 0..m {
                q.deq();
            }

            let nanos = Instant::now().duration_since(start).as_nanos();
            elapsed_clone.fetch_add(nanos as u64, Ordering::SeqCst);
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    Duration::from_nanos(Arc::try_unwrap(elapsed).unwrap().into_inner())
}

fn main() {
    // loop operation.
    let m = 100000;
    let mut dur = queue_loop_n(m);
    println!("queue {} ops: {:?}, avg: {:?}", m, dur, dur.checked_div(m));

    dur = list_loop_n(m);
    println!("list {} ops: {:?}, avg: {:?}", m, dur, dur.checked_div(m));

    // multi-thread operation.
    for n in vec![2, 4, 8] {
        dur = queue_thread_n_m(n, m);
        println!(
            "thread {} queue ops: {:?}, avg: {:?}",
            n,
            dur,
            dur / (n * m)
        );

        dur = list_thread_n_m(n, m);
        println!("thread {} list ops: {:?}, avg: {:?}", n, dur, dur / (n * m));
    }
}
