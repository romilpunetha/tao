// TAO Database - Production-ready async implementation with SQLx and Thrift

// Core modules
pub mod cache;
pub mod database;
pub mod entities;
pub mod thrift_utils;
pub mod models;
pub mod viewer;
pub mod id_generator;
pub mod tao_operations;
pub mod inverse_associations;

// Ent Framework modules
pub mod ent_schema;
pub mod ent_codegen;
pub mod ent_hooks;
pub mod ent_privacy;
pub mod schemas;

// Generated entities with Thrift + TAO integration
pub mod generated;

// Application architecture
pub mod error;
pub mod config;
pub mod app_state;
pub mod tao_interface;

// Services - High-level business logic
pub mod services;

// Re-exports for convenience
pub use app_state::AppState;
pub use config::Config;
pub use error::{AppError, AppResult};
