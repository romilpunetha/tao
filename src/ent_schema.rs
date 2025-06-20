// Ent Schema Framework - Meta's Schema-as-Code implementation in Rust
// Provides declarative schema definition with automatic code generation

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::models::EntityType;

/// Schema definition trait - equivalent to Meta's ent.Schema
pub trait EntSchema: Send + Sync {
    /// Entity type this schema defines
    fn entity_type() -> EntityType where Self: Sized;
    
    /// Define fields for this entity
    fn fields() -> Vec<FieldDefinition> where Self: Sized;
    
    /// Define edges (relationships) for this entity  
    fn edges() -> Vec<EdgeDefinition> where Self: Sized;
    
    /// Define indexes for this entity
    fn indexes() -> Vec<IndexDefinition> where Self: Sized { Vec::new() }
    
    /// Define hooks for this entity
    fn hooks() -> Vec<HookDefinition> where Self: Sized { Vec::new() }
    
    /// Define privacy policies for this entity
    fn policies() -> Vec<PolicyDefinition> where Self: Sized { Vec::new() }
    
    /// Define annotations for this entity
    fn annotations() -> Vec<AnnotationDefinition> where Self: Sized { Vec::new() }
}

/// Field definition - equivalent to Meta's field package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDefinition {
    pub name: String,
    pub field_type: FieldType,
    pub optional: bool,
    pub default: Option<FieldDefault>,
    pub unique: bool,
    pub immutable: bool,
    pub validators: Vec<FieldValidator>,
    pub storage_key: Option<String>,
    pub annotations: Vec<AnnotationDefinition>,
}

impl FieldDefinition {
    pub fn new(name: &str, field_type: FieldType) -> Self {
        Self {
            name: name.to_string(),
            field_type,
            optional: false,
            default: None,
            unique: false,
            immutable: false,
            validators: Vec::new(),
            storage_key: None,
            annotations: Vec::new(),
        }
    }
    
    /// Mark field as optional
    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }
    
    /// Mark field as unique
    pub fn unique(mut self) -> Self {
        self.unique = true;
        self
    }
    
    /// Mark field as immutable (can't be updated after creation)
    pub fn immutable(mut self) -> Self {
        self.immutable = true;
        self
    }
    
    /// Add default value
    pub fn default_value(mut self, default: FieldDefault) -> Self {
        self.default = Some(default);
        self
    }
    
    /// Add field validator
    pub fn validate(mut self, validator: FieldValidator) -> Self {
        self.validators.push(validator);
        self
    }
}

/// Field types supported by Ent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldType {
    String,
    Int,
    Int64,
    Float,
    Bool,
    Time,
    UUID,
    Bytes,
    JSON,
    Enum(Vec<String>),
}

/// Field default values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldDefault {
    String(String),
    Int(i32),
    Int64(i64),
    Float(f64),
    Bool(bool),
    Function(String), // Function name for dynamic defaults
}

/// Field validators - equivalent to Meta's validation system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldValidator {
    MinLength(usize),
    MaxLength(usize),
    Pattern(String), // Regex pattern
    Range(f64, f64), // Min, Max for numeric types
    Custom(String),  // Custom validator function name
}

/// Edge definition - equivalent to Meta's edge package
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeDefinition {
    pub name: String,
    pub target_entity: EntityType,
    pub edge_type: EdgeType,
    pub cardinality: EdgeCardinality,
    pub required: bool,
    pub unique: bool,
    pub immutable: bool,
    pub bidirectional: bool,
    pub inverse_name: Option<String>,
    pub storage_key: Option<String>,
    pub annotations: Vec<AnnotationDefinition>,
    pub constraints: Vec<EdgeConstraint>,
}

impl EdgeDefinition {
    /// Create an edge to another entity (owner side)
    pub fn to(name: &str, target: EntityType) -> Self {
        Self {
            name: name.to_string(),
            target_entity: target,
            edge_type: EdgeType::To,
            cardinality: EdgeCardinality::OneToMany,
            required: false,
            unique: false,
            immutable: false,
            bidirectional: false,
            inverse_name: None,
            storage_key: None,
            annotations: Vec::new(),
            constraints: Vec::new(),
        }
    }
    
    /// Create an edge from another entity (back-reference)
    pub fn from(name: &str, target: EntityType, inverse_edge: &str) -> Self {
        Self {
            name: name.to_string(),
            target_entity: target,
            edge_type: EdgeType::From,
            cardinality: EdgeCardinality::ManyToOne,
            required: false,
            unique: false,
            immutable: false,
            bidirectional: false,
            inverse_name: Some(inverse_edge.to_string()),
            storage_key: None,
            annotations: Vec::new(),
            constraints: Vec::new(),
        }
    }
    
