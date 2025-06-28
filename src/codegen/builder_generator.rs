// Builder pattern generator with save() function
use crate::ent_framework::{FieldDefinition, EntityType, SchemaRegistry};
use super::utils;

pub struct BuilderGenerator<'a> {
    _registry: &'a SchemaRegistry,
}

impl<'a> BuilderGenerator<'a> {
    pub fn new(registry: &'a SchemaRegistry) -> Self {
        Self { _registry: registry }
    }

    /// Generate builder with create() static method and save() instance method
    pub fn generate_builder(&self, entity_type: &EntityType, fields: &[FieldDefinition]) -> Result<(), String> {
        let domain_name = utils::entity_domain_name(entity_type);
        let struct_name = utils::entity_struct_name(entity_type);
        let builder_name = utils::entity_builder_name(entity_type);
        let builder_path = format!("src/domains/{}/builder.rs", domain_name);

        let mut builder_content = String::new();

        // Generate file header
        builder_content.push_str(&utils::generate_file_header("Builder pattern implementation", entity_type));

        // Generate imports
        builder_content.push_str(&self.generate_imports(&struct_name));

        // Generate builder struct
        builder_content.push_str(&self.generate_builder_struct(&builder_name, fields)?);

        // Generate builder implementation
        builder_content.push_str(&self.generate_builder_impl(entity_type, &struct_name, &builder_name, fields)?);

        // Generate entity create() method
        builder_content.push_str(&self.generate_entity_create_method(&struct_name, &builder_name, fields)?);

        // Write to file
        std::fs::write(&builder_path, builder_content)
            .map_err(|e| format!("Failed to write builder file {}: {}", builder_path, e))?;

        Ok(())
    }

