use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};


use crate::storage::StorageError;
use crate::models::FlowError;
use crate::api::ErrorResponse;

/// API error types
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),
    
    #[error("Flow error: {0}")]
    Flow(#[from] FlowError),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Bad request: {0}")]
    BadRequest(String),
    
    #[error("Internal server error: {0}")]
    Internal(String),
    
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message, details) = match &self {
            ApiError::Storage(StorageError::FlowNotFound(id)) => {
                (StatusCode::NOT_FOUND, "Flow not found", Some(id.clone()))
            }
            ApiError::Storage(StorageError::FlowAlreadyExists(id)) => {
                (StatusCode::CONFLICT, "Flow already exists", Some(id.clone()))
            }
            ApiError::Storage(StorageError::StorageFull) => {
                (StatusCode::INSUFFICIENT_STORAGE, "Storage capacity exceeded", None)
            }
            ApiError::Storage(StorageError::ReadOnly) => {
                (StatusCode::FORBIDDEN, "Database is read-only", None)
            }
            ApiError::Storage(StorageError::InvalidQuery(msg)) => {
                (StatusCode::BAD_REQUEST, "Invalid query", Some(msg.clone()))
            }
            ApiError::Flow(_) => {
                (StatusCode::BAD_REQUEST, "Invalid flow data", None)
            }
            ApiError::Validation(msg) => {
                (StatusCode::BAD_REQUEST, "Validation failed", Some(msg.clone()))
            }
            ApiError::NotFound(resource) => {
                (StatusCode::NOT_FOUND, "Resource not found", Some(resource.clone()))
            }
            ApiError::BadRequest(msg) => {
                (StatusCode::BAD_REQUEST, "Bad request", Some(msg.clone()))
            }
            ApiError::Internal(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error", Some(msg.clone()))
            }
            ApiError::Json(_) => {
                (StatusCode::BAD_REQUEST, "Invalid JSON", None)
            }
        };

        let body = Json(ErrorResponse {
            error: error_message.to_string(),
            message: self.to_string(),
            details,
        });

        (status, body).into_response()
    }
}

/// Result type for API handlers
pub type ApiResult<T> = Result<T, ApiError>;

impl ApiError {
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }
    
    pub fn not_found(resource: impl Into<String>) -> Self {
        Self::NotFound(resource.into())
    }
    
    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self::BadRequest(msg.into())
    }
    
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
} 