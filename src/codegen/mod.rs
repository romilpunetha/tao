// Modular Code Generation System for TAO Ent Framework
// This module provides a clean, maintainable code generation architecture

pub mod thrift_generator;
pub mod rust_generator;
pub mod builder_generator;
pub mod ent_generator;
pub mod utils;

use std::collections::HashMap;
use crate::ent_framework::{
    FieldDefinition, EdgeDefinition, SchemaRegistry, EntityType
};

/// Main code generator orchestrator
pub struct CodeGenerator {
    registry: SchemaRegistry,
}

impl CodeGenerator {
    pub fn new(registry: SchemaRegistry) -> Self {
        Self {
            registry,
        }
    }

    /// Generate all code for entities - modular pipeline
    pub fn generate_all(&self) -> Result<HashMap<EntityType, String>, String> {
        println!("🚀 Starting modular Ent codegen pipeline");

        // Step 1: Clean up previous generated files
        self.cleanup_previous_generated_files()?;
        println!("✅ Cleaned up previous generated files");

        // Step 2: Validate schemas
        self.registry.validate().map_err(|errors| {
            format!("Schema validation failed:\n{}", errors.join("\n"))
        })?;
        println!("✅ Schema validation passed");

        // Step 3: Collect schemas from registry
        let schemas = self.collect_schemas()?;
        println!("✅ Collected {} entity schemas", schemas.len());

        // Step 4: Create domain directories
        self.create_domain_directories(&schemas)?;
        println!("✅ Created domain directories");

        // Step 4: Generate Thrift definitions (for thrift compiler)
        let thrift_gen = thrift_generator::ThriftGenerator::new(&self.registry);
        for (entity_type, (fields, _edges)) in &schemas {
            thrift_gen.generate_thrift_file(entity_type, fields)?;
        }
        println!("✅ Generated Thrift definitions");

        // Step 5: Compile Thrift files to generate Rust entity structs
        self.compile_thrift_files(&schemas)?;
        println!("✅ Compiled Thrift files to Rust entities");

        // Step 6: Generate builders with save() function
        let builder_gen = builder_generator::BuilderGenerator::new(&self.registry);
        for (entity_type, (fields, _edges)) in &schemas {
            builder_gen.generate_builder(entity_type, fields)?;
        }
        println!("✅ Generated builders with save() function");

        // Step 7: Generate Ent trait implementations
        let ent_gen = ent_generator::EntGenerator::new(&self.registry);
        for (entity_type, (fields, edges)) in &schemas {
            ent_gen.generate_ent_impl(entity_type, fields, edges)?;
        }
        println!("✅ Generated Ent implementations");

        // Step 8: Generate domain modules with entity.rs enabled
        self.generate_domain_modules_with_entities(&schemas)?;
        println!("✅ Generated domain modules with entities enabled");

        println!("🎉 Modular codegen pipeline completed successfully!");

        Ok(HashMap::new())
    }

    /// Collect schemas from registry
    fn collect_schemas(&self) -> Result<HashMap<EntityType, (Vec<FieldDefinition>, Vec<EdgeDefinition>)>, String> {
        let mut schemas = HashMap::new();

        for entity_type in self.registry.get_entity_types() {
            if let Some((fields, edges)) = self.registry.get_schema(entity_type) {
                schemas.insert(entity_type.clone(), (fields.clone(), edges.clone()));
            }
        }

        Ok(schemas)
    }

