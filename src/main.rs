use log::info;

use intdb::api::routes::create_router;
use intdb::api::handlers::AppState;
use intdb::storage::engine::StorageEngine;

#[tokio::main]
async fn main() {
    // 初始化日志
    env_logger::init();
    
    info!("🚀 Starting IntDB API Server...");
    
    // 创建数据库引擎
    let engine = StorageEngine::new();
    
    // 创建应用状态
    let app_state = AppState::new(engine);
    
    // 创建路由
    let app = create_router(app_state);
    
    // 启动服务器
    let listener = tokio::net::TcpListener::bind("127.0.0.1:2999")
        .await
        .expect("Failed to bind to address");
    
    info!("🌐 IntDB API Server running on http://127.0.0.1:2999");
    info!("📊 Available endpoints:");
    info!("   POST /flows - Insert legacy flow data");
    info!("   GET  /flows/:id - Get legacy flow data");
    info!("   POST /st-flows - Insert spatiotemporal flow data");
    info!("   GET  /st-flows/:id - Get spatiotemporal flow data");
    info!("   POST /st-query - Spatiotemporal query");
    info!("   POST /query - Legacy query");
    
    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
} 