use std::sync::Arc;
use std::time::SystemTime;
use axum::{
    extract::{Path, State, Query},
    http::StatusCode,
    Json,
};
use std::collections::HashMap;

use crate::models::{Flow, SpatiotemporalFlow};
use crate::storage::{StorageEngine, QueryBuilder, TimeCondition};
use crate::api::{
    ApiError, ApiResult,
    InsertFlowRequest, InsertFlowResponse,
    FlowResponse, FlowsResponse,
    QueryRequest, QueryResponse,
    HealthResponse, StatsResponse,
    // New DTOs for spatiotemporal flows
    InsertSpatiotemporalFlowRequest, SpatiotemporalFlowResponse,
    SpatiotemporalQueryRequest, SpatiotemporalQueryResponse,
    GrafanaQueryRequest, GrafanaQueryResponse,
    GrafanaTimeSeries,
};

/// Application state containing the storage engine
#[derive(Debug, Clone)]
pub struct AppState {
    pub engine: Arc<StorageEngine>,
    pub start_time: SystemTime,
}

impl AppState {
    pub fn new(engine: StorageEngine) -> Self {
        Self {
            engine: Arc::new(engine),
            start_time: SystemTime::now(),
        }
    }
}

/// Health check endpoint
pub async fn health_check(State(state): State<AppState>) -> ApiResult<Json<HealthResponse>> {
    let uptime = state.start_time
        .elapsed()
        .map_err(|e| ApiError::internal(format!("Time error: {}", e)))?
        .as_secs();
    
    let response = HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: uptime,
        flow_count: state.engine.flow_count(),
    };
    
    Ok(Json(response))
}

/// Get statistics
pub async fn get_stats(State(state): State<AppState>) -> ApiResult<Json<StatsResponse>> {
    // For now, return basic stats. In a full implementation, 
    // we'd add a stats() method to StorageEngine
    let response = StatsResponse {
        total_flows: state.engine.flow_count(),
        unique_paths: 0, // Would need to implement this
        unique_switches: 0, // Would need to implement this  
        time_buckets: 0, // Would need to implement this
        memory_usage_estimate: state.engine.estimate_memory_usage(),
    };
    
    Ok(Json(response))
}

/// Insert a new flow (legacy format)
pub async fn insert_flow(
    State(state): State<AppState>,
    Json(request): Json<InsertFlowRequest>,
) -> ApiResult<Json<InsertFlowResponse>> {
    // Convert FlowInput to Flow
    let flow = Flow::try_from(request.flow)?;
    let flow_id = flow.flow_id.clone();
    
    // Insert into storage
    state.engine.insert_flow(flow)?;
    
    let response = InsertFlowResponse {
        flow_id: flow_id.clone(),
        status: "success".to_string(),
        message: format!("Flow {} inserted successfully", flow_id),
    };
    
    Ok(Json(response))
}

/// Insert a new spatiotemporal flow (new format)
pub async fn insert_spatiotemporal_flow(
    State(state): State<AppState>,
    Json(request): Json<InsertSpatiotemporalFlowRequest>,
) -> ApiResult<Json<InsertFlowResponse>> {
    // Convert SpatiotemporalFlowInput to SpatiotemporalFlow
    let spatiotemporal_flow = SpatiotemporalFlow::try_from(request.flow)?;
    let flow_id = spatiotemporal_flow.flow_id.clone();
    
    // Convert to legacy Flow format for storage (until storage engine is updated)
    let legacy_flow = convert_spatiotemporal_to_legacy(spatiotemporal_flow)?;
    
    // Insert into storage
    state.engine.insert_flow(legacy_flow)?;
    
    let response = InsertFlowResponse {
        flow_id: flow_id.clone(),
        status: "success".to_string(),
        message: format!("Spatiotemporal flow {} inserted successfully", flow_id),
    };
    
    Ok(Json(response))
}

/// Helper function to convert spatiotemporal flow to legacy format
/// TODO: This is temporary until we update the storage engine
fn convert_spatiotemporal_to_legacy(st_flow: SpatiotemporalFlow) -> Result<Flow, ApiError> {
    if st_flow.spatiotemporal_windows.is_empty() {
        return Err(ApiError::bad_request("No spatiotemporal windows found"));
    }
    
    // Use the first window to create a legacy flow
    let window = &st_flow.spatiotemporal_windows[0];
    
    let hops: Vec<crate::models::Hop> = window.spatial_hops.iter().map(|spatial_hop| {
        // Use the first temporal sample if available
        let (timestamp, metrics) = if let Some(sample) = spatial_hop.temporal_samples.first() {
            (sample.timestamp, sample.metrics.clone())
        } else {
            // Create default values
            (chrono::Utc::now(), crate::models::TelemetryMetrics::default())
        };
        
        crate::models::Hop::new(
            spatial_hop.logical_index,
            spatial_hop.switch_id.clone(),
            timestamp,
            metrics,
        )
    }).collect();
    
    Flow::new(st_flow.flow_id, hops)
        .map_err(|e| ApiError::bad_request(&format!("Invalid flow data: {}", e)))
}

/// Get a flow by ID
pub async fn get_flow(
    State(state): State<AppState>,
    Path(flow_id): Path<String>,
) -> ApiResult<Json<FlowResponse>> {
    let flow = state.engine.get_flow(&flow_id)
        .ok_or_else(|| ApiError::not_found(format!("Flow {}", flow_id)))?;
    
    let response = FlowResponse { flow };
    
    Ok(Json(response))
}

/// Get a spatiotemporal flow by ID
pub async fn get_spatiotemporal_flow(
    State(state): State<AppState>,
    Path(flow_id): Path<String>,
) -> ApiResult<Json<SpatiotemporalFlowResponse>> {
    let flow = state.engine.get_flow(&flow_id)
        .ok_or_else(|| ApiError::not_found(format!("Flow {}", flow_id)))?;
    
    // Convert legacy flow to spatiotemporal format
    let spatiotemporal_flow = flow.to_spatiotemporal();
    
    let response = SpatiotemporalFlowResponse { 
        flow: spatiotemporal_flow 
    };
    
    Ok(Json(response))
}

