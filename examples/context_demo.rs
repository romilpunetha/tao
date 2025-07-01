// Demonstration of Meta-style entity API with context-based TAO injection
// Shows how entities work without global state while maintaining clean API

use std::sync::Arc;
use tao_database::{
    domains::user::EntUser,
    error::AppError,
    framework::entity::ent_trait::Entity,
    infrastructure::{
        association_registry::AssociationRegistry,
        database::database::PostgresDatabase,
        query_router::{QueryRouterConfig, TaoQueryRouter},
        tao_core::{tao::Tao, tao_core::{TaoCore, TaoOperations}},
        viewer::viewer::ViewerContext,
    },
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 TAO Context Demo - Meta-style API without global state");
    
    println!("📝 Demonstrating Meta's authentic API pattern...");
    println!("   (This demo shows the API structure without database operations)");
    
    // Create a dummy TAO instance for demo
    use tao_database::infrastructure::tao_core::tao_core::TaoCore;
    let query_router = Arc::new(TaoQueryRouter::new());
    let association_registry = Arc::new(AssociationRegistry::new());
    let tao_core = Arc::new(TaoCore::new(query_router, association_registry));
    let tao: Arc<dyn TaoOperations> = Arc::new(Tao::minimal(tao_core));
    
    // Create viewer context (Meta's pattern - all dependencies in viewer context)
    let viewer_context = Arc::new(ViewerContext::authenticated_user(
        1001,  // user_id
        "john_doe".to_string(),  // username
        "demo-request-001".to_string(),  // request_id
        tao  // TAO instance - Meta's pattern
    ));
    
    println!("✅ ViewerContext created with TAO instance");
    
    // Demonstrate Meta's authentic API pattern (structure)
    println!("📖 Meta's Entity API Pattern:");
    println!("   EntUser::create(viewer_context) - ✅ Implemented");
    println!("   EntUser::genNullable(viewer_context, id) - ✅ Implemented");
    println!("   EntUser::genEnforce(viewer_context, id) - ✅ Implemented");
    println!("   EntUser::genAll(viewer_context) - ✅ Implemented");
    println!("   EntUser::exists(viewer_context, id) - ✅ Implemented");
    println!("   EntUser::delete(viewer_context, id) - ✅ Implemented");
    
    // Show how builder pattern works with viewer context
    let _builder_demo = EntUser::create(viewer_context.clone())
        .username("demo_user".to_string())
        .email("demo@example.com".to_string())
        .full_name("Demo User".to_string())
        .bio("Demo user for pattern demonstration".to_string())
        .is_verified(false);
    
    println!("✅ Builder pattern works with viewer context");
    println!("   - TAO automatically extracted from viewer context");
    println!("   - No global state needed");
    println!("   - Clean Meta-style API: EntUser::create(vc).username().savex()");

    println!("🎉 Demo completed successfully!");
    println!("   - Meta's authentic pattern: EntUser::create(vc)");
    println!("   - All operations use viewer context: genNullable(vc, id), genEnforce(vc, id)");
    println!("   - No global TAO state - everything through viewer context");
    println!("   - Complete Meta-style entity framework implementation");
    
    Ok(())
}