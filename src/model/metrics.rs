use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Metrics is based on [OpenTelemetry](https://github.com/open-telemetry/opentelemetry-proto/blob/v0.9.0/opentelemetry/proto/metrics/v1/metrics.proto#L141)
#[derive(Serialize, Deserialize, Debug)]
pub struct Metric {
    /// name of the metric, including its DNS name prefix. It must be unique
    name: String,
    /// unit in which the metric value is reported. Follows the format
    /// described by http://unitsofmeasure.org/ucum.html
    unit: String,
    /// Data determines the aggregation type (if any) of the metric, what is the
    /// reported value type for the data points, as well as the relatationship to
    /// the time interval over which they are reported.
    data: MetricData,
    /// The set of key/value pairs that uniquely identify the timeseries from
    /// where this point belongs. The list may be empty (may contain 0 elements).
    attributes: HashMap<String, String>,
}

/// # Time
///
/// This field is required, having consistent interpretation across
/// DataPoint types.  Time is the moment corresponding to when
/// the data point's aggregate value was captured.
///
/// Data points with the 0 value for Time SHOULD be rejected
/// by consumers.
///
/// # StartTime
///
/// StartTimein general allows detecting when a sequence of
/// observations is unbroken.  This field indicates to consumers the
/// start time for points with cumulative and delta
/// AggregationTemporality, and it should be included whenever possible
/// to support correct rate calculation.  Although it may be omitted
/// when the start time is truly unknown, setting StartTime is
/// strongly encouraged.
#[derive(Serialize, Deserialize, Debug)]
pub enum MetricData {
    /// The last bool If "true" means that the sum is monotonic.
    Sum(DataPoint, AggregationTemporality, bool),
    Gauge(DataPoint),
    Histogram(HistogramDataPoint, AggregationTemporality),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DataPoint {
    start_time: u64,
    time: u64,
    value: f64,
}
/// HistogramDataPoint is a single data point in a timeseries that describes the
/// time-varying values of a Histogram of double values. A Histogram contains
/// summary statistics for a population of values, it may optionally contain the
/// distribution of those values across a set of buckets.
///
/// If the histogram contains the distribution of values, then both
/// "explicit_bounds" and "bucket counts" fields must be defined.
/// If the histogram does not contain the distribution of values, then both
/// "explicit_bounds" and "bucket_counts" must be omitted and only "count" and
/// "sum" are known.
#[derive(Serialize, Deserialize, Debug)]
pub struct HistogramDataPoint {
    start_time: u64,
    time: u64,
    /// count is the number of values in the population. Must be non-negative. This
    /// value must be equal to the sum of the "count" fields in buckets if a
    /// histogram is provided.
    count: u64,
    /// sum of the values in the population. If count is zero then this field
    /// must be zero. This value must be equal to the sum of the "sum" fields in
    /// buckets if a histogram is provided.
    sum: f64,

    /// bucket_counts is an optional field contains the count values of histogram
    /// for each bucket.
    ///
    /// The sum of the bucket_counts must equal the value in the count field.
    bucket_counts: Box<[u64]>,
    /// explicit_bounds specifies buckets with explicitly defined bounds for values.
    ///
    /// This defines size(explicit_bounds) + 1 (= N) buckets. The boundaries for
    /// bucket at index i are:
    ///
    /// (-infinity, explicit_bounds[i]] for i == 0
    /// (explicit_bounds[i-1], explicit_bounds[i]] for 0 < i < N-1
    /// (explicit_bounds[i], +infinity) for i == N-1
    ///
    /// The values in the explicit_bounds array must be strictly increasing.
    ///
    /// Histogram buckets are inclusive of their upper boundary, except the last
    /// bucket where the boundary is at infinity. This format is intentionally
    /// compatible with the OpenMetrics histogram definition.
    explicity_bouds: Box<[f64]>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum AggregationTemporality {
    None,
    /// DELTA is an AggregationTemporality for a metric aggregator which reports
    /// changes since last report time. Successive metrics contain aggregation of
    /// values from continuous and non-overlapping intervals.
    ///
    /// The values for a DELTA metric are based only on the time interval
    /// associated with one measurement cycle. There is no dependency on
    /// previous measurements like is the case for CUMULATIVE metrics.
    ///
    /// For example, consider a system measuring the number of requests that
    /// it receives and reports the sum of these requests every second as a
    /// DELTA metric:
    ///
    ///   1. The system starts receiving at time=t_0.
    ///   2. A request is received, the system measures 1 request.
    ///   3. A request is received, the system measures 1 request.
    ///   4. A request is received, the system measures 1 request.
    ///   5. The 1 second collection cycle ends. A metric is exported for the
    ///      number of requests received over the interval of time t_0 to
    ///      t_0+1 with a value of 3.
    ///   6. A request is received, the system measures 1 request.
    ///   7. A request is received, the system measures 1 request.
    ///   8. The 1 second collection cycle ends. A metric is exported for the
    ///      number of requests received over the interval of time t_0+1 to
    ///      t_0+2 with a value of 2.
    Delta,
    /// CUMULATIVE is an AggregationTemporality for a metric aggregator which
    /// reports changes since a fixed start time. This means that current values
    /// of a CUMULATIVE metric depend on all previous measurements since the
    /// start time. Because of this, the sender is required to retain this state
    /// in some form. If this state is lost or invalidated, the CUMULATIVE metric
    /// values MUST be reset and a new fixed start time following the last
    /// reported measurement time sent MUST be used.
    ///
    /// For example, consider a system measuring the number of requests that
    /// it receives and reports the sum of these requests every second as a
    /// CUMULATIVE metric:
    ///
    ///   1. The system starts receiving at time=t_0.
    ///   2. A request is received, the system measures 1 request.
    ///   3. A request is received, the system measures 1 request.
    ///   4. A request is received, the system measures 1 request.
    ///   5. The 1 second collection cycle ends. A metric is exported for the
    ///      number of requests received over the interval of time t_0 to
    ///      t_0+1 with a value of 3.
    ///   6. A request is received, the system measures 1 request.
    ///   7. A request is received, the system measures 1 request.
    ///   8. The 1 second collection cycle ends. A metric is exported for the
    ///      number of requests received over the interval of time t_0 to
    ///      t_0+2 with a value of 5.
    ///   9. The system experiences a fault and loses state.
    ///   10. The system recovers and resumes receiving at time=t_1.
    ///   11. A request is received, the system measures 1 request.
    ///   12. The 1 second collection cycle ends. A metric is exported for the
    ///      number of requests received over the interval of time t_1 to
    ///      t_0+1 with a value of 1.
    ///
    /// Note: Even though, when reporting changes since last report time, using
    /// CUMULATIVE is valid, it is not recommended. This may cause problems for
    /// systems that do not use start_time to determine when the aggregation
    /// value was reset (e.g. Prometheus).
    Cumulative,
}
