use crate::model::{alarm::AlarmConfig, metrics};
use chrono::{DateTime, TimeDelta, Utc};
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
        let oldest_possible_metric =
            Utc::now().checked_sub_signed(TimeDelta::minutes(self.config.time_window));
        let mut metrics = self.metrics.lock().unwrap();
        let mut max = -1.0;

        // remove entries that are not relevant for our alarm
        metrics.retain(|&k, _| DateTime::from_timestamp_millis(k as i64) > oldest_possible_metric);

        for (_, value) in metrics.iter() {
            match &value.data {
                metrics::MetricData::Gauge(data) => max = f64::max(max, data.value),
                _ => todo!(),
            }
        }
        //TODO compare the max with the current limit and alarm if needed
    }

    fn identifier(&self) -> String {
        self.id.clone()
    }

    fn metrics(&self) -> Vec<metrics::Metric> {
        todo!()
    }
}
