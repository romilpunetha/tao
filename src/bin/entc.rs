// entc - Ent Code Generator CLI
// Equivalent to Meta's entc command for generating entity code from schemas

use std::env;
use std::fs;
use std::path::Path;
use tao_database::{
    codegen::CodeGenerator,
    schemas::{create_schema_registry, validate_schemas},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: entc <command>");
        eprintln!("Commands:");
        eprintln!("  generate - Generate entity code from schemas");
        eprintln!("  validate - Validate schema definitions");
        return Ok(());
    }

    match args[1].as_str() {
        "generate" => generate_code()?,
        "validate" => validate_schemas_cmd()?,
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            eprintln!("Use 'generate' or 'validate'");
        }
    }

    Ok(())
}

fn generate_code() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 TAO Code Generator - Generating entities with builder pattern and save() function...");

    // Create schema registry with all schemas
    let registry = create_schema_registry();
    println!("✅ Created schema registry with all entities");

    // Create code generator
    let generator = CodeGenerator::new(registry);

    // Generate all code
    match generator.generate_all() {
        Ok(_) => {
            println!("🎉 Code generation completed successfully!");
            println!();
            println!("Generated files for each entity:");
            println!("  - src/domains/<entity>/entity.thrift   (for thrift compiler)");
            println!("  - src/domains/<entity>/builder.rs      (builder with save() function)");
            println!("  - src/domains/<entity>/ent_impl.rs     (Ent trait implementation)");
            println!("  - src/domains/<entity>/mod.rs          (domain module)");
            println!();
            println!("🔧 Next steps:");
            println!("  1. Run thrift compiler on .thrift files to generate entity.rs");
            println!("  2. Use Entity::create().field().save().await pattern");
            println!("  3. Compile and test the generated code");
        },
        Err(error) => {
            eprintln!("❌ Entity generation failed: {}", error);
            return Err(error.into());
        }
    }

    Ok(())
}


fn validate_schemas_cmd() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Validating schema definitions...");

    match validate_schemas() {
        Ok(()) => {
            println!("✅ All schemas are valid!");
        },
        Err(errors) => {
            eprintln!("❌ Schema validation failed:");
            for error in errors {
                eprintln!("  - {}", error);
            }
            return Err("Schema validation failed".into());
        }
    }

    Ok(())
}