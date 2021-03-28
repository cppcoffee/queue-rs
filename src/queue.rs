use std::mem::MaybeUninit;
use std::ptr;
use std::sync::atomic::{AtomicPtr, Ordering};

unsafe impl<T: Send> Sync for Queue<T> {}

#[derive(Debug)]
struct Node<T: Send> {
    next: AtomicPtr<Node<T>>,
    value: MaybeUninit<*mut T>,
}

impl<T: Send> Node<T> {
    fn new(v: *mut T) -> Self {
        Self {
            next: AtomicPtr::default(),
            value: MaybeUninit::new(v),
        }
    }

    fn sentinel() -> Self {
        Self {
            next: Default::default(),
            value: MaybeUninit::uninit(),
        }
    }
}

#[derive(Debug)]
pub struct Queue<T: Send> {
    head: AtomicPtr<Node<T>>,
    tail: AtomicPtr<Node<T>>,
}

impl<T: Send> Queue<T> {
    pub fn new() -> Self {
        let dummy_ptr = Box::into_raw(Box::new(Node::sentinel()));

        Self {
            head: AtomicPtr::new(dummy_ptr),
            tail: AtomicPtr::new(dummy_ptr),
        }
    }

    pub fn enq(&self, v: *mut T) {
        let node = Box::into_raw(Box::new(Node::new(v)));

        loop {
            let tail = self.tail.load(Ordering::Acquire);
            let next = unsafe { tail.as_ref() }
                .unwrap()
                .next
                .load(Ordering::Relaxed);

            if next.is_null() {
                if unsafe { tail.as_ref() }
                    .unwrap()
                    .next
                    .compare_exchange(next, node, Ordering::Release, Ordering::Relaxed)
                    .is_ok()
                {
                    let _ = self.tail.compare_exchange(
                        tail,
                        node,
                        Ordering::Release,
                        Ordering::Relaxed,
                    );
                    return;
                }
            } else {
                let _ =
                    self.tail
                        .compare_exchange(tail, next, Ordering::Release, Ordering::Relaxed);
            }
        }
    }

    pub fn deq(&self) -> Option<*mut T> {
        loop {
            let p = self.head.load(Ordering::Acquire);
            let next = unsafe { p.as_ref() }.unwrap().next.load(Ordering::Acquire);

            if ptr::eq(next, ptr::null_mut()) {
                return None;
            }

            if self
                .head
                .compare_exchange(p, next, Ordering::Release, Ordering::Relaxed)
                .is_ok()
            {
                unsafe {
                    let node = Box::from_raw(p);
                    return Some(ptr::read(node.as_ref().value.as_ptr()));
                }
            }
        }
    }
}

mod tests {
    #[test]
    fn queue_op() {
        let q = super::Queue::new();
        q.enq(1 as *mut u8);
        q.enq(2 as *mut u8);
        assert_eq!(q.deq(), Some(1 as *mut u8));
        assert_eq!(q.deq(), Some(2 as *mut u8));
        assert_eq!(q.deq(), None);
    }

    #[test]
    fn empty_queue() {
        let q: super::Queue<Option<u8>> = super::Queue::new();
        assert_eq!(q.deq(), None);
        assert_eq!(q.deq(), None);
    }
}