/// Query flows (legacy format)
pub async fn query_flows(
    State(state): State<AppState>,
    Json(request): Json<QueryRequest>,
) -> ApiResult<Json<QueryResponse>> {
    // Build query from request
    let mut query_builder = QueryBuilder::new();
    
    // Add path conditions
    for condition_dto in request.path_conditions {
        let condition = condition_dto.into();
        query_builder = query_builder.with_path_condition(condition);
    }
    
    // Add time conditions
    for condition_dto in request.time_conditions {
        let condition = condition_dto.into();
        query_builder = query_builder.with_time_condition(condition);
    }
    
    // Add metric conditions
    for condition_dto in request.metric_conditions {
        let condition = condition_dto.into();
        query_builder = query_builder.with_metric_condition(condition);
    }
    
    // Add pagination
    if let Some(limit) = request.limit {
        query_builder = query_builder.limit(limit);
    }
    
    if let Some(skip) = request.skip {
        query_builder = query_builder.skip(skip);
    }
    
    // Execute query
    let query_result = state.engine.query(query_builder)?;
    
    // Convert to response
    let mut response: QueryResponse = query_result.into();
    
    // Include full flow data if requested
    if request.include_flows {
        let flows = state.engine.get_flows(&response.flow_ids);
        response.flows = Some(flows);
    }
    
    Ok(Json(response))
}

/// Query spatiotemporal flows (new format)
pub async fn query_spatiotemporal_flows(
    State(state): State<AppState>,
    Json(request): Json<SpatiotemporalQueryRequest>,
) -> ApiResult<Json<SpatiotemporalQueryResponse>> {
    // Build query from request (reuse legacy query builder for now)
    let mut query_builder = QueryBuilder::new();
    
    // Add logical path conditions
    if let Some(logical_path_conditions) = request.logical_path_conditions {
        for condition_dto in logical_path_conditions {
            let condition = condition_dto.into();
            query_builder = query_builder.with_path_condition(condition);
        }
    }
    
    // Add temporal conditions
    if let Some(temporal_conditions) = request.temporal_conditions {
        for condition_dto in temporal_conditions {
            let condition = condition_dto.into();
            query_builder = query_builder.with_time_condition(condition);
        }
    }
    
    // Add spatial conditions (TODO: implement spatial query logic)
    if request.spatial_conditions.is_some() {
        return Err(ApiError::bad_request("Spatial queries not yet implemented"));
    }
    
    // Add pagination
    if let Some(limit) = request.limit {
        query_builder = query_builder.limit(limit);
    }
    
    if let Some(skip) = request.skip {
        query_builder = query_builder.skip(skip);
    }
    
    // Execute query
    let query_result = state.engine.query(query_builder)?;
    
    // Convert flows to spatiotemporal format
    let spatiotemporal_flows = if request.include_flows {
        let flows = state.engine.get_flows(&query_result.flow_ids);
        Some(flows.into_iter().map(|f| f.to_spatiotemporal()).collect())
    } else {
        None
    };
    
    let response = SpatiotemporalQueryResponse {
        flow_ids: query_result.flow_ids,
        total_count: query_result.total_count,
        limit: query_result.limit,
        flows: spatiotemporal_flows,
    };
    
    Ok(Json(response))
}

/// Get multiple flows by IDs (via query parameters or POST body)
pub async fn get_flows(
    State(state): State<AppState>,
    Json(flow_ids): Json<Vec<String>>,
) -> ApiResult<Json<FlowsResponse>> {
    let flows = state.engine.get_flows(&flow_ids);
    
    let response = FlowsResponse {
        count: flows.len(),
        flows,
    };
    
    Ok(Json(response))
}

/// Delete a flow by ID (optional - may not be needed for INT use case)
pub async fn delete_flow(
    State(state): State<AppState>,
    Path(flow_id): Path<String>,
) -> ApiResult<StatusCode> {
    // Check if flow exists
    if state.engine.get_flow(&flow_id).is_none() {
        return Err(ApiError::not_found(format!("Flow {}", flow_id)));
    }
    
    // For now, return method not allowed since deletion might not be needed for INT
    Err(ApiError::bad_request("Flow deletion not implemented"))
}

/// Quick query endpoint for common cases
pub async fn quick_query_through_switch(
    State(state): State<AppState>,
    Path(switch_id): Path<String>,
) -> ApiResult<Json<QueryResponse>> {
    let query = QueryBuilder::through_switch(&switch_id).limit(100);
    let query_result = state.engine.query(query)?;
    
    let response: QueryResponse = query_result.into();
    Ok(Json(response))
}

/// Quick query endpoint for exact path
pub async fn quick_query_exact_path(
    State(state): State<AppState>,
    Json(switches): Json<Vec<String>>,
) -> ApiResult<Json<QueryResponse>> {
    let path = crate::models::NetworkPath::new(switches);
    let query = QueryBuilder::exact_path(path).limit(100);
    let query_result = state.engine.query(query)?;
    
    let response: QueryResponse = query_result.into();
    Ok(Json(response))
}

/// Quick query endpoint for recent flows
pub async fn quick_query_recent(
    State(state): State<AppState>,
    Path(minutes): Path<i64>,
) -> ApiResult<Json<QueryResponse>> {
    let query = QueryBuilder::in_last_minutes(minutes).limit(100);
    let query_result = state.engine.query(query)?;
    
    let response: QueryResponse = query_result.into();
    Ok(Json(response))
}

/// Spatiotemporal-specific quick queries

/// Quick query for flows in spatial region
pub async fn quick_query_spatial_region(
    State(_state): State<AppState>,
    Json(_bounds): Json<crate::models::SpatialExtent>,
) -> ApiResult<Json<SpatiotemporalQueryResponse>> {
    // TODO: Implement spatial indexing and queries
    Err(ApiError::bad_request("Spatial region queries not yet implemented"))
}

