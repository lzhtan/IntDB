//! Simple demonstration of INT data formats in IntDB
//! Shows how to create and work with both legacy and spatiotemporal flows

use chrono::Utc;
use intdb::models::*;
use intdb::storage::StorageEngine;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌐 IntDB INT Data Format Demo");
    println!("=============================");
    
    // Create storage engine
    let engine = StorageEngine::new();
    
    // Demo 1: Legacy Flow format (backward compatibility)
    println!("\n📊 Demo 1: Legacy Flow Format");
    let legacy_flow = create_legacy_int_flow()?;
    engine.insert_flow(legacy_flow.clone())?;
    println!("✅ Created legacy flow: {}", legacy_flow.flow_id);
    println!("   Path: {}", legacy_flow.path);
    println!("   Hops: {}", legacy_flow.hops.len());
    println!("   Total delay: {:?} ns", legacy_flow.total_delay());
    
    // Demo 2: Spatiotemporal Flow (logical path only)
    println!("\n🔀 Demo 2: Spatiotemporal Flow (Logical Path Only)");
    let logical_st_flow = create_logical_spatiotemporal_flow();
    let legacy_converted = convert_st_to_legacy(&logical_st_flow)?;
    engine.insert_flow(legacy_converted)?;
    println!("✅ Created spatiotemporal flow: {}", logical_st_flow.flow_id);
    println!("   Logical path: {:?}", logical_st_flow.spatial_metadata.logical_path);
    println!("   Has spatial info: {}", logical_st_flow.spatial_metadata.has_spatial_info);
    println!("   Windows: {}", logical_st_flow.spatiotemporal_windows.len());
    
    // Demo 3: Spatiotemporal Flow (with spatial coordinates)
    println!("\n🗺️  Demo 3: Spatiotemporal Flow (With Spatial Info)");
    let spatial_st_flow = create_spatial_spatiotemporal_flow();
    let legacy_converted = convert_st_to_legacy(&spatial_st_flow)?;
    engine.insert_flow(legacy_converted)?;
    println!("✅ Created spatial flow: {}", spatial_st_flow.flow_id);
    println!("   Logical path: {:?}", spatial_st_flow.spatial_metadata.logical_path);
    println!("   Has spatial info: {}", spatial_st_flow.spatial_metadata.has_spatial_info);
    if let Some(extent) = &spatial_st_flow.spatial_metadata.spatial_extent {
        println!("   Spatial extent: ({}, {}) to ({}, {})", 
            extent.min_x, extent.min_y, extent.max_x, extent.max_y);
    }
    
    // Demo 4: JSON serialization examples
    println!("\n📄 Demo 4: JSON Serialization");
    demonstrate_json_formats()?;
    
    // Demo 5: Query examples
    println!("\n🔍 Demo 5: Query Examples");
    demonstrate_queries(&engine)?;
    
    println!("\n✨ Demo completed successfully!");
    println!("💡 Key benefits of the new format:");
    println!("   - Supports both logical and spatial network topologies");
    println!("   - Time-windowed data for continuous INT streams");
    println!("   - Quality metrics for data completeness tracking");
    println!("   - Backward compatibility with legacy flows");
    println!("   - Optimized for spatiotemporal queries");
    
    Ok(())
}

/// Create a legacy flow representing INT data
fn create_legacy_int_flow() -> Result<Flow, FlowError> {
    let now = Utc::now();
    
    // Simulate INT telemetry from a 3-hop path
    let hops = vec![
        Hop::with_basic_metrics(
            0, 
            "spine1".to_string(), 
            now, 
            0.15,  // 15% queue utilization
            250    // 250ns delay
        ),
        Hop::with_basic_metrics(
            1, 
            "leaf2".to_string(), 
            now + chrono::Duration::microseconds(250), 
            0.45,  // 45% queue utilization  
            380    // 380ns delay
        ),
        Hop::with_basic_metrics(
            2, 
            "server3".to_string(), 
            now + chrono::Duration::microseconds(630), 
            0.25,  // 25% queue utilization
            200    // 200ns delay
        ),
    ];
    
    Flow::new("int_flow_legacy_001".to_string(), hops)
}

/// Create a spatiotemporal flow with logical path only
fn create_logical_spatiotemporal_flow() -> SpatiotemporalFlow {
    let logical_path = vec![
        "spine1".to_string(), 
        "leaf2".to_string(), 
        "server3".to_string()
    ];
    
    let mut st_flow = SpatiotemporalFlow::new_logical(
        "int_flow_st_logical_001".to_string(), 
        logical_path
    );
    
    // Add a time window with INT telemetry data
    let now = Utc::now();
    let window = SpatiotemporalWindow {
        st_window_id: format!("window_{}_{}", now.timestamp(), "logical"),
        temporal_bounds: TemporalBounds {
            start: now,
            end: now + chrono::Duration::seconds(60),
        },
        spatial_bounds: None,
        packet_count: 1250, // Number of packets in this window
        quality_metrics: QualityMetrics {
            path_completeness: 0.98,  // 98% of expected hops received
            spatial_coverage: None,   // No spatial info
            temporal_continuity: 0.95, // 95% temporal continuity
        },
        spatial_hops: vec![
            create_spatial_hop(0, "spine1", None, now, 0.15, 250),
            create_spatial_hop(1, "leaf2", None, now + chrono::Duration::microseconds(250), 0.45, 380),
            create_spatial_hop(2, "server3", None, now + chrono::Duration::microseconds(630), 0.25, 200),
        ],
    };
    
    st_flow.add_window(window);
    st_flow
}

