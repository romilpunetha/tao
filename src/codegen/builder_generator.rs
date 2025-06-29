// Builder pattern generator with save() function
use super::utils;
use crate::ent_framework::{EntityType, FieldDefinition, SchemaRegistry};

pub struct BuilderGenerator<'a> {
    _registry: &'a SchemaRegistry,
}

impl<'a> BuilderGenerator<'a> {
    pub fn new(registry: &'a SchemaRegistry) -> Self {
        Self {
            _registry: registry,
        }
    }

    /// Generate builder with create() static method and save() instance method
    pub fn generate_builder(
        &self,
        entity_type: &EntityType,
        fields: &[FieldDefinition],
    ) -> Result<(), String> {
        let domain_name = utils::entity_domain_name(entity_type);
        let struct_name = utils::entity_struct_name(entity_type);
        let builder_name = utils::entity_builder_name(entity_type);
        let builder_path = format!("src/domains/{}/builder.rs", domain_name);

        let mut builder_content = String::new();

        // Generate file header
        builder_content.push_str(&utils::generate_file_header(
            "Builder pattern implementation",
            entity_type,
        ));

        // Generate imports
        builder_content.push_str(&self.generate_imports(&struct_name));

        // Generate builder struct
        builder_content.push_str(&self.generate_builder_struct(&builder_name, fields)?);

        // Generate builder implementation
        builder_content.push_str(&self.generate_builder_impl(
            entity_type,
            &struct_name,
            &builder_name,
            fields,
        )?);

        // Generate EntBuilder implementation
        builder_content.push_str(&self.generate_ent_builder_impl(
            entity_type,
            &struct_name,
            &builder_name,
            fields,
        )?);

        // Generate entity create() method
        builder_content.push_str(&self.generate_entity_create_method(
            &struct_name,
            &builder_name,
            fields,
        )?);

        // Write to file
        std::fs::write(&builder_path, builder_content)
            .map_err(|e| format!("Failed to write builder file {}: {}", builder_path, e))?;

        Ok(())
    }

    /// Generate necessary imports for builder
    fn generate_imports(&self, struct_name: &str) -> String {
        format!(
            r#"use crate::ent_framework::ent_builder::EntBuilder;
use crate::ent_framework::Entity;
use crate::infrastructure::tao::{{current_time_millis, Tao}};
use crate::infrastructure::tao_core::TaoOperations;
use crate::error::AppResult;
use super::entity::{};
use thrift::protocol::TSerializable;
use thrift::protocol::TCompactOutputProtocol; // Added for serialization
use std::io::Cursor; // Added for serialization
use crate::infrastructure::global_tao::get_global_tao; // Import global_tao
use async_trait::async_trait;

"#,
            struct_name
        )
    }

    /// Generate builder struct definition
    fn generate_builder_struct(
        &self,
        builder_name: &str,
        fields: &[FieldDefinition],
    ) -> Result<String, String> {
        let mut builder_struct = format!(
            "#[derive(Debug, Default)]\npub struct {} {{\n",
            builder_name
        );

        // Add fields (all optional, including ID for save() method)
        builder_struct.push_str(&format!("    {}: Option<{}>,\n", "id", "i64"));
        for field in fields {
            let rust_type = utils::field_type_to_rust(&field.field_type, false);
            builder_struct.push_str(&format!("    {}: Option<{}>,\n", field.name, rust_type));
        }

        builder_struct.push_str("}\n\n");
        Ok(builder_struct)
    }

    /// Generate builder implementation with fluent methods and save()
    fn generate_builder_impl(
        &self,
        entity_type: &EntityType,
        struct_name: &str,
        builder_name: &str,
        fields: &[FieldDefinition],
    ) -> Result<String, String> {
        let mut impl_block = format!("impl {} {{\n", builder_name);

        // Generate new() method
        impl_block.push_str("    pub fn new() -> Self {\n");
        impl_block.push_str("        Self::default()\n");
        impl_block.push_str("    }\n\n");

        // Generate fluent setter methods
        for field in fields {
            let rust_type = utils::field_type_to_rust(&field.field_type, false);
            let method_name = &field.name;

            impl_block.push_str(&format!(
                "    pub fn {}(mut self, {}: {}) -> Self {{\n",
                method_name, method_name, rust_type
            ));
            impl_block.push_str(&format!(
                "        self.{} = Some({});\n",
                method_name, method_name
            ));
            impl_block.push_str("        self\n");
            impl_block.push_str("    }\n\n");
        }

        // Generate the build() method that just builds the object
        impl_block.push_str(&self.generate_build_method(entity_type, struct_name, fields)?);

        // Generate the savex() method that saves to database
        impl_block.push_str(&self.generate_savex_method(entity_type, struct_name, fields)?);

        impl_block.push_str("}\n\n");
        Ok(impl_block)
    }

