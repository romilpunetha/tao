// Ent trait implementation generator
use crate::ent_framework::{FieldDefinition, EdgeDefinition, EntityType, SchemaRegistry};
use super::utils;

pub struct EntGenerator<'a> {
    _registry: &'a SchemaRegistry,
}

impl<'a> EntGenerator<'a> {
    pub fn new(registry: &'a SchemaRegistry) -> Self {
        Self { _registry: registry }
    }

    /// Generate Ent trait implementation
    pub fn generate_ent_impl(&self, entity_type: &EntityType, fields: &[FieldDefinition], edges: &[EdgeDefinition]) -> Result<(), String> {
        let domain_name = utils::entity_domain_name(entity_type);
        let struct_name = utils::entity_struct_name(entity_type);
        let ent_impl_path = format!("src/domains/{}/ent_impl.rs", domain_name);
        
        let mut ent_content = String::new();
        
        // Generate file header
        ent_content.push_str(&utils::generate_file_header("Ent trait implementation", entity_type));
        
        // Generate imports
        ent_content.push_str(&self.generate_imports(&struct_name, edges));
        
        // Generate Ent trait implementation
        ent_content.push_str(&self.generate_ent_trait_impl(entity_type, &struct_name, fields)?);
        
        // Generate edge traversal methods
        ent_content.push_str(&self.generate_edge_methods(&struct_name, edges)?);
        
        // Write to file
        std::fs::write(&ent_impl_path, ent_content)
            .map_err(|e| format!("Failed to write ent_impl file {}: {}", ent_impl_path, e))?;
        
        Ok(())
    }

