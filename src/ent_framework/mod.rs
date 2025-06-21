// TAO Ent Framework - Entity schema system and traits

pub mod ent_schema;
pub mod ent_hooks;
pub mod ent_privacy;
pub mod ent_trait;

// Re-export all ent framework types for convenience
pub use ent_schema::*;
pub use ent_trait::Entity;

// Re-export TAO from infrastructure
pub use crate::infrastructure::tao::{Tao, TaoOperations, TaoId, TaoTime, TaoType, AssocType, TaoAssociation, TaoObject, AssocQuery, ObjectQuery, generate_tao_id, initialize_tao, get_tao};