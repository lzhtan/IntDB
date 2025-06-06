use std::error::Error;
use chrono::Utc;

use intdb::{
    StorageEngine, AppState, 
    FlowInput, HopInput,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🚀 IntDB API 演示");
    println!("==================");
    
    // 创建存储引擎和应用状态
    let engine = StorageEngine::new();
    let state = AppState::new(engine);
    
    // 示例数据：创建几个测试流
    let demo_flows = create_demo_flows();
    let flow_count = demo_flows.len();
    
    println!("\n📊 插入演示数据...");
    for (i, flow) in demo_flows.into_iter().enumerate() {
        println!("   插入流 {}: {}", i + 1, flow.flow_id);
        
        // 模拟API调用: POST /flows
        let flow_result = intdb::Flow::try_from(flow)?;
        state.engine.insert_flow(flow_result)?;
    }
    
    println!("✅ 已插入 {} 个流", flow_count);
    
    // 演示各种查询功能
    demonstrate_queries(&state).await?;
    
    Ok(())
}

fn create_demo_flows() -> Vec<FlowInput> {
    let base_time = Utc::now();
    
    vec![
        // 流 1: s1 → s2 → s3 (高延迟)
        FlowInput {
            flow_id: "flow1".to_string(),
            telemetry: vec![
                HopInput {
                    switch_id: "s1".to_string(),
                    timestamp: base_time,
                    queue_util: Some(0.8),
                    delay_ns: Some(200),
                    bandwidth_bps: Some(1000),
                    drop_count: None,
                    egress_port: None,
                    ingress_port: None,
                },
                HopInput {
                    switch_id: "s2".to_string(),
                    timestamp: base_time + chrono::Duration::milliseconds(200),
                    queue_util: Some(0.9),
                    delay_ns: Some(300),
                    bandwidth_bps: Some(900),
                    drop_count: None,
                    egress_port: None,
                    ingress_port: None,
                },
                HopInput {
                    switch_id: "s3".to_string(),
                    timestamp: base_time + chrono::Duration::milliseconds(500),
                    queue_util: Some(0.7),
                    delay_ns: Some(150),
                    bandwidth_bps: Some(950),
                    drop_count: None,
                    egress_port: None,
                    ingress_port: None,
                },
            ],
        },
        
        // 流 2: s1 → s2 → s4 (低延迟)
        FlowInput {
            flow_id: "flow2".to_string(),
            telemetry: vec![
                HopInput {
                    switch_id: "s1".to_string(),
                    timestamp: base_time + chrono::Duration::minutes(1),
                    queue_util: Some(0.3),
                    delay_ns: Some(50),
                    bandwidth_bps: Some(1200),
                    drop_count: None,
                    egress_port: None,
                    ingress_port: None,
                },
                HopInput {
                    switch_id: "s2".to_string(),
                    timestamp: base_time + chrono::Duration::minutes(1) + chrono::Duration::milliseconds(50),
                    queue_util: Some(0.4),
                    delay_ns: Some(80),
                    bandwidth_bps: Some(1100),
                    drop_count: None,
                    egress_port: None,
                    ingress_port: None,
                },
                HopInput {
                    switch_id: "s4".to_string(),
                    timestamp: base_time + chrono::Duration::minutes(1) + chrono::Duration::milliseconds(130),
                    queue_util: Some(0.2),
                    delay_ns: Some(40),
                    bandwidth_bps: Some(1150),
                    drop_count: None,
                    egress_port: None,
                    ingress_port: None,
                },
            ],
        },
        
        // 流 3: s2 → s3 → s4 (中等延迟)
        FlowInput {
            flow_id: "flow3".to_string(),
            telemetry: vec![
                HopInput {
                    switch_id: "s2".to_string(),
                    timestamp: base_time + chrono::Duration::minutes(2),
                    queue_util: Some(0.6),
                    delay_ns: Some(120),
                    bandwidth_bps: Some(800),
                    drop_count: None,
                    egress_port: None,
                    ingress_port: None,
                },
                HopInput {
                    switch_id: "s3".to_string(),
                    timestamp: base_time + chrono::Duration::minutes(2) + chrono::Duration::milliseconds(120),
                    queue_util: Some(0.7),
                    delay_ns: Some(180),
                    bandwidth_bps: Some(750),
                    drop_count: None,
                    egress_port: None,
                    ingress_port: None,
                },
                HopInput {
                    switch_id: "s4".to_string(),
                    timestamp: base_time + chrono::Duration::minutes(2) + chrono::Duration::milliseconds(300),
                    queue_util: Some(0.5),
                    delay_ns: Some(100),
                    bandwidth_bps: Some(800),
                    drop_count: None,
                    egress_port: None,
                    ingress_port: None,
                },
            ],
        },
    ]
}

