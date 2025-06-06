use std::collections::{BTreeMap, BTreeSet, HashMap};
use chrono::{DateTime, Utc};
use crate::models::{NetworkPath, Flow};

/// Path prefix tree for efficient path-based queries
#[derive(Debug, Clone)]
pub struct PathIndex {
    /// Maps path hash to set of flow IDs
    exact_paths: HashMap<String, BTreeSet<String>>,
    
    /// Maps individual switches to flow IDs that pass through them
    switch_flows: HashMap<String, BTreeSet<String>>,
    
    /// Maps path prefixes to flow IDs
    prefix_index: HashMap<String, BTreeSet<String>>,
}

impl PathIndex {
    /// Create a new empty path index
    pub fn new() -> Self {
        Self {
            exact_paths: HashMap::new(),
            switch_flows: HashMap::new(),
            prefix_index: HashMap::new(),
        }
    }
    
    /// Add a flow to the path index
    pub fn add_flow(&mut self, flow: &Flow) {
        let flow_id = flow.flow_id.clone();
        let path_hash = flow.path_hash();
        
        // Add to exact path index
        self.exact_paths
            .entry(path_hash)
            .or_insert_with(BTreeSet::new)
            .insert(flow_id.clone());
        
        // Add to switch index
        for switch in &flow.path.switches {
            self.switch_flows
                .entry(switch.clone())
                .or_insert_with(BTreeSet::new)
                .insert(flow_id.clone());
        }
        
        // Add to prefix index
        for i in 1..=flow.path.switches.len() {
            let prefix = flow.path.switches[..i].join("->");
            self.prefix_index
                .entry(prefix)
                .or_insert_with(BTreeSet::new)
                .insert(flow_id.clone());
        }
    }
    
    /// Remove a flow from the path index
    pub fn remove_flow(&mut self, flow: &Flow) {
        let flow_id = &flow.flow_id;
        let path_hash = flow.path_hash();
        
        // Remove from exact path index
        if let Some(flows) = self.exact_paths.get_mut(&path_hash) {
            flows.remove(flow_id);
            if flows.is_empty() {
                self.exact_paths.remove(&path_hash);
            }
        }
        
        // Remove from switch index
        for switch in &flow.path.switches {
            if let Some(flows) = self.switch_flows.get_mut(switch) {
                flows.remove(flow_id);
                if flows.is_empty() {
                    self.switch_flows.remove(switch);
                }
            }
        }
        
        // Remove from prefix index
        for i in 1..=flow.path.switches.len() {
            let prefix = flow.path.switches[..i].join("->");
            if let Some(flows) = self.prefix_index.get_mut(&prefix) {
                flows.remove(flow_id);
                if flows.is_empty() {
                    self.prefix_index.remove(&prefix);
                }
            }
        }
    }
    
    /// Find flows with exact path match
    pub fn find_exact_path(&self, path: &NetworkPath) -> BTreeSet<String> {
        let path_hash = path.hash();
        self.exact_paths.get(&path_hash).cloned().unwrap_or_default()
    }
    
    /// Find flows that pass through a specific switch
    pub fn find_flows_through_switch(&self, switch_id: &str) -> BTreeSet<String> {
        self.switch_flows.get(switch_id).cloned().unwrap_or_default()
    }
    
    /// Find flows that contain the given path as a subpath
    pub fn find_flows_containing_path(&self, path: &[String]) -> BTreeSet<String> {
        if path.is_empty() {
            return BTreeSet::new();
        }
        
        let search_key = path.join("->");
        let mut result = BTreeSet::new();
        
        // Look for exact prefix match
        if let Some(flows) = self.prefix_index.get(&search_key) {
            result.extend(flows.clone());
        }
        
        // Also look for flows where this appears as a substring
        for (prefix, flows) in &self.prefix_index {
            if prefix.contains(&search_key) {
                result.extend(flows.clone());
            }
        }
        
        result
    }
    
    /// Find flows that start with the given path prefix
    pub fn find_flows_with_prefix(&self, prefix: &[String]) -> BTreeSet<String> {
        if prefix.is_empty() {
            return BTreeSet::new();
        }
        
        let prefix_key = prefix.join("->");
        let mut result = BTreeSet::new();
        
        for (stored_prefix, flows) in &self.prefix_index {
            if stored_prefix.starts_with(&prefix_key) {
                result.extend(flows.clone());
            }
        }
        
        result
    }
    
    /// Get statistics about the index
    pub fn stats(&self) -> IndexStats {
        IndexStats {
            unique_paths: self.exact_paths.len(),
            unique_switches: self.switch_flows.len(),
            prefix_entries: self.prefix_index.len(),
            total_flow_refs: self.exact_paths.values().map(|s| s.len()).sum(),
        }
    }
}

