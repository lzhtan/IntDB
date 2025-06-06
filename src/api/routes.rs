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
        
        // Flow management endpoints
        .route("/flows", post(insert_flow))
        .route("/flows/:id", get(get_flow))
        .route("/flows/:id", delete(delete_flow))
        .route("/flows/batch", post(get_flows))
        
        // Query endpoints
        .route("/query", post(query_flows))
        
        // Quick query endpoints for common use cases
        .route("/quick/through/:switch_id", get(quick_query_through_switch))
        .route("/quick/path", post(quick_query_exact_path))
        .route("/quick/recent/:minutes", get(quick_query_recent))
        
        // Set the application state
        .with_state(state)
}

/// Create a minimal router for testing/development
pub fn create_minimal_router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health_check))
        .route("/flows", post(insert_flow))
        .route("/flows/:id", get(get_flow))
        .route("/query", post(query_flows))
        .with_state(state)
} 