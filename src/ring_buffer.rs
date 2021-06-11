use std::{
    cell::UnsafeCell,
    sync::atomic::{AtomicUsize, Ordering},
};

#[derive(Debug)]
pub(crate) struct RingBuffer<T> {
    data: Box<[UnsafeCell<Option<T>>]>,
    head: AtomicUsize,
    tail: AtomicUsize,
    // having len at all will make this not lockfree
    len: AtomicUsize,
}

unsafe impl<T: Send> Send for RingBuffer<T> {}
unsafe impl<T: Send> Sync for RingBuffer<T> {}

impl<T> RingBuffer<T> {
    pub(crate) fn new(capacity: usize) -> Self {
        let mut data = Vec::with_capacity(0);
        for _ in 0..capacity {
            data.push(UnsafeCell::new(None));
        }
        Self {
            data: data.into_boxed_slice(),
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            len: AtomicUsize::new(0),
        }
    }

    pub(crate) fn push(&self, value: T) -> Result<(), T> {
        let mut head = self.head.load(Ordering::SeqCst);

        loop {
            if self.len.load(Ordering::SeqCst) == self.capacity() {
                return Err(value);
            }

            let mut new_head = head + 1;
            new_head %= self.capacity();

            match self
                .head
                .compare_exchange(head, new_head, Ordering::SeqCst, Ordering::Relaxed)
            {
                Ok(_) => {
                    // SAFETY: not yet safe
                    unsafe { self.data[head].get().write(Some(value)) };

                    self.len.fetch_add(1, Ordering::SeqCst);
                    return Ok(());
                }
                Err(actual_head) => head = actual_head,
            }
        }
    }

    pub(crate) fn pop(&self) -> Option<T> {
        let mut tail = self.tail.load(Ordering::SeqCst);

        loop {
            if self.len.load(Ordering::SeqCst) == 0 {
                return None;
            }

            let mut new_tail = tail + 1;
            new_tail %= self.capacity();

            match self
                .tail
                .compare_exchange(tail, new_tail, Ordering::SeqCst, Ordering::Relaxed)
            {
                Ok(_) => {
                    // SAFETY: not yet safe
                    let value = unsafe { self.data[tail].get().read() }.take();

                    self.len.fetch_sub(1, Ordering::SeqCst);

                    debug_assert!(value.is_some());

                    return value;
                }
                Err(actual_tail) => {
                    tail = actual_tail;
                }
            }
        }
    }

    fn capacity(&self) -> usize {
        self.data.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_and_pop_once() {
        let buf = RingBuffer::new(1);

        buf.push(1).unwrap();
        assert_eq!(buf.pop().unwrap(), 1);
    }

    #[test]
    fn pushing_into_a_full_buffer_fails() {
        let buf = RingBuffer::new(1);
        dbg!(&buf);
        buf.push(1).unwrap();
        assert_eq!(dbg!(buf).push(2), Err(2));
    }

    #[test]
    fn pop_returns_none_for_an_empty_buffer() {
        let buf: RingBuffer<i32> = RingBuffer::new(1);

        assert_eq!(buf.pop(), None);
    }

    #[test]
    fn push_wraps_around() {
        let buf = RingBuffer::new(2);

        buf.push(1).unwrap();
        assert_eq!(buf.pop().unwrap(), 1);
        buf.push(2).unwrap();
        buf.push(3).unwrap();
        assert_eq!(buf.pop().unwrap(), 2);
        assert_eq!(buf.pop().unwrap(), 3);
    }
}
