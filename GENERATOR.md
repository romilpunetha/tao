# Meta TAO Entity Generator

This generator creates complete TAO entities with all necessary Meta TAO patterns automatically.

## Usage

```bash
python3 scripts/generate_entity.py EntUserSchema
python3 scripts/generate_entity.py EntPostSchema  
python3 scripts/generate_entity.py EntEventSchema
```

## What Gets Generated

For `EntEventSchema`, the generator creates:

### 1. Schema Definition
- `schemas/ent_event.thrift` - Thrift schema with sensible defaults
- Namespace: `tao_db.schemas.ent_event`

### 2. Entity Implementation  
- `src/entities/ent_event.rs` - Complete Meta TAO methods:
  - `EntEvent::genNullable(ctx, id)` - Get entity by ID (nullable)
  - `EntEvent::gen(ctx, id)` - Get entity by ID (throws if not found)
  - `EntEvent::genMulti(ctx, ids)` - Batch get entities
  - `EntEvent::genAll(ctx, limit)` - Get all entities with limit
  - `EntEvent::create(ctx, entity)` - Create new entity
  - `EntEvent::update(ctx, id, entity)` - Update entity
  - `EntEvent::delete(ctx, id)` - Delete entity
  - `EntEvent::get_associations(ctx, id, assoc_type)` - Get associations
  - `EntEvent::create_many(ctx, entities)` - Batch create

### 3. Generated Thrift Code
- `src/models/ent_event.rs` - Auto-generated Thrift struct with serialization
- Full TSerializable implementation for binary storage

### 4. Framework Updates
- Updates `EntityType` enum in `src/models/mod.rs`
- Updates `build.rs` to include new schema
- Updates module exports in `src/models/mod.rs` and `src/entities/mod.rs`

## Default Field Patterns

The generator creates intelligent defaults based on entity names:

- **EntUser**: username, email, full_name
- **EntPost**: author_id, content, like_count  
- **EntEvent**: title, description, start_time
- **Generic**: name, description

All entities get: `created_time`, `updated_time`

## Build and Use

```bash
# Generate the entity
python3 scripts/generate_entity.py EntEventSchema

# Build to generate thrift code
cargo build

# Use in code
use crate::entities::{EntityContext, EntEvent};

let event = EntEvent {
    title: "Conference".to_string(),
    description: Some("Tech conference".to_string()),
    start_time: 1640995200,
    created_time: Utc::now().timestamp(),
    updated_time: None,
};

let event_id = EntEvent::create(&entity_ctx, &event).await?;
let event = EntEvent::gen_nullable(&entity_ctx, event_id).await?;
```

## Pure TAO Architecture

- No individual services (UserService, PostService, etc.)
- Unified `TaoInterface` for all HTTP operations
- Entities handle their own database operations
- True Meta TAO pattern: `EntUser::genNullable(id)` instead of `db.get_user(id)`
- Binary thrift serialization for all storage
- Full association support via TAO database

## API Endpoints

The unified TAO interface provides:

```
POST   /api/v1/tao/users          - Create user
GET    /api/v1/tao/users          - Get all users  
GET    /api/v1/tao/users/{id}     - Get user by ID

POST   /api/v1/tao/posts          - Create post
GET    /api/v1/tao/posts          - Get all posts
GET    /api/v1/tao/posts/{id}     - Get post by ID

GET    /api/v1/tao/entities/{id}  - Get any entity by ID
DELETE /api/v1/tao/entities/{id}  - Delete any entity

POST   /api/v1/tao/associations   - Create association
GET    /api/v1/tao/associations   - Query associations
DELETE /api/v1/tao/associations/{src}/{tgt}/{type} - Delete association
```

This follows Meta's actual TAO architecture where entities are self-contained and the database is a unified object/association store.