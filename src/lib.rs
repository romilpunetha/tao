// TAO Database - Clean architecture implementation

// Ent Framework - Entity schema system and code generation
pub mod ent_framework;

// Modular Code Generation System
pub mod codegen;

// Core types and primitives
pub mod core;

// TAO Infrastructure - Database, caching, and infrastructure components
pub mod infrastructure;

// Schema Definitions - Entity schemas defined by developers
pub mod schemas;

// Domain-Driven Organization - Entities organized by domain
pub mod domains;

// Common utilities
pub mod error;
pub mod data_seeder;

// Re-exports for convenience  
pub use error::{AppError, AppResult};
