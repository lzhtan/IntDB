use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::models::{Hop, HopInput, NetworkPath};

/// A complete network flow with path and telemetry data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Flow {
    /// Unique identifier for this flow
    pub flow_id: String,
    
    /// Network path taken by this flow
    pub path: NetworkPath,
    
    /// Ordered sequence of hops with telemetry data
    pub hops: Vec<Hop>,
    
    /// Start time of the flow (timestamp of first hop)
    pub start_time: DateTime<Utc>,
    
    /// End time of the flow (timestamp of last hop)
    pub end_time: DateTime<Utc>,
    
    /// Flow completion status
    pub status: FlowStatus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FlowStatus {
    /// Flow is complete with all expected hops
    Complete,
    /// Flow is still being assembled (missing hops)
    Partial,
    /// Flow timed out waiting for missing hops
    Timeout,
    /// Flow has errors or inconsistencies
    Error(String),
}

impl Flow {
    /// Create a new flow from hops
    pub fn new(flow_id: String, hops: Vec<Hop>) -> Result<Self, FlowError> {
        if hops.is_empty() {
            return Err(FlowError::EmptyFlow);
        }
        
        // Validate hop ordering
        for (i, hop) in hops.iter().enumerate() {
            if hop.hop_index != i as u32 {
                return Err(FlowError::InvalidHopOrdering);
            }
        }
        
        // Extract path from hops
        let switches: Vec<String> = hops.iter().map(|h| h.switch_id.clone()).collect();
        let path = NetworkPath::new(switches);
        
        // Determine time range
        let start_time = hops.first().unwrap().timestamp;
        let end_time = hops.last().unwrap().timestamp;
        
        if start_time > end_time {
            return Err(FlowError::InvalidTimeOrdering);
        }
        
        Ok(Self {
            flow_id,
            path,
            hops,
            start_time,
            end_time,
            status: FlowStatus::Complete,
        })
    }
    
    /// Create a partial flow that can be completed later
    pub fn new_partial(flow_id: String, mut hops: Vec<Hop>) -> Self {
        // Sort hops by index
        hops.sort_by_key(|h| h.hop_index);
        
        let switches: Vec<String> = hops.iter().map(|h| h.switch_id.clone()).collect();
        let path = NetworkPath::new(switches);
        
        let start_time = hops.first().map(|h| h.timestamp).unwrap_or_else(Utc::now);
        let end_time = hops.last().map(|h| h.timestamp).unwrap_or(start_time);
        
        Self {
            flow_id,
            path,
            hops,
            start_time,
            end_time,
            status: FlowStatus::Partial,
        }
    }
    
    /// Add a hop to a partial flow
    pub fn add_hop(&mut self, hop: Hop) -> Result<(), FlowError> {
        // Check if hop already exists
        if self.hops.iter().any(|h| h.hop_index == hop.hop_index) {
            return Err(FlowError::DuplicateHop);
        }
        
        self.hops.push(hop);
        self.hops.sort_by_key(|h| h.hop_index);
        
        // Rebuild path
        let switches: Vec<String> = self.hops.iter().map(|h| h.switch_id.clone()).collect();
        self.path = NetworkPath::new(switches);
        
        // Update time range
        self.start_time = self.hops.first().unwrap().timestamp;
        self.end_time = self.hops.last().unwrap().timestamp;
        
        Ok(())
    }
    
    /// Mark flow as complete
    pub fn mark_complete(&mut self) {
        self.status = FlowStatus::Complete;
    }
    
    /// Mark flow as timed out
    pub fn mark_timeout(&mut self) {
        self.status = FlowStatus::Timeout;
    }
    
    /// Get the total path length
    pub fn path_length(&self) -> usize {
        self.path.length()
    }
    
    /// Get the total end-to-end delay
    pub fn total_delay(&self) -> Option<u64> {
        let delays: Vec<u64> = self.hops.iter().filter_map(|h| h.delay()).collect();
        if delays.is_empty() {
            None
        } else {
            Some(delays.iter().sum())
        }
    }
    
    /// Get the maximum queue utilization across all hops
    pub fn max_queue_utilization(&self) -> Option<f64> {
        self.hops.iter()
            .filter_map(|h| h.queue_utilization())
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
    }
    
    /// Get the average queue utilization across all hops
    pub fn avg_queue_utilization(&self) -> Option<f64> {
        let utils: Vec<f64> = self.hops.iter().filter_map(|h| h.queue_utilization()).collect();
        if utils.is_empty() {
            None
        } else {
            Some(utils.iter().sum::<f64>() / utils.len() as f64)
        }
    }
    
    /// Check if flow contains the given switch
    pub fn contains_switch(&self, switch_id: &str) -> bool {
        self.hops.iter().any(|h| h.switch_id == switch_id)
    }
    
    /// Get flow duration in milliseconds
    pub fn duration_ms(&self) -> i64 {
        self.end_time.timestamp_millis() - self.start_time.timestamp_millis()
    }
    