/// Quick query for flows with spatial information
pub async fn quick_query_spatial_flows(
    State(state): State<AppState>,
) -> ApiResult<Json<SpatiotemporalQueryResponse>> {
    // For now, return all flows and filter client-side
    // TODO: Add spatial filtering to storage engine
    let query = QueryBuilder::new().limit(100);
    let query_result = state.engine.query(query)?;
    
    let flows = state.engine.get_flows(&query_result.flow_ids);
    let spatiotemporal_flows: Vec<SpatiotemporalFlow> = flows
        .into_iter()
        .map(|f| f.to_spatiotemporal())
        .filter(|sf| sf.spatial_metadata.has_spatial_info)
        .collect();
    
    let filtered_flow_ids: Vec<String> = spatiotemporal_flows
        .iter()
        .map(|sf| sf.flow_id.clone())
        .collect();
    
    let response = SpatiotemporalQueryResponse {
        flow_ids: filtered_flow_ids,
        total_count: spatiotemporal_flows.len(),
        limit: Some(100),
        flows: Some(spatiotemporal_flows),
    };
    
    Ok(Json(response))
}

/// Prometheus metrics endpoint for Grafana integration
pub async fn prometheus_metrics(State(state): State<AppState>) -> ApiResult<String> {
    let flow_count = state.engine.flow_count();
    let uptime = state.start_time
        .elapsed()
        .map_err(|e| ApiError::internal(format!("Time error: {}", e)))?
        .as_secs();
    
    // Get detailed network statistics
    let mut query_builder = QueryBuilder::new();
    let query_result = state.engine.query(query_builder)?;
    let flows = state.engine.get_flows(&query_result.flow_ids);
    
    // Calculate network metrics
    let delay_values: Vec<u64> = flows.iter()
        .flat_map(|f| &f.hops)
        .filter_map(|h| h.metrics.delay_ns)
        .collect();
    
    let queue_values: Vec<f64> = flows.iter()
        .flat_map(|f| &f.hops)
        .filter_map(|h| h.metrics.queue_util)
        .collect();
    
    let unique_switches: std::collections::HashSet<_> = flows.iter()
        .flat_map(|f| &f.hops)
        .map(|h| &h.switch_id)
        .collect();
    
    let unique_paths: std::collections::HashSet<_> = flows.iter()
        .map(|f| &f.path.switches)
        .collect();
    
    // Calculate statistics
    let avg_delay = if !delay_values.is_empty() {
        delay_values.iter().sum::<u64>() as f64 / delay_values.len() as f64
    } else { 0.0 };
    
    let max_delay = delay_values.iter().max().unwrap_or(&0).clone() as f64;
    
    let avg_queue_util = if !queue_values.is_empty() {
        queue_values.iter().sum::<f64>() / queue_values.len() as f64
    } else { 0.0 };
    
    let max_queue_util = queue_values.iter().fold(0.0f64, |a, &b| a.max(b));
    
    let congested_hops = queue_values.iter().filter(|&&x| x > 0.8).count();
    let congestion_ratio = if !queue_values.is_empty() {
        congested_hops as f64 / queue_values.len() as f64
    } else { 0.0 };
    
    let avg_path_length = if !flows.is_empty() {
        flows.iter().map(|f| f.hops.len()).sum::<usize>() as f64 / flows.len() as f64
    } else { 0.0 };
    
    // Count flows by status
    use crate::models::flow::FlowStatus;
    let active_flows = flows.iter().filter(|f| matches!(f.status, FlowStatus::Partial)).count();
    let complete_flows = flows.iter().filter(|f| matches!(f.status, FlowStatus::Complete)).count();
    let timeout_flows = flows.iter().filter(|f| matches!(f.status, FlowStatus::Timeout)).count();
    
    // Generate Prometheus format metrics
    let metrics = format!(
        r#"# HELP intdb_flows_total Total number of flows stored
# TYPE intdb_flows_total gauge
intdb_flows_total {}

# HELP intdb_uptime_seconds Service uptime in seconds
# TYPE intdb_uptime_seconds gauge
intdb_uptime_seconds {}

# HELP intdb_memory_usage_estimate_bytes Estimated memory usage in bytes
# TYPE intdb_memory_usage_estimate_bytes gauge
intdb_memory_usage_estimate_bytes {}

# HELP intdb_api_health Service health status (1=healthy, 0=unhealthy)
# TYPE intdb_api_health gauge
intdb_api_health 1

# HELP intdb_avg_delay_ns Average network delay in nanoseconds
# TYPE intdb_avg_delay_ns gauge
intdb_avg_delay_ns {}

# HELP intdb_max_delay_ns Maximum network delay in nanoseconds
# TYPE intdb_max_delay_ns gauge
intdb_max_delay_ns {}

# HELP intdb_avg_queue_utilization Average queue utilization ratio (0-1)
# TYPE intdb_avg_queue_utilization gauge
intdb_avg_queue_utilization {}

# HELP intdb_max_queue_utilization Maximum queue utilization ratio (0-1)
# TYPE intdb_max_queue_utilization gauge
intdb_max_queue_utilization {}

# HELP intdb_queue_congestion_ratio Ratio of congested hops (queue > 80%)
# TYPE intdb_queue_congestion_ratio gauge
intdb_queue_congestion_ratio {}

# HELP intdb_unique_switches Number of unique switches in the network
# TYPE intdb_unique_switches gauge
intdb_unique_switches {}

# HELP intdb_unique_paths Number of unique network paths
# TYPE intdb_unique_paths gauge
intdb_unique_paths {}

# HELP intdb_avg_path_length Average path length (number of hops)
# TYPE intdb_avg_path_length gauge
intdb_avg_path_length {}

# HELP intdb_flows_active Number of active flows
# TYPE intdb_flows_active gauge
intdb_flows_active {}

# HELP intdb_flows_complete Number of completed flows
# TYPE intdb_flows_complete gauge
intdb_flows_complete {}

# HELP intdb_flows_timeout Number of timed-out flows
# TYPE intdb_flows_timeout gauge
intdb_flows_timeout {}
"#,
        flow_count,
        uptime,
        flow_count * 1024,  // Rough memory estimate
        avg_delay,
        max_delay,
        avg_queue_util,
        max_queue_util,
        congestion_ratio,
        unique_switches.len(),
        unique_paths.len(),
        avg_path_length,
        active_flows,
        complete_flows,
        timeout_flows
    );
    
    Ok(metrics)
}

