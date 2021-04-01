use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;

use queue_rs::Queue;

#[test]
fn empty_queue() {
    let q: Queue<Option<u8>> = Queue::new();
    assert_eq!(q.deq(), None);
    assert_eq!(q.deq(), None);
}

#[test]
fn queue_op() {
    let q = Queue::new();
    q.enq(1);
    q.enq(2);
    assert_eq!(q.deq(), Some(1));
    assert_eq!(q.deq(), Some(2));
    assert_eq!(q.deq(), None);
}

#[test]
fn correct_verify() {
    let n_threads = 2;
    let count = 10000;

    let enq_sum = Arc::new(AtomicU64::new(0));
    let deq_sum = Arc::new(AtomicU64::new(0));
    let enq_count = Arc::new(AtomicU64::new(0));
    let deq_count = Arc::new(AtomicU64::new(0));

    let mut handles = Vec::new();
    let queue = Arc::new(Queue::new());

    for _ in 0..n_threads {
        let queue_clone = queue.clone();
        let enq_count_clone = enq_count.clone();
        let enq_sum_clone = enq_sum.clone();

        handles.push(thread::spawn(move || {
            for i in 0..count {
                queue_clone.enq(i);
                enq_sum_clone.fetch_add(i, Ordering::SeqCst);
                enq_count_clone.fetch_add(1, Ordering::SeqCst);
            }
        }));

        let total_count = count * n_threads;
        let queue_clone = queue.clone();
        let deq_count_clone = deq_count.clone();
        let deq_sum_clone = deq_sum.clone();

        handles.push(thread::spawn(move || {
            while total_count > deq_count_clone.load(Ordering::Relaxed) {
                while let Some(v) = queue_clone.deq() {
                    deq_sum_clone.fetch_add(v, Ordering::SeqCst);
                    deq_count_clone.fetch_add(1, Ordering::SeqCst);
                }
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(
        enq_count.load(Ordering::Relaxed),
        deq_count.load(Ordering::Relaxed)
    );

    assert_eq!(
        enq_sum.load(Ordering::Relaxed),
        deq_sum.load(Ordering::Relaxed)
    );
}
