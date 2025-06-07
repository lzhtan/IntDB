use std::error::Error;
use tokio::net::TcpListener;
use log::info;

use intdb::{StorageEngine, AppState, create_router};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    env_logger::init();
    
    // Create storage engine
    let engine = StorageEngine::new();
    
    // Create application state
    let state = AppState::new(engine);
    
    // Create router
    let app = create_router(state);
    
    // Create listener
    let listener = TcpListener::bind("127.0.0.1:2999").await?;
    info!("🚀 IntDB API server starting on http://127.0.0.1:2999");
    
    // Print available endpoints
    println!("\n📡 Available endpoints:");
    println!("   GET  /health                - Health check");
    println!("   GET  /stats                 - Database statistics");
    println!("   POST /flows                 - Insert a new flow");
    println!("   GET  /flows/:id             - Get flow by ID");
    println!("   POST /flows/batch           - Get multiple flows");
    println!("   POST /query                 - Advanced query flows");
    println!("   GET  /quick/through/:switch - Flows through a switch");
    println!("   POST /quick/path            - Flows with exact path");
    println!("   GET  /quick/recent/:minutes - Recent flows");
    println!("\n💡 Example requests:");
    println!("   curl http://127.0.0.1:2999/health");
    println!("   curl -X POST http://127.0.0.1:2999/flows \\");
    println!("        -H 'Content-Type: application/json' \\");
    println!("        -d '{{\"flow\": {{\"path\": [\"s1\", \"s2\", \"s3\"], \"hops\": [...]}}}}'");
    
    // Start server
    axum::serve(listener, app).await?;
    
    Ok(())
} 