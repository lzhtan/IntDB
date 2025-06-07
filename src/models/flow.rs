use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::models::{Hop, HopInput, NetworkPath, TelemetryMetrics};
use std::collections::HashMap;

/// Spatial metadata for flows (optional for spatiotemporal features)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpatialMetadata {
    /// Path signature hash for indexing
    pub path_signature: String,
    
    /// Logical path (always required)
    pub logical_path: Vec<String>,
    
    /// Physical topology coordinates (optional)
    pub topology_coordinates: Option<Vec<TopologyCoordinate>>,
    
    /// GIS-style path geometry (optional)
    pub path_geometry: Option<String>,
    
    /// Spatial extent/bounding box (optional)  
    pub spatial_extent: Option<SpatialExtent>,
    
    /// Network adjacency matrix (optional)
    pub adjacency_matrix: Option<Vec<Vec<u8>>>,
    
    /// Whether this flow has spatial information
    pub has_spatial_info: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TopologyCoordinate {
    pub switch: String,
    pub topo_x: f64,
    pub topo_y: f64,
    pub zone: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpatialExtent {
    pub min_x: f64,
    pub min_y: f64,
    pub max_x: f64,
    pub max_y: f64,
}

/// Temporal metadata for flows
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemporalMetadata {
    pub flow_state: FlowState,
    pub creation_time: DateTime<Utc>,
    pub last_update: DateTime<Utc>,
    pub window_duration: Option<u64>, // milliseconds
    pub retention_policy: Option<String>, // e.g., "7d"
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FlowState {
    Active,
    Completed,
    Timeout,
}

/// Quality metrics for time windows
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QualityMetrics {
    /// Logical path completeness ratio
    pub path_completeness: f64,
    
    /// Spatial coverage ratio (optional)
    pub spatial_coverage: Option<f64>,
    
    /// Temporal continuity ratio
    pub temporal_continuity: f64,
}

/// Spatial hop with both logical and physical information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpatialHop {
    /// Logical index in the path
    pub logical_index: u32,
    
    /// Switch identifier
    pub switch_id: String,
    
    /// Physical coordinates (optional)
    pub coordinates: Option<Coordinate3D>,
    
    /// Neighboring nodes (optional)
    pub neighborhood: Option<Vec<String>>,
    
    /// Time-series samples
    pub temporal_samples: Vec<TemporalSample>,
    
    /// Aggregated metrics for this hop
    pub aggregated_metrics: AggregatedMetrics,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Coordinate3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemporalSample {
    pub timestamp: DateTime<Utc>,
    pub metrics: TelemetryMetrics,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AggregatedMetrics {
    /// Average delay in nanoseconds
    pub avg_delay: Option<u64>,
    
    /// Maximum queue utilization
    pub max_queue: Option<f64>,
    
    /// Spatial gradient (optional)
    pub spatial_gradient: Option<SpatialGradient>,
    
    /// Temporal trend
    pub temporal_trend: Option<String>,
    
    /// Additional aggregated metrics
    pub additional_metrics: Option<HashMap<String, f64>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpatialGradient {
    pub dx_delay: Option<f64>, // delay change per hop in x direction
    pub dy_delay: Option<f64>, // delay change per hop in y direction
}

/// Spatiotemporal window containing time-bounded data
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpatiotemporalWindow {
    /// Window identifier (includes spatial info if available)
    pub st_window_id: String,
    
    /// Time bounds for this window
    pub temporal_bounds: TemporalBounds,
    
    /// Spatial bounds (optional)
    pub spatial_bounds: Option<SpatialExtent>,
    
    /// Number of packets in this window
    pub packet_count: u64,
    
    /// Quality metrics for this window
    pub quality_metrics: QualityMetrics,
    
    /// Hops with spatiotemporal data
    pub spatial_hops: Vec<SpatialHop>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemporalBounds {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

/// Index references for spatiotemporal queries
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpatiotemporalIndices {
    /// R-Tree spatial index reference (optional)
    pub rtree_index: Option<String>,
    
    /// B+ Tree temporal index reference
    pub temporal_btree: String,
    
    /// Compound spatiotemporal index reference (optional) 
    pub st_compound_index: Option<String>,
    
    /// Logical path trie index reference
    pub logical_path_trie: String,
}

/// Complete spatiotemporal flow record
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SpatiotemporalFlow {
    /// Unique flow identifier
    pub flow_id: String,
    
    /// Spatial metadata (with optional fields)
    pub spatial_metadata: SpatialMetadata,
    
    /// Temporal metadata
    pub temporal_metadata: TemporalMetadata,
    
    /// Time-windowed spatiotemporal data
    pub spatiotemporal_windows: Vec<SpatiotemporalWindow>,
    
    /// Index references
    pub spatiotemporal_indices: SpatiotemporalIndices,
}

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

impl SpatialMetadata {
    /// Create spatial metadata with only logical path (no spatial info)
    pub fn logical_only(logical_path: Vec<String>) -> Self {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        for switch in &logical_path {
            hasher.update(switch.as_bytes());
            hasher.update(b"->");
        }
        let path_signature = format!("{:x}", hasher.finalize());
        
        Self {
            path_signature,
            logical_path,
            topology_coordinates: None,
            path_geometry: None,
            spatial_extent: None,
            adjacency_matrix: None,
            has_spatial_info: false,
        }
    }
    
    /// Create spatial metadata with full spatial information
    pub fn with_spatial_info(
        logical_path: Vec<String>,
        topology_coordinates: Vec<TopologyCoordinate>,
        path_geometry: Option<String>,
    ) -> Self {
        use sha2::{Sha256, Digest};
        
        let mut hasher = Sha256::new();
        for switch in &logical_path {
            hasher.update(switch.as_bytes());
            hasher.update(b"->");
        }
        let path_signature = format!("{:x}", hasher.finalize());
        
        // Calculate spatial extent from coordinates
        let spatial_extent = if !topology_coordinates.is_empty() {
            let min_x = topology_coordinates.iter().map(|c| c.topo_x).fold(f64::INFINITY, f64::min);
            let max_x = topology_coordinates.iter().map(|c| c.topo_x).fold(f64::NEG_INFINITY, f64::max);
            let min_y = topology_coordinates.iter().map(|c| c.topo_y).fold(f64::INFINITY, f64::min);
            let max_y = topology_coordinates.iter().map(|c| c.topo_y).fold(f64::NEG_INFINITY, f64::max);
            
            Some(SpatialExtent { min_x, min_y, max_x, max_y })
        } else {
            None
        };
        
        Self {
            path_signature,
            logical_path,
            topology_coordinates: Some(topology_coordinates),
            path_geometry,
            spatial_extent,
            adjacency_matrix: None, // TODO: Auto-generate from topology
            has_spatial_info: true,
        }
    }
}

impl SpatiotemporalFlow {
    /// Create a new spatiotemporal flow with logical path only
    pub fn new_logical(flow_id: String, logical_path: Vec<String>) -> Self {
        let spatial_metadata = SpatialMetadata::logical_only(logical_path);
        let temporal_metadata = TemporalMetadata {
            flow_state: FlowState::Active,
            creation_time: Utc::now(),
            last_update: Utc::now(),
            window_duration: Some(60000), // 60 seconds default
            retention_policy: Some("7d".to_string()),
        };
        
        Self {
            flow_id: flow_id.clone(),
            spatial_metadata,
            temporal_metadata,
            spatiotemporal_windows: Vec::new(),
            spatiotemporal_indices: SpatiotemporalIndices {
                rtree_index: None,
                temporal_btree: format!("time_idx_{}", flow_id),
                st_compound_index: None,
                logical_path_trie: format!("path_idx_{}", flow_id),
            },
        }
    }
    
    /// Add a new time window with telemetry data
    pub fn add_window(&mut self, window: SpatiotemporalWindow) {
        self.spatiotemporal_windows.push(window);
        self.temporal_metadata.last_update = Utc::now();
    }
    
    /// Convert from legacy Flow format
    pub fn from_legacy_flow(flow: Flow) -> Self {
        let mut spatiotemporal_flow = Self::new_logical(
            flow.flow_id,
            flow.path.switches,
        );
        
        // Convert hops to a single spatiotemporal window
        let window = SpatiotemporalWindow {
            st_window_id: format!("st_w1_{}_logical", flow.start_time.timestamp()),
            temporal_bounds: TemporalBounds {
                start: flow.start_time,
                end: flow.end_time,
            },
            spatial_bounds: None,
            packet_count: flow.hops.len() as u64,
            quality_metrics: QualityMetrics {
                path_completeness: 1.0, // Assume complete for legacy flows
                spatial_coverage: None,
                temporal_continuity: 1.0,
            },
            spatial_hops: flow.hops.into_iter().map(|hop| {
                SpatialHop {
                    logical_index: hop.hop_index,
                    switch_id: hop.switch_id,
                    coordinates: None,
                    neighborhood: None,
                    temporal_samples: vec![TemporalSample {
                        timestamp: hop.timestamp,
                        metrics: hop.metrics.clone(),
                    }],
                    aggregated_metrics: AggregatedMetrics {
                        avg_delay: hop.metrics.delay_ns,
                        max_queue: hop.metrics.queue_util,
                        spatial_gradient: None,
                        temporal_trend: Some("stable".to_string()),
                        additional_metrics: None,
                    },
                }
            }).collect(),
        };
        
        spatiotemporal_flow.add_window(window);
        spatiotemporal_flow.temporal_metadata.flow_state = match flow.status {
            FlowStatus::Complete => FlowState::Completed,
            FlowStatus::Partial => FlowState::Active,
            FlowStatus::Timeout => FlowState::Timeout,
            FlowStatus::Error(_) => FlowState::Timeout,
        };
        
        spatiotemporal_flow
    }
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
    
    /// Convert to new spatiotemporal format
    pub fn to_spatiotemporal(self) -> SpatiotemporalFlow {
        SpatiotemporalFlow::from_legacy_flow(self)
    }
}

/// Input format for creating flows from JSON
#[derive(Debug, Serialize, Deserialize)]
pub struct FlowInput {
    pub flow_id: String,
    pub telemetry: Vec<HopInput>,
}

/// Input format for creating spatiotemporal flows
#[derive(Debug, Serialize, Deserialize)]
pub struct SpatiotemporalFlowInput {
    pub flow_id: String,
    pub logical_path: Vec<String>,
    pub topology_coordinates: Option<Vec<TopologyCoordinate>>,
    pub telemetry_data: Vec<HopTelemetryInput>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HopTelemetryInput {
    pub hop_index: u32,
    pub switch_id: String,
    pub coordinates: Option<Coordinate3D>,
    pub temporal_samples: Vec<TemporalSampleInput>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TemporalSampleInput {
    pub timestamp: DateTime<Utc>,
    pub queue_util: Option<f64>,
    pub delay_ns: Option<u64>,
    pub bandwidth_bps: Option<u64>,
    pub drop_count: Option<u64>,
    pub egress_port: Option<u32>,
    pub ingress_port: Option<u32>,
}

impl TryFrom<SpatiotemporalFlowInput> for SpatiotemporalFlow {
    type Error = FlowError;
    
    fn try_from(input: SpatiotemporalFlowInput) -> Result<Self, Self::Error> {
        let spatial_metadata = if let Some(coordinates) = input.topology_coordinates {
            SpatialMetadata::with_spatial_info(input.logical_path, coordinates, None)
        } else {
            SpatialMetadata::logical_only(input.logical_path)
        };
        
        let mut spatiotemporal_flow = SpatiotemporalFlow {
            flow_id: input.flow_id.clone(),
            spatial_metadata: spatial_metadata.clone(),
            temporal_metadata: TemporalMetadata {
                flow_state: FlowState::Active,
                creation_time: Utc::now(),
                last_update: Utc::now(),
                window_duration: Some(60000),
                retention_policy: Some("7d".to_string()),
            },
            spatiotemporal_windows: Vec::new(),
            spatiotemporal_indices: SpatiotemporalIndices {
                rtree_index: None,
                temporal_btree: format!("time_idx_{}", input.flow_id),
                st_compound_index: None,
                logical_path_trie: format!("path_idx_{}", input.flow_id),
            },
        };
        
        // Group telemetry by time windows (simplified - use first and last timestamps)
        if !input.telemetry_data.is_empty() {
            let start_time = input.telemetry_data.iter()
                .flat_map(|hop| &hop.temporal_samples)
                .map(|sample| sample.timestamp)
                .min()
                .unwrap_or_else(Utc::now);
                
            let end_time = input.telemetry_data.iter()
                .flat_map(|hop| &hop.temporal_samples)
                .map(|sample| sample.timestamp)
                .max()
                .unwrap_or(start_time);
            
            let window = SpatiotemporalWindow {
                st_window_id: format!("st_w1_{}_{}",
                    start_time.timestamp(),
                    if spatial_metadata.has_spatial_info { "spatial" } else { "logical" }
                ),
                temporal_bounds: TemporalBounds {
                    start: start_time,
                    end: end_time,
                },
                spatial_bounds: spatial_metadata.spatial_extent.clone(),
                packet_count: input.telemetry_data.iter()
                    .map(|hop| hop.temporal_samples.len() as u64)
                    .sum(),
                quality_metrics: QualityMetrics {
                    path_completeness: 1.0,
                    spatial_coverage: if spatial_metadata.has_spatial_info { Some(1.0) } else { None },
                    temporal_continuity: 1.0,
                },
                spatial_hops: input.telemetry_data.into_iter().map(|hop_input| {
                    let temporal_samples: Vec<TemporalSample> = hop_input.temporal_samples
                        .into_iter()
                        .map(|sample| TemporalSample {
                            timestamp: sample.timestamp,
                            metrics: TelemetryMetrics {
                                queue_util: sample.queue_util,
                                delay_ns: sample.delay_ns,
                                bandwidth_bps: sample.bandwidth_bps,
                                drop_count: sample.drop_count,
                                egress_port: sample.egress_port,
                                ingress_port: sample.ingress_port,
                                custom_metrics: None,
                            },
                        })
                        .collect();
                    
                    let avg_delay = temporal_samples.iter()
                        .filter_map(|s| s.metrics.delay_ns)
                        .sum::<u64>()
                        .checked_div(temporal_samples.len() as u64);
                    
                    let max_queue = temporal_samples.iter()
                        .filter_map(|s| s.metrics.queue_util)
                        .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                    
                    SpatialHop {
                        logical_index: hop_input.hop_index,
                        switch_id: hop_input.switch_id,
                        coordinates: hop_input.coordinates,
                        neighborhood: None, // TODO: Extract from spatial metadata
                        temporal_samples,
                        aggregated_metrics: AggregatedMetrics {
                            avg_delay,
                            max_queue,
                            spatial_gradient: None, // TODO: Calculate from coordinates
                            temporal_trend: Some("stable".to_string()),
                            additional_metrics: None,
                        },
                    }
                }).collect(),
            };
            
            spatiotemporal_flow.add_window(window);
        }
        
        Ok(spatiotemporal_flow)
    }
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