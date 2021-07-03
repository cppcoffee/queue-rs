use std::sync::atomic::Ordering::{Acquire, Relaxed, Release};

use crossbeam_epoch::{self as epoch, Atomic, Owned, Shared};

unsafe impl<T: Send> Sync for Queue<T> {}

struct Node<T: Send> {
    next: Atomic<Node<T>>,
    data: Option<T>,
}

impl<T: Send> Node<T> {
    fn new(v: T) -> Self {
        Self {
            next: Default::default(),
            data: Some(v),
        }
    }

    fn sentinel() -> Self {
        Self {
            next: Atomic::null(),
            data: None,
        }
    }
}

pub struct Queue<T: Send> {
    head: Atomic<Node<T>>,
    tail: Atomic<Node<T>>,
}

impl<T: Send> Queue<T> {
    pub fn new() -> Self {
        let q = Queue {
            head: Atomic::null(),
            tail: Atomic::null(),
        };
        let sentinel = Owned::new(Node::sentinel());

        let guard = unsafe { &epoch::unprotected() };

        let sentinel = sentinel.into_shared(guard);
        q.head.store(sentinel, Relaxed);
        q.tail.store(sentinel, Relaxed);
        q
    }

    pub fn enq(&self, v: T) {
        unsafe { self.try_enq(v) }
    }

    unsafe fn try_enq(&self, v: T) {
        let guard = &epoch::pin();
        let node = Owned::new(Node::new(v)).into_shared(guard);

        loop {
            let p = self.tail.load(Acquire, guard);

            if (*p.as_raw())
                .next
                .compare_exchange(Shared::null(), node, Release, Relaxed, guard)
                .is_ok()
            {
                let _ = self.tail.compare_exchange(p, node, Release, Relaxed, guard);
                return;
            } else {
                let _ = self.tail.compare_exchange(
                    p,
                    (*p.as_raw()).next.load(Acquire, guard),
                    Release,
                    Relaxed,
                    guard,
                );
            }
        }
    }

    pub fn deq(&self) -> Option<T> {
        unsafe { self.try_deq() }
    }

    unsafe fn try_deq(&self) -> Option<T> {
        let guard = &epoch::pin();

        loop {
            let p = self.head.load(Acquire, guard);

            if (*p.as_raw()).next.load(Acquire, guard).is_null() {
                return None;
            }

            if self
                .head
                .compare_exchange(
                    p,
                    (*p.as_raw()).next.load(Acquire, guard),
                    Release,
                    Relaxed,
                    guard,
                )
                .is_ok()
            {
                let next = (*p.as_raw()).next.load(Acquire, guard).as_raw() as *mut Node<T>;
                return (*next).data.take();
            }
        }
    }
}
