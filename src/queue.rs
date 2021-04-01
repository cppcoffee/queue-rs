use std::ptr;
use std::sync::atomic::{AtomicPtr, Ordering};

unsafe impl<T: Send> Sync for Queue<T> {}

struct Node<T: Send> {
    next: AtomicPtr<Node<T>>,
    value: Option<T>,
}

impl<T: Send> Node<T> {
    fn new(v: T) -> Self {
        Self {
            next: Default::default(),
            value: Some(v),
        }
    }

    fn sentinel() -> Self {
        Self {
            next: Default::default(),
            value: None,
        }
    }
}

pub struct Queue<T: Send> {
    head: AtomicPtr<Node<T>>,
    tail: AtomicPtr<Node<T>>,
}

impl<T: Send> Drop for Queue<T> {
    fn drop(&mut self) {
        let mut p = self.head.load(Ordering::Relaxed);

        while !p.is_null() {
            unsafe {
                let next = (*p).next.load(Ordering::Relaxed);
                Box::from_raw(p);
                p = next;
            }
        }
    }
}

impl<T: Send> Queue<T> {
    pub fn new() -> Self {
        let dummy_ptr = Box::into_raw(Box::new(Node::sentinel()));

        Self {
            head: AtomicPtr::new(dummy_ptr),
            tail: AtomicPtr::new(dummy_ptr),
        }
    }

    pub fn enq(&self, v: T) {
        unsafe { self.try_enq(v) }
    }

    unsafe fn try_enq(&self, v: T) {
        let node = Box::into_raw(Box::new(Node::new(v)));

        loop {
            let p = self.tail.load(Ordering::Acquire);

            if (*p)
                .next
                .compare_exchange(ptr::null_mut(), node, Ordering::Release, Ordering::Relaxed)
                .is_ok()
            {
                let _ = self
                    .tail
                    .compare_exchange(p, node, Ordering::Acquire, Ordering::Relaxed);
                return;
            } else {
                let _ = self.tail.compare_exchange(
                    p,
                    (*p).next.load(Ordering::Acquire),
                    Ordering::Release,
                    Ordering::Relaxed,
                );
            }
        }
    }

    pub fn deq(&self) -> Option<T> {
        unsafe { self.try_deq() }
    }

    unsafe fn try_deq(&self) -> Option<T> {
        let mut p;

        loop {
            p = self.head.load(Ordering::Acquire);

            if (*p).next.load(Ordering::Acquire).is_null() {
                return None;
            }

            if self
                .head
                .compare_exchange(
                    p,
                    (*p).next.load(Ordering::Acquire),
                    Ordering::Release,
                    Ordering::Relaxed,
                )
                .is_ok()
            {
                break;
            }
        }

        (*(*p).next.load(Ordering::Acquire)).value.take()
    }
}
