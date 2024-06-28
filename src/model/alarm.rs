use crate::model::metrics::Metric;

pub struct AlarmConfig {
    pub alarm_type: AlarmType,
}

pub struct TagBasedAlarmConfig {
    pub matchers: Vec<Match>,
    pub agg: Aggregation,
    pub value: f64,
    pub value_comp: ThresholdType,
    // size of the window in minutes
    pub time_window: i64,
}

pub struct CombinationAlarmConfig {
    // TODO time_window maybe
    pub alarms: AlarmLogicalOperator,
}

pub enum AlarmLogicalOperator {
    Identity(Box<AlarmConfig>),
    And(Box<AlarmLogicalOperator>),
    Or(Box<AlarmLogicalOperator>),
    Not(Box<AlarmLogicalOperator>),
}

pub enum AlarmType {
    Combination(CombinationAlarmConfig), //TODO
    TagBased(TagBasedAlarmConfig),
}

pub enum ThresholdType {
    Eq,
    NotEq,
    LessThan,
    GreaterThan,
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

impl TagBasedAlarmConfig {
    pub fn metric_matches(&self, metric: &Metric) -> bool {
        todo!()
    }
}
