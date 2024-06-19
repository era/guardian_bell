use crate::model::metrics;
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

pub enum Comp {
    GreaterOrEqual,
    Greater,
    LessOrEqual,
    Less,
}

pub struct MaxAlarm {
    id: String,
    limit: f64,
    comp: Comp,
    metrics: Mutex<Vec<metrics::Metric>>,
    is_alarming: Mutex<bool>,
    time_window_duration: i64,
}