    /// Check if flow is complete
    pub fn is_complete(&self) -> bool {
        matches!(self.status, FlowStatus::Complete)
    }
    
    /// Get the path hash for indexing
    pub fn path_hash(&self) -> String {
        self.path.hash()
    }
}

/// Input format for creating flows from JSON
#[derive(Debug, Deserialize)]
pub struct FlowInput {
    pub flow_id: String,
    pub telemetry: Vec<HopInput>,
}

impl TryFrom<FlowInput> for Flow {
    type Error = FlowError;
    
    fn try_from(input: FlowInput) -> Result<Self, Self::Error> {
        if input.telemetry.is_empty() {
            return Err(FlowError::EmptyFlow);
        }
        
        let hops: Vec<Hop> = input.telemetry
            .into_iter()
            .enumerate()
            .map(|(i, hop_input)| Hop::from((i as u32, hop_input)))
            .collect();
            
        Flow::new(input.flow_id, hops)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum FlowError {
    #[error("Flow cannot be empty")]
    EmptyFlow,
    
    #[error("Invalid hop ordering")]
    InvalidHopOrdering,
    
    #[error("Invalid time ordering")]
    InvalidTimeOrdering,
    
    #[error("Duplicate hop index")]
    DuplicateHop,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_hops() -> Vec<Hop> {
        let now = Utc::now();
        vec![
            Hop::with_basic_metrics(0, "s1".to_string(), now, 0.1, 100),
            Hop::with_basic_metrics(1, "s2".to_string(), now + chrono::Duration::milliseconds(10), 0.2, 200),
            Hop::with_basic_metrics(2, "s3".to_string(), now + chrono::Duration::milliseconds(20), 0.3, 300),
        ]
    }

    #[test]
    fn test_flow_creation() {
        let hops = create_test_hops();
        let flow = Flow::new("flow1".to_string(), hops.clone()).unwrap();
        
        assert_eq!(flow.flow_id, "flow1");
        assert_eq!(flow.path_length(), 3);
        assert_eq!(flow.hops.len(), 3);
        assert!(flow.is_complete());
        assert_eq!(flow.total_delay(), Some(600));
    }

    #[test]
    fn test_flow_statistics() {
        let hops = create_test_hops();
        let flow = Flow::new("flow1".to_string(), hops).unwrap();
        
        assert_eq!(flow.max_queue_utilization(), Some(0.3));
        assert!((flow.avg_queue_utilization().unwrap() - 0.2).abs() < 0.001);
        assert!(flow.contains_switch("s2"));
        assert!(!flow.contains_switch("s4"));
    }

    #[test]
    fn test_partial_flow() {
        let hops = vec![
            Hop::with_basic_metrics(0, "s1".to_string(), Utc::now(), 0.1, 100),
            Hop::with_basic_metrics(2, "s3".to_string(), Utc::now(), 0.3, 300),
        ];
        
        let mut flow = Flow::new_partial("flow1".to_string(), hops);
        assert!(!flow.is_complete());
        
        let missing_hop = Hop::with_basic_metrics(1, "s2".to_string(), Utc::now(), 0.2, 200);
        flow.add_hop(missing_hop).unwrap();
        
        assert_eq!(flow.path_length(), 3);
        assert_eq!(flow.hops[1].switch_id, "s2"); // Should be sorted correctly
    }

    #[test]
    fn test_flow_from_input() {
        let input = FlowInput {
            flow_id: "flow1".to_string(),
            telemetry: vec![
                HopInput {
                    switch_id: "s1".to_string(),
                    timestamp: Utc::now(),
                    queue_util: Some(0.1),
                    delay_ns: Some(100),
                    bandwidth_bps: None,
                    drop_count: None,
                    egress_port: None,
                    ingress_port: None,
                },
                HopInput {
                    switch_id: "s2".to_string(),
                    timestamp: Utc::now(),
                    queue_util: Some(0.2),
                    delay_ns: Some(200),
                    bandwidth_bps: None,
                    drop_count: None,
                    egress_port: None,
                    ingress_port: None,
                },
            ],
        };
        
        let flow = Flow::try_from(input).unwrap();
        assert_eq!(flow.flow_id, "flow1");
        assert_eq!(flow.path_length(), 2);
    }

    #[test]
    fn test_flow_errors() {
        // Empty flow
        assert!(matches!(Flow::new("flow1".to_string(), vec![]), Err(FlowError::EmptyFlow)));
        
        // Invalid hop ordering
        let bad_hops = vec![
            Hop::with_basic_metrics(1, "s1".to_string(), Utc::now(), 0.1, 100),
            Hop::with_basic_metrics(0, "s2".to_string(), Utc::now(), 0.2, 200),
        ];
        assert!(matches!(Flow::new("flow1".to_string(), bad_hops), Err(FlowError::InvalidHopOrdering)));
    }
} 