# TAO Database System - AI Context Documentation

## Project Overview

TAO (The Associations and Objects) is a Rust-based database system inspired by Facebook's TAO, designed for managing social graph data with high performance and type safety. This document provides comprehensive context for AI assistants to understand and maintain the system.

## System Architecture

### Architecture Principles
- Tao object should only be used within the ent implementation files.
- Tao should never be exposed publicly other than to Ent framework.

### Core Components

1. **TAO Core** (`src/infrastructure/tao_core.rs`)
   - Main database abstraction layer
   - Handles object storage, associations, and queries
   - Provides connection pooling and caching
   - Multi-shard support with PostgreSQL backends

2. **Entity Framework** (`src/ent_framework/`)
   - Type-safe entity definitions and operations
   - Automatic code generation for entities
   - Builder pattern for entity creation
   - Validation and serialization using Thrift

3. **Code Generation System** (`src/codegen/`)
   - Modular code generator for entities, builders, and implementations
   - Schema-driven generation from JSON definitions
   - Thrift integration for serialization
   - Domain-driven file organization

### Database Layer

- **Backend**: PostgreSQL with multi-shard support
- **Caching**: Multi-level cache system with LRU eviction
- **Serialization**: Apache Thrift for efficient data serialization
- **Connection Management**: Pooled connections with automatic failover

### Entity System

#### Entity Structure
All entities follow this pattern:
```rust
pub struct EntUser {
    pub id: i64,
    pub username: String,
    pub email: String,
    pub created_time: i64,
    // ... other fields
}
```

#### Builder Pattern
Each entity has a corresponding builder:
```rust
pub struct EntUserBuilder {
    id: Option<i64>,
    username: Option<String>,
    // ... all fields as Option<T>
}
```

#### Key Methods
- `build(id: i64)` - Creates entity with specified ID (no database interaction)
- `savex()` - Saves entity to database with auto-generated ID
- `create()` - Static method returning new builder instance

## Current State

### Generated Entities
The system currently has 6 entity types:
1. **EntUser** - User profiles with authentication data
2. **EntPost** - Social media posts with content and metadata
3. **EntComment** - Comments on posts
4. **EntGroup** - Social groups/communities
5. **EntPage** - Pages/organizations
6. **EntEvent** - Events and activities

### File Organization
```
src/
├── ent_framework/          # Core entity framework
│   ├── ent_trait.rs        # Entity trait definition
│   ├── ent_builder.rs      # Builder trait for entities
│   └── ent_schema.rs       # Schema definitions
├── infrastructure/         # Core TAO infrastructure
│   ├── tao_core.rs         # Main TAO implementation
│   ├── tao.rs              # TAO interface
│   ├── database.rs         # Database connections
│   ├── cache_layer.rs      # Caching system
│   └── global_tao.rs       # Global TAO instance
├── domains/                # Generated entity domains
│   ├── user/               # User entity domain
│   │   ├── entity.rs       # Generated entity struct
│   │   ├── builder.rs      # Generated builder
│   │   ├── ent_impl.rs     # Entity trait implementation
│   │   └── mod.rs          # Module exports
│   └── [post|comment|group|page|event]/  # Other domains
├── codegen/                # Code generation system
│   ├── mod.rs              # Main generator
│   ├── builder_generator.rs # Builder code generation
│   ├── ent_generator.rs    # Entity code generation
│   └── utils.rs            # Generation utilities
└── bin/                    # Executable binaries
    ├── codegen.rs          # Code generation CLI
    ├── entc.rs             # Entity compiler
    └── tao_web_server.rs   # REST API server
```

## Key Design Principles

### Type Safety
- All entities implement the `Entity` trait for consistent behavior
- Builders implement `EntBuilder` trait for uniform creation patterns
- Compile-time verification of entity types and field requirements

### Performance
- Multi-level caching with automatic invalidation
- Connection pooling for database efficiency
- Batch operations for loading multiple entities
- Efficient serialization with Thrift

