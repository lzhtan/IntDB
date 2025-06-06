use std::collections::{BTreeSet, HashMap};
use std::sync::{Arc, RwLock};
use chrono::{DateTime, Utc};

use crate::models::Flow;
use crate::storage::{PathIndex, TimeIndex, QueryBuilder, QueryResult, PathCondition, TimeCondition};

/// IntDB storage engine configuration
#[derive(Debug, Clone)]
pub struct EngineConfig {
    /// Time bucket size for time index in seconds
    pub time_bucket_size: i64,
    
    /// Maximum number of flows to keep in memory
    pub max_flows: Option<usize>,
    
    /// Automatically clean up old flows after this duration (in hours)
    pub auto_cleanup_hours: Option<i64>,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            time_bucket_size: 60, // 1 minute buckets
            max_flows: Some(1_000_000), // 1M flows
            auto_cleanup_hours: Some(24), // Keep 24 hours
        }
    }
}

/// Storage engine error types
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("Flow not found: {0}")]
    FlowNotFound(String),
    
    #[error("Flow already exists: {0}")]
    FlowAlreadyExists(String),
    
    #[error("Storage full: reached maximum capacity")]
    StorageFull,
    
    #[error("Invalid query: {0}")]
    InvalidQuery(String),
    
    #[error("Engine is read-only")]
    ReadOnly,
}

/// Thread-safe IntDB storage engine
#[derive(Debug)]
pub struct StorageEngine {
    /// Main flow storage
    flows: Arc<RwLock<HashMap<String, Flow>>>,
    
    /// Path-based index
    path_index: Arc<RwLock<PathIndex>>,
    
    /// Time-based index
    time_index: Arc<RwLock<TimeIndex>>,
    
    /// Engine configuration
    config: EngineConfig,
    
    /// Whether engine is read-only
    read_only: bool,
}

impl StorageEngine {
    /// Create a new storage engine with default configuration
    pub fn new() -> Self {
        Self::with_config(EngineConfig::default())
    }
    
    /// Create a new storage engine with custom configuration
    pub fn with_config(config: EngineConfig) -> Self {
        Self {
            flows: Arc::new(RwLock::new(HashMap::new())),
            path_index: Arc::new(RwLock::new(PathIndex::new())),
            time_index: Arc::new(RwLock::new(TimeIndex::new(config.time_bucket_size))),
            config,
            read_only: false,
        }
    }

    /// Insert a new flow into the storage
    pub fn insert_flow(&self, flow: Flow) -> Result<(), StorageError> {
        if self.read_only {
            return Err(StorageError::ReadOnly);
        }
        
        let flow_id = flow.flow_id.clone();
        
        // Check if flow already exists
        {
            let flows = self.flows.read().unwrap();
            if flows.contains_key(&flow_id) {
                return Err(StorageError::FlowAlreadyExists(flow_id));
            }
        }
        
        // Check capacity
        if let Some(max_flows) = self.config.max_flows {
            let flows = self.flows.read().unwrap();
            if flows.len() >= max_flows {
                return Err(StorageError::StorageFull);
            }
        }
        
        // Insert into main storage
        {
            let mut flows = self.flows.write().unwrap();
            flows.insert(flow_id.clone(), flow.clone());
        }
        
        // Update indexes
        {
            let mut path_index = self.path_index.write().unwrap();
            path_index.add_flow(&flow);
        }
        
        {
            let mut time_index = self.time_index.write().unwrap();
            time_index.add_flow(&flow);
        }
        
        Ok(())
    }
    
    /// Get a flow by ID
    pub fn get_flow(&self, flow_id: &str) -> Option<Flow> {
        let flows = self.flows.read().unwrap();
        flows.get(flow_id).cloned()
    }

    /// Execute a query
    pub fn query(&self, query: QueryBuilder) -> Result<QueryResult, StorageError> {
        // Get candidate flow IDs from indexes
        let candidate_ids = self.get_candidate_flows(&query)?;
        
        // Apply all conditions
        let flows_guard = self.flows.read().unwrap();
        let mut matching_flows: Vec<_> = candidate_ids
            .into_iter()
            .filter_map(|flow_id| {
                flows_guard.get(&flow_id).map(|flow| (flow_id, flow))
            })
            .filter(|(_, flow)| self.matches_all_conditions(flow, &query))
            .collect();
        
        // Sort by start time (most recent first)
        matching_flows.sort_by(|a, b| b.1.start_time.cmp(&a.1.start_time));
        
        let total_count = matching_flows.len();
        
        // Apply pagination
        let (limit, skip) = query.pagination();
        let skip = skip.unwrap_or(0);
        
        let flow_ids: Vec<String> = matching_flows
            .into_iter()
            .skip(skip)
            .take(limit.unwrap_or(usize::MAX))
            .map(|(flow_id, _)| flow_id)
            .collect();
        
        Ok(QueryResult::new(flow_ids, total_count, limit))
    }

