use crate::model::metrics;
use crate::wal::{Config as WALConfig, Error as WALError, WAL};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Error while setting up WAL {0}")]
    WALError(#[from] WALError),
    #[error("Could not serialize object {0}")]
    SerializeError(#[from] serde_json::Error),
    #[error("Invalid entry in log {0}")]
    InvalidEntryInLog(#[from] std::str::Utf8Error),
}

pub struct AlarmService {
    wal: WAL,
    alarms: HashMap<String, Box<dyn Alarm>>,
}

pub struct Config {
    pub max_size_per_page_wal: usize,
    pub storage_path: PathBuf,
}

impl AlarmService {
    fn new(config: Config, alarms: Vec<Box<dyn Alarm>>) -> Result<Self, Error> {
        let wal_config = WALConfig {
            dir: config.storage_path,
            max_size_per_page: config.max_size_per_page_wal,
        };

        let mut map = HashMap::new();

        for alarm in alarms {
            map.insert(alarm.identifier(), alarm);
        }

        let wal = WAL::new(wal_config)?;
        let mut service = Self { wal, alarms: map };

        service.recover()?;

        Ok(service)
    }

    fn add(&mut self, alarm: Box<dyn Alarm>) {
        self.alarms.insert(alarm.identifier(), alarm);
    }

    fn delete(&mut self, alarm_id: &str) -> bool {
        self.alarms.remove(alarm_id).is_some()
    }

    /// Check if the metric is needed for any alarm
    /// and if so, consumes it.
    /// Any metric that is used by an alarm is saved into our WAL,
    /// otherwise we just drop it since no one is using the data.
    fn consume(&mut self, metric: metrics::Metric) -> Result<(), Error> {
        let mut should_save_in_wal = false;
        for (_, mut alarm) in &mut self.alarms {
            should_save_in_wal = alarm.consume(&metric) || should_save_in_wal
        }
        if should_save_in_wal {
            //TODO for now using json, but in the future we should use something better
            let mut serialized = serde_json::to_vec(&metric)?;
            let mut data = vec![];
            data.extend_from_slice(&serialized.len().to_ne_bytes());
            data.append(&mut serialized);
            self.wal.write(&data)?;
        }
        Ok(())
    }

    /// checks if any alarm should alarm / disable alarm and also cleans
    /// old metrics from memory
    fn tick(&self) {
        for (_, alarm) in &self.alarms {
            alarm.tick();
        }
    }

    /// recover tries to recover the configuration and metrics
    /// from disk in case of a restart
    fn recover(&mut self) -> Result<(), Error> {
        if self.wal.last_page() == 0 {
            return Ok(());
        }
        loop {
            let mut page = 0;
            let mut offset = 0;
            let mut size = [0 as u8; 8];
            let read = self.wal.read(page, offset, &mut size)?;

            if read == 0 {
                page += 1;
                offset = 0;
                continue;
            } else {
                // should always be 8
                offset += read as u64;
            }

            if page == self.wal.last_page() {
                break;
            }

            let mut entry = Vec::with_capacity(usize::from_ne_bytes(size));
            let read = self.wal.read(page, offset, &mut entry)?;
            offset += read as u64;

            let metric: metrics::Metric = serde_json::from_str(std::str::from_utf8(&entry)?)?;

            self.consume(metric)?;
        }

        Ok(())
    }
}

trait Alarm {
    /// consume a new metric, metric: returns true if it consumed it
    fn consume(&self, metric: &metrics::Metric) -> bool;

    /// checks if should alarm / disable alarm and also cleans
    /// old metrics from memory
    fn tick(&self);

    /// returns the alarm identifier
    fn identifier(&self) -> String;
}

/*
 * Alarm should look something like
 *alarm {Mutex<is_alarming>, Mutex<(MetricData, CurrAlarmValue)>}
 */

#[cfg(test)]
mod test {
    use super::*;
    use std::sync::Mutex;

    struct ConsumeAllMetricsAlarm {
        metrics: Mutex<Vec<metrics::Metric>>,
    }

    impl Alarm for ConsumeAllMetricsAlarm {
        fn consume(&self, metric: &metrics::Metric) -> bool {
            self.metrics.lock().unwrap().push(metric.clone());
            true
        }
        fn tick(&self) {
            //no_op
        }
        fn identifier(&self) -> String {
            "AlarmForTest".to_string()
        }
    }
    #[test]
    fn alarm_service_correctly_handles_consume() {
        // for this to be true it must call the alarms to consume the data
        // and if consumed it should write to the WAL.
        // after dropping and restarting the service we should get back the same state as before


    }
}