### Code Generation
- Schema-driven development with JSON entity definitions
- Automatic generation of boilerplate code
- Consistent patterns across all entities
- Regeneration preserves manual customizations in designated areas

### Database Design
- Object-centric storage with TAO principles
- Association management for relationships
- Sharding support for horizontal scaling
- ACID compliance with PostgreSQL

## API Endpoints

The web server (`tao_web_server.rs`) provides REST endpoints:
- `POST /api/users` - Create new user
- `GET /api/users/{id}` - Get user by ID
- `GET /api/users` - List all users
- `POST /api/seed` - Seed database with sample data
- Health check endpoints

## Development Workflow

### Adding New Entities
1. Define entity schema in `schemas/` directory
2. Run `cargo run --bin codegen` to generate code
3. Implement custom business logic in `ent_impl.rs` files
4. Add API endpoints in web server if needed

### Builder Usage Patterns
```rust
// Create entity without saving (for testing/validation)
let user = EntUser::create()
    .username("john_doe".to_string())
    .email("john@example.com".to_string())
    .build(123)?;

// Create and save entity to database
let user = EntUser::create()
    .username("john_doe".to_string())
    .email("john@example.com".to_string())
    .savex().await?;
```

### Recent Updates
- Added `build()` method to builders for creating entities without database interaction
- Enhanced builder pattern with ID field support
- Implemented `EntBuilder` trait for consistent builder interface
- Improved code generation with trait implementations

## Testing and Validation

### Entity Validation
- All entities implement `validate()` method
- Field-level validation with custom rules
- Required field checking in builders
- Type-safe field access patterns

### Database Testing
- Multi-shard configuration for testing
- Sample data seeding functionality
- Health check endpoints for monitoring
- Error handling with structured error types

## Configuration

### Database Configuration
- PostgreSQL connection strings for multiple shards
- Connection pool settings
- Cache configuration parameters
- Logging and monitoring setup

### Environment Variables
- Database URLs for different environments
- Cache size and eviction policies
- Web server port and host configuration

## Troubleshooting Guide

### Common Issues
1. **Missing ID field in builders**: Run `cargo run --bin codegen` to regenerate
2. **Compilation errors after schema changes**: Clean and rebuild project
3. **Database connection failures**: Check PostgreSQL service and connection strings
4. **Cache inconsistencies**: Clear cache or restart service

### Debugging Tips
- Enable debug logging for TAO operations
- Use `cargo check` for quick compilation validation
- Monitor database connections and query performance
- Check Thrift serialization for data integrity

## AI Assistant Guidelines

### When Working on This Project
1. **Always preserve existing patterns**: Follow established conventions for entities, builders, and traits
2. **Use code generation**: Don't manually create entity files; use the codegen system
3. **Maintain type safety**: Ensure all changes preserve compile-time guarantees
4. **Test thoroughly**: Verify both builder patterns (`build()` and `savex()`) work correctly
5. **Update documentation**: Keep this file current with any architectural changes

### Code Modification Principles
- Prefer editing generated code through the code generator, not manually
- If the code is generated by codegen, it'll have the string `// Generated by TAO Ent Framework` and if it is generated by thrift it'll have the string `// Autogenerated by Thrift Compiler`. Do not edit such files directly.
- Before generating the code using codegen, remove all the existing generated code from the domain folder and clear `mod.rs` so that we don't have compilation errors. Then generate the code using the codegen and then add the generated entities to the `mod.rs` file.
- When adding new functionality, follow the existing trait-based patterns
- If there is a major change to the system that requires not adhering to backward compatibility, ask first. During initial development phase we don't care about backward compatibility and can tear down the system and rebuild from scratch so it is ok to delete stuff and replace it. This is true for DB schema also. In the initial phase of development it is ok to delete the database and recreate.
- Ensure all database operations are properly error-handled

### Performance Considerations
- Consider caching implications for new features
- Use batch operations for multiple entity operations
- Monitor database query patterns and optimize as needed
- Preserve connection pooling and transaction safety

This document should be updated whenever significant architectural changes are made to the system.