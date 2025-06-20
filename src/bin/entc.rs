// entc - Ent Code Generator CLI
// Equivalent to Meta's entc command for generating entity code from schemas

use std::env;
use std::fs;
use std::path::Path;
use tao_database::{
    framework::EntCodeGenerator,
    schemas::{create_schema_registry, validate_schemas},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: entc <command>");
        eprintln!("Commands:");
        eprintln!("  generate  - Generate entity code from schemas");
        eprintln!("  validate  - Validate schema definitions");
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
    println!("🚀 TAO Ent Code Generator - Generating complete entities with Thrift + TAO functionality...");

    // Validate schemas first
    if let Err(errors) = validate_schemas() {
        eprintln!("❌ Schema validation failed:");
        for error in errors {
            eprintln!("  - {}", error);
        }
        return Err("Schema validation failed".into());
    }

    println!("✅ Schema validation passed");

    // Create schema registry and code generator
    let registry = create_schema_registry();
    let generator = EntCodeGenerator::new(registry);

    // Generate complete entities with built-in Thrift serialization + TAO functionality
    match generator.generate_all() {
        Ok(_generated_code) => {
            // All output messages are handled inside generate_all()
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