//! Example showing how to work with INT data in IntDB
//! This demonstrates both legacy Flow format and new SpatiotemporalFlow format

use chrono::Utc;
use intdb::models::*;
use intdb::storage::StorageEngine;
use intdb::api::{create_minimal_router, AppState};
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌐 IntDB INT Data Example");
    println!("========================");
    
    // Create storage engine
    let engine = StorageEngine::new();
    let app_state = AppState::new(engine);
    
    // Example 1: Legacy Flow format (simple)
    println!("\n📊 Example 1: Legacy Flow Format");
    let legacy_flow = create_legacy_flow_example()?;
    app_state.engine.insert_flow(legacy_flow.clone())?;
    println!("✅ Inserted legacy flow: {}", legacy_flow.flow_id);
    
    // Example 2: Spatiotemporal Flow format (logical path only)
    println!("\n🔀 Example 2: Spatiotemporal Flow (Logical Path Only)");
    let logical_st_flow = create_logical_spatiotemporal_flow_example();
    let legacy_converted = convert_to_legacy(&logical_st_flow)?;
    app_state.engine.insert_flow(legacy_converted)?;
    println!("✅ Inserted spatiotemporal flow (logical): {}", logical_st_flow.flow_id);
    
    // Example 3: Spatiotemporal Flow format (with spatial info)
    println!("\n🗺️  Example 3: Spatiotemporal Flow (With Spatial Info)");
    let spatial_st_flow = create_spatial_spatiotemporal_flow_example();
    let legacy_converted = convert_to_legacy(&spatial_st_flow)?;
    app_state.engine.insert_flow(legacy_converted)?;
    println!("✅ Inserted spatiotemporal flow (spatial): {}", spatial_st_flow.flow_id);
    
    // Example 4: JSON representation for API usage
    println!("\n📄 Example 4: JSON API Format");
    print_json_examples()?;
    
    // Example 5: Query examples
    println!("\n🔍 Example 5: Query Examples");
    demonstrate_queries(&app_state).await?;
    
    // Start API server for testing
    println!("\n🚀 Starting API Server on http://127.0.0.1:3001");
    println!("Test endpoints:");
    println!("  GET  /health");
    println!("  POST /flows (legacy format)");
    println!("  POST /st-flows (spatiotemporal format)");
    println!("  POST /query (legacy queries)");
    println!("  POST /st-query (spatiotemporal queries)");
    
    let app = create_minimal_router(app_state);
    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
    let listener = TcpListener::bind(addr).await?;
    
    println!("\n✨ Server ready! Press Ctrl+C to stop.");
    axum::serve(listener, app).await?;
    
    Ok(())
}

/// Create a legacy flow example
fn create_legacy_flow_example() -> Result<Flow, FlowError> {
    let now = Utc::now();
    
    let hops = vec![
        Hop::with_basic_metrics(0, "s1".to_string(), now, 0.1, 200),
        Hop::with_basic_metrics(1, "s2".to_string(), now + chrono::Duration::milliseconds(10), 0.3, 300),
        Hop::with_basic_metrics(2, "s3".to_string(), now + chrono::Duration::milliseconds(20), 0.2, 250),
    ];
    
    Flow::new("legacy_flow_001".to_string(), hops)
}

/// Create a logical-only spatiotemporal flow example
fn create_logical_spatiotemporal_flow_example() -> SpatiotemporalFlow {
    let logical_path = vec!["s1".to_string(), "s2".to_string(), "s3".to_string()];
    let mut st_flow = SpatiotemporalFlow::new_logical("st_logical_flow_001".to_string(), logical_path);
    
    // Add a time window with sample data
    let now = Utc::now();
    let window = SpatiotemporalWindow {
        st_window_id: "st_w1_logical".to_string(),
        temporal_bounds: TemporalBounds {
            start: now,
            end: now + chrono::Duration::seconds(60),
        },
        spatial_bounds: None,
        packet_count: 100,
        quality_metrics: QualityMetrics {
            path_completeness: 0.98,
            spatial_coverage: None,
            temporal_continuity: 0.95,
        },
        spatial_hops: vec![
            SpatialHop {
                logical_index: 0,
                switch_id: "s1".to_string(),
                coordinates: None,
                neighborhood: None,
                temporal_samples: vec![
                    TemporalSample {
                        timestamp: now,
                        metrics: TelemetryMetrics::with_basic(0.1, 200),
                    }
                ],
                aggregated_metrics: AggregatedMetrics {
                    avg_delay: Some(200),
                    max_queue: Some(0.1),
                    spatial_gradient: None,
                    temporal_trend: Some("stable".to_string()),
                    additional_metrics: None,
                },
            },
            SpatialHop {
                logical_index: 1,
                switch_id: "s2".to_string(),
                coordinates: None,
                neighborhood: None,
                temporal_samples: vec![
                    TemporalSample {
                        timestamp: now + chrono::Duration::milliseconds(10),
                        metrics: TelemetryMetrics::with_basic(0.3, 300),
                    }
                ],
                aggregated_metrics: AggregatedMetrics {
                    avg_delay: Some(300),
                    max_queue: Some(0.3),
                    spatial_gradient: None,
                    temporal_trend: Some("increasing".to_string()),
                    additional_metrics: None,
                },
            },
        ],
    };
    
    st_flow.add_window(window);
    st_flow
}

