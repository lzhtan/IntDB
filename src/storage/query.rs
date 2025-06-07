use chrono::{DateTime, Utc};
use crate::models::{Flow, NetworkPath};

/// Query builder for IntDB
#[derive(Debug, Clone)]
pub struct QueryBuilder {
    /// Path-based conditions
    path_conditions: Vec<PathCondition>,
    
    /// Time-based conditions
    time_conditions: Vec<TimeCondition>,
    
    /// Metric-based conditions
    metric_conditions: Vec<MetricCondition>,
    
    /// Limit on number of results
    limit: Option<usize>,
    
    /// Skip some results (for pagination)
    skip: Option<usize>,
}

#[derive(Debug, Clone)]
pub enum PathCondition {
    /// Exact path match
    ExactPath(NetworkPath),
    
    /// Path contains specific switches in order
    ContainsPath(Vec<String>),
    
    /// Path starts with given prefix
    StartsWith(Vec<String>),
    
    /// Path ends with given suffix
    EndsWith(Vec<String>),
    
    /// Flow passes through specific switch
    ThroughSwitch(String),
    
    /// Path length equals
    LengthEquals(usize),
    
    /// Path length is in range
    LengthInRange(usize, usize),
}

#[derive(Debug, Clone)]
pub enum TimeCondition {
    /// Flows after specific time
    After(DateTime<Utc>),
    
    /// Flows before specific time
    Before(DateTime<Utc>),
    
    /// Flows in time range
    InRange(DateTime<Utc>, DateTime<Utc>),
    
    /// Flows within last N seconds
    WithinLast(i64),
    
    /// Flows within last N minutes
    WithinLastMinutes(i64),
    
    /// Flows within last N hours
    WithinLastHours(i64),
}

#[derive(Debug, Clone)]
pub enum MetricCondition {
    /// Total delay greater than
    TotalDelayGreaterThan(u64),
    
    /// Total delay less than
    TotalDelayLessThan(u64),
    
    /// Total delay in range
    TotalDelayInRange(u64, u64),
    
    /// Max queue utilization greater than
    MaxQueueUtilGreaterThan(f64),
    
    /// Max queue utilization less than
    MaxQueueUtilLessThan(f64),
    
    /// Average queue utilization greater than
    AvgQueueUtilGreaterThan(f64),
    
    /// Duration greater than (in milliseconds)
    DurationGreaterThan(i64),
    
    /// Duration less than (in milliseconds)
    DurationLessThan(i64),
}

impl QueryBuilder {
    /// Create a new query builder
    pub fn new() -> Self {
        Self {
            path_conditions: Vec::new(),
            time_conditions: Vec::new(),
            metric_conditions: Vec::new(),
            limit: None,
            skip: None,
        }
    }
    
    /// Add a path condition
    pub fn with_path_condition(mut self, condition: PathCondition) -> Self {
        self.path_conditions.push(condition);
        self
    }
    
    /// Add a time condition
    pub fn with_time_condition(mut self, condition: TimeCondition) -> Self {
        self.time_conditions.push(condition);
        self
    }
    
    /// Add a metric condition
    pub fn with_metric_condition(mut self, condition: MetricCondition) -> Self {
        self.metric_conditions.push(condition);
        self
    }
    
    /// Set limit on results
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }
    
    /// Set number of results to skip
    pub fn skip(mut self, skip: usize) -> Self {
        self.skip = Some(skip);
        self
    }
    
    /// Convenience method: find flows with exact path
    pub fn exact_path(path: NetworkPath) -> Self {
        Self::new().with_path_condition(PathCondition::ExactPath(path))
    }
    
    /// Convenience method: find flows through specific switch
    pub fn through_switch(switch_id: &str) -> Self {
        Self::new().with_path_condition(PathCondition::ThroughSwitch(switch_id.to_string()))
    }
    
    /// Convenience method: find flows in time range
    pub fn in_time_range(start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        Self::new().with_time_condition(TimeCondition::InRange(start, end))
    }
    
    /// Convenience method: find flows in last N minutes
    pub fn in_last_minutes(minutes: i64) -> Self {
        Self::new().with_time_condition(TimeCondition::WithinLastMinutes(minutes))
    }
    
    /// Convenience method: find flows with high delay
    pub fn with_high_delay(threshold_ns: u64) -> Self {
        Self::new().with_metric_condition(MetricCondition::TotalDelayGreaterThan(threshold_ns))
    }
    
    /// Get all conditions for inspection
    pub fn conditions(&self) -> (&[PathCondition], &[TimeCondition], &[MetricCondition]) {
        (&self.path_conditions, &self.time_conditions, &self.metric_conditions)
    }
    
    /// Get pagination settings
    pub fn pagination(&self) -> (Option<usize>, Option<usize>) {
        (self.limit, self.skip)
    }
}

