// TAO Database - Clean architecture implementation

// Ent Framework - Entity schema system and code generation
// Ent Framework - Entity schema system and code generation
pub mod framework;

// Modular Code Generation System
// pub mod codegen; // Moved to framework

// Core types and primitives
pub mod core;

// TAO Infrastructure - Database, caching, and infrastructure components
pub mod infrastructure;

// Schema Definitions - Entity schemas defined by developers
pub mod schemas;

pub mod domains;
pub mod models; // Added for graph models

// Common utilities
pub mod data_seeder;
pub mod error;

// Re-exports for convenience
pub use error::{AppError, AppResult};
