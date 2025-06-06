use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::fmt;

/// Represents a network path as a sequence of switch identifiers
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NetworkPath {
    /// Ordered sequence of switch identifiers
    pub switches: Vec<String>,
    
    /// Cached hash of the path for efficient lookups
    #[serde(skip)]
    pub path_hash: Option<String>,
}

impl NetworkPath {
    /// Create a new network path
    pub fn new(switches: Vec<String>) -> Self {
        let mut path = Self {
            switches,
            path_hash: None,
        };
        path.compute_hash();
        path
    }
    
    /// Create path from a slice of switch IDs
    pub fn from_switches(switches: &[&str]) -> Self {
        Self::new(switches.iter().map(|s| s.to_string()).collect())
    }
    
    /// Get the length of the path (number of hops)
    pub fn length(&self) -> usize {
        self.switches.len()
    }
    
    /// Check if path is empty
    pub fn is_empty(&self) -> bool {
        self.switches.is_empty()
    }
    
    /// Get the hash of this path
    pub fn hash(&self) -> String {
        if let Some(ref hash) = self.path_hash {
            hash.clone()
        } else {
            // This should not happen if constructed properly, but handle gracefully
            let mut mutable_self = self.clone();
            mutable_self.compute_hash();
            mutable_self.path_hash.unwrap()
        }
    }
    
    /// Compute and cache the hash of this path
    fn compute_hash(&mut self) {
        let mut hasher = Sha256::new();
        for switch in &self.switches {
            hasher.update(switch.as_bytes());
            hasher.update(b"->");
        }
        let result = hasher.finalize();
        self.path_hash = Some(format!("{:x}", result));
    }
    
    /// Check if this path contains the given sub-path
    pub fn contains_subpath(&self, subpath: &[String]) -> bool {
        if subpath.is_empty() {
            return true;
        }
        if subpath.len() > self.switches.len() {
            return false;
        }
        
        self.switches.windows(subpath.len()).any(|window| window == subpath)
    }
    
    /// Check if this path starts with the given prefix
    pub fn starts_with(&self, prefix: &[String]) -> bool {
        if prefix.len() > self.switches.len() {
            return false;
        }
        self.switches.starts_with(prefix)
    }
    
    /// Check if this path ends with the given suffix
    pub fn ends_with(&self, suffix: &[String]) -> bool {
        if suffix.len() > self.switches.len() {
            return false;
        }
        self.switches.ends_with(suffix)
    }
    
    /// Get a sub-path from start_index to end_index (exclusive)
    pub fn subpath(&self, start_index: usize, end_index: usize) -> Option<NetworkPath> {
        if start_index >= end_index || end_index > self.switches.len() {
            return None;
        }
        Some(NetworkPath::new(self.switches[start_index..end_index].to_vec()))
    }
    
    /// Get the first switch in the path
    pub fn source(&self) -> Option<&String> {
        self.switches.first()
    }
    
    /// Get the last switch in the path
    pub fn destination(&self) -> Option<&String> {
        self.switches.last()
    }
}

impl fmt::Display for NetworkPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.switches.join(" -> "))
    }
}

impl From<Vec<String>> for NetworkPath {
    fn from(switches: Vec<String>) -> Self {
        Self::new(switches)
    }
}

impl From<Vec<&str>> for NetworkPath {
    fn from(switches: Vec<&str>) -> Self {
        Self::from_switches(&switches)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_creation() {
        let path = NetworkPath::from_switches(&["s1", "s2", "s3"]);
        assert_eq!(path.length(), 3);
        assert!(!path.is_empty());
        assert_eq!(path.source(), Some(&"s1".to_string()));
        assert_eq!(path.destination(), Some(&"s3".to_string()));
    }

    #[test]
    fn test_path_hash() {
        let path1 = NetworkPath::from_switches(&["s1", "s2", "s3"]);
        let path2 = NetworkPath::from_switches(&["s1", "s2", "s3"]);
        let path3 = NetworkPath::from_switches(&["s1", "s2", "s4"]);
        
        assert_eq!(path1.hash(), path2.hash());
        assert_ne!(path1.hash(), path3.hash());
    }

    #[test]
    fn test_subpath_contains() {
        let path = NetworkPath::from_switches(&["s1", "s2", "s3", "s4"]);
        
        assert!(path.contains_subpath(&["s2".to_string(), "s3".to_string()]));
        assert!(path.contains_subpath(&["s1".to_string(), "s2".to_string(), "s3".to_string()]));
        assert!(!path.contains_subpath(&["s2".to_string(), "s4".to_string()]));
        assert!(path.contains_subpath(&[])); // Empty subpath should return true
    }

    #[test]
    fn test_path_prefix_suffix() {
        let path = NetworkPath::from_switches(&["s1", "s2", "s3", "s4"]);
        
        assert!(path.starts_with(&["s1".to_string(), "s2".to_string()]));
        assert!(!path.starts_with(&["s2".to_string(), "s3".to_string()]));
        
        assert!(path.ends_with(&["s3".to_string(), "s4".to_string()]));
        assert!(!path.ends_with(&["s2".to_string(), "s3".to_string()]));
    }

    #[test]
    fn test_subpath_extraction() {
        let path = NetworkPath::from_switches(&["s1", "s2", "s3", "s4"]);
        
        let sub = path.subpath(1, 3).unwrap();
        assert_eq!(sub.switches, vec!["s2", "s3"]);
        
        assert!(path.subpath(2, 1).is_none()); // Invalid range
        assert!(path.subpath(0, 10).is_none()); // Out of bounds
    }

    #[test]
    fn test_path_display() {
        let path = NetworkPath::from_switches(&["s1", "s2", "s3"]);
        assert_eq!(path.to_string(), "s1 -> s2 -> s3");
    }

    #[test]
    fn test_path_serialization() {
        let path = NetworkPath::from_switches(&["s1", "s2", "s3"]);
        let json = serde_json::to_string(&path).unwrap();
        let deserialized: NetworkPath = serde_json::from_str(&json).unwrap();
        
        // Note: hash is not serialized, so we need to recompute it
        assert_eq!(path.switches, deserialized.switches);
        assert_eq!(path.hash(), deserialized.hash());
    }
} 