    /// Generate necessary imports including cross-entity imports for edges
    fn generate_imports(&self, struct_name: &str, edges: &[EdgeDefinition]) -> String {
        let mut imports = format!(r#"use std::sync::Arc;
use crate::ent_framework::Entity;
use crate::error::AppResult;
use super::entity::{};
use crate::infrastructure::tao_core::TaoOperations;
"#, struct_name);

        // Add cross-entity imports for edge traversal, excluding current entity to avoid duplicates
        let current_entity_type = self.entity_type_from_struct_name(struct_name);
        let mut imported_entities = std::collections::HashSet::new();
        
        for edge in edges {
            // Skip importing the current entity type to avoid duplicate imports
            if edge.target_entity != current_entity_type {
                let entity_import = match edge.target_entity {
                    crate::ent_framework::EntityType::EntUser => "use crate::domains::user::EntUser;",
                    crate::ent_framework::EntityType::EntPost => "use crate::domains::post::EntPost;",
                    crate::ent_framework::EntityType::EntGroup => "use crate::domains::group::EntGroup;",
                    crate::ent_framework::EntityType::EntPage => "use crate::domains::page::EntPage;",
                    crate::ent_framework::EntityType::EntEvent => "use crate::domains::event::EntEvent;",
                    crate::ent_framework::EntityType::EntComment => "use crate::domains::comment::EntComment;",
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
    fn entity_type_from_struct_name(&self, struct_name: &str) -> crate::ent_framework::EntityType {
        match struct_name {
            "EntUser" => crate::ent_framework::EntityType::EntUser,
            "EntPost" => crate::ent_framework::EntityType::EntPost,
            "EntGroup" => crate::ent_framework::EntityType::EntGroup,
            "EntPage" => crate::ent_framework::EntityType::EntPage,
            "EntEvent" => crate::ent_framework::EntityType::EntEvent,
            "EntComment" => crate::ent_framework::EntityType::EntComment,
            _ => panic!("Unknown entity type for struct: {}", struct_name),
        }
    }

    /// Generate Entity trait implementation with comprehensive validations
    fn generate_ent_trait_impl(&self, entity_type: &EntityType, struct_name: &str, fields: &[FieldDefinition]) -> Result<String, String> {
        let mut impl_block = String::new();
        
        // Generate Entity implementation with required methods
        impl_block.push_str(&format!("impl Entity for {} {{\n", struct_name));
        impl_block.push_str(&format!("    const ENTITY_TYPE: &'static str = \"{}\";\n", entity_type));
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
                    crate::ent_framework::FieldType::String => {
                        impl_block.push_str(&format!("        // Validate {} (required)\n", field_display));
                        impl_block.push_str(&format!("        if self.{}.trim().is_empty() {{\n", field.name));
                        impl_block.push_str(&format!("            errors.push(\"{} cannot be empty\".to_string());\n", field_display));
                        impl_block.push_str("        }\n");
                    },
                    crate::ent_framework::FieldType::Bool => {
                        // Bool fields don't need empty validation
                    },
                    _ => {}
                }
            }
            
            // Generate validation based on field validators
            for validator in &field.validators {
                match validator {
                    crate::ent_framework::FieldValidator::MinLength(min) => {
                        if field.optional {
                            impl_block.push_str(&format!("        // Validate {} min length\n", field_display));
                            impl_block.push_str(&format!("        if let Some(ref val) = self.{} {{\n", field.name));
                            impl_block.push_str(&format!("            if val.len() < {} {{\n", min));
                            impl_block.push_str(&format!("                errors.push(\"{} must be at least {} characters\".to_string());\n", field_display, min));
                            impl_block.push_str("            }\n");
                            impl_block.push_str("        }\n");
                        } else {
                            impl_block.push_str(&format!("        // Validate {} min length\n", field_display));
                            impl_block.push_str(&format!("        if self.{}.len() < {} {{\n", field.name, min));
                            impl_block.push_str(&format!("            errors.push(\"{} must be at least {} characters\".to_string());\n", field_display, min));
                            impl_block.push_str("        }\n");
                        }
                    },
                    crate::ent_framework::FieldValidator::MaxLength(max) => {
                        if field.optional {
                            impl_block.push_str(&format!("        // Validate {} max length\n", field_display));
                            impl_block.push_str(&format!("        if let Some(ref val) = self.{} {{\n", field.name));
                            impl_block.push_str(&format!("            if val.len() > {} {{\n", max));
                            impl_block.push_str(&format!("                errors.push(\"{} cannot exceed {} characters\".to_string());\n", field_display, max));
                            impl_block.push_str("            }\n");
                            impl_block.push_str("        }\n");
                        } else {
                            impl_block.push_str(&format!("        // Validate {} max length\n", field_display));
                            impl_block.push_str(&format!("        if self.{}.len() > {} {{\n", field.name, max));
                            impl_block.push_str(&format!("            errors.push(\"{} cannot exceed {} characters\".to_string());\n", field_display, max));
                            impl_block.push_str("        }\n");
                        }
                    },
                    crate::ent_framework::FieldValidator::Pattern(pattern) => {
                        impl_block.push_str(&format!("        // Validate {} pattern\n", field_display));
                        impl_block.push_str(&format!("        let {}_regex = regex::Regex::new(r\"{}\").unwrap();\n", field.name, pattern.replace('\\', "\\\\")));
                        if field.optional {
                            impl_block.push_str(&format!("        if let Some(ref val) = self.{} {{\n", field.name));
                            impl_block.push_str(&format!("            if !{}_regex.is_match(val) {{\n", field.name));
                            impl_block.push_str(&format!("                errors.push(\"{} format is invalid\".to_string());\n", field_display));
                            impl_block.push_str("            }\n");
                            impl_block.push_str("        }\n");
                        } else {
                            impl_block.push_str(&format!("        if !{}_regex.is_match(&self.{}) {{\n", field.name, field.name));
                            impl_block.push_str(&format!("            errors.push(\"{} format is invalid\".to_string());\n", field_display));
                            impl_block.push_str("        }\n");
                        }
                    },
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

    /// Generate edge traversal methods based on schema with real TAO implementation
    fn generate_edge_methods(&self, struct_name: &str, edges: &[EdgeDefinition]) -> Result<String, String> {
        let mut edge_methods = format!("impl {} {{\n", struct_name);
        edge_methods.push_str("    // Edge traversal methods\n");
        edge_methods.push_str("    \n");
        
        if edges.is_empty() {
            edge_methods.push_str("    // No edges defined for this entity\n");
        } else {
            for edge in edges {
                let method_name = format!("get_{}", edge.name);
                let return_type = match edge.target_entity {
                    crate::ent_framework::EntityType::EntUser => "EntUser",
                    crate::ent_framework::EntityType::EntPost => "EntPost", 
                    crate::ent_framework::EntityType::EntGroup => "EntGroup",
                    crate::ent_framework::EntityType::EntPage => "EntPage",
                    crate::ent_framework::EntityType::EntEvent => "EntEvent",
                    crate::ent_framework::EntityType::EntComment => "EntComment",
                };
                
                let _edge_type = edge.name.to_uppercase();
                
                // Generate get method with real TAO implementation
                edge_methods.push_str(&format!("    /// Get {} via TAO edge traversal\n", edge.name.replace('_', " ")));
                edge_methods.push_str(&format!("    pub async fn {}(&self) -> AppResult<Vec<{}>> {{\n", method_name, return_type));
                edge_methods.push_str("        let tao = crate::infrastructure::tao::get_tao().await?;
");
                edge_methods.push_str(&format!("        let neighbor_ids = tao.get_neighbor_ids(self.id(), \"{}\".to_string(), Some(100)).await?;
", edge.name));
                edge_methods.push_str("        
");
                edge_methods.push_str("        let mut results = Vec::new();
");
                edge_methods.push_str("        for id in neighbor_ids {
");
                edge_methods.push_str(&format!("            if let Some(entity) = {}::gen_nullable(&(tao.clone() as Arc<dyn TaoOperations>), Some(id)).await? {{
", return_type));
                edge_methods.push_str("                results.push(entity);\n");
                edge_methods.push_str("            }\n");
                edge_methods.push_str("        }\n");
                edge_methods.push_str("        \n");
                edge_methods.push_str("        Ok(results)\n");
                edge_methods.push_str("    }\n");
                edge_methods.push_str("    \n");
                
                // Generate count method with real TAO implementation
                let count_method = format!("count_{}", edge.name);
                edge_methods.push_str(&format!("    /// Count {} via TAO edge traversal\n", edge.name.replace('_', " ")));
                edge_methods.push_str(&format!("    pub async fn {}(&self) -> AppResult<i64> {{\n", count_method));
                edge_methods.push_str("        let tao = crate::infrastructure::tao::get_tao().await?;
");
                edge_methods.push_str(&format!("        let count = tao.assoc_count(self.id(), \"{}\".to_string()).await?;\n", edge.name));
                edge_methods.push_str("        Ok(count as i64)\n");
                edge_methods.push_str("    }\n");
                edge_methods.push_str("    \n");
                
                // Generate add edge method for bidirectional relationships
                if edge.bidirectional {
                    let add_method = format!("add_{}", edge.name.trim_end_matches('s')); // Remove plural 's'
                    edge_methods.push_str(&format!("    /// Add {} association via TAO\n", edge.name.trim_end_matches('s').replace('_', " ")));
                    edge_methods.push_str(&format!("    pub async fn {}(&self, target_id: i64) -> AppResult<()> {{\n", add_method));
                    edge_methods.push_str("        let tao = crate::infrastructure::tao::get_tao().await?;
");
                    edge_methods.push_str(&format!("        let assoc = crate::infrastructure::tao::create_tao_association(self.id(), \"{}\".to_string(), target_id, None);\n", edge.name));
                    edge_methods.push_str("        tao.assoc_add(assoc).await?;\n");
                    edge_methods.push_str("        Ok(())\n");
                    edge_methods.push_str("    }\n");
                    edge_methods.push_str("    \n");
                    
                    // Generate remove edge method
                    let remove_method = format!("remove_{}", edge.name.trim_end_matches('s'));
                    edge_methods.push_str(&format!("    /// Remove {} association via TAO\n", edge.name.trim_end_matches('s').replace('_', " ")));
                    edge_methods.push_str(&format!("    pub async fn {}(&self, target_id: i64) -> AppResult<bool> {{\n", remove_method));
                    edge_methods.push_str("        let tao = crate::infrastructure::tao::get_tao().await?;
");
                    edge_methods.push_str(&format!("        tao.assoc_delete(self.id(), \"{}\".to_string(), target_id).await\n", edge.name));
                    edge_methods.push_str("    }\n");
                    edge_methods.push_str("    \n");
                }
            }
        }
        
        edge_methods.push_str("}\n\n");
        
        Ok(edge_methods)
    }

}