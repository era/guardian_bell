use crate::alarm::alarm::Alarm;
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

#[derive(Clone)]
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
    fn consume(&mut self, metric: metrics::Metric, recover_mode: bool) -> Result<(), Error> {
        let mut should_save_in_wal = false;
        for (_, alarm) in &mut self.alarms {
            should_save_in_wal = alarm.consume(&metric) || should_save_in_wal
        }
        if should_save_in_wal && !recover_mode {
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
        if self.wal.is_empty_wal() {
            return Ok(());
        }
        let mut page = 0;
        let mut offset = 0;
        let mut size = [0 as u8; 8];
        loop {
            let read = match self.wal.read(page, offset, &mut size) {
                Ok(read) => read,
                Err(WALError::PageIndexOutOfRange) => break,
                Err(e) => return Err(Error::WALError(e)),
            };

            if read == 0 {
                page += 1;
                offset = 0;
                continue;
            } else {
                // should always be 8
                offset += read as u64;
            }

            let size = usize::from_ne_bytes(size);

            let mut entry = Vec::with_capacity(size);

            //FIXME work around, check if there is a better way
            for _i in 0..size {
                entry.push(0);
            }

            let read = self.wal.read(page, offset, &mut entry)?;
            offset += read as u64;
            let metric: metrics::Metric = serde_json::from_str(std::str::from_utf8(&entry)?)?;

            let _ = self.consume(metric, true)?;
        }

        Ok(())
    }
}

/*
 * Alarm should look something like
 *alarm {Mutex<is_alarming>, Mutex<(MetricData, CurrAlarmValue)>}
 */

#[cfg(test)]
mod test {
    use super::*;
    use std::sync::Mutex;
    use temp_dir::TempDir;

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

        fn metrics(&self) -> Vec<metrics::Metric> {
            self.metrics.lock().unwrap().clone()
        }
    }

    fn fake_metric() -> metrics::Metric {
        metrics::Metric {
            name: "MetricName".to_string(),
            unit: "rqs".to_string(),
            data: metrics::MetricData::Gauge(metrics::DataPoint {
                start_time: 0,
                time: 0,
                value: 0.1,
            }),
            attributes: HashMap::new(),
            time: 0,
        }
    }

    #[test]
    fn alarm_service_correctly_handles_consume() {
        // for this to be true it must call the alarms to consume the data
        // and if consumed it should write to the WAL.
        // after dropping and restarting the service we should get back the same state as before
        let path = TempDir::new().unwrap();
        let config = Config {
            max_size_per_page_wal: 500,
            storage_path: path.path().to_owned(),
        };
        let mut alarm_service = AlarmService::new(
            config.clone(),
            vec![Box::new(ConsumeAllMetricsAlarm {
                metrics: Mutex::new(vec![]),
            })],
        )
        .unwrap();

        let number_of_metrics = 3;

        for i in 0..number_of_metrics {
            let _ = alarm_service.consume(fake_metric(), false);
        }
        assert_eq!(
            3,
            alarm_service
                .alarms
                .get("AlarmForTest")
                .unwrap()
                .metrics()
                .len()
        );
        drop(alarm_service);

        let mut alarm_service = AlarmService::new(
            config.clone(),
            vec![Box::new(ConsumeAllMetricsAlarm {
                metrics: Mutex::new(vec![]),
            })],
        )
        .unwrap();

        assert_eq!(
            3,
            alarm_service
                .alarms
                .get("AlarmForTest")
                .unwrap()
                .metrics()
                .len()
        );
    }
}
