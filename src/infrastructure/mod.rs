// TAO Infrastructure - Database, caching, and infrastructure components

pub mod database;
pub mod cache;
pub mod id_generator;
pub mod viewer;
pub mod tao;

// Re-export infrastructure components
pub use database::{DatabaseInterface, PostgresDatabase, DatabaseTransaction, TaoAssocQueryResult, TaoObjectQueryResult, initialize_database, get_database, initialize_database_default};
pub use cache::*;
pub use id_generator::TaoIdGenerator;
pub use viewer::ViewerContext;
pub use tao::{Tao, TaoOperations, TaoId, TaoTime, TaoType, AssocType, TaoAssociation, TaoObject, AssocQuery, ObjectQuery, generate_tao_id, initialize_tao, get_tao};