async fn demonstrate_queries(state: &AppState) -> Result<(), Box<dyn Error>> {
    println!("\n🔍 演示查询功能");
    println!("================");
    
    // 1. 基础统计查询
    println!("\n1️⃣ 基础统计:");
    println!("   总流数量: {}", state.engine.flow_count());
    
    // 2. 路径查询 - 精确路径
    println!("\n2️⃣ 精确路径查询 (s1 → s2 → s3):");
    let path = intdb::NetworkPath::new(vec!["s1".to_string(), "s2".to_string(), "s3".to_string()]);
    let exact_path_query = intdb::QueryBuilder::exact_path(path);
    let result = state.engine.query(exact_path_query)?;
    println!("   找到 {} 个匹配的流", result.count());
    
    // 3. 交换机查询 - 通过特定交换机
    println!("\n3️⃣ 通过交换机查询 (经过 s2):");
    let through_switch_query = intdb::QueryBuilder::through_switch("s2");
    let result = state.engine.query(through_switch_query)?;
    println!("   找到 {} 个经过 s2 的流", result.count());
    
    // 4. 时间查询 - 最近5分钟
    println!("\n4️⃣ 时间查询 (最近5分钟):");
    let recent_query = intdb::QueryBuilder::in_last_minutes(5);
    let result = state.engine.query(recent_query)?;
    println!("   找到 {} 个最近5分钟的流", result.count());
    
    // 5. 指标查询 - 高延迟流
    println!("\n5️⃣ 指标查询 (总延迟 > 400ns):");
    let high_delay_query = intdb::QueryBuilder::new()
        .with_metric_condition(intdb::storage::MetricCondition::TotalDelayGreaterThan(400));
    let result = state.engine.query(high_delay_query)?;
    println!("   找到 {} 个高延迟流", result.count());
    
    // 6. 复合查询 - 经过s2且队列利用率高
    println!("\n6️⃣ 复合查询 (经过 s2 且最大队列利用率 > 0.8):");
    let complex_query = intdb::QueryBuilder::through_switch("s2")
        .with_metric_condition(intdb::storage::MetricCondition::MaxQueueUtilGreaterThan(0.8));
    let result = state.engine.query(complex_query)?;
    println!("   找到 {} 个匹配的流", result.count());
    
    // 7. 路径长度查询
    println!("\n7️⃣ 路径长度查询 (3跳路径):");
    let length_query = intdb::QueryBuilder::new()
        .with_path_condition(intdb::storage::PathCondition::LengthEquals(3));
    let result = state.engine.query(length_query)?;
    println!("   找到 {} 个3跳路径的流", result.count());
    
    // 8. 展示一些实际流数据
    println!("\n8️⃣ 查看流详细信息:");
    let all_flows_query = intdb::QueryBuilder::new().limit(3);
    let result = state.engine.query(all_flows_query)?;
    let flows = state.engine.get_flows(&result.flow_ids);
    
    for flow in flows.iter() {
        println!("   流 {}: 路径长度={}, 总延迟={:?}ns, 最大队列利用率={:.2}", 
                 flow.flow_id, 
                 flow.path_length(),
                 flow.total_delay(),
                 flow.max_queue_utilization().unwrap_or(0.0)
        );
    }
    
    println!("\n✅ 查询演示完成!");
    println!("\n💡 这些查询展示了IntDB针对INT场景的优势:");
    println!("   • 路径语义原生支持");
    println!("   • 时空数据统一查询");  
    println!("   • 多维索引高效检索");
    println!("   • 复合条件灵活组合");
    
    Ok(())
} 