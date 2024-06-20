use crate::model::metrics::Metric;

pub struct AlarmConfig {
    pub matchers: Box<[Match]>,
    pub agg: Aggregation,
    pub value: f64,
    // size of the window in minutes
    pub time_window: i64,
}

pub struct Match {
    pub attribute: String,
    pub match_type: MatchType,
    pub value: String,
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
