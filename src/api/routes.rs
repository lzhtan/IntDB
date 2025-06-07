use axum::{
    routing::{get, post, delete},
    Router,
};

use crate::api::handlers::*;

/// Create the main application router
pub fn create_router(state: AppState) -> Router {
    Router::new()
        // Health and status endpoints
        .route("/health", get(health_check))
        .route("/stats", get(get_stats))
        .route("/metrics", get(prometheus_metrics))  // Prometheus metrics for Grafana
        
        // Standard Prometheus API endpoints for Grafana compatibility
        .route("/api/v1/query", get(prometheus_query))
        .route("/api/v1/query_range", get(prometheus_query_range))
        .route("/api/v1/label/__name__/values", get(prometheus_label_values))
        .route("/api/v1/labels", get(prometheus_labels))
        
        // Grafana integration endpoints
        .route("/grafana/query", post(grafana_query))
        
        // Legacy flow management endpoints
        .route("/flows", post(insert_flow))
        .route("/flows/:id", get(get_flow))
        .route("/flows/:id", delete(delete_flow))
        .route("/flows/batch", post(get_flows))
        
        // New spatiotemporal flow endpoints
        .route("/st-flows", post(insert_spatiotemporal_flow))
        .route("/st-flows/:id", get(get_spatiotemporal_flow))
        
        // Legacy query endpoints
        .route("/query", post(query_flows))
        
        // New spatiotemporal query endpoints
        .route("/st-query", post(query_spatiotemporal_flows))
        
        // Quick query endpoints for common use cases (legacy)
        .route("/quick/through/:switch_id", get(quick_query_through_switch))
        .route("/quick/path", post(quick_query_exact_path))
        .route("/quick/recent/:minutes", get(quick_query_recent))
        
        // Quick spatiotemporal query endpoints
        .route("/st-quick/spatial-region", post(quick_query_spatial_region))
        .route("/st-quick/spatial-flows", get(quick_query_spatial_flows))
        
        // Set the application state
        .with_state(state)
}

/// Create a minimal router for testing/development
pub fn create_minimal_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        
        // Basic legacy endpoints
        .route("/flows", post(insert_flow))
        .route("/flows/:id", get(get_flow))
        .route("/query", post(query_flows))
        
        // Basic spatiotemporal endpoints
        .route("/st-flows", post(insert_spatiotemporal_flow))
        .route("/st-flows/:id", get(get_spatiotemporal_flow))
        .route("/st-query", post(query_spatiotemporal_flows))
        
        .with_state(state)
} 