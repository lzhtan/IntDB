//! # IntDB - Spatiotemporal Database for In-band Network Telemetry
//!
//! IntDB is a specialized database designed for storing and querying In-band Network Telemetry (INT) data.
//! It provides native support for path semantics and temporal relationships in network flows.

pub mod models;
pub mod storage;
pub mod api;

// Re-export commonly used types
pub use models::*;
pub use storage::{StorageEngine, QueryBuilder, QueryResult};
pub use api::{create_router, create_minimal_router, AppState};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
} 