    /// Clean up all previously generated files to ensure clean builds
    fn cleanup_previous_generated_files(&self) -> Result<(), String> {
        use std::fs;
        use std::path::Path;

        let domains_dir = Path::new("src/domains");
        if !domains_dir.exists() {
            return Ok(()); // Nothing to clean up
        }

        // Read all domain directories
        let entries = fs::read_dir(domains_dir)
            .map_err(|e| format!("Failed to read domains directory: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read domain entry: {}", e))?;
            let path = entry.path();

            if path.is_dir() {
                // Remove generated files in each domain directory
                let files_to_remove = vec![
                    "entity.thrift",
                    "entity.rs",
                    "builder.rs",
                    "ent_impl.rs"
                ];

                for file_name in files_to_remove {
                    let file_path = path.join(file_name);
                    if file_path.exists() {
                        fs::remove_file(&file_path)
                            .map_err(|e| format!("Failed to remove {}: {}", file_path.display(), e))?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Create domain directories
    fn create_domain_directories(&self, schemas: &HashMap<EntityType, (Vec<FieldDefinition>, Vec<EdgeDefinition>)>) -> Result<(), String> {
        for entity_type in schemas.keys() {
            let domain_name = utils::entity_domain_name(entity_type);
            let domain_path = format!("src/domains/{}", domain_name);

            std::fs::create_dir_all(&domain_path)
                .map_err(|e| format!("Failed to create domain directory {}: {}", domain_path, e))?;
        }

        Ok(())
    }

    /// Generate domain module files
    fn generate_domain_modules(&self, schemas: &HashMap<EntityType, (Vec<FieldDefinition>, Vec<EdgeDefinition>)>) -> Result<(), String> {
        // Generate main domains mod.rs
        let mut domains_mod = String::from("// Generated domain modules\n// DO NOT EDIT\n\n");

        let mut domain_names = std::collections::HashSet::new();
        for entity_type in schemas.keys() {
            domain_names.insert(utils::entity_domain_name(entity_type));
        }

        for domain_name in domain_names {
            domains_mod.push_str(&format!("pub mod {};\n", domain_name));
        }

        std::fs::write("src/domains/mod.rs", domains_mod)
            .map_err(|e| format!("Failed to write domains/mod.rs: {}", e))?;

        // Generate individual domain mod.rs files
        for entity_type in schemas.keys() {
            let domain_name = utils::entity_domain_name(entity_type);
            let mod_content = format!(
                "// Generated domain module for {}\n// DO NOT EDIT\n// Note: entity.rs will be generated by thrift compiler\n\n// pub mod entity;  // Uncomment after running thrift compiler\npub mod builder;\npub mod ent_impl;\n\n// pub use entity::*;  // Uncomment after running thrift compiler\npub use builder::*;\npub use ent_impl::*;\n",
                entity_type
            );

            let mod_path = format!("src/domains/{}/mod.rs", domain_name);
            std::fs::write(mod_path, mod_content)
                .map_err(|e| format!("Failed to write domain mod.rs: {}", e))?;
        }
        Ok(())
    }

    /// Compile Thrift files to generate Rust entity structs
    fn compile_thrift_files(&self, schemas: &HashMap<EntityType, (Vec<FieldDefinition>, Vec<EdgeDefinition>)>) -> Result<(), String> {
        // Check if thrift compiler is available
        let thrift_check = std::process::Command::new("thrift")
            .arg("--version")
            .output();

        if thrift_check.is_err() {
            return Err("Thrift compiler not found. Please install Apache Thrift.".to_string());
        }

        // Compile each thrift file
        for entity_type in schemas.keys() {
            let domain_name = utils::entity_domain_name(entity_type);
            let thrift_file = format!("src/domains/{}/entity.thrift", domain_name);

            if !std::path::Path::new(&thrift_file).exists() {
                return Err(format!("Thrift file not found: {}", thrift_file));
            }

            // Run thrift compiler to generate Rust code
            let output = std::process::Command::new("thrift")
                .arg("--gen")
                .arg("rs")
                .arg("-out")
                .arg(format!("src/domains/{}", domain_name))
                .arg(&thrift_file)
                .output()
                .map_err(|e| format!("Failed to run thrift compiler on {}: {}", thrift_file, e))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(format!("Thrift compilation failed for {}: {}", thrift_file, stderr));
            }

            // Move the generated file to entity.rs
            let generated_file = format!("src/domains/{}/entity.rs", domain_name);
            if !std::path::Path::new(&generated_file).exists() {
                // Thrift might generate with a different name, try to find it
                let domain_dir_path = format!("src/domains/{}", domain_name);
                let domain_dir = std::path::Path::new(&domain_dir_path);
                if let Ok(entries) = std::fs::read_dir(domain_dir) {
                    for entry in entries {
                        if let Ok(entry) = entry {
                            let file_name = entry.file_name();
                            if let Some(name) = file_name.to_str() {
                                if name.ends_with(".rs") && name != "mod.rs" && name != "builder.rs" && name != "ent_impl.rs" {
                                    // Found the generated thrift file, rename it to entity.rs
                                    std::fs::rename(entry.path(), &generated_file)
                                        .map_err(|e| format!("Failed to rename {} to entity.rs: {}", name, e))?;
                                    break;
                                }
                            }
                        }
                    }
                }
            }

            println!("  ✓ Compiled {} -> entity.rs", thrift_file);
        }

        Ok(())
    }

    /// Generate domain modules with entity.rs imports enabled
    fn generate_domain_modules_with_entities(&self, schemas: &HashMap<EntityType, (Vec<FieldDefinition>, Vec<EdgeDefinition>)>) -> Result<(), String> {
        // Generate main domains mod.rs
        let mut domains_mod = String::from("// Generated domain modules\n// DO NOT EDIT\n\n");

        let mut domain_names = std::collections::HashSet::new();
        for entity_type in schemas.keys() {
            domain_names.insert(utils::entity_domain_name(entity_type));
        }

        for domain_name in domain_names {
            domains_mod.push_str(&format!("pub mod {};\n", domain_name));
        }

        std::fs::write("src/domains/mod.rs", domains_mod)
            .map_err(|e| format!("Failed to write domains/mod.rs: {}", e))?;

        // Generate individual domain mod.rs files with entity.rs enabled
        for entity_type in schemas.keys() {
            let domain_name = utils::entity_domain_name(entity_type);
            let mod_content = format!(
                "// Generated domain module for {}\n// DO NOT EDIT\n\npub mod entity;\npub mod builder;\npub mod ent_impl;\n\npub use entity::*;\npub use builder::*;\npub use ent_impl::*;\n",
                entity_type
            );
            let mod_path = format!("src/domains/{}/mod.rs", domain_name);
            std::fs::write(mod_path, mod_content)
                .map_err(|e| format!("Failed to write domain mod.rs: {}", e))?;
        }
        Ok(())
    }
} 