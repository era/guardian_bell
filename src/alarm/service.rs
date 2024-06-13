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
            let serialized = serde_json::to_vec(&metric)?;
            self.wal.write(&serialized)?;
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
        // start from page 0, offset 0 and keep walking until
        // we reach the end
        todo!()
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
