// TAO Code Generator - Generate entities from schema definitions
use tao_database::framework::codegen::CodeGenerator;
use tao_database::schemas::{create_schema_registry, validate_schemas};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 TAO Entity Code Generation");
    println!("==============================");

    // Validate schemas first
    println!("🔍 Validating schemas...");
    match validate_schemas() {
        Ok(()) => println!("✅ Schema validation passed"),
        Err(errors) => {
            println!("❌ Schema validation failed:");
            for error in errors {
                println!("   - {}", error);
            }
            return Err("Schema validation failed".into());
        }
    }

    // Create schema registry
    let registry = create_schema_registry();
    let entity_types = registry.get_entity_types();
    println!(
        "📊 Found {} entity types: {:?}",
        entity_types.len(),
        entity_types
    );

    // Initialize code generator
    let generator = CodeGenerator::new(registry);

    // Generate all entity code
    println!("\n🔧 Generating entity code...");
    match generator.generate_all() {
        Ok(_) => {
            println!("✅ Code generation completed successfully!");
            println!("📝 Generated entity files in src/domains/!");

            println!("\n🎯 Next steps:");
            println!("   1. Review generated entities in src/domains/");
            println!("   2. Run 'cargo build' to compile generated code");
            println!("   3. Use entities in your application code");
            println!("   4. Run the web server to see entities in action");
        }
        Err(error) => {
            println!("❌ Code generation failed: {}", error);
            return Err(error.into());
        }
    }

    Ok(())
}