/// Create a spatiotemporal flow with spatial information
fn create_spatial_spatiotemporal_flow_example() -> SpatiotemporalFlow {
    let logical_path = vec!["rack1_s1".to_string(), "rack2_s2".to_string(), "rack3_s3".to_string()];
    let topology_coordinates = vec![
        TopologyCoordinate {
            switch: "rack1_s1".to_string(),
            topo_x: 100.0,
            topo_y: 200.0,
            zone: Some("rack1".to_string()),
        },
        TopologyCoordinate {
            switch: "rack2_s2".to_string(),
            topo_x: 300.0,
            topo_y: 200.0,
            zone: Some("rack2".to_string()),
        },
        TopologyCoordinate {
            switch: "rack3_s3".to_string(),
            topo_x: 500.0,
            topo_y: 200.0,
            zone: Some("rack3".to_string()),
        },
    ];
    
    let spatial_metadata = SpatialMetadata::with_spatial_info(
        logical_path,
        topology_coordinates,
        Some("LINESTRING(100 200, 300 200, 500 200)".to_string()),
    );
    
    let temporal_metadata = TemporalMetadata {
        flow_state: FlowState::Active,
        creation_time: Utc::now(),
        last_update: Utc::now(),
        window_duration: Some(60000),
        retention_policy: Some("7d".to_string()),
    };
    
    let mut st_flow = SpatiotemporalFlow {
        flow_id: "st_spatial_flow_001".to_string(),
        spatial_metadata,
        temporal_metadata,
        spatiotemporal_windows: Vec::new(),
        spatiotemporal_indices: SpatiotemporalIndices {
            rtree_index: Some("rtree_spatial_001".to_string()),
            temporal_btree: "btree_temporal_001".to_string(),
            st_compound_index: Some("st_compound_001".to_string()),
            logical_path_trie: "trie_path_001".to_string(),
        },
    };
    
    // Add a window with spatial data
    let now = Utc::now();
    let window = SpatiotemporalWindow {
        st_window_id: "st_w1_spatial".to_string(),
        temporal_bounds: TemporalBounds {
            start: now,
            end: now + chrono::Duration::seconds(60),
        },
        spatial_bounds: Some(SpatialExtent {
            min_x: 100.0,
            min_y: 200.0,
            max_x: 500.0,
            max_y: 200.0,
        }),
        packet_count: 150,
        quality_metrics: QualityMetrics {
            path_completeness: 1.0,
            spatial_coverage: Some(1.0),
            temporal_continuity: 0.98,
        },
        spatial_hops: vec![
            SpatialHop {
                logical_index: 0,
                switch_id: "rack1_s1".to_string(),
                coordinates: Some(Coordinate3D { x: 100.0, y: 200.0, z: 0.0 }),
                neighborhood: Some(vec!["rack2_s2".to_string()]),
                temporal_samples: vec![
                    TemporalSample {
                        timestamp: now,
                        metrics: TelemetryMetrics::with_basic(0.15, 180),
                    }
                ],
                aggregated_metrics: AggregatedMetrics {
                    avg_delay: Some(180),
                    max_queue: Some(0.15),
                    spatial_gradient: Some(SpatialGradient {
                        dx_delay: Some(0.5), // delay increases 0.5ns per unit x
                        dy_delay: Some(0.0), // no y variation
                    }),
                    temporal_trend: Some("stable".to_string()),
                    additional_metrics: None,
                },
            },
        ],
    };
    
    st_flow.add_window(window);
    st_flow
}

