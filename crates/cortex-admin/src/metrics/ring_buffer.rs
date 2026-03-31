//! In-memory ring buffer for metric samples (SPEC 22 §10.1).

use std::collections::VecDeque;

use crate::types::MetricSample;

/// A fixed-capacity ring buffer that stores the most recent metric samples.
///
/// When the buffer is full, inserting a new sample discards the oldest entry.
/// The default capacity is 720 entries (= 1 hour at 5-second intervals).
#[derive(Debug, Clone)]
pub struct MetricRingBuffer {
    buffer: VecDeque<MetricSample>,
    capacity: usize,
}

impl MetricRingBuffer {
    /// Create a new ring buffer with the given capacity.
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "ring buffer capacity must be > 0");
        Self {
            buffer: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    /// Create a ring buffer with the default capacity (720 entries).
    pub fn with_default_capacity() -> Self {
        Self::new(crate::constants::METRICS_RING_BUFFER_CAPACITY)
    }

    /// Push a new sample into the buffer. If the buffer is full, the oldest
    /// sample is discarded.
    pub fn push(&mut self, sample: MetricSample) {
        if self.buffer.len() == self.capacity {
            self.buffer.pop_front();
        }
        self.buffer.push_back(sample);
    }

    /// Get the most recent sample, if any.
    pub fn latest(&self) -> Option<&MetricSample> {
        self.buffer.back()
    }

    /// Get all samples within the given time range (inclusive).
    ///
    /// `from` and `to` are ISO 8601 timestamp strings. Samples whose
    /// `timestamp` falls within `[from, to]` are returned.
    pub fn range(&self, from: &str, to: &str) -> Vec<&MetricSample> {
        self.buffer
            .iter()
            .filter(|s| s.timestamp.as_str() >= from && s.timestamp.as_str() <= to)
            .collect()
    }

    /// Return the number of samples currently in the buffer.
    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    /// Return true if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    /// Return the buffer's capacity.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Drain all samples from the buffer, returning them.
    pub fn drain_all(&mut self) -> VecDeque<MetricSample> {
        std::mem::take(&mut self.buffer)
    }

    /// Clear all samples from the buffer.
    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(ts: &str, cpu: f32) -> MetricSample {
        MetricSample {
            timestamp: ts.to_string(),
            cpu_percent: cpu,
            memory_used_bytes: 4_000_000_000,
            memory_total_bytes: 16_000_000_000,
            swap_used_bytes: 0,
            swap_total_bytes: 8_000_000_000,
            running_app_count: 1,
            error_count: 0,
        }
    }

    #[test]
    fn new_buffer_is_empty() {
        let buf = MetricRingBuffer::new(10);
        assert!(buf.is_empty());
        assert_eq!(buf.len(), 0);
        assert!(buf.latest().is_none());
    }

    #[test]
    fn push_and_latest() {
        let mut buf = MetricRingBuffer::new(10);
        buf.push(sample("2026-03-30T12:00:00Z", 10.0));
        buf.push(sample("2026-03-30T12:00:05Z", 20.0));
        assert_eq!(buf.len(), 2);
        let latest = buf.latest().unwrap();
        assert_eq!(latest.cpu_percent, 20.0);
    }

    #[test]
    fn push_over_capacity_discards_oldest() {
        let mut buf = MetricRingBuffer::new(3);
        buf.push(sample("2026-03-30T12:00:00Z", 10.0));
        buf.push(sample("2026-03-30T12:00:05Z", 20.0));
        buf.push(sample("2026-03-30T12:00:10Z", 30.0));
        assert_eq!(buf.len(), 3);

        // Pushing a 4th sample should discard the first.
        buf.push(sample("2026-03-30T12:00:15Z", 40.0));
        assert_eq!(buf.len(), 3);

        let latest = buf.latest().unwrap();
        assert_eq!(latest.cpu_percent, 40.0);

        // The oldest (10.0) should be gone.
        let timestamps: Vec<&str> = buf.buffer.iter().map(|s| s.timestamp.as_str()).collect();
        assert!(!timestamps.contains(&"2026-03-30T12:00:00Z"));
        assert!(timestamps.contains(&"2026-03-30T12:00:05Z"));
    }

    #[test]
    fn range_filters_by_timestamp() {
        let mut buf = MetricRingBuffer::new(10);
        buf.push(sample("2026-03-30T12:00:00Z", 10.0));
        buf.push(sample("2026-03-30T12:00:05Z", 20.0));
        buf.push(sample("2026-03-30T12:00:10Z", 30.0));
        buf.push(sample("2026-03-30T12:00:15Z", 40.0));

        let result = buf.range("2026-03-30T12:00:05Z", "2026-03-30T12:00:10Z");
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].cpu_percent, 20.0);
        assert_eq!(result[1].cpu_percent, 30.0);
    }

    #[test]
    fn range_returns_empty_if_no_match() {
        let mut buf = MetricRingBuffer::new(10);
        buf.push(sample("2026-03-30T12:00:00Z", 10.0));
        let result = buf.range("2099-01-01T00:00:00Z", "2099-01-01T23:59:59Z");
        assert!(result.is_empty());
    }

    #[test]
    fn drain_all_returns_and_clears() {
        let mut buf = MetricRingBuffer::new(10);
        buf.push(sample("2026-03-30T12:00:00Z", 10.0));
        buf.push(sample("2026-03-30T12:00:05Z", 20.0));
        let drained = buf.drain_all();
        assert_eq!(drained.len(), 2);
        assert!(buf.is_empty());
    }

    #[test]
    fn clear_empties_buffer() {
        let mut buf = MetricRingBuffer::new(10);
        buf.push(sample("2026-03-30T12:00:00Z", 10.0));
        buf.clear();
        assert!(buf.is_empty());
    }

    #[test]
    fn default_capacity_is_720() {
        let buf = MetricRingBuffer::with_default_capacity();
        assert_eq!(buf.capacity(), 720);
    }

    #[test]
    #[should_panic(expected = "ring buffer capacity must be > 0")]
    fn zero_capacity_panics() {
        MetricRingBuffer::new(0);
    }
}
