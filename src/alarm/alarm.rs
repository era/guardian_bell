use crate::model::{
    alarm::Aggregation, alarm::CombinationAlarmConfig, alarm::LogicalOperator, alarm::Matcher,
    alarm::TagBasedAlarmConfig, alarm::ThresholdType, metrics,
};
use chrono::{DateTime, TimeDelta, Utc};
use std::collections::BTreeMap;
use std::sync::{atomic::AtomicBool, atomic::Ordering, Mutex};

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

pub trait Notifier {
    fn notify(&self, description: String);
}

pub struct NoOpNotifier {}
impl Notifier for NoOpNotifier {
    fn notify(&self, description: String) {
        // no_op
    }
}

/// Allow us to configure the alarms as `A or B`.
pub type CombinationAlarmLogicalOperator = LogicalOperator<CombinationAlarm>;

// uses other alarms as base
// alarm if A and B are alarming
pub struct CombinationAlarm {
    id: String,
    data_point_alarm: DataPointAlarm,
}

pub struct DataPointAlarm {
    id: String,
    config: TagBasedAlarmConfig,
    //btreemap of time(round by minute), (quantity_of_metrics, aggregated_value)
    metrics: Mutex<BTreeMap<u64, (u64, f64)>>,
    is_alarming: AtomicBool,
    notifier: Box<dyn Notifier>,
}

impl Alarm for DataPointAlarm {
    fn consume(&self, metric: &metrics::Metric) -> bool {
        if self.config.metric_matches(metric) {
            let value = match &metric.data {
                metrics::MetricData::Gauge(data) => data.value,
                _ => todo!(),
            };
            self.metrics
                .lock()
                // TODO make sure time is always round to the minute
                .unwrap()
                .entry(metric.time)
                .and_modify(|v| {
                    v.1 = match self.config.agg {
                        Aggregation::Max => f64::max(v.1, value),
                        Aggregation::Min => f64::min(v.1, value),
                        Aggregation::Avg => value + v.1,
                    };
                    v.0 += 1;
                })
                .or_insert((1, value));
            true
        } else {
            false
        }
    }

    //TODO we need a configuration for the alarm to become green
    // because we may want to alarm with 5 data points, and only mark
    // green after 10 data points
    fn tick(&self) {
        let oldest_possible_metric =
            Utc::now().checked_sub_signed(TimeDelta::minutes(self.config.time_window));
        let mut metrics = self.metrics.lock().unwrap();
        let mut should_alarm = true;

        // remove entries that are not relevant for our alarm
        metrics.retain(|&k, _| DateTime::from_timestamp_millis(k as i64) > oldest_possible_metric);

        for (_, datapoint) in metrics.iter() {
            let alarm_val = match self.config.agg {
                Aggregation::Avg => datapoint.1 / datapoint.0 as f64,
                _ => datapoint.1,
            };

            // we should only alarm if all data points within
            // this time window are infringing the threshold.
            should_alarm &= match self.config.value_comp {
                ThresholdType::Eq => alarm_val == self.config.value,
                ThresholdType::NotEq => alarm_val != self.config.value,
                ThresholdType::LessThan => alarm_val < self.config.value,
                ThresholdType::GreaterThan => alarm_val > self.config.value,
            };
        }

        if should_alarm {
            // https://doc.rust-lang.org/std/sync/atomic/enum.Ordering.html
            self.is_alarming.store(true, Ordering::Relaxed);
            self.notifier.notify(
                //TODO
                "should configure how to build this string later".to_string(),
            );
        }
    }

    fn identifier(&self) -> String {
        self.id.clone()
    }

    fn metrics(&self) -> Vec<metrics::Metric> {
        todo!()
    }
}