impl Default for QueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Query execution result
#[derive(Debug, Clone)]
pub struct QueryResult {
    /// Matching flow IDs
    pub flow_ids: Vec<String>,
    
    /// Total number of matching flows (before limit/skip)
    pub total_count: usize,
    
    /// Applied limit
    pub limit: Option<usize>,
}

impl QueryResult {
    /// Create a new query result
    pub fn new(flow_ids: Vec<String>, total_count: usize, limit: Option<usize>) -> Self {
        Self {
            flow_ids,
            total_count,
            limit,
        }
    }
    
    /// Check if result is empty
    pub fn is_empty(&self) -> bool {
        self.flow_ids.is_empty()
    }
    
    /// Get number of returned flows
    pub fn count(&self) -> usize {
        self.flow_ids.len()
    }
}

/// Helper functions for query condition evaluation
impl PathCondition {
    /// Check if a flow matches this path condition
    pub fn matches(&self, flow: &Flow) -> bool {
        match self {
            PathCondition::ExactPath(path) => flow.path.hash() == path.hash(),
            PathCondition::ContainsPath(subpath) => flow.path.contains_subpath(subpath),
            PathCondition::StartsWith(prefix) => flow.path.starts_with(prefix),
            PathCondition::EndsWith(suffix) => flow.path.ends_with(suffix),
            PathCondition::ThroughSwitch(switch_id) => flow.contains_switch(switch_id),
            PathCondition::LengthEquals(length) => flow.path_length() == *length,
            PathCondition::LengthInRange(min, max) => {
                let len = flow.path_length();
                len >= *min && len <= *max
            }
        }
    }
}

impl TimeCondition {
    /// Check if a flow matches this time condition
    pub fn matches(&self, flow: &Flow) -> bool {
        let now = Utc::now();
        match self {
            TimeCondition::After(time) => flow.start_time >= *time,
            TimeCondition::Before(time) => flow.start_time <= *time,
            TimeCondition::InRange(start, end) => flow.start_time >= *start && flow.start_time <= *end,
            TimeCondition::WithinLast(seconds) => {
                flow.start_time >= now - chrono::Duration::seconds(*seconds)
            }
            TimeCondition::WithinLastMinutes(minutes) => {
                flow.start_time >= now - chrono::Duration::minutes(*minutes)
            }
            TimeCondition::WithinLastHours(hours) => {
                flow.start_time >= now - chrono::Duration::hours(*hours)
            }
        }
    }
}

