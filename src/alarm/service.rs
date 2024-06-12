use crate::model::metrics;
use crate::wal::{Config as WALConfig, Error as WALError, WAL};
use std::path::PathBuf;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Error while setting up WAL {0}")]
    WALError(#[from] WALError),
}

pub struct AlarmService {
    wal: WAL,
    alarms: Vec<Box<dyn Alarm>>,
}

pub struct Config {
    pub max_size_per_page_wal: usize,
    pub storage_path: PathBuf,
}

impl AlarmService {
    fn new(config: Config) -> Result<Self, Error> {
        let wal_config = WALConfig {
            dir: config.storage_path,
            max_size_per_page: config.max_size_per_page_wal,
        };
        let wal = WAL::new(wal_config)?;
        Ok(Self {
            wal,
            alarms: vec![],
        })
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
