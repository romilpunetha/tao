// Ent trait implementation generator
use super::utils;
use crate::framework::schema::ent_schema::{
    EdgeDefinition, EntityType, FieldDefinition, SchemaRegistry,
};

pub struct EntGenerator<'a> {
    _registry: &'a SchemaRegistry,
}

impl<'a> EntGenerator<'a> {
    pub fn new(registry: &'a SchemaRegistry) -> Self {
        Self {
            _registry: registry,
        }
    }

    /// Generate Ent trait implementation
    pub fn generate_ent_impl(
        &self,
        entity_type: &EntityType,
        fields: &[FieldDefinition],
        edges: &[EdgeDefinition],
    ) -> Result<(), String> {
        let domain_name = utils::entity_domain_name(entity_type);
        let struct_name = utils::entity_struct_name(entity_type);
        let ent_impl_path = format!("src/domains/{}/ent_impl.rs", domain_name);

        let mut ent_content = String::new();

        // Generate file header
        ent_content.push_str(&utils::generate_file_header(
            "Ent trait implementation",
            entity_type,
        ));

        // Generate imports
        ent_content.push_str(&self.generate_imports(&struct_name, edges));

        // Generate Ent trait implementation (Entity trait methods)
        ent_content.push_str(&self.generate_ent_trait_impl(entity_type, &struct_name, fields)?);

        // Start a new impl block for associated functions
        ent_content.push_str(&format!("impl {} {{\n", struct_name));

        // Generate from_tao_object method (associated function)
        ent_content.push_str(&self.generate_from_tao_object_method_content(&struct_name)?);

        // Generate edge traversal methods (associated functions)
        ent_content.push_str(&self.generate_edge_methods_content(&struct_name, edges)?);

        // Close the impl block
        ent_content.push_str("}\n\n");

        // Write to file
        std::fs::write(&ent_impl_path, ent_content)
            .map_err(|e| format!("Failed to write ent_impl file {}: {}", ent_impl_path, e))?;

        Ok(())
    }

    /// Generate necessary imports including cross-entity imports for edges
    fn generate_imports(&self, struct_name: &str, edges: &[EdgeDefinition]) -> String {
        let mut imports = String::from("use std::sync::Arc;\n");
        imports.push_str("use crate::framework::entity::ent_trait::Entity;\n");
        imports.push_str("use crate::error::AppResult;\n");
        imports.push_str(&format!("use super::entity::{};\n", struct_name));
        imports.push_str(
            "use crate::infrastructure::tao_core::tao_core::{TaoOperations, TaoObject};\n",
        );
        imports.push_str("use crate::infrastructure::tao_core::tao::Tao;\n");
        imports.push_str("use thrift::protocol::{TCompactInputProtocol, TSerializable};\n");
        imports.push_str("use crate::infrastructure::global_tao::get_global_tao;\n");
        imports.push_str("use std::io::Cursor;\n");
        imports.push_str("use regex;\n");

        // Add cross-entity imports for edge traversal, excluding current entity to avoid duplicates
        let current_entity_type = self.entity_type_from_struct_name(struct_name);
        let mut imported_entities = std::collections::HashSet::new();

        for edge in edges {
            // Skip importing the current entity type to avoid duplicate imports
            if edge.target_entity != current_entity_type {
                let entity_import = match edge.target_entity {
                    EntityType::EntUser => "use crate::domains::user::EntUser;",
                    EntityType::EntPost => "use crate::domains::post::EntPost;",
                    EntityType::EntGroup => "use crate::domains::group::EntGroup;",
                    EntityType::EntPage => "use crate::domains::page::EntPage;",
                    EntityType::EntEvent => "use crate::domains::event::EntEvent;",
                    EntityType::EntComment => "use crate::domains::comment::EntComment;",
                };
                imported_entities.insert(entity_import);
            }
        }

        for import in imported_entities {
            imports.push_str(import);
            imports.push('\n');
        }
        imports.push('\n');

        imports
    }

    /// Helper to determine entity type from struct name
    fn entity_type_from_struct_name(
        &self,
        struct_name: &str,
    ) -> crate::framework::schema::ent_schema::EntityType {
        match struct_name {
            "EntUser" => crate::framework::schema::ent_schema::EntityType::EntUser,
            "EntPost" => crate::framework::schema::ent_schema::EntityType::EntPost,
            "EntGroup" => crate::framework::schema::ent_schema::EntityType::EntGroup,
            "EntPage" => crate::framework::schema::ent_schema::EntityType::EntPage,
            "EntEvent" => crate::framework::schema::ent_schema::EntityType::EntEvent,
            "EntComment" => crate::framework::schema::ent_schema::EntityType::EntComment,
            _ => panic!("Unknown entity type for struct: {}", struct_name),
        }
    }