impl MetricCondition {
    /// Check if a flow matches this metric condition
    pub fn matches(&self, flow: &Flow) -> bool {
        match self {
            MetricCondition::TotalDelayGreaterThan(threshold) => {
                flow.total_delay().map_or(false, |delay| delay > *threshold)
            }
            MetricCondition::TotalDelayLessThan(threshold) => {
                flow.total_delay().map_or(false, |delay| delay < *threshold)
            }
            MetricCondition::TotalDelayInRange(min, max) => {
                flow.total_delay().map_or(false, |delay| delay >= *min && delay <= *max)
            }
            MetricCondition::MaxQueueUtilGreaterThan(threshold) => {
                flow.max_queue_utilization().map_or(false, |util| util > *threshold)
            }
            MetricCondition::MaxQueueUtilLessThan(threshold) => {
                flow.max_queue_utilization().map_or(false, |util| util < *threshold)
            }
            MetricCondition::AvgQueueUtilGreaterThan(threshold) => {
                flow.avg_queue_utilization().map_or(false, |util| util > *threshold)
            }
            MetricCondition::DurationGreaterThan(threshold) => {
                flow.duration_ms() > *threshold
            }
            MetricCondition::DurationLessThan(threshold) => {
                flow.duration_ms() < *threshold
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Hop, TelemetryMetrics};
    use chrono::Utc;

    fn create_test_flow(flow_id: &str, switches: &[&str], start_time: DateTime<Utc>) -> Flow {
        let hops: Vec<Hop> = switches
            .iter()
            .enumerate()
            .map(|(i, switch)| {
                Hop::new(
                    i as u32,
                    switch.to_string(),
                    start_time + chrono::Duration::milliseconds(i as i64 * 10),
                    TelemetryMetrics::with_basic(0.1 * i as f64, 100 * i as u64),
                )
            })
            .collect();
        
        Flow::new(flow_id.to_string(), hops).unwrap()
    }

    #[test]
    fn test_path_conditions() {
        let now = Utc::now();
        let flow = create_test_flow("flow1", &["s1", "s2", "s3"], now);
        
        // Test exact path
        let exact_condition = PathCondition::ExactPath(flow.path.clone());
        assert!(exact_condition.matches(&flow));
        
        // Test contains path
        let contains_condition = PathCondition::ContainsPath(vec!["s2".to_string(), "s3".to_string()]);
        assert!(contains_condition.matches(&flow));
        
        // Test starts with
        let starts_condition = PathCondition::StartsWith(vec!["s1".to_string(), "s2".to_string()]);
        assert!(starts_condition.matches(&flow));
        
        // Test through switch
        let through_condition = PathCondition::ThroughSwitch("s2".to_string());
        assert!(through_condition.matches(&flow));
        
        // Test length
        let length_condition = PathCondition::LengthEquals(3);
        assert!(length_condition.matches(&flow));
        
        let length_range_condition = PathCondition::LengthInRange(2, 4);
        assert!(length_range_condition.matches(&flow));
    }

    #[test]
    fn test_time_conditions() {
        let base_time = DateTime::from_timestamp(1640995200, 0).unwrap(); // Fixed time for testing
        let flow = create_test_flow("flow1", &["s1", "s2"], base_time);
        
        // Test after condition
        let after_condition = TimeCondition::After(base_time - chrono::Duration::minutes(1));
        assert!(after_condition.matches(&flow));
        
        let after_condition_false = TimeCondition::After(base_time + chrono::Duration::minutes(1));
        assert!(!after_condition_false.matches(&flow));
        
        // Test range condition
        let range_condition = TimeCondition::InRange(
            base_time - chrono::Duration::minutes(1),
            base_time + chrono::Duration::minutes(1),
        );
        assert!(range_condition.matches(&flow));
    }

    #[test]
    fn test_metric_conditions() {
        let now = Utc::now();
        let flow = create_test_flow("flow1", &["s1", "s2", "s3"], now);
        
        // Flow has total delay of 300 (100 + 200 + 300)
        let delay_condition = MetricCondition::TotalDelayGreaterThan(200);
        assert!(delay_condition.matches(&flow));
        
        let delay_condition_false = MetricCondition::TotalDelayGreaterThan(400);
        assert!(!delay_condition_false.matches(&flow));
        
        // Test delay range
        let delay_range_condition = MetricCondition::TotalDelayInRange(250, 350);
        assert!(delay_range_condition.matches(&flow));
        
        // Test queue utilization (max should be 0.2)
        let queue_condition = MetricCondition::MaxQueueUtilGreaterThan(0.1);
        assert!(queue_condition.matches(&flow));
    }

    #[test]
    fn test_query_builder() {
        let path = NetworkPath::from_switches(&["s1", "s2", "s3"]);
        
        let query = QueryBuilder::new()
            .with_path_condition(PathCondition::ExactPath(path))
            .with_time_condition(TimeCondition::WithinLastMinutes(60))
            .with_metric_condition(MetricCondition::TotalDelayGreaterThan(100))
            .limit(10)
            .skip(5);
        
        let (path_conds, time_conds, metric_conds) = query.conditions();
        assert_eq!(path_conds.len(), 1);
        assert_eq!(time_conds.len(), 1);
        assert_eq!(metric_conds.len(), 1);
        
        let (limit, skip) = query.pagination();
        assert_eq!(limit, Some(10));
        assert_eq!(skip, Some(5));
    }

    #[test]
    fn test_convenience_methods() {
        let path = NetworkPath::from_switches(&["s1", "s2"]);
        
        let query1 = QueryBuilder::exact_path(path);
        assert_eq!(query1.conditions().0.len(), 1);
        
        let query2 = QueryBuilder::through_switch("s1");
        assert_eq!(query2.conditions().0.len(), 1);
        
        let query3 = QueryBuilder::in_last_minutes(30);
        assert_eq!(query3.conditions().1.len(), 1);
        
        let query4 = QueryBuilder::with_high_delay(1000);
        assert_eq!(query4.conditions().2.len(), 1);
    }

    #[test]
    fn test_query_result() {
        let flow_ids = vec!["flow1".to_string(), "flow2".to_string()];
        let result = QueryResult::new(flow_ids.clone(), 5, Some(2));
        
        assert_eq!(result.count(), 2);
        assert_eq!(result.total_count, 5);
        assert!(result.has_more);
        assert!(!result.is_empty());
        
        let result_no_limit = QueryResult::new(flow_ids, 2, None);
        assert!(!result_no_limit.has_more);
    }
} 