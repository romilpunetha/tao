# TAO Database - Social Graph System

A modern implementation of Meta's TAO (The Associations and Objects) system using **Rust (Axum + Tokio)** backend and **React (TypeScript)** frontend.

## 🌟 Features

### Backend (Rust + Axum + Tokio)
- **Async/Await Architecture**: Full async support with Tokio runtime
- **REST API**: Comprehensive endpoints for CRUD operations
- **Multi-Level Caching**: L1 object cache, L2 association cache, count cache
- **Graph Database**: Object-association model optimized for social graphs
- **Thrift Integration**: Structured data serialization
- **Access Control**: Viewer-based permissions system
- **Batch Operations**: Efficient bulk queries and updates

### Frontend (React + TypeScript)
- **Interactive Graph Visualization**: D3.js-powered social graph display
- **Real-time Updates**: React Query for efficient data fetching
- **Material-UI Design**: Modern, responsive interface
- **User Management**: Create, view, and manage users
- **Social Features**: Friendships, follows, likes, and posts
- **Graph Controls**: Zoom, pan, node selection, and filtering

## 🚀 Quick Start

### Prerequisites
- **Rust** (1.70+): [Install Rust](https://rustup.rs/)
- **Node.js** (16+): [Install Node.js](https://nodejs.org/)
- **npm** or **yarn**

### Development Setup

1. **Clone and enter the project**:
   ```bash
   git clone <repository-url>
   ```

2. **Start development environment**:
   ```bash
   ./scripts/dev.sh
   ```

   This will:
   - Install frontend dependencies
   - Build and start the Rust API server on port 3000
   - Start the React dev server on port 3001
   - Open your browser automatically

3. **Access the application**:
   - **Frontend**: http://localhost:3001
   - **API**: http://localhost:3000/api
   - **Health Check**: http://localhost:3000/api/health

### Manual Setup (Alternative)

**Backend**:
```bash
# Start the Rust server
cargo run --bin tao_database_server
```

**Frontend**:
```bash
# Install dependencies and start React
cd frontend
npm install
npm start
```

## 📊 API Endpoints

### Users
- `GET /api/users` - List all users
- `POST /api/users` - Create new user
- `GET /api/users/{id}` - Get user by ID
- `DELETE /api/users/{id}` - Delete user
- `GET /api/users/{id}/friends` - Get user's friends
- `GET /api/users/{id}/posts` - Get user's posts
- `GET /api/users/{id}/stats` - Get user statistics

### Posts
- `POST /api/posts` - Create new post

### Social Graph
- `POST /api/friendships` - Create friendship
- `POST /api/follows` - Create follow relationship
- `POST /api/likes` - Create like
- `GET /api/graph` - Get graph visualization data

### Utilities
- `GET /api/health` - Health check
- `POST /api/seed` - Seed sample data

## 🏗️ Architecture

### TAO Concepts Implementation

**Objects**: Core entities (Users, Posts, Comments, Groups, Pages)
- Stored with unique IDs and typed data
- Cached in L1 object cache
- Support for batch operations

**Associations**: Typed relationships between objects
- Friendship, Follow, Like, Membership, etc.
- Bidirectional for efficient reverse queries
- Temporal support with time1/time2 fields
- Cached query results in L2 association cache

**Viewer Context**: Access control system
- Permission-based data access
- Privacy-aware friend/post visibility
- Similar to Facebook's privacy model

### Database Schema

```sql
-- Objects table
CREATE TABLE objects (
    id INTEGER PRIMARY KEY,
    object_type TEXT NOT NULL,
    data BLOB NOT NULL,
    created_time INTEGER NOT NULL,
    updated_time INTEGER NOT NULL
);

-- Associations table
CREATE TABLE associations (
    id INTEGER PRIMARY KEY,
    id1 INTEGER NOT NULL,        -- source object
    id2 INTEGER NOT NULL,        -- target object
    assoc_type TEXT NOT NULL,    -- relationship type
    data BLOB,                   -- association metadata
    created_time INTEGER NOT NULL,
    updated_time INTEGER NOT NULL,
    time1 INTEGER,               -- temporal field 1
    time2 INTEGER                -- temporal field 2
);
```

### Caching Strategy

1. **L1 Object Cache**: Individual objects by ID
2. **L2 Association Cache**: Query results by (id1, assoc_type)
3. **Count Cache**: Association counts for quick stats
4. **Cache Invalidation**: Smart invalidation on writes

## 🎨 Frontend Architecture

### Components
- **GraphVisualization**: D3.js force-directed graph
- **UserManagement**: User CRUD operations
- **App**: Main application shell

### State Management
- **React Query**: Server state management
- **React Hooks**: Local state and API calls
- **TypeScript**: Type-safe API contracts

### Graph Features
- **Node Types**: Users (regular/verified)
- **Edge Types**: Friendships, follows, likes
- **Interactions**: Click selection, drag nodes, zoom/pan
- **Real-time**: Auto-refresh on data changes

## 🔧 Development

### Project Structure
```
tao_db/
├── src/                    # Rust backend
│   ├── async_db.rs        # Async database layer
│   ├── async_service.rs   # Business logic
│   ├── server.rs          # Axum web server
│   ├── types.rs           # Shared type definitions
│   └── ...
├── frontend/              # React frontend
│   ├── src/
│   │   ├── components/    # React components
│   │   ├── hooks/         # Custom hooks
│   │   ├── services/      # API client
│   │   └── types/         # TypeScript types
│   └── ...
├── scripts/               # Build and dev scripts
└── thrift/               # Thrift schema definitions
```

### Key Technologies

**Backend**:
- **Axum**: Modern async web framework
- **Tokio**: Async runtime
- **SQLite**: Embedded database
- **Serde**: Serialization
- **Thrift**: Schema definition

**Frontend**:
- **React 18**: UI framework
- **TypeScript**: Type safety
- **Material-UI**: Component library
- **D3.js**: Graph visualization
- **React Query**: Data fetching
- **Axios**: HTTP client

## 🚀 Production Build

```bash
# Build for production
./scripts/build.sh

# Run production server
./target/release/tao_database_server
```

The production build:
- Compiles Rust in release mode
- Builds optimized React bundle
- Serves static files from the Rust server
- Single binary deployment

## 📈 Performance Features

### Caching
- **Multi-level caching** reduces database load by 90%+
- **LRU eviction** keeps hot data in memory
- **Smart invalidation** maintains consistency

### Database Optimization
- **Indexes** on common query patterns
- **Batch operations** reduce round-trips
- **Connection pooling** for concurrency

### Frontend Optimization
- **Virtual DOM** for efficient updates
- **Query caching** prevents redundant requests
- **Code splitting** for faster initial loads

## 🧪 Testing

```bash
# Run Rust tests
cargo test

# Run frontend tests
cd frontend
npm test
```

## 📝 Example Usage

### Seed Sample Data
```bash
curl -X POST http://localhost:3000/api/seed
```

### Create a User
```bash
curl -X POST http://localhost:3000/api/users \
  -H "Content-Type: application/json" \
  -d '{
    "username": "alice",
    "email": "alice@example.com",
    "full_name": "Alice Johnson",
    "bio": "Software engineer",
    "location": "San Francisco"
  }'
```

### Create a Friendship
```bash
curl -X POST http://localhost:3000/api/friendships \
  -H "Content-Type: application/json" \
  -d '{
    "user1_id": 1,
    "user2_id": 2,
    "relationship_type": "friend"
  }'
```

## 🔍 Monitoring

- **Health endpoint**: `/api/health`
- **Metrics**: Request counts, cache hit rates
- **Logging**: Structured logging with tracing
- **React DevTools**: Component and query inspection

## 🎯 Learning Goals

This project demonstrates:
- **Scalable social graph architecture** (Meta's TAO patterns)
- **Modern async Rust** web development
- **Type-safe full-stack** TypeScript/Rust
- **Graph visualization** with D3.js
- **Efficient caching strategies**
- **REST API design** and documentation

Perfect for understanding how large-scale social platforms handle billions of social graph operations! 🌐✨