/// Grafana-compatible query endpoint
/// Returns time series data for network flows
pub async fn grafana_query(
    State(state): State<AppState>,
    Json(request): Json<GrafanaQueryRequest>,
) -> ApiResult<Json<GrafanaQueryResponse>> {
    // Extract time range from request
    let from_time = request.range.from.parse::<chrono::DateTime<chrono::Utc>>()
        .map_err(|e| ApiError::bad_request(&format!("Invalid from time: {}", e)))?;
    let to_time = request.range.to.parse::<chrono::DateTime<chrono::Utc>>()
        .map_err(|e| ApiError::bad_request(&format!("Invalid to time: {}", e)))?;

    // Build query based on Grafana target
    let mut query_builder = QueryBuilder::new();
    
    // Add time condition
    query_builder = query_builder.with_time_condition(
        TimeCondition::InRange(from_time, to_time)
    );
    
    // Parse target metric (e.g., "delay", "queue_util", "flow_count")
    let target = &request.targets[0];
    let metric_type = target.target.as_str();
    
    // Execute query
    let query_result = state.engine.query(query_builder)?;
    let flows = state.engine.get_flows(&query_result.flow_ids);
    
    // Convert to Grafana time series format
    let mut datapoints = Vec::new();
    
    match metric_type {
        "flow_count" => {
            // Return flow count over time
            let count = flows.len() as f64;
            let timestamp = chrono::Utc::now().timestamp_millis();
            datapoints.push(vec![count, timestamp as f64]);
        },
        "avg_delay" => {
            // Calculate average delay
            let delay_values: Vec<u64> = flows.iter()
                .flat_map(|f| &f.hops)
                .filter_map(|h| h.metrics.delay_ns)
                .collect();
            
            if !delay_values.is_empty() {
                let avg_delay = delay_values.iter().sum::<u64>() as f64 / delay_values.len() as f64;
                let timestamp = chrono::Utc::now().timestamp_millis();
                datapoints.push(vec![avg_delay, timestamp as f64]);
            }
        },
        "avg_queue_util" => {
            // Calculate average queue utilization
            let queue_values: Vec<f64> = flows.iter()
                .flat_map(|f| &f.hops)
                .filter_map(|h| h.metrics.queue_util)
                .collect();
            
            if !queue_values.is_empty() {
                let avg_queue = queue_values.iter().sum::<f64>() / queue_values.len() as f64;
                let timestamp = chrono::Utc::now().timestamp_millis();
                datapoints.push(vec![avg_queue, timestamp as f64]);
            }
        },
        _ => {
            return Err(ApiError::bad_request(&format!("Unknown metric: {}", metric_type)));
        }
    }
    
    let response = GrafanaQueryResponse {
        data: vec![GrafanaTimeSeries {
            target: target.target.clone(),
            datapoints,
        }],
    };
    
    Ok(Json(response))
}

