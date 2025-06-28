# 🔗 TAO Graph Database

A modern, high-performance social graph database inspired by Meta's TAO (The Associations and Objects), implemented in Rust with a complete web visualization frontend.

## ✨ Features

### 🏗️ Architecture
- **Decorator Pattern**: Pluggable production features (WAL, caching, metrics, circuit breaker)
- **Shard-Aware**: Query router handles efficient shard selection and routing
- **Write-Ahead Log**: Durable transaction logging for failure recovery
- **Multi-Tier Caching**: L1 (local) and L2 (distributed) caching layers
- **Circuit Breaker**: Fault tolerance for external dependencies
- **Metrics Collection**: Comprehensive monitoring and observability

### 🎯 Core Capabilities
- **Type-Safe Entity Framework**: Generated entities with compile-time safety
- **Social Graph Operations**: Users, relationships, posts, comments, likes
- **REST API**: Complete HTTP API for all graph operations
- **Real-Time Visualization**: Interactive D3.js-powered graph visualization
- **Sample Data Generation**: Realistic social network data for testing

### 🔧 Technical Stack
- **Backend**: Rust with Tokio async runtime
- **Database**: PostgreSQL with connection pooling
- **Web Framework**: Axum with CORS support
- **Serialization**: Thrift for efficient data serialization
- **Frontend**: Vanilla JavaScript with D3.js visualization
- **Build System**: Cargo with optimized release builds

## 🚀 Quick Start

### Prerequisites
- Rust 1.70+ (with Cargo)
- PostgreSQL 12+
- Node.js (for frontend dependencies, optional)

### Installation

1. **Clone the repository**
   ```bash
   git clone <repository-url>
   cd tao
   ```

2. **Start PostgreSQL**
   ```bash
   # Using Homebrew (macOS)
   brew services start postgresql
   
   # Using Docker
   docker run -d --name postgres -p 5432:5432 \
     -e POSTGRES_PASSWORD=password postgres:14
   ```

3. **Run the complete demo**
   ```bash
   ./run_demo.sh
   ```

   This script will:
   - Build the TAO system
   - Create the database
   - Generate sample social graph data
   - Start the web server at `http://localhost:3000`

### Manual Setup

1. **Build the system**
   ```bash
   cargo build --release
   ```

2. **Set up database**
   ```bash
   export DATABASE_URL="postgresql://postgres:password@localhost:5432/tao_graph"
   createdb tao_graph
   ```

3. **Generate sample data**
   ```bash
   ./target/release/generate_sample_data
   ```

4. **Start the web server**
   ```bash
   ./target/release/tao_web_server
   ```

5. **Open your browser**
   Navigate to `http://localhost:3000` to see the graph visualization

## 📊 Sample Data

The system generates realistic social network data including:

- **10 Users**: Software engineers with profiles and bios
- **50+ Relationships**: Friendships, follows, colleagues, mentorships
- **8 Posts**: Social media posts with hashtags and metadata
- **Comments & Likes**: Engagement data for realistic interaction patterns

### User Profiles
- Alice Johnson (Software Engineer)
- Bob Smith (Product Manager)
- Carol Wilson (UX Designer)
- David Brown (Data Scientist)
- Eve Davis (DevOps Engineer)
- And 5 more...

## 🌐 API Reference

### Users
```bash
# Create user
POST /api/users
{
  "name": "John Doe",
  "email": "john@example.com",
  "bio": "Software Engineer"
}

# Get user
GET /api/users/{id}
```

### Relationships
```bash
# Create relationship
POST /api/relationships
{
  "from_user_id": 1,
  "to_user_id": 2,
  "relationship_type": "friendship"
}
```

### Graph Data
```bash
# Get complete graph
GET /api/graph?limit=50
```

## 🎨 Frontend Features

### Interactive Graph Visualization
- **Drag & Drop**: Move nodes around to explore connections
- **Color-Coded Nodes**: Each user has a unique color
- **Relationship Links**: Visual representation of connections
- **User Information**: Click nodes to see user details
- **Real-Time Updates**: Graph updates when new data is added

