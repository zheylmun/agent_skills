use std::collections::VecDeque;

/// The most recent readings from one sensor, oldest first. A full buffer
/// drops its oldest reading to make room for a new one.
pub struct SampleBuffer {
    values: VecDeque<f32>,
    capacity: usize,
}

impl SampleBuffer {
    /// An empty buffer that holds at most `capacity` readings.
    pub fn new(capacity: usize) -> Self {
        Self {
            values: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    /// Adds a reading, dropping the oldest one when the buffer is full.
    pub fn push(&mut self, value: f32) {
        if self.values.len() == self.capacity {
            self.values.pop_front();
        }
        self.values.push_back(value);
    }

    /// The newest reading, or `None` when the buffer is empty.
    pub fn latest(&self) -> Option<&f32> {
        self.values.back()
    }

    /// How many readings the buffer holds.
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Whether the buffer holds no readings.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    /// The readings, oldest first.
    pub fn iter(&self) -> impl Iterator<Item = f32> + '_ {
        self.values.iter().copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn latest_is_the_newest_push() {
        let mut buffer = SampleBuffer::new(3);
        assert_eq!(buffer.latest(), None);
        buffer.push(1.0);
        buffer.push(2.0);
        assert_eq!(buffer.latest(), Some(&2.0));
    }

    #[test]
    fn a_full_buffer_drops_the_oldest() {
        let mut buffer = SampleBuffer::new(2);
        buffer.push(1.0);
        buffer.push(2.0);
        buffer.push(3.0);
        assert_eq!(buffer.iter().collect::<Vec<_>>(), vec![2.0, 3.0]);
    }
}