impl Default for PathIndex {
    fn default() -> Self {
        Self::new()
    }
}

/// Time-based index for temporal queries
#[derive(Debug, Clone)]
pub struct TimeIndex {
    /// Maps time buckets to flow IDs
    time_buckets: BTreeMap<DateTime<Utc>, BTreeSet<String>>,
    
    /// Bucket size in seconds
    bucket_size_secs: i64,
}

impl TimeIndex {
    /// Create a new time index with given bucket size
    pub fn new(bucket_size_secs: i64) -> Self {
        Self {
            time_buckets: BTreeMap::new(),
            bucket_size_secs,
        }
    }
    
    /// Create a new time index with 1-minute buckets
    pub fn with_minute_buckets() -> Self {
        Self::new(60)
    }
    
    /// Create a new time index with 5-minute buckets
    pub fn with_five_minute_buckets() -> Self {
        Self::new(300)
    }
    
    /// Get the bucket timestamp for a given time
    fn get_bucket(&self, timestamp: DateTime<Utc>) -> DateTime<Utc> {
        let bucket_timestamp = (timestamp.timestamp() / self.bucket_size_secs) * self.bucket_size_secs;
        DateTime::from_timestamp(bucket_timestamp, 0).unwrap_or(timestamp)
    }
    
    /// Add a flow to the time index
    pub fn add_flow(&mut self, flow: &Flow) {
        let bucket = self.get_bucket(flow.start_time);
        self.time_buckets
            .entry(bucket)
            .or_insert_with(BTreeSet::new)
            .insert(flow.flow_id.clone());
    }
    
    /// Remove a flow from the time index
    pub fn remove_flow(&mut self, flow: &Flow) {
        let bucket = self.get_bucket(flow.start_time);
        if let Some(flows) = self.time_buckets.get_mut(&bucket) {
            flows.remove(&flow.flow_id);
            if flows.is_empty() {
                self.time_buckets.remove(&bucket);
            }
        }
    }
    
    /// Find flows within a time range
    pub fn find_flows_in_range(
        &self,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> BTreeSet<String> {
        let start_bucket = self.get_bucket(start_time);
        let end_bucket = self.get_bucket(end_time);
        
        let mut result = BTreeSet::new();
        
        for (bucket_time, flows) in self.time_buckets.range(start_bucket..=end_bucket) {
            result.extend(flows.clone());
        }
        
        result
    }
    
    /// Find flows after a specific time
    pub fn find_flows_after(&self, timestamp: DateTime<Utc>) -> BTreeSet<String> {
        let start_bucket = self.get_bucket(timestamp);
        let mut result = BTreeSet::new();
        
        for (_, flows) in self.time_buckets.range(start_bucket..) {
            result.extend(flows.clone());
        }
        
        result
    }
    
    /// Find flows before a specific time
    pub fn find_flows_before(&self, timestamp: DateTime<Utc>) -> BTreeSet<String> {
        let end_bucket = self.get_bucket(timestamp);
        let mut result = BTreeSet::new();
        
        for (_, flows) in self.time_buckets.range(..end_bucket) {
            result.extend(flows.clone());
        }
        
        result
    }
    
    /// Get the earliest time bucket
    pub fn earliest_time(&self) -> Option<DateTime<Utc>> {
        self.time_buckets.keys().next().copied()
    }
    
    /// Get the latest time bucket
    pub fn latest_time(&self) -> Option<DateTime<Utc>> {
        self.time_buckets.keys().next_back().copied()
    }
    
    /// Get statistics about the time index
    pub fn stats(&self) -> TimeIndexStats {
        TimeIndexStats {
            bucket_count: self.time_buckets.len(),
            bucket_size_secs: self.bucket_size_secs,
            earliest_time: self.earliest_time(),
            latest_time: self.latest_time(),
            total_flow_refs: self.time_buckets.values().map(|s| s.len()).sum(),
        }
    }
}

impl Default for TimeIndex {
    fn default() -> Self {
        Self::with_minute_buckets()
    }
}

/// Statistics about the path index
#[derive(Debug, Clone)]
pub struct IndexStats {
    pub unique_paths: usize,
    pub unique_switches: usize,
    pub prefix_entries: usize,
    pub total_flow_refs: usize,
}

/// Statistics about the time index
#[derive(Debug, Clone)]
pub struct TimeIndexStats {
    pub bucket_count: usize,
    pub bucket_size_secs: i64,
    pub earliest_time: Option<DateTime<Utc>>,
    pub latest_time: Option<DateTime<Utc>>,
    pub total_flow_refs: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Hop, TelemetryMetrics};
    use chrono::Utc;