    /// Generate Entity trait implementation with comprehensive validations
    fn generate_ent_trait_impl(
        &self,
        entity_type: &EntityType,
        struct_name: &str,
        fields: &[FieldDefinition],
    ) -> Result<String, String> {
        let mut impl_block = String::new();

        // Generate Entity implementation with required methods
        impl_block.push_str(&format!("impl Entity for {} {{\n", struct_name));
        impl_block.push_str(&format!(
            "    const ENTITY_TYPE: &'static str = \"{}\";\n",
            entity_type
        ));
        impl_block.push_str("    \n");
        impl_block.push_str("    fn id(&self) -> i64 {\n");
        impl_block.push_str("        self.id\n");
        impl_block.push_str("    }\n\n");
        impl_block.push_str("    fn validate(&self) -> AppResult<Vec<String>> {\n");
        impl_block.push_str("        let mut errors = Vec::new();\n");
        impl_block.push_str("        \n");

        // Generate comprehensive validation based on schema
        for field in fields {
            if field.name == "id" || field.name == "created_time" {
                continue; // Skip ID and timestamp fields
            }

            let field_display = field.name.replace('_', " ");

            // Generate validation for required fields
            if !field.optional {
                match field.field_type {
                    crate::framework::schema::ent_schema::FieldType::String => {
                        impl_block.push_str(&format!(
                            "        // Validate {} (required)\n",
                            field_display
                        ));
                        impl_block.push_str(&format!(
                            "        if self.{}.trim().is_empty() {{\n",
                            field.name
                        ));
                        impl_block.push_str(&format!(
                            "            errors.push(\"{} cannot be empty\".to_string());\n",
                            field_display
                        ));
                        impl_block.push_str("        }\n");
                    }
                    crate::framework::schema::ent_schema::FieldType::Bool => {
                        // Bool fields don't need empty validation
                    }
                    _ => {}
                }
            }

            // Generate validation based on field validators
            for validator in &field.validators {
                match validator {
                    crate::framework::schema::ent_schema::FieldValidator::MinLength(min) => {
                        if field.optional {
                            impl_block.push_str(&format!(
                                "        // Validate {} min length\n",
                                field_display
                            ));
                            impl_block.push_str(&format!(
                                "        if let Some(ref val) = self.{} {{\n",
                                field.name
                            ));
                            impl_block
                                .push_str(&format!("            if val.len() < {} {{\n", min));
                            impl_block.push_str(&format!("                errors.push(\"{} must be at least {} characters\".to_string());\n", field_display, min));
                            impl_block.push_str("            }\n");
                            impl_block.push_str("        }\n");
                        } else {
                            impl_block.push_str(&format!(
                                "        // Validate {} min length\n",
                                field_display
                            ));
                            impl_block.push_str(&format!(
                                "        if self.{}.len() < {} {{\n",
                                field.name, min
                            ));
                            impl_block.push_str(&format!("            errors.push(\"{} must be at least {} characters\".to_string());\n", field_display, min));
                            impl_block.push_str("        }\n");
                        }
                    }
                    crate::framework::schema::ent_schema::FieldValidator::MaxLength(max) => {
                        if field.optional {
                            impl_block.push_str(&format!(
                                "        // Validate {} max length\n",
                                field_display
                            ));
                            impl_block.push_str(&format!(
                                "        if let Some(ref val) = self.{} {{\n",
                                field.name
                            ));
                            impl_block
                                .push_str(&format!("            if val.len() > {} {{\n", max));
                            impl_block.push_str(&format!("                errors.push(\"{} cannot exceed {} characters\".to_string());\n", field_display, max));
                            impl_block.push_str("            }\n");
                            impl_block.push_str("        }\n");
                        } else {
                            impl_block.push_str(&format!(
                                "        // Validate {} max length\n",
                                field_display
                            ));
                            impl_block.push_str(&format!(
                                "        if self.{}.len() > {} {{\n",
                                field.name, max
                            ));
                            impl_block.push_str(&format!("            errors.push(\"{} cannot exceed {} characters\".to_string());\n", field_display, max));
                            impl_block.push_str("        }\n");
                        }
                    }
                    crate::framework::schema::ent_schema::FieldValidator::Pattern(pattern) => {
                        impl_block
                            .push_str(&format!("        // Validate {} pattern\n", field_display));
                        impl_block.push_str(&format!(
                            "        let {}_regex = regex::Regex::new(r\"{}\").unwrap();\n",
                            field.name,
                            pattern.replace('\\', "\\")
                        ));
                        if field.optional {
                            impl_block.push_str(&format!(
                                "        if let Some(ref val) = self.{} {{\n",
                                field.name
                            ));
                            impl_block.push_str(&format!(
                                "            if !{}_regex.is_match(val) {{\n",
                                field.name
                            ));
                            impl_block.push_str(&format!("                errors.push(\"{} format is invalid\".to_string());\n", field_display));
                            impl_block.push_str("            }\n");
                            impl_block.push_str("        }\n");
                        } else {
                            impl_block.push_str(&format!(
                                "        if !{}_regex.is_match(&self.{}) {{\n",
                                field.name, field.name
                            ));
                            impl_block.push_str(&format!(
                                "            errors.push(\"{} format is invalid\".to_string());\n",
                                field_display
                            ));
                            impl_block.push_str("        }\n");
                        }
                    }
                    _ => {} // Handle other validators as needed
                }
            }

            impl_block.push_str("        \n");
        }

        impl_block.push_str("        Ok(errors)\n");
        impl_block.push_str("    }\n");
        impl_block.push_str("}\n\n");

        Ok(impl_block)
    }

