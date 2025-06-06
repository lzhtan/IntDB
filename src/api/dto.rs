use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::models::{Flow, FlowInput};
use crate::storage::{QueryResult, PathCondition, TimeCondition, MetricCondition};

/// Flow insertion request
#[derive(Debug, Deserialize)]
pub struct InsertFlowRequest {
    pub flow: FlowInput,
}

/// Flow insertion response
#[derive(Debug, Serialize)]
pub struct InsertFlowResponse {
    pub flow_id: String,
    pub status: String,
    pub message: String,
}

/// Single flow response
#[derive(Debug, Serialize)]
pub struct FlowResponse {
    pub flow: Flow,
}

/// Multiple flows response
#[derive(Debug, Serialize)]
pub struct FlowsResponse {
    pub flows: Vec<Flow>,
    pub count: usize,
}

/// Query request for flows
#[derive(Debug, Deserialize)]
pub struct QueryRequest {
    /// Path-based conditions
    #[serde(default)]
    pub path_conditions: Vec<PathConditionDto>,
    
    /// Time-based conditions
    #[serde(default)]
    pub time_conditions: Vec<TimeConditionDto>,
    
    /// Metric-based conditions
    #[serde(default)]
    pub metric_conditions: Vec<MetricConditionDto>,
    
    /// Maximum number of results
    pub limit: Option<usize>,
    
    /// Number of results to skip
    pub skip: Option<usize>,
    
    /// Whether to include full flow data or just IDs
    #[serde(default)]
    pub include_flows: bool,
}

/// Query response
#[derive(Debug, Serialize)]
pub struct QueryResponse {
    pub flow_ids: Vec<String>,
    pub flows: Option<Vec<Flow>>,
    pub total_count: usize,
    pub has_more: bool,
    pub count: usize,
}

/// Path condition DTO
#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum PathConditionDto {
    #[serde(rename = "exact_path")]
    ExactPath { switches: Vec<String> },
    
    #[serde(rename = "contains_path")]
    ContainsPath { switches: Vec<String> },
    
    #[serde(rename = "starts_with")]
    StartsWith { switches: Vec<String> },
    
    #[serde(rename = "ends_with")]
    EndsWith { switches: Vec<String> },
    
    #[serde(rename = "through_switch")]
    ThroughSwitch { switch_id: String },
    
    #[serde(rename = "length_equals")]
    LengthEquals { length: usize },
    
    #[serde(rename = "length_range")]
    LengthInRange { min: usize, max: usize },
}

/// Time condition DTO
#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum TimeConditionDto {
    #[serde(rename = "after")]
    After { time: DateTime<Utc> },
    
    #[serde(rename = "before")]
    Before { time: DateTime<Utc> },
    
    #[serde(rename = "range")]
    InRange { start: DateTime<Utc>, end: DateTime<Utc> },
    
    #[serde(rename = "within_seconds")]
    WithinLast { seconds: i64 },
    
    #[serde(rename = "within_minutes")]
    WithinLastMinutes { minutes: i64 },
    
    #[serde(rename = "within_hours")]
    WithinLastHours { hours: i64 },
}

/// Metric condition DTO
#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum MetricConditionDto {
    #[serde(rename = "delay_gt")]
    TotalDelayGreaterThan { threshold: u64 },
    
    #[serde(rename = "delay_lt")]
    TotalDelayLessThan { threshold: u64 },
    
    #[serde(rename = "delay_range")]
    TotalDelayInRange { min: u64, max: u64 },
    
    #[serde(rename = "queue_util_gt")]
    MaxQueueUtilGreaterThan { threshold: f64 },
    
    #[serde(rename = "queue_util_lt")]
    MaxQueueUtilLessThan { threshold: f64 },
    
    #[serde(rename = "avg_queue_util_gt")]
    AvgQueueUtilGreaterThan { threshold: f64 },
    
    #[serde(rename = "duration_gt")]
    DurationGreaterThan { threshold: i64 },
    
    #[serde(rename = "duration_lt")]
    DurationLessThan { threshold: i64 },
}

/// API error response
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
    pub details: Option<String>,
}

/// Health check response
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub flow_count: usize,
}

/// Statistics response
#[derive(Debug, Serialize)]
pub struct StatsResponse {
    pub total_flows: usize,
    pub unique_paths: usize,
    pub unique_switches: usize,
    pub time_buckets: usize,
    pub memory_usage_estimate: usize,
}

/// Conversion implementations
impl From<PathConditionDto> for PathCondition {
    fn from(dto: PathConditionDto) -> Self {
        match dto {
            PathConditionDto::ExactPath { switches } => {
                PathCondition::ExactPath(crate::models::NetworkPath::new(switches))
            }
            PathConditionDto::ContainsPath { switches } => {
                PathCondition::ContainsPath(switches)
            }
            PathConditionDto::StartsWith { switches } => {
                PathCondition::StartsWith(switches)
            }
            PathConditionDto::EndsWith { switches } => {
                PathCondition::EndsWith(switches)
            }
            PathConditionDto::ThroughSwitch { switch_id } => {
                PathCondition::ThroughSwitch(switch_id)
            }
            PathConditionDto::LengthEquals { length } => {
                PathCondition::LengthEquals(length)
            }
            PathConditionDto::LengthInRange { min, max } => {
                PathCondition::LengthInRange(min, max)
            }
        }
    }
}

impl From<TimeConditionDto> for TimeCondition {
    fn from(dto: TimeConditionDto) -> Self {
        match dto {
            TimeConditionDto::After { time } => TimeCondition::After(time),
            TimeConditionDto::Before { time } => TimeCondition::Before(time),
            TimeConditionDto::InRange { start, end } => TimeCondition::InRange(start, end),
            TimeConditionDto::WithinLast { seconds } => TimeCondition::WithinLast(seconds),
            TimeConditionDto::WithinLastMinutes { minutes } => TimeCondition::WithinLastMinutes(minutes),
            TimeConditionDto::WithinLastHours { hours } => TimeCondition::WithinLastHours(hours),
        }
    }
}

impl From<MetricConditionDto> for MetricCondition {
    fn from(dto: MetricConditionDto) -> Self {
        match dto {
            MetricConditionDto::TotalDelayGreaterThan { threshold } => {
                MetricCondition::TotalDelayGreaterThan(threshold)
            }
            MetricConditionDto::TotalDelayLessThan { threshold } => {
                MetricCondition::TotalDelayLessThan(threshold)
            }
            MetricConditionDto::TotalDelayInRange { min, max } => {
                MetricCondition::TotalDelayInRange(min, max)
            }
            MetricConditionDto::MaxQueueUtilGreaterThan { threshold } => {
                MetricCondition::MaxQueueUtilGreaterThan(threshold)
            }
            MetricConditionDto::MaxQueueUtilLessThan { threshold } => {
                MetricCondition::MaxQueueUtilLessThan(threshold)
            }
            MetricConditionDto::AvgQueueUtilGreaterThan { threshold } => {
                MetricCondition::AvgQueueUtilGreaterThan(threshold)
            }
            MetricConditionDto::DurationGreaterThan { threshold } => {
                MetricCondition::DurationGreaterThan(threshold)
            }
            MetricConditionDto::DurationLessThan { threshold } => {
                MetricCondition::DurationLessThan(threshold)
            }
        }
    }
}

impl From<QueryResult> for QueryResponse {
    fn from(result: QueryResult) -> Self {
        Self {
            flow_ids: result.flow_ids.clone(),
            flows: None, // Will be populated by handler if needed
            total_count: result.total_count,
            has_more: result.has_more,
            count: result.count(),
        }
    }
} 