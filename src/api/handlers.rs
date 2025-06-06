use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use crate::models::Flow;
use crate::storage::{StorageEngine, QueryBuilder};
use crate::api::{
    ApiError, ApiResult,
    InsertFlowRequest, InsertFlowResponse,
    FlowResponse, FlowsResponse,
    QueryRequest, QueryResponse,
    HealthResponse, StatsResponse,
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
        memory_usage_estimate: state.engine.flow_count() * 1024, // Rough estimate
    };
    
    Ok(Json(response))
}

/// Insert a new flow
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

/// Query flows
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
    
    let mut response: QueryResponse = query_result.into();
    
    // Include flow data for recent queries
    let flows = state.engine.get_flows(&response.flow_ids);
    response.flows = Some(flows);
    
    Ok(Json(response))
} 