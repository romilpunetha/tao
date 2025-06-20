// TAO Ent Framework - Core framework components for entity schema and code generation

pub mod ent_schema;
pub mod ent_codegen;
pub mod ent_hooks;
pub mod ent_privacy;
pub mod ent_trait;

// Re-export all framework types for convenience
pub use ent_schema::*;
pub use ent_codegen::EntCodeGenerator;
pub use ent_trait::Ent;