// TAO Ent Framework - Entity schema system and traits

pub mod ent_schema;
pub mod ent_hooks;
pub mod ent_privacy;
pub mod ent_trait;

// Re-export all ent framework types for convenience
pub use ent_schema::*;
pub use ent_trait::Entity;

// Re-export TAO from infrastructure
pub use crate::infrastructure::{
    // Main TAO interface for developers
    Tao, TaoConfig, TaoFactory, initialize_tao, initialize_tao_with_config, get_tao,
    // Core TAO types and operations
    TaoOperations, TaoId, TaoTime, TaoType, AssocType, TaoAssociation, TaoObject, 
    AssocQuery, ObjectQuery, generate_tao_id, create_tao_association, current_time_millis
};