use serde::{Deserialize, Serialize};

/// Network telemetry metrics collected at each hop
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TelemetryMetrics {
    /// Queue utilization (0.0 to 1.0)
    pub queue_util: Option<f64>,
    
    /// Delay in nanoseconds
    pub delay_ns: Option<u64>,
    
    /// Bandwidth utilization in bytes per second
    pub bandwidth_bps: Option<u64>,
    
    /// Packet drop count
    pub drop_count: Option<u64>,
    
    /// Egress port identifier
    pub egress_port: Option<u32>,
    
    /// Ingress port identifier  
    pub ingress_port: Option<u32>,
    
    /// Additional custom metrics
    pub custom_metrics: Option<indexmap::IndexMap<String, serde_json::Value>>,
}

impl TelemetryMetrics {
    /// Create new empty metrics
    pub fn new() -> Self {
        Self {
            queue_util: None,
            delay_ns: None,
            bandwidth_bps: None,
            drop_count: None,
            egress_port: None,
            ingress_port: None,
            custom_metrics: None,
        }
    }
    
    /// Create metrics with basic delay and queue utilization
    pub fn with_basic(queue_util: f64, delay_ns: u64) -> Self {
        Self {
            queue_util: Some(queue_util),
            delay_ns: Some(delay_ns),
            ..Self::new()
        }
    }
    
    /// Add a custom metric
    pub fn add_custom_metric(&mut self, key: String, value: serde_json::Value) {
        if self.custom_metrics.is_none() {
            self.custom_metrics = Some(indexmap::IndexMap::new());
        }
        self.custom_metrics.as_mut().unwrap().insert(key, value);
    }
    
    /// Check if metrics are empty
    pub fn is_empty(&self) -> bool {
        self.queue_util.is_none() 
            && self.delay_ns.is_none()
            && self.bandwidth_bps.is_none()
            && self.drop_count.is_none()
            && self.egress_port.is_none()
            && self.ingress_port.is_none()
            && self.custom_metrics.as_ref().map_or(true, |m| m.is_empty())
    }
}

impl Default for TelemetryMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_metrics_creation() {
        let metrics = TelemetryMetrics::with_basic(0.75, 1000);
        assert_eq!(metrics.queue_util, Some(0.75));
        assert_eq!(metrics.delay_ns, Some(1000));
        assert!(!metrics.is_empty());
    }

    #[test]
    fn test_custom_metrics() {
        let mut metrics = TelemetryMetrics::new();
        metrics.add_custom_metric("custom_field".to_string(), serde_json::json!(42));
        
        assert!(metrics.custom_metrics.is_some());
        assert!(!metrics.is_empty());
    }

    #[test]
    fn test_serialization() {
        let metrics = TelemetryMetrics::with_basic(0.5, 500);
        let json = serde_json::to_string(&metrics).unwrap();
        let deserialized: TelemetryMetrics = serde_json::from_str(&json).unwrap();
        assert_eq!(metrics, deserialized);
    }
} 