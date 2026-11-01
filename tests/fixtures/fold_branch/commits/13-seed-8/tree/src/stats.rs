use crate::buffer::SampleBuffer;

/// A number of readings to average over, never zero.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SampleCount(usize);

impl SampleCount {
    /// `None` when `count` is 0.
    pub fn new(count: usize) -> Option<Self> {
        (count > 0).then_some(Self(count))
    }

    /// The count.
    pub fn get(self) -> usize {
        self.0
    }
}

/// The mean of every reading in the buffer, or `None` when it is empty.
pub fn mean(buffer: &SampleBuffer) -> Option<f32> {
    if buffer.is_empty() {
        return None;
    }
    Some(buffer.iter().sum::<f32>() / buffer.len() as f32)
}

/// The mean of the newest `window` readings, or `None` when the buffer
/// holds fewer than `window` readings.
pub fn window_mean(buffer: &SampleBuffer, window: SampleCount) -> Option<f32> {
    let window = window.get();
    if buffer.len() < window {
        return None;
    }
    let newest = buffer.iter().skip(buffer.len() - window);
    Some(newest.sum::<f32>() / window as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn filled(values: &[f32]) -> SampleBuffer {
        let mut buffer = SampleBuffer::new(values.len());
        for &value in values {
            buffer.push(value);
        }
        buffer
    }

    fn count(n: usize) -> SampleCount {
        SampleCount::new(n).unwrap()
    }

    #[test]
    fn a_sample_count_is_never_zero() {
        assert_eq!(SampleCount::new(0), None);
        assert_eq!(SampleCount::new(3).map(SampleCount::get), Some(3));
    }

    #[test]
    fn mean_of_an_empty_buffer_is_none() {
        assert_eq!(mean(&SampleBuffer::new(4)), None);
    }

    #[test]
    fn mean_averages_every_reading() {
        assert_eq!(mean(&filled(&[1.0, 2.0, 6.0])), Some(3.0));
    }

    #[test]
    fn window_mean_averages_the_newest() {
        assert_eq!(window_mean(&filled(&[9.0, 2.0, 4.0]), count(2)), Some(3.0));
    }

    #[test]
    fn window_mean_needs_enough_readings() {
        assert_eq!(window_mean(&filled(&[1.0, 2.0]), count(3)), None);
    }
}
