use std::sync::Arc;

use crate::ring_buffer::RingBuffer;

#[derive(Clone)]
// this is bad
pub struct Channel<T> {
    buffer: Arc<RingBuffer<T>>,
}

impl<T> Channel<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: Arc::new(RingBuffer::new(capacity)),
        }
    }
    pub fn try_send(&self, value: T) -> Result<(), T> {
        self.buffer.push(value)
    }
    pub fn try_recv(&self) -> Option<T> {
        self.buffer.pop()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chan_is_send() {
        fn is_send(_: impl Send) {}
        is_send(Channel::<i32>::new(1));
    }

    #[test]
    fn chan_is_sync() {
        fn is_sync(_: impl Sync) {}
        is_sync(Channel::<i32>::new(1));
    }
}