/// Helper to convert spatiotemporal flow to legacy format
fn convert_to_legacy(st_flow: &SpatiotemporalFlow) -> Result<Flow, Box<dyn std::error::Error>> {
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

/// Print JSON examples for API usage
fn print_json_examples() -> Result<(), Box<dyn std::error::Error>> {
    // Legacy flow input
    let legacy_input = FlowInput {
        flow_id: "example_legacy".to_string(),
        telemetry: vec![
            HopInput {
                switch_id: "s1".to_string(),
                timestamp: Utc::now(),
                queue_util: Some(0.1),
                delay_ns: Some(200),
                bandwidth_bps: Some(1000000),
                drop_count: Some(0),
                egress_port: Some(1),
                ingress_port: Some(0),
            },
        ],
    };
    
    println!("Legacy Flow Input JSON:");
    println!("{}", serde_json::to_string_pretty(&legacy_input)?);
    
    // Spatiotemporal flow input (logical only)
    let st_input = SpatiotemporalFlowInput {
        flow_id: "example_st_logical".to_string(),
        logical_path: vec!["s1".to_string(), "s2".to_string()],
        topology_coordinates: None,
        telemetry_data: vec![
            HopTelemetryInput {
                hop_index: 0,
                switch_id: "s1".to_string(),
                coordinates: None,
                temporal_samples: vec![
                    TemporalSampleInput {
                        timestamp: Utc::now(),
                        queue_util: Some(0.1),
                        delay_ns: Some(200),
                        bandwidth_bps: Some(1000000),
                        drop_count: Some(0),
                        egress_port: Some(1),
                        ingress_port: Some(0),
                    }
                ],
            },
        ],
    };
    
    println!("\nSpatiotemporal Flow Input JSON (Logical Only):");
    println!("{}", serde_json::to_string_pretty(&st_input)?);
    
    // Spatiotemporal flow input (with spatial info)
    let st_spatial_input = SpatiotemporalFlowInput {
        flow_id: "example_st_spatial".to_string(),
        logical_path: vec!["rack1_s1".to_string(), "rack2_s2".to_string()],
        topology_coordinates: Some(vec![
            TopologyCoordinate {
                switch: "rack1_s1".to_string(),
                topo_x: 100.0,
                topo_y: 200.0,
                zone: Some("rack1".to_string()),
            },
            TopologyCoordinate {
                switch: "rack2_s2".to_string(),
                topo_x: 300.0,
                topo_y: 200.0,
                zone: Some("rack2".to_string()),
            },
        ]),
        telemetry_data: vec![
            HopTelemetryInput {
                hop_index: 0,
                switch_id: "rack1_s1".to_string(),
                coordinates: Some(Coordinate3D { x: 100.0, y: 200.0, z: 0.0 }),
                temporal_samples: vec![
                    TemporalSampleInput {
                        timestamp: Utc::now(),
                        queue_util: Some(0.15),
                        delay_ns: Some(180),
                        bandwidth_bps: Some(10000000),
                        drop_count: Some(0),
                        egress_port: Some(2),
                        ingress_port: Some(1),
                    }
                ],
            },
        ],
    };
    
    println!("\nSpatiotemporal Flow Input JSON (With Spatial Info):");
    println!("{}", serde_json::to_string_pretty(&st_spatial_input)?);
    
    Ok(())
}

/// Demonstrate various queries
async fn demonstrate_queries(app_state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
    use intdb::storage::{QueryBuilder, PathCondition, TimeCondition};
    
    println!("Query 1: Find flows through switch 's2'");
    let query = QueryBuilder::through_switch("s2").limit(10);
    let result = app_state.engine.query(query)?;
    println!("Found {} flows", result.flow_ids.len());
    
    println!("\nQuery 2: Find flows in the last 5 minutes");
    let query = QueryBuilder::in_last_minutes(5).limit(10);
    let result = app_state.engine.query(query)?;
    println!("Found {} recent flows", result.flow_ids.len());
    
    println!("\nQuery 3: Find flows with specific path");
    let path = NetworkPath::new(vec!["s1".to_string(), "s2".to_string(), "s3".to_string()]);
    let query = QueryBuilder::exact_path(path).limit(10);
    let result = app_state.engine.query(query)?;
    println!("Found {} flows with exact path", result.flow_ids.len());
    
    // Get actual flow data
    if !result.flow_ids.is_empty() {
        let flows = app_state.engine.get_flows(&result.flow_ids);
        for flow in flows.iter().take(3) {
            println!("  - Flow {}: {} hops, total delay: {:?}", 
                flow.flow_id, 
                flow.hops.len(), 
                flow.total_delay()
            );
        }
    }
    
    Ok(())
} 