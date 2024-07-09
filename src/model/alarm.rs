use crate::model::metrics::Metric;
// TODO: this is really messy right now,
// we are at the moment forced to mirror
// the shape of this structures inside alarm service
// if we are coupled that tight we are doing something
// very wrong.

/// AlarmConfig as setup by the user.
pub enum AlarmConfig {
    /// Combination allow users to write things like:
    /// cpu.usage > 80 AND memory.usage > 80
    Combination(CombinationAlarmConfig),
    /// TagBased is the most basic type of alarm:
    /// metric_name=cpu.usage service=my_nice_service > 80
    TagBased(TagBasedAlarmConfig),
}

/// TagBasedAlarmConfig represents the configuration as setup by the user.
pub struct TagBasedAlarmConfig {
    /// Matchers for the tags: metric_name=cpu.usage; env=prod
    pub matchers: Vec<Match>,
    /// How to aggregate multiple metrics (e.g. cpu.usage for the whole fleet) for each
    /// period.
    pub agg: Aggregation,
    /// Value that we should compare each data point.
    pub value: f64,
    /// How we should compare the threshold to the data point.
    pub value_comp: ThresholdType,
    /// size of the window in minutes.
    pub time_window: i64,
    //TODO: Maybe in the future add a "number of data points before cleaning alarm"
}
/// CombinationAlarmConfig represents the configuration as setup by the user.
pub struct CombinationAlarmConfig {
    pub alarm: AlarmLogicalOperator,
    pub time_window: i64,
}

/// Represents our alarm logical operators (so that we can aggregate alarms)
pub type AlarmLogicalOperator = LogicalOperator<TagBasedAlarmConfig>;

/// Generic LogicalOperator
pub enum LogicalOperator<I> {
    /// The item itself
    Identity(Box<I>),
    /// And logical operator
    And(Box<AlarmLogicalOperator>, Box<AlarmLogicalOperator>),
    /// Or logical Operator
    Or(Box<AlarmLogicalOperator>, Box<AlarmLogicalOperator>),
    /// Not logical Operator
    Not(Box<AlarmLogicalOperator>),
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

pub trait Matcher {
    fn metric_matches(&self, metric: &Metric) -> bool;
}

impl Matcher for AlarmLogicalOperator {
    fn metric_matches(&self, metric: &Metric) -> bool {
        todo!()
    }
}
impl Matcher for TagBasedAlarmConfig {
    fn metric_matches(&self, metric: &Metric) -> bool {
        todo!()
    }
}
