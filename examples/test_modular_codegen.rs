// Test the new modular code generation system
use tao_database::{
    framework::SchemaRegistry,
    codegen::CodeGenerator,
    schemas::user_schema::UserSchema,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Testing Modular Code Generation System");
    println!("==========================================");
    
    // Create and populate schema registry
    let mut registry = SchemaRegistry::new();
    registry.register::<UserSchema>();
    
    println!("✅ Registered UserSchema");
    
    // Create code generator
    let generator = CodeGenerator::new(registry);
    
    // Generate all code
    match generator.generate_all() {
        Ok(_) => {
            println!("🎉 Code generation completed successfully!");
            println!();
            println!("Generated files:");
            println!("  - src/domains/user/entity.thrift");
            println!("  - src/domains/user/builder.rs");
            println!("  - src/domains/user/ent_impl.rs");
            println!("  - src/domains/user/mod.rs");
            println!();
            println!("Next steps:");
            println!("  1. Run thrift compiler on entity.thrift to generate entity.rs");
            println!("  2. Use EntUser::create().username().email().save().await pattern");
        },
        Err(e) => {
            eprintln!("❌ Code generation failed: {}", e);
            return Err(e.into());
        }
    }
    
    Ok(())
}