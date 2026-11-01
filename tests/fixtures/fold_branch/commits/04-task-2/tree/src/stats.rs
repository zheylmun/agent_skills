use crate::buffer::SampleBuffer;

/// The mean of every reading in the buffer, or `None` when it is empty.
pub fn mean(buffer: &SampleBuffer) -> Option<f32> {
    if buffer.is_empty() {
        return None;
    }
    Some(buffer.iter().sum::<f32>() / buffer.len() as f32)
}

/// The mean of the newest `window` readings, or `None` when `window` is 0
/// or the buffer holds fewer than `window` readings.
pub fn window_mean(buffer: &SampleBuffer, window: usize) -> Option<f32> {
    if window == 0 || buffer.len() < window {
        return None;
    }
    let first = buffer.len() - window;
    let mut total = 0.0;
    for (index, value) in buffer.iter().enumerate() {
        if index >= first {
            total += value;
        }
    }
    Some(total / window as f32)
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
        assert_eq!(window_mean(&filled(&[9.0, 2.0, 4.0]), 2), Some(3.0));
    }

    #[test]
    fn window_mean_needs_enough_readings() {
        assert_eq!(window_mean(&filled(&[1.0, 2.0]), 3), None);
        assert_eq!(window_mean(&filled(&[1.0, 2.0]), 0), None);
    }
}