    /// Get candidate flow IDs from indexes
    fn get_candidate_flows(&self, query: &QueryBuilder) -> Result<BTreeSet<String>, StorageError> {
        let (path_conditions, time_conditions, _) = query.conditions();
        
        let mut candidates: Option<BTreeSet<String>> = None;
        
        // Apply path-based index optimizations
        for condition in path_conditions {
            let path_candidates = self.get_path_candidates(condition)?;
            
            candidates = match candidates {
                None => Some(path_candidates),
                Some(existing) => Some(existing.intersection(&path_candidates).cloned().collect()),
            };
            
            // Early exit if no candidates
            if candidates.as_ref().map_or(false, |c| c.is_empty()) {
                return Ok(BTreeSet::new());
            }
        }
        
        // Apply time-based index optimizations
        for condition in time_conditions {
            let time_candidates = self.get_time_candidates(condition)?;
            
            candidates = match candidates {
                None => Some(time_candidates),
                Some(existing) => Some(existing.intersection(&time_candidates).cloned().collect()),
            };
            
            // Early exit if no candidates
            if candidates.as_ref().map_or(false, |c| c.is_empty()) {
                return Ok(BTreeSet::new());
            }
        }
        
        // If no index-based conditions, return all flows
        Ok(candidates.unwrap_or_else(|| {
            let flows = self.flows.read().unwrap();
            flows.keys().cloned().collect()
        }))
    }

    /// Get candidate flows from path index
    fn get_path_candidates(&self, condition: &PathCondition) -> Result<BTreeSet<String>, StorageError> {
        let path_index = self.path_index.read().unwrap();
        
        Ok(match condition {
            PathCondition::ExactPath(path) => path_index.find_exact_path(path),
            PathCondition::ThroughSwitch(switch_id) => path_index.find_flows_through_switch(switch_id),
            PathCondition::ContainsPath(subpath) => path_index.find_flows_containing_path(subpath),
            PathCondition::StartsWith(prefix) => path_index.find_flows_with_prefix(prefix),
            // For conditions that can't be optimized by index, return all flows
            _ => {
                let flows = self.flows.read().unwrap();
                flows.keys().cloned().collect()
            }
        })
    }
    
    /// Get candidate flows from time index
    fn get_time_candidates(&self, condition: &TimeCondition) -> Result<BTreeSet<String>, StorageError> {
        let time_index = self.time_index.read().unwrap();
        let now = Utc::now();
        
        Ok(match condition {
            TimeCondition::After(time) => time_index.find_flows_after(*time),
            TimeCondition::Before(time) => time_index.find_flows_before(*time),
            TimeCondition::InRange(start, end) => time_index.find_flows_in_range(*start, *end),
            TimeCondition::WithinLast(seconds) => {
                time_index.find_flows_after(now - chrono::Duration::seconds(*seconds))
            }
            TimeCondition::WithinLastMinutes(minutes) => {
                time_index.find_flows_after(now - chrono::Duration::minutes(*minutes))
            }
            TimeCondition::WithinLastHours(hours) => {
                time_index.find_flows_after(now - chrono::Duration::hours(*hours))
            }
        })
    }
    
    /// Check if a flow matches all conditions
    fn matches_all_conditions(&self, flow: &Flow, query: &QueryBuilder) -> bool {
        let (path_conditions, time_conditions, metric_conditions) = query.conditions();
        
        // Check path conditions
        for condition in path_conditions {
            if !condition.matches(flow) {
                return false;
            }
        }
        
        // Check time conditions
        for condition in time_conditions {
            if !condition.matches(flow) {
                return false;
            }
        }
        
        // Check metric conditions
        for condition in metric_conditions {
            if !condition.matches(flow) {
                return false;
            }
        }
        
        true
    }

    /// Get flows by IDs
    pub fn get_flows(&self, flow_ids: &[String]) -> Vec<Flow> {
        let flows = self.flows.read().unwrap();
        flow_ids
            .iter()
            .filter_map(|id| flows.get(id).cloned())
            .collect()
    }

    /// Get the number of flows currently stored
    pub fn flow_count(&self) -> usize {
        let flows = self.flows.read().unwrap();
        flows.len()
    }
}

impl Default for StorageEngine {
    fn default() -> Self {
        Self::new()
    }
} 