    fn create_test_flow(flow_id: &str, switches: &[&str], start_time: DateTime<Utc>) -> Flow {
        let hops: Vec<Hop> = switches
            .iter()
            .enumerate()
            .map(|(i, switch)| {
                Hop::new(
                    i as u32,
                    switch.to_string(),
                    start_time + chrono::Duration::milliseconds(i as i64 * 10),
                    TelemetryMetrics::with_basic(0.1 * i as f64, 100 * i as u64),
                )
            })
            .collect();
        
        Flow::new(flow_id.to_string(), hops).unwrap()
    }

    #[test]
    fn test_path_index() {
        let mut index = PathIndex::new();
        let now = Utc::now();
        
        let flow1 = create_test_flow("flow1", &["s1", "s2", "s3"], now);
        let flow2 = create_test_flow("flow2", &["s1", "s2", "s4"], now);
        let flow3 = create_test_flow("flow3", &["s2", "s3", "s4"], now);
        
        index.add_flow(&flow1);
        index.add_flow(&flow2);
        index.add_flow(&flow3);
        
        // Test exact path matching
        assert_eq!(index.find_exact_path(&flow1.path).len(), 1);
        assert!(index.find_exact_path(&flow1.path).contains("flow1"));
        
        // Test switch-based queries
        let flows_through_s2 = index.find_flows_through_switch("s2");
        assert_eq!(flows_through_s2.len(), 3);
        
        let flows_through_s4 = index.find_flows_through_switch("s4");
        assert_eq!(flows_through_s4.len(), 2);
        
        // Test prefix queries
        let flows_with_s1_s2 = index.find_flows_with_prefix(&["s1".to_string(), "s2".to_string()]);
        assert_eq!(flows_with_s1_s2.len(), 2);
        assert!(flows_with_s1_s2.contains("flow1"));
        assert!(flows_with_s1_s2.contains("flow2"));
        
        // Test statistics
        let stats = index.stats();
        assert_eq!(stats.unique_paths, 3);
        assert!(stats.unique_switches >= 4);
    }

    #[test]
    fn test_time_index() {
        let mut index = TimeIndex::with_minute_buckets();
        let base_time = DateTime::from_timestamp(1640995200, 0).unwrap(); // 2022-01-01 00:00:00 UTC
        
        let flow1 = create_test_flow("flow1", &["s1", "s2"], base_time);
        let flow2 = create_test_flow("flow2", &["s2", "s3"], base_time + chrono::Duration::seconds(30));
        let flow3 = create_test_flow("flow3", &["s3", "s4"], base_time + chrono::Duration::minutes(2));
        
        index.add_flow(&flow1);
        index.add_flow(&flow2);
        index.add_flow(&flow3);
        
        // Test range queries
        let flows_in_first_minute = index.find_flows_in_range(
            base_time,
            base_time + chrono::Duration::minutes(1),
        );
        assert_eq!(flows_in_first_minute.len(), 2);
        assert!(flows_in_first_minute.contains("flow1"));
        assert!(flows_in_first_minute.contains("flow2"));
        
        // Test after/before queries
        let flows_after = index.find_flows_after(base_time + chrono::Duration::minutes(1));
        assert_eq!(flows_after.len(), 1);
        assert!(flows_after.contains("flow3"));
        
        let flows_before = index.find_flows_before(base_time + chrono::Duration::minutes(2));
        assert_eq!(flows_before.len(), 2);
        
        // Test statistics
        let stats = index.stats();
        assert!(stats.bucket_count >= 2);
        assert_eq!(stats.bucket_size_secs, 60);
        assert!(stats.earliest_time.is_some());
        assert!(stats.latest_time.is_some());
    }

    #[test]
    fn test_index_removal() {
        let mut path_index = PathIndex::new();
        let mut time_index = TimeIndex::new(60);
        let now = Utc::now();
        
        let flow = create_test_flow("flow1", &["s1", "s2", "s3"], now);
        
        // Add flow
        path_index.add_flow(&flow);
        time_index.add_flow(&flow);
        
        assert_eq!(path_index.find_flows_through_switch("s2").len(), 1);
        assert_eq!(time_index.find_flows_after(now - chrono::Duration::minutes(1)).len(), 1);
        
        // Remove flow
        path_index.remove_flow(&flow);
        time_index.remove_flow(&flow);
        
        assert_eq!(path_index.find_flows_through_switch("s2").len(), 0);
        assert_eq!(time_index.find_flows_after(now - chrono::Duration::minutes(1)).len(), 0);
    }
} 