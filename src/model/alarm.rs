use crate::model::metrics::Metric;

pub struct AlarmConfig {
    matchers: Box<[Match]>,
    agg: Aggregation,
    value: f64,
    // size of the window in miliseconds
    time_window: f64,
}

pub struct Match {
    attribute: String,
    match_type: MatchType,
    value: String,
}

pub enum MatchType {
    Eq,
    NotEq,
}

pub enum Aggregation {
    Avg,
    Max,
    Min,
}

impl AlarmConfig {
    pub fn metric_matches(&self, metric: &Metric) -> bool {
        todo!()
    }
}