    /// Generate necessary imports for builder
    fn generate_imports(&self, struct_name: &str) -> String {
        format!(r#"use crate::ent_framework::Entity;
use crate::infrastructure::tao::{{get_tao, current_time_millis}};
use crate::error::AppResult;
use super::entity::{};
use thrift::protocol::TSerializable;

"#, struct_name)
    }

    /// Generate builder struct definition
    fn generate_builder_struct(&self, builder_name: &str, fields: &[FieldDefinition]) -> Result<String, String> {
        let mut builder_struct = format!("#[derive(Debug, Default)]\npub struct {} {{\n", builder_name);

        // Add fields (all optional except ID which is generated)
        for field in fields {
            if field.name == "id" {
                continue; // ID is generated, not set by user
            }

            let rust_type = utils::field_type_to_rust(&field.field_type, false);
            builder_struct.push_str(&format!("    {}: Option<{}>,\n", field.name, rust_type));
        }

        builder_struct.push_str("}\n\n");
        Ok(builder_struct)
    }

    /// Generate builder implementation with fluent methods and save()
    fn generate_builder_impl(&self, entity_type: &EntityType, struct_name: &str, builder_name: &str, fields: &[FieldDefinition]) -> Result<String, String> {
        let mut impl_block = format!("impl {} {{\n", builder_name);

        // Generate new() method
        impl_block.push_str("    pub fn new() -> Self {\n");
        impl_block.push_str("        Self::default()\n");
        impl_block.push_str("    }\n\n");

        // Generate fluent setter methods
        for field in fields {
            if field.name == "id" {
                continue; // Skip ID field
            }

            let rust_type = utils::field_type_to_rust(&field.field_type, false);
            let method_name = &field.name;

            impl_block.push_str(&format!("    pub fn {}(mut self, {}: {}) -> Self {{\n",
                method_name, method_name, rust_type));
            impl_block.push_str(&format!("        self.{} = Some({});\n", method_name, method_name));
            impl_block.push_str("        self\n");
            impl_block.push_str("    }\n\n");
        }

        // Generate the crucial save() method
        impl_block.push_str(&self.generate_save_method(entity_type, struct_name, fields)?);

        impl_block.push_str("}\n\n");
        Ok(impl_block)
    }

    /// Generate the save() method that uses ID generator and TAO
    fn generate_save_method(&self, entity_type: &EntityType, struct_name: &str, fields: &[FieldDefinition]) -> Result<String, String> {
        let mut save_method = "    /// Save the entity to database via TAO\n".to_string();
        save_method.push_str(&format!("    pub async fn save(self) -> AppResult<{}> {{\n", struct_name));

        // Set current time for timestamp fields
        save_method.push_str("        let current_time = current_time_millis();\n\n");

        // Build the entity (without ID, TAO will generate it)
        save_method.push_str(&format!("        let entity = {} {{\n", struct_name));
        save_method.push_str("            id: 0, // TAO will generate the actual ID\n");

        for field in fields {
            if field.name == "id" {
                continue; // Already handled above
            }

            match field.name.as_str() {
                "created_time" => {
                    save_method.push_str("            created_time: current_time,\n");
                },
                "updated_time" | "time_updated" => {
                    if field.optional {
                        save_method.push_str("            updated_time: Some(current_time),\n");
                    } else {
                        save_method.push_str("            updated_time: current_time,\n");
                    }
                },
                _ => {
                    if field.optional {
                        save_method.push_str(&format!("            {}: self.{},\n", field.name, field.name));
                    } else {
                        save_method.push_str(&format!("            {}: self.{}.ok_or_else(|| crate::error::AppError::Validation(\n", field.name, field.name));
                        save_method.push_str(&format!("                \"Required field '{}' not provided\".to_string()\n", field.name));
                        save_method.push_str("            ))?,\n");
                    }
                }
            }
        }
        save_method.push_str("        };\n\n");

        // Validate the entity
        save_method.push_str("        // Validate entity before saving\n");
        save_method.push_str("        let validation_errors = entity.validate()?;\n");
        save_method.push_str("        if !validation_errors.is_empty() {\n");
        save_method.push_str("            return Err(crate::error::AppError::Validation(\n");
        save_method.push_str("                format!(\"Validation failed: {}\", validation_errors.join(\", \"))\n");
        save_method.push_str("            ));\n");
        save_method.push_str("        }\n\n");

        // Serialize using Thrift directly
        save_method.push_str("        // Serialize entity to bytes for TAO storage\n");
        save_method.push_str("        let data = {\n");
        save_method.push_str("            use thrift::protocol::TCompactOutputProtocol;\n");
        save_method.push_str("            use std::io::Cursor;\n");
        save_method.push_str("            let mut buffer = Vec::new();\n");
        save_method.push_str("            let mut cursor = Cursor::new(&mut buffer);\n");
        save_method.push_str("            let mut protocol = TCompactOutputProtocol::new(&mut cursor);\n");
        save_method.push_str("            entity.write_to_out_protocol(&mut protocol)\n");
        save_method.push_str("                .map_err(|e| crate::error::AppError::SerializationError(e.to_string()))?;\n");
        save_method.push_str("            buffer\n");
        save_method.push_str("        };\n\n");

        save_method.push_str("        // Get TAO singleton instance and save\n");
        save_method.push_str("        let tao = get_tao().await?;\n");

        // Create object using TAO - TAO handles ID generation internally
        let entity_type_str = entity_type.as_str();
        save_method.push_str(&format!("        let generated_id = tao.obj_add(\"{}\".to_string(), data, None).await?;\n\n", entity_type_str));

        // Update entity with the generated ID
        save_method.push_str("        // Create final entity with generated ID\n");
        save_method.push_str(&format!("        let mut final_entity = entity;\n"));
        save_method.push_str("        final_entity.id = generated_id;\n\n");

        save_method.push_str(&format!("        println!(\"✅ Created {} with TAO ID: {{}}\", generated_id);\n\n", struct_name));

        save_method.push_str("        Ok(final_entity)\n");
        save_method.push_str("    }\n\n");

        Ok(save_method)
    }

    /// Generate create() static method for entity
    fn generate_entity_create_method(&self, struct_name: &str, builder_name: &str, _fields: &[FieldDefinition]) -> Result<String, String> {
        let mut create_method = format!("impl {} {{\n", struct_name);

        create_method.push_str("    /// Create a new entity builder\n");
        create_method.push_str(&format!("    pub fn create() -> {} {{\n", builder_name));
        create_method.push_str(&format!("        {}::new()\n", builder_name));
        create_method.push_str("    }\n");


        // Note: Validation is handled by the Entity trait's validate() method

        create_method.push_str("}\n\n");

        Ok(create_method)
    }
}
