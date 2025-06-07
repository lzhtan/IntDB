use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::models::TelemetryMetrics;

/// A single hop in a network path with telemetry data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Hop {
    /// Index of this hop in the path (0-based)
    pub hop_index: u32,
    
    /// Switch/router identifier
    pub switch_id: String,
    
    /// Timestamp when packet was processed at this hop
    pub timestamp: DateTime<Utc>,
    
    /// Telemetry metrics collected at this hop
    pub metrics: TelemetryMetrics,
}

impl Hop {
    /// Create a new hop
    pub fn new(
        hop_index: u32,
        switch_id: String, 
        timestamp: DateTime<Utc>,
        metrics: TelemetryMetrics,
    ) -> Self {
        Self {
            hop_index,
            switch_id,
            timestamp,
            metrics,
        }
    }
    
    /// Create a hop with basic metrics
    pub fn with_basic_metrics(
        hop_index: u32,
        switch_id: String,
        timestamp: DateTime<Utc>, 
        queue_util: f64,
        delay_ns: u64,
    ) -> Self {
        Self::new(
            hop_index,
            switch_id,
            timestamp,
            TelemetryMetrics::with_basic(queue_util, delay_ns),
        )
    }
    
    /// Get the delay at this hop, if available
    pub fn delay(&self) -> Option<u64> {
        self.metrics.delay_ns
    }
    
    /// Get the queue utilization at this hop, if available
    pub fn queue_utilization(&self) -> Option<f64> {
        self.metrics.queue_util
    }
    
    /// Check if this hop has any meaningful telemetry data
    pub fn has_telemetry(&self) -> bool {
        !self.metrics.is_empty()
    }
}

/// Input format for creating hops from JSON
#[derive(Debug, Serialize, Deserialize)]
pub struct HopInput {
    pub switch_id: String,
    pub timestamp: DateTime<Utc>,
    pub queue_util: Option<f64>,
    pub delay_ns: Option<u64>,
    pub bandwidth_bps: Option<u64>,
    pub drop_count: Option<u64>,
    pub egress_port: Option<u32>,
    pub ingress_port: Option<u32>,
}

impl From<(u32, HopInput)> for Hop {
    fn from((hop_index, input): (u32, HopInput)) -> Self {
        let metrics = TelemetryMetrics {
            queue_util: input.queue_util,
            delay_ns: input.delay_ns,
            bandwidth_bps: input.bandwidth_bps,
            drop_count: input.drop_count,
            egress_port: input.egress_port,
            ingress_port: input.ingress_port,
            custom_metrics: None,
        };
        
        Self::new(hop_index, input.switch_id, input.timestamp, metrics)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_hop_creation() {
        let now = Utc::now();
        let hop = Hop::with_basic_metrics(0, "s1".to_string(), now, 0.75, 1000);
        
        assert_eq!(hop.hop_index, 0);
        assert_eq!(hop.switch_id, "s1");
        assert_eq!(hop.timestamp, now);
        assert_eq!(hop.delay(), Some(1000));
        assert_eq!(hop.queue_utilization(), Some(0.75));
        assert!(hop.has_telemetry());
    }

    #[test]
    fn test_hop_from_input() {
        let input = HopInput {
            switch_id: "s2".to_string(),
            timestamp: Utc::now(),
            queue_util: Some(0.5),
            delay_ns: Some(500),
            bandwidth_bps: Some(1000000),
            drop_count: Some(10),
            egress_port: Some(1),
            ingress_port: Some(2),
        };

        let hop = Hop::from((1, input));
        assert_eq!(hop.hop_index, 1);
        assert_eq!(hop.switch_id, "s2");
        assert!(hop.has_telemetry());
    }

    #[test]
    fn test_hop_serialization() {
        let hop = Hop::with_basic_metrics(0, "s1".to_string(), Utc::now(), 0.75, 1000);
        let json = serde_json::to_string(&hop).unwrap();
        let deserialized: Hop = serde_json::from_str(&json).unwrap();
        assert_eq!(hop, deserialized);
    }
} 