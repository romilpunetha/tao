#!/usr/bin/env python3
"""
Meta TAO Entity Generator

Usage: python scripts/generate_entity.py EntUserSchema
This will generate:
- schemas/ent_user_schema.thrift
- src/entities/ent_user.rs
- Updates to EntityType enum
- Updates to build.rs
- Updates to module exports
"""

import sys
import os
import re
from pathlib import Path
from typing import List, Dict, Any

def snake_case(name: str) -> str:
    """Convert CamelCase to snake_case"""
    s1 = re.sub('(.)([A-Z][a-z]+)', r'\1_\2', name)
    return re.sub('([a-z0-9])([A-Z])', r'\1_\2', s1).lower()

def extract_entity_name(schema_name: str) -> str:
    """Extract entity name from schema name (EntUserSchema -> EntUser)"""
    if schema_name.endswith('Schema'):
        return schema_name[:-6]  # Remove 'Schema'
    return schema_name

class TaoEntityGenerator:
    def __init__(self, project_root: str):
        self.project_root = Path(project_root)
        self.schemas_dir = self.project_root / "schemas"
        self.entities_dir = self.project_root / "src" / "entities"
        self.models_dir = self.project_root / "src" / "models"
        
    def generate_entity(self, schema_name: str, fields: List[Dict[str, Any]] = None):
        """Generate a complete entity from schema name"""
        entity_name = extract_entity_name(schema_name)
        entity_snake = snake_case(entity_name)
        schema_snake = snake_case(schema_name)
        
        print(f"Generating entity: {entity_name}")
        print(f"Schema file: {schema_snake}.thrift")
        print(f"Entity file: {entity_snake}.rs")
        
        # Create directories if they don't exist
        self.schemas_dir.mkdir(exist_ok=True)
        self.entities_dir.mkdir(exist_ok=True)
        
        # Generate default fields if none provided
        if fields is None:
            fields = self._get_default_fields(entity_name)
        
        # Generate files
        self._generate_schema_file(schema_name, entity_name, fields)
        self._generate_entity_implementation(entity_name)
        self._update_entity_type_enum(entity_name)
        self._update_build_rs(entity_snake)
        self._update_models_mod(entity_name, entity_snake)
        self._update_entities_mod(entity_name, entity_snake)
        
        print(f"✅ Entity {entity_name} generated successfully!")
        print(f"Run 'cargo build' to generate thrift code and compile.")
        
    def _get_default_fields(self, entity_name: str) -> List[Dict[str, Any]]:
        """Generate sensible default fields based on entity name"""
        base_fields = [
            {"id": 1, "name": "created_time", "type": "i64", "required": True},
            {"id": 2, "name": "updated_time", "type": "i64", "required": False},
        ]
        
        # Add entity-specific fields based on naming patterns
        if "User" in entity_name:
            base_fields.extend([
                {"id": 3, "name": "username", "type": "string", "required": True},
                {"id": 4, "name": "email", "type": "string", "required": True},
                {"id": 5, "name": "full_name", "type": "string", "required": False},
            ])
        elif "Post" in entity_name:
            base_fields.extend([
                {"id": 3, "name": "author_id", "type": "i64", "required": True},
                {"id": 4, "name": "content", "type": "string", "required": True},
                {"id": 5, "name": "like_count", "type": "i32", "required": True},
            ])
        elif "Event" in entity_name:
            base_fields.extend([
                {"id": 3, "name": "title", "type": "string", "required": True},
                {"id": 4, "name": "description", "type": "string", "required": False},
                {"id": 5, "name": "start_time", "type": "i64", "required": True},
            ])
        else:
            # Generic entity
            base_fields.extend([
                {"id": 3, "name": "name", "type": "string", "required": True},
                {"id": 4, "name": "description", "type": "string", "required": False},
            ])
            
        return base_fields
        
    def _generate_schema_file(self, schema_name: str, entity_name: str, fields: List[Dict[str, Any]]):
        """Generate the thrift schema file"""
        entity_snake = snake_case(entity_name)
        namespace = f"tao_db.schemas.{entity_snake}"
        
        field_definitions = []
        for field in fields:
            req_opt = "required" if field["required"] else "optional"
            field_def = f"  {field['id']}: {req_opt} {field['type']} {field['name']}"
            
            # Add comments for special fields
            if field['name'] == 'created_time':
                field_def += ", // Unix timestamp when entity was created"
            elif field['name'] == 'updated_time':
                field_def += ", // Unix timestamp when entity was last updated"
                
            field_definitions.append(field_def)
        
        thrift_content = f"""namespace rs {namespace}

// {entity_name} - TAO Entity Schema
struct {entity_name} {{
{chr(10).join(field_definitions)}
}}"""

        schema_file = self.schemas_dir / f"{entity_snake}.thrift"
        with open(schema_file, 'w') as f:
            f.write(thrift_content)
        
        print(f"✅ Generated schema: {schema_file}")
        
    def _generate_entity_implementation(self, entity_name: str):
        """Generate the entity implementation with Meta TAO methods"""
        entity_snake = snake_case(entity_name)
        
        entity_content = f"""// {entity_name} - Meta's Entity Framework database methods

use anyhow::Result;
use sqlx::Row;
use super::{{Entity, EntityContext}};
use crate::models::{{EntityType, {entity_name}}};
use crate::thrift_utils::{{thrift_serialize, thrift_deserialize}};

impl Entity for {entity_name} {{
    fn entity_type() -> EntityType {{
        EntityType::{entity_name}
    }}
}}

impl {entity_name} {{
    // Meta TAO pattern: {entity_name}::genNullable(id)
    pub async fn gen_nullable(ctx: &EntityContext, id: i64) -> Result<Option<(i64, Self)>> {{
        if let Some(obj) = ctx.db.get_object(id).await? {{
            if obj.object_type == Self::entity_type_str() {{
                let entity: Self = thrift_deserialize(&obj.data)?;
                return Ok(Some((obj.id, entity)));
            }}
        }}
        Ok(None)
    }}

    // Meta TAO pattern: {entity_name}::genMulti(ids)
    pub async fn gen_multi(ctx: &EntityContext, ids: Vec<i64>) -> Result<Vec<(i64, Self)>> {{
        let mut entities = Vec::new();
        for id in ids {{
            if let Some(entity) = Self::gen_nullable(ctx, id).await? {{
                entities.push(entity);
            }}
        }}
        Ok(entities)
    }}

    // Meta TAO pattern: {entity_name}::genAll(limit)
    pub async fn gen_all(ctx: &EntityContext, limit: Option<i32>) -> Result<Vec<(i64, Self)>> {{
        let limit = limit.unwrap_or(100);
        
        let entity_ids: Vec<i64> = sqlx::query(
            "SELECT id FROM objects WHERE object_type = ? LIMIT ?"
        )
        .bind(Self::entity_type_str())
        .bind(limit)
        .fetch_all(&ctx.db.pool)
        .await?
        .into_iter()
        .map(|row| row.get::<i64, _>(0))
        .collect();

        Self::gen_multi(ctx, entity_ids).await
    }}

    // Meta TAO pattern: {entity_name}::gen_enforce(id) - throws if not found
    pub async fn gen_enforce(ctx: &EntityContext, id: i64) -> Result<(i64, Self)> {{
        Self::gen_nullable(ctx, id).await?
            .ok_or_else(|| anyhow::anyhow!("{entity_name} with id {{}} not found", id))
    }}

    // Create new entity
    pub async fn create(ctx: &EntityContext, entity: &Self) -> Result<i64> {{
        let data = thrift_serialize(entity)?;
        let obj = ctx.db.create_object(Self::entity_type(), &data).await?;
        Ok(obj.id)
    }}

    // Update entity
    pub async fn update(ctx: &EntityContext, id: i64, entity: &Self) -> Result<()> {{
        let data = thrift_serialize(entity)?;
        ctx.db.update_object(id, &data).await
    }}

    // Delete entity
    pub async fn delete(ctx: &EntityContext, id: i64) -> Result<()> {{
        ctx.db.delete_object(id).await
    }}

    // TAO Association helpers for {entity_name}
    pub async fn get_associations(
        ctx: &EntityContext, 
        id: i64, 
        assoc_type: crate::models::AssociationType
    ) -> Result<Vec<i64>> {{
        ctx.db.get_entity_associations_by_index(id, assoc_type, None).await
    }}

    // Batch operations
    pub async fn create_many(ctx: &EntityContext, entities: &[Self]) -> Result<Vec<i64>> {{
        let mut ids = Vec::new();
        for entity in entities {{
            let id = Self::create(ctx, entity).await?;
            ids.push(id);
        }}
        Ok(ids)
    }}
}}"""

        entity_file = self.entities_dir / f"{entity_snake}.rs"
        with open(entity_file, 'w') as f:
            f.write(entity_content)
        
        print(f"✅ Generated entity implementation: {entity_file}")
        
    def _update_entity_type_enum(self, entity_name: str):
        """Update the EntityType enum in models/mod.rs"""
        models_mod = self.models_dir / "mod.rs"
        entity_snake = snake_case(entity_name)
        
        with open(models_mod, 'r') as f:
            content = f.read()
        
        # Add to enum variants
        enum_pattern = r'(#\[derive\(Debug, Clone, PartialEq\)\]\s*pub enum EntityType \{[^}]*)'
        enum_match = re.search(enum_pattern, content, re.DOTALL)
        if enum_match and entity_name not in content:
            enum_content = enum_match.group(1)
            if not enum_content.rstrip().endswith(','):
                enum_content = enum_content.rstrip() + ','
            enum_content += f"\n    {entity_name},"
            content = content.replace(enum_match.group(1), enum_content)
        
        # Add to as_str match
        as_str_pattern = r'(impl EntityType \{.*?fn as_str\(&self\) -> &\'static str \{.*?match self \{[^}]*)'
        as_str_match = re.search(as_str_pattern, content, re.DOTALL)
        if as_str_match and f"EntityType::{entity_name}" not in content:
            as_str_content = as_str_match.group(1)
            if not as_str_content.rstrip().endswith(','):
                as_str_content = as_str_content.rstrip() + ','
            as_str_content += f"\n            EntityType::{entity_name} => \"{entity_snake}\","
            content = content.replace(as_str_match.group(1), as_str_content)
        
        with open(models_mod, 'w') as f:
            f.write(content)
            
        print(f"✅ Updated EntityType enum with {entity_name}")
        
    def _update_build_rs(self, entity_snake: str):
        """Update build.rs to include new schema file"""
        build_file = self.project_root / "build.rs"
        
        with open(build_file, 'r') as f:
            content = f.read()
        
        schema_file = f"\"schemas/{entity_snake}.thrift\""
        if schema_file not in content:
            # Find the thrift_files array and add the new file
            array_pattern = r'(let thrift_files = \[[^\]]*)'
            array_match = re.search(array_pattern, content, re.DOTALL)
            if array_match:
                array_content = array_match.group(1)
                if not array_content.rstrip().endswith(','):
                    array_content = array_content.rstrip() + ','
                array_content += f"\n        {schema_file},"
                content = content.replace(array_match.group(1), array_content)
        
        with open(build_file, 'w') as f:
            f.write(content)
            
        print(f"✅ Updated build.rs with {schema_file}")
        
    def _update_models_mod(self, entity_name: str, entity_snake: str):
        """Update models/mod.rs to include new entity module and export"""
        models_mod = self.models_dir / "mod.rs"
        
        with open(models_mod, 'r') as f:
            content = f.read()
        
        # Add module declaration
        module_line = f"pub mod {entity_snake};"
        if module_line not in content:
            # Find where other entity modules are declared
            if "pub mod ent_user;" in content:
                content = content.replace("pub mod ent_user;", f"pub mod ent_user;\n{module_line}")
            else:
                # Add after tao_core
                content = content.replace("pub mod tao_core;", f"pub mod tao_core;\n{module_line}")
        
        # Add to re-exports
        export_line = f"pub use {entity_snake}::{entity_name};"
        if export_line not in content:
            if "pub use ent_user::EntUser;" in content:
                content = content.replace("pub use ent_user::EntUser;", f"pub use ent_user::EntUser;\n{export_line}")
            else:
                # Add after other exports
                content = content.replace("pub use tao_core::", f"{export_line}\npub use tao_core::")
        
        with open(models_mod, 'w') as f:
            f.write(content)
            
        print(f"✅ Updated models/mod.rs with {entity_name}")
        
    def _update_entities_mod(self, entity_name: str, entity_snake: str):
        """Update entities/mod.rs to include new entity module"""
        entities_mod = self.entities_dir / "mod.rs"
        
        with open(entities_mod, 'r') as f:
            content = f.read()
        
        # Add module declaration
        module_line = f"pub mod {entity_snake};"
        if module_line not in content:
            if "pub mod ent_user;" in content:
                content = content.replace("pub mod ent_user;", f"pub mod ent_user;\n{module_line}")
            else:
                content = content.replace("// Re-export entity implementations", f"// Re-export entity implementations\n{module_line}")
        
        # Add to re-exports
        if f"{entity_name}" not in content:
            re_export_pattern = r'(pub use crate::models::\{[^}]*)'
            re_export_match = re.search(re_export_pattern, content)
            if re_export_match:
                re_export_content = re_export_match.group(1)
                if not re_export_content.rstrip().endswith(','):
                    re_export_content = re_export_content.rstrip() + ','
                re_export_content += f" {entity_name},"
                content = content.replace(re_export_match.group(1), re_export_content)
        
        with open(entities_mod, 'w') as f:
            f.write(content)
            
        print(f"✅ Updated entities/mod.rs with {entity_name}")

def main():
    if len(sys.argv) != 2:
        print("Usage: python scripts/generate_entity.py <EntitySchemaName>")
        print("Example: python scripts/generate_entity.py EntUserSchema")
        sys.exit(1)
    
    schema_name = sys.argv[1]
    if not schema_name.endswith('Schema'):
        print("Error: Schema name must end with 'Schema'")
        print("Example: EntUserSchema, EntPostSchema, EntEventSchema")
        sys.exit(1)
    
    # Get project root (parent of scripts directory)
    script_dir = Path(__file__).parent
    project_root = script_dir.parent
    
    generator = TaoEntityGenerator(str(project_root))
    generator.generate_entity(schema_name)

if __name__ == "__main__":
    main()