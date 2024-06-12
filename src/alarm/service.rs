use crate::model::metrics;
use crate::wal::{Config as WALConfig, Error as WALError, WAL};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Error while setting up WAL {0}")]
    WALError(#[from] WALError),
}

pub struct AlarmService {
    wal: WAL,
    alarms: BTreeMap<String, Box<dyn Alarm>>,
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

        let mut btree = BTreeMap::new();

        for alarm in alarms {
            btree.insert(alarm.identifier(), alarm);
        }

        let wal = WAL::new(wal_config)?;
        Ok(Self { wal, alarms: btree })
    }

    fn add(&mut self, alarm: Box<dyn Alarm>) {
        // check if should update instead
        self.alarms.insert(alarm.identifier(), alarm);
    }

    fn delete(&mut self, alarm_id: &str) {
        todo!()
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
            // FIXME: handle errors
            let serialized = serde_json::to_vec(&metric).unwrap();
            self.wal.write(&serialized)?;
        }
        Ok(())
    }

    /// recover tries to recover the configuration and metrics
    /// from disk in case of a restart
    fn recover(&mut self) -> Result<(), Error> {
        todo!()
    }
}

trait Alarm {
    /// consume a new metric, metric: returns true if it consumed it
    fn consume(&self, metric: &metrics::Metric) -> bool;
    /// checks if should alarm / disable alarm and also cleans
    /// old metrics from memory
    fn tick(&self);

    // returns the alarm identifier
    fn identifier(&self) -> String;
}

/*
 * Alarm should look something like
 *alarm {Mutex<is_alarming>, Mutex<(MetricData, CurrAlarmValue)>}
 */
