//! Metrics collector service (SPEC 22 §10.1).
//!
//! Collects system metrics at a configurable interval and stores them in
//! an in-memory ring buffer. In a full deployment, periodic flush to SQLite
//! would aggregate to 1-minute intervals.

use std::sync::Arc;

use tokio::sync::RwLock;
use tokio::time::{Duration, Interval};

use crate::error::{AdminError, Result};
use crate::metrics::ring_buffer::MetricRingBuffer;
use crate::types::MetricSample;

/// Background metrics collector.
///
/// The collector runs as a `tokio` task that samples system metrics at a
/// configurable interval and pushes them into a ring buffer. In this
/// crate-level implementation we provide the data structures and a
/// **mock** sample function (since `sysinfo` is not in scope for the
/// minimal build). In production the sample function reads real system
/// counters.
pub struct MetricsCollector {
    buffer: Arc<RwLock<MetricRingBuffer>>,
    interval_secs: u64,
}

impl MetricsCollector {
    /// Create a new collector with the given sample interval in seconds.
    pub fn new(interval_secs: u64) -> Self {
        Self {
            buffer: Arc::new(RwLock::new(MetricRingBuffer::with_default_capacity())),
            interval_secs,
        }
    }

    /// Create a collector with a custom ring buffer capacity.
    pub fn with_capacity(interval_secs: u64, capacity: usize) -> Self {
        Self {
            buffer: Arc::new(RwLock::new(MetricRingBuffer::new(capacity))),
            interval_secs,
        }
    }

    /// Return the configured sample interval.
    pub fn interval(&self) -> Duration {
        Duration::from_secs(self.interval_secs)
    }

    /// Return a reference to the shared ring buffer (for reading).
    pub fn buffer(&self) -> Arc<RwLock<MetricRingBuffer>> {
        Arc::clone(&self.buffer)
    }

    /// Collect a single metric sample and push it into the ring buffer.
    ///
    /// In a full deployment this reads real CPU/memory/disk counters via
    /// `sysinfo`. For the crate-level implementation we accept a pre-built
    /// sample so that the collection plumbing is exercised independently of
    /// hardware access.
    pub async fn record(&self, sample: MetricSample) {
        let mut buf = self.buffer.write().await;
        buf.push(sample);
    }

    /// Get the latest metric sample from the ring buffer.
    pub async fn latest(&self) -> Option<MetricSample> {
        let buf = self.buffer.read().await;
        buf.latest().cloned()
    }

    /// Get metric samples within a time range.
    pub async fn range(&self, from: &str, to: &str) -> Vec<MetricSample> {
        let buf = self.buffer.read().await;
        buf.range(from, to).into_iter().cloned().collect()
    }

    /// Flush the ring buffer, draining all samples and returning them for
    /// persistence. In a full deployment these would be aggregated and
    /// written to SQLite.
    pub async fn flush(&self) -> Vec<MetricSample> {
        let mut buf = self.buffer.write().await;
        buf.drain_all().into_iter().collect()
    }

    /// Run the collector loop, calling `sampler` at each interval tick.
    /// The sampler closure returns a `MetricSample` or an error.
    ///
    /// On error the sample is skipped. After 10 consecutive failures the
    /// loop exits (SPEC 22 §12.1).
    pub async fn run<F>(&self, mut sampler: F) -> Result<()>
    where
        F: FnMut() -> std::result::Result<MetricSample, AdminError>,
    {
        let mut interval: Interval = tokio::time::interval(self.interval());
        let mut consecutive_failures: u32 = 0;

        loop {
            interval.tick().await;
            match sampler() {
                Ok(sample) => {
                    self.record(sample).await;
                    consecutive_failures = 0;
                }
                Err(e) => {
                    consecutive_failures += 1;
                    tracing::warn!(error = %e, failures = consecutive_failures, "metrics sample failed");
                    if consecutive_failures >= 10 {
                        tracing::error!(
                            "10 consecutive metric sample failures, disabling collector"
                        );
                        return Err(AdminError::MetricsError(
                            "10 consecutive failures, collector disabled".to_string(),
                        ));
                    }
                }
            }
        }
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

    #[tokio::test]
    async fn record_and_latest() {
        let collector = MetricsCollector::new(5);
        assert!(collector.latest().await.is_none());

        collector.record(sample("2026-03-30T12:00:00Z", 10.0)).await;
        collector.record(sample("2026-03-30T12:00:05Z", 20.0)).await;

        let latest = collector.latest().await.unwrap();
        assert_eq!(latest.cpu_percent, 20.0);
    }

    #[tokio::test]
    async fn range_returns_subset() {
        let collector = MetricsCollector::new(5);
        collector.record(sample("2026-03-30T12:00:00Z", 10.0)).await;
        collector.record(sample("2026-03-30T12:00:05Z", 20.0)).await;
        collector.record(sample("2026-03-30T12:00:10Z", 30.0)).await;

        let range = collector
            .range("2026-03-30T12:00:05Z", "2026-03-30T12:00:05Z")
            .await;
        assert_eq!(range.len(), 1);
        assert_eq!(range[0].cpu_percent, 20.0);
    }

    #[tokio::test]
    async fn flush_drains_buffer() {
        let collector = MetricsCollector::new(5);
        collector.record(sample("2026-03-30T12:00:00Z", 10.0)).await;
        collector.record(sample("2026-03-30T12:00:05Z", 20.0)).await;

        let flushed = collector.flush().await;
        assert_eq!(flushed.len(), 2);
        assert!(collector.latest().await.is_none());
    }

    #[tokio::test]
    async fn ring_buffer_overflow() {
        let collector = MetricsCollector::with_capacity(5, 3);
        collector.record(sample("2026-03-30T12:00:00Z", 10.0)).await;
        collector.record(sample("2026-03-30T12:00:05Z", 20.0)).await;
        collector.record(sample("2026-03-30T12:00:10Z", 30.0)).await;
        collector.record(sample("2026-03-30T12:00:15Z", 40.0)).await;

        // Oldest (10.0) should be discarded.
        let range = collector
            .range("2026-03-30T00:00:00Z", "2099-01-01T00:00:00Z")
            .await;
        assert_eq!(range.len(), 3);
        assert_eq!(range[0].cpu_percent, 20.0); // 10.0 was dropped
        assert_eq!(range[2].cpu_percent, 40.0);
    }

    #[tokio::test]
    async fn run_collector_loop() {
        let collector = MetricsCollector::with_capacity(1, 100);
        let counter = Arc::new(std::sync::atomic::AtomicU32::new(0));
        let counter_clone = Arc::clone(&counter);

        let collector_clone = Arc::new(collector);
        let cc = Arc::clone(&collector_clone);

        let handle = tokio::spawn(async move {
            cc.run(move || {
                let n = counter_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                if n >= 3 {
                    // Stop after 3 samples.
                    Err(AdminError::MetricsError("done".to_string()))
                } else {
                    Ok(sample(
                        &format!("2026-03-30T12:00:{n:02}Z"),
                        n as f32 * 10.0,
                    ))
                }
            })
            .await
        });

        // Let the collector run. It will stop after 3 successful samples
        // (4th call returns an error).
        let _ = handle.await;

        // We should have recorded 3 samples.
        let range = collector_clone
            .range("2026-03-30T00:00:00Z", "2099-01-01T00:00:00Z")
            .await;
        assert!(range.len() >= 3);
    }
}
