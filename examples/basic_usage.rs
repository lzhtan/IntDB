use chrono::Utc;
use intdb::models::{Flow, Hop, TelemetryMetrics, NetworkPath};
use intdb::storage::{StorageEngine, QueryBuilder, PathCondition, TimeCondition, MetricCondition};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 IntDB Storage Engine Demo");
    
    // Create storage engine
    let engine = StorageEngine::new();
    
    // Create some example flows
    let now = Utc::now();
    
    // Flow 1: s1 -> s2 -> s3
    let flow1 = create_flow(
        "flow_001", 
        &["s1", "s2", "s3"], 
        now,
        &[(0.1, 100), (0.3, 200), (0.5, 300)]
    )?;
    
    // Flow 2: s1 -> s2 -> s4  
    let flow2 = create_flow(
        "flow_002",
        &["s1", "s2", "s4"],
        now + chrono::Duration::seconds(30),
        &[(0.2, 150), (0.4, 250), (0.1, 100)]
    )?;
    
    // Flow 3: s2 -> s3 -> s4
    let flow3 = create_flow(
        "flow_003",
        &["s2", "s3", "s4"],
        now + chrono::Duration::minutes(2),
        &[(0.6, 400), (0.2, 200), (0.3, 300)]
    )?;
    
    // Insert flows
    println!("\n📥 Inserting flows...");
    engine.insert_flow(flow1.clone())?;
    engine.insert_flow(flow2.clone())?;
    engine.insert_flow(flow3.clone())?;
    
    println!("✅ Inserted {} flows", engine.flow_count());
    
    // Query examples
    println!("\n🔍 Query Examples:");
    
    // 1. Find flows with exact path
    println!("\n1. Flows with exact path [s1 -> s2 -> s3]:");
    let query = QueryBuilder::exact_path(flow1.path.clone());
    let result = engine.query(query)?;
    println!("   Found {} flows: {:?}", result.count(), result.flow_ids);
    
    // 2. Find flows through specific switch
    println!("\n2. Flows passing through switch 's2':");
    let query = QueryBuilder::through_switch("s2");
    let result = engine.query(query)?;
    println!("   Found {} flows: {:?}", result.count(), result.flow_ids);
    
    // 3. Find flows in last 5 minutes
    println!("\n3. Flows in last 5 minutes:");
    let query = QueryBuilder::in_last_minutes(5);
    let result = engine.query(query)?;
    println!("   Found {} flows: {:?}", result.count(), result.flow_ids);
    
    // 4. Find flows with high delay
    println!("\n4. Flows with total delay > 500ns:");
    let query = QueryBuilder::with_high_delay(500);
    let result = engine.query(query)?;
    println!("   Found {} flows: {:?}", result.count(), result.flow_ids);
    
    // 5. Complex query: flows through s2 with high queue utilization
    println!("\n5. Complex query: flows through 's2' with max queue util > 0.4:");
    let query = QueryBuilder::new()
        .with_path_condition(PathCondition::ThroughSwitch("s2".to_string()))
        .with_metric_condition(MetricCondition::MaxQueueUtilGreaterThan(0.4));
    let result = engine.query(query)?;
    println!("   Found {} flows: {:?}", result.count(), result.flow_ids);
    
    // 6. Retrieve and display flows
    if !result.flow_ids.is_empty() {
        println!("\n📊 Flow Details:");
        let flows = engine.get_flows(&result.flow_ids);
        for flow in flows {
            println!("   Flow {}: path={}, delay={}ns, max_queue_util={:.2}", 
                flow.flow_id,
                flow.path,
                flow.total_delay().unwrap_or(0),
                flow.max_queue_utilization().unwrap_or(0.0)
            );
        }
    }
    
    println!("\n✨ Demo completed successfully!");
    Ok(())
}

fn create_flow(
    flow_id: &str, 
    switches: &[&str], 
    start_time: chrono::DateTime<Utc>,
    metrics: &[(f64, u64)]  // (queue_util, delay_ns)
) -> Result<Flow, Box<dyn std::error::Error>> {
    let hops: Vec<Hop> = switches
        .iter()
        .enumerate()
        .map(|(i, switch)| {
            let (queue_util, delay_ns) = metrics.get(i).unwrap_or(&(0.0, 0));
            Hop::new(
                i as u32,
                switch.to_string(),
                start_time + chrono::Duration::milliseconds(i as i64 * 10),
                TelemetryMetrics::with_basic(*queue_util, *delay_ns),
            )
        })
        .collect();
    
    Ok(Flow::new(flow_id.to_string(), hops)?)
}

fn get_flows(engine: &StorageEngine, flow_ids: &[String]) -> Vec<Flow> {
    flow_ids.iter()
        .filter_map(|id| engine.get_flow(id))
        .collect()
} 