    /// Generates the from_tao_object method for the entity struct.
    fn generate_from_tao_object_method_content(&self, struct_name: &str) -> Result<String, String> {
        let mut method_block = String::new();
        method_block.push_str("    /// Create an entity from a TaoObject\n");
        method_block.push_str(&format!(
            "    pub(crate) async fn from_tao_object(tao_obj: TaoObject) -> AppResult<Option<{}>> {{\n",
            struct_name
        ));
        method_block.push_str(&format!(
            "        if tao_obj.otype != {}::ENTITY_TYPE {{\n",
            struct_name
        ));
        method_block.push_str("            return Ok(None);\n");
        method_block.push_str("        }\n");
        method_block.push_str("        \n");
        method_block.push_str("        let mut cursor = Cursor::new(&tao_obj.data);\n");
        method_block
            .push_str("        let mut protocol = TCompactInputProtocol::new(&mut cursor);\n");
        method_block.push_str(&format!(
            "        let mut entity = {}::read_from_in_protocol(&mut protocol)\n",
            struct_name
        ));
        method_block.push_str("            .map_err(|e| crate::error::AppError::SerializationError(e.to_string()))?;\n");
        method_block.push_str("        \n");
        method_block.push_str("        Ok(Some(entity))\n");
        method_block.push_str("    }\n\n");
        Ok(method_block)
    }