    /// Mark edge as unique (O2O or limiting cardinality)
    pub fn unique(mut self) -> Self {
        self.unique = true;
        if self.cardinality == EdgeCardinality::OneToMany {
            self.cardinality = EdgeCardinality::OneToOne;
        }
        self
    }
    
    /// Mark edge as required
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }
    
    /// Mark edge as bidirectional (automatic inverse)
    pub fn bidirectional(mut self) -> Self {
        self.bidirectional = true;
        self
    }
    
    /// Set inverse edge name for bidirectional edges
    pub fn inverse(mut self, name: &str) -> Self {
        self.inverse_name = Some(name.to_string());
        self
    }
}

/// Edge types - direction of relationship
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EdgeType {
    To,   // This entity owns the relationship
    From, // This entity is referenced by the relationship
}

/// Edge cardinality - relationship multiplicity  
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EdgeCardinality {
    OneToOne,
    OneToMany,
    ManyToOne,
    ManyToMany,
}

/// Edge constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EdgeConstraint {
    DeleteCascade,
    DeleteRestrict,
    UpdateCascade,
    UpdateRestrict,
}

/// Index definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexDefinition {
    pub name: String,
    pub fields: Vec<String>,
    pub unique: bool,
    pub storage_key: Option<String>,
}

impl IndexDefinition {
    pub fn new(name: &str, fields: Vec<&str>) -> Self {
        Self {
            name: name.to_string(),
            fields: fields.into_iter().map(|s| s.to_string()).collect(),
            unique: false,
            storage_key: None,
        }
    }
    
    pub fn unique(mut self) -> Self {
        self.unique = true;
        self
    }
}

/// Hook definition - middleware for mutations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookDefinition {
    pub name: String,
    pub hook_type: HookType,
    pub function: String, // Function name to call
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HookType {
    BeforeCreate,
    AfterCreate,
    BeforeUpdate,
    AfterUpdate,
    BeforeDelete,
    AfterDelete,
}

/// Privacy policy definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDefinition {
    pub name: String,
    pub policy_type: PolicyType,
    pub rule: String, // Rule expression or function name
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyType {
    Query,  // Controls read access
    Mutation, // Controls write access
}

/// Annotation definition for metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotationDefinition {
    pub name: String,
    pub value: String,
}

/// Schema registry - holds all defined schemas
#[derive(Default)]
pub struct SchemaRegistry {
    field_definitions: HashMap<EntityType, Vec<FieldDefinition>>,
    edge_definitions: HashMap<EntityType, Vec<EdgeDefinition>>,
}

impl SchemaRegistry {
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Register a schema
    pub fn register<T: EntSchema + 'static>(&mut self) {
        let entity_type = T::entity_type();
        let fields = T::fields();
        let edges = T::edges();
        
        self.field_definitions.insert(entity_type.clone(), fields);
        self.edge_definitions.insert(entity_type, edges);
    }
    
    /// Get field definitions for an entity
    pub fn get_fields(&self, entity_type: &EntityType) -> Option<&Vec<FieldDefinition>> {
        self.field_definitions.get(entity_type)
    }
    
    /// Get edge definitions for an entity
    pub fn get_edges(&self, entity_type: &EntityType) -> Option<&Vec<EdgeDefinition>> {
        self.edge_definitions.get(entity_type)
    }
    
    /// Validate schema consistency
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        // Validate edge references
        for (entity_type, edges) in &self.edge_definitions {
            for edge in edges {
                // Check if target entity exists
                if !self.field_definitions.contains_key(&edge.target_entity) {
                    errors.push(format!(
                        "Entity {:?} has edge '{}' pointing to undefined entity {:?}",
                        entity_type, edge.name, edge.target_entity
                    ));
                }
                
                // Validate bidirectional edge consistency
                if edge.bidirectional {
                    if let Some(target_edges) = self.edge_definitions.get(&edge.target_entity) {
                        let default_inverse = format!("{}s", entity_type.as_str());
                        let inverse_name = edge.inverse_name.as_ref()
                            .unwrap_or(&default_inverse); // Default inverse name
                        
                        if !target_edges.iter().any(|e| e.name == *inverse_name) {
                            errors.push(format!(
                                "Bidirectional edge '{}' on {:?} has no corresponding inverse '{}' on {:?}",
                                edge.name, entity_type, inverse_name, edge.target_entity
                            ));
                        }
                    }
                }
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}