    /// Generate the build() method that just builds the object without saving
    fn generate_build_method(
        &self,
        _entity_type: &EntityType,
        struct_name: &str,
        fields: &[FieldDefinition],
    ) -> Result<String, String> {
        let mut build_method = "    /// Build the entity without saving to database\n".to_string();
        build_method.push_str(&format!(
            "    pub fn build(self, id: i64) -> Result<{}, String> {{\n",
            struct_name
        ));

        // Set current time for timestamp fields
        build_method.push_str("        let current_time = current_time_millis();\n\n");

        // Build the entity
        build_method.push_str(&format!("        let entity = {} {{\n", struct_name));
        build_method.push_str("            id,\n");
        for field in fields {
            match field.name.as_str() {
                "created_time" => {
                    build_method.push_str("            created_time: current_time,\n");
                }
                "updated_time" | "time_updated" => {
                    if field.optional {
                        build_method.push_str("            updated_time: Some(current_time),\n");
                    } else {
                        build_method.push_str("            updated_time: current_time,\n");
                    }
                }
                _ => {
                    if field.optional {
                        build_method.push_str(&format!(
                            "            {}: self.{},\n",
                            field.name, field.name
                        ));
                    } else {
                        build_method.push_str(&format!(
                            "            {}: self.{}.ok_or_else(|| \n",
                            field.name, field.name
                        ));
                        build_method.push_str(&format!(
                            "                \"Required field '{}' not provided\".to_string()\n",
                            field.name
                        ));
                        build_method.push_str("            )?,\n");
                    }
                }
            }
        }
        build_method.push_str("        };\n\n");

        build_method.push_str("        Ok(entity)\n");
        build_method.push_str("    }\n\n");

        Ok(build_method)
    }

    /// Generate the savex() method that uses the generic TAO create method
    fn generate_savex_method(
        &self,
        _entity_type: &EntityType,
        struct_name: &str,
        _fields: &[FieldDefinition],
    ) -> Result<String, String> {
        let mut save_method = "    /// Save the entity to database via TAO\n".to_string();
        save_method.push_str(&format!(
            "    pub async fn savex(self) -> AppResult<{}> {{\n",
            struct_name
        ));
        save_method.push_str("        let tao = get_global_tao()?.clone();\n");
        save_method.push_str("        tao.create(self, None).await\n");
        save_method.push_str("    }\n\n");
        Ok(save_method)
    }

    /// Generate the EntBuilder trait implementation for the builder
    fn generate_ent_builder_impl(
        &self,
        entity_type: &EntityType,
        struct_name: &str,
        builder_name: &str,
        fields: &[FieldDefinition],
    ) -> Result<String, String> {
        let mut impl_block = format!("#[async_trait]\nimpl EntBuilder for {} {{\n", builder_name);
        impl_block.push_str(&format!("    type EntityType = {};\n\n", struct_name));

        // Implement build()
        impl_block.push_str("    fn build(self, id: i64) -> Result<Self::EntityType, String> {\n");
        impl_block.push_str("        let current_time = current_time_millis();\n\n");
        impl_block.push_str(&format!("        let entity = {} {{\n", struct_name));
        impl_block.push_str("            id,\n");
        for field in fields {
            match field.name.as_str() {
                "created_time" => {
                    impl_block.push_str("            created_time: current_time,\n");
                }
                "updated_time" | "time_updated" => {
                    if field.optional {
                        impl_block.push_str("            updated_time: Some(current_time),\n");
                    } else {
                        impl_block.push_str("            updated_time: current_time,\n");
                    }
                }
                _ => {
                    if field.optional {
                        impl_block.push_str(&format!(
                            "            {}: self.{},\n",
                            field.name, field.name
                        ));
                    } else {
                        impl_block.push_str(&format!(
                            "            {}: self.{}.ok_or_else(|| \n",
                            field.name, field.name
                        ));
                        impl_block.push_str(&format!(
                            "                \"Required field '{}' not provided\".to_string()\n",
                            field.name
                        ));
                        impl_block.push_str("            )?,\n");
                    }
                }
            }
        }
        impl_block.push_str("        };\n\n");
        impl_block.push_str("        Ok(entity)\n");
        impl_block.push_str("    }\n\n");

        // Implement entity_type()
        impl_block.push_str("    fn entity_type() -> &'static str {\n");
        let entity_type_str = entity_type.as_str();
        impl_block.push_str(&format!("        \"{}\"\n", entity_type_str));
        impl_block.push_str("    }\n");

        impl_block.push_str("}\n\n");
        Ok(impl_block)
    }

    /// Generate create() static method for entity
    fn generate_entity_create_method(
        &self,
        struct_name: &str,
        builder_name: &str,
        _fields: &[FieldDefinition],
    ) -> Result<String, String> {
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