    /// Generate edge traversal methods based on schema with real TAO implementation
    fn generate_edge_methods_content(
        &self,
        struct_name: &str,
        edges: &[EdgeDefinition],
    ) -> Result<String, String> {
        let mut edge_methods = String::new();

        if edges.is_empty() {
            edge_methods.push_str("    // No edges defined for this entity\n");
        } else {
            edge_methods.push_str("    // Edge traversal methods\n");
            edge_methods.push_str("    \n");
            for edge in edges {
                let method_name = format!("get_{}", edge.name);
                let return_type = match edge.target_entity {
                    EntityType::EntUser => "EntUser",
                    EntityType::EntPost => "EntPost",
                    EntityType::EntGroup => "EntGroup",
                    EntityType::EntPage => "EntPage",
                    EntityType::EntEvent => "EntEvent",
                    EntityType::EntComment => "EntComment",
                };

                let _edge_type = edge.name.to_uppercase();

                // Generate get method with real TAO implementation
                edge_methods.push_str(&format!(
                    "    /// Get {} via TAO edge traversal\n",
                    edge.name.replace('_', " ")
                ));
                edge_methods.push_str(&format!(
                    "    pub async fn {}(&self) -> AppResult<Vec<{}>> {{\n", // Removed tao parameter
                    method_name, return_type
                ));
                edge_methods.push_str("        let tao = get_global_tao()?.clone();\n"); // Get global tao instance
                edge_methods.push_str(&format!("        let neighbor_ids = tao.get_neighbor_ids(self.id(), \"{}\".to_string(), Some(100)).await?;\n", edge.name));
                edge_methods.push('\n');
                edge_methods.push_str("        let mut results = Vec::new();\n");
                edge_methods.push_str("        for id in neighbor_ids {\n");
                edge_methods
                    .push_str("            if let Some(tao_obj) = tao.obj_get(id).await? {\n");
                edge_methods.push_str(&format!(
                    "                if let Some(entity) = {}::from_tao_object(tao_obj).await? {{\n",
                    return_type
                )); // Removed tao parameter
                edge_methods.push_str("                    results.push(entity);\n");
                edge_methods.push_str("                }\n");
                edge_methods.push_str("            }\n");
                edge_methods.push_str("        }\n");
                edge_methods.push_str("        \n");
                edge_methods.push_str("        Ok(results)\n");
                edge_methods.push_str("    }\n");
                edge_methods.push_str("    \n");

                // Generate count method with real TAO implementation
                let count_method = format!("count_{}", edge.name);
                edge_methods.push_str(&format!(
                    "    /// Count {} via TAO edge traversal\n",
                    edge.name.replace('_', " ")
                ));
                edge_methods.push_str(&format!(
                    "    pub async fn {}(&self) -> AppResult<i64> {{\n", // Removed tao parameter
                    count_method
                ));
                edge_methods.push_str("        let tao = get_global_tao()?.clone();\n"); // Get global tao instance
                edge_methods.push_str(&format!(
                    "        let count = tao.assoc_count(self.id(), \"{}\".to_string()).await?;\n",
                    edge.name
                ));
                edge_methods.push_str("        Ok(count as i64)\n");
                edge_methods.push_str("    }\n");
                edge_methods.push_str("    \n");

                // Generate add edge method for bidirectional relationships
                if edge.bidirectional {
                    let add_method = format!("add_{}", edge.name.trim_end_matches('s')); // Remove plural 's'
                    edge_methods.push_str(&format!(
                        "    /// Add {} association via TAO\n",
                        edge.name.trim_end_matches('s').replace('_', " ")
                    ));
                    edge_methods.push_str(&format!(
                        "    pub async fn {}(&self, target_id: i64) -> AppResult<()> {{\n",
                        add_method
                    )); // Removed tao parameter
                    edge_methods.push_str("        let tao = get_global_tao()?.clone();\n"); // Get global tao instance
                    edge_methods.push_str(&format!("        // Fetch the {} to ensure it exists before creating an association\n", return_type));
                    edge_methods.push_str(&format!(
                        "        let _{} = {}::from_tao_object(\n",
                        edge.name.trim_end_matches('s'),
                        return_type
                    ));
                    edge_methods.push_str("            tao.obj_get(target_id).await?\n");
                    edge_methods.push_str(&format!("                .ok_or_else(|| crate::error::AppError::NotFound(format!(\"{} with id {{}} not found\", target_id)))?\n", return_type));
                    edge_methods.push_str("        ).await?;\n");
                    edge_methods.push('\n');
                    edge_methods.push_str(&format!("        let assoc = crate::infrastructure::tao_core::tao_core::create_tao_association(self.id(), \"{}\".to_string(), target_id, None);\n", edge.name));
                    edge_methods.push_str("        tao.assoc_add(assoc).await?;\n");
                    edge_methods.push_str("        Ok(())\n");
                    edge_methods.push_str("    }\n");
                    edge_methods.push_str("    \n");

                    // Generate remove edge method
                    let remove_method = format!("remove_{}", edge.name.trim_end_matches('s'));
                    edge_methods.push_str(&format!(
                        "    /// Remove {} association via TAO\n",
                        edge.name.trim_end_matches('s').replace('_', " ")
                    ));
                    edge_methods.push_str(&format!(
                        "    pub async fn {}(&self, target_id: i64) -> AppResult<bool> {{\n",
                        remove_method
                    )); // Removed tao parameter
                    edge_methods.push_str("        let tao = get_global_tao()?.clone();\n"); // Get global tao instance
                    edge_methods.push_str(&format!("        tao.assoc_delete(self.id(), \"{}\".to_string(), target_id).await\n", edge.name));
                    edge_methods.push_str("    }\n");
                    edge_methods.push_str("    \n");
                }
            }
        }
        Ok(edge_methods)
    }
}
