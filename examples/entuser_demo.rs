// Demo showing complete EntUser creation flow with schema-based validation
// Run with: cargo run --example entuser_demo

use tao_database::{
    domains::user::{EntUser, EntUserBuilder},
    framework::Ent,
    schemas::user_schema::UserSchema,
    framework::EntSchema,
    error::AppResult,
};

#[tokio::main]
async fn main() -> AppResult<()> {
    println!("🚀 EntUser Demo - Schema to Entity Creation");
    println!("============================================");
    
    // Show the schema fields
    println!("📋 UserSchema fields:");
    let fields = UserSchema::fields();
    for field in fields {
        let req = if field.optional { "optional" } else { "required" };
        println!("  - {} ({}): {:?}", field.name, req, field.field_type);
    }
    
    println!();
    println!("🔗 UserSchema edges:");
    let edges = UserSchema::edges();
    for edge in edges {
        println!("  - {}: {:?} -> {:?}", edge.name, UserSchema::entity_type(), edge.target_entity);
    }
    
    println!();
    println!("🏗️  Building EntUser with fluent builder:");
    
    // Create user using fluent builder pattern
    let user = EntUserBuilder::new()
        .username("john_doe")
        .email("john@example.com")
        .full_name("John Doe")
        .bio("Software engineer passionate about databases")
        .is_verified(true)
        .location("San Francisco, CA")
        .build()?;
    
    println!("   ✅ Built user: {:?}", user);
    println!("   📧 Email: {}", user.email);
    println!("   🆔 ID before creation: {:?}", user.get_id());
    
    println!();
    println!("💾 Creating user in TAO database...");
    
    // This would create the user in the database with generated TAO ID
    // let created_user = user.gen_create().await?;
    // println!("   ✅ Created with TAO ID: {}", created_user.id());
    
    // For demo, let's just show validation
    let validation_errors = user.validate()?;
    if validation_errors.is_empty() {
        println!("   ✅ User passes all schema validation rules");
    } else {
        println!("   ❌ Validation errors: {:?}", validation_errors);
    }
    
    println!();
    println!("🎯 What's next to complete:");
    println!("   1. Fix database connection setup");
    println!("   2. Implement edge traversal methods (friends, posts, etc.)");
    println!("   3. Add proper schema validation with regex patterns");
    println!("   4. Implement gen_nullable, gen_enforce methods");
    println!("   5. Add association creation for relationships");
    
    Ok(())
}