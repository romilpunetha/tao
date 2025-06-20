// TAO Database Server - Unified Meta TAO Interface

use axum::Router;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

use tao_database::{
    config::Config,
    app_state::AppState,
    tao_interface::create_tao_router,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load configuration
    let config = Config::from_env()?;

    // Initialize application state
    let app_state = AppState::new(config.clone()).await?;

    // Create unified TAO router
    let tao_router = create_tao_router(app_state.tao_interface.clone());

    // Build main application router
    let app = Router::new()
        .nest("/api/v1/tao", tao_router)
        .layer(CorsLayer::permissive());

    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], config.server.port));
    println!("🚀 TAO Database Server starting on http://{}", addr);
    println!("📋 API Documentation:");
    println!("  GET    /api/v1/tao/entities/{{id}}              - Get entity");
    println!("  DELETE /api/v1/tao/entities/{{id}}              - Delete entity");
    println!("  GET    /api/v1/tao/viewer/entity/{{id}}         - View entity with associations");
    println!("  POST   /api/v1/tao/associations                - Create association");
    println!("  GET    /api/v1/tao/associations                - Get associations");
    println!("  DELETE /api/v1/tao/associations/{{src}}/{{tgt}}/{{type}} - Delete association");

    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}