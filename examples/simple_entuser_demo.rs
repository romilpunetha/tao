// Simple demo showing EntUser generation from schema (without compilation issues)
// Run with: cargo run --example simple_entuser_demo

use tao_database::{
    domains::user::{EntUser, EntUserBuilder},
    infrastructure::TaoIdGenerator,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 EntUser Schema-Based Generation Demo");
    println!("======================================");
    
    println!();
    println!("🏗️  Creating EntUser with fluent builder (based on UserSchema):");
    
    // Create user using fluent builder pattern
    let user_result = EntUserBuilder::new()
        .username("john_doe")
        .email("john@example.com")
        .full_name("John Doe")
        .bio("Software engineer passionate about databases")
        .is_verified(true)
        .location("San Francisco, CA")
        .build();
    
    match user_result {
        Ok(user) => {
            println!("   ✅ Built user successfully!");
            println!("   👤 Username: {}", user.username);
            println!("   📧 Email: {}", user.email);
            println!("   📝 Bio: {}", user.bio.as_ref().unwrap_or(&"None".to_string()));
            println!("   🆔 ID before persistence: {:?}", user.get_id());
            println!("   ✅ Verified status: {}", user.is_verified);
            
            // Test ID generation separately
            println!();
            println!("🆔 TAO ID Generation Demo:");
            let id_gen = TaoIdGenerator::new(42); // Shard 42
            let id1 = id_gen.next_id();
            let id2 = id_gen.next_id();
            let id3 = id_gen.next_id();
            
            println!("   Generated IDs:");
            println!("   - ID 1: {} (shard: {})", id1, TaoIdGenerator::extract_shard_id(id1));
            println!("   - ID 2: {} (shard: {})", id2, TaoIdGenerator::extract_shard_id(id2));
            println!("   - ID 3: {} (shard: {})", id3, TaoIdGenerator::extract_shard_id(id3));
            
            println!();
            println!("✨ What we've accomplished:");
            println!("   ✅ EntUser entity generated from UserSchema");
            println!("   ✅ All schema fields properly typed (required/optional)");
            println!("   ✅ Fluent builder pattern with validation");
            println!("   ✅ TAO ID generator integrated");
            println!("   ✅ Clean architecture: Schema → Entity → Builder → TAO");
            
            println!();
            println!("🔧 What still needs completion:");
            println!("   🔲 Fix EntCodeGenerator compilation issues");
            println!("   🔲 Implement database connection setup");
            println!("   🔲 Complete gen_create method with actual TAO layer");
            println!("   🔲 Implement edge traversal methods (friends, posts, etc.)");
            println!("   🔲 Add regex validation for username/email patterns");
            println!("   🔲 Implement remaining Ent trait methods");
        }
        Err(e) => {
            println!("   ❌ Failed to build user: {:?}", e);
        }
    }
    
    Ok(())
}