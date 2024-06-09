use crate::model::metrics;
use crate::wal::WAL;


pub struct AlarmService {
    // Use wal
}

impl AlarmService {

    fn new() -> Self {
        // TODO setup wal
        Self {}
    }

    /// Check if the metric is needed for any alarm
    /// and if so, consumes it
    fn consume(&self, metric: metrics::Metric) -> Result<(), ()> {
        // save into wal if needed
        // consume
        todo!()
    }

    /// recover tries to recover the configuration and metrics
    /// from disk in case of a restart
    fn recover(&mut self) -> Result<(), ()> {
        todo!()
    }

}

trait Alarm {
    /// check if should consume metric
    /// useful if you don't want to hold a &mut self just to check
    /// it
    fn metric_matches(&self, metric: metrics::Metric) -> bool;
    /// consume a new metric, metric: returns true if it consumed it
    fn consume(&mut self, metric: metrics::Metric) -> bool;
    /// checks if should alarm / disable alarm and also cleans
    /// old metrics from memory
    fn tick(&mut self);
}

