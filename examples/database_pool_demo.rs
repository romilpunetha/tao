// Database Connection Pool Performance Demo
// This example demonstrates the improved connection pooling configuration

use tao_database::infrastructure::{initialize_database, database_health_check, database_pool_stats};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔗 TAO Database Connection Pool Demo");
    println!();
    
    // === CONFIGURATION ===
    println!("📋 Connection Pool Configuration:");
    println!("   Environment Variables (set these for custom configuration):");
    println!("   • DB_MAX_CONNECTIONS: Maximum concurrent connections (default: 20)");
    println!("   • DB_MIN_CONNECTIONS: Minimum connections to maintain (default: 5)");
    println!("   • DB_ACQUIRE_TIMEOUT_SECS: Connection acquisition timeout (default: 8s)");
    println!();
    
    // === BEFORE vs AFTER ===
    println!("⚡ Performance Improvements:");
    println!("   Before: Single connection + manual connection management");
    println!("   After:  Production-ready connection pool with:");
    println!("   ✅ 20 max concurrent connections (configurable)");
    println!("   ✅ 5 minimum connections always available");
    println!("   ✅ 8 second connection timeout");
    println!("   ✅ 10 minute idle connection timeout");
    println!("   ✅ 30 minute max connection lifetime");
    println!("   ✅ Connection health testing before use");
    println!();
    
    // === DEMONSTRATION ===
    println!("🚀 Initializing database with production pool settings...");
    
    // Use test database URL (in real usage, set DATABASE_URL env var)
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost/tao_test".to_string());
    
    match initialize_database(&db_url).await {
        Ok(()) => {
            println!("✅ Database pool initialized successfully!");
            println!();
            
            // Perform health check
            println!("🏥 Performing database health check...");
            match database_health_check().await {
                Ok(()) => println!("✅ Database health check passed!"),
                Err(e) => println!("❌ Database health check failed: {}", e),
            }
            println!();
            
            // Show pool statistics
            println!("📊 Connection Pool Statistics:");
            match database_pool_stats().await {
                Ok((idle, total)) => {
                    println!("   • Idle connections: {}", idle);
                    println!("   • Total connections: {}", total);
                    println!("   • Active connections: {}", total - idle);
                },
                Err(e) => println!("   ❌ Failed to get pool stats: {}", e),
            }
            println!();
            
            // === BENEFITS ===
            println!("🎯 Production Benefits:");
            println!("   ✅ Handles high concurrent load (up to 20 simultaneous operations)");
            println!("   ✅ Efficient connection reuse (no connection setup overhead)");
            println!("   ✅ Automatic connection lifecycle management");
            println!("   ✅ Connection health monitoring and recovery");
            println!("   ✅ Configurable timeouts prevent hanging operations");
            println!("   ✅ Environment-based configuration for different deployments");
            
        },
        Err(e) => {
            println!("❌ Failed to initialize database: {}", e);
            println!("   💡 Make sure PostgreSQL is running and accessible");
            println!("   💡 Set DATABASE_URL environment variable if needed");
        }
    }
    
    Ok(())
}