use crate::model::{alarm::AlarmConfig, metrics};
use std::collections::BTreeMap;
use std::sync::Mutex;

pub(crate) trait Alarm {
    /// consume a new metric, metric: returns true if it consumed it
    fn consume(&self, metric: &metrics::Metric) -> bool;

    /// checks if should alarm / disable alarm and also cleans
    /// old metrics from memory
    fn tick(&self);

    /// returns the alarm identifier
    fn identifier(&self) -> String;

    /// returns all metrics used in the alarm. Only for tests,
    /// may not be kept in production to reduce memory usage.
    fn metrics(&self) -> Vec<metrics::Metric>;
}

pub struct MaxAlarm {
    id: String,
    config: AlarmConfig,
    // later when we need to delete old entries:
    // https://stackoverflow.com/questions/35663342/how-to-modify-partially-remove-a-range-from-a-btreemap
    metrics: Mutex<BTreeMap<u64, metrics::Metric>>,
    is_alarming: Mutex<bool>,
    time_window_duration: u64,
}

impl Alarm for MaxAlarm {
    fn consume(&self, metric: &metrics::Metric) -> bool {
        if self.config.metric_matches(metric) {
            self.metrics
                .lock()
                .unwrap()
                .insert(metric.time, metric.clone());
            true
        } else {
            false
        }
    }

    fn tick(&self) {
        todo!()
    }

    fn identifier(&self) -> String {
        self.id.clone()
    }

    fn metrics(&self) -> Vec<metrics::Metric> {
        todo!()
    }
}