### Management Interface
- **Create Users**: Add new users with name, email, and bio
- **Create Relationships**: Build connections between users
- **Sample Data Generator**: One-click generation of realistic data
- **Statistics Dashboard**: View user count, relationships, and metrics

## 🏗️ Architecture Deep Dive

### Decorator Pattern Implementation
```rust
// Create TAO with production features
let decorated_tao = TaoBuilder::new(core_tao)
    .with_circuit_breaker(5, Duration::from_secs(30))
    .with_wal(wal_instance)  
    .with_metrics()
    .with_caching(object_ttl, association_ttl)
    .build();

let tao = Tao::new(decorated_tao);
```

### Write-Ahead Log (WAL)
- **Operation Logging**: All write operations recorded for replay
- **No Shard Calculation**: WAL focuses on operation logging only
- **Query Router Responsibility**: Shard routing handled by tao_core
- **Failure Recovery**: Failed operations can be replayed from WAL

### Query Routing
- **Shard Selection**: Automatic routing based on object IDs
- **Per-Shard Transactions**: No distributed transactions
- **Load Balancing**: Even distribution across shards
- **Health Monitoring**: Automatic shard health checking

## 🔧 Development

### Project Structure
```
src/
├── bin/                        # Executables
│   ├── tao_web_server.rs      # Web server with REST API
│   └── generate_sample_data.rs # Sample data generator
├── infrastructure/            # Core TAO infrastructure
│   ├── tao.rs                # Main TAO interface
│   ├── tao_core.rs           # Core operations
│   ├── tao_decorators.rs     # Decorator implementations
│   ├── write_ahead_log.rs    # WAL implementation
│   ├── query_router.rs       # Query routing
│   └── ...
├── ent_framework/            # Entity framework
└── static/                   # Frontend assets
    └── index.html           # Graph visualization UI
```

### Key Components

1. **TaoCore**: Core graph operations and shard management
2. **TaoDecorators**: Pluggable production features
3. **QueryRouter**: Intelligent shard selection and routing
4. **WriteAheadLog**: Durable transaction logging
5. **EntityFramework**: Type-safe entity operations

### Building & Testing
```bash
# Build all components
cargo build --release

# Run tests
cargo test

# Check code quality
cargo clippy

# Format code
cargo fmt
```

## 🎯 Use Cases

### Social Networks
- User profiles and connections
- Content sharing and engagement
- Real-time activity feeds
- Recommendation systems

### Professional Networks
- Career connections and mentorships
- Skill sharing and endorsements
- Company organizational charts
- Project collaboration tracking

### Content Platforms
- Author-content relationships
- User engagement metrics
- Comment threading systems
- Content recommendation graphs

## 🚀 Performance

### Optimizations
- **Async/Await**: Non-blocking I/O throughout the stack
- **Connection Pooling**: Efficient database connection management
- **Batch Operations**: Bulk loading for improved throughput
- **Caching Strategy**: Multi-tier caching reduces database load
- **Compiled Binaries**: Rust's zero-cost abstractions for performance

### Scalability Features
- **Horizontal Sharding**: Distribute data across multiple databases
- **Read Replicas**: Scale read operations independently
- **Circuit Breakers**: Prevent cascade failures
- **Metrics Collection**: Monitor performance bottlenecks

## 🛡️ Security

- **Input Validation**: Comprehensive request validation
- **SQL Injection Protection**: SQLx compile-time query checking
- **CORS Configuration**: Secure cross-origin requests
- **Error Handling**: Safe error propagation without information leakage

## 📈 Monitoring

### Built-in Metrics
- Request latency and throughput
- Database connection health
- Cache hit/miss ratios
- Error rates and types
- Business metrics (user growth, engagement)

### Health Checks
- Database connectivity
- Shard availability
- WAL status
- Cache performance

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Add comprehensive tests
4. Update documentation
5. Submit a pull request

## 📄 License

MIT License - see LICENSE file for details

## 🙏 Acknowledgments

- Inspired by Meta's TAO architecture
- Built with the Rust ecosystem's excellent async libraries
- D3.js for beautiful graph visualizations

---

**Ready to explore social graphs at scale? Start with `./run_demo.sh` and see TAO in action!** 🚀