/// Standard Prometheus API query endpoint
/// This is what Grafana actually calls when configured as a Prometheus data source
pub async fn prometheus_query(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> ApiResult<Json<serde_json::Value>> {
    let query = params.get("query").unwrap_or(&"".to_string()).clone();
    
    // Get real-time data from the storage engine
    let flow_count = state.engine.flow_count();
    let uptime = state.start_time
        .elapsed()
        .map_err(|e| ApiError::internal(format!("Time error: {}", e)))?
        .as_secs();
    
    // Calculate network metrics from actual flow data
    let network_metrics = calculate_network_metrics(&state);
    
    // Parse basic Prometheus queries and return real data
    match query.as_str() {
        "intdb_flows_total" => {
            Ok(Json(serde_json::json!({
                "status": "success",
                "data": {
                    "resultType": "vector",
                    "result": [
                        {
                            "metric": {"__name__": "intdb_flows_total"},
                            "value": [chrono::Utc::now().timestamp(), flow_count.to_string()]
                        }
                    ]
                }
            })))
        },
        "intdb_uptime_seconds" => {
            Ok(Json(serde_json::json!({
                "status": "success",
                "data": {
                    "resultType": "vector",
                    "result": [
                        {
                            "metric": {"__name__": "intdb_uptime_seconds"},
                            "value": [chrono::Utc::now().timestamp(), uptime.to_string()]
                        }
                    ]
                }
            })))
        },
        "intdb_memory_usage_estimate_bytes" => {
            Ok(Json(serde_json::json!({
                "status": "success",
                "data": {
                    "resultType": "vector",
                    "result": [
                        {
                            "metric": {"__name__": "intdb_memory_usage_estimate_bytes"},
                            "value": [chrono::Utc::now().timestamp(), (flow_count * 1024).to_string()]
                        }
                    ]
                }
            })))
        },
        "intdb_api_health" => {
            Ok(Json(serde_json::json!({
                "status": "success",
                "data": {
                    "resultType": "vector",
                    "result": [
                        {
                            "metric": {"__name__": "intdb_api_health"},
                            "value": [chrono::Utc::now().timestamp(), "1"]
                        }
                    ]
                }
            })))
        },
        "intdb_avg_delay_ns" => {
            Ok(Json(serde_json::json!({
                "status": "success",
                "data": {
                    "resultType": "vector",
                    "result": [
                        {
                            "metric": {"__name__": "intdb_avg_delay_ns"},
                            "value": [chrono::Utc::now().timestamp(), network_metrics.avg_delay.to_string()]
                        }
                    ]
                }
            })))
        },
        "intdb_max_delay_ns" => {
            Ok(Json(serde_json::json!({
                "status": "success",
                "data": {
                    "resultType": "vector",
                    "result": [
                        {
                            "metric": {"__name__": "intdb_max_delay_ns"},
                            "value": [chrono::Utc::now().timestamp(), network_metrics.max_delay.to_string()]
                        }
                    ]
                }
            })))
        },
        "intdb_avg_queue_utilization" => {
            Ok(Json(serde_json::json!({
                "status": "success",
                "data": {
                    "resultType": "vector",
                    "result": [
                        {
                            "metric": {"__name__": "intdb_avg_queue_utilization"},
                            "value": [chrono::Utc::now().timestamp(), network_metrics.avg_queue_util.to_string()]
                        }
                    ]
                }
            })))
        },
        "intdb_max_queue_utilization" => {
            Ok(Json(serde_json::json!({
                "status": "success",
                "data": {
                    "resultType": "vector",
                    "result": [
                        {
                            "metric": {"__name__": "intdb_max_queue_utilization"},
                            "value": [chrono::Utc::now().timestamp(), network_metrics.max_queue_util.to_string()]
                        }
                    ]
                }
            })))
        },
        "intdb_queue_congestion_ratio" => {
            Ok(Json(serde_json::json!({
                "status": "success",
                "data": {
                    "resultType": "vector",
                    "result": [
                        {
                            "metric": {"__name__": "intdb_queue_congestion_ratio"},
                            "value": [chrono::Utc::now().timestamp(), network_metrics.congestion_ratio.to_string()]
                        }
                    ]
                }
            })))
        },
        "intdb_unique_switches" => {
            Ok(Json(serde_json::json!({
                "status": "success",
                "data": {
                    "resultType": "vector",
                    "result": [
                        {
                            "metric": {"__name__": "intdb_unique_switches"},
                            "value": [chrono::Utc::now().timestamp(), network_metrics.unique_switches.to_string()]
                        }
                    ]
                }
            })))
        },
        "intdb_unique_paths" => {
            Ok(Json(serde_json::json!({
                "status": "success",
                "data": {
                    "resultType": "vector",
                    "result": [
                        {
                            "metric": {"__name__": "intdb_unique_paths"},
                            "value": [chrono::Utc::now().timestamp(), network_metrics.unique_paths.to_string()]
                        }
                    ]
                }
            })))
        },
        "intdb_avg_path_length" => {
            Ok(Json(serde_json::json!({
                "status": "success",
                "data": {
                    "resultType": "vector",
                    "result": [
                        {
                            "metric": {"__name__": "intdb_avg_path_length"},
                            "value": [chrono::Utc::now().timestamp(), network_metrics.avg_path_length.to_string()]
                        }
                    ]
                }
            })))
        },
        "intdb_flows_active" => {
            // For now, assume all flows are active (would need flow state tracking)
            Ok(Json(serde_json::json!({
                "status": "success",
                "data": {
                    "resultType": "vector",
                    "result": [
                        {
                            "metric": {"__name__": "intdb_flows_active"},
                            "value": [chrono::Utc::now().timestamp(), flow_count.to_string()]
                        }
                    ]
                }
            })))
        },
        "intdb_flows_complete" => {
            // For now, return 0 (would need flow state tracking)
            Ok(Json(serde_json::json!({
                "status": "success",
                "data": {
                    "resultType": "vector",
                    "result": [
                        {
                            "metric": {"__name__": "intdb_flows_complete"},
                            "value": [chrono::Utc::now().timestamp(), "0"]
                        }
                    ]
                }
            })))
        },
        "intdb_flows_timeout" => {
            // For now, return 0 (would need flow state tracking)
            Ok(Json(serde_json::json!({
                "status": "success",
                "data": {
                    "resultType": "vector",
                    "result": [
                        {
                            "metric": {"__name__": "intdb_flows_timeout"},
                            "value": [chrono::Utc::now().timestamp(), "0"]
                        }
                    ]
                }
            })))
        },
        _ => {
            // Return empty result for unknown queries
            Ok(Json(serde_json::json!({
                "status": "success",
                "data": {
                    "resultType": "vector",
                    "result": []
                }
            })))
        }
    }
}

/// Prometheus range query endpoint
pub async fn prometheus_query_range(
    State(state): State<AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> ApiResult<Json<serde_json::Value>> {
    let query = params.get("query").unwrap_or(&"".to_string()).clone();
    
    // Parse time range parameters from Grafana
    let start_time = params.get("start")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or_else(|| chrono::Utc::now().timestamp() - 3600); // Default: 1 hour ago
    
    let end_time = params.get("end")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or_else(|| chrono::Utc::now().timestamp()); // Default: now
    
    let step = params.get("step")
        .and_then(|s| s.parse::<i64>().ok())
        .unwrap_or(30); // Default: 30 seconds
    
    // Get real-time data from the storage engine
    let flow_count = state.engine.flow_count();
    let uptime = state.start_time
        .elapsed()
        .map_err(|e| ApiError::internal(format!("Time error: {}", e)))?
        .as_secs();
    
    // Calculate network metrics from actual flow data
    let network_metrics = calculate_network_metrics(&state);
    
    // Generate time series data for the range query
    let current_timestamp = chrono::Utc::now().timestamp();
    
    match query.as_str() {
        "intdb_flows_total" => {
            // Create a simple time series with current value
            Ok(Json(serde_json::json!({
                "status": "success",
                "data": {
                    "resultType": "matrix",
                    "result": [
                        {
                            "metric": {"__name__": "intdb_flows_total"},
                            "values": [
                                [current_timestamp - 60, flow_count.to_string()],
                                [current_timestamp, flow_count.to_string()]
                            ]
                        }
                    ]
                }
            })))
        },
        "intdb_uptime_seconds" => {
            Ok(Json(serde_json::json!({
                "status": "success",
                "data": {
                    "resultType": "matrix",
                    "result": [
                        {
                            "metric": {"__name__": "intdb_uptime_seconds"},
                            "values": [
                                [current_timestamp - 60, (uptime - 60).to_string()],
                                [current_timestamp, uptime.to_string()]
                            ]
                        }
                    ]
                }
            })))
        },
        "intdb_memory_usage_estimate_bytes" => {
            Ok(Json(serde_json::json!({
                "status": "success",
                "data": {
                    "resultType": "matrix",
                    "result": [
                        {
                            "metric": {"__name__": "intdb_memory_usage_estimate_bytes"},
                            "values": [
                                [current_timestamp - 60, (flow_count * 1024).to_string()],
                                [current_timestamp, (flow_count * 1024).to_string()]
                            ]
                        }
                    ]
                }
            })))
        },
        "intdb_api_health" => {
            Ok(Json(serde_json::json!({
                "status": "success",
                "data": {
                    "resultType": "matrix",
                    "result": [
                        {
                            "metric": {"__name__": "intdb_api_health"},
                            "values": [
                                [current_timestamp - 60, "1"],
                                [current_timestamp, "1"]
                            ]
                        }
                    ]
                }
            })))
        },
        "intdb_avg_delay_ns" => {
            // Extract real historical data from IntDB flows based on time range
            let real_values = extract_historical_delay_data(&state, "avg", start_time, end_time, step);
            
            Ok(Json(serde_json::json!({
                "status": "success",
                "data": {
                    "resultType": "matrix",
                    "result": [
                        {
                            "metric": {"__name__": "intdb_avg_delay_ns"},
                            "values": real_values
                        }
                    ]
                }
            })))
        },
        "intdb_max_delay_ns" => {
            // Extract real historical data from IntDB flows based on time range
            let real_values = extract_historical_delay_data(&state, "max", start_time, end_time, step);
            
            Ok(Json(serde_json::json!({
                "status": "success",
                "data": {
                    "resultType": "matrix",
                    "result": [
                        {
                            "metric": {"__name__": "intdb_max_delay_ns"},
                            "values": real_values
                        }
                    ]
                }
            })))
        },
        "intdb_avg_queue_utilization" => {
            // Extract real historical data from IntDB flows based on time range
            let real_values = extract_historical_queue_data(&state, "avg", start_time, end_time, step);
            
            Ok(Json(serde_json::json!({
                "status": "success",
                "data": {
                    "resultType": "matrix",
                    "result": [
                        {
                            "metric": {"__name__": "intdb_avg_queue_utilization"},
                            "values": real_values
                        }
                    ]
                }
            })))
        },
        "intdb_max_queue_utilization" => {
            // Extract real historical data from IntDB flows based on time range
            let real_values = extract_historical_queue_data(&state, "max", start_time, end_time, step);
            
            Ok(Json(serde_json::json!({
                "status": "success",
                "data": {
                    "resultType": "matrix",
                    "result": [
                        {
                            "metric": {"__name__": "intdb_max_queue_utilization"},
                            "values": real_values
                        }
                    ]
                }
            })))
        },
        "intdb_queue_congestion_ratio" => {
            // Extract real historical data from IntDB flows based on time range
            let real_values = extract_historical_queue_data(&state, "congestion", start_time, end_time, step);
            
            Ok(Json(serde_json::json!({
                "status": "success",
                "data": {
                    "resultType": "matrix",
                    "result": [
                        {
                            "metric": {"__name__": "intdb_queue_congestion_ratio"},
                            "values": real_values
                        }
                    ]
                }
            })))
        },
        "intdb_unique_switches" => {
            Ok(Json(serde_json::json!({
                "status": "success",
                "data": {
                    "resultType": "matrix",
                    "result": [
                        {
                            "metric": {"__name__": "intdb_unique_switches"},
                            "values": [
                                [current_timestamp - 60, network_metrics.unique_switches.to_string()],
                                [current_timestamp, network_metrics.unique_switches.to_string()]
                            ]
                        }
                    ]
                }
            })))
        },
        "intdb_unique_paths" => {
            Ok(Json(serde_json::json!({
                "status": "success",
                "data": {
                    "resultType": "matrix",
                    "result": [
                        {
                            "metric": {"__name__": "intdb_unique_paths"},
                            "values": [
                                [current_timestamp - 60, network_metrics.unique_paths.to_string()],
                                [current_timestamp, network_metrics.unique_paths.to_string()]
                            ]
                        }
                    ]
                }
            })))
        },
        "intdb_avg_path_length" => {
            Ok(Json(serde_json::json!({
                "status": "success",
                "data": {
                    "resultType": "matrix",
                    "result": [
                        {
                            "metric": {"__name__": "intdb_avg_path_length"},
                            "values": [
                                [current_timestamp - 60, network_metrics.avg_path_length.to_string()],
                                [current_timestamp, network_metrics.avg_path_length.to_string()]
                            ]
                        }
                    ]
                }
            })))
        },
        "intdb_flows_active" => {
            Ok(Json(serde_json::json!({
                "status": "success",
                "data": {
                    "resultType": "matrix",
                    "result": [
                        {
                            "metric": {"__name__": "intdb_flows_active"},
                            "values": [
                                [current_timestamp - 60, flow_count.to_string()],
                                [current_timestamp, flow_count.to_string()]
                            ]
                        }
                    ]
                }
            })))
        },
        "intdb_flows_complete" => {
            Ok(Json(serde_json::json!({
                "status": "success",
                "data": {
                    "resultType": "matrix",
                    "result": [
                        {
                            "metric": {"__name__": "intdb_flows_complete"},
                            "values": [
                                [current_timestamp - 60, "0"],
                                [current_timestamp, "0"]
                            ]
                        }
                    ]
                }
            })))
        },
        "intdb_flows_timeout" => {
            Ok(Json(serde_json::json!({
                "status": "success",
                "data": {
                    "resultType": "matrix",
                    "result": [
                        {
                            "metric": {"__name__": "intdb_flows_timeout"},
                            "values": [
                                [current_timestamp - 60, "0"],
                                [current_timestamp, "0"]
                            ]
                        }
                    ]
                }
            })))
        },
        _ => {
            // Return empty result for unknown queries
            Ok(Json(serde_json::json!({
                "status": "success",
                "data": {
                    "resultType": "matrix",
                    "result": []
                }
            })))
        }
    }
}

/// Prometheus label values endpoint
pub async fn prometheus_label_values(
    State(_state): State<AppState>,
) -> ApiResult<Json<serde_json::Value>> {
    // Return available metric names
    Ok(Json(serde_json::json!({
        "status": "success",
        "data": [
            "intdb_flows_total",
            "intdb_uptime_seconds", 
            "intdb_memory_usage_estimate_bytes",
            "intdb_api_health",
            "intdb_avg_delay_ns",
            "intdb_max_delay_ns",
            "intdb_avg_queue_utilization",
            "intdb_max_queue_utilization",
            "intdb_queue_congestion_ratio",
            "intdb_unique_switches",
            "intdb_unique_paths",
            "intdb_avg_path_length",
            "intdb_flows_active",
            "intdb_flows_complete",
            "intdb_flows_timeout"
        ]
    })))
}

/// Prometheus labels endpoint
pub async fn prometheus_labels(
    State(_state): State<AppState>,
) -> ApiResult<Json<serde_json::Value>> {
    // Return available label names
    Ok(Json(serde_json::json!({
        "status": "success",
        "data": ["__name__"]
    })))
}

/// Network metrics calculated from flow data
#[derive(Debug, Clone)]
struct NetworkMetrics {
    avg_delay: f64,
    max_delay: i32,
    avg_queue_util: f64,
    max_queue_util: f64,
    congestion_ratio: f64,
    unique_switches: usize,
    unique_paths: usize,
    avg_path_length: f64,
}

/// Extract real historical delay data from IntDB flows based on time range
fn extract_historical_delay_data(state: &AppState, metric_type: &str, start_time: i64, end_time: i64, step: i64) -> Vec<[serde_json::Value; 2]> {
    let mut time_values = Vec::new();
    
    // Get flows and extract real historical data from hops
    if let Ok(query_result) = state.engine.query(
        crate::storage::QueryBuilder::new().limit(100) // Get more flows for better time coverage
    ) {
        let flows = state.engine.get_flows(&query_result.flow_ids);
        
        // Collect all hop data within the time range
        for flow in flows {
            for hop in &flow.hops {
                if let Some(delay) = hop.metrics.delay_ns {
                    let timestamp = hop.timestamp.timestamp();
                    
                    // Only include data within the requested time range
                    if timestamp >= start_time && timestamp <= end_time {
                        let delay_value = match metric_type {
                            "avg" => delay as f64,
                            "max" => delay as f64, // For max, we'll use the same value (simplification)
                            _ => delay as f64,
                        };
                        time_values.push([serde_json::Value::Number(serde_json::Number::from(timestamp)), serde_json::Value::String(delay_value.to_string())]);
                    }
                }
            }
        }
    }
    
    // Sort by timestamp
    time_values.sort_by_key(|v| v[0].as_i64().unwrap_or(0));
    
    // If we have real data, aggregate it into time buckets based on step
    if !time_values.is_empty() {
        let mut aggregated_values = Vec::new();
        let mut current_bucket_start = start_time;
        
        while current_bucket_start < end_time {
            let bucket_end = current_bucket_start + step;
            
            // Find all values in this time bucket
            let bucket_values: Vec<f64> = time_values.iter()
                .filter_map(|v| {
                    let timestamp = v[0].as_i64().unwrap_or(0);
                    if timestamp >= current_bucket_start && timestamp < bucket_end {
                        v[1].as_str().and_then(|s| s.parse::<f64>().ok())
                    } else {
                        None
                    }
                })
                .collect();
            
            // Calculate aggregated value for this bucket
            if !bucket_values.is_empty() {
                let aggregated_value = match metric_type {
                    "avg" => bucket_values.iter().sum::<f64>() / bucket_values.len() as f64,
                    "max" => bucket_values.iter().fold(0.0f64, |a, &b| a.max(b)),
                    _ => bucket_values.iter().sum::<f64>() / bucket_values.len() as f64,
                };
                
                // Use the middle of the time bucket as the timestamp
                let bucket_timestamp = current_bucket_start + step / 2;
                aggregated_values.push([
                    serde_json::Value::Number(serde_json::Number::from(bucket_timestamp)), 
                    serde_json::Value::String(aggregated_value.to_string())
                ]);
            }
            
            current_bucket_start += step;
        }
        
        if !aggregated_values.is_empty() {
            return aggregated_values;
        }
    }
    
    // If no real data, return current value at the end time
    let network_metrics = calculate_network_metrics(state);
    let current_value = match metric_type {
        "avg" => network_metrics.avg_delay,
        "max" => network_metrics.max_delay as f64,
        _ => network_metrics.avg_delay,
    };
    vec![[serde_json::Value::Number(serde_json::Number::from(end_time)), serde_json::Value::String(current_value.to_string())]]
}

/// Extract real historical queue utilization data from IntDB flows based on time range
fn extract_historical_queue_data(state: &AppState, metric_type: &str, start_time: i64, end_time: i64, step: i64) -> Vec<[serde_json::Value; 2]> {
    let mut time_values = Vec::new();
    
    // Get flows and extract real historical data from hops
    if let Ok(query_result) = state.engine.query(
        crate::storage::QueryBuilder::new().limit(100) // Get more flows for better time coverage
    ) {
        let flows = state.engine.get_flows(&query_result.flow_ids);
        
        // Collect all hop data within the time range
        for flow in flows {
            for hop in &flow.hops {
                if let Some(queue_util) = hop.metrics.queue_util {
                    let timestamp = hop.timestamp.timestamp();
                    
                    // Only include data within the requested time range
                    if timestamp >= start_time && timestamp <= end_time {
                        let queue_value = match metric_type {
                            "avg" => queue_util,
                            "max" => queue_util, // For max, we'll use the same value (simplification)
                            "congestion" => if queue_util > 0.7 { 1.0 } else { 0.0 },
                            _ => queue_util,
                        };
                        time_values.push([serde_json::Value::Number(serde_json::Number::from(timestamp)), serde_json::Value::String(queue_value.to_string())]);
                    }
                }
            }
        }
    }
    
    // Sort by timestamp
    time_values.sort_by_key(|v| v[0].as_i64().unwrap_or(0));
    
    // If we have real data, aggregate it into time buckets based on step
    if !time_values.is_empty() {
        let mut aggregated_values = Vec::new();
        let mut current_bucket_start = start_time;
        
        while current_bucket_start < end_time {
            let bucket_end = current_bucket_start + step;
            
            // Find all values in this time bucket
            let bucket_values: Vec<f64> = time_values.iter()
                .filter_map(|v| {
                    let timestamp = v[0].as_i64().unwrap_or(0);
                    if timestamp >= current_bucket_start && timestamp < bucket_end {
                        v[1].as_str().and_then(|s| s.parse::<f64>().ok())
                    } else {
                        None
                    }
                })
                .collect();
            
            // Calculate aggregated value for this bucket
            if !bucket_values.is_empty() {
                let aggregated_value = match metric_type {
                    "avg" => bucket_values.iter().sum::<f64>() / bucket_values.len() as f64,
                    "max" => bucket_values.iter().fold(0.0f64, |a, &b| a.max(b)),
                    "congestion" => bucket_values.iter().sum::<f64>() / bucket_values.len() as f64, // Average congestion ratio
                    _ => bucket_values.iter().sum::<f64>() / bucket_values.len() as f64,
                };
                
                // Use the middle of the time bucket as the timestamp
                let bucket_timestamp = current_bucket_start + step / 2;
                aggregated_values.push([
                    serde_json::Value::Number(serde_json::Number::from(bucket_timestamp)), 
                    serde_json::Value::String(aggregated_value.to_string())
                ]);
            }
            
            current_bucket_start += step;
        }
        
        if !aggregated_values.is_empty() {
            return aggregated_values;
        }
    }
    
    // If no real data, return current value at the end time
    let network_metrics = calculate_network_metrics(state);
    let current_value = match metric_type {
        "avg" => network_metrics.avg_queue_util,
        "max" => network_metrics.max_queue_util,
        "congestion" => network_metrics.congestion_ratio,
        _ => network_metrics.avg_queue_util,
    };
    vec![[serde_json::Value::Number(serde_json::Number::from(end_time)), serde_json::Value::String(current_value.to_string())]]
}

/// Calculate network metrics from all flows in the system
fn calculate_network_metrics(state: &AppState) -> NetworkMetrics {
    // Get all flows
    let flow_count = state.engine.flow_count();
    
    if flow_count == 0 {
        return NetworkMetrics {
            avg_delay: 0.0,
            max_delay: 0,
            avg_queue_util: 0.0,
            max_queue_util: 0.0,
            congestion_ratio: 0.0,
            unique_switches: 0,
            unique_paths: 0,
            avg_path_length: 0.0,
        };
    }
    
    // For now, get all flow IDs and collect metrics
    // This is a simplified implementation - in a real system you'd want to optimize this
    let mut all_delays: Vec<u64> = Vec::new();
    let mut all_queue_utils: Vec<f64> = Vec::new();
    let mut unique_switches = std::collections::HashSet::new();
    let mut path_lengths = Vec::new();
    
    // Since we don't have a direct way to iterate all flows, we'll use a query
    // to get recent flows and calculate metrics from them
    if let Ok(query_result) = state.engine.query(
        crate::storage::QueryBuilder::new().limit(1000) // Get up to 1000 flows
    ) {
        let flows = state.engine.get_flows(&query_result.flow_ids);
        
        for flow in flows {
            // Collect path length
            path_lengths.push(flow.path.switches.len() as f64);
            
            // Collect unique switches
            for switch in &flow.path.switches {
                unique_switches.insert(switch.clone());
            }
            
            // Collect metrics from hops
            for hop in &flow.hops {
                if let Some(delay) = hop.metrics.delay_ns {
                    all_delays.push(delay);
                }
                if let Some(queue_util) = hop.metrics.queue_util {
                    all_queue_utils.push(queue_util);
                }
            }
        }
    }
    
    // Calculate averages and maximums
    let avg_delay = if all_delays.is_empty() {
        0.0
    } else {
        all_delays.iter().sum::<u64>() as f64 / all_delays.len() as f64
    };
    
    let max_delay = all_delays.iter().max().copied().unwrap_or(0) as i32;
    
    let avg_queue_util = if all_queue_utils.is_empty() {
        0.0
    } else {
        all_queue_utils.iter().sum::<f64>() / all_queue_utils.len() as f64
    };
    
    let max_queue_util = all_queue_utils.iter().fold(0.0f64, |a, &b| a.max(b));
    
    // Calculate congestion ratio (percentage of hops with queue utilization > 0.7)
    let congested_hops = all_queue_utils.iter().filter(|&&util| util > 0.7).count();
    let congestion_ratio = if all_queue_utils.is_empty() {
        0.0
    } else {
        congested_hops as f64 / all_queue_utils.len() as f64
    };
    
    let avg_path_length = if path_lengths.is_empty() {
        0.0
    } else {
        path_lengths.iter().sum::<f64>() / path_lengths.len() as f64
    };
    
    NetworkMetrics {
        avg_delay,
        max_delay,
        avg_queue_util,
        max_queue_util,
        congestion_ratio,
        unique_switches: unique_switches.len(),
        unique_paths: flow_count, // Each flow represents a unique path in our current implementation
        avg_path_length,
    }
} 