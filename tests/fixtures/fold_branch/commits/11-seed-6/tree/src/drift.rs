use crate::buffer::SampleBuffer;
use crate::stats::mean;

/// Whether the newest reading is more than `limit` away from the mean of
/// the buffer. An empty buffer has not drifted.
pub fn drifted(buffer: &SampleBuffer, limit: f32) -> bool {
    match (buffer.latest(), mean(buffer)) {
        (Some(latest), Some(mean)) => (latest - mean).abs() > limit,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_reading_far_from_the_mean_has_drifted() {
        let mut buffer = SampleBuffer::new(4);
        for value in [1.0, 1.0, 1.0, 9.0] {
            buffer.push(value);
        }
        assert!(drifted(&buffer, 2.0));
        assert!(!drifted(&buffer, 10.0));
    }

    #[test]
    fn an_empty_buffer_has_not_drifted() {
        assert!(!drifted(&SampleBuffer::new(4), 0.0));
    }
}