/// Create a spatiotemporal flow with spatial coordinates
fn create_spatial_spatiotemporal_flow() -> SpatiotemporalFlow {
    let logical_path = vec![
        "dc1_spine1".to_string(), 
        "dc1_leaf2".to_string(), 
        "dc1_server3".to_string()
    ];
    
    // Define physical topology coordinates
    let topology_coordinates = vec![
        TopologyCoordinate {
            switch: "dc1_spine1".to_string(),
            topo_x: 100.0,
            topo_y: 300.0, // Spine layer
            zone: Some("spine_layer".to_string()),
        },
        TopologyCoordinate {
            switch: "dc1_leaf2".to_string(),
            topo_x: 200.0,
            topo_y: 200.0, // Leaf layer
            zone: Some("leaf_layer".to_string()),
        },
        TopologyCoordinate {
            switch: "dc1_server3".to_string(),
            topo_x: 250.0,
            topo_y: 100.0, // Server layer
            zone: Some("server_layer".to_string()),
        },
    ];
    
    let spatial_metadata = SpatialMetadata::with_spatial_info(
        logical_path,
        topology_coordinates,
        Some("LINESTRING(100 300, 200 200, 250 100)".to_string()),
    );
    
    let temporal_metadata = TemporalMetadata {
        flow_state: FlowState::Active,
        creation_time: Utc::now(),
        last_update: Utc::now(),
        window_duration: Some(60000), // 60 second windows
        retention_policy: Some("7d".to_string()),
    };
    
    let mut st_flow = SpatiotemporalFlow {
        flow_id: "int_flow_st_spatial_001".to_string(),
        spatial_metadata,
        temporal_metadata,
        spatiotemporal_windows: Vec::new(),
        spatiotemporal_indices: SpatiotemporalIndices {
            rtree_index: Some("rtree_dc1_001".to_string()),
            temporal_btree: "btree_temporal_001".to_string(),
            st_compound_index: Some("st_compound_001".to_string()),
            logical_path_trie: "trie_path_001".to_string(),
        },
    };
    
    // Add a window with spatial INT data
    let now = Utc::now();
    let window = SpatiotemporalWindow {
        st_window_id: format!("window_{}_{}", now.timestamp(), "spatial"),
        temporal_bounds: TemporalBounds {
            start: now,
            end: now + chrono::Duration::seconds(60),
        },
        spatial_bounds: Some(SpatialExtent {
            min_x: 100.0,
            min_y: 100.0,
            max_x: 250.0,
            max_y: 300.0,
        }),
        packet_count: 1800,
        quality_metrics: QualityMetrics {
            path_completeness: 1.0,    // Complete path
            spatial_coverage: Some(1.0), // Full spatial coverage
            temporal_continuity: 0.98,  // 98% temporal continuity
        },
        spatial_hops: vec![
            create_spatial_hop(
                0, 
                "dc1_spine1", 
                Some(Coordinate3D { x: 100.0, y: 300.0, z: 2.0 }), 
                now, 
                0.12, 
                220
            ),
            create_spatial_hop(
                1, 
                "dc1_leaf2", 
                Some(Coordinate3D { x: 200.0, y: 200.0, z: 1.0 }), 
                now + chrono::Duration::microseconds(220), 
                0.38, 
                350
            ),
            create_spatial_hop(
                2, 
                "dc1_server3", 
                Some(Coordinate3D { x: 250.0, y: 100.0, z: 0.0 }), 
                now + chrono::Duration::microseconds(570), 
                0.22, 
                180
            ),
        ],
    };
    
    st_flow.add_window(window);
    st_flow
}

/// Helper to create a spatial hop
fn create_spatial_hop(
    index: u32,
    switch_id: &str,
    coordinates: Option<Coordinate3D>,
    timestamp: chrono::DateTime<Utc>,
    queue_util: f64,
    delay_ns: u64,
) -> SpatialHop {
    SpatialHop {
        logical_index: index,
        switch_id: switch_id.to_string(),
        coordinates: coordinates.clone(),
        neighborhood: None, // Could be populated from topology
        temporal_samples: vec![
            TemporalSample {
                timestamp,
                metrics: TelemetryMetrics::with_basic(queue_util, delay_ns),
            }
        ],
        aggregated_metrics: AggregatedMetrics {
            avg_delay: Some(delay_ns),
            max_queue: Some(queue_util),
            spatial_gradient: coordinates.map(|_| SpatialGradient {
                dx_delay: Some(1.2), // Delay increases 1.2ns per unit distance
                dy_delay: Some(-0.5), // Delay decreases 0.5ns per unit height
            }),
            temporal_trend: Some("stable".to_string()),
            additional_metrics: None,
        },
    }
}

