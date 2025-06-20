// TAO Database - Production-ready async implementation with SQLx and Thrift

// Core Framework - Ent schema system and code generation
pub mod framework;

// Core Infrastructure - Database, caching, utilities
// pub mod core;

// TAO Infrastructure - High-level TAO operations
// pub mod infrastructure;

// Schema Definitions - Entity schemas defined by developers
pub mod schemas;

// Generated Code - Complete entities with Thrift + TAO functionality
pub mod models;

// Domain-Driven Organization - Entities organized by domain
pub mod domains;

// Application Layer
// pub mod config;
// pub mod app_state;
// pub mod services;

// Common utilities
pub mod error;

// Re-exports for convenience  
pub use error::{AppError, AppResult};
