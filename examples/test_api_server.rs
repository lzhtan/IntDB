use std::error::Error;
use tokio::net::TcpListener;
use chrono::Utc;

use intdb::{
    StorageEngine, AppState, create_minimal_router,
    FlowInput, HopInput, Flow,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🚀 IntDB 测试API服务器");
    println!("======================");
    
    // 创建存储引擎
    let mut engine = StorageEngine::new();
    
    // 预加载一些测试数据
    populate_test_data(&mut engine)?;
    
    // 创建应用状态
    let state = AppState::new(engine);
    
    // 创建简化路由
    let app = create_minimal_router(state);
    
    // 启动服务器
    let listener = TcpListener::bind("127.0.0.1:3000").await?;
    println!("🌐 服务器启动在 http://127.0.0.1:3000");
    
    // 打印测试端点
    println!("\n📋 可用端点:");
    println!("   GET  /health          - 健康检查");
    println!("   POST /flows           - 插入新流");
    println!("   GET  /flows/{{id}}      - 获取指定流");
    println!("   POST /query           - 高级查询");
    
    println!("\n🧪 测试命令:");
    println!("curl http://127.0.0.1:3000/health");
    println!("curl http://127.0.0.1:3000/flows/test_flow_1");
    
    println!("\n⚡ 测试查询命令:");
    println!("curl -X POST http://127.0.0.1:3000/query \\");
    println!("  -H 'Content-Type: application/json' \\");
    println!("  -d '{{\"path_conditions\": [{{\"type\": \"through_switch\", \"value\": {{\"switch_id\": \"s2\"}}}}]}}'");
    
    println!("\n👍 按 Ctrl+C 停止服务器");
    
    // 启动服务器
    axum::serve(listener, app).await?;
    
    Ok(())
}

fn populate_test_data(engine: &mut StorageEngine) -> Result<(), Box<dyn Error>> {
    println!("📊 加载测试数据...");
    
    let base_time = Utc::now();
    
    // 测试流 1: s1 → s2 → s3 (高延迟)
    let flow1 = FlowInput {
        flow_id: "test_flow_1".to_string(),
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
    };
    
    // 测试流 2: s1 → s2 → s4 (低延迟)
    let flow2 = FlowInput {
        flow_id: "test_flow_2".to_string(),
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
    };
    
    // 测试流 3: 大数据中心路径 s1 → s2 → s5 → s6 → s7
    let flow3 = FlowInput {
        flow_id: "test_flow_3".to_string(),
        telemetry: vec![
            HopInput {
                switch_id: "s1".to_string(),
                timestamp: base_time + chrono::Duration::minutes(2),
                queue_util: Some(0.5),
                delay_ns: Some(100),
                bandwidth_bps: Some(10000),
                drop_count: None,
                egress_port: None,
                ingress_port: None,
            },
            HopInput {
                switch_id: "s2".to_string(),
                timestamp: base_time + chrono::Duration::minutes(2) + chrono::Duration::milliseconds(100),
                queue_util: Some(0.6),
                delay_ns: Some(120),
                bandwidth_bps: Some(9800),
                drop_count: None,
                egress_port: None,
                ingress_port: None,
            },
            HopInput {
                switch_id: "s5".to_string(),
                timestamp: base_time + chrono::Duration::minutes(2) + chrono::Duration::milliseconds(220),
                queue_util: Some(0.7),
                delay_ns: Some(150),
                bandwidth_bps: Some(9500),
                drop_count: None,
                egress_port: None,
                ingress_port: None,
            },
            HopInput {
                switch_id: "s6".to_string(),
                timestamp: base_time + chrono::Duration::minutes(2) + chrono::Duration::milliseconds(370),
                queue_util: Some(0.8),
                delay_ns: Some(180),
                bandwidth_bps: Some(9200),
                drop_count: None,
                egress_port: None,
                ingress_port: None,
            },
            HopInput {
                switch_id: "s7".to_string(),
                timestamp: base_time + chrono::Duration::minutes(2) + chrono::Duration::milliseconds(550),
                queue_util: Some(0.4),
                delay_ns: Some(90),
                bandwidth_bps: Some(9400),
                drop_count: None,
                egress_port: None,
                ingress_port: None,
            },
        ],
    };
    
    // 插入测试流
    let flows = vec![flow1, flow2, flow3];
    for flow_input in flows {
        let flow = Flow::try_from(flow_input)?;
        engine.insert_flow(flow)?;
    }
    
    println!("✅ 已加载 {} 个测试流", engine.flow_count());
    
    Ok(())
} 