/// Convert spatiotemporal flow to legacy format for storage
fn convert_st_to_legacy(st_flow: &SpatiotemporalFlow) -> Result<Flow, Box<dyn std::error::Error>> {
    if st_flow.spatiotemporal_windows.is_empty() {
        return Err("No spatiotemporal windows found".into());
    }
    
    let window = &st_flow.spatiotemporal_windows[0];
    let hops: Vec<Hop> = window.spatial_hops.iter().map(|spatial_hop| {
        let (timestamp, metrics) = if let Some(sample) = spatial_hop.temporal_samples.first() {
            (sample.timestamp, sample.metrics.clone())
        } else {
            (chrono::Utc::now(), TelemetryMetrics::default())
        };
        
        Hop::new(
            spatial_hop.logical_index,
            spatial_hop.switch_id.clone(),
            timestamp,
            metrics,
        )
    }).collect();
    
    Flow::new(st_flow.flow_id.clone(), hops).map_err(|e| e.into())
}

/// Demonstrate JSON serialization for API usage
fn demonstrate_json_formats() -> Result<(), Box<dyn std::error::Error>> {
    // Example of legacy flow input JSON
    let legacy_input = FlowInput {
        flow_id: "int_legacy_json_example".to_string(),
        telemetry: vec![
            HopInput {
                switch_id: "spine1".to_string(),
                timestamp: Utc::now(),
                queue_util: Some(0.15),
                delay_ns: Some(250),
                bandwidth_bps: Some(10_000_000_000), // 10 Gbps
                drop_count: Some(0),
                egress_port: Some(1),
                ingress_port: Some(0),
            },
            HopInput {
                switch_id: "leaf2".to_string(),
                timestamp: Utc::now() + chrono::Duration::microseconds(250),
                queue_util: Some(0.45),
                delay_ns: Some(380),
                bandwidth_bps: Some(10_000_000_000),
                drop_count: Some(2),
                egress_port: Some(2),
                ingress_port: Some(1),
            },
        ],
    };
    
    println!("Legacy Flow JSON format:");
    println!("{}", serde_json::to_string_pretty(&legacy_input)?);
    
    // Example of spatiotemporal flow input JSON
    let st_input = SpatiotemporalFlowInput {
        flow_id: "int_st_json_example".to_string(),
        logical_path: vec!["spine1".to_string(), "leaf2".to_string()],
        topology_coordinates: Some(vec![
            TopologyCoordinate {
                switch: "spine1".to_string(),
                topo_x: 100.0,
                topo_y: 300.0,
                zone: Some("spine".to_string()),
            },
        ]),
        telemetry_data: vec![
            HopTelemetryInput {
                hop_index: 0,
                switch_id: "spine1".to_string(),
                coordinates: Some(Coordinate3D { x: 100.0, y: 300.0, z: 2.0 }),
                temporal_samples: vec![
                    TemporalSampleInput {
                        timestamp: Utc::now(),
                        queue_util: Some(0.15),
                        delay_ns: Some(250),
                        bandwidth_bps: Some(10_000_000_000),
                        drop_count: Some(0),
                        egress_port: Some(1),
                        ingress_port: Some(0),
                    }
                ],
            },
        ],
    };
    
    println!("\nSpatiotemporal Flow JSON format:");
    println!("{}", serde_json::to_string_pretty(&st_input)?);
    
    Ok(())
}

/// Demonstrate various query capabilities
fn demonstrate_queries(engine: &StorageEngine) -> Result<(), Box<dyn std::error::Error>> {
    use intdb::storage::QueryBuilder;
    
    println!("Query 1: Find all flows through 'leaf2'");
    let query = QueryBuilder::through_switch("leaf2").limit(10);
    let result = engine.query(query)?;
    println!("Found {} flows", result.flow_ids.len());
    
    println!("\nQuery 2: Find flows in the last 5 minutes");
    let query = QueryBuilder::in_last_minutes(5).limit(10);
    let result = engine.query(query)?;
    println!("Found {} recent flows", result.flow_ids.len());
    
    println!("\nQuery 3: Find flows with exact path");
    let path = NetworkPath::new(vec![
        "spine1".to_string(), 
        "leaf2".to_string(), 
        "server3".to_string()
    ]);
    let query = QueryBuilder::exact_path(path).limit(10);
    let result = engine.query(query)?;
    println!("Found {} flows with exact path", result.flow_ids.len());
    
    // Show flow details
    if !result.flow_ids.is_empty() {
        let flows = engine.get_flows(&result.flow_ids);
        for flow in flows.iter().take(2) {
            println!("  Flow {}: {} hops, total delay: {:?} ns", 
                flow.flow_id, 
                flow.hops.len(), 
                flow.total_delay()
            );
        }
    }
    
